//! Compact summary of cargo `--message-format=json` output.

use serde_json::Value;

const MAX_ERRORS: usize = 20;
const MAX_WARNINGS: usize = 10;
const RENDERED_ERRORS: usize = 3;

/// One compiler diagnostic, reduced to what the summary needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub is_error: bool,
    pub code: Option<String>,
    pub file: String,
    pub line: u64,
    pub column: u64,
    pub message: String,
    pub help: Option<String>,
    pub rendered: Option<String>,
}

/// Result of summarizing cargo JSON output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub text: String,
    pub errors: usize,
    pub warnings: usize,
}

impl Summary {
    /// Process exit code: 1 when any error was found.
    pub fn exit_code(&self) -> i32 {
        i32::from(self.errors > 0)
    }
}

fn parse_line(line: &str) -> Option<Diagnostic> {
    let value: Value = serde_json::from_str(line.trim()).ok()?;
    if value.get("reason")?.as_str()? != "compiler-message" {
        return None;
    }
    let msg = value.get("message")?;
    let is_error = match msg.get("level")?.as_str()? {
        "error" => true,
        "warning" => false,
        _ => return None,
    };
    let spans = msg.get("spans").and_then(Value::as_array);
    let primary = spans.and_then(|s| {
        s.iter()
            .find(|sp| sp.get("is_primary").and_then(Value::as_bool) == Some(true))
            .or_else(|| s.first())
    });
    let (file, line, column) = match primary {
        Some(sp) => (
            sp.get("file_name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_owned(),
            sp.get("line_start").and_then(Value::as_u64).unwrap_or(0),
            sp.get("column_start").and_then(Value::as_u64).unwrap_or(0),
        ),
        None => (String::new(), 0, 0),
    };
    let help = msg
        .get("children")
        .and_then(Value::as_array)
        .and_then(|c| {
            c.iter()
                .find(|ch| ch.get("level").and_then(Value::as_str) == Some("help"))
        })
        .and_then(|ch| ch.get("message").and_then(Value::as_str))
        .map(str::to_owned);
    Some(Diagnostic {
        is_error,
        code: msg
            .get("code")
            .and_then(|c| c.get("code"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        file,
        line,
        column,
        message: msg.get("message")?.as_str()?.to_owned(),
        help,
        rendered: msg
            .get("rendered")
            .and_then(Value::as_str)
            .map(str::to_owned),
    })
}

/// Parse, deduplicate and order diagnostics from cargo JSON lines.
pub fn collect(input: &str) -> Vec<Diagnostic> {
    let mut out: Vec<Diagnostic> = Vec::new();
    for diag in input.lines().filter_map(parse_line) {
        let dup = out.iter().any(|d| {
            d.code == diag.code
                && d.file == diag.file
                && d.line == diag.line
                && d.column == diag.column
                && d.message == diag.message
        });
        if !dup {
            out.push(diag);
        }
    }
    out.sort_by(|a, b| {
        b.is_error
            .cmp(&a.is_error)
            .then_with(|| a.file.cmp(&b.file))
            .then_with(|| a.line.cmp(&b.line))
            .then_with(|| a.column.cmp(&b.column))
    });
    out
}

fn format_one(d: &Diagnostic) -> String {
    let tag = match &d.code {
        Some(code) => code.clone(),
        None if d.is_error => "error".to_owned(),
        None => "warning".to_owned(),
    };
    let mut line = format!("{tag} {}:{}:{} {}", d.file, d.line, d.column, d.message);
    if let Some(help) = &d.help {
        line.push_str(" | help: ");
        line.push_str(help);
    }
    line
}

/// Build the compact summary from cargo JSON lines.
pub fn summarize(input: &str) -> Summary {
    let diags = collect(input);
    let errors: Vec<&Diagnostic> = diags.iter().filter(|d| d.is_error).collect();
    let warnings: Vec<&Diagnostic> = diags.iter().filter(|d| !d.is_error).collect();
    let mut text = String::new();
    for (i, d) in errors.iter().take(MAX_ERRORS).enumerate() {
        text.push_str(&format_one(d));
        text.push('\n');
        if i < RENDERED_ERRORS
            && let Some(rendered) = &d.rendered
        {
            for l in rendered.trim_end().lines() {
                text.push_str("    ");
                text.push_str(l);
                text.push('\n');
            }
        }
    }
    for d in warnings.iter().take(MAX_WARNINGS) {
        text.push_str(&format_one(d));
        text.push('\n');
    }
    let more_e = errors.len().saturating_sub(MAX_ERRORS);
    let more_w = warnings.len().saturating_sub(MAX_WARNINGS);
    if more_e > 0 || more_w > 0 {
        text.push_str(&format!(
            "... {more_e} more errors, {more_w} more warnings omitted\n"
        ));
    }
    text.push_str(&format!(
        "errors={} warnings={}\n",
        errors.len(),
        warnings.len()
    ));
    Summary {
        text,
        errors: errors.len(),
        warnings: warnings.len(),
    }
}
