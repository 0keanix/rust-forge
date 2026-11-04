#![expect(clippy::unwrap_used, reason = "tests")]

mod common;

use std::path::Path;

use common::{BADLY_FORMATTED_LIB, CLEAN_LIB, TempDir, run, stdout, write, write_crate};
use serde_json::{Value, json};

const MANIFEST: &str = r#"[package]
name = "fixture"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
anyhow = "1"

[target.'cfg(unix)'.dependencies]
libc = "0.2"
"#;

fn hook(event: &str, cwd: &Path, input: &Value) -> (i32, String) {
    let out = run(&["hook", event], cwd, &input.to_string());
    (out.status.code().unwrap(), stdout(&out))
}

fn edit(dir: &Path, old: &str, new: &str) -> (i32, String) {
    let input = json!({
        "session_id": "s1",
        "cwd": dir,
        "hook_event_name": "PreToolUse",
        "tool_name": "Edit",
        "tool_input": {
            "file_path": dir.join("Cargo.toml"),
            "old_string": old,
            "new_string": new
        }
    });
    hook("pre-edit-manifest", dir, &input)
}

fn manifest_dir(tag: &str) -> TempDir {
    let dir = TempDir::new(tag);
    write(&dir.path().join("Cargo.toml"), MANIFEST);
    write(&dir.path().join("src/lib.rs"), "");
    dir
}

fn assert_denied(out: (i32, String), name: &str) {
    assert_eq!(out.0, 0);
    let v: Value = serde_json::from_str(&out.1).unwrap();
    assert_eq!(v["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    assert_eq!(v["hookSpecificOutput"]["permissionDecision"], "deny");
    assert_eq!(
        v["hookSpecificOutput"]["permissionDecisionReason"],
        format!("use cargo add {name} after forge-tools crate-check {name}")
    );
}

fn assert_allowed(out: (i32, String)) {
    assert_eq!(out, (0, String::new()));
}

#[test]
fn manifest_blocks_new_dependency() {
    let dir = manifest_dir("pre-new");
    assert_denied(
        edit(
            dir.path(),
            "anyhow = \"1\"\n",
            "anyhow = \"1\"\nclap = \"4\"\n",
        ),
        "clap",
    );
    assert_denied(
        edit(
            dir.path(),
            "libc = \"0.2\"\n",
            "libc = \"0.2\"\nnix = \"0.30\"\n",
        ),
        "nix",
    );
}

#[test]
fn manifest_blocks_version_bump() {
    let dir = manifest_dir("pre-bump");
    assert_denied(
        edit(dir.path(), "anyhow = \"1\"", "anyhow = \"2\""),
        "anyhow",
    );
    assert_denied(
        edit(
            dir.path(),
            "version = \"1.0\", features",
            "version = \"1.1\", features",
        ),
        "serde",
    );
}

#[test]
fn manifest_blocks_write_with_new_dependency() {
    let dir = manifest_dir("pre-write");
    let input = json!({
        "cwd": dir.path(),
        "tool_name": "Write",
        "tool_input": {
            "file_path": dir.path().join("Cargo.toml"),
            "content": format!("{MANIFEST}\n[dev-dependencies]\nrstest = \"0.26\"\n")
        }
    });
    assert_denied(hook("pre-edit-manifest", dir.path(), &input), "rstest");
}

#[test]
fn manifest_allows_features_removal_and_other_edits() {
    let dir = manifest_dir("pre-allow");
    assert_allowed(edit(dir.path(), "[\"derive\"]", "[\"derive\", \"rc\"]"));
    assert_allowed(edit(dir.path(), "anyhow = \"1\"\n", ""));
    assert_allowed(edit(
        dir.path(),
        "version = \"0.1.0\"",
        "version = \"0.2.0\"",
    ));
}

#[test]
fn manifest_git_dependency_needs_marker() {
    let dir = manifest_dir("pre-git");
    assert_allowed(edit(
        dir.path(),
        "anyhow = \"1\"\n",
        "anyhow = \"1\"\nfoo = { git = \"https://github.com/example/foo\" } # forge: allow-git\n",
    ));
    assert_denied(
        edit(
            dir.path(),
            "anyhow = \"1\"\n",
            "anyhow = \"1\"\nfoo = { git = \"https://github.com/example/foo\" }\n",
        ),
        "foo",
    );
}

#[test]
fn manifest_ignores_other_files() {
    let dir = manifest_dir("pre-other");
    let input = json!({
        "cwd": dir.path(),
        "tool_name": "Write",
        "tool_input": { "file_path": dir.path().join("src/lib.rs"), "content": "clap = \"4\"" }
    });
    assert_allowed(hook("pre-edit-manifest", dir.path(), &input));
}

fn stop_input(dir: &Path) -> Value {
    json!({
        "session_id": "sess-1",
        "cwd": dir,
        "hook_event_name": "Stop",
        "stop_hook_active": false
    })
}

#[test]
fn stop_blocks_on_failure_then_lets_through_after_three() {
    let dir = TempDir::new("stop-fail");
    write_crate(dir.path(), BADLY_FORMATTED_LIB);
    for i in 1..=3 {
        let (code, out) = hook("stop", dir.path(), &stop_input(dir.path()));
        assert_eq!(code, 0);
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["decision"], "block", "attempt {i}");
        assert!(v["reason"].as_str().unwrap().contains("FAIL G0-fmt"));
    }
    let (code, out) = hook("stop", dir.path(), &stop_input(dir.path()));
    assert_eq!(code, 0);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert!(v.get("decision").is_none(), "{out}");
    assert!(v["systemMessage"].as_str().unwrap().contains("still red"));
}

#[test]
fn stop_passes_on_clean_fixture() {
    let dir = TempDir::new("stop-clean");
    write_crate(dir.path(), CLEAN_LIB);
    assert_eq!(
        hook("stop", dir.path(), &stop_input(dir.path())),
        (0, String::new())
    );
}

#[test]
fn post_edit_reports_errors_and_is_silent_when_clean() {
    let dir = TempDir::new("post-edit");
    write_crate(dir.path(), CLEAN_LIB);
    let input = json!({
        "cwd": dir.path(),
        "hook_event_name": "PostToolUse",
        "tool_name": "Edit",
        "tool_input": { "file_path": dir.path().join("src/lib.rs") }
    });
    assert_eq!(hook("post-edit", dir.path(), &input), (0, String::new()));
    write(
        &dir.path().join("src/lib.rs"),
        "pub fn f() -> u32 {\n    missing_value\n}\n",
    );
    let (code, out) = hook("post-edit", dir.path(), &input);
    assert_eq!(code, 0);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["decision"], "block");
    assert!(v["reason"].as_str().unwrap().contains("E0425"), "{out}");
}

#[test]
fn handlers_silent_outside_cargo_project() {
    let dir = TempDir::new("outside");
    let input = json!({
        "session_id": "s",
        "cwd": dir.path(),
        "tool_name": "Edit",
        "tool_input": {
            "file_path": dir.path().join("Cargo.toml"),
            "old_string": "",
            "new_string": "[dependencies]\nclap = \"4\"\n"
        }
    });
    for event in ["post-edit", "stop", "pre-edit-manifest"] {
        let out = run(&["hook", event], dir.path(), &input.to_string());
        assert_eq!(out.status.code(), Some(0), "{event}");
        assert!(out.stdout.is_empty(), "{event}");
        assert!(out.stderr.is_empty(), "{event}");
    }
}
