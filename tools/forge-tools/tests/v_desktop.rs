//! v-desktop validation: behaviour PRD 06 specifies in its sections (Deliver, Skill content, Rules, Eval case)
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

fn verified_tauri() -> String {
    let skill = read("skills/rust-domain-desktop/SKILL.md");
    frontmatter(&skill)
        .lines()
        .find_map(|l| l.trim().strip_prefix("forge-verified:"))
        .unwrap()
        .split(',')
        .find_map(|p| p.trim().strip_prefix("tauri="))
        .unwrap_or_else(|| panic!("no tauri in forge-verified"))
        .to_owned()
}

/// PRD 06 "Eval case" and "Deliver": prompt text and tags, Skill grader, llm rubric from the expected behaviour,
/// scaffold writes desktop-notes whose Cargo.toml declares tauri at the verified version.
#[test]
fn domain_desktop_case_matches_prd() {
    let prompt = read("evals/domain-desktop/prompt.md");
    assert!(
        frontmatter(&prompt).contains("tags: [domain, desktop]"),
        "prompt tags"
    );
    assert_eq!(
        body(&prompt).trim(),
        "This repo is a workspace with a notes core crate and a stub Tauri 2 app crate. Expose list_notes and \
         add_note as Tauri commands with capabilities restricted to what those commands need, keeping the logic \
         testable without the app."
    );

    let skill = read("evals/domain-desktop/graders/skill-fired.md");
    assert!(skill.contains("type: tool_used") && skill.contains("tool: Skill"));
    assert!(skill.contains("rust-domain-desktop"));

    let rubric = read("evals/domain-desktop/graders/desktop-behaviour.md");
    assert!(frontmatter(&rubric).contains("type: llm"));
    let rubric = flat(body(&rubric));
    assert!(
        rubric.contains(
            "adds two commands in the app crate that delegate to the core, a capability file granting only what \
             is needed, and tests on the core crate; gates pass for the core"
        ),
        "rubric lacks the PRD expected behaviour"
    );
    for needle in [
        "list_notes",
        "add_note",
        "capabilities/",
        "no `tauri` dependency",
        "wildcard",
        "FAIL if",
    ] {
        assert!(rubric.contains(needle), "rubric lacks {needle:?}");
    }

    let scaffold = read("evals/domain-desktop/scaffold.sh");
    let version = verified_tauri();
    let app = heredoc(&scaffold, "src-tauri/Cargo.toml");
    assert!(
        app.contains("name = \"desktop-notes\""),
        "scaffold is not desktop-notes"
    );
    assert!(
        app.lines()
            .any(|l| l.starts_with("tauri =") && l.contains(&version)),
        "desktop-notes does not declare tauri {version}: {app}"
    );
    let core = heredoc(&scaffold, "notes-core/Cargo.toml");
    assert!(!core.contains("tauri"), "core crate depends on tauri");
    let lock = heredoc(&scaffold, "Cargo.lock");
    assert!(
        lock.contains(&format!("name = \"tauri\"\nversion = \"{version}\"")),
        "Cargo.lock does not pin tauri {version}"
    );
    // The stub app must not already expose the commands, or a run that does nothing would satisfy the rubric.
    let main = heredoc(&scaffold, "src-tauri/src/main.rs");
    assert!(!main.contains("generate_handler") && !main.contains("tauri::command"));
    assert!(
        !scaffold.contains("capabilities"),
        "scaffold already grants"
    );
}

/// PRD 06 "Rules for this skill", "Skill content", "Deliver" (line budget, README line, eval row), and the API
/// facts checked against tauri 2.12.7 / tauri-build 2.7.6 sources.
#[test]
fn domain_desktop_skill_and_readme_follow_prd() {
    let text = read("skills/rust-domain-desktop/SKILL.md");
    assert!(text.lines().count() <= 150, "SKILL.md over 150 lines");
    assert!(
        !repo()
            .join("skills/rust-domain-desktop/references")
            .exists(),
        "references/ added although SKILL.md fits in 150 lines"
    );
    let v = verified_tauri();
    assert!(
        v.split('.').count() == 3 && v.split('.').all(|p| p.parse::<u64>().is_ok()),
        "tauri={v} is not an exact version"
    );
    let desc = frontmatter(&text)
        .lines()
        .find_map(|l| l.strip_prefix("description: "))
        .unwrap();
    assert!((1..=1024).contains(&desc.chars().count()));
    let first = desc.split(". ").next().unwrap();
    assert!(
        first.contains("Tauri") && first.contains("desktop"),
        "first sentence names no trigger: {first}"
    );
    assert!(desc.contains("tauri"), "description names no crate");
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
        "#[tauri::command]",
        "tauri::generate_handler!",
        "capabilities",
        "least privilege",
        "allow-<command>",
        "Builder::manage",
        "tauri::State<'_, T>",
        "no `tauri` dependency",
        "WebKitGTK",
        "WebView2",
        "Xcode",
        "tauri::test::mock_builder",
        "without a window",
    ] {
        assert!(body.contains(needle), "SKILL.md lacks {needle:?}");
    }
    // tauri 2.12.7 `Builder::invoke_handler` assigns `self.invoke_handler`; a second call replaces the first.
    assert!(body.contains("A second `invoke_handler` call replaces the first"));
    // tauri-macros 2.7.6: the default argument case is camelCase.
    assert!(body.contains("camelCase by default"));
    for bad in ["TODO", "TBD", "[insert"] {
        assert!(!text.contains(bad), "SKILL.md contains {bad}");
    }

    let readme = read("README.md");
    let domains = readme.split("## Domains").nth(1).unwrap();
    assert!(domains.contains("`rust-domain-desktop`"));
    assert!(
        readme.lines().any(|l| l.starts_with("| `domain-desktop` |")
            && l.contains("--case domain-desktop --runs 1 --ablation none")
            && l.contains('$')),
        "README Evals has no domain-desktop row with score and cost"
    );
}
