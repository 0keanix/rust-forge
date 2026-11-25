//! PRD 06 D-3: the domain-desktop eval scaffold is detected as the desktop domain with the pinned tauri version.
#![expect(clippy::expect_used, clippy::panic, reason = "tests")]

mod common;

use std::path::Path;
use std::process::Command;

use common::{TempDir, run, stdout};

const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// The tauri version named in the skill's `metadata.forge-verified`.
fn verified_tauri_version() -> String {
    let skill =
        std::fs::read_to_string(Path::new(REPO).join("skills/rust-domain-desktop/SKILL.md"))
            .expect("reading SKILL.md");
    let line = skill
        .lines()
        .find(|l| l.trim_start().starts_with("forge-verified:"))
        .expect("forge-verified line");
    line.split(['=', ','])
        .skip_while(|s| !s.trim_end().ends_with("tauri"))
        .nth(1)
        .map(|v| v.trim().to_owned())
        .unwrap_or_else(|| panic!("no tauri pair in: {line}"))
}

#[test]
fn domain_desktop_scaffold_is_detected_as_desktop_with_verified_version() {
    let dir = TempDir::new("domain-desktop");
    let script = Path::new(REPO).join("evals/domain-desktop/scaffold.sh");
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
        .find(|l| l.split_whitespace().next() == Some("desktop"))
        .unwrap_or_else(|| panic!("no desktop domain in: {text}"));
    assert!(line.contains("skill=rust-domain-desktop"), "{line}");
    assert!(line.contains("crate=tauri"), "{line}");
    let version = verified_tauri_version();
    assert!(
        line.split_whitespace()
            .any(|w| w == format!("version={version}")),
        "expected version={version} in: {line}"
    );
}
