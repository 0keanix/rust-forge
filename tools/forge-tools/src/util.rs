//! Small helpers shared by the subcommands.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use serde_json::Value;

/// Cargo workspace root for `dir`, or `None` when `dir` is not inside a cargo project.
pub fn workspace_root(dir: &Path) -> Option<PathBuf> {
    let out = Command::new("cargo")
        .args(["locate-project", "--workspace", "--message-format", "plain"])
        .current_dir(dir)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let manifest = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    manifest.parent().map(Path::to_path_buf)
}

/// Whether an executable named `name` is on `PATH`.
pub fn on_path(name: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| {
        let candidate = dir.join(name);
        candidate.is_file() || candidate.with_extension("exe").is_file()
    })
}

/// Seconds since the Unix epoch.
pub fn unix_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// `git rev-parse HEAD` in `dir`, if it is a git checkout with commits.
pub fn git_head(dir: &Path) -> Option<String> {
    let out = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()?;
    out.status
        .success()
        .then(|| String::from_utf8_lossy(&out.stdout).trim().to_owned())
}

fn state_path(root: &Path) -> PathBuf {
    root.join("target").join("forge").join("state.json")
}

/// Load `target/forge/state.json` as a JSON object (empty object when absent or invalid).
pub fn load_state(root: &Path) -> Value {
    std::fs::read_to_string(state_path(root))
        .ok()
        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
        .filter(Value::is_object)
        .unwrap_or_else(|| Value::Object(serde_json::Map::new()))
}

/// Write `target/forge/state.json`.
pub fn save_state(root: &Path, state: &Value) -> Result<()> {
    let path = state_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating {}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(state).context("serializing state")?;
    std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))
}
