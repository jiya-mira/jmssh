use app::AppContext;
use clap::Parser;
use cli::{Command, ConnectArgs, EditProfileArgs, ProfileWithoutArgs};

mod app;
mod cli;
mod db;
mod entity;
mod error;
mod handlers;
mod usecase;
mod term;
mod infra;

use crate::db::init_schema;
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
        Command::Gui(args) => {
            println!("[jmssh] gui (TODO: start web server and open browser)");
        }
        Command::Profile(args) => handlers::profile::handle_profile(&ctx, args).await?,
        Command::Connect(args) => handlers::connect::handle_connect(&ctx, args).await?,
        Command::_Complete(args) => {}
    }

    Ok(())
}

fn handle_profile_add(args: EditProfileArgs) {
    println!("[jmssh] profile add");
    println!("  label: {}", args.label);
    println!("  host:  {:?}", args.host);
    println!("  user:  {:?}", args.user);
    println!("  port:  {:?}", args.port);
    println!("  mode:  {:?}", args.mode);
    println!("  tags:  {:?}", args.tags);
    println!("  note:  {:?}", args.note);
    println!("  jumps: {:?}", args.jumps);
    // TODO: insert into DB
}

fn handle_profile_set(args: EditProfileArgs) {
    println!("[jmssh] profile set");
    println!("  label: {}", args.label);
    println!("  host:  {:?}", args.host);
    println!("  user:  {:?}", args.user);
    println!("  port:  {:?}", args.port);
    println!("  mode:  {:?}", args.mode);
    println!("  tags:  {:?}", args.tags);
    println!("  note:  {:?}", args.note);
    println!("  jumps: {:?}", args.jumps);
    // TODO: update DB
}

fn handle_profile_rm(args: ProfileWithoutArgs) {
    println!("[jmssh] profile rm");
    println!("  label: {}", args.label);
    // TODO: delete profile + routes + local_auth
}

fn handle_profile_show(args: ProfileWithoutArgs) {
    println!("[jmssh] profile show");
    println!("  label: {}", args.label);
    // TODO: query and pretty-print profile + jumps
}

fn handle_profile_list() {
    println!("[jmssh] profile list");
    // TODO: list profiles from DB
}

fn handle_connect(args: ConnectArgs) {
    println!("[jmssh] connect");
    println!("  target: {}", args.target);
    println!("  id:     {:?}", args.id);
    // TODO: resolve profile + build ssh command + exec
}

fn handle_complete_labels() {
    // 这里先占位，方便调通 CLI；
    // 真正实现时应该一行一个 label 打到 stdout。
    println!("[jmssh] _complete labels (TODO: print labels for shell completion)");
}
