#!/usr/bin/env bash
set -euo pipefail
mkdir -p src tests
cat > Cargo.toml <<'EOF'
[package]
name = "cli-notes"
version = "0.1.0"
edition = "2024"

[dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
csv = "1"
dirs = "6"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.9"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"

[workspace]
EOF
cat > src/main.rs <<'EOF'
//! cli-notes: keep short notes in a plain text file, one per line.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

/// Keep short notes.
#[derive(Debug, Parser)]
#[command(name = "cli-notes", version)]
struct Cli {
    /// File that stores the notes.
    #[arg(long, global = true, default_value = "notes.txt")]
    store: PathBuf,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Add a note.
    Add {
        /// Text of the note.
        text: String,
    },
    /// List all notes.
    List,
}

fn read_notes(store: &Path) -> Result<Vec<String>> {
    match fs::read_to_string(store) {
        Ok(text) => Ok(text.lines().map(str::to_owned).collect()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(e).with_context(|| format!("reading {}", store.display())),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Add { text } => {
            anyhow::ensure!(!text.trim().is_empty(), "note text is empty");
            let mut file = fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&cli.store)
                .with_context(|| format!("opening {}", cli.store.display()))?;
            writeln!(file, "{}", text.replace('\n', " "))
                .with_context(|| format!("writing {}", cli.store.display()))?;
        }
        Command::List => {
            for (i, note) in read_notes(&cli.store)?.iter().enumerate() {
                println!("{}: {note}", i + 1);
            }
        }
    }
    Ok(())
}
EOF
cat > tests/cli.rs <<'EOF'
use assert_cmd::Command;
use predicates::str::contains;

#[test]
fn add_then_list() {
    let dir = std::env::temp_dir().join(format!("cli-notes-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let store = dir.join("notes.txt");
    let _ = std::fs::remove_file(&store);
    Command::cargo_bin("cli-notes")
        .unwrap()
        .args(["--store", store.to_str().unwrap(), "add", "buy milk"])
        .assert()
        .success();
    Command::cargo_bin("cli-notes")
        .unwrap()
        .args(["--store", store.to_str().unwrap(), "list"])
        .assert()
        .success()
        .stdout(contains("1: buy milk"));
    let _ = std::fs::remove_dir_all(&dir);
}
EOF
printf 'target/\nnotes.txt\n' > .gitignore
git init -q
git add -A
git -c user.name=eval -c user.email=eval@example.invalid commit -q -m "initial fixture"
