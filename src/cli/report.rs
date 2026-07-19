//! Read-only reporting commands: `info`, `diff`, and `lint` (SPEC-013,
//! SPEC-023, SPEC-050/051). Also the crate's one hand-rolled JSON string
//! escaper (SPEC-097 dedup — `lint::report` used to carry an independent
//! copy). Split out of `cli/mod.rs` (SPEC-097) — no behavior change.

use std::io::Cursor;
use std::path::{Path, PathBuf};

use crate::error::ImageError;
use crate::image::Image;
use crate::quality;
use crate::source::{self, SourceError};

use super::{CliError, GlobalArgs};

// ── Info command ─────────────────────────────────────────────────────────────

/// CLI-local, serde-serializable inspection report (NOT the pixel-core
/// `ImageInfo`, which is not Serialize and holds non-Serialize `image::` types).
/// Built from `ImageInfo` + the file-size-on-disk + the optional EXIF dump.
#[derive(Debug, Clone, serde::Serialize)]
struct InfoReport {
    /// Input path as given (or "-" for stdin).
    input: String,
    width: u32,
    height: u32,
    /// Stable lowercase format label, e.g. "png", "jpeg".
    format: String,
    /// Encoded file size on disk in bytes (NOT the decoded buffer length).
    file_size_bytes: u64,
    /// Decoded in-memory pixel-buffer length in bytes (distinct from file size).
    decoded_bytes: u64,
    /// Stable lowercase color-type label, e.g. "rgb8", "rgba8", "l8".
    color_type: String,
    /// Bits per channel (8, 16, …).
    bit_depth: u8,
    has_alpha: bool,
    has_icc: bool,
    has_exif: bool,
    /// Present only when --exif is passed: the read EXIF tags (possibly empty).
    /// Omitted entirely (serde `skip_serializing_if`) when --exif is absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    exif: Option<Vec<ExifTag>>,
}

/// One EXIF tag rendered for output (read-only; kamadak-exif, DEC-013).
#[derive(Debug, Clone, serde::Serialize)]
struct ExifTag {
    /// Tag name, e.g. "Make", "Orientation" (kamadak-exif's Tag Display).
    tag: String,
    /// Which IFD the tag came from, e.g. "primary", "thumbnail" (IFD Display).
    ifd: String,
    /// Human-readable value via Field::display_value().with_unit(&exif).
    value: String,
}

/// Map an `image::ImageFormat` to a stable lowercase label for output.
/// Free fn so it is directly unit-testable; no panic on any variant.
pub(super) fn format_label(fmt: ::image::ImageFormat) -> String {
    match fmt {
        ::image::ImageFormat::Png => "png".to_owned(),
        ::image::ImageFormat::Jpeg => "jpeg".to_owned(),
        ::image::ImageFormat::Gif => "gif".to_owned(),
        ::image::ImageFormat::Bmp => "bmp".to_owned(),
        ::image::ImageFormat::Tiff => "tiff".to_owned(),
        ::image::ImageFormat::Ico => "ico".to_owned(),
        // Non-exhaustive: stable lowercase fallback for any other variant.
        _ => format!("{fmt:?}").to_ascii_lowercase(),
    }
}

/// Map an `image::ColorType` to a stable lowercase label, e.g. "rgb8".
/// Free fn; unit-testable; no panic on any variant.
fn color_type_label(ct: ::image::ColorType) -> String {
    match ct {
        ::image::ColorType::Rgb8 => "rgb8".to_owned(),
        ::image::ColorType::Rgba8 => "rgba8".to_owned(),
        ::image::ColorType::L8 => "l8".to_owned(),
        ::image::ColorType::La8 => "la8".to_owned(),
        ::image::ColorType::Rgb16 => "rgb16".to_owned(),
        ::image::ColorType::Rgba16 => "rgba16".to_owned(),
        ::image::ColorType::L16 => "l16".to_owned(),
        ::image::ColorType::La16 => "la16".to_owned(),
        ::image::ColorType::Rgb32F => "rgb32f".to_owned(),
        ::image::ColorType::Rgba32F => "rgba32f".to_owned(),
        // Non-exhaustive: stable lowercase fallback for any other variant.
        _ => format!("{ct:?}").to_ascii_lowercase(),
    }
}

/// Read EXIF tags from full container bytes (read-only, DEC-013). Returns an
/// empty Vec when there is NO EXIF (`exif::Error::NotFound`) or the EXIF is
/// malformed/unreadable — "no EXIF" is NOT an error. Never panics.
fn read_exif_tags(bytes: &[u8]) -> Vec<ExifTag> {
    match exif::Reader::new().read_from_container(&mut Cursor::new(bytes)) {
        Ok(exif) => exif
            .fields()
            .map(|f| ExifTag {
                tag: f.tag.to_string(),
                ifd: f.ifd_num.to_string(),
                value: f.display_value().with_unit(&exif).to_string(),
            })
            .collect(),
        // NotFound OR malformed → "no EXIF", not an error.
        Err(_) => Vec::new(),
    }
}

/// Escape a string value for inclusion in a hand-rolled JSON object.
///
/// `"` → `\"`, `\` → `\\`, control chars < 0x20 → `\u00XX`.
///
/// The crate's ONE JSON escaper (SPEC-097 dedup — `lint::report` used to carry
/// an independent, previously-drift-prone copy); `pub(crate)` so `lint::report`
/// can reuse it across the `cli`/`lint` module boundary.
pub(crate) fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04X}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}

/// Emit the `InfoReport` as a single-line JSON object to `out`.
///
/// Hand-rolled (no serde_json runtime dep) following the locked schema table
/// in the spec. Escapes all string values. Propagates I/O errors via `?`.
fn write_json(report: &InfoReport, out: &mut impl std::io::Write) -> std::io::Result<()> {
    write!(
        out,
        "{{\"input\":\"{}\",\"width\":{},\"height\":{},\"format\":\"{}\",\
         \"file_size_bytes\":{},\"decoded_bytes\":{},\"color_type\":\"{}\",\
         \"bit_depth\":{},\"has_alpha\":{},\"has_icc\":{},\"has_exif\":{}",
        escape_json(&report.input),
        report.width,
        report.height,
        escape_json(&report.format),
        report.file_size_bytes,
        report.decoded_bytes,
        escape_json(&report.color_type),
        report.bit_depth,
        report.has_alpha,
        report.has_icc,
        report.has_exif,
    )?;
    // Emit `exif` key only when --exif was passed.
    if let Some(ref tags) = report.exif {
        write!(out, ",\"exif\":[")?;
        for (i, tag) in tags.iter().enumerate() {
            if i > 0 {
                write!(out, ",")?;
            }
            write!(
                out,
                "{{\"tag\":\"{}\",\"ifd\":\"{}\",\"value\":\"{}\"}}",
                escape_json(&tag.tag),
                escape_json(&tag.ifd),
                escape_json(&tag.value),
            )?;
        }
        write!(out, "]")?;
    }
    writeln!(out, "}}")
}

/// Print the `InfoReport` as human-readable labeled lines to `out`.
///
/// The exact label wording satisfies the spec's assertable substrings:
/// the `{w}x{h}` form, the format label, the color-type label, and the
/// ICC/EXIF presence words.
fn print_human(report: &InfoReport, out: &mut impl std::io::Write) -> std::io::Result<()> {
    writeln!(out, "input:      {}", report.input)?;
    writeln!(out, "dimensions: {}x{}", report.width, report.height)?;
    writeln!(out, "format:     {}", report.format)?;
    writeln!(out, "file size:  {} bytes", report.file_size_bytes)?;
    writeln!(out, "color type: {}", report.color_type)?;
    writeln!(out, "bit depth:  {}", report.bit_depth)?;
    writeln!(
        out,
        "alpha:      {}",
        if report.has_alpha { "yes" } else { "no" }
    )?;
    writeln!(
        out,
        "icc:        {}",
        if report.has_icc { "yes" } else { "no" }
    )?;
    writeln!(
        out,
        "exif:       {}",
        if report.has_exif { "yes" } else { "no" }
    )?;
    // Emit EXIF tag dump only when --exif was passed.
    if let Some(ref tags) = report.exif {
        if tags.is_empty() {
            writeln!(out, "exif tags:  (none)")?;
        } else {
            writeln!(out, "exif tags:")?;
            for tag in tags {
                writeln!(out, "  {}: {}", tag.tag, tag.value)?;
            }
        }
    }
    Ok(())
}

/// The `info` path: resolve the first input, load the image and its raw
/// bytes (one read), build the report, and print human text or JSON to
/// stdout. Single-image: resolves the FIRST input on a directory/glob.
pub(super) fn run_info(
    input: &str,
    exif: bool,
    json: bool,
    _global: &GlobalArgs,
) -> Result<(), CliError> {
    let resolved = source::resolve(input, &mut std::io::stdin().lock())?;
    let first = resolved
        .into_iter()
        .next()
        .ok_or(CliError::Source(SourceError::NotFound(input.to_owned())))?;

    // Read the raw bytes ONCE: they give the file size, the decoded
    // image, and the EXIF source. (For a path, std::fs::read io-error
    // maps to ImageError::Io → exit 3, consistent with Image::load.)
    //
    // Decode through the SAME path-aware routing as every other command: a
    // `Path` goes through `Image::decode_path` so RAW extensions extract the
    // embedded preview (SPEC-061, DEC-055) instead of being mis-decoded by the
    // generic byte decoder; stdin has no path, so it stays on `from_bytes`.
    let (raw, label, img): (Vec<u8>, String, Image) = match &first {
        crate::source::Input::Path(p) => {
            let bytes = std::fs::read(p).map_err(ImageError::Io)?;
            let img = Image::decode_path(p, &bytes)?;
            (bytes, p.display().to_string(), img)
        }
        crate::source::Input::Stdin { bytes, .. } => {
            let img = Image::from_bytes(bytes)?;
            (bytes.clone(), "-".to_owned(), img)
        }
    };
    let info = img.info();

    let exif_tags = if exif {
        Some(read_exif_tags(&raw))
    } else {
        None
    };

    let report = InfoReport {
        input: label,
        width: info.width,
        height: info.height,
        format: format_label(info.format),
        file_size_bytes: raw.len() as u64,
        decoded_bytes: info.byte_len,
        color_type: color_type_label(info.color_type),
        bit_depth: info.bit_depth,
        has_alpha: info.has_alpha,
        has_icc: info.has_icc,
        has_exif: info.has_exif,
        exif: exif_tags,
    };

    let mut out = std::io::stdout().lock();
    if json {
        write_json(&report, &mut out).map_err(crate::sink::SinkError::Io)?;
    } else {
        print_human(&report, &mut out).map_err(crate::sink::SinkError::Io)?;
    }
    Ok(())
}

// ── diff command (SPEC-023, DEC-025) ──────────────────────────────────────────

/// Whether a `diff` score passes the `--fail-under` gate: a score ≥ the threshold,
/// or `true` when there is no gate. The single decision the gate exit code keys off.
fn diff_passes(score: f64, fail_under: Option<f64>) -> bool {
    fail_under.is_none_or(|t| score >= t)
}

/// Emit the `diff` result as a single-line JSON object to `out` (hand-rolled, no
/// serde_json runtime dep — mirrors [`write_json`]). `fail_under` is the number or
/// the literal `null`; `passed` is a bare bool.
fn write_diff_json(
    out: &mut impl std::io::Write,
    a: &str,
    b: &str,
    score: f64,
    fail_under: Option<f64>,
    passed: bool,
) -> std::io::Result<()> {
    write!(
        out,
        "{{\"a\":\"{}\",\"b\":\"{}\",\"score\":{:.4},\"fail_under\":",
        escape_json(a),
        escape_json(b),
        score,
    )?;
    match fail_under {
        Some(t) => write!(out, "{t:.4}")?,
        None => write!(out, "null")?,
    }
    writeln!(out, ",\"passed\":{passed}}}")
}

/// The `diff` path: load two images, score `b` against `a` with SSIMULACRA2, print
/// the score (human or `--json`), and apply the `--fail-under` CI gate (DEC-025).
///
/// - `--fail-under` outside `0..=100` → usage error (exit 2).
/// - The two images MUST have equal dimensions (SSIMULACRA2 requires it); a mismatch
///   is a usage error (exit 2), NOT an implicit resize.
/// - The score line is printed to stdout BEFORE any gate failure, so CI captures both
///   the number and the verdict. A failed gate returns [`CliError::CheckFailed`]
///   (exit 7); the diagnostic goes to stderr (unless `--quiet`).
pub(super) fn run_diff(
    a: &str,
    b: &str,
    fail_under: Option<f64>,
    json: bool,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    use std::io::Write;

    if let Some(t) = fail_under {
        if !(0.0..=100.0).contains(&t) {
            return Err(CliError::Usage(format!(
                "--fail-under must be a score in 0..=100, got {t}"
            )));
        }
    }

    let img_a = Image::load(a)?;
    let img_b = Image::load(b)?;
    if img_a.width() != img_b.width() || img_a.height() != img_b.height() {
        return Err(CliError::Usage(format!(
            "cannot compare images of different dimensions ({}x{} vs {}x{})",
            img_a.width(),
            img_a.height(),
            img_b.width(),
            img_b.height()
        )));
    }

    let score = quality::score(img_a.pixels(), img_b.pixels())?;
    let passed = diff_passes(score, fail_under);

    let mut out = std::io::stdout().lock();
    if json {
        write_diff_json(&mut out, a, b, score, fail_under, passed)
            .map_err(crate::sink::SinkError::Io)?;
    } else {
        writeln!(out, "ssimulacra2: {score:.4}").map_err(crate::sink::SinkError::Io)?;
    }

    if !passed {
        if !global.quiet {
            eprintln!(
                "diff: ssimulacra2 {score:.4} is below --fail-under {}",
                fail_under.unwrap_or(0.0)
            );
        }
        return Err(CliError::CheckFailed);
    }
    Ok(())
}

// ── lint command (SPEC-050/051, DEC-050) ───────────────────────────────────────

/// The parsed `lint` config/severity flags (SPEC-051), borrowed from `Commands`.
pub(super) struct LintFlags<'a> {
    pub(super) config: Option<&'a str>,
    pub(super) no_config: bool,
    pub(super) select: &'a [String],
    pub(super) ignore: &'a [String],
    pub(super) max_warnings: Option<usize>,
    pub(super) max_intended_width: Option<u32>,
    pub(super) savings_threshold: Option<&'a str>,
}

/// Parse `--savings-threshold BYTES:PERCENT` into a [`SavingsThreshold`].
///
/// A malformed value (missing `:`, non-integer, percent > 100) is a usage error
/// (exit 2), never a panic.
fn parse_savings_threshold(s: &str) -> Result<crate::lint::config::SavingsThreshold, CliError> {
    let (bytes_s, percent_s) = s.split_once(':').ok_or_else(|| {
        CliError::Usage(format!(
            "invalid --savings-threshold '{s}': expected BYTES:PERCENT (e.g. 4096:10)"
        ))
    })?;
    let min_bytes: u64 = bytes_s
        .trim()
        .parse()
        .map_err(|_| CliError::Usage(format!("invalid --savings-threshold bytes '{bytes_s}'")))?;
    let min_percent: u32 = percent_s.trim().parse().map_err(|_| {
        CliError::Usage(format!("invalid --savings-threshold percent '{percent_s}'"))
    })?;
    if min_percent > 100 {
        return Err(CliError::Usage(format!(
            "invalid --savings-threshold percent '{min_percent}': must be 0..=100"
        )));
    }
    Ok(crate::lint::config::SavingsThreshold {
        min_bytes,
        min_percent,
    })
}

/// Build the effective [`LintConfig`](crate::lint::config::LintConfig) from the
/// discovered/forced file + CLI overrides. A config error maps to a usage error
/// (exit 2).
fn build_lint_config(
    flags: &LintFlags,
    first_input: &str,
) -> Result<crate::lint::config::LintConfig, CliError> {
    use crate::lint::config::{discover_config, effective_config, CliOverrides};

    let savings_threshold = flags
        .savings_threshold
        .map(parse_savings_threshold)
        .transpose()?;

    let overrides = CliOverrides {
        select: flags.select.to_vec(),
        ignore: flags.ignore.to_vec(),
        max_intended_width: flags.max_intended_width,
        savings_threshold,
    };

    let forced = flags.config.map(PathBuf::from);
    // Auto-discovery anchors on the first input (or cwd) walking up to root.
    let discovered = if flags.no_config || forced.is_some() {
        None
    } else {
        discover_config(Path::new(first_input))
    };

    effective_config(
        &overrides,
        forced.as_deref(),
        discovered.as_deref(),
        flags.no_config,
    )
    .map_err(|e| CliError::Usage(e.to_string()))
}

/// The `lint` path: build the effective config (SPEC-051), resolve every PATH
/// via the shared source fan-out (globs/dirs/files, non-images skipped), run the
/// active rule set per image, print the grouped-by-file human report to stdout,
/// and map the outcome to a CI-native exit code reusing [`CliError::CheckFailed`]
/// (exit 7, DEC-025).
///
/// - No PATHS ⇒ lint the current directory (ergonomic default).
/// - No inputs resolved ⇒ [`SourceError::NotFound`] (exit 3).
/// - An invalid glob pattern / bad config ⇒ exit 2.
/// - ≥1 `Error`-severity finding, or `warn` count over `--max-warnings` ⇒
///   `CheckFailed` (exit 7). `Info` never fails; `Warn` fails only over the cap.
///
/// Read-only: the runner NEVER writes an image; a decode failure is a *finding*
/// (`size/truncated-or-corrupt`), not an abort (DEC-050).
pub(super) fn run_lint(
    paths: &[String],
    flags: &LintFlags,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    use std::io::Write;

    // Default to the current directory when no PATHS are given.
    let default = [".".to_owned()];
    let args: &[String] = if paths.is_empty() { &default } else { paths };

    // The config is discovered relative to the first input (or cwd).
    let config = build_lint_config(flags, &args[0])?;

    // Resolve every arg, concatenating inputs. A NotFound arg contributes
    // nothing (so `no inputs resolved` collapses to exit 3 below); a malformed
    // glob or stdin error propagates (exit 2/3).
    let mut inputs = Vec::new();
    for arg in args {
        match source::resolve(arg, &mut std::io::stdin().lock()) {
            Ok(v) => inputs.extend(v),
            Err(SourceError::NotFound(_)) => {}
            Err(e) => return Err(e.into()),
        }
    }
    if inputs.is_empty() {
        return Err(CliError::Source(SourceError::NotFound(args.join(", "))));
    }

    // Report format: `lint --format human|json|sarif`. To avoid a clap
    // duplicate-arg conflict with the global `--format` (encode format), lint
    // reads that same global flag; only these three values are valid here.
    let report_format = lint_report_format(global.format.as_deref())?;

    let rules = crate::lint::default_rules();
    let outcome = crate::lint::run_lint(&inputs, &rules, &config);
    let passed = crate::lint::exit_code(&outcome, flags.max_warnings) == 0;

    let mut out = std::io::stdout().lock();
    match report_format {
        LintReportFormat::Human => {
            crate::lint::render_human(&outcome, &mut out).map_err(crate::sink::SinkError::Io)?;
        }
        LintReportFormat::Json => {
            crate::lint::write_json(&outcome, passed, &mut out)
                .and_then(|()| writeln!(out))
                .map_err(crate::sink::SinkError::Io)?;
        }
        LintReportFormat::Sarif => {
            // Relativize finding paths to the cwd (the repo root in CI) so GitHub
            // code-scanning anchors them to repo files.
            let base = std::env::current_dir().ok();
            crate::lint::write_sarif(&outcome, crate::version(), base.as_deref(), &mut out)
                .and_then(|()| writeln!(out))
                .map_err(crate::sink::SinkError::Io)?;
        }
    }

    if !passed {
        if !global.quiet {
            eprintln!(
                "lint: {} error, {} warn across {} file(s)",
                outcome.error_count(),
                outcome.warn_count(),
                outcome.files_scanned
            );
        }
        return Err(CliError::CheckFailed);
    }
    Ok(())
}

/// The `lint` report format (SPEC-052 human/json; SPEC-056 sarif).
enum LintReportFormat {
    Human,
    Json,
    Sarif,
}

/// Interpret the global `--format` value for `lint`: `None`/`human` ⇒ human,
/// `json` ⇒ JSON, `sarif` ⇒ SARIF (GitHub code-scanning), anything else ⇒ a
/// usage error (exit 2).
fn lint_report_format(format: Option<&str>) -> Result<LintReportFormat, CliError> {
    match format {
        None | Some("human") => Ok(LintReportFormat::Human),
        Some("json") => Ok(LintReportFormat::Json),
        Some("sarif") => Ok(LintReportFormat::Sarif),
        Some(other) => Err(CliError::Usage(format!(
            "lint --format must be 'human', 'json', or 'sarif', got '{other}'"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── format_label_maps_core_formats ───────────────────────────────────────

    #[test]
    fn format_label_maps_core_formats() {
        assert_eq!(format_label(::image::ImageFormat::Png), "png");
        assert_eq!(format_label(::image::ImageFormat::Jpeg), "jpeg");
        assert_eq!(format_label(::image::ImageFormat::Gif), "gif");
        assert_eq!(format_label(::image::ImageFormat::Bmp), "bmp");
        assert_eq!(format_label(::image::ImageFormat::Tiff), "tiff");
        assert_eq!(format_label(::image::ImageFormat::Ico), "ico");
    }

    // ── color_type_label_maps_color_types ────────────────────────────────────

    #[test]
    fn color_type_label_maps_color_types() {
        assert_eq!(color_type_label(::image::ColorType::Rgb8), "rgb8");
        assert_eq!(color_type_label(::image::ColorType::Rgba8), "rgba8");
        assert_eq!(color_type_label(::image::ColorType::L8), "l8");
        assert_eq!(color_type_label(::image::ColorType::Rgb16), "rgb16");
    }

    // ── read_exif_tags_graceful_on_no_exif ───────────────────────────────────

    #[test]
    fn read_exif_tags_graceful_on_no_exif() {
        use ::image::{DynamicImage, ImageFormat, RgbImage};
        use std::io::Cursor;

        // Empty bytes: no EXIF, no panic.
        assert!(read_exif_tags(&[]).is_empty());

        // Garbage bytes: no EXIF, no panic.
        assert!(read_exif_tags(b"not an image").is_empty());

        // Plain PNG (no EXIF segment): empty result.
        let img = RgbImage::from_pixel(4, 4, ::image::Rgb([1u8, 2, 3]));
        let mut buf = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(img)
            .write_to(&mut buf, ImageFormat::Png)
            .unwrap();
        let png_bytes = buf.into_inner();
        assert!(read_exif_tags(&png_bytes).is_empty());

        // Build a minimal JPEG with a synthetic EXIF APP1 (zero-entry IFD).
        // We replicate the fixture logic inline since unit tests can't reach tests/common.
        let base_jpeg = {
            let jimg = RgbImage::from_pixel(4, 4, ::image::Rgb([128u8, 64, 32]));
            let mut jbuf = Cursor::new(Vec::new());
            DynamicImage::ImageRgb8(jimg)
                .write_to(&mut jbuf, ImageFormat::Jpeg)
                .unwrap();
            jbuf.into_inner()
        };
        let mut payload: Vec<u8> = Vec::new();
        payload.extend_from_slice(b"Exif\0\0");
        payload.extend_from_slice(b"II");
        payload.extend_from_slice(&[0x2A, 0x00]);
        payload.extend_from_slice(&[0x08, 0x00, 0x00, 0x00]);
        payload.extend_from_slice(&[0x00, 0x00]); // 0 IFD entries
        payload.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        let seg_len = (payload.len() + 2) as u16;
        let mut jpeg_with_exif: Vec<u8> = Vec::new();
        jpeg_with_exif.extend_from_slice(&base_jpeg[0..2]); // SOI
        jpeg_with_exif.push(0xFF);
        jpeg_with_exif.push(0xE1);
        jpeg_with_exif.extend_from_slice(&seg_len.to_be_bytes());
        jpeg_with_exif.extend_from_slice(&payload);
        jpeg_with_exif.extend_from_slice(&base_jpeg[2..]);

        // read_exif_tags must return without panicking (len >= 0).
        let tags = read_exif_tags(&jpeg_with_exif);
        // The zero-entry IFD may yield 0 tags — that is correct.
        let _ = tags;
    }

    // ── escape_json (SPEC-097 post-merge regression) ──────────────────────────
    //
    // `cli::report::escape_json` used to have an independent twin in
    // `lint::report`; `escape_json_impls_are_equivalent` proved the two were
    // byte-identical on this exact adversarial set BEFORE they were merged
    // (see the SPEC-097 build history). These two tests are that proof's
    // permanent replacement: they assert the single surviving helper still
    // produces the same proven outputs.

    #[test]
    fn escape_json_escapes_quotes_backslashes_and_control_chars() {
        assert_eq!(escape_json(""), "");
        assert_eq!(escape_json("plain ascii"), "plain ascii");
        assert_eq!(escape_json("\"quoted\""), "\\\"quoted\\\"");
        assert_eq!(escape_json("back\\slash"), "back\\\\slash");
        assert_eq!(
            escape_json("both\"and\\together"),
            "both\\\"and\\\\together"
        );
        assert_eq!(escape_json("\n\t\r"), "\\u000A\\u0009\\u000D");
        assert_eq!(escape_json("\0"), "\\u0000");
        assert_eq!(escape_json("\u{01}\u{02}\u{1F}"), "\\u0001\\u0002\\u001F");
    }

    #[test]
    fn escape_json_passes_through_chars_at_or_above_the_escape_boundary() {
        // Everything >= 0x20 is untouched: the boundary itself, DEL, multi-byte
        // unicode, the code points immediately flanking the surrogate range (a
        // lone surrogate cannot be represented in a Rust `&str`), and U+FFFD.
        for s in [
            "\u{7F}",
            " ",
            "emoji \u{1F980} crab",
            "multi-byte \u{00E9}\u{4E2D}\u{6587}",
            "\u{D7FF}",
            "\u{E000}",
            "\u{FFFD}",
        ] {
            assert_eq!(escape_json(s), s, "must pass through unchanged: {s:?}");
        }
    }

    #[test]
    fn escape_json_handles_a_mixed_adversarial_string() {
        let input = "mixed \"\\\n\u{1F600}\u{0}end";
        // `"` -> `\"`, `\` -> `\\`, newline -> `\u000A`, emoji unchanged,
        // NUL -> `\u0000`.
        let expected = "mixed \\\"\\\\\\u000A\u{1F600}\\u0000end";
        assert_eq!(escape_json(input), expected);
    }

    // ── info_report_serializes_fields ────────────────────────────────────────

    #[test]
    fn info_report_serializes_fields() {
        // No --exif: the "exif" key must be absent.
        let report = InfoReport {
            input: "test.png".to_owned(),
            width: 8,
            height: 8,
            format: "png".to_owned(),
            file_size_bytes: 200,
            decoded_bytes: 192,
            color_type: "rgb8".to_owned(),
            bit_depth: 8,
            has_alpha: false,
            has_icc: false,
            has_exif: false,
            exif: None,
        };
        let val = serde_json::to_value(&report).unwrap();
        assert_eq!(val["width"], 8u64);
        assert_eq!(val["height"], 8u64);
        assert_eq!(val["format"], "png");
        assert_eq!(val["color_type"], "rgb8");
        assert_eq!(val["bit_depth"], 8u64);
        assert_eq!(val["has_alpha"], false);
        assert_eq!(val["has_icc"], false);
        assert_eq!(val["has_exif"], false);
        assert_eq!(val["file_size_bytes"], 200u64);
        assert_eq!(val["decoded_bytes"], 192u64);
        // exif key must be absent when None.
        assert!(
            val.get("exif").is_none(),
            "exif key must be absent when exif: None"
        );

        // With --exif and empty Vec: the "exif" key must be present as an empty array.
        let report_with_exif = InfoReport {
            exif: Some(vec![]),
            ..report
        };
        let val2 = serde_json::to_value(&report_with_exif).unwrap();
        assert!(
            val2.get("exif").is_some(),
            "exif key must be present when exif: Some(_)"
        );
        assert!(
            val2["exif"].as_array().unwrap().is_empty(),
            "exif must be an empty array"
        );
    }

    // ── diff_passes_gate ───────────────────────────────────────────────────────

    #[test]
    fn diff_passes_gate() {
        assert!(diff_passes(95.0, Some(90.0)), "95 ≥ 90 passes");
        assert!(!diff_passes(85.0, Some(90.0)), "85 < 90 fails");
        assert!(diff_passes(12.0, None), "no gate always passes");
        // Boundary: equal to the threshold passes.
        assert!(diff_passes(90.0, Some(90.0)), "90 ≥ 90 passes");
    }
}
