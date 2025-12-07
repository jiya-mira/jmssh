use app::AppContext;
use clap::Parser;
use cli::Command;

mod app;
mod cli;
mod db;
mod entity;
mod error;
mod handlers;
mod infra;
mod term;
mod usecase;

use crate::db::init_schema;
use crate::term::log_warn;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    let db = db::connect_db().await?;
    let ctx = AppContext::new(db.clone());

    match cli.command {
        Command::Init => {
            init_schema(&db).await?;
        }
        Command::Gui(_) => {
            log_warn("gui command is not implemented yet");
        }
        Command::Password(args) => handlers::password::handle_password(&ctx, args).await?,
        Command::Profile(args) => handlers::profile::handle_profile(&ctx, args).await?,
        Command::Connect(args) => handlers::connect::handle_connect(&ctx, args).await?,
        Command::_Complete(_) => {}
    }

    Ok(())
}
