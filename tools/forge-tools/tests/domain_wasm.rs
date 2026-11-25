//! PRD 07 D-3: the domain-wasm eval scaffold is detected as the wasm domain with the pinned leptos version.
#![expect(clippy::expect_used, clippy::panic, reason = "tests")]

mod common;

use std::path::Path;
use std::process::Command;

use common::{TempDir, run, stdout};

const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// The leptos version named in the skill's `metadata.forge-verified`.
fn verified_leptos_version() -> String {
    let skill = std::fs::read_to_string(Path::new(REPO).join("skills/rust-domain-wasm/SKILL.md"))
        .expect("reading SKILL.md");
    let line = skill
        .lines()
        .find(|l| l.trim_start().starts_with("forge-verified:"))
        .expect("forge-verified line");
    line.split(['=', ','])
        .skip_while(|s| !s.trim_end().ends_with("leptos"))
        .nth(1)
        .map(|v| v.trim().to_owned())
        .unwrap_or_else(|| panic!("no leptos pair in: {line}"))
}

#[test]
fn domain_wasm_scaffold_is_detected_as_wasm_with_verified_version() {
    let dir = TempDir::new("domain-wasm");
    let script = Path::new(REPO).join("evals/domain-wasm/scaffold.sh");
    let status = Command::new("bash")
        .arg(&script)
        .current_dir(dir.path())
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .status()
        .expect("running scaffold.sh");
    assert!(status.success(), "scaffold.sh failed: {status}");

    let out = run(&["domains"], dir.path(), "");
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = stdout(&out);
    let line = text
        .lines()
        .find(|l| l.split_whitespace().next() == Some("wasm"))
        .unwrap_or_else(|| panic!("no wasm domain in: {text}"));
    assert!(line.contains("skill=rust-domain-wasm"), "{line}");
    assert!(line.contains("crate=leptos"), "{line}");
    let version = verified_leptos_version();
    assert!(
        line.split_whitespace()
            .any(|w| w == format!("version={version}")),
        "expected version={version} in: {line}"
    );
}
