use crate::app::AppContext;
use crate::cli::{EditProfileArgs, ProfileArgs, ProfileCommand, RmArgs, ShowArgs};
use crate::error::{AppError, AppResult};
use crate::term::{c_accent, log_error, log_info, log_warn};
use crate::usecase;
use itertools::Itertools;
use std::io;
use std::io::Write;
use tabwriter::TabWriter;

pub async fn handle_profile(ctx: &AppContext, args: ProfileArgs) -> AppResult<()> {
    match args.cmd {
        ProfileCommand::Add(args) => profile_add(ctx, args).await?,
        ProfileCommand::Set(args) => profile_set(ctx, args).await?,
        ProfileCommand::List => profile_list(ctx).await?,
        ProfileCommand::Rm(args) => profile_rm(ctx, args).await?,
        ProfileCommand::Show(args) => profile_show(ctx, args).await?,
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

async fn profile_list(ctx: &AppContext) -> AppResult<()> {
    let profiles = usecase::profile::list_profiles(ctx).await?;

    if profiles.is_empty() {
        log_info("no profiles found");
        return Ok(());
    }

    let mut tw = TabWriter::new(io::stdout());

    // 表头：label + 目标 + mode
    writeln!(&mut tw, "LABEL\tDEST\tMODE")?;

    profiles.into_iter().for_each(|p| {
        writeln!(
            &mut tw,
            "{}\t{}@{}:{}\t{}",
            p.label, p.user, p.host, p.port, p.mode,
        )
        .unwrap_or_default();
    });

    tw.flush()?;
    Ok(())
}

async fn profile_show(ctx: &AppContext, args: ShowArgs) -> AppResult<()> {
    let (base, jumps) = usecase::profile::get_profile_detail_by_label(ctx, args.label).await?;

    let mut tw = TabWriter::new(io::stdout());

    writeln!(&mut tw, "FIELD\tVALUE")?;
    writeln!(&mut tw, "label\t{}", base.label)?;
    writeln!(&mut tw, "host\t{}", base.host)?;
    writeln!(&mut tw, "user\t{}", base.user)?;
    writeln!(&mut tw, "port\t{}", base.port)?;
    writeln!(&mut tw, "mode\t{}", base.mode)?;

    if !jumps.is_empty() {
        let jumps_str = jumps.iter().map(|j| j.label.as_str()).join(" -> ");
        writeln!(&mut tw, "jumps\t{jumps_str}")?;
    }

    if let Some(tags) = base.tags {
        writeln!(&mut tw, "tags\t{tags}")?;
    }
    if let Some(note) = base.note {
        writeln!(&mut tw, "note\t{note}")?;
    }

    tw.flush()?;
    Ok(())
}

async fn profile_rm(ctx: &AppContext, args: RmArgs) -> AppResult<()> {
    let label = args.label;
    match usecase::profile::delete_profile_by_label(ctx, label.clone()).await {
        Ok(()) => {
            log_warn(format!("profile {} removed", c_accent(&label)));
        }
        Err(AppError::ProfileNotFound(_)) => {
            log_error(format!("profile not found for label {}", c_accent(&label)));
        }
        Err(e) => return Err(e),
    }
    Ok(())
}
