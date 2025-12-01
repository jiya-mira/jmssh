use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "jmssh", version, about = "jmssh - SSH profile manager")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}
#[derive(Subcommand)]
pub enum Command {
    Init,
    Gui(GuiArgs),
    Profile(ProfileArgs),
    Connect(ConnectArgs),
    Password(PasswordArgs),
    #[command(hide = true)]
    _Complete(CompleteArgs),
}

#[derive(Args)]
pub struct ProfileArgs {
    #[command(subcommand)]
    pub cmd: ProfileCommand,
}

#[derive(Subcommand)]
pub enum ProfileCommand {
    Add(EditProfileArgs),
    Set(EditProfileArgs),
    Rm(ProfileWithoutArgs),
    Show(ProfileWithoutArgs),
    List,
}

#[derive(Args)]
pub struct EditProfileArgs {
    /// 用户可见标识，例如 "cc.lax"
    pub label: String,

    /// 新建时必填；更新时不写代表保持原值
    #[arg(long)]
    pub host: Option<String>,

    /// 新建时不写 => 默认 "root"；更新时不写 => 保持原值
    #[arg(long)]
    pub user: Option<String>,

    /// 新建时不写 => 默认 22；更新时不写 => 保持原值
    #[arg(long)]
    pub port: Option<u16>,

    /// 新建时不写 => 默认 "agent"；更新时不写 => 保持原值
    #[arg(long)]
    pub mode: Option<String>,

    /// 不写 => 保持原值；写空串 => 清空
    #[arg(long)]
    pub tags: Option<String>,

    /// 不写 => 保持原值；写空串 => 清空
    #[arg(long)]
    pub note: Option<String>,

    /// 跳板链：可多次传入
    /// - 空 => 不动 jump 链
    /// - 非空 => 覆盖整个 jump 链
    #[arg(long = "jump")]
    pub jumps: Vec<String>,
}

#[derive(Args)]
pub struct ConnectArgs {
    pub target: String,

    #[arg(long)]
    pub id: Option<u32>,
}

#[derive(Args)]
pub struct ProfileWithoutArgs {
    /// Profile 标识，例如 "cc.lax"
    pub label: String,
}

#[derive(Args)]
pub struct PasswordArgs {
    #[command(subcommand)]
    pub cmd: PasswordCommand,
}

#[derive(Subcommand)]
pub enum PasswordCommand {
    Set(PasswordLabelArgs),
    Show(PasswordLabelArgs),
    Clear(PasswordLabelArgs),
}

#[derive(Args)]
pub struct PasswordLabelArgs {
    pub label: String,
}

#[derive(Args)]
pub struct CompleteArgs {
    #[command(subcommand)]
    pub cmd: CompleteSubcommand,
}

#[derive(Subcommand)]
pub enum CompleteSubcommand {
    /// Print all profile labels (one per line)
    Labels,
}

#[derive(Args)]
pub struct GuiArgs {
    /// Bind address for the HTTP server (default: 127.0.0.1)
    #[arg(long, default_value = "127.0.0.1")]
    pub bind: String,

    /// Port for the HTTP server (0 = auto choose a free port)
    #[arg(long, default_value_t = 0)]
    pub port: u16,

    /// Do NOT auto-open browser
    #[arg(long)]
    pub no_open: bool,

    /// Optional remote endpoint (control-plane), reserved for future use
    #[arg(long)]
    pub endpoint: Option<String>,
}
