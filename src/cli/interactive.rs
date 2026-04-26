use std::io::{Write as _, stdout};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    style::Print,
    terminal,
};
use owo_colors::OwoColorize;

use crate::cli::args::{AuditArgs, CheckArgs, DiffArgs, ScaffoldArgs, ScanArgs, SecretsArgs};
use crate::cli::commands;

const BANNER_ART: &str = "
  ███████╗███╗   ██╗███████╗██████╗ ███████╗ ██████╗████████╗
  ██╔════╝████╗  ██║██╔════╝██╔══██╗██╔════╝██╔════╝╚══██╔══╝
  █████╗  ██╔██╗ ██║███████╗██████╔╝█████╗  ██║        ██║
  ██╔══╝  ██║╚██╗██║╚════██║██╔═══╝ ██╔══╝  ██║        ██║
  ███████╗██║ ╚████║███████║██║     ███████╗╚██████╗   ██║
  ╚══════╝╚═╝  ╚═══╝╚══════╝╚═╝     ╚══════╝ ╚═════╝   ╚═╝";

const VERSION: &str = env!("CARGO_PKG_VERSION");

// Box inner-content width (between the │ borders).
// Banner art lines are 61 visual chars wide; +2 gives comfortable right padding.
const INNER_W: usize = 63;

// Input box geometry — "  > " = 4 chars used by the prompt prefix inside the box
const PROMPT_INNER: usize = 4;

// Visual column where user typing begins: "  │  > " = 7 chars (0-based)
const CURSOR_COL: u16 = 7;

// ── Box helpers ───────────────────────────────────────────────────────────────

fn top_border(label: &str) -> String {
    let prefix = format!("─ {} ", label);
    let dashes = INNER_W.saturating_sub(prefix.chars().count());
    format!("╭{}{}╮", prefix, "─".repeat(dashes))
}

fn bottom_border_with_label(label: &str) -> String {
    let labeled = format!(" {} ", label);
    let label_len = labeled.chars().count();
    let total = INNER_W.saturating_sub(label_len);
    let left = total / 2;
    let right = total - left;
    format!("╰{}{}{}╯", "─".repeat(left), labeled, "─".repeat(right))
}

/// Build `│{content padded to INNER_W}│` as a plain string, then colour the whole thing.
fn box_row(content: &str, visible_len: usize) -> String {
    let pad = INNER_W.saturating_sub(visible_len);
    format!("│{}{}│", content, " ".repeat(pad))
}

fn empty_row() -> String {
    format!("│{}│", " ".repeat(INNER_W))
}

// ── Welcome screen ────────────────────────────────────────────────────────────

fn print_welcome() {
    println!();
    println!("  {}", top_border(&format!("v{VERSION}")).bright_yellow());
    println!("  {}", empty_row().bright_yellow());

    for line in BANNER_ART.lines().filter(|l| !l.trim().is_empty()) {
        let visible_len = line.chars().count();
        println!("  {}", box_row(line, visible_len).bright_yellow().bold());
    }

    println!("  {}", empty_row().bright_yellow());
    println!(
        "  {}",
        bottom_border_with_label("Environment Variable Auditor")
            .bright_yellow()
            .bold()
    );
    println!();

    println!("  {}", "Tips for getting started:".bright_yellow().bold());
    println!();

    let tips: &[(&str, &str)] = &[
        (
            "enspect audit",
            "Run a full environment variable audit on your project",
        ),
        (
            "enspect secrets",
            "Scan .env files for leaked API keys and credentials",
        ),
        (
            "enspect scan",
            "List all env var references found in source code",
        ),
        (
            "enspect scaffold",
            "Generate .env.local from missing keys in your code",
        ),
        (
            "enspect help",
            "Show all available commands and their usage",
        ),
    ];
    for (cmd, desc) in tips {
        println!("  {:<22}  {}", cmd.bold(), desc.dimmed());
    }

    println!();
    println!("  {}", "─".repeat(INNER_W + 2).dimmed());
    println!();
}

// ── Input box (crossterm raw mode) ───────────────────────────────────────────

enum ReadLine {
    Line(String),
    CtrlC,
    CtrlD,
}

/// Draw the complete 3-row box, position the cursor inside it, and read a line
/// using crossterm raw mode — giving us all 4 borders visible from the start.
fn read_boxed_line(history: &[String]) -> std::io::Result<ReadLine> {
    // Compute width dynamically from current terminal size.
    // Leave 4 chars for "  │  │" (2 indent + 2 borders); minimum = INNER_W.
    let dyn_inner_w = terminal::size()
        .map(|(cols, _)| (cols as usize).saturating_sub(6).max(INNER_W))
        .unwrap_or(INNER_W);
    let dyn_input_area = dyn_inner_w.saturating_sub(PROMPT_INNER);

    // ── Draw all three rows of the box up front ──
    let top = format!("╭{}╮", "─".repeat(dyn_inner_w));
    let bot = format!("╰{}╯", "─".repeat(dyn_inner_w));

    println!("  {}", top.bright_yellow());
    // Middle row: │  > [dyn_input_area spaces]│  — right border always visible
    println!(
        "  {}  {} {:width$}{}",
        "│".bright_yellow(),
        ">".bright_yellow().bold(),
        "",
        "│".bright_yellow(),
        width = dyn_input_area,
    );
    println!("  {}", bot.bright_yellow());
    // Extra blank line — forces the terminal viewport to scroll enough that the
    // bottom border (one row above here) stays visible after MoveUp.
    println!();
    stdout().flush()?;

    // Move cursor back 3 rows (top=0, mid=1, bot=2, blank=3 → we're at 4) to mid
    execute!(
        stdout(),
        cursor::MoveUp(3),
        cursor::MoveToColumn(CURSOR_COL),
    )?;

    // ── Raw-mode line editing ──
    terminal::enable_raw_mode()?;

    let mut input = String::new();
    let mut hist_idx: Option<usize> = None;

    let result = loop {
        let ev = match event::read() {
            Ok(e) => e,
            Err(e) => {
                let _ = terminal::disable_raw_mode();
                return Err(e.into());
            }
        };

        let Event::Key(key) = ev else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }

        match (key.code, key.modifiers) {
            (KeyCode::Enter, _) => break ReadLine::Line(input),

            (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => {
                break ReadLine::CtrlC;
            }
            (KeyCode::Char('d'), m) if m.contains(KeyModifiers::CONTROL) => {
                break ReadLine::CtrlD;
            }

            (KeyCode::Backspace, _) => {
                if !input.is_empty() {
                    input.pop();
                    execute!(
                        stdout(),
                        cursor::MoveLeft(1),
                        Print(' '),
                        cursor::MoveLeft(1),
                    )?;
                }
            }

            (KeyCode::Up, _) => {
                if !history.is_empty() {
                    let idx = match hist_idx {
                        None => history.len() - 1,
                        Some(i) if i > 0 => i - 1,
                        Some(i) => i,
                    };
                    hist_idx = Some(idx);
                    redraw_input(&history[idx], &mut input)?;
                }
            }
            (KeyCode::Down, _) => {
                let new_val = match hist_idx {
                    Some(i) if i + 1 < history.len() => {
                        hist_idx = Some(i + 1);
                        history[i + 1].clone()
                    }
                    _ => {
                        hist_idx = None;
                        String::new()
                    }
                };
                redraw_input(&new_val, &mut input)?;
            }

            (KeyCode::Char(c), _) if input.len() < dyn_input_area => {
                input.push(c);
                execute!(stdout(), Print(c))?;
            }

            _ => {}
        }
        stdout().flush()?;
    };

    terminal::disable_raw_mode()?;

    // Move cursor to 3 rows below mid (past bot + blank) and position at col 0.
    // No extra println here — caller manages spacing between output and next box.
    execute!(stdout(), cursor::MoveDown(3), cursor::MoveToColumn(0),)?;

    Ok(result)
}

/// Replace the current input with `new_val`, clearing any leftover chars.
fn redraw_input(new_val: &str, input: &mut String) -> std::io::Result<()> {
    let old_len = input.len();
    *input = new_val.to_string();
    execute!(
        stdout(),
        cursor::MoveToColumn(CURSOR_COL),
        Print(format!(
            "{:<width$}",
            input,
            width = old_len.max(input.len())
        )),
        cursor::MoveToColumn(CURSOR_COL + input.len() as u16),
    )
}

// ── Help ──────────────────────────────────────────────────────────────────────

fn print_help() {
    println!();
    println!("  {}", "─".repeat(INNER_W + 2).dimmed());
    println!("  {}", "Available commands:".bright_yellow().bold());
    println!();

    let cmds: &[(&str, &str)] = &[
        (
            "enspect audit",
            "Full environment variable audit on the current directory",
        ),
        ("enspect audit --root <path>", "Audit a specific directory"),
        (
            "enspect scan",
            "List all env var references in source files",
        ),
        (
            "enspect check <VAR>",
            "Deep-check a single variable across all sources",
        ),
        ("enspect secrets", "Run secret detection on all .env files"),
        (
            "enspect diff <file1> <file2>",
            "Compare two .env files key-by-key",
        ),
        (
            "enspect init",
            "Generate .Enspect.toml in current directory",
        ),
        ("enspect hook install", "Install pre-commit git hook"),
        ("enspect hook uninstall", "Remove installed git hook"),
        ("enspect scaffold", "Generate .env.local from missing keys"),
        ("help", "Show this list"),
        ("quit / exit", "Exit Enspect"),
    ];
    for (cmd, desc) in cmds {
        println!("  {:<34}  {}", cmd.bold(), desc.dimmed());
    }

    println!();
    println!(
        "  {}  {}",
        "Tip:".bright_yellow(),
        "Run 'enspect audit' to get started.".dimmed(),
    );
    println!("  {}", "─".repeat(INNER_W + 2).dimmed());
    println!();
}

// ── Command dispatcher ────────────────────────────────────────────────────────

fn dispatch(input: &str) {
    let input = input
        .trim()
        .strip_prefix("enspect ")
        .unwrap_or(input.trim());
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }

    match parts[0] {
        "audit" => {
            let args = parse_audit_args(&parts[1..]);
            if let Err(e) = commands::audit::run(&args) {
                eprintln!("  {} {e:#}", "Error:".red().bold());
            }
        }
        "scan" => {
            let args = ScanArgs {
                json: parts[1..].contains(&"--json"),
                root: extract_flag(&parts[1..], "--root").unwrap_or(".".to_string()),
            };
            if let Err(e) = commands::scan::run(&args) {
                eprintln!("  {} {e:#}", "Error:".red().bold());
            }
        }
        "check" => {
            if parts.len() < 2 {
                eprintln!(
                    "  {} {}",
                    "Usage:".bright_yellow(),
                    "enspect check <VAR_NAME>".bold()
                );
                return;
            }
            let args = CheckArgs {
                var_name: parts[1].to_string(),
                root: extract_flag(&parts[2..], "--root").unwrap_or(".".to_string()),
            };
            if let Err(e) = commands::check::run(&args) {
                eprintln!("  {} {e:#}", "Error:".red().bold());
            }
        }
        "secrets" => {
            let args = SecretsArgs {
                path: extract_flag(&parts[1..], "--path"),
                root: extract_flag(&parts[1..], "--root").unwrap_or(".".to_string()),
            };
            if let Err(e) = commands::secrets::run(&args) {
                eprintln!("  {} {e:#}", "Error:".red().bold());
            }
        }
        "diff" => {
            if parts.len() < 3 {
                eprintln!(
                    "  {} {}",
                    "Usage:".bright_yellow(),
                    "enspect diff <file1> <file2>".bold()
                );
                return;
            }
            let args = DiffArgs {
                file1: parts[1].to_string(),
                file2: parts[2].to_string(),
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
        "scaffold" => {
            let args = ScaffoldArgs {
                root: extract_flag(&parts[1..], "--root").unwrap_or(".".to_string()),
                config: extract_flag(&parts[1..], "--config"),
                output: extract_flag(&parts[1..], "-o")
                    .or_else(|| extract_flag(&parts[1..], "--output"))
                    .unwrap_or(".env.local".to_string()),
                force: parts[1..].contains(&"--force"),
                dry_run: parts[1..].contains(&"--dry-run"),
                include_undocumented: parts[1..].contains(&"--include-undocumented"),
            };
            if let Err(e) = commands::scaffold::run(&args) {
                eprintln!("  {} {e:#}", "Error:".red().bold());
            }
        }
        "hook" => {
            if parts.len() < 2 {
                eprintln!(
                    "  {} {}",
                    "Usage:".bright_yellow(),
                    "enspect hook install | uninstall | run".bold()
                );
                return;
            }
            let action = match parts[1] {
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
        "quit" | "exit" | "q" => {
            println!("\n  {}\n", "Goodbye.".dimmed());
            std::process::exit(0);
        }
        "clear" | "cls" => print!("\x1b[2J\x1b[H"),
        other => {
            eprintln!(
                "  {} Unknown command: {}  —  type {} for available commands.",
                "[!]".red().bold(),
                other.bold(),
                "help".bright_yellow(),
            );
        }
    }
}

// ── Argument helpers ──────────────────────────────────────────────────────────

fn parse_audit_args(rest: &[&str]) -> AuditArgs {
    AuditArgs {
        root: extract_flag(rest, "--root").unwrap_or(".".to_string()),
        config: extract_flag(rest, "--config"),
        format: extract_flag(rest, "--format").unwrap_or("pretty".to_string()),
        fail_on: extract_flag(rest, "--fail-on"),
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

fn extract_flag(args: &[&str], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| *a == flag)
        .and_then(|i| args.get(i + 1))
        .map(|v| v.to_string())
}

// ── REPL loop ─────────────────────────────────────────────────────────────────

pub fn run_interactive() -> std::io::Result<()> {
    print_welcome();

    let mut history: Vec<String> = Vec::new();

    loop {
        match read_boxed_line(&history)? {
            ReadLine::Line(line) => {
                let trimmed = line.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }
                history.push(trimmed.clone());
                dispatch(&trimmed);
                println!();
            }
            ReadLine::CtrlC => {
                println!(
                    "  {} Type {} or press {} to exit.\n",
                    "Tip:".bright_yellow(),
                    "quit".bold(),
                    "Ctrl+D".bold(),
                );
            }
            ReadLine::CtrlD => {
                println!("  {}\n", "Goodbye.".dimmed());
                break;
            }
        }
    }

    Ok(())
}
