#[cfg(unix)]
pub mod connect;

#[cfg(not(unix))]
pub mod connect {
    use crate::app::AppContext;
    use crate::error::AppResult;
    use crate::term::{c_accent, log_warn};
    use crate::usecase::ProfileView;

    pub async fn pick_profile_for_connect(_ctx: &AppContext) -> AppResult<Option<ProfileView>> {
        log_warn(c_accent(
            "interactive picker is not supported on this platform yet.",
        ));
        Ok(None)
    }
}
