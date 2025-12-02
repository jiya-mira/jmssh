use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    // --- 通用错误 ---
    #[error("database error: {0}")]
    Db(#[from] sea_orm::DbErr),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    // --- 业务错误（profile 相关） ---
    #[error("profile not found: {0}")]
    ProfileNotFound(String),

    #[error("profile already exists: {0}")]
    ProfileAlreadyExists(String),

    #[error("nothing to update for profile: {0}")]
    ProfileNothingToUpdate(String),

    #[error("route target profile not found for label: {0}")]
    ProfileRouteTargetNotFound(String),
    #[error("invalid auth mode: {0}")]
    InvalidAuthMode(String),

    #[error("password store error: {0}")]
    PasswordStoreError(String),

    #[error("io error: {0}")]
    IoError(String),
    
    #[error("internal server error: {0}")]
    InternalError(String),
}

pub type AppResult<T> = anyhow::Result<T, AppError>;
