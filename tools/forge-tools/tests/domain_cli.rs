//! PRD 01 D-3: the domain-cli eval scaffold is detected as the cli domain.
#![expect(clippy::expect_used, clippy::panic, reason = "tests")]

mod common;

use std::path::Path;
use std::process::Command;

use common::{TempDir, run, stdout};

#[test]
fn domain_cli_scaffold_is_detected_as_cli() {
    let dir = TempDir::new("domain-cli");
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../evals/domain-cli/scaffold.sh");
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
        .find(|l| l.split_whitespace().next() == Some("cli"))
        .unwrap_or_else(|| panic!("no cli domain in: {text}"));
    assert!(line.contains("skill=rust-domain-cli"), "{line}");
    assert!(line.contains("crate=clap"), "{line}");
}
