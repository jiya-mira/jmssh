use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "jmssh",
    version,
    about = "jmssh - SSH profile manager"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Initialize local config and database
    Init,

    /// Start local HTTP server for the web GUI
    Gui(GuiArgs),

    /// Manage SSH profiles
    Profile(ProfileArgs),

    /// Connect to a profile (optionally by id)
    Connect(ConnectArgs),

    /// Manage passwords in the OS keyring
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

    /// Remove a profile by label
    Rm(ProfileWithoutArgs),

    /// Show a single profile by label
    Show(ProfileWithoutArgs),

    /// List all profiles
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
    pub target: String,

    /// Optional numeric profile id; when set, overrides the label
    #[arg(long, help = "Optional profile id; if provided, it takes precedence over label")]
    pub id: Option<u32>,
}

#[derive(Args)]
pub struct ProfileWithoutArgs {
    /// Profile label, e.g. "prod.web-1"
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