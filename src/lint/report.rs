//! Lint report renderers — the grouped-by-file human report and the
//! hand-rolled JSON report (SPEC-052, DEC-050).
//!
//! The JSON report is **hand-rolled**, matching the shipped
//! `write_json`/`write_diff_json`/`ExplainTrace::write_json` pattern — no
//! `serde_json` runtime dependency (constraint `no-new-top-level-deps-without-decision`).
//! Determinism (SPEC-049's discipline): findings are already sorted by
//! `(path, severity, rule)`, strings are escaped, integers are exact, and there
//! is no wall-clock — so a synthetic-`LintOutcome` golden test is byte-stable
//! across the 3-OS CI. The schema id `crustyimg.lint/v1` is part of DEC-050's
//! stability surface.

use std::io::{self, Write};
use std::path::Path;

use super::{Finding, LintOutcome, Severity};

/// Render the outcome as a grouped-by-file human report (eslint/ruff style).
///
/// Findings are grouped by file (already globally sorted by `(path, severity,
/// rule)`), each line carrying its severity, rule id, and message, and — on the
/// next line — the runnable `crustyimg` fix. Ends with a summary line (counts +
/// total potential savings when any). Writes to `out` (stdout, so `-o -` stays
/// unaffected).
pub fn render_human(outcome: &LintOutcome, out: &mut impl Write) -> io::Result<()> {
    let mut current: Option<&Path> = None;
    for finding in &outcome.findings {
        if current != Some(finding.file()) {
            if current.is_some() {
                writeln!(out)?;
            }
            writeln!(out, "{}", finding.file().display())?;
            current = Some(finding.file());
        }
        writeln!(
            out,
            "  {} {}: {}",
            finding.severity().label(),
            finding.rule(),
            finding.message(),
        )?;
        if let Some(cmd) = finding.fix_command() {
            writeln!(out, "    fix: {cmd}")?;
        }
    }
    if !outcome.findings.is_empty() {
        writeln!(out)?;
    }
    let saved = outcome.potential_bytes_saved();
    if saved > 0 {
        writeln!(
            out,
            "{} scanned · {} error · {} warn · {} info · ~{} bytes saveable",
            outcome.files_scanned,
            outcome.error_count(),
            outcome.warn_count(),
            outcome.info_count(),
            saved,
        )
    } else {
        writeln!(
            out,
            "{} scanned · {} error · {} warn · {} info",
            outcome.files_scanned,
            outcome.error_count(),
            outcome.warn_count(),
            outcome.info_count(),
        )
    }
}

/// Emit the outcome as a single-line, hand-rolled JSON object to `out` (no
/// trailing newline; the caller adds one).
///
/// Schema (`crustyimg.lint/v1`): `{schema, findings[{file, rule, severity,
/// message, fix, bytes_saved?}], summary{files_scanned, errors, warnings, infos,
/// potential_bytes_saved, passed}}`. `fix` is the full runnable command
/// (`crustyimg <fix> <file>`) or `null`; `bytes_saved` is omitted when absent.
/// `passed` is the CI gate result the caller computed (no `Error`, warnings
/// within `--max-warnings`), so the output format never changes the exit code.
pub fn write_json(outcome: &LintOutcome, passed: bool, out: &mut impl Write) -> io::Result<()> {
    write!(out, "{{\"schema\":\"crustyimg.lint/v1\",\"findings\":[")?;
    for (i, f) in outcome.findings.iter().enumerate() {
        if i > 0 {
            write!(out, ",")?;
        }
        write_finding(f, out)?;
    }
    write!(
        out,
        "],\"summary\":{{\"files_scanned\":{},\"errors\":{},\"warnings\":{},\"infos\":{},\
         \"potential_bytes_saved\":{},\"passed\":{}}}}}",
        outcome.files_scanned,
        outcome.error_count(),
        outcome.warn_count(),
        outcome.info_count(),
        outcome.potential_bytes_saved(),
        passed,
    )
}

/// Write one finding object.
fn write_finding(f: &Finding, out: &mut impl Write) -> io::Result<()> {
    write!(
        out,
        "{{\"file\":\"{}\",\"rule\":\"{}\",\"severity\":\"{}\",\"message\":\"{}\",\"fix\":",
        escape_json(&f.file().display().to_string()),
        escape_json(f.rule()),
        f.severity().label(),
        escape_json(f.message()),
    )?;
    match f.fix_command() {
        Some(cmd) => write!(out, "\"{}\"", escape_json(&cmd))?,
        None => write!(out, "null")?,
    }
    if let Some(bytes) = f.bytes_saved() {
        write!(out, ",\"bytes_saved\":{bytes}")?;
    }
    write!(out, "}}")
}

/// Escape a string for a hand-rolled JSON value (mirrors the shipped
/// `escape_json`): `"` → `\"`, `\` → `\\`, control chars < 0x20 → `\u00XX`.
///
/// `pub(crate)` only so `cli::tests::escape_json_impls_are_equivalent`
/// (SPEC-097) can prove this is byte-identical to `cli::escape_json` before
/// the two are merged into one shared helper; not part of the public API.
pub(crate) fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

// ── SARIF report (SPEC-056) ───────────────────────────────────────────────────

/// crustyimg's repo — the SARIF `informationUri` / rule `helpUri`.
const CRUSTYIMG_URI: &str = "https://github.com/jysf/crustyimg";

/// Emit the outcome as a single-line, hand-rolled **SARIF 2.1.0** object (no
/// trailing newline; the caller adds one) for GitHub code-scanning
/// (`github/codeql-action/upload-sarif`).
///
/// One `run` with the `crustyimg` tool driver, a `rules[]` catalog entry per rule
/// referenced by the findings (id + default level), and one `result` per finding.
/// `version` is a **parameter** (not the live crate version) so the golden is
/// version-independent. `base`, when `Some`, relativizes each location `uri` (and
/// forward-slashes it) so GitHub anchors findings to repo-relative files rather
/// than the absolute canonicalized path. Hand-rolled — no `serde_json`.
pub fn write_sarif(
    outcome: &LintOutcome,
    version: &str,
    base: Option<&Path>,
    out: &mut impl Write,
) -> io::Result<()> {
    // Distinct referenced rule ids, sorted — a stable `rules[]` independent of
    // finding order.
    let mut rule_ids: Vec<&str> = outcome.findings.iter().map(|f| f.rule()).collect();
    rule_ids.sort_unstable();
    rule_ids.dedup();

    // Default level per rule id, looked up from the catalog (the *default*,
    // distinct from a per-result, config-overridden level).
    let catalog = super::default_rules();
    let default_level = |id: &str| -> &'static str {
        catalog
            .iter()
            .find(|r| r.id() == id)
            .map(|r| sarif_level(r.default_severity()))
            .unwrap_or("warning")
    };

    write!(
        out,
        "{{\"version\":\"2.1.0\",\
         \"$schema\":\"https://json.schemastore.org/sarif-2.1.0.json\",\
         \"runs\":[{{\"tool\":{{\"driver\":{{\"name\":\"crustyimg\",\
         \"informationUri\":\"{CRUSTYIMG_URI}\",\"version\":\"{}\",\"rules\":[",
        escape_json(version),
    )?;
    for (i, id) in rule_ids.iter().enumerate() {
        if i > 0 {
            write!(out, ",")?;
        }
        write!(
            out,
            "{{\"id\":\"{}\",\"defaultConfiguration\":{{\"level\":\"{}\"}},\"helpUri\":\"{CRUSTYIMG_URI}\"}}",
            escape_json(id),
            default_level(id),
        )?;
    }
    write!(out, "]}}}},\"results\":[")?;
    for (i, f) in outcome.findings.iter().enumerate() {
        if i > 0 {
            write!(out, ",")?;
        }
        write_sarif_result(f, base, out)?;
    }
    write!(out, "]}}]}}")
}

/// Write one SARIF `result` object.
fn write_sarif_result(f: &Finding, base: Option<&Path>, out: &mut impl Write) -> io::Result<()> {
    let message = match f.fix_command() {
        Some(cmd) => format!("{} — fix: {}", f.message(), cmd),
        None => f.message().to_string(),
    };
    write!(
        out,
        "{{\"ruleId\":\"{}\",\"level\":\"{}\",\"message\":{{\"text\":\"{}\"}},\
         \"locations\":[{{\"physicalLocation\":{{\"artifactLocation\":{{\"uri\":\"{}\"}}}}}}]}}",
        escape_json(f.rule()),
        sarif_level(f.severity()),
        escape_json(&message),
        escape_json(&sarif_uri(f.file(), base)),
    )
}

/// Map a [`Severity`] to a SARIF `level`.
fn sarif_level(sev: Severity) -> &'static str {
    match sev {
        Severity::Error => "error",
        Severity::Warn => "warning",
        Severity::Info => "note",
    }
}

/// A SARIF-friendly location uri: relativized to `base` (when the path is under
/// it) and forward-slashed, so GitHub code-scanning anchors it to a repo file.
fn sarif_uri(file: &Path, base: Option<&Path>) -> String {
    let rel = match base {
        Some(b) => file.strip_prefix(b).unwrap_or(file),
        None => file,
    };
    rel.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::super::{Finding, LintOutcome, Severity};
    use super::*;

    /// A fixed, synthetic outcome (forward-slash paths → identical `display()`
    /// on every OS, so the golden is cross-platform safe, per SPEC-049).
    fn synthetic() -> LintOutcome {
        LintOutcome {
            findings: vec![
                Finding::new(
                    "img/leak.jpg",
                    "privacy/gps-metadata-leak",
                    Severity::Error,
                    "image carries GPS location metadata",
                    Some("meta clean --gps".to_string()),
                ),
                Finding::new(
                    "img/photo.png",
                    "size/oversized-bytes",
                    Severity::Warn,
                    "exceeds the byte budget",
                    Some("optimize".to_string()),
                )
                .with_bytes_saved(12000),
            ],
            files_scanned: 3,
        }
    }

    #[test]
    fn json_golden_is_an_exact_byte_stable_string() {
        let outcome = synthetic();
        let mut buf = Vec::new();
        write_json(&outcome, false, &mut buf).unwrap();
        let got = String::from_utf8(buf).unwrap();
        let expected = concat!(
            r#"{"schema":"crustyimg.lint/v1","findings":["#,
            r#"{"file":"img/leak.jpg","rule":"privacy/gps-metadata-leak","severity":"error","#,
            r#""message":"image carries GPS location metadata","fix":"crustyimg meta clean --gps img/leak.jpg"},"#,
            r#"{"file":"img/photo.png","rule":"size/oversized-bytes","severity":"warn","#,
            r#""message":"exceeds the byte budget","fix":"crustyimg optimize img/photo.png","bytes_saved":12000}"#,
            r#"],"summary":{"files_scanned":3,"errors":1,"warnings":1,"infos":0,"#,
            r#""potential_bytes_saved":12000,"passed":false}}"#,
        );
        assert_eq!(got, expected);
    }

    #[test]
    fn json_determinism_two_renders_are_byte_identical() {
        let outcome = synthetic();
        let mut a = Vec::new();
        let mut b = Vec::new();
        write_json(&outcome, true, &mut a).unwrap();
        write_json(&outcome, true, &mut b).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn clean_outcome_is_passed_true_with_empty_findings() {
        let outcome = LintOutcome {
            findings: vec![],
            files_scanned: 2,
        };
        let mut buf = Vec::new();
        write_json(&outcome, true, &mut buf).unwrap();
        let got = String::from_utf8(buf).unwrap();
        assert!(got.contains("\"findings\":[]"));
        assert!(got.contains("\"passed\":true"));
        assert!(got.contains("\"potential_bytes_saved\":0"));
    }

    #[test]
    fn human_render_groups_by_file_shows_fix_and_savings_summary() {
        let outcome = synthetic();
        let mut buf = Vec::new();
        render_human(&outcome, &mut buf).unwrap();
        let got = String::from_utf8(buf).unwrap();
        // Grouped by file, runnable fix present.
        assert!(got.contains("img/leak.jpg"));
        assert!(got.contains("error privacy/gps-metadata-leak:"));
        assert!(got.contains("fix: crustyimg meta clean --gps img/leak.jpg"));
        // Savings summary present when a finding carries bytes_saved.
        assert!(got.contains("~12000 bytes saveable"), "summary: {got}");
    }

    #[test]
    fn json_escapes_quotes_and_backslashes() {
        let outcome = LintOutcome {
            findings: vec![Finding::new(
                "weird\"name\\x.png",
                "size/truncated-or-corrupt",
                Severity::Error,
                "bad \"quote\"",
                None,
            )],
            files_scanned: 1,
        };
        let mut buf = Vec::new();
        write_json(&outcome, false, &mut buf).unwrap();
        let got = String::from_utf8(buf).unwrap();
        assert!(got.contains(r#"weird\"name\\x.png"#));
        assert!(got.contains(r#"bad \"quote\""#));
        assert!(got.contains("\"fix\":null"));
    }

    // ── SARIF (SPEC-056) ─────────────────────────────────────────────────────

    fn sarif_string(outcome: &LintOutcome) -> String {
        let mut buf = Vec::new();
        // Version pinned + base None → version- and cwd-independent (stable golden).
        write_sarif(outcome, "0.4.0", None, &mut buf).unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn sarif_golden_is_an_exact_byte_stable_string() {
        let got = sarif_string(&synthetic());
        let expected = concat!(
            r#"{"version":"2.1.0","$schema":"https://json.schemastore.org/sarif-2.1.0.json","#,
            r#""runs":[{"tool":{"driver":{"name":"crustyimg","#,
            r#""informationUri":"https://github.com/jysf/crustyimg","version":"0.4.0","rules":["#,
            r#"{"id":"privacy/gps-metadata-leak","defaultConfiguration":{"level":"error"},"#,
            r#""helpUri":"https://github.com/jysf/crustyimg"},"#,
            r#"{"id":"size/oversized-bytes","defaultConfiguration":{"level":"error"},"#,
            r#""helpUri":"https://github.com/jysf/crustyimg"}]}},"results":["#,
            r#"{"ruleId":"privacy/gps-metadata-leak","level":"error","#,
            r#""message":{"text":"image carries GPS location metadata — fix: crustyimg meta clean --gps img/leak.jpg"},"#,
            r#""locations":[{"physicalLocation":{"artifactLocation":{"uri":"img/leak.jpg"}}}]},"#,
            r#"{"ruleId":"size/oversized-bytes","level":"warning","#,
            r#""message":{"text":"exceeds the byte budget — fix: crustyimg optimize img/photo.png"},"#,
            r#""locations":[{"physicalLocation":{"artifactLocation":{"uri":"img/photo.png"}}}]}"#,
            r#"]}]}"#,
        );
        assert_eq!(got, expected);
    }

    #[test]
    fn sarif_determinism_two_renders_are_byte_identical() {
        let outcome = synthetic();
        assert_eq!(sarif_string(&outcome), sarif_string(&outcome));
    }

    #[test]
    fn sarif_maps_severities_and_empty_results_on_clean() {
        // Level mapping across the three severities.
        let outcome = LintOutcome {
            findings: vec![
                Finding::new(
                    "a.png",
                    "size/truncated-or-corrupt",
                    Severity::Error,
                    "e",
                    None,
                ),
                Finding::new(
                    "b.png",
                    "orient/orientation-not-baked",
                    Severity::Warn,
                    "w",
                    None,
                ),
                Finding::new("c.png", "color/missing-icc", Severity::Info, "i", None),
            ],
            files_scanned: 3,
        };
        let s = sarif_string(&outcome);
        assert!(s.contains(r#""level":"error""#));
        assert!(s.contains(r#""level":"warning""#));
        assert!(s.contains(r#""level":"note""#));

        // A clean outcome → empty results.
        let clean = LintOutcome {
            findings: vec![],
            files_scanned: 2,
        };
        let cs = sarif_string(&clean);
        assert!(cs.contains(r#""rules":[]"#));
        assert!(cs.contains(r#""results":[]"#));
    }

    #[test]
    fn sarif_relativizes_and_forward_slashes_the_uri() {
        use std::path::{Path, PathBuf};
        // A path under a base dir is emitted repo-relative, forward-slashed.
        let base = PathBuf::from("/work/repo");
        let outcome = LintOutcome {
            findings: vec![Finding::new(
                Path::new("/work/repo").join("assets").join("hero.jpg"),
                "privacy/gps-metadata-leak",
                Severity::Error,
                "leak",
                None,
            )],
            files_scanned: 1,
        };
        let mut buf = Vec::new();
        write_sarif(&outcome, "0.4.0", Some(&base), &mut buf).unwrap();
        let s = String::from_utf8(buf).unwrap();
        assert!(
            s.contains(r#""uri":"assets/hero.jpg""#),
            "relative + slashed: {s}"
        );
    }
}
