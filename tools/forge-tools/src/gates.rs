//! Cargo quality gates.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::diag;
use crate::util;

/// Gate level selected on the command line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Level {
    Fast,
    Full,
    Deps,
    Unsafe,
}

impl Level {
    fn as_str(self) -> &'static str {
        match self {
            Level::Fast => "fast",
            Level::Full => "full",
            Level::Deps => "deps",
            Level::Unsafe => "unsafe",
        }
    }
}

/// Outcome of a single gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    Fail,
    Skipped,
}

impl Outcome {
    fn as_str(self) -> &'static str {
        match self {
            Outcome::Pass => "PASS",
            Outcome::Fail => "FAIL",
            Outcome::Skipped => "SKIPPED",
        }
    }
}

/// Result of a `gates` run.
#[derive(Debug, Clone)]
pub struct Report {
    pub text: String,
    pub exit_code: i32,
    pub results: Vec<(String, Outcome)>,
}

struct Gate {
    name: String,
    args: Vec<String>,
    /// Executable that must be on PATH; `None` means a custom check is done elsewhere.
    tool: Option<&'static str>,
    install: &'static str,
    json: bool,
}

fn gate(name: &str, args: &[&str], tool: &'static str, install: &'static str, json: bool) -> Gate {
    Gate {
        name: name.to_owned(),
        args: args.iter().map(|s| (*s).to_owned()).collect(),
        tool: Some(tool),
        install,
        json,
    }
}

fn fast_gates() -> Vec<Gate> {
    vec![
        gate(
            "G0-fmt",
            &["fmt", "--all", "--", "--check"],
            "cargo-fmt",
            "rustup component add rustfmt",
            false,
        ),
        gate(
            "G1-check",
            &[
                "check",
                "--workspace",
                "--all-targets",
                "--message-format=json",
            ],
            "cargo",
            "https://rustup.rs",
            true,
        ),
        gate(
            "G2-clippy",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--all-features",
                "--message-format=json",
                "--",
                "-D",
                "warnings",
            ],
            "cargo-clippy",
            "rustup component add clippy",
            true,
        ),
    ]
}

fn test_gates() -> Vec<Gate> {
    let tests = if util::on_path("cargo-nextest") {
        gate(
            "G3-test",
            &["nextest", "run", "--workspace", "--all-features"],
            "cargo-nextest",
            "cargo install cargo-nextest --locked",
            false,
        )
    } else {
        gate(
            "G3-test",
            &["test", "--workspace", "--all-features"],
            "cargo",
            "https://rustup.rs",
            false,
        )
    };
    vec![
        tests,
        gate(
            "G4-doctest",
            &["test", "--workspace", "--doc"],
            "cargo",
            "https://rustup.rs",
            false,
        ),
    ]
}

fn deps_gates() -> Vec<Gate> {
    vec![
        gate(
            "G7-deny",
            &["deny", "check"],
            "cargo-deny",
            "cargo install cargo-deny --locked",
            false,
        ),
        gate(
            "G7-audit",
            &["audit"],
            "cargo-audit",
            "cargo install cargo-audit --locked",
            false,
        ),
        gate(
            "G7-machete",
            &["machete"],
            "cargo-machete",
            "cargo install cargo-machete --locked",
            false,
        ),
    ]
}

fn contains_unsafe(dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|e| {
        let path = e.path();
        if path.is_dir() {
            path.file_name().is_none_or(|n| n != "target") && contains_unsafe(&path)
        } else {
            path.extension().is_some_and(|x| x == "rs")
                && fs::read_to_string(&path).is_ok_and(|s| s.contains("unsafe"))
        }
    })
}

fn unsafe_gates(root: &Path) -> Vec<Gate> {
    let Ok(out) = Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .current_dir(root)
        .output()
    else {
        return Vec::new();
    };
    let meta: Value = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    let mut gates = Vec::new();
    for pkg in meta["packages"].as_array().into_iter().flatten() {
        let (Some(name), Some(manifest)) = (pkg["name"].as_str(), pkg["manifest_path"].as_str())
        else {
            continue;
        };
        let Some(dir) = Path::new(manifest).parent() else {
            continue;
        };
        if contains_unsafe(dir) {
            gates.push(Gate {
                name: format!("G6-miri-{name}"),
                args: ["+nightly", "miri", "test", "-p", name]
                    .iter()
                    .map(|s| (*s).to_owned())
                    .collect(),
                tool: None,
                install: "rustup +nightly component add miri",
                json: false,
            });
        }
    }
    gates
}

fn miri_installed(root: &Path) -> bool {
    Command::new("cargo")
        .args(["+nightly", "miri", "--version"])
        .current_dir(root)
        .output()
        .is_ok_and(|o| o.status.success())
}

fn tail(text: &str, n: usize) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines.len().saturating_sub(n);
    lines[start..].join("\n")
}

fn run_gate(root: &Path, logs: &Path, ts: u64, gate: &Gate) -> Result<(Outcome, String)> {
    let out = Command::new("cargo")
        .args(&gate.args)
        .current_dir(root)
        .output()
        .with_context(|| format!("running cargo for {}", gate.name))?;
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    let log: PathBuf = logs.join(format!("{ts}-{}.log", gate.name));
    fs::write(&log, format!("{stdout}{stderr}"))
        .with_context(|| format!("writing {}", log.display()))?;
    let outcome = if out.status.success() {
        Outcome::Pass
    } else {
        Outcome::Fail
    };
    let mut text = format!("{} {}\n", outcome.as_str(), gate.name);
    if outcome == Outcome::Fail {
        let detail = if gate.json {
            diag::summarize(&stdout).text
        } else {
            format!("{}\n", tail(&format!("{stdout}{stderr}"), 30))
        };
        text.push_str(&detail);
        text.push_str(&format!("log: {}\n", log.display()));
    }
    Ok((outcome, text))
}

/// Run gates of `level` for the cargo workspace containing `dir`.
pub fn run(level: Level, dir: &Path) -> Result<Report> {
    let Some(root) = util::workspace_root(dir) else {
        return Ok(Report {
            text: "not a cargo project\n".to_owned(),
            exit_code: 2,
            results: Vec::new(),
        });
    };
    let gates = match level {
        Level::Fast => fast_gates(),
        Level::Full => fast_gates().into_iter().chain(test_gates()).collect(),
        Level::Deps => deps_gates(),
        Level::Unsafe => unsafe_gates(&root),
    };
    let logs = root.join("target").join("forge").join("logs");
    fs::create_dir_all(&logs).with_context(|| format!("creating {}", logs.display()))?;
    let ts = util::unix_ts();
    let mut text = String::new();
    let mut results = Vec::new();
    let mut failed = false;
    let miri = level == Level::Unsafe && !gates.is_empty() && miri_installed(&root);
    if level == Level::Unsafe && gates.is_empty() {
        text.push_str("no workspace member contains unsafe code\n");
    }
    for gate in &gates {
        let installed = match gate.tool {
            Some(tool) => util::on_path(tool),
            None => miri,
        };
        if !installed {
            let tool = gate.tool.unwrap_or("miri");
            text.push_str(&format!(
                "SKIPPED {}: {tool} not installed (install: {})\n",
                gate.name, gate.install
            ));
            results.push((gate.name.clone(), Outcome::Skipped));
            continue;
        }
        let (outcome, gate_text) = run_gate(&root, &logs, ts, gate)?;
        text.push_str(&gate_text);
        results.push((gate.name.clone(), outcome));
        if outcome == Outcome::Fail {
            failed = true;
            break;
        }
    }
    let head = util::git_head(&root);
    let mut state = util::load_state(&root);
    let gates_json: serde_json::Map<String, Value> = results
        .iter()
        .map(|(n, o)| (n.clone(), json!(o.as_str())))
        .collect();
    state["level"] = json!(level.as_str());
    state["gates"] = Value::Object(gates_json);
    state["head"] = json!(head);
    state["timestamp"] = json!(ts);
    state["green"] = json!(!failed);
    if !failed {
        state["last_green"] = json!({ "level": level.as_str(), "head": head, "timestamp": ts });
    }
    util::save_state(&root, &state)?;
    Ok(Report {
        text,
        exit_code: i32::from(failed),
        results,
    })
}
