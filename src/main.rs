use app::AppContext;
use clap::Parser;
use cli::Command;
use std::io::IsTerminal;

mod app;
mod cli;
mod db;
mod entity;
mod error;
mod handlers;
mod infra;
mod term;
mod ui;
mod usecase;

use crate::cli::Cli;
use crate::db::init_schema;
use crate::term::{c_accent, log_error, log_info, log_warn};
use crate::ui::tui::connect::pick_profile_for_connect;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    let db = db::connect_db().await?;
    let ctx = AppContext::new(db);
    dispatch(&ctx, cli).await
}

async fn dispatch(ctx: &AppContext, cli: Cli) -> Result<()> {
    let is_tty = std::io::stdin().is_terminal() && std::io::stdout().is_terminal();

    if cli.interactive && !is_tty {
        log_error(c_accent("Error: --interactive requires a TTY."));
        std::process::exit(1);
    }

    let can_interactive = !cli.no_interactive && is_tty;

    match cli.command {
        None => {
            if !can_interactive {
                log_error(c_accent(
                    "Error: Missing subcommand in non-interactive mode.",
                ));
                log_info(c_accent("Try 'jmssh --help' for usage."));
                std::process::exit(1);
            }

            if let Some(p) = pick_profile_for_connect(ctx).await? {
                handlers::connect::handle_connect(
                    ctx,
                    cli::ConnectArgs {
                        target: Some(p.label),
                        id: None,
                    },
                )
                .await?;
            }
            Ok(())
        }
        Some(Command::Init) => {
            init_schema(&ctx.db).await?;
            Ok(())
        }
        Some(Command::Gui(_)) => {
            log_warn("gui command is not implemented yet");
            Ok(())
        }
        Some(Command::Password(args)) => {
            handlers::password::handle_password(&ctx, args).await?;
            Ok(())
        }
        Some(Command::Profile(args)) => {
            handlers::profile::handle_profile(&ctx, args).await?;
            Ok(())
        }
        Some(Command::Connect(args)) => {
            handlers::connect::handle_connect(&ctx, args).await?;
            Ok(())
        }
        Some(Command::_Complete(_)) => Ok(()),
    }
}
