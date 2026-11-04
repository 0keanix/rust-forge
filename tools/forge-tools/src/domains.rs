//! Detect project domains from declared dependencies.

use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::Value;

/// Domain → key crates, in report order.
pub const MAPPING: &[(&str, &[&str])] = &[
    ("cli", &["clap"]),
    ("game", &["bevy"]),
    ("web", &["axum"]),
    ("desktop", &["tauri"]),
    ("wasm", &["leptos", "yew", "wasm-bindgen"]),
    ("ffi-mobile", &["uniffi"]),
    ("ml", &["candle-core", "burn", "llama-cpp-2"]),
    ("embedded", &["embassy-executor", "cortex-m"]),
];

fn lock_version(lock: &str, krate: &str) -> Option<String> {
    let doc: toml_edit::DocumentMut = lock.parse().ok()?;
    doc.get("package")?
        .as_array_of_tables()?
        .iter()
        .find(|p| p.get("name").and_then(|n| n.as_str()) == Some(krate))?
        .get("version")?
        .as_str()
        .map(str::to_owned)
}

/// Report detected domains for the cargo project at `dir`.
pub fn run(dir: &Path) -> Result<String> {
    let out = Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--no-deps",
            "--offline",
        ])
        .current_dir(dir)
        .output()
        .context("running cargo metadata")?;
    if !out.status.success() {
        bail!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    let meta: Value = serde_json::from_slice(&out.stdout).context("parsing cargo metadata")?;
    let deps: Vec<&str> = meta["packages"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|p| p["dependencies"].as_array().into_iter().flatten())
        .filter_map(|d| d["name"].as_str())
        .collect();
    let lock = meta["workspace_root"]
        .as_str()
        .and_then(|root| std::fs::read_to_string(Path::new(root).join("Cargo.lock")).ok());
    let mut text = String::new();
    for (domain, crates) in MAPPING {
        let Some(krate) = crates.iter().find(|c| deps.contains(c)) else {
            continue;
        };
        let version = lock
            .as_deref()
            .and_then(|l| lock_version(l, krate))
            .unwrap_or_else(|| "unknown".to_owned());
        text.push_str(&format!(
            "{domain} skill=rust-domain-{domain} crate={krate} version={version}\n"
        ));
    }
    if text.is_empty() {
        text.push_str("no domains detected\n");
    }
    Ok(text)
}
