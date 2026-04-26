mod audit;
mod cli;
mod config;
mod detector;
mod git;
mod parser;
mod reporter;
mod scanner;
mod utils;

use clap::Parser;
use cli::args::{Cli, Commands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        // Default: launch interactive REPL
        None => {
            if let Err(e) = cli::interactive::run_interactive() {
                eprintln!("Error: {e}");
                std::process::exit(2);
            }
        }
        Some(Commands::Audit(args)) => match cli::commands::audit::run(&args) {
            Ok(code) => std::process::exit(code as i32),
            Err(e) => {
                eprintln!("Error: {e:#}");
                std::process::exit(2);
            }
        },
        Some(Commands::Init) => {
            if let Err(e) = cli::commands::init::run() {
                eprintln!("Error: {e:#}");
                std::process::exit(2);
            }
        }
        Some(Commands::Scan(args)) => {
            if let Err(e) = cli::commands::scan::run(&args) {
                eprintln!("Error: {e:#}");
                std::process::exit(2);
            }
        }
        Some(Commands::Check(args)) => {
            if let Err(e) = cli::commands::check::run(&args) {
                eprintln!("Error: {e:#}");
                std::process::exit(2);
            }
        }
        Some(Commands::Secrets(args)) => {
            if let Err(e) = cli::commands::secrets::run(&args) {
                eprintln!("Error: {e:#}");
                std::process::exit(2);
            }
        }
        Some(Commands::Diff(args)) => {
            if let Err(e) = cli::commands::diff::run(&args) {
                eprintln!("Error: {e:#}");
                std::process::exit(2);
            }
        }
        Some(Commands::Hook(args)) => {
            if let Err(e) = cli::commands::hook::run(&args.action) {
                eprintln!("Error: {e:#}");
                std::process::exit(2);
            }
        }
        Some(Commands::Scaffold(args)) => {
            if let Err(e) = cli::commands::scaffold::run(&args) {
                eprintln!("Error: {e:#}");
                std::process::exit(2);
            }
        }
        Some(Commands::Completion(args)) => {
            use clap::CommandFactory;
            use clap_complete::generate;
            let mut cmd = Cli::command();
            generate(args.shell, &mut cmd, "enspect", &mut std::io::stdout());
        }
    };
}
