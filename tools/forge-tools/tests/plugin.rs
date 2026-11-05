//! Plugin-level checks: manifests, agents, skills, eval suite and scaffolds (PRD 00 AC-9 … AC-12).
//! Written to be reused unchanged by later stages.
#![expect(clippy::unwrap_used, clippy::panic, reason = "tests")]

mod common;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use common::{TempDir, run, stdout};
use forge_tools::domains::MAPPING;

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .unwrap()
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

/// Non-hidden entries of `dir`, sorted. Hidden entries (such as tool state directories) are not
/// plugin content.
fn sorted_entries(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|rd| {
            rd.map(|e| e.unwrap().path())
                .filter(|p| !p.file_name().unwrap().to_string_lossy().starts_with('.'))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

fn files_under(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for p in sorted_entries(dir) {
        if p.is_dir() {
            out.extend(files_under(&p));
        } else {
            out.push(p);
        }
    }
    out
}

/// Minimal YAML frontmatter: top-level key → raw value; indented lines become `parent.child` keys,
/// and block scalars (`>`, `|`, or an empty value followed by indented text) are joined.
fn frontmatter(text: &str) -> Option<BTreeMap<String, String>> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;
    Some(yaml_map(&rest[..end]))
}

fn yaml_map(block: &str) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    let mut parent: Option<String> = None;
    for line in block.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let indented = line.starts_with(' ') || line.starts_with('\t');
        if !indented {
            let Some((k, v)) = line.split_once(':') else {
                continue;
            };
            let k = k.trim().to_owned();
            map.insert(k.clone(), unquote(v.trim()));
            parent = Some(k);
            continue;
        }
        let Some(p) = parent.clone() else { continue };
        let t = line.trim();
        let value = map.get(&p).cloned().unwrap_or_default();
        if let Some((k, v)) = t.split_once(':').filter(|(k, _)| is_key(k)) {
            map.insert(format!("{p}.{}", k.trim()), unquote(v.trim()));
        } else if value.is_empty() || value == ">" || value == "|" || !value.contains(':') {
            let joined = if value.is_empty() || value == ">" || value == "|" {
                t.to_owned()
            } else {
                format!("{value} {t}")
            };
            map.insert(p, joined);
        }
    }
    map
}

fn is_key(k: &str) -> bool {
    !k.is_empty()
        && k.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn unquote(v: &str) -> String {
    let v = v.trim();
    for q in ['"', '\''] {
        if v.len() >= 2 && v.starts_with(q) && v.ends_with(q) {
            return v[1..v.len() - 1].to_owned();
        }
    }
    v.to_owned()
}

fn list(v: &str) -> Vec<String> {
    v.trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|s| unquote(s.trim()))
        .filter(|s| !s.is_empty())
        .collect()
}

fn parse_json(p: &Path) -> serde_json::Value {
    serde_json::from_str(&read(p)).unwrap_or_else(|e| panic!("{}: {e}", p.display()))
}

#[test]
fn manifests_parse_without_version() {
    let root = repo();
    for f in ["plugin.json", "marketplace.json"] {
        let v = parse_json(&root.join(".claude-plugin").join(f));
        assert!(v.get("version").is_none(), "{f} must not set version");
        if let Some(plugins) = v["plugins"].as_array() {
            for p in plugins {
                assert!(p.get("version").is_none(), "{f} plugin entry sets version");
            }
        }
    }
    let hooks = parse_json(&root.join("hooks").join("hooks.json"));
    assert!(hooks["hooks"].is_object());
}

#[test]
fn agents_have_required_frontmatter() {
    let models = ["fable", "opus", "sonnet", "haiku", "inherit"];
    let agents = sorted_entries(&repo().join("agents"));
    assert!(!agents.is_empty());
    for a in agents {
        let fm =
            frontmatter(&read(&a)).unwrap_or_else(|| panic!("{}: no frontmatter", a.display()));
        for key in ["name", "description", "tools", "model"] {
            assert!(
                fm.get(key).is_some_and(|v| !v.is_empty()),
                "{}: missing {key}",
                a.display()
            );
        }
        assert_eq!(
            fm["name"],
            a.file_stem().unwrap().to_str().unwrap(),
            "{}",
            a.display()
        );
        assert!(models.contains(&fm["model"].as_str()), "{}", a.display());
    }
}

fn domain_crates(domain: &str) -> &'static [&'static str] {
    MAPPING
        .iter()
        .find(|(d, _)| *d == domain)
        .map(|(_, c)| *c)
        .unwrap_or(&[])
}

fn valid_version(v: &str) -> bool {
    let core = v.split(['-', '+']).next().unwrap_or("");
    let parts: Vec<&str> = core.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

#[test]
fn skills_follow_spec() {
    let skills = sorted_entries(&repo().join("skills"));
    assert!(!skills.is_empty());
    for dir in skills.iter().filter(|p| p.is_dir()) {
        let dir_name = dir.file_name().unwrap().to_str().unwrap();
        let text = read(&dir.join("SKILL.md"));
        let fm = frontmatter(&text).unwrap_or_else(|| panic!("{dir_name}: no frontmatter"));
        assert_eq!(fm.get("name").map(String::as_str), Some(dir_name));
        assert!(dir_name.len() <= 64);
        assert!(
            dir_name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        );
        let desc = fm.get("description").cloned().unwrap_or_default();
        assert!(
            (1..=1024).contains(&desc.chars().count()),
            "{dir_name}: description length {}",
            desc.chars().count()
        );
        for f in files_under(&dir.join("references")) {
            let rel = f
                .strip_prefix(dir)
                .unwrap()
                .to_str()
                .unwrap()
                .replace('\\', "/");
            assert!(
                text.contains(&rel),
                "{dir_name}: SKILL.md does not mention {rel}"
            );
        }
        let lines = text.lines().count();
        if dir_name == "rust-forge" {
            assert!(lines <= 200, "rust-forge SKILL.md has {lines} lines");
        }
        if let Some(domain) = dir_name.strip_prefix("rust-domain-") {
            assert!(lines <= 150, "{dir_name} SKILL.md has {lines} lines");
            let verified = fm
                .get("metadata.forge-verified")
                .unwrap_or_else(|| panic!("{dir_name}: no metadata.forge-verified"));
            let key = domain_crates(domain);
            assert!(!key.is_empty(), "{dir_name}: unknown domain");
            for pair in verified.split(',') {
                let (krate, version) = pair
                    .split_once('=')
                    .unwrap_or_else(|| panic!("{dir_name}: bad forge-verified {verified}"));
                assert!(
                    valid_version(version),
                    "{dir_name}: inexact version {version}"
                );
                assert!(
                    key.contains(&krate),
                    "{dir_name}: {krate} is not a key crate"
                );
            }
        }
    }
}

#[test]
fn no_placeholders() {
    let root = repo();
    let mut files = vec![root.join("README.md")];
    for d in [
        ".claude-plugin",
        "agents",
        "commands",
        "hooks",
        "skills",
        "evals",
    ] {
        let dir = root.join(d);
        files.extend(
            files_under(&dir)
                .into_iter()
                .filter(|p| !p.starts_with(dir.join("results"))),
        );
    }
    for f in files {
        let Ok(text) = std::fs::read_to_string(&f) else {
            continue;
        };
        for bad in ["TODO", "TBD", "[insert"] {
            assert!(!text.contains(bad), "{} contains {bad}", f.display());
        }
    }
}

const PROMPT_KEYS: &[&str] = &[
    "schema_version",
    "name",
    "description",
    "tags",
    "plugins",
    "runs",
    "expected_outcome",
    "model",
    "max_turns",
    "timeout_seconds",
    "allowed_tools",
    "append_system_prompt",
    "env",
];
const GRADER_TYPES: &[&str] = &[
    "regex",
    "tool_used",
    "tool_order",
    "file_exists",
    "llm",
    "baseline",
];

#[test]
fn eval_suite_is_well_formed() {
    let root = repo();
    let evals = root.join("evals");
    let skills: BTreeSet<String> = sorted_entries(&root.join("skills"))
        .iter()
        .filter(|p| p.is_dir())
        .map(|p| p.file_name().unwrap().to_str().unwrap().to_owned())
        .collect();
    let (mut positive, mut negative) = (0, 0);
    for case in sorted_entries(&evals).into_iter().filter(|p| p.is_dir()) {
        let name = case.file_name().unwrap().to_str().unwrap().to_owned();
        if name == "results" {
            continue;
        }
        let prompt = case.join("prompt.md");
        let yaml = case.join("case.yaml");
        if !prompt.exists() && !yaml.exists() {
            continue;
        }
        let mut tags = Vec::new();
        if prompt.exists() {
            let fm = frontmatter(&read(&prompt)).unwrap_or_default();
            for k in fm.keys().filter(|k| !k.contains('.')) {
                assert!(
                    PROMPT_KEYS.contains(&k.as_str()),
                    "{name}: prompt.md key {k}"
                );
            }
            if let Some(t) = fm.get("tags") {
                tags.extend(list(t));
            }
        }
        if yaml.exists() {
            let y = yaml_map(&read(&yaml));
            if let Some(t) = y.get("tags") {
                tags.extend(list(t));
            }
            if let Some(script) = y.get("context.scaffold_script") {
                let path = case.join(script);
                assert!(path.is_file(), "{name}: missing {script}");
                let ok = Command::new("bash").arg("-n").arg(&path).status().unwrap();
                assert!(ok.success(), "{name}: bash -n {script} failed");
            }
        }
        let kinds: Vec<&String> = tags
            .iter()
            .filter(|t| ["core", "trigger", "domain"].contains(&t.as_str()))
            .collect();
        assert_eq!(kinds.len(), 1, "{name}: tags {tags:?}");
        let graders: Vec<PathBuf> = sorted_entries(&case.join("graders"))
            .into_iter()
            .filter(|p| p.extension().is_some_and(|e| e == "md"))
            .collect();
        assert!(!graders.is_empty(), "{name}: no graders");
        let mut skill_counts = Vec::new();
        for g in graders {
            let fm =
                frontmatter(&read(&g)).unwrap_or_else(|| panic!("{}: no frontmatter", g.display()));
            let ty = fm.get("type").map(String::as_str).unwrap_or("");
            assert!(GRADER_TYPES.contains(&ty), "{}: type {ty}", g.display());
            if ty == "tool_used" && fm.get("tool").map(String::as_str) == Some("Skill") {
                let m = fm.get("input_match").cloned().unwrap_or_default();
                assert!(
                    skills.iter().any(|s| m.contains(s.as_str())),
                    "{}: names no existing skill",
                    g.display()
                );
                skill_counts.push(fm.get("max").cloned());
            }
        }
        if kinds[0] == "trigger" {
            if skill_counts.iter().any(|m| m.as_deref() == Some("0")) {
                negative += 1;
            } else if !skill_counts.is_empty() {
                positive += 1;
            }
        }
    }
    assert!(positive >= 3, "positive trigger cases: {positive}");
    assert!(negative >= 2, "negative trigger cases: {negative}");
}

#[test]
fn models_set_on_real_agents_changes_only_strong_model_lines() {
    let dir = TempDir::new("plugin-models");
    let src = repo().join("agents");
    let dst = dir.path().join("agents");
    std::fs::create_dir_all(&dst).unwrap();
    let agents = sorted_entries(&src);
    for a in &agents {
        std::fs::copy(a, dst.join(a.file_name().unwrap())).unwrap();
    }
    let root = dir.path().to_str().unwrap();
    let out = run(
        &["models", "set", "--strong", "opus", "--plugin-root", root],
        dir.path(),
        "",
    );
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    for a in &agents {
        let name = a.file_stem().unwrap().to_str().unwrap();
        let before = read(a);
        let after = read(&dst.join(a.file_name().unwrap()));
        if !["crate-architect", "reviewer", "unsafe-auditor"].contains(&name) {
            assert_eq!(before, after, "{name}");
            continue;
        }
        let b: Vec<&str> = before.lines().collect();
        let a: Vec<&str> = after.lines().collect();
        assert_eq!(a.len(), b.len());
        let diffs: Vec<(&&str, &&str)> = b.iter().zip(&a).filter(|(x, y)| x != y).collect();
        assert_eq!(diffs, [(&"model: fable", &"model: opus")], "{name}");
        assert_eq!(before.ends_with('\n'), after.ends_with('\n'));
    }
}

fn scaffold(case: &str, dir: &Path) {
    let script = repo().join("evals").join(case).join("scaffold.sh");
    let out = Command::new("bash")
        .arg(&script)
        .current_dir(dir)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("HOME", dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{case}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn cargo_check(dir: &Path) -> std::process::Output {
    Command::new("cargo")
        .args(["check", "--offline", "--message-format=short"])
        .current_dir(dir)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_BUILD_TARGET_DIR")
        .output()
        .unwrap()
}

#[test]
fn borrow_fix_scaffold_fails_with_three_error_codes() {
    let dir = TempDir::new("scaffold-borrow");
    let ws = dir.path().join("ws");
    std::fs::create_dir_all(&ws).unwrap();
    scaffold("core-borrow-fix", &ws);
    let out = cargo_check(&ws);
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    let codes: BTreeSet<&str> = stderr
        .match_indices("error[E")
        .map(|(i, _)| &stderr[i + 6..i + 11])
        .collect();
    assert!(codes.len() >= 3, "codes {codes:?}\n{stderr}");
}

#[test]
fn new_dependency_scaffold_compiles() {
    let dir = TempDir::new("scaffold-dep");
    let ws = dir.path().join("ws");
    std::fs::create_dir_all(&ws).unwrap();
    scaffold("core-new-dependency", &ws);
    let out = cargo_check(&ws);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
