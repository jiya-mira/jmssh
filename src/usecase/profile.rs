use crate::app::AppContext;
use crate::entity;
use crate::entity::profiles;
use crate::entity::profiles::AuthMode;
use crate::error::{AppError, AppResult};
use crate::usecase::{EditProfileInput, ProfileView};
use itertools::Itertools;
use sea_orm::{ActiveModelTrait, QueryFilter, QueryOrder, TransactionTrait};
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, Set};
use std::collections::HashMap;

fn to_view(model: profiles::Model) -> ProfileView {
    ProfileView {
        id: model.id,
        label: model.label.unwrap_or_default(),
        host: model.hostname,
        user: model.username,
        port: model.port.unwrap_or(22),
        mode: model.auth_mode.as_str().to_string(),
        tags: model.tags,
        note: model.note,
    }
}

async fn replace_jumps_for_profile<C>(db: &C, profile_id: u32, jumps: &[String]) -> AppResult<()>
where
    C: ConnectionTrait,
{
    if jumps.is_empty() {
        // 业务约定：空数组 = 不动 jump 链，由外层控制
        return Ok(());
    }

    // 1) 查出所有目标 profile（按 label 匹配）
    let via_profiles = profiles::Entity::find()
        .filter(profiles::Column::Label.is_in(jumps.to_vec()))
        .all(db)
        .await?;

    let label_to_id = via_profiles
        .into_iter()
        .filter_map(|p| p.label.map(|label| (label, p.id)))
        .collect::<HashMap<_, _>>();

    let missing = jumps
        .iter()
        .filter(|lbl| !label_to_id.contains_key(*lbl))
        .cloned()
        .collect_vec();

    if !missing.is_empty() {
        // 严格校验：只要有一个不存在就报错
        return Err(AppError::ProfileRouteTargetNotFound(missing.join(",")));
    }

    entity::routes::Entity::delete_many()
        .filter(entity::routes::Column::ProfileId.eq(profile_id))
        .exec(db)
        .await?;

    // 5) 重建 routes（保留传入顺序）
    for (idx, lbl) in jumps.iter().enumerate() {
        let via_id = label_to_id[lbl];

        let active = entity::routes::ActiveModel {
            profile_id: Set(profile_id),
            seq: Set(idx as u32),
            via_profile_id: Set(via_id),
            ..Default::default()
        };

        active.insert(db).await?;
    }

    Ok(())
}

pub async fn add_profile(ctx: &AppContext, input: EditProfileInput) -> AppResult<ProfileView> {
    let host = input.host.unwrap_or_else(|| "127.0.0.1".to_string());
    let user = input.user.unwrap_or_else(|| "root".to_string());
    let port = input.port.unwrap_or(22);

    let txn = ctx.db.begin().await?;

    if profiles::Entity::find()
        .filter(profiles::Column::Label.eq(input.label.clone()))
        .one(&txn)
        .await?
        .is_some()
    {
        return Err(AppError::ProfileAlreadyExists(input.label.clone()));
    }

    let auth_mode = AuthMode::from_str(input.mode.as_deref())?;

    let active = profiles::ActiveModel {
        label: Set(Some(input.label.clone())),
        hostname: Set(host.clone()),
        username: Set(user.clone()),
        port: Set(Some(port)),
        auth_mode: Set(auth_mode),
        tags: Set(input.tags.clone()),
        note: Set(input.notes.clone()),
        ..Default::default()
    };

    let model = active.insert(&txn).await?;

    if !input.jumps.is_empty() {
        replace_jumps_for_profile(&txn, model.id, &input.jumps).await?;
    }

    txn.commit().await?;

    Ok(to_view(model))
}

pub async fn set_profile(ctx: &AppContext, input: EditProfileInput) -> AppResult<ProfileView> {
    let label = input.label.clone();

    let txn = ctx.db.begin().await?;

    let model = profiles::Entity::find()
        .filter(profiles::Column::Label.eq(label.clone()))
        .one(&txn)
        .await?
        .ok_or(AppError::ProfileNotFound(label))?;

    let mut active: profiles::ActiveModel = model.into();

    if let Some(host) = input.host {
        active.hostname = Set(host);
    }
    if let Some(user) = input.user {
        active.username = Set(user);
    }
    if let Some(port) = input.port {
        active.port = Set(Some(port)); // 你 schema 里是 Option<u16> 的话正好
    }
    if let Some(tags) = input.tags {
        active.tags = Set(Some(tags));
    }
    if let Some(note) = input.notes {
        active.note = Set(Some(note));
    }
    if let Some(mode_str) = input.mode {
        active.auth_mode = Set(AuthMode::from_str(Some(mode_str.as_str()))?);
    }

    let model = active.update(&txn).await?;

    if !input.jumps.is_empty() {
        replace_jumps_for_profile(&txn, model.id, &input.jumps).await?;
    }

    txn.commit().await?;

    Ok(to_view(model))
}

pub async fn list_profiles(ctx: &AppContext) -> AppResult<Vec<ProfileView>> {
    let rows = profiles::Entity::find()
        .order_by_asc(profiles::Column::Label)
        .all(&ctx.db)
        .await?;

    Ok(rows.into_iter().map(to_view).collect_vec())
}
#[allow(dead_code)]
pub async fn get_profile_by_label(ctx: &AppContext, label: String) -> AppResult<ProfileView> {
    let model = profiles::Entity::find()
        .filter(profiles::Column::Label.eq(label.clone()))
        .one(&ctx.db)
        .await?
        .ok_or(AppError::ProfileNotFound(label))?;

    Ok(to_view(model))
}

pub async fn get_profile_detail_by_label(
    ctx: &AppContext,
    label: String,
) -> AppResult<(ProfileView, Vec<ProfileView>)> {
    // 1) 先查 profile
    let model = profiles::Entity::find()
        .filter(profiles::Column::Label.eq(label.clone()))
        .one(&ctx.db)
        .await?
        .ok_or(AppError::ProfileNotFound(label))?;

    // 2) 再查 routes（按 seq 排）
    let routes = entity::routes::Entity::find()
        .filter(entity::routes::Column::ProfileId.eq(model.id))
        .order_by_asc(entity::routes::Column::Seq)
        .all(&ctx.db)
        .await?;

    // 3) 批量查 via profile，再按 route 顺序组装
    let via_ids = routes.iter().map(|r| r.via_profile_id).collect_vec();

    let via_models = profiles::Entity::find()
        .filter(profiles::Column::Id.is_in(via_ids.clone()))
        .all(&ctx.db)
        .await?;

    let id2view = via_models
        .into_iter()
        .map(|m| (m.id, to_view(m)))
        .collect::<HashMap<_, _>>();

    let jumps = routes
        .iter()
        .filter_map(|r| id2view.get(&r.via_profile_id).cloned())
        .collect_vec();

    Ok((to_view(model), jumps))
}

pub async fn delete_profile_by_label(ctx: &AppContext, label: String) -> AppResult<()> {
    let txn = ctx.db.begin().await?;

    let model = profiles::Entity::find()
        .filter(profiles::Column::Label.eq(label.clone()))
        .one(&txn)
        .await?
        .ok_or(AppError::ProfileNotFound(label))?;

    // 先删 routes，再删 profile（如果没有外键约束，这两步顺序也无所谓）
    entity::routes::Entity::delete_many()
        .filter(entity::routes::Column::ProfileId.eq(model.id))
        .exec(&txn)
        .await?;

    profiles::Entity::delete_by_id(model.id).exec(&txn).await?;

    txn.commit().await?;

    Ok(())
}
