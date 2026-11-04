#![expect(clippy::unwrap_used, reason = "tests")]

mod common;

use common::{TempDir, fixtures, run, stdout};

#[test]
fn set_strong_opus_changes_only_model_lines() {
    let dir = TempDir::new("models");
    let agents = dir.path().join("agents");
    std::fs::create_dir_all(&agents).unwrap();
    let names = [
        "crate-architect",
        "reviewer",
        "unsafe-auditor",
        "test-writer",
    ];
    for n in names {
        std::fs::copy(
            fixtures().join("agents").join(format!("{n}.md")),
            agents.join(format!("{n}.md")),
        )
        .unwrap();
    }
    let root = dir.path().to_str().unwrap();
    let out = run(
        &["models", "set", "--strong", "opus", "--plugin-root", root],
        dir.path(),
        "",
    );
    assert_eq!(out.status.code(), Some(0), "{}", stdout(&out));
    for n in names {
        let before =
            std::fs::read_to_string(fixtures().join("agents").join(format!("{n}.md"))).unwrap();
        let after = std::fs::read_to_string(agents.join(format!("{n}.md"))).unwrap();
        if n == "test-writer" {
            assert_eq!(before, after);
            continue;
        }
        let b: Vec<&str> = before.lines().collect();
        let a: Vec<&str> = after.lines().collect();
        assert_eq!(a.len(), b.len());
        let diffs: Vec<(&&str, &&str)> = b.iter().zip(&a).filter(|(x, y)| x != y).collect();
        assert_eq!(diffs, [(&"model: fable", &"model: opus")], "{n}");
        assert_eq!(before.ends_with('\n'), after.ends_with('\n'));
    }
}
