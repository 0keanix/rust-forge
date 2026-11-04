use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use forge_tools::{crate_check, diag, domains, gates, hook, models, util};

#[derive(Parser)]
#[command(
    name = "forge-tools",
    version,
    about = "Deterministic Rust tooling for rust-forge"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Summarize cargo --message-format=json output read from stdin.
    Diag,
    /// Run cargo quality gates.
    Gates {
        #[arg(value_enum)]
        level: gates::Level,
    },
    /// Verify a crate against crates.io.
    CrateCheck { name: String },
    /// Detect project domains from dependencies.
    Domains,
    /// Claude Code hook handlers (hook JSON on stdin).
    Hook {
        #[arg(value_enum)]
        event: hook::Event,
    },
    /// Model tier switching for the plugin's agents.
    Models {
        #[command(subcommand)]
        command: ModelsCmd,
    },
}

#[derive(Subcommand)]
enum ModelsCmd {
    /// Set the model of the strong-tier agents.
    Set {
        #[arg(long, value_enum)]
        strong: models::Strong,
        #[arg(long, default_value = ".")]
        plugin_root: PathBuf,
    },
}

fn stdin() -> Result<String> {
    let mut s = String::new();
    std::io::stdin()
        .read_to_string(&mut s)
        .context("reading stdin")?;
    Ok(s)
}

fn run() -> Result<i32> {
    let cli = Cli::parse();
    match cli.command {
        Cmd::Diag => {
            let summary = diag::summarize(&stdin()?);
            print!("{}", summary.text);
            Ok(summary.exit_code())
        }
        Cmd::Gates { level } => {
            let dir = std::env::current_dir().context("reading current directory")?;
            let report = gates::run(level, &dir)?;
            print!("{}", report.text);
            Ok(report.exit_code)
        }
        Cmd::CrateCheck { name } => {
            let today = i64::try_from(util::unix_ts() / 86_400).unwrap_or(0);
            let report = crate_check::check(&name, &crate_check::UreqClient, today);
            print!("{}", report.text);
            Ok(report.verdict.exit_code())
        }
        Cmd::Domains => {
            let dir = std::env::current_dir().context("reading current directory")?;
            print!("{}", domains::run(&dir)?);
            Ok(0)
        }
        Cmd::Hook { event } => {
            // A hook must never break the session: errors become a silent exit 0.
            let input = stdin().unwrap_or_default();
            match hook::handle(event, &input) {
                Ok(out) => print!("{out}"),
                Err(e) => eprintln!("forge-tools hook: {e:#}"),
            }
            Ok(0)
        }
        Cmd::Models {
            command:
                ModelsCmd::Set {
                    strong,
                    plugin_root,
                },
        } => {
            print!("{}", models::set(&plugin_root, strong)?);
            Ok(0)
        }
    }
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(u8::try_from(code).unwrap_or(1)),
        Err(e) => {
            eprintln!("forge-tools: {e:#}");
            ExitCode::from(1)
        }
    }
}
