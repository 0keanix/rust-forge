//! Claude Code hook handlers.
//!
//! Follows the Claude Code hooks reference at https://code.claude.com/docs/en/hooks
//! (read on 2026-11-04): input JSON on stdin with `session_id`, `cwd`, `hook_event_name`,
//! `tool_name`, `tool_input`; output JSON on stdout with exit code 0. PreToolUse denies via
//! `hookSpecificOutput.permissionDecision = "deny"` + `permissionDecisionReason`; PostToolUse
//! and Stop block via top-level `decision = "block"` + `reason`; `systemMessage` shows a
//! warning to the user; `hookSpecificOutput.additionalContext` adds context for Claude.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::{diag, gates, util};

/// Budget for the post-edit check.
const POST_EDIT_BUDGET: Duration = Duration::from_secs(120);
/// Consecutive Stop blocks per session before the stop is let through.
const MAX_STOP_BLOCKS: u64 = 3;
const ALLOW_GIT: &str = "# forge: allow-git";

/// Hook event handled by `forge-tools hook`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Event {
    PostEdit,
    Stop,
    PreEditManifest,
}

/// Handle `event` for hook `input` (raw stdin). Returns the stdout text (possibly empty).
pub fn handle(event: Event, input: &str) -> Result<String> {
    let input: Value = serde_json::from_str(input).unwrap_or(Value::Null);
    let cwd = input["cwd"]
        .as_str()
        .map(PathBuf::from)
        .map_or_else(std::env::current_dir, Ok)
        .context("reading current directory")?;
    let Some(root) = util::workspace_root(&cwd) else {
        return Ok(String::new());
    };
    let out = match event {
        Event::PostEdit => post_edit(&root, &cwd, &input)?,
        Event::Stop => stop(&root, &input)?,
        Event::PreEditManifest => pre_edit_manifest(&cwd, &input),
    };
    Ok(out.map(|v| format!("{v}\n")).unwrap_or_default())
}

fn resolve(cwd: &Path, file: &str) -> PathBuf {
    let p = Path::new(file);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        cwd.join(p)
    }
}

fn post_edit(root: &Path, cwd: &Path, input: &Value) -> Result<Option<Value>> {
    let Some(file) = input["tool_input"]["file_path"].as_str() else {
        return Ok(None);
    };
    let file = resolve(cwd, file);
    if file.extension().is_none_or(|e| e != "rs") {
        return Ok(None);
    }
    let start = Instant::now();
    // Formatting failures (e.g. syntax errors) surface through `cargo check` below.
    let _ = Command::new("rustfmt")
        .args(["--edition", "2024"])
        .arg(&file)
        .current_dir(root)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let logs = root.join("target").join("forge").join("logs");
    fs::create_dir_all(&logs).with_context(|| format!("creating {}", logs.display()))?;
    let log = logs.join(format!("{}-post-edit-check.log", util::unix_ts()));
    let log_file = fs::File::create(&log).with_context(|| format!("creating {}", log.display()))?;
    let mut child = Command::new("cargo")
        .args([
            "check",
            "--workspace",
            "--all-targets",
            "--message-format=json",
        ])
        .current_dir(root)
        .stdout(log_file)
        .stderr(Stdio::null())
        .spawn()
        .context("spawning cargo check")?;
    let status = loop {
        if let Some(status) = child.try_wait().context("waiting for cargo check")? {
            break status;
        }
        if start.elapsed() >= POST_EDIT_BUDGET {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(Some(json!({
                "hookSpecificOutput": {
                    "hookEventName": "PostToolUse",
                    "additionalContext": "forge-tools: check deferred (120 s budget exceeded)"
                }
            })));
        }
        std::thread::sleep(Duration::from_millis(100));
    };
    let output = fs::read_to_string(&log).unwrap_or_default();
    let summary = diag::summarize(&output);
    if summary.errors == 0 && status.success() {
        return Ok(None);
    }
    Ok(Some(json!({ "decision": "block", "reason": summary.text })))
}

fn changed_since(root: &Path, head: &str) -> bool {
    let run = |args: &[&str]| -> Option<String> {
        let out = Command::new("git")
            .args(args)
            .current_dir(root)
            .output()
            .ok()?;
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    };
    let (Some(diff), Some(untracked)) = (
        run(&["diff", "--name-only", head]),
        run(&["ls-files", "--others", "--exclude-standard"]),
    ) else {
        return true;
    };
    diff.lines()
        .chain(untracked.lines())
        .any(|f| f.ends_with(".rs") || f.ends_with("Cargo.toml") || f.ends_with("Cargo.lock"))
}

fn stop(root: &Path, input: &Value) -> Result<Option<Value>> {
    let state = util::load_state(root);
    let last_green = state["last_green"]["head"].as_str();
    let needs_run = match last_green {
        Some(head) => changed_since(root, head),
        None => true,
    };
    if !needs_run {
        return Ok(None);
    }
    let report = gates::run(gates::Level::Full, root)?;
    let session = input["session_id"].as_str().unwrap_or("default").to_owned();
    let mut state = util::load_state(root);
    let count = state["stop_blocks"][&session].as_u64().unwrap_or(0);
    let (count, out) = if report.exit_code == 0 {
        (0, None)
    } else if count >= MAX_STOP_BLOCKS {
        (
            0,
            Some(json!({
                "systemMessage": format!(
                    "forge-tools: gates are still red after {MAX_STOP_BLOCKS} blocked stops; letting the stop through\n{}",
                    report.text
                )
            })),
        )
    } else {
        (
            count + 1,
            Some(json!({ "decision": "block", "reason": report.text })),
        )
    };
    if !state["stop_blocks"].is_object() {
        state["stop_blocks"] = json!({});
    }
    state["stop_blocks"][&session] = json!(count);
    util::save_state(root, &state)?;
    Ok(out)
}

fn apply_edit(before: &str, input: &Value) -> Option<String> {
    let ti = &input["tool_input"];
    match input["tool_name"].as_str()? {
        "Write" => ti["content"].as_str().map(str::to_owned),
        "Edit" => {
            let old = ti["old_string"].as_str()?;
            let new = ti["new_string"].as_str()?;
            Some(if ti["replace_all"].as_bool() == Some(true) {
                before.replace(old, new)
            } else {
                before.replacen(old, new, 1)
            })
        }
        "MultiEdit" => {
            let mut text = before.to_owned();
            for e in ti["edits"].as_array()? {
                let old = e["old_string"].as_str()?;
                let new = e["new_string"].as_str()?;
                text = if e["replace_all"].as_bool() == Some(true) {
                    text.replace(old, new)
                } else {
                    text.replacen(old, new, 1)
                };
            }
            Some(text)
        }
        _ => None,
    }
}

/// Source-defining fields of a dependency; features and flags are ignored.
const SOURCE_KEYS: &[&str] = &[
    "version",
    "git",
    "branch",
    "tag",
    "rev",
    "path",
    "registry",
    "package",
    "workspace",
];

#[derive(Debug, PartialEq, Eq)]
struct Dep {
    name: String,
    source: Vec<(String, String)>,
    git: bool,
}

fn dep_of(name: &str, item: &toml_edit::Item) -> Dep {
    let mut source = Vec::new();
    if let Some(v) = item.as_str() {
        source.push(("version".to_owned(), v.to_owned()));
    } else if let Some(t) = item.as_table_like() {
        for key in SOURCE_KEYS {
            if let Some(v) = t.get(key) {
                let v = v
                    .as_value()
                    .map_or_else(|| v.to_string(), |v| v.to_string());
                source.push(((*key).to_owned(), v.trim().to_owned()));
            }
        }
    }
    let git = source.iter().any(|(k, _)| k == "git");
    Dep {
        name: name.to_owned(),
        source,
        git,
    }
}

fn collect_deps(doc: &toml_edit::DocumentMut) -> BTreeMap<String, Dep> {
    const KINDS: &[&str] = &["dependencies", "dev-dependencies", "build-dependencies"];
    let mut tables: Vec<(String, &toml_edit::Item)> = Vec::new();
    for kind in KINDS {
        if let Some(t) = doc.get(kind) {
            tables.push(((*kind).to_owned(), t));
        }
    }
    if let Some(t) = doc.get("workspace").and_then(|w| w.get("dependencies")) {
        tables.push(("workspace.dependencies".to_owned(), t));
    }
    if let Some(targets) = doc.get("target").and_then(toml_edit::Item::as_table_like) {
        for (cfg, t) in targets.iter() {
            for kind in KINDS {
                if let Some(deps) = t.get(kind) {
                    tables.push((format!("target.{cfg}.{kind}"), deps));
                }
            }
        }
    }
    let mut out = BTreeMap::new();
    for (table, item) in tables {
        let Some(t) = item.as_table_like() else {
            continue;
        };
        for (name, dep) in t.iter() {
            out.insert(format!("{table}/{name}"), dep_of(name, dep));
        }
    }
    out
}

fn has_allow_git(text: &str, name: &str) -> bool {
    text.lines().any(|line| {
        let t = line.trim_start();
        let declares = t
            .strip_prefix(name)
            .is_some_and(|rest| rest.trim_start().starts_with(['=', '.']))
            || (t.starts_with('[') && t.contains(&format!(".{name}]")));
        declares && line.contains(ALLOW_GIT)
    })
}

fn pre_edit_manifest(cwd: &Path, input: &Value) -> Option<Value> {
    let file = input["tool_input"]["file_path"].as_str()?;
    let path = resolve(cwd, file);
    if path.file_name().is_none_or(|n| n != "Cargo.toml") {
        return None;
    }
    let before_text = fs::read_to_string(&path).unwrap_or_default();
    let after_text = apply_edit(&before_text, input)?;
    let before: toml_edit::DocumentMut = before_text.parse().unwrap_or_default();
    let after: toml_edit::DocumentMut = after_text.parse().ok()?;
    let before = collect_deps(&before);
    for (key, dep) in collect_deps(&after) {
        if before.get(&key).is_some_and(|b| b.source == dep.source) {
            continue;
        }
        if dep.git && has_allow_git(&after_text, &dep.name) {
            continue;
        }
        return Some(json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "deny",
                "permissionDecisionReason": format!(
                    "use cargo add {0} after forge-tools crate-check {0}", dep.name
                )
            }
        }));
    }
    None
}
