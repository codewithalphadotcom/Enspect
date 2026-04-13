use owo_colors::OwoColorize;
use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result as RlResult};

use crate::cli::args::{AuditArgs, CheckArgs, DiffArgs, ScanArgs, SecretsArgs};
use crate::cli::commands;

const BANNER: &str = r#"
  ███████╗███╗   ██╗███████╗██████╗ ███████╗ ██████╗████████╗
  ██╔════╝████╗  ██║██╔════╝██╔══██╗██╔════╝██╔════╝╚══██╔══╝
  █████╗  ██╔██╗ ██║███████╗██████╔╝█████╗  ██║        ██║
  ██╔══╝  ██║╚██╗██║╚════██║██╔═══╝ ██╔══╝  ██║        ██║
  ███████╗██║ ╚████║███████║██║     ███████╗╚██████╗   ██║
  ╚══════╝╚═╝  ╚═══╝╚══════╝╚═╝     ╚══════╝ ╚═════╝   ╚═╝
"#;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_welcome() {
    // Banner in gold
    for line in BANNER.lines() {
        println!("{}", line.bright_yellow().bold());
    }
    println!(
        "  {}  {}\n",
        "Environment Variable Auditor".dimmed(),
        format!("v{VERSION}").dimmed(),
    );

    // Tips box
    let border = "─".repeat(56);
    println!("  {}", border.dimmed());
    println!(
        "  {}",
        "Tips for getting started:".bright_yellow().bold()
    );
    println!();
    println!(
        "    {}  {}",
        "1.".dimmed(),
        "audit".bold()
    );
    println!(
        "       {}",
        "Run a full environment variable audit on your project.".dimmed()
    );
    println!(
        "    {}  {}",
        "2.".dimmed(),
        "secrets".bold()
    );
    println!(
        "       {}",
        "Scan .env files for leaked API keys and credentials.".dimmed()
    );
    println!(
        "    {}  {}",
        "3.".dimmed(),
        "scan".bold()
    );
    println!(
        "       {}",
        "List all env var references found in your source code.".dimmed()
    );
    println!(
        "    {}  {}",
        "4.".dimmed(),
        "help".bold()
    );
    println!(
        "       {}",
        "Show all available commands and their usage.".dimmed()
    );
    println!("  {}\n", border.dimmed());
}

fn print_help() {
    let border = "─".repeat(56);
    println!("\n  {}", border.dimmed());
    println!(
        "  {}",
        "Available commands:".bright_yellow().bold()
    );
    println!();

    let cmds: &[(&str, &str)] = &[
        ("audit [--flags]", "Full environment variable audit"),
        ("scan [--json]", "List all env var references in source"),
        ("check <VAR>", "Deep-check a single variable"),
        ("secrets [--path <file>]", "Secret detection on .env files"),
        ("diff <file1> <file2>", "Compare two .env files key-by-key"),
        ("init", "Generate .Enspect.toml config"),
        ("hook install|uninstall|run", "Manage pre-commit git hook"),
        ("help", "Show this help"),
        ("quit / exit", "Exit Enspect"),
    ];

    for (cmd, desc) in cmds {
        println!(
            "    {:<30} {}",
            cmd.bold(),
            desc.dimmed(),
        );
    }

    println!();
    println!(
        "  {}  {}",
        "Tip:".bright_yellow(),
        "Run 'audit' to get started with a full project scan.".dimmed(),
    );
    println!("  {}\n", border.dimmed());
}

fn dispatch(input: &str) {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    let cmd = parts[0];
    let rest = &parts[1..];

    match cmd {
        "audit" => {
            let args = parse_audit_args(rest);
            match commands::audit::run(&args) {
                Ok(_code) => {}
                Err(e) => eprintln!("  {} {e:#}", "Error:".red().bold()),
            }
        }
        "scan" => {
            let json = rest.contains(&"--json");
            let root = extract_flag_value(rest, "--root").unwrap_or(".".to_string());
            let args = ScanArgs { root, json };
            if let Err(e) = commands::scan::run(&args) {
                eprintln!("  {} {e:#}", "Error:".red().bold());
            }
        }
        "check" => {
            if rest.is_empty() {
                eprintln!(
                    "  {} {}",
                    "Usage:".bright_yellow(),
                    "check <VAR_NAME>".bold()
                );
                return;
            }
            let var_name = rest[0].to_string();
            let root = extract_flag_value(rest, "--root").unwrap_or(".".to_string());
            let args = CheckArgs { var_name, root };
            if let Err(e) = commands::check::run(&args) {
                eprintln!("  {} {e:#}", "Error:".red().bold());
            }
        }
        "secrets" => {
            let path = extract_flag_value(rest, "--path");
            let root = extract_flag_value(rest, "--root").unwrap_or(".".to_string());
            let args = SecretsArgs { path, root };
            if let Err(e) = commands::secrets::run(&args) {
                eprintln!("  {} {e:#}", "Error:".red().bold());
            }
        }
        "diff" => {
            if rest.len() < 2 {
                eprintln!(
                    "  {} {}",
                    "Usage:".bright_yellow(),
                    "diff <file1> <file2>".bold()
                );
                return;
            }
            let args = DiffArgs {
                file1: rest[0].to_string(),
                file2: rest[1].to_string(),
            };
            if let Err(e) = commands::diff::run(&args) {
                eprintln!("  {} {e:#}", "Error:".red().bold());
            }
        }
        "init" => {
            if let Err(e) = commands::init::run() {
                eprintln!("  {} {e:#}", "Error:".red().bold());
            }
        }
        "hook" => {
            if rest.is_empty() {
                eprintln!(
                    "  {} {}",
                    "Usage:".bright_yellow(),
                    "hook install | uninstall | run".bold()
                );
                return;
            }
            let action = match rest[0] {
                "install" => crate::cli::args::HookAction::Install,
                "uninstall" => crate::cli::args::HookAction::Uninstall,
                "run" => crate::cli::args::HookAction::Run,
                other => {
                    eprintln!(
                        "  {} Unknown hook action: {}",
                        "Error:".red().bold(),
                        other.bold()
                    );
                    return;
                }
            };
            if let Err(e) = commands::hook::run(&action) {
                eprintln!("  {} {e:#}", "Error:".red().bold());
            }
        }
        "help" | "?" => print_help(),
        "quit" | "exit" | "q" => std::process::exit(0),
        "clear" | "cls" => {
            // ANSI clear screen
            print!("\x1b[2J\x1b[H");
        }
        _ => {
            eprintln!(
                "  {} Unknown command: {}. Type {} for available commands.",
                "!".red().bold(),
                cmd.bold(),
                "help".bright_yellow(),
            );
        }
    }
}

fn parse_audit_args(rest: &[&str]) -> AuditArgs {
    AuditArgs {
        root: extract_flag_value(rest, "--root").unwrap_or(".".to_string()),
        config: extract_flag_value(rest, "--config"),
        format: extract_flag_value(rest, "--format").unwrap_or("pretty".to_string()),
        fail_on: extract_flag_value(rest, "--fail-on"),
        no_color: rest.contains(&"--no-color"),
        quiet: rest.contains(&"-q") || rest.contains(&"--quiet"),
        verbose: rest.contains(&"-v") || rest.contains(&"--verbose"),
        no_secrets: rest.contains(&"--no-secrets"),
        no_git: rest.contains(&"--no-git"),
        no_unused: rest.contains(&"--no-unused"),
        no_empty: rest.contains(&"--no-empty"),
        show_values: rest.contains(&"--show-values"),
        ci: rest.contains(&"--ci"),
        show_all: rest.contains(&"--show-all"),
    }
}

fn extract_flag_value(args: &[&str], flag: &str) -> Option<String> {
    for (i, arg) in args.iter().enumerate() {
        if *arg == flag {
            return args.get(i + 1).map(|v| v.to_string());
        }
    }
    None
}

pub fn run_interactive() -> RlResult<()> {
    print_welcome();

    let mut rl = DefaultEditor::new()?;

    // Build the styled prompt: gold ">" with a space
    let prompt = format!("  {} ", ">".bright_yellow().bold());

    loop {
        match rl.readline(&prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    continue;
                }
                let _ = rl.add_history_entry(trimmed);
                println!();
                dispatch(trimmed);
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C — show hint, don't exit
                println!(
                    "\n  {} Type {} or {} to exit.\n",
                    "Tip:".bright_yellow(),
                    "quit".bold(),
                    "Ctrl+D".bold(),
                );
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D — exit gracefully
                println!("\n  {}\n", "Goodbye.".dimmed());
                break;
            }
            Err(err) => {
                eprintln!("  {} {err}", "Error:".red().bold());
                break;
            }
        }
    }

    Ok(())
}
