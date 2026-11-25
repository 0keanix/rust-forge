//! v-web validation: behaviour PRD 05 specifies in its sections (Deliver, Skill content, Rules, Eval case)
//! outside the D-1..D-5 list.
#![expect(clippy::unwrap_used, clippy::panic, reason = "tests")]

use std::path::{Path, PathBuf};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// Body after the closing `---` of the frontmatter.
fn body(text: &str) -> &str {
    let rest = text.strip_prefix("---\n").unwrap();
    let end = rest.find("\n---\n").unwrap();
    &rest[end + 5..]
}

fn frontmatter(text: &str) -> &str {
    let rest = text.strip_prefix("---\n").unwrap();
    &rest[..rest.find("\n---\n").unwrap()]
}

/// Collapses line wrapping so phrases can be matched across lines.
fn flat(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Contents of the `cat > <file> <<'EOF'` heredoc in scaffold.sh.
fn heredoc<'a>(scaffold: &'a str, file: &str) -> &'a str {
    scaffold
        .split(&format!("cat > {file} <<'EOF'\n"))
        .nth(1)
        .and_then(|s| s.split("\nEOF\n").next())
        .unwrap_or_else(|| panic!("scaffold.sh does not write {file}"))
}

fn verified_axum() -> String {
    let skill = read("skills/rust-domain-web/SKILL.md");
    frontmatter(&skill)
        .lines()
        .find_map(|l| l.trim().strip_prefix("forge-verified:"))
        .unwrap()
        .split(',')
        .find_map(|p| p.trim().strip_prefix("axum="))
        .unwrap_or_else(|| panic!("no axum in forge-verified"))
        .to_owned()
}

/// PRD 05 "Eval case" and "Deliver": prompt text and tags, Skill grader, llm rubric from the expected behaviour,
/// scaffold writes web-notes-api whose Cargo.toml declares axum at the verified version.
#[test]
fn domain_web_case_matches_prd() {
    let prompt = read("evals/domain-web/prompt.md");
    assert!(
        frontmatter(&prompt).contains("tags: [domain, web]"),
        "prompt tags"
    );
    assert_eq!(
        body(&prompt).trim(),
        "This repo is an axum service. Add POST /notes and GET /notes/{id} backed by SQLite through sqlx with a \
         migration, keep the query data offline-ready, and add router tests that do not bind a port."
    );

    let skill = read("evals/domain-web/graders/skill-fired.md");
    assert!(skill.contains("type: tool_used") && skill.contains("tool: Skill"));
    assert!(skill.contains("rust-domain-web"));

    let rubric = read("evals/domain-web/graders/web-behaviour.md");
    assert!(frontmatter(&rubric).contains("type: llm"));
    let rubric = flat(body(&rubric));
    assert!(
        rubric.contains(
            "adds handlers with typed extractors and an error type that maps to status codes, a sqlx migration \
             and offline query data, and tests via the router as a tower service"
        ),
        "rubric lacks the PRD expected behaviour"
    );
    for needle in ["IntoResponse", "migration", ".sqlx", "oneshot", "port"] {
        assert!(rubric.contains(needle), "rubric lacks {needle:?}");
    }

    let scaffold = read("evals/domain-web/scaffold.sh");
    let version = verified_axum();
    let toml = heredoc(&scaffold, "Cargo.toml");
    assert!(
        toml.contains("name = \"web-notes-api\""),
        "scaffold is not web-notes-api"
    );
    assert!(
        toml.lines()
            .any(|l| l.starts_with("axum =") && l.contains(&version)),
        "Cargo.toml does not declare axum {version}: {toml}"
    );
    assert!(toml.contains("\n[workspace]"), "no empty [workspace] table");
    let lock = heredoc(&scaffold, "Cargo.lock");
    assert!(
        lock.contains(&format!("name = \"axum\"\nversion = \"{version}\"")),
        "Cargo.lock does not pin axum {version}"
    );
}

/// PRD 05 "Rules for this skill", "Skill content", "Deliver" (line budget, README line, eval row), and the
/// sqlx-cli 0.9.1 README rule for where `cargo sqlx prepare` writes `.sqlx`.
#[test]
fn domain_web_skill_and_readme_follow_prd() {
    let text = read("skills/rust-domain-web/SKILL.md");
    assert!(text.lines().count() <= 150, "SKILL.md over 150 lines");
    let v = verified_axum();
    assert!(
        v.split('.').count() == 3 && v.split('.').all(|p| p.parse::<u64>().is_ok()),
        "axum={v} is not an exact version"
    );
    let desc = frontmatter(&text)
        .lines()
        .find_map(|l| l.strip_prefix("description: "))
        .unwrap();
    assert!((1..=1024).contains(&desc.chars().count()));
    let first = desc.split(". ").next().unwrap();
    assert!(
        first.contains("axum") && first.contains("tokio") && first.contains("tower"),
        "first sentence names no trigger: {first}"
    );
    assert!(desc.contains("rust-forge"), "no mention of the core skill");

    let body = body(&text);
    let first_section = flat(body.split("\n## ").nth(1).unwrap());
    assert!(
        first_section.contains("forge-tools domains")
            && first_section.contains("docs.rs")
            && first_section.contains("version drift"),
        "first step is not `forge-tools domains` + docs.rs + why: {first_section}"
    );
    let body = flat(body);
    for needle in [
        "Router::",
        "extractors",
        "AppState",
        "Arc<Mutex<_>>",
        "IntoResponse",
        "tower layers",
        "cargo sqlx prepare",
        "migrations",
        "with_graceful_shutdown",
        "tracing",
        "oneshot",
        "without binding a port",
    ] {
        assert!(body.contains(needle), "SKILL.md lacks {needle:?}");
    }
    // sqlx-cli 0.9.1 README: `prepare` writes `.sqlx` in the current directory; only `--workspace` puts a single
    // one at the workspace root.
    assert!(
        !body.contains("`.sqlx` directory at the workspace root.")
            && body.contains("cargo sqlx prepare --workspace"),
        "SKILL.md misstates where `cargo sqlx prepare` writes `.sqlx`"
    );
    for bad in ["TODO", "TBD", "[insert"] {
        assert!(!text.contains(bad), "SKILL.md contains {bad}");
    }

    let readme = read("README.md");
    let domains = readme.split("## Domains").nth(1).unwrap();
    assert!(domains.contains("`rust-domain-web`"));
    assert!(
        readme.lines().any(|l| l.starts_with("| `domain-web` |")
            && l.contains("--case domain-web --runs 1 --ablation none")
            && l.contains('$')),
        "README Evals has no domain-web row with score and cost"
    );
}
