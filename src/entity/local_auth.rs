use sea_orm::entity::prelude::*;
use sea_orm::DerivePrimaryKey;
use sea_orm::{ActiveModelBehavior, DeriveEntityModel, EnumIter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "local_auth")]
pub struct Model {
    #[sea_orm(primary_key, column_name = "id")]
    pub id: u32,
    #[sea_orm(column_name = "profile_id")]
    pub profile_id: u32,
    #[sea_orm(column_name = "key_path_local")]
    pub key_path_local: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
