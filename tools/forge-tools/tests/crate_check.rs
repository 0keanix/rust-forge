mod common;

use common::fixture;
use forge_tools::crate_check::{HttpClient, HttpResponse, Verdict, check, days_from_date};

struct Recorded(Result<(u16, String), String>);

impl HttpClient for Recorded {
    fn get(&self, url: &str) -> Result<HttpResponse, String> {
        assert!(url.starts_with("https://crates.io/api/v1/crates/"));
        self.0
            .clone()
            .map(|(status, body)| HttpResponse { status, body })
    }
}

fn today() -> i64 {
    days_from_date("2026-11-04").unwrap_or(0)
}

#[test]
fn ok_for_recent_release() {
    let r = check(
        "serde",
        &Recorded(Ok((200, fixture("crates-io/serde.json")))),
        today(),
    );
    assert_eq!(r.verdict, Verdict::Ok);
    assert_eq!(r.verdict.exit_code(), 0);
    assert!(r.text.starts_with("verdict: OK\n"));
    assert!(r.text.contains("newest stable: 1.0.228\n"));
    assert!(r.text.contains("last update: 2026-09-27\n"));
    assert!(r.text.contains("recent downloads: 65000000\n"));
    assert!(
        r.text
            .contains("repository: https://github.com/serde-rs/serde\n")
    );
}

#[test]
fn stale_for_old_release() {
    let r = check(
        "old-crate",
        &Recorded(Ok((200, fixture("crates-io/stale.json")))),
        today(),
    );
    assert_eq!(r.verdict, Verdict::Stale);
    assert_eq!(r.verdict.exit_code(), 0);
    assert!(r.text.starts_with("verdict: STALE\n"));
}

#[test]
fn not_found_on_404() {
    let r = check(
        "no-such-crate-xyz",
        &Recorded(Ok((404, fixture("crates-io/not-found.json")))),
        today(),
    );
    assert_eq!(r.verdict, Verdict::NotFound);
    assert_eq!(r.verdict.exit_code(), 1);
}

#[test]
fn unknown_on_transport_error() {
    let r = check("serde", &Recorded(Err("dns failure".to_owned())), today());
    assert_eq!(r.verdict, Verdict::Unknown);
    assert_eq!(r.verdict.exit_code(), 3);
    assert!(r.text.starts_with("verdict: UNKNOWN\n"));
}

#[test]
fn date_math() {
    assert_eq!(days_from_date("1970-01-01"), Some(0));
    assert_eq!(days_from_date("2000-03-01"), Some(11_017));
}

#[test]
fn user_agent_names_tool_and_repo() {
    let ua = forge_tools::crate_check::USER_AGENT;
    assert!(ua.starts_with("forge-tools/"));
    assert!(ua.contains("github.com/"));
    assert!(!ua.contains('@'));
}
