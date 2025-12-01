use crate::infra::password_store::{NoopPasswordStore, OsPasswordStore, PasswordStore};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

pub struct AppContext {
    pub db: DatabaseConnection,
    pub password_store: Arc<dyn PasswordStore>,
}

impl AppContext {
    pub fn new(db: DatabaseConnection) -> Self {
        let store: Arc<dyn PasswordStore> = if cfg!(any(target_os = "macos", target_os = "windows"))
        {
            Arc::new(OsPasswordStore::new("com.jiyamira.jmssh"))
        } else {
            Arc::new(NoopPasswordStore)
        };

        Self {
            db,
            password_store: store,
        }
    }
}
