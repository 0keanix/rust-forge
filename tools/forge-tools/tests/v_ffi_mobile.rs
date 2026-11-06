//! v-ffi-mobile validation: behaviour PRD 03 specifies in its sections (Deliver, Skill content, Rules, Eval case)
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

fn verified_uniffi() -> String {
    let skill = read("skills/rust-domain-ffi-mobile/SKILL.md");
    frontmatter(&skill)
        .lines()
        .find_map(|l| l.trim().strip_prefix("forge-verified:"))
        .unwrap()
        .split(',')
        .find_map(|p| p.trim().strip_prefix("uniffi="))
        .unwrap()
        .to_owned()
}

/// PRD 03 "Eval case" and "Deliver": prompt text and tags, Skill grader, llm rubric from the expected behaviour,
/// scaffold writes ffi-notes-core whose Cargo.toml declares uniffi at the verified version.
#[test]
fn domain_ffi_mobile_case_matches_prd() {
    let prompt = read("evals/domain-ffi-mobile/prompt.md");
    assert!(
        frontmatter(&prompt).contains("tags: [domain, ffi-mobile]"),
        "prompt tags"
    );
    assert_eq!(
        body(&prompt).trim(),
        "This repo is a workspace with a pure Rust notes core crate and an empty ffi crate that already depends \
         on uniffi. Expose NoteStore (add, list, search) to Kotlin and Swift through the ffi crate, and make it \
         build for aarch64-linux-android and aarch64-apple-ios."
    );

    let skill = read("evals/domain-ffi-mobile/graders/skill-fired.md");
    assert!(skill.contains("type: tool_used") && skill.contains("tool: Skill"));
    assert!(skill.contains("rust-domain-ffi-mobile"));

    let rubric = read("evals/domain-ffi-mobile/graders/ffi-behaviour.md");
    assert!(frontmatter(&rubric).contains("type: llm"));
    for needle in [
        "thin layer",
        "error type",
        "host tests",
        "Android",
        "iOS",
        "build commands",
    ] {
        assert!(body(&rubric).contains(needle), "rubric lacks {needle:?}");
    }

    let scaffold = read("evals/domain-ffi-mobile/scaffold.sh");
    let version = verified_uniffi();
    let ffi_toml = heredoc(&scaffold, "notes-ffi/Cargo.toml");
    assert!(
        ffi_toml
            .lines()
            .any(|l| l.starts_with("uniffi =") && l.contains(&version)),
        "notes-ffi does not declare uniffi {version}: {ffi_toml}"
    );
    let core_toml = heredoc(&scaffold, "notes-core/Cargo.toml");
    assert!(
        !core_toml.contains("uniffi"),
        "core crate must stay free of FFI"
    );
    let lock = heredoc(&scaffold, "Cargo.lock");
    assert!(
        lock.contains(&format!("name = \"uniffi\"\nversion = \"{version}\"")),
        "Cargo.lock does not pin uniffi {version}"
    );
    let core = heredoc(&scaffold, "notes-core/src/lib.rs");
    for f in ["pub fn add", "pub fn list", "pub fn search", "#[test]"] {
        assert!(core.contains(f), "notes core lacks {f:?}");
    }
}

/// PRD 03 "Rules for this skill", "Skill content", "Deliver" (line budget, README line, eval row), and the
/// verified uniffi 0.32 bindgen CLI (`--library` is deprecated there).
#[test]
fn domain_ffi_mobile_skill_and_readme_follow_prd() {
    let text = read("skills/rust-domain-ffi-mobile/SKILL.md");
    assert!(text.lines().count() <= 150, "SKILL.md over 150 lines");
    let desc = frontmatter(&text)
        .lines()
        .find_map(|l| l.strip_prefix("description: "))
        .unwrap();
    assert!((1..=1024).contains(&desc.chars().count()));
    let first = desc.split(". ").next().unwrap();
    assert!(
        first.contains("uniffi") && first.contains("cbindgen"),
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
        "proc-macro",
        "UDL",
        "cbindgen",
        "cargo-ndk",
        "XCFramework",
        "core crate",
        "own unit tests",
        "Errors across the boundary",
        "Ownership across the boundary",
        "On the host",
        "On a device or simulator",
    ] {
        assert!(body.contains(needle), "SKILL.md lacks {needle:?}");
    }
    assert!(
        !body.contains("--library <"),
        "SKILL.md recommends `--library`, deprecated in uniffi 0.32"
    );
    for bad in ["TODO", "TBD", "[insert"] {
        assert!(!text.contains(bad), "SKILL.md contains {bad}");
    }

    let readme = read("README.md");
    let domains = readme.split("## Domains").nth(1).unwrap();
    assert!(domains.contains("`rust-domain-ffi-mobile`"));
    assert!(
        readme
            .lines()
            .any(|l| l.starts_with("| `domain-ffi-mobile` |")
                && l.contains("--case domain-ffi-mobile --runs 1 --ablation none")
                && l.contains('$')),
        "README Evals has no domain-ffi-mobile row with score and cost"
    );
}
