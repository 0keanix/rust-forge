//! PRD 03 D-3: the domain-ffi-mobile eval scaffold is detected as the ffi-mobile domain with the pinned uniffi version.
#![expect(clippy::expect_used, clippy::panic, reason = "tests")]

mod common;

use std::path::Path;
use std::process::Command;

use common::{TempDir, run, stdout};

const REPO: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// The uniffi version named in the skill's `metadata.forge-verified`.
fn verified_uniffi_version() -> String {
    let skill =
        std::fs::read_to_string(Path::new(REPO).join("skills/rust-domain-ffi-mobile/SKILL.md"))
            .expect("reading SKILL.md");
    let line = skill
        .lines()
        .find(|l| l.trim_start().starts_with("forge-verified:"))
        .expect("forge-verified line");
    line.split(['=', ','])
        .skip_while(|s| !s.trim_end().ends_with("uniffi"))
        .nth(1)
        .map(|v| v.trim().to_owned())
        .unwrap_or_else(|| panic!("no uniffi pair in: {line}"))
}

#[test]
fn domain_ffi_mobile_scaffold_is_detected_as_ffi_mobile() {
    let dir = TempDir::new("domain-ffi-mobile");
    let script = Path::new(REPO).join("evals/domain-ffi-mobile/scaffold.sh");
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
        .find(|l| l.split_whitespace().next() == Some("ffi-mobile"))
        .unwrap_or_else(|| panic!("no ffi-mobile domain in: {text}"));
    assert!(line.contains("skill=rust-domain-ffi-mobile"), "{line}");
    assert!(line.contains("crate=uniffi"), "{line}");
    let version = verified_uniffi_version();
    assert!(
        line.split_whitespace()
            .any(|w| w == format!("version={version}")),
        "expected version={version} in: {line}"
    );
}
