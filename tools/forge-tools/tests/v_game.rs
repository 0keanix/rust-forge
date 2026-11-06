//! v-game validation: behaviour PRD 02 specifies in its sections (Deliver, Skill content, Rules, Eval case)
//! outside the D-1..D-6 list.
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

/// The bevy version in the skill's `metadata.forge-verified`.
fn verified_bevy() -> String {
    let skill = read("skills/rust-domain-game/SKILL.md");
    frontmatter(&skill)
        .lines()
        .find_map(|l| l.trim().strip_prefix("forge-verified:"))
        .unwrap()
        .split(',')
        .find_map(|p| p.trim().strip_prefix("bevy="))
        .unwrap()
        .to_owned()
}

/// PRD 02 "Eval case": prompt text, tags, Skill grader for rust-domain-game, llm rubric from the expected
/// behaviour, and the game-stamina scaffold.
#[test]
fn domain_game_case_matches_prd() {
    let prompt = read("evals/domain-game/prompt.md");
    assert!(
        frontmatter(&prompt)
            .lines()
            .any(|l| l.trim() == "tags: [domain, game]"),
        "{prompt}"
    );
    assert_eq!(
        body(&prompt).trim(),
        "This repo is a Bevy prototype with a moving player. Add stamina: sprinting drains it, it regenerates \
         after a short idle delay, and sprint is disabled at zero. Add headless tests for the drain and \
         regeneration rules."
    );

    let skill = read("evals/domain-game/graders/skill-fired.md");
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
        pattern.starts_with("\"skill\"") && pattern.ends_with("rust-domain-game\""),
        "grader does not target the rust-domain-game skill: {pattern}"
    );

    let rubric = read("evals/domain-game/graders/game-behaviour.md");
    assert!(frontmatter(&rubric).contains("type: llm"), "{rubric}");
    for needle in [
        "stamina component",
        "plugin",
        "explicit system ordering",
        "resource",
        "headless `App`",
        "no window or rendering",
        "update",
    ] {
        assert!(body(&rubric).contains(needle), "rubric lacks {needle:?}");
    }

    let scaffold = read("evals/domain-game/scaffold.sh");
    assert!(
        scaffold.contains("name = \"game-stamina\""),
        "scaffold is not game-stamina"
    );
}

/// PRD 02 "Deliver": scaffold.sh writes a Cargo.lock that pins bevy to the forge-verified version, and
/// Cargo.toml declares bevy.
#[test]
fn scaffold_lock_pins_verified_bevy() {
    let scaffold = read("evals/domain-game/scaffold.sh");
    let version = verified_bevy();
    let lock = scaffold
        .split("cat > Cargo.lock <<'EOF'\n")
        .nth(1)
        .and_then(|s| s.split("\nEOF\n").next())
        .unwrap();
    assert!(
        lock.contains(&format!("name = \"bevy\"\nversion = \"{version}\"")),
        "Cargo.lock does not pin bevy {version}: {lock}"
    );
    let toml = scaffold
        .split("cat > Cargo.toml <<'EOF'\n")
        .nth(1)
        .and_then(|s| s.split("\nEOF\n").next())
        .unwrap();
    assert!(
        toml.lines().any(|l| l.starts_with("bevy =")),
        "Cargo.toml does not declare bevy: {toml}"
    );
}

/// PRD 02 "Rules for this skill", "Skill content" and "Deliver" (reference file, README line, eval row).
#[test]
fn domain_game_skill_and_readme_follow_prd() {
    let text = read("skills/rust-domain-game/SKILL.md");
    let desc = frontmatter(&text)
        .lines()
        .find_map(|l| l.strip_prefix("description: "))
        .unwrap();
    let first = desc.split(". ").next().unwrap();
    assert!(
        first.contains("Bevy"),
        "first sentence names no trigger: {first}"
    );
    assert!(
        desc.contains("rust-forge"),
        "description does not mention the core skill"
    );

    let body = body(&text);
    let first_section = body.split("\n## ").nth(1).unwrap();
    assert!(
        first_section.contains("forge-tools domains") && first_section.contains("docs.rs"),
        "first step is not `forge-tools domains` + docs.rs: {first_section}"
    );

    // Skill content topics.
    for needle in [
        "Components",
        "Systems",
        "Resources",
        "Messages",
        "plugin",
        "ordering",
        "States",
        "dynamic_linking",
        "AssetServer",
        "MinimalPlugins",
        "migration guide",
        "one minor version at a time",
        "cargo check",
    ] {
        assert!(body.contains(needle), "SKILL.md lacks {needle:?}");
    }
    let dev = body.split("## 7.").nth(1).unwrap();
    assert!(
        dev.contains("must not reach a release"),
        "dynamic_linking is not restricted to dev builds"
    );

    // The reference is named with the condition for reading it and covers the four PRD topics.
    assert!(
        body.lines()
            .any(|l| l.contains("references/bevy-testing.md") && l.contains("before writing")),
        "SKILL.md does not name references/bevy-testing.md with a reading condition"
    );
    let reference = read("skills/rust-domain-game/references/bevy-testing.md");
    for heading in [
        "## Build a headless App",
        "## Advance time deterministically",
        "## Send input",
        "## Assert on state",
    ] {
        assert!(
            reference.contains(heading),
            "bevy-testing.md lacks {heading:?}"
        );
    }

    let readme = read("README.md");
    let domains = readme.split("## Domains").nth(1).unwrap();
    assert!(
        domains.contains("`rust-domain-game`"),
        "README Domains has no rust-domain-game line"
    );
    assert!(
        readme.lines().any(|l| l.starts_with("| `domain-game` |")
            && l.contains("--case domain-game --runs 1 --ablation none")
            && l.contains('$')),
        "README Evals has no domain-game row with score and cost"
    );
}
