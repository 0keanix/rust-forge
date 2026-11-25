//! v-ml validation: behaviour PRD 04 specifies in its sections (Deliver, Skill content, Rules, Eval case)
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

/// Contents of the `cat > <file> <<'EOF'` heredoc in scaffold.sh.
fn heredoc<'a>(scaffold: &'a str, file: &str) -> &'a str {
    scaffold
        .split(&format!("cat > {file} <<'EOF'\n"))
        .nth(1)
        .and_then(|s| s.split("\nEOF\n").next())
        .unwrap_or_else(|| panic!("scaffold.sh does not write {file}"))
}

fn verified(krate: &str) -> String {
    let skill = read("skills/rust-domain-ml/SKILL.md");
    frontmatter(&skill)
        .lines()
        .find_map(|l| l.trim().strip_prefix("forge-verified:"))
        .unwrap()
        .split(',')
        .find_map(|p| p.trim().strip_prefix(&format!("{krate}=")))
        .unwrap_or_else(|| panic!("no {krate} in forge-verified"))
        .to_owned()
}

/// PRD 04 "Eval case" and "Deliver": prompt text and tags, Skill grader, llm rubric from the expected behaviour,
/// scaffold writes ml-embed whose Cargo.toml declares candle-core at the verified version.
#[test]
fn domain_ml_case_matches_prd() {
    let prompt = read("evals/domain-ml/prompt.md");
    assert!(
        frontmatter(&prompt).contains("tags: [domain, ml]"),
        "prompt tags"
    );
    assert_eq!(
        body(&prompt).trim(),
        "This repo is a CLI that should embed text. Add an embed command that loads a small sentence-embedding \
         model from safetensors with candle, uses Metal on macOS and falls back to CPU, and returns normalized \
         vectors. Tests must not download a model."
    );

    let skill = read("evals/domain-ml/graders/skill-fired.md");
    assert!(skill.contains("type: tool_used") && skill.contains("tool: Skill"));
    assert!(skill.contains("rust-domain-ml"));

    let rubric = read("evals/domain-ml/graders/ml-behaviour.md");
    assert!(frontmatter(&rubric).contains("type: llm"));
    for needle in [
        "Metal",
        "feature",
        "memory-maps",
        "normalizes",
        "tiny fixture model",
        "recorded tensors",
        "network",
    ] {
        assert!(body(&rubric).contains(needle), "rubric lacks {needle:?}");
    }

    let scaffold = read("evals/domain-ml/scaffold.sh");
    let version = verified("candle-core");
    let toml = heredoc(&scaffold, "Cargo.toml");
    assert!(
        toml.contains("name = \"ml-embed\""),
        "scaffold is not ml-embed"
    );
    assert!(
        toml.lines()
            .any(|l| l.starts_with("candle-core =") && l.contains(&version)),
        "Cargo.toml does not declare candle-core {version}: {toml}"
    );
    let lock = heredoc(&scaffold, "Cargo.lock");
    assert!(
        lock.contains(&format!("name = \"candle-core\"\nversion = \"{version}\"")),
        "Cargo.lock does not pin candle-core {version}"
    );
}

/// PRD 04 "Rules for this skill", "Skill content", "Deliver" (line budget, README line, eval row), and the
/// verified burn 0.22.1 default features (no backend enabled).
#[test]
fn domain_ml_skill_and_readme_follow_prd() {
    let text = read("skills/rust-domain-ml/SKILL.md");
    assert!(text.lines().count() <= 150, "SKILL.md over 150 lines");
    for krate in ["candle-core", "burn", "llama-cpp-2"] {
        let v = verified(krate);
        assert!(
            v.split('.').count() == 3 && v.split('.').all(|p| p.parse::<u64>().is_ok()),
            "{krate}={v} is not an exact version"
        );
    }
    let desc = frontmatter(&text)
        .lines()
        .find_map(|l| l.strip_prefix("description: "))
        .unwrap();
    assert!((1..=1024).contains(&desc.chars().count()));
    let first = desc.split(". ").next().unwrap();
    assert!(
        first.contains("candle") && first.contains("burn") && first.contains("llama"),
        "first sentence names no trigger: {first}"
    );
    assert!(desc.contains("rust-forge"), "no mention of the core skill");

    let body = body(&text);
    let first_section = body.split("\n## ").nth(1).unwrap();
    assert!(
        first_section.contains("forge-tools domains")
            && first_section.contains("docs.rs")
            && first_section.contains("version drift"),
        "first step is not `forge-tools domains` + docs.rs + why: {first_section}"
    );
    for needle in [
        "safetensors",
        "GGUF",
        "metal",
        "cuda",
        "Metal on Apple Silicon",
        "CPU fallback",
        "Memory-map",
        "tokenizer",
        "Batch",
        "tiny model",
        "fixture",
        "benches/",
    ] {
        assert!(body.contains(needle), "SKILL.md lacks {needle:?}");
    }
    assert!(
        body.contains("burn's default features enable\n  no backend")
            || body.contains("burn's default features enable no backend"),
        "SKILL.md must say burn 0.22.1 enables no backend by default"
    );
    for bad in ["TODO", "TBD", "[insert"] {
        assert!(!text.contains(bad), "SKILL.md contains {bad}");
    }

    let readme = read("README.md");
    let domains = readme.split("## Domains").nth(1).unwrap();
    assert!(domains.contains("`rust-domain-ml`"));
    assert!(
        readme.lines().any(|l| l.starts_with("| `domain-ml` |")
            && l.contains("--case domain-ml --runs 1 --ablation none")
            && l.contains('$')),
        "README Evals has no domain-ml row with score and cost"
    );
}
