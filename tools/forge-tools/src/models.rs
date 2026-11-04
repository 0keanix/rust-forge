//! Switch the model tier of the strong agents.

use std::path::Path;

use anyhow::{Context, Result, bail};

/// Agents in the strong tier.
pub const STRONG_AGENTS: &[&str] = &["crate-architect", "reviewer", "unsafe-auditor"];

/// Strong-tier model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Strong {
    Fable,
    Opus,
}

impl Strong {
    pub fn as_str(self) -> &'static str {
        match self {
            Strong::Fable => "fable",
            Strong::Opus => "opus",
        }
    }
}

/// Replace the `model:` value inside the leading frontmatter of `text`.
pub fn rewrite(text: &str, model: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut fences = 0;
    let mut done = false;
    for line in text.split_inclusive('\n') {
        let body = line.trim_end_matches(['\n', '\r']);
        if body.trim_end() == "---" {
            fences += 1;
        } else if fences == 0 {
            return None;
        }
        if fences == 1 && !done && body.starts_with("model:") {
            out.push_str("model: ");
            out.push_str(model);
            out.push_str(&line[body.len()..]);
            done = true;
        } else {
            out.push_str(line);
        }
    }
    done.then_some(out)
}

/// Set the strong-tier model in `<plugin_root>/agents/<agent>.md`.
pub fn set(plugin_root: &Path, strong: Strong) -> Result<String> {
    let mut report = String::new();
    for agent in STRONG_AGENTS {
        let path = plugin_root.join("agents").join(format!("{agent}.md"));
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let Some(new) = rewrite(&text, strong.as_str()) else {
            bail!("{}: no model: line in frontmatter", path.display());
        };
        if new != text {
            std::fs::write(&path, new).with_context(|| format!("writing {}", path.display()))?;
        }
        report.push_str(&format!("{agent}: model: {}\n", strong.as_str()));
    }
    Ok(report)
}
