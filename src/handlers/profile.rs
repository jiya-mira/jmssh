use crate::app::AppContext;
use crate::cli::{EditProfileArgs, ProfileArgs, ProfileCommand};
use crate::error::AppResult;
use crate::term::{c_accent, log_info, log_warn};
use crate::usecase;

pub async fn handle_profile(ctx: &AppContext, args: ProfileArgs) -> AppResult<()> {
    match args.cmd {
        ProfileCommand::Add(args) => profile_add(ctx, args).await?,
        ProfileCommand::Set(args) => profile_set(ctx, args).await?,
        _ => profile_not_implemented(ctx).await?,
    };

    Ok(())
}

async fn profile_add(ctx: &AppContext, args: EditProfileArgs) -> AppResult<()> {
    let view = usecase::profile::add_profile(
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
    log_info(format!(
        "profile {} created ({}@{}:{})",
        c_accent(&view.label),
        c_accent(&view.user),
        c_accent(&view.host),
        c_accent(&view.port.to_string()),
    ));
    Ok(())
}

async fn profile_set(ctx: &AppContext, args: EditProfileArgs) -> AppResult<()> {
    let view = usecase::profile::set_profile(
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

    log_info(format!("profile {} updated", c_accent(&view.label),));

    Ok(())
}

async fn profile_not_implemented(_ctx: &AppContext) -> AppResult<()> {
    log_warn("this profile subcommand is not implemented yet");
    Ok(())
}
