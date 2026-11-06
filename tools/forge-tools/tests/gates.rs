#![expect(clippy::unwrap_used, reason = "tests")]

mod common;

use common::{BADLY_FORMATTED_LIB, CLEAN_LIB, TempDir, run, run_env, stdout, write_crate};
use serde_json::Value;

fn state(dir: &std::path::Path) -> Value {
    let text = std::fs::read_to_string(dir.join("target/forge/state.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

#[test]
fn fmt_failure_stops_before_later_gates() {
    let dir = TempDir::new("gates-fmt");
    write_crate(dir.path(), BADLY_FORMATTED_LIB);
    let out = run(&["gates", "fast"], dir.path(), "");
    let text = stdout(&out);
    assert_eq!(out.status.code(), Some(1), "{text}");
    assert!(text.starts_with("FAIL G0-fmt\n"), "{text}");
    assert!(!text.contains("G1-check"));
    assert!(!text.contains("G2-clippy"));
    let s = state(dir.path());
    assert_eq!(s["level"], "fast");
    assert_eq!(s["gates"], serde_json::json!({ "G0-fmt": "FAIL" }));
    assert!(s["timestamp"].is_u64());
    assert!(s.get("head").is_some());
    let logs: Vec<_> = std::fs::read_dir(dir.path().join("target/forge/logs"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(logs.iter().any(|l| l.ends_with("-G0-fmt.log")), "{logs:?}");
}

#[test]
fn clean_fixture_passes_fast() {
    let dir = TempDir::new("gates-clean");
    write_crate(dir.path(), CLEAN_LIB);
    let out = run(&["gates", "fast"], dir.path(), "");
    let text = stdout(&out);
    assert_eq!(out.status.code(), Some(0), "{text}");
    assert_eq!(text, "PASS G0-fmt\nPASS G1-check\nPASS G2-clippy\n");
    let s = state(dir.path());
    assert_eq!(
        s["gates"],
        serde_json::json!({ "G0-fmt": "PASS", "G1-check": "PASS", "G2-clippy": "PASS" })
    );
}

#[test]
fn not_a_cargo_project_exits_2() {
    let dir = TempDir::new("gates-none");
    let out = run(&["gates", "fast"], dir.path(), "");
    assert_eq!(out.status.code(), Some(2));
}

#[cfg(unix)]
#[test]
fn missing_tool_is_skipped_not_passed() {
    let dir = TempDir::new("gates-skip");
    write_crate(dir.path(), CLEAN_LIB);
    let bin = TempDir::new("gates-skip-bin");
    let cargo = std::env::var("CARGO").unwrap();
    std::os::unix::fs::symlink(cargo, bin.path().join("cargo")).unwrap();
    let path = bin.path().to_str().unwrap();
    let out = run_env(&["gates", "deps"], dir.path(), "", &[("PATH", path)]);
    let text = stdout(&out);
    assert_eq!(out.status.code(), Some(0), "{text}");
    for (gate, tool) in [
        ("G7-deny", "cargo-deny"),
        ("G7-audit", "cargo-audit"),
        ("G7-machete", "cargo-machete"),
    ] {
        assert!(
            text.contains(&format!(
                "SKIPPED {gate}: {tool} not installed (install: cargo install {tool} --locked)"
            )),
            "{text}"
        );
    }
    assert!(!text.contains("PASS"));
    let s = state(dir.path());
    assert_eq!(
        s["gates"],
        serde_json::json!({ "G7-deny": "SKIPPED", "G7-audit": "SKIPPED", "G7-machete": "SKIPPED" })
    );
}

#[test]
fn bin_only_crate_skips_doctests_and_stays_green() {
    let dir = TempDir::new("gates-bin");
    common::write(
        &dir.path().join("Cargo.toml"),
        "[package]\nname = \"fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\n",
    );
    common::write(
        &dir.path().join("src").join("main.rs"),
        "fn main() {\n    println!(\"{}\", double(2));\n}\n\nfn double(x: u32) -> u32 {\n    x * 2\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn doubles() {\n        assert_eq!(super::double(2), 4);\n    }\n}\n",
    );
    let out = run(&["gates", "full"], dir.path(), "");
    let text = stdout(&out);
    assert_eq!(out.status.code(), Some(0), "{text}");
    assert!(text.contains("PASS G3-test\n"), "{text}");
    assert!(
        text.contains("SKIPPED G4-doctest: no library target in the workspace\n"),
        "{text}"
    );
    let s = state(dir.path());
    assert_eq!(s["gates"]["G4-doctest"], "SKIPPED");
    assert_eq!(s["green"], true);
}
