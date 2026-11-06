//! v-cli validation: behaviour PRD 01 specifies outside its D-1..D-5 list, plus the G4 change made during
//! the cli stage (2c58cea).
#![expect(clippy::unwrap_used, reason = "tests")]

mod common;

use std::path::{Path, PathBuf};

use common::{TempDir, run, stdout, write_crate};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo().join(rel)).unwrap()
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

/// 2c58cea skips G4 only without a library target; a library crate must still run doctests.
#[test]
fn lib_crate_still_runs_doctests() {
    let dir = TempDir::new("vcli-doctest");
    write_crate(
        dir.path(),
        "/// Adds one.\n///\n/// ```\n/// assert_eq!(fixture::add_one(1), 3);\n/// ```\npub fn add_one(x: u32) -> u32 {\n    x + 1\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn adds() {\n        assert_eq!(super::add_one(1), 2);\n    }\n}\n",
    );
    let out = run(&["gates", "full"], dir.path(), "");
    let text = stdout(&out);
    assert_ne!(out.status.code(), Some(0), "{text}");
    assert!(text.contains("FAIL G4-doctest"), "{text}");
    assert!(!text.contains("SKIPPED G4-doctest"), "{text}");
}

/// PRD 01 "Eval case": prompt text, tags, Skill grader for rust-domain-cli, llm rubric from the expected behaviour.
#[test]
fn domain_cli_case_matches_prd() {
    let prompt = read("evals/domain-cli/prompt.md");
    assert!(
        frontmatter(&prompt)
            .lines()
            .any(|l| l.trim() == "tags: [domain, cli]"),
        "{prompt}"
    );
    assert_eq!(
        body(&prompt).trim(),
        "This repo is a small notes CLI on clap. Add an export subcommand that writes all notes as JSON or CSV, \
         with the default format taken from a config file in the user config dir and overridable by a flag. \
         Cover it with tests that run the binary."
    );

    let skill = read("evals/domain-cli/graders/skill-fired.md");
    let fm = frontmatter(&skill);
    assert!(fm.contains("type: tool_used"), "{skill}");
    assert!(fm.contains("tool: Skill"), "{skill}");
    assert!(
        !fm.contains("max:"),
        "positive case must not cap the skill: {skill}"
    );
    let pattern = fm
        .lines()
        .find_map(|l| l.strip_prefix("input_match: "))
        .unwrap()
        .trim_matches('\'');
    assert!(
        pattern.starts_with("\"skill\"") && pattern.ends_with("rust-domain-cli\""),
        "grader does not target the rust-domain-cli skill: {pattern}"
    );

    let rubric = read("evals/domain-cli/graders/cli-behaviour.md");
    assert!(frontmatter(&rubric).contains("type: llm"), "{rubric}");
    for needle in [
        "clap derive",
        "value enum",
        "flag overrides the config file",
        "non-zero",
        "assert_cmd",
    ] {
        assert!(body(&rubric).contains(needle), "rubric lacks {needle:?}");
    }

    let scaffold = read("evals/domain-cli/scaffold.sh");
    assert!(
        scaffold.contains("name = \"cli-notes\""),
        "scaffold is not cli-notes"
    );
}

/// PRD 01 "Rules for this skill" and "Deliver" (README line, eval row).
#[test]
fn domain_cli_skill_and_readme_follow_prd() {
    let text = read("skills/rust-domain-cli/SKILL.md");
    let desc = frontmatter(&text)
        .lines()
        .find_map(|l| l.strip_prefix("description: "))
        .unwrap();
    let first = desc.split(". ").next().unwrap();
    assert!(
        first.contains("clap"),
        "first sentence names no trigger: {first}"
    );
    assert!(
        desc.contains("rust-forge"),
        "description does not mention the core skill"
    );

    let first_section = body(&text).split("\n## ").nth(1).unwrap();
    assert!(
        first_section.contains("forge-tools domains") && first_section.contains("docs.rs"),
        "first step is not `forge-tools domains` + docs.rs: {first_section}"
    );

    let readme = read("README.md");
    let domains = readme.split("## Domains").nth(1).unwrap();
    assert!(
        domains.contains("`rust-domain-cli`"),
        "README Domains has no rust-domain-cli line"
    );
    assert!(
        readme.lines().any(|l| l.starts_with("| `domain-cli` |")
            && l.contains("--case domain-cli --runs 1 --ablation none")
            && l.contains('$')),
        "README Evals has no domain-cli row with score and cost"
    );
}
