use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "enspect",
    version,
    about = "Environment variable auditor — detect missing, leaked, and undocumented env vars",
    long_about = "Enspect audits environment variable usage across your codebase.\nIt detects missing variables, leaked secrets, undocumented env vars, and more."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run a full environment variable audit
    Audit(AuditArgs),

    /// Initialize .Enspect.toml with defaults
    Init,

    /// Scan source files and list all found env var references
    Scan(ScanArgs),

    /// Check a single variable across all sources
    Check(CheckArgs),

    /// Run secret detection on .env files
    Secrets(SecretsArgs),

    /// Compare two .env files key-by-key
    Diff(DiffArgs),

    /// Manage pre-commit git hook
    Hook(HookArgs),

    /// Generate shell completion scripts
    Completion(CompletionArgs),
}

#[derive(Parser)]
pub struct AuditArgs {
    /// Root directory to scan
    #[arg(long, default_value = ".")]
    pub root: String,

    /// Path to .Enspect.toml config file
    #[arg(long)]
    pub config: Option<String>,

    /// Output format: pretty, json, sarif, github
    #[arg(long, default_value = "pretty")]
    pub format: String,

    /// Comma-separated list of categories that cause non-zero exit
    #[arg(long)]
    pub fail_on: Option<String>,

    /// Disable ANSI color codes
    #[arg(long)]
    pub no_color: bool,

    /// Minimal output — only show findings
    #[arg(short, long)]
    pub quiet: bool,

    /// Extra output — show all references and files scanned
    #[arg(short, long)]
    pub verbose: bool,

    /// Skip secret/entropy detection
    #[arg(long)]
    pub no_secrets: bool,

    /// Skip git integration checks
    #[arg(long)]
    pub no_git: bool,

    /// Don't report unused variables
    #[arg(long)]
    pub no_unused: bool,

    /// Don't report empty/placeholder variables
    #[arg(long)]
    pub no_empty: bool,

    /// Show actual values (dangerous — only for local use)
    #[arg(long)]
    pub show_values: bool,

    /// CI mode: json output + appropriate exit codes
    #[arg(long)]
    pub ci: bool,

    /// Show all categories including low severity
    #[arg(long)]
    pub show_all: bool,
}

#[derive(Parser)]
pub struct ScanArgs {
    /// Root directory to scan
    #[arg(long, default_value = ".")]
    pub root: String,

    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Parser)]
pub struct CheckArgs {
    /// Variable name to check
    pub var_name: String,

    /// Root directory
    #[arg(long, default_value = ".")]
    pub root: String,
}

#[derive(Parser)]
pub struct SecretsArgs {
    /// Specific .env file to check
    #[arg(long)]
    pub path: Option<String>,

    /// Root directory
    #[arg(long, default_value = ".")]
    pub root: String,
}

#[derive(Parser)]
pub struct DiffArgs {
    /// First .env file
    pub file1: String,

    /// Second .env file
    pub file2: String,
}

#[derive(Parser)]
pub struct HookArgs {
    #[command(subcommand)]
    pub action: HookAction,
}

#[derive(Subcommand)]
pub enum HookAction {
    /// Install pre-commit git hook
    Install,
    /// Remove installed git hook
    Uninstall,
    /// Run hook check manually
    Run,
}

#[derive(Parser)]
pub struct CompletionArgs {
    /// Shell to generate completions for
    pub shell: clap_complete::Shell,
}
