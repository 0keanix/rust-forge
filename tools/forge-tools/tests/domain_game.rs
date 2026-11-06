//! PRD 02 D-3, D-6: the domain-game eval scaffold is detected as the game domain with the pinned Bevy version.
#![expect(clippy::expect_used, clippy::panic, reason = "tests")]

mod common;

use std::path::Path;
use std::process::Command;

use common::{TempDir, run, stdout};

const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// The bevy version named in the skill's `metadata.forge-verified`.
fn verified_bevy_version() -> String {
    let skill = std::fs::read_to_string(Path::new(REPO).join("skills/rust-domain-game/SKILL.md"))
        .expect("reading SKILL.md");
    let line = skill
        .lines()
        .find(|l| l.trim_start().starts_with("forge-verified:"))
        .expect("forge-verified line");
    line.split(['=', ','])
        .skip_while(|s| !s.trim_end().ends_with("bevy"))
        .nth(1)
        .map(|v| v.trim().to_owned())
        .unwrap_or_else(|| panic!("no bevy pair in: {line}"))
}

#[test]
fn domain_game_scaffold_is_detected_as_game_with_verified_version() {
    let dir = TempDir::new("domain-game");
    let script = Path::new(REPO).join("evals/domain-game/scaffold.sh");
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
        .find(|l| l.split_whitespace().next() == Some("game"))
        .unwrap_or_else(|| panic!("no game domain in: {text}"));
    assert!(line.contains("skill=rust-domain-game"), "{line}");
    assert!(line.contains("crate=bevy"), "{line}");
    let version = verified_bevy_version();
    assert!(
        line.split_whitespace()
            .any(|w| w == format!("version={version}")),
        "expected version={version} in: {line}"
    );
}
