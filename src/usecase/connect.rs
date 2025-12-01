use crate::app::AppContext;
use crate::entity;
use crate::entity::{profiles, routes};
use crate::error::{AppError, AppResult};
use crate::usecase::{ConnectHop, ConnectInput, ConnectPlan};
use itertools::Itertools;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::{ColumnTrait, QueryOrder};
use std::collections::HashMap;

pub async fn build_connect_plan(ctx: &AppContext, input: ConnectInput) -> AppResult<ConnectPlan> {
    // 1) 解析主 profile：优先 id，其次 label
    let base_profile = if let Some(id) = input.id {
        profiles::Entity::find_by_id(id)
            .one(&ctx.db)
            .await?
            .ok_or(AppError::ProfileNotFound(format!("#{id}")))? // 复用现有错误
    } else {
        profiles::Entity::find()
            .filter(profiles::Column::Label.eq(input.target.clone()))
            .one(&ctx.db)
            .await?
            .ok_or(AppError::ProfileNotFound(input.target.clone()))?
    };

    let base_id = base_profile.id;

    // 2) 查出这条 profile 的 routes（按 seq 排序）
    let route_rows = routes::Entity::find()
        .filter(routes::Column::ProfileId.eq(base_id))
        .order_by_asc(routes::Column::Seq)
        .all(&ctx.db)
        .await?;

    // route 里的 via_profile_id 顺序就是 jump 链
    let via_ids = route_rows.iter().map(|r| r.via_profile_id).collect_vec();

    // 3) 需要涉及的全部 profile id = 所有 jump + 自己
    let all_profile_ids = [via_ids.clone(), vec![base_id]].concat();

    // 4) 一次性查出所有 profile
    let all_profiles = profiles::Entity::find()
        .filter(profiles::Column::Id.is_in(all_profile_ids.clone()))
        .all(&ctx.db)
        .await?;

    let profile_map = all_profiles
        .into_iter()
        .map(|p| (p.id, p))
        .collect::<HashMap<_, _>>();

    // 5) 为这些 profile 查 local_auth（key_path）
    let local_auth_rows = entity::local_auth::Entity::find()
        .filter(entity::local_auth::Column::ProfileId.is_in(all_profile_ids.clone()))
        .all(&ctx.db)
        .await?;

    // let mut key_map: HashMap<u32, Option<String>> = HashMap::new();
    // for row in local_auth_rows {
    //     key_map.insert(row.profile_id, row.key_path_local);
    // }

    let key_map = local_auth_rows
        .into_iter()
        .map(|r| (r.profile_id, r.key_path_local))
        .collect::<HashMap<_, _>>();

    // 6) 按顺序组装 hops：先 jumps，再最终目标
    let hops = via_ids
        .into_iter()
        // 6.1 jumps
        .map(|via_id| {
            let p = profile_map
                .get(&via_id)
                .ok_or_else(|| AppError::ProfileRouteTargetNotFound(format!("#{via_id}")))?;

            Ok::<_, AppError>(build_connect_hop(
                p,
                key_map.get(&via_id).cloned().unwrap_or(None),
            ))
        })
        // 6.2 最终目标
        .chain(std::iter::once(Ok(build_connect_hop(
            &base_profile,
            key_map.get(&base_id).cloned().unwrap_or(None),
        ))))
        .collect::<Result<Vec<_>, AppError>>()?; // 这里再用 ? 往外抛 AppError

    Ok(ConnectPlan { hops })
}

fn build_connect_hop(p: &profiles::Model, key_path_local: Option<String>) -> ConnectHop {
    ConnectHop {
        id: p.id,
        label: p.label.clone().unwrap_or_else(|| format!("#{}", p.id)),
        host: p.hostname.clone(),
        user: p.username.clone(),
        port: p.port.unwrap_or(22),
        auth_mode: p.auth_mode.clone(),
        key_path_local,
    }
}
