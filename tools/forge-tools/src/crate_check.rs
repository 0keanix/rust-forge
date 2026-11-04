//! Verify a crate against the crates.io API.

use serde_json::Value;

/// User-Agent sent to crates.io, naming the tool and its repository.
pub const USER_AGENT: &str = concat!(
    "forge-tools/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/0keanix/rust-forge)"
);

/// Releases older than this many days are considered stale (18 months).
const STALE_DAYS: i64 = 548;

/// A minimal HTTP response.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

/// HTTP transport, replaceable in tests.
pub trait HttpClient {
    /// GET `url`; `Err` means a transport failure (no HTTP status).
    fn get(&self, url: &str) -> Result<HttpResponse, String>;
}

/// Real client backed by `ureq`.
pub struct UreqClient;

impl HttpClient for UreqClient {
    fn get(&self, url: &str) -> Result<HttpResponse, String> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(std::time::Duration::from_secs(20)))
            .build()
            .into();
        let mut resp = agent
            .get(url)
            .header("User-Agent", USER_AGENT)
            .call()
            .map_err(|e| e.to_string())?;
        let status = resp.status().as_u16();
        let body = resp
            .body_mut()
            .read_to_string()
            .map_err(|e| e.to_string())?;
        Ok(HttpResponse { status, body })
    }
}

/// Crate verdict.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Ok,
    Stale,
    NotFound,
    Unknown,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Ok => "OK",
            Verdict::Stale => "STALE",
            Verdict::NotFound => "NOT_FOUND",
            Verdict::Unknown => "UNKNOWN",
        }
    }

    pub fn exit_code(self) -> i32 {
        match self {
            Verdict::Ok | Verdict::Stale => 0,
            Verdict::NotFound => 1,
            Verdict::Unknown => 3,
        }
    }
}

/// Result of a crate check.
#[derive(Debug, Clone)]
pub struct Report {
    pub verdict: Verdict,
    pub text: String,
}

/// Days since 1970-01-01 for a `YYYY-MM-DD...` timestamp.
pub fn days_from_date(s: &str) -> Option<i64> {
    let y: i64 = s.get(0..4)?.parse().ok()?;
    let m: i64 = s.get(5..7)?.parse().ok()?;
    let d: i64 = s.get(8..10)?.parse().ok()?;
    // Howard Hinnant's days_from_civil.
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146_097 + doe - 719_468)
}

fn unknown(name: &str, why: &str) -> Report {
    Report {
        verdict: Verdict::Unknown,
        text: format!("verdict: UNKNOWN\ncrate: {name}\nreason: {why}\n"),
    }
}

/// Check `name` using `client`; `today` is days since the Unix epoch.
pub fn check(name: &str, client: &dyn HttpClient, today: i64) -> Report {
    let url = format!("https://crates.io/api/v1/crates/{name}");
    let resp = match client.get(&url) {
        Ok(r) => r,
        Err(e) => return unknown(name, &format!("network error: {e}")),
    };
    if resp.status == 404 {
        return Report {
            verdict: Verdict::NotFound,
            text: format!("verdict: NOT_FOUND\ncrate: {name}\n"),
        };
    }
    if resp.status != 200 {
        return unknown(name, &format!("HTTP status {}", resp.status));
    }
    let Ok(json) = serde_json::from_str::<Value>(&resp.body) else {
        return unknown(name, "invalid JSON from crates.io");
    };
    let krate = &json["crate"];
    let stable = krate["max_stable_version"].as_str();
    let released = stable.and_then(|v| {
        json["versions"]
            .as_array()?
            .iter()
            .find(|x| x["num"].as_str() == Some(v))?["created_at"]
            .as_str()
    });
    let updated = krate["updated_at"].as_str().unwrap_or("unknown");
    let stale = match released.and_then(days_from_date) {
        Some(day) => today - day > STALE_DAYS,
        None => true,
    };
    let verdict = if stale { Verdict::Stale } else { Verdict::Ok };
    let mut text = format!(
        "verdict: {}\ncrate: {name}\nnewest stable: {}\nlast update: {}\nrecent downloads: {}\nrepository: {}\n",
        verdict.as_str(),
        stable.unwrap_or("none"),
        updated.get(0..10).unwrap_or(updated),
        krate["recent_downloads"]
            .as_u64()
            .map_or_else(|| "unknown".to_owned(), |n| n.to_string()),
        krate["repository"].as_str().unwrap_or("none"),
    );
    if stale {
        text.push_str("note: newest stable release is older than 18 months; likely unmaintained, consider an alternative\n");
    }
    Report { verdict, text }
}
