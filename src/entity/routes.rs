use sea_orm::entity::prelude::*;
use sea_orm::{ActiveModelBehavior, DeriveEntityModel, EnumIter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, DeriveEntityModel)]
#[sea_orm(table_name = "routes")]
pub struct Model {
    #[sea_orm(primary_key, column_name = "id")]
    pub id: u32,
    #[sea_orm(column_name = "profile_id")]
    pub profile_id: u32,
    #[sea_orm(column_name = "seq")]
    pub seq: u32,
    #[sea_orm(column_name = "via_profile_id")]
    pub via_profile_id: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
