use crate::app::AppContext;
use crate::entity::profiles;
use crate::error::{AppError, AppResult};
use sea_orm::ColumnTrait;
use sea_orm::{EntityTrait, QueryFilter};

async fn find_profile_id_by_label(ctx: &AppContext, label: String) -> AppResult<profiles::Model> {
    let model = profiles::Entity::find()
        .filter(profiles::Column::Label.eq(label.clone()))
        .one(&ctx.db)
        .await?
        .ok_or_else(|| AppError::ProfileNotFound(label.to_string()))?;

    Ok(model)
}

pub async fn set_profile_password_by_label(
    ctx: &AppContext,
    label: String,
    password: Option<String>,
) -> AppResult<()> {
    let model = find_profile_id_by_label(&ctx, label.clone()).await?;
    ctx.password_store
        .set_profile_password(model.id, password)?;
    Ok(())
}

pub async fn get_profile_password_by_label(
    ctx: &AppContext,
    label: String,
) -> AppResult<Option<String>> {
    let model = find_profile_id_by_label(&ctx, label.clone()).await?;
    ctx.password_store.get_profile_password(model.id)
}

pub async fn clear_profile_password_by_label(ctx: &AppContext, label: String) -> AppResult<()> {
    let model = find_profile_id_by_label(&ctx, label.clone()).await?;
    ctx.password_store.set_profile_password(model.id, None)?;
    Ok(())
}
