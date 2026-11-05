//! Checks for PRD 00 behaviour specified outside its AC list (added by validation run v-core):
//! hook wiring and fail-open (§4), the subagent table (§5), the root manifest (§3), the core skill's
//! error-code reference (§6), scaffold rules (§7.1), the build-proof grader (§7.1/§7.2) and the eval wrapper (§7.3).
#![allow(clippy::unwrap_used, clippy::panic, reason = "tests")]

mod common;

use std::path::{Path, PathBuf};
use std::process::Command;

use common::TempDir;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .unwrap()
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// Value of a top-level `key: value` line in the frontmatter of `text`.
fn fm_value(text: &str, key: &str) -> Option<String> {
    let body = text.strip_prefix("---\n")?;
    let end = body.find("\n---")?;
    body[..end].lines().find_map(|l| {
        l.strip_prefix(key)
            .and_then(|r| r.strip_prefix(':'))
            .map(|v| v.trim().to_owned())
    })
}

#[test]
fn hooks_wire_each_event_and_fail_open_without_forge_tools() {
    let hooks: serde_json::Value =
        serde_json::from_str(&read(&repo().join("hooks").join("hooks.json"))).unwrap();
    let empty_path = TempDir::new("vcore-nopath");
    for (event, sub, needs_matcher) in [
        ("PreToolUse", "forge-tools hook pre-edit-manifest", true),
        ("PostToolUse", "forge-tools hook post-edit", true),
        ("Stop", "forge-tools hook stop", false),
    ] {
        let groups = hooks["hooks"][event]
            .as_array()
            .unwrap_or_else(|| panic!("{event}: missing"));
        assert_eq!(groups.len(), 1, "{event}");
        let group = &groups[0];
        if needs_matcher {
            let m = group["matcher"].as_str().unwrap_or_default();
            let tools: Vec<&str> = m.split('|').collect();
            assert!(
                tools.contains(&"Edit") && tools.contains(&"Write"),
                "{event}: matcher {m}"
            );
        }
        let handlers = group["hooks"].as_array().unwrap();
        assert_eq!(handlers.len(), 1, "{event}");
        let h = &handlers[0];
        assert_eq!(h["type"], "command", "{event}");
        assert!(h["timeout"].as_u64().is_some_and(|t| t > 0), "{event}");
        let cmd = h["command"].as_str().unwrap();
        assert!(cmd.trim_end().ends_with(sub), "{event}: {cmd}");

        // forge-tools is not on PATH: the hook must exit 0 and print nothing (no block).
        let out = Command::new("/bin/bash")
            .arg("-c")
            .arg(cmd)
            .current_dir(empty_path.path())
            .env("PATH", empty_path.path())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0), "{event}: {cmd}");
        assert!(out.stdout.is_empty(), "{event}: stdout {:?}", out.stdout);
    }
}

#[test]
fn agents_match_prd_table() {
    let table = [
        (
            "crate-architect",
            "fable",
            &["Read", "Grep", "Glob", "Bash"][..],
        ),
        (
            "implementer",
            "inherit",
            &["Read", "Edit", "Write", "Grep", "Glob", "Bash"][..],
        ),
        (
            "compile-fixer",
            "opus",
            &["Read", "Edit", "Grep", "Bash"][..],
        ),
        (
            "test-author",
            "sonnet",
            &["Read", "Edit", "Write", "Grep", "Bash"][..],
        ),
        ("reviewer", "fable", &["Read", "Grep", "Glob"][..]),
        ("unsafe-auditor", "fable", &["Read", "Grep", "Bash"][..]),
        ("deps-guardian", "haiku", &["Bash", "Read"][..]),
    ];
    for (name, model, tools) in table {
        let text = read(&repo().join("agents").join(format!("{name}.md")));
        assert_eq!(fm_value(&text, "model").as_deref(), Some(model), "{name}");
        let mut got: Vec<String> = fm_value(&text, "tools")
            .unwrap_or_default()
            .split(',')
            .map(|t| t.trim().to_owned())
            .collect();
        got.sort();
        let mut want: Vec<String> = tools.iter().map(|t| (*t).to_owned()).collect();
        want.sort();
        assert_eq!(got, want, "{name}");
    }
}

#[test]
fn root_manifest_excludes_evals_and_fixtures() {
    let doc: toml_edit::DocumentMut = read(&repo().join("Cargo.toml")).parse().unwrap();
    let exclude: Vec<&str> = doc["workspace"]["exclude"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert!(exclude.contains(&"evals"), "{exclude:?}");
    assert!(
        exclude.contains(&"tools/forge-tools/tests/fixtures"),
        "{exclude:?}"
    );
}

#[test]
fn error_codes_reference_covers_every_prd_code() {
    let text = read(
        &repo()
            .join("skills")
            .join("rust-forge")
            .join("references")
            .join("error-codes.md"),
    );
    for code in [
        "E0382", "E0499", "E0502", "E0505", "E0597", "E0106", "E0277", "E0308", "E0599", "E0425",
    ] {
        assert!(
            text.lines().any(|l| l.starts_with('#') && l.contains(code)),
            "no section for {code}"
        );
    }
}

fn cargo_tomls(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        let name = p.file_name().unwrap().to_string_lossy().into_owned();
        if p.is_dir() && name != ".git" && name != "target" {
            cargo_tomls(&p, out);
        } else if name == "Cargo.toml" {
            out.push(p);
        }
    }
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("HOME", dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "git {args:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// §7.1: the script works from anywhere (no reads from the case directory), every fixture `Cargo.toml` has an
/// empty `[workspace]` table, and the fixture is committed once with a clean tree.
#[test]
fn scaffolds_are_self_contained_isolated_and_committed() {
    for case in ["core-borrow-fix", "core-new-dependency"] {
        let tmp = TempDir::new("vcore-scaffold");
        let script_dir = tmp.path().join("script");
        let ws = tmp.path().join("ws");
        std::fs::create_dir_all(&script_dir).unwrap();
        std::fs::create_dir_all(&ws).unwrap();
        let script = script_dir.join("scaffold.sh");
        std::fs::copy(repo().join("evals").join(case).join("scaffold.sh"), &script).unwrap();
        let out = Command::new("bash")
            .arg(&script)
            .current_dir(&ws)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("HOME", &ws)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{case}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let mut manifests = Vec::new();
        cargo_tomls(&ws, &mut manifests);
        assert!(!manifests.is_empty(), "{case}: no Cargo.toml");
        for m in manifests {
            let doc: toml_edit::DocumentMut = read(&m).parse().unwrap();
            let ws_table = doc
                .get("workspace")
                .and_then(|w| w.as_table())
                .unwrap_or_else(|| panic!("{case}: {} has no [workspace]", m.display()));
            assert!(ws_table.is_empty(), "{case}: [workspace] is not empty");
        }
        assert_eq!(
            git(&ws, &["rev-list", "--count", "HEAD"]).trim(),
            "1",
            "{case}"
        );
        assert_eq!(
            git(&ws, &["status", "--porcelain"]),
            "",
            "{case}: uncommitted files"
        );
    }
}

/// §7.1/§7.2: core-borrow-fix proves the build through state.json written by the Stop hook, and its prompt tells
/// the model that cargo cannot run in Bash.
#[test]
fn borrow_fix_proves_build_through_state_json() {
    let case = repo().join("evals").join("core-borrow-fix");
    let mut found = false;
    for e in std::fs::read_dir(case.join("graders")).unwrap() {
        let text = read(&e.unwrap().path());
        if fm_value(&text, "type").as_deref() == Some("regex")
            && text.contains("path: target/forge/state.json")
        {
            assert!(text.contains("source: file"), "{text}");
            assert!(text.contains(r#""G3-test":\s*"PASS""#), "{text}");
            assert!(text.contains(r#""green":\s*true"#), "{text}");
            found = true;
        }
    }
    assert!(found, "no regex grader on target/forge/state.json");
    let prompt = read(&case.join("prompt.md"));
    assert!(prompt.contains("append_system_prompt:"));
    assert!(prompt.contains("cargo") && prompt.contains("hooks"));
}

/// §7.3: the wrapper always passes the fixed flags and grants exactly the tools the cases need.
#[test]
fn plugin_eval_script_passes_fixed_flags() {
    let path = repo().join("scripts").join("plugin-eval.sh");
    let text = read(&path);
    for needle in [
        "--trust-plugin",
        "--scaffold",
        "--threshold \"${FORGE_EVAL_THRESHOLD:-0.8}\"",
        "--no-publish",
        "--max-cost-usd \"${FORGE_EVAL_MAX_COST:-15}\"",
        "--model \"$FORGE_EVAL_MODEL\"",
        "--judge-model \"$FORGE_EVAL_JUDGE_MODEL\"",
        "--allow-tools",
        "\"WebFetch(domain:index.crates.io)\"",
        "\"WebFetch(domain:static.crates.io)\"",
        "\"WebFetch(domain:crates.io)\"",
    ] {
        assert!(text.contains(needle), "plugin-eval.sh lacks {needle}");
    }
    let ok = Command::new("bash").arg("-n").arg(&path).status().unwrap();
    assert!(ok.success());
}
