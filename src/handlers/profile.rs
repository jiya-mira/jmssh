use crate::app::AppContext;
use crate::cli::{EditProfileArgs, ProfileArgs, ProfileCommand};
use crate::error::AppResult;
use crate::usecase;

pub async fn handle_profile(ctx: &AppContext, args: ProfileArgs) -> AppResult<()> {
    match args.cmd {
        ProfileCommand::Add(args) => profile_add(ctx, args).await?,
        ProfileCommand::Set(args) => profile_set(ctx, args).await?,
        _ => profile_unknown(ctx).await?,
    };

    Ok(())
}

async fn profile_add(ctx: &AppContext, args: EditProfileArgs) -> AppResult<()> {
    usecase::profile::add_profile(
        ctx,
        usecase::EditProfileInput {
            label: args.label,
            host: args.host,
            user: args.user,
            port: args.port,
            mode: args.mode,
            tags: args.tags,
            notes: args.note,
            jumps: args.jumps,
        },
    )
    .await?;

    Ok(())
}

async fn profile_set(ctx: &AppContext, args: EditProfileArgs) -> AppResult<()> {
    usecase::profile::set_profile(
        ctx,
        usecase::EditProfileInput {
            label: args.label,
            host: args.host,
            user: args.user,
            port: args.port,
            mode: args.mode,
            tags: args.tags,
            notes: args.note,
            jumps: args.jumps,
        },
    )
    .await?;

    Ok(())
}

async fn profile_unknown(ctx: &AppContext) -> AppResult<()> {
    Ok(())
}
