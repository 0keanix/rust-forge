#![expect(clippy::unwrap_used, reason = "tests")]

mod common;

use common::{TempDir, fixture, run, stdout};
use forge_tools::diag::summarize;

#[test]
fn dedupes_orders_and_adds_help() {
    let s = summarize(&fixture("diag/mixed.jsonl"));
    assert_eq!((s.errors, s.warnings), (4, 1));
    let heads: Vec<&str> = s.text.lines().filter(|l| !l.starts_with("    ")).collect();
    assert_eq!(
        heads,
        [
            "E0425 src/lib.rs:7:13 cannot find value `y` in this scope | help: a local variable with a similar name exists: `x`",
            "error src/lib.rs:20:1 expected item, found `}`",
            "E0599 src/main.rs:2:3 no method named `foo` found",
            "E0308 src/main.rs:10:5 mismatched types",
            "unused_variables src/lib.rs:3:9 unused variable: `x` | help: if this is intentional, prefix it with an underscore: `_x`",
            "errors=4 warnings=1",
        ]
    );
}

#[test]
fn renders_only_first_three_errors() {
    let s = summarize(&fixture("diag/mixed.jsonl"));
    assert!(
        s.text
            .contains("    error[E0425]: cannot find value `y` in this scope\n")
    );
    assert!(s.text.contains("    error: expected item, found `}`\n"));
    assert!(
        s.text
            .contains("    error[E0599]: no method named `foo` found\n")
    );
    assert!(!s.text.contains("error[E0308]"));
    assert!(!s.text.contains("    warning:"));
}

#[test]
fn caps_with_omitted_line() {
    let s = summarize(&fixture("diag/many.jsonl"));
    let lines: Vec<&str> = s.text.lines().collect();
    assert_eq!(lines.iter().filter(|l| l.starts_with("E0425 ")).count(), 20);
    assert_eq!(
        lines.iter().filter(|l| l.starts_with("dead_code ")).count(),
        10
    );
    assert_eq!(
        lines[lines.len() - 2],
        "... 5 more errors, 2 more warnings omitted"
    );
    assert_eq!(lines[lines.len() - 1], "errors=25 warnings=12");
    assert!(!s.text.contains("src/e20.rs"));
}

#[test]
fn ignores_non_json_and_other_reasons() {
    let s = summarize("not json\n{\"reason\":\"build-finished\",\"success\":true}\n\n");
    assert_eq!(s.text, "errors=0 warnings=0\n");
    assert_eq!(s.exit_code(), 0);
}

#[test]
fn binary_exit_codes() {
    let dir = TempDir::new("diag");
    let err = run(&["diag"], dir.path(), &fixture("diag/mixed.jsonl"));
    assert_eq!(err.status.code(), Some(1));
    assert!(stdout(&err).ends_with("errors=4 warnings=1\n"));
    let warn = run(&["diag"], dir.path(), &fixture("diag/warnings-only.jsonl"));
    assert_eq!(warn.status.code(), Some(0));
    assert!(stdout(&warn).ends_with("errors=0 warnings=1\n"));
    let none = run(&["diag"], dir.path(), "garbage\n");
    assert_eq!(none.status.code().unwrap(), 0);
}
