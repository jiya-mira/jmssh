use crate::error::{AppError, AppResult};
use keyring::Entry;

pub trait PasswordStore: Send + Sync {
    fn set_profile_password(&self, profile_id: u32, password: Option<String>) -> AppResult<()>;
    fn get_profile_password(&self, profile_id: u32) -> AppResult<Option<String>>;
}

pub struct NoopPasswordStore;

impl PasswordStore for NoopPasswordStore {
    fn set_profile_password(&self, _profile_id: u32, _password: Option<String>) -> AppResult<()> {
        Ok(())
    }

    fn get_profile_password(&self, _profile_id: u32) -> AppResult<Option<String>> {
        Ok(None)
    }
}

#[derive(Clone)]
pub struct OsPasswordStore {
    service: String,
}

impl OsPasswordStore {
    pub fn new<S: Into<String>>(service: S) -> Self {
        Self {
            service: service.into(),
        }
    }

    fn entry_for_profile(&self, profile_id: u32) -> AppResult<Entry> {
        let user = format!("profile:{profile_id}");
        Entry::new(&self.service, &user).map_err(|e| {
            AppError::PasswordStoreError(format!(
                "failed to create keyring entry for profile #{profile_id}: {e}"
            ))
        })
    }
}

impl PasswordStore for OsPasswordStore {
    fn set_profile_password(&self, profile_id: u32, password: Option<String>) -> AppResult<()> {
        let entry = self.entry_for_profile(profile_id)?;
        match password {
            Some(pwd) => {
                entry.set_password(&pwd).map_err(|e| {
                    AppError::PasswordStoreError(format!(
                        "failed to set password for profile #{profile_id}: {e}"
                    ))
                })?;
            }
            None => match entry.delete_credential() {
                Ok(()) => {}
                Err(keyring::Error::NoEntry) => {}
                Err(e) => {
                    return Err(AppError::PasswordStoreError(format!(
                        "failed to delete password for profile #{profile_id}: {e}"
                    )));
                }
            },
        }

        Ok(())
    }

    fn get_profile_password(&self, profile_id: u32) -> AppResult<Option<String>> {
        let entry = self.entry_for_profile(profile_id)?;

        match entry.get_password() {
            Ok(pwd) => Ok(Some(pwd)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(AppError::PasswordStoreError(format!(
                "failed to get password for profile #{profile_id}: {e}"
            ))),
        }
    }
}
