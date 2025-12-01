use crate::error::{AppError, AppResult};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelBehavior, DeriveActiveEnum, DeriveEntityModel, DeriveRelation, EnumIter};
use serde::{Deserialize, Serialize};

#[derive(
    Debug, Serialize, Deserialize, PartialEq, Clone, Eq, EnumIter, DeriveActiveEnum, DeriveDisplay,
)]
#[sea_orm(rs_type = "u8", db_type = "TinyUnsigned")]
pub enum AuthMode {
    #[sea_orm(num_value = 0)]
    Agent,
    #[sea_orm(num_value = 1)]
    Password,
    #[sea_orm(num_value = 2)]
    Key,
}

impl AuthMode {
    pub fn from_str(s: Option<&str>) -> AppResult<Self> {
        let s = s.unwrap_or_default().to_ascii_lowercase();
        match s.as_str() {
            "" | "auto" | "agent" => Ok(Self::Agent),
            "password" => Ok(Self::Password),
            "key" => Ok(Self::Key),
            other => Err(AppError::InvalidAuthMode(other.to_string())),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AuthMode::Agent => "agent",
            AuthMode::Password => "password",
            AuthMode::Key => "key",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "profiles")]
pub struct Model {
    #[sea_orm(primary_key, column_name = "id")]
    pub id: u32,
    #[sea_orm(column_name = "label")]
    pub label: Option<String>,
    #[sea_orm(column_name = "hostname")]
    pub hostname: String,
    #[sea_orm(column_name = "username")]
    pub username: String,
    #[sea_orm(column_name = "port")]
    pub port: Option<u16>,
    #[sea_orm(column_name = "auth_mode")]
    pub auth_mode: AuthMode,
    #[sea_orm(column_name = "tags")]
    pub tags: Option<String>,
    #[sea_orm(column_name = "note")]
    pub note: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
