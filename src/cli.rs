use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "jmssh", version, about = "jmssh - SSH profile manager")]
pub struct Cli {
    /// Disable interactive TUI (fail on missing args)
    #[arg(long = "no-interactive", global = true)]
    pub no_interactive: bool,

    /// Force interactive TUI (skim)
    #[arg(short = 'i', long = "interactive", global = true)]
    pub interactive: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize local config and database
    Init,

    /// Start local HTTP server for the web GUI
    Gui(GuiArgs),

    /// Manage SSH profiles
    #[command(visible_alias = "p")]
    Profile(ProfileArgs),

    /// Connect to a profile (optionally by id)
    #[command(visible_alias = "c")]
    Connect(ConnectArgs),

    /// Manage passwords in the OS keyring
    #[command(visible_alias = "pwd")]
    Password(PasswordArgs),

    /// Internal completion helper (hidden)
    #[command(hide = true)]
    _Complete(CompleteArgs),
}

#[derive(Args)]
pub struct ProfileArgs {
    /// Profile-related subcommands
    #[command(subcommand)]
    pub cmd: ProfileCommand,
}

#[derive(Subcommand)]
pub enum ProfileCommand {
    /// Create a new profile
    Add(EditProfileArgs),

    /// Update an existing profile in place
    Set(EditProfileArgs),

    /// Remove a profile
    /// If no label provided, opens interactive selection
    Rm(RmArgs),

    /// Show a single profile by label
    Show(ShowArgs),

    /// List all profiles
    #[command(visible_alias = "ls")]
    List,
}

#[derive(Args)]
pub struct EditProfileArgs {
    /// Human-readable profile label, e.g. "prod.web-1"
    pub label: String,

    /// Hostname or IP address of the SSH server
    #[arg(long, help = "Hostname or IP, e.g. 'example.com' or '10.0.0.1'")]
    pub host: Option<String>,

    /// SSH username (default: 'root' on create)
    #[arg(long, help = "SSH username, default 'root' when creating")]
    pub user: Option<String>,

    /// SSH port (default: 22 on create)
    #[arg(long, help = "SSH port number, default 22 when creating")]
    pub port: Option<u16>,

    /// Auth mode: auto | password | key
    #[arg(
        long,
        help = "Auth mode: 'auto' (ssh agent/default), 'password' (keyring+sshpass), 'key' (local private key)"
    )]
    pub mode: Option<String>,

    /// Optional tags (comma-separated)
    #[arg(
        long,
        help = "Optional tags for future filtering (currently reserved / no behavior yet)"
    )]
    pub tags: Option<String>,

    /// Optional note for human use
    #[arg(
        long,
        help = "Optional free-form note (currently reserved / no behavior yet)"
    )]
    pub note: Option<String>,

    /// Jump chain labels. If present, replaces the entire chain.
    ///
    /// Example: --jump bastion --jump edge
    #[arg(
        long = "jump",
        help = "Jump chain labels; pass multiple --jump foo --jump bar. Non-empty list replaces the whole chain."
    )]
    pub jumps: Vec<String>,
}

#[derive(Args)]
pub struct ConnectArgs {
    /// Profile label to connect to, e.g. 'prod.web-1'
    #[arg(help = "Profile label (recommended) or raw host. Leave empty for interactive mode.")]
    pub target: Option<String>,

    /// Optional numeric profile id; when set, overrides the label
    #[arg(long, help = "Profile numeric id; overrides label matching when set")]
    pub id: Option<u32>,
}

#[derive(Args)]
pub struct RmArgs {
    /// Profile label to remove.
    /// If NOT provided, opens interactive TUI to pick a profile to delete.
    #[arg(help = "Profile label. Leave empty to select interactively.")]
    pub label: String,
}

// Show 不做交互式，因为看详情通常是脚本行为，或者既然都交互了直接看 Preview 就行
#[derive(Args)]
pub struct ShowArgs {
    /// Profile label
    #[arg(help = "Profile label")]
    pub label: String, // 这里保持必填，因为 "jmssh show" 不带参数没意义
}

#[derive(Args)]
pub struct ProfileWithoutArgs {
    /// Profile label, e.g. "prod.web-1"
    #[arg(help = "Profile label (recommended) or raw host, e.g. 'prod.web-1'")]
    pub label: String,
}

#[derive(Args)]
pub struct PasswordArgs {
    /// Password-related subcommands
    #[command(subcommand)]
    pub cmd: PasswordCommand,
}

#[derive(Subcommand)]
pub enum PasswordCommand {
    /// Prompt and store password for a profile in OS keyring
    Set(PasswordLabelArgs),

    /// Print stored password for a profile to stdout
    Show(PasswordLabelArgs),

    /// Delete stored password for a profile from OS keyring
    Clear(PasswordLabelArgs),
}

#[derive(Args)]
pub struct PasswordLabelArgs {
    /// Profile label to operate on
    #[arg(help = "Profile label (recommended) or raw host, e.g. 'prod.web-1'")]
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
    #[arg(
        long,
        default_value = "127.0.0.1",
        help = "Bind address for the GUI HTTP server, e.g. 127.0.0.1"
    )]
    pub bind: String,

    /// Port for the HTTP server (0 = auto choose a free port)
    #[arg(
        long,
        default_value_t = 0,
        help = "Port for the GUI HTTP server; 0 = auto select a free port"
    )]
    pub port: u16,

    /// Do NOT auto-open browser
    #[arg(
        long,
        help = "If set, do not auto-open browser after starting the GUI server"
    )]
    pub no_open: bool,

    /// Optional remote endpoint (control-plane), reserved for future use
    #[arg(
        long,
        help = "Optional remote control-plane endpoint (for future remote GUI use)"
    )]
    pub endpoint: Option<String>,
}
