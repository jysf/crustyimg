//! SPEC-088: the unified audit report (`--json` / `--timing`) across the
//! auto-decision verbs (`optimize` / `web` / `apply --recipe web`) plus the
//! committed offline benchmark harness.
//!
//! Every test drives the real compiled binary via `env!("CARGO_BIN_EXE_crustyimg")`.
//! The bench tests additionally drive `scripts/bench.py` over the committed
//! `bench/corpus/` (license-clean synthetic images) — offline, no telemetry.

use std::path::{Path, PathBuf};
use std::process::Command;

mod common;

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");
/// The crate root, so the bench script + corpus resolve regardless of CWD.
const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");

fn stdout_str(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}
fn stderr_str(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).trim().to_owned()
}

/// Write bytes to `dir/name`, returning the full path.
fn write_bytes(dir: &tempfile::TempDir, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.path().join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

/// Pull the top-level object keys out of one of our single-line, hand-rolled JSON
/// reports. Tracks `{}`/`[]` nesting and string state so nested objects
/// (`features`, `timing`), arrays (`candidates`), and colons inside string values
/// are not mistaken for top-level keys. The schema values we emit carry no
/// depth-0 commas, which keeps this deliberately small parser correct.
fn top_level_keys(json: &str) -> Vec<String> {
    let s = json.trim();
    assert!(
        s.starts_with('{') && s.ends_with('}'),
        "not a JSON object: {s}"
    );
    let inner = &s[1..s.len() - 1];
    let mut keys = Vec::new();
    let mut depth = 0i32;
    let mut expect_key = true; // at object start / right after a depth-0 comma
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        match c {
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            ',' if depth == 0 => expect_key = true,
            '"' if depth == 0 && expect_key => {
                let mut key = String::new();
                for k in chars.by_ref() {
                    if k == '"' {
                        break;
                    }
                    key.push(k);
                }
                keys.push(key);
                expect_key = false;
            }
            _ => {}
        }
    }
    keys.sort();
    keys
}

/// Pull a `"key":<number>` (possibly nested) from a flat-ish JSON string.
fn json_number(json: &str, key: &str) -> Option<f64> {
    let needle = format!("\"{key}\":");
    let start = json.find(&needle)? + needle.len();
    let rest = &json[start..];
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(rest.len());
    rest[..end].parse().ok()
}

// ── 1. --timing reports decode/encode/total, folded into --json ───────────────

/// `--timing` yields a decode/encode/total readout on stderr, and `--json`
/// carries the same numbers in a `timing` object; the numbers are plausible
/// (`total_ms >= encode_ms`, and each is finite/non-negative).
#[test]
fn timing_flag_reports_and_json_includes_it() {
    let dir = tempfile::tempdir().unwrap();
    let photo = write_bytes(&dir, "photo.jpg", &common::jpeg_with_exif(256, 256));

    // Human channel: the timing readout goes to stderr (stdout stays pipe-clean).
    let human = Command::new(BIN)
        .args([
            "web",
            photo.to_str().unwrap(),
            "--out-dir",
            dir.path().join("h").to_str().unwrap(),
            "--timing",
        ])
        .output()
        .unwrap();
    assert_eq!(
        human.status.code(),
        Some(0),
        "stderr: {}",
        stderr_str(&human)
    );
    let err = stderr_str(&human);
    assert!(err.contains("decode"), "timing must mention decode: {err}");
    assert!(err.contains("encode"), "timing must mention encode: {err}");
    assert!(err.contains("total"), "timing must mention total: {err}");

    // JSON channel: the timing object rides the report on stdout.
    let json_out = Command::new(BIN)
        .args([
            "web",
            photo.to_str().unwrap(),
            "--out-dir",
            dir.path().join("j").to_str().unwrap(),
            "--json",
            "--timing",
        ])
        .output()
        .unwrap();
    assert_eq!(
        json_out.status.code(),
        Some(0),
        "stderr: {}",
        stderr_str(&json_out)
    );
    let json = stdout_str(&json_out);
    assert!(
        json.contains("\"timing\":{"),
        "json must carry timing: {json}"
    );
    let decode = json_number(&json, "decode_ms").expect("decode_ms present");
    let encode = json_number(&json, "encode_ms").expect("encode_ms present");
    let total = json_number(&json, "total_ms").expect("total_ms present");
    assert!(
        decode >= 0.0 && encode >= 0.0 && total >= 0.0,
        "timings must be non-negative: d={decode} e={encode} t={total}"
    );
    assert!(
        total + 1e-6 >= encode,
        "total ({total}) must be >= encode ({encode})"
    );
}

// ── 2. --json shape consistent across optimize / web / apply ──────────────────

/// `optimize` / `web` / `apply --recipe web` all emit the SAME versioned report
/// shape (identical top-level key set) — the `optimize.explain/v1` schema, not a
/// per-command fork. Asserted against a golden key set.
#[test]
fn json_shape_consistent_across_verbs() {
    let dir = tempfile::tempdir().unwrap();
    let photo = write_bytes(&dir, "photo.jpg", &common::jpeg_with_exif(256, 256));

    // Golden top-level key set with `--json --timing` and a scored (lossy) winner.
    // `optimize` is run with `--verify` so its ssim field is present like web/apply.
    let golden: Vec<String> = {
        let mut k = vec![
            "schema",
            "source_format",
            "class",
            "profile",
            "mode",
            "features",
            "source_bytes",
            "candidates",
            "winner",
            "out_bytes",
            "savings_percent",
            "ssim",
            "timing",
        ]
        .into_iter()
        .map(String::from)
        .collect::<Vec<_>>();
        k.sort();
        k
    };

    let run = |args: &[&str], tag: &str| -> Vec<String> {
        let out = Command::new(BIN).args(args).output().unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "{tag} failed; stderr: {}",
            stderr_str(&out)
        );
        let json = stdout_str(&out);
        // One report object per input; single input ⇒ one line.
        assert!(
            json.contains("crustyimg.optimize.explain/v1"),
            "{tag}: {json}"
        );
        top_level_keys(&json)
    };

    let odir = dir.path().join("o");
    let opt = run(
        &[
            "optimize",
            photo.to_str().unwrap(),
            "--out-dir",
            odir.to_str().unwrap(),
            "--json",
            "--timing",
            "--verify",
        ],
        "optimize",
    );
    let wdir = dir.path().join("w");
    let web = run(
        &[
            "web",
            photo.to_str().unwrap(),
            "--out-dir",
            wdir.to_str().unwrap(),
            "--json",
            "--timing",
        ],
        "web",
    );
    let adir = dir.path().join("a");
    let apply = run(
        &[
            "apply",
            "--recipe",
            "web",
            photo.to_str().unwrap(),
            "--out-dir",
            adir.to_str().unwrap(),
            "--json",
            "--timing",
        ],
        "apply",
    );

    assert_eq!(opt, golden, "optimize keys diverge from golden");
    assert_eq!(web, golden, "web keys diverge from golden");
    assert_eq!(apply, golden, "apply keys diverge from golden");
}

// ── 3. non-audit output is byte-identical (regression anchor) ─────────────────

/// A plain run (no `--json` / `--timing`) is unchanged by this spec: stdout stays
/// pipe-clean (only image bytes on `-o -`), and the stderr summary is a single
/// clean line with NO timing line and NO JSON — exactly as before SPEC-088.
#[test]
fn non_json_output_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let photo = write_bytes(&dir, "photo.jpg", &common::jpeg_with_exif(256, 256));

    // Writing to --out-dir: stdout is empty, stderr is exactly one summary line.
    let out = Command::new(BIN)
        .args([
            "web",
            photo.to_str().unwrap(),
            "--out-dir",
            dir.path().join("o").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "stderr: {}", stderr_str(&out));
    assert!(
        out.stdout.is_empty(),
        "stdout must be pipe-clean, got {} bytes",
        out.stdout.len()
    );
    let err = stderr_str(&out);
    assert_eq!(
        err.lines().count(),
        1,
        "default summary must be exactly one line: {err}"
    );
    for banned in ["timing", "decode_ms", "total_ms", "\"schema\"", "{\""] {
        assert!(
            !err.contains(banned),
            "default summary must not carry audit output ({banned}): {err}"
        );
    }

    // Writing to stdout (`-o -`): stdout carries only the image, stderr the summary
    // (no audit output). This is the pipe-clean guarantee the audit surface must keep.
    let piped = Command::new(BIN)
        .args(["web", photo.to_str().unwrap(), "-o", "-"])
        .output()
        .unwrap();
    assert_eq!(
        piped.status.code(),
        Some(0),
        "stderr: {}",
        stderr_str(&piped)
    );
    assert!(
        piped.stdout.len() > 8,
        "stdout must carry the encoded image bytes"
    );
    assert!(
        !stderr_str(&piped).contains("timing"),
        "no timing without --timing: {}",
        stderr_str(&piped)
    );
}

// ── 4. committed bench harness runs offline over the committed corpus ─────────

/// Locate `python3`, returning `None` (so the test skips) when it is absent.
fn python3() -> Option<String> {
    for cand in ["python3", "python"] {
        if Command::new(cand).arg("--version").output().is_ok() {
            return Some(cand.to_owned());
        }
    }
    None
}

/// `scripts/bench.py` runs over the committed corpus with zero network and no
/// telemetry, printing a savings/time/score table; its `--json` mode parses.
#[test]
fn bench_runs_offline_on_committed_corpus() {
    let Some(py) = python3() else {
        eprintln!("python3 not found; skipping bench harness test");
        return;
    };
    let script = Path::new(MANIFEST_DIR).join("scripts/bench.py");
    let corpus = Path::new(MANIFEST_DIR).join("bench/corpus");
    assert!(
        script.is_file(),
        "bench harness missing: {}",
        script.display()
    );
    assert!(
        corpus.is_dir(),
        "committed corpus missing: {}",
        corpus.display()
    );

    // Table mode: human savings/time/score table.
    let table = Command::new(&py)
        .arg(&script)
        .args(["--corpus", corpus.to_str().unwrap(), "--bin", BIN])
        // Neutralize any proxy config; the harness must never touch the network.
        .env_remove("http_proxy")
        .env_remove("https_proxy")
        .env_remove("HTTP_PROXY")
        .env_remove("HTTPS_PROXY")
        .output()
        .unwrap();
    assert_eq!(
        table.status.code(),
        Some(0),
        "bench table failed; stderr: {}",
        stderr_str(&table)
    );
    let t = stdout_str(&table);
    for needle in ["web", "optimize", "savings"] {
        assert!(
            t.to_lowercase().contains(needle),
            "table missing {needle}: {t}"
        );
    }

    // JSON mode: machine-readable aggregate that parses.
    let json = Command::new(&py)
        .arg(&script)
        .args(["--corpus", corpus.to_str().unwrap(), "--bin", BIN, "--json"])
        .output()
        .unwrap();
    assert_eq!(
        json.status.code(),
        Some(0),
        "bench --json failed; stderr: {}",
        stderr_str(&json)
    );
    let j = stdout_str(&json);
    assert!(
        j.starts_with('{') || j.starts_with('['),
        "bench --json not JSON: {j}"
    );
    assert!(
        j.contains("savings_percent"),
        "bench --json missing savings: {j}"
    );
    assert!(
        j.contains("web") && j.contains("optimize"),
        "bench --json missing verbs: {j}"
    );
}

// ── 5. the committed corpus is license-clean (no private-photo EXIF) ───────────

/// Every committed corpus image is synthetic/CC0 with documented provenance and
/// carries NO EXIF (a private photo's GPS/camera tags would fail this). Verified
/// through the real `info --json` decoder, plus the provenance README.
#[test]
fn bench_corpus_is_license_clean() {
    let corpus = Path::new(MANIFEST_DIR).join("bench/corpus");
    assert!(
        corpus.is_dir(),
        "committed corpus missing: {}",
        corpus.display()
    );

    // Provenance/license note must exist.
    let readme = corpus.join("README.md");
    assert!(
        readme.is_file(),
        "corpus provenance README missing: {}",
        readme.display()
    );
    let prov = std::fs::read_to_string(&readme).unwrap().to_lowercase();
    assert!(
        prov.contains("synthetic") || prov.contains("cc0"),
        "README must document a license-clean provenance"
    );

    let mut checked = 0usize;
    for entry in std::fs::read_dir(&corpus).unwrap() {
        let path = entry.unwrap().path();
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp") {
            continue;
        }
        let out = Command::new(BIN)
            .args(["info", path.to_str().unwrap(), "--json"])
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(0),
            "info failed on {}: {}",
            path.display(),
            stderr_str(&out)
        );
        let json = stdout_str(&out);
        assert!(
            json.contains("\"has_exif\":false"),
            "corpus image {} must carry no EXIF: {json}",
            path.display()
        );
        checked += 1;
    }
    assert!(
        checked >= 2,
        "expected a small committed corpus, found {checked} images"
    );
}
