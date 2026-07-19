//! Single-image and pixel-transform command handlers: `view`, `resize`,
//! `thumbnail`, `auto-orient`, `watermark`, `edit`, and the `meta`
//! container-lane group (`strip`/`clean`/`set`/`copy`) — plus the shared
//! `run_pixel_op` fan-out every pixel-lane handler (here and in `optimize.rs`)
//! replays through (DEC-015). Split out of `cli/mod.rs` (SPEC-097) — no
//! behavior change.

use std::path::{Path, PathBuf};

use crate::error::ImageError;
use crate::image::Image;
use crate::operation::{Gravity, OperationParams, OperationRegistry, RegistryError, Watermark};
use crate::pipeline::Pipeline;
use crate::recipe::Recipe;
use crate::sink::{Overwrite, Sink, SinkError, SinkInput};
use crate::source::{self, SourceError};

use super::common::resolve_format;
use super::optimize::resolve_effective_quality;
use super::{AutoQuality, CliError, GlobalArgs};

/// The `view` path: resolve the single input, load the image, and render it
/// via the display Sink. Resolves the FIRST input when a directory/glob yields
/// many (single-image command). A non-tty stdout refuses with
/// `SinkError::NotATty` → exit 5.
pub(super) fn run_view(
    input: &str,
    width: Option<u32>,
    height: Option<u32>,
    _global: &GlobalArgs,
) -> Result<(), CliError> {
    let resolved = source::resolve(input, &mut std::io::stdin().lock())?;
    let first = resolved
        .into_iter()
        .next()
        .ok_or(CliError::Source(SourceError::NotFound(input.to_owned())))?;
    let img = match &first {
        crate::source::Input::Path(p) => Image::load(p)?,
        crate::source::Input::Stdin { bytes, .. } => Image::from_bytes(bytes)?,
    };
    let sink = Sink::Display { width, height };
    let sink_input = SinkInput {
        stem: first.stem(),
        path: first.path(),
    };
    sink.write(
        &img,
        &sink_input,
        Overwrite::Forbid,
        None,
        &mut std::io::stdout().lock(),
    )?;
    Ok(())
}

// ── Resize helpers ────────────────────────────────────────────────────────────

/// Parse a `WxH` dimension string (e.g. "800x600") into (width, height).
///
/// Both parts must be positive integers separated by a single ASCII 'x'
/// (case-insensitive: 'x' or 'X'). A malformed string (no separator,
/// extra separator, empty part, non-integer, zero, negative, overflow)
/// is a typed usage error → `CliError::Usage` (`code()` == 2).
fn parse_wxh(s: &str) -> Result<(u32, u32), CliError> {
    // Find the single 'x' or 'X' separator.
    let sep_pos = s
        .char_indices()
        .filter(|(_, c)| *c == 'x' || *c == 'X')
        .collect::<Vec<_>>();

    if sep_pos.len() != 1 {
        return Err(CliError::Usage(format!(
            "invalid WxH '{s}': expected exactly one 'x' separator"
        )));
    }

    let idx = sep_pos[0].0;
    let w_str = &s[..idx];
    let h_str = &s[idx + 1..];

    if w_str.is_empty() || h_str.is_empty() {
        return Err(CliError::Usage(format!(
            "invalid WxH '{s}': width and height must be non-empty"
        )));
    }

    let w: u32 = w_str.parse::<u32>().map_err(|_| {
        CliError::Usage(format!(
            "invalid WxH '{s}': width '{w_str}' is not a positive integer"
        ))
    })?;
    let h: u32 = h_str.parse::<u32>().map_err(|_| {
        CliError::Usage(format!(
            "invalid WxH '{s}': height '{h_str}' is not a positive integer"
        ))
    })?;

    if w == 0 {
        return Err(CliError::Usage(format!(
            "invalid WxH '{s}': width must be > 0"
        )));
    }
    if h == 0 {
        return Err(CliError::Usage(format!(
            "invalid WxH '{s}': height must be > 0"
        )));
    }

    Ok((w, h))
}

/// Map the active resize mode flag to the `OperationParams` the registry's
/// "resize" constructor expects (SPEC-010 PINNED schema).
///
/// Exactly one flag is set (clap's ArgGroup guarantees it). WxH strings are
/// parsed via `parse_wxh`. Dim-range validation is the op's job.
fn resize_params(
    max: Option<u32>,
    exact: Option<&str>,
    percent: Option<f32>,
    fit: Option<&str>,
    fill: Option<&str>,
    cover: Option<&str>,
) -> Result<OperationParams, CliError> {
    use std::collections::BTreeMap;

    let mut map: BTreeMap<String, toml::Value> = BTreeMap::new();

    if let Some(n) = max {
        map.insert("mode".into(), toml::Value::String("max".into()));
        map.insert("width".into(), toml::Value::Integer(n as i64));
        return Ok(OperationParams::from_map(map));
    }

    if let Some(s) = exact {
        let (w, h) = parse_wxh(s)?;
        map.insert("mode".into(), toml::Value::String("exact".into()));
        map.insert("width".into(), toml::Value::Integer(w as i64));
        map.insert("height".into(), toml::Value::Integer(h as i64));
        return Ok(OperationParams::from_map(map));
    }

    if let Some(p) = percent {
        map.insert("mode".into(), toml::Value::String("percent".into()));
        map.insert("percent".into(), toml::Value::Float(p as f64));
        return Ok(OperationParams::from_map(map));
    }

    if let Some(s) = fit {
        let (w, h) = parse_wxh(s)?;
        map.insert("mode".into(), toml::Value::String("fit".into()));
        map.insert("width".into(), toml::Value::Integer(w as i64));
        map.insert("height".into(), toml::Value::Integer(h as i64));
        return Ok(OperationParams::from_map(map));
    }

    if let Some(s) = fill {
        let (w, h) = parse_wxh(s)?;
        map.insert("mode".into(), toml::Value::String("fill".into()));
        map.insert("width".into(), toml::Value::Integer(w as i64));
        map.insert("height".into(), toml::Value::Integer(h as i64));
        return Ok(OperationParams::from_map(map));
    }

    if let Some(s) = cover {
        let (w, h) = parse_wxh(s)?;
        map.insert("mode".into(), toml::Value::String("cover".into()));
        map.insert("width".into(), toml::Value::Integer(w as i64));
        map.insert("height".into(), toml::Value::Integer(h as i64));
        return Ok(OperationParams::from_map(map));
    }

    // Defensive: clap ArgGroup guarantees exactly one flag is set.
    Err(CliError::Usage(
        "resize requires exactly one mode flag (--max, --exact, --percent, --fit, --fill, --cover)"
            .into(),
    ))
}

/// Decide the output `ImageFormat` for one input (DEC-015):
///   1. `--format FMT`       → that format (force; FMT via resolve_format).
///   2. else `-o <path>` ext → inferred from the path extension.
///   3. else                 → PRESERVE the input's source_format().
///
/// An unrecognized `--format` is a typed `SinkError` (exit 4) surfaced via
/// `resolve_format`. An unrecognized `-o` extension is `SinkError` (exit 4).
fn output_format_for(
    global: &GlobalArgs,
    output_path: Option<&Path>,
    source_format: ::image::ImageFormat,
) -> Result<::image::ImageFormat, CliError> {
    // 1. Explicit --format wins.
    if let Some(fmt) = resolve_format(global.format.as_deref())? {
        return Ok(fmt);
    }

    // 2. -o <path> extension.
    if let Some(path) = output_path {
        if let Ok(fmt) = crate::sink::format_from_extension(path) {
            return Ok(fmt);
        }
        // If extension is unrecognized, fall through to source format
        // rather than erroring — consistent with single-input behavior.
    }

    // 3. Preserve source format.
    Ok(source_format)
}

// ── resize handler ─────────────────────────────────────────────────────────────

/// The six mutually-exclusive resize-mode flags, bundled so `run_resize` stays
/// within clippy's argument-count limit and the mode set travels as one value.
/// clap's `ArgGroup` guarantees exactly one is `Some` (usage error → exit 2).
pub(super) struct ResizeModes<'a> {
    pub(super) max: Option<u32>,
    pub(super) exact: Option<&'a str>,
    pub(super) percent: Option<f32>,
    pub(super) fit: Option<&'a str>,
    pub(super) fill: Option<&'a str>,
    pub(super) cover: Option<&'a str>,
}

/// Wire the `resize` subcommand: parse flags, build op via registry, fan-out.
///
/// Single-input: uses the `-o`/`-o -`/`--out-dir` sink from global flags.
/// Multi-input: requires `--out-dir`; fan-out is SEQUENTIAL (no rayon, DEC-006).
/// Partial failures in multi-input → continue + print to stderr + exit 6 (DEC-015).
pub(super) fn run_resize(
    inputs: &[String],
    modes: &ResizeModes<'_>,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    // Step 1: build OperationParams from the active flag.
    let params = resize_params(
        modes.max,
        modes.exact,
        modes.percent,
        modes.fit,
        modes.fill,
        modes.cover,
    )?;

    // Step 2: build the op via the registry (same path as recipes, DEC-014).
    // RegistryError → CliError::Usage (exit 2): dim/param rejections are usage errors.
    let op = OperationRegistry::with_builtins()
        .build("resize", &params)
        .map_err(|e| match e {
            RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason),
            RegistryError::Unknown { name } => {
                CliError::Usage(format!("unknown operation '{name}'"))
            }
        })?;

    // Build the pipeline with this single op (builder-style: push consumes self).
    let pipeline = Pipeline::new().push(op);

    run_pixel_op(pipeline, inputs, global, global.quality, None, None)
}

// ── Shared pixel-op fan-out helper ───────────────────────────────────────────

/// Run a built single-op `Pipeline` over one-or-many resolved inputs and
/// write the outputs — the shared CLI fan-out for pixel commands (DEC-015).
///
/// - Resolves every `inputs` arg via `source::resolve`, flattening to one
///   `Vec<Input>`; a resolution error (missing path / empty glob) is a HARD
///   error (exit 3/2), NOT partial-batch; an empty result → `NotFound` (exit 3).
/// - 1 input: single `-o`/`-o -`/`--out-dir` sink, per-input format via
///   `output_format_for`; a failure keeps its natural code (3/1/4/5).
/// - More than 1 input: REQUIRE `--out-dir` (else `CliError::Usage`, exit 2);
///   sequential fan-out; per-input failures collected + stderr + exit 6 (DEC-015).
/// - `quality` is threaded to every `sink.write` call (DEC-016).
/// - `forced_format`: when `Some(fmt)`, override the per-input `output_format_for`
///   resolution with `fmt` for EVERY input. Used by `run_convert` (DEC-015 / SPEC-014).
/// - `auto`: when `Some(mode)`, search the quality per-input on the output pixels
///   (JPEG outputs only; ignored for other formats) instead of using the fixed
///   `quality` — perceptual (`--target`/`--ssim`, SPEC-016) or a byte budget
///   (`--max-size`, SPEC-017). Used by `run_optimize` and `run_convert`.
pub(super) fn run_pixel_op(
    pipeline: Pipeline,
    inputs: &[String],
    global: &GlobalArgs,
    quality: Option<u8>,
    forced_format: Option<::image::ImageFormat>,
    auto: Option<AutoQuality>,
) -> Result<(), CliError> {
    // Resolve every input arg, flattening into one Vec<Input>.
    // Resolution errors (missing path / empty glob) are hard errors (exit 3/2),
    // NOT partial-batch (exit 6). Partial-batch applies only to per-input
    // load/run/write failures AFTER successful resolution.
    let mut all: Vec<crate::source::Input> = Vec::new();
    let mut stdin_lock = std::io::stdin().lock();
    for arg in inputs {
        let resolved = source::resolve(arg, &mut stdin_lock)?;
        all.extend(resolved);
    }

    if all.is_empty() {
        let joined = inputs.join(" ");
        return Err(CliError::Source(SourceError::NotFound(joined)));
    }

    let overwrite = if global.yes {
        Overwrite::Allow
    } else {
        Overwrite::Forbid
    };

    // Single vs. multi by the FLATTENED resolved count.
    if all.len() == 1 {
        let input = &all[0];
        let img = match input {
            crate::source::Input::Path(p) => Image::load(p)?,
            crate::source::Input::Stdin { bytes, .. } => Image::from_bytes(bytes)?,
        };

        let out_img = pipeline.run(img.clone())?;

        // Resolve per-input output format (DEC-015): forced_format wins if Some.
        let output_path = global.output.as_ref().map(|s| Path::new(s.as_str()));
        let fmt = match forced_format {
            Some(f) => f,
            None => output_format_for(global, output_path, img.source_format())?,
        };

        // Build the sink with the resolved format.
        let sink = if let Some(ref out) = global.output {
            if out == "-" {
                Sink::Stdout { format: Some(fmt) }
            } else {
                Sink::File {
                    path: PathBuf::from(out),
                    format: Some(fmt),
                }
            }
        } else if let Some(ref dir) = global.out_dir {
            let template = global
                .name_template
                .clone()
                .unwrap_or_else(|| "{stem}.{ext}".to_owned());
            Sink::Dir {
                dir: PathBuf::from(dir),
                template,
                format: Some(fmt),
            }
        } else {
            // No output flag: default to stdout (format preserved).
            Sink::Stdout { format: Some(fmt) }
        };

        // Resolve the effective quality (auto-quality search for JPEG, else fixed).
        let label = input
            .path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| input.stem().to_owned());
        let plan = resolve_effective_quality(quality, &auto, fmt, &out_img, global, &label)?;
        // A `--max-size` downscale (SPEC-021) replaces the pixels we write.
        let write_img = plan.image.as_ref().unwrap_or(&out_img);

        let sink_input = SinkInput {
            stem: input.stem(),
            path: input.path(),
        };
        sink.write(
            write_img,
            &sink_input,
            overwrite,
            plan.quality,
            &mut std::io::stdout().lock(),
        )?;
    } else {
        // Multi-input: require --out-dir.
        let out_dir = global
            .out_dir
            .as_ref()
            .ok_or_else(|| CliError::Usage("multiple inputs require --out-dir".into()))?;

        let template = global
            .name_template
            .clone()
            .unwrap_or_else(|| "{stem}.{ext}".to_owned());

        let total = all.len();
        let mut failed: usize = 0;

        for input in &all {
            // Label for error messages.
            let label = match input {
                crate::source::Input::Path(p) => p.display().to_string(),
                crate::source::Input::Stdin { stem, .. } => stem.clone(),
            };

            // Load, run, resolve format, build sink, write — catch per-input errors.
            let result = (|| -> Result<(), CliError> {
                let img = match input {
                    crate::source::Input::Path(p) => Image::load(p)?,
                    crate::source::Input::Stdin { bytes, .. } => Image::from_bytes(bytes)?,
                };

                let out_img = pipeline.run(img.clone())?;

                // Per-input format resolution (DEC-015): forced_format wins if Some; no -o path in fan-out.
                let fmt = match forced_format {
                    Some(f) => f,
                    None => output_format_for(global, None, img.source_format())?,
                };

                let sink = Sink::Dir {
                    dir: PathBuf::from(out_dir),
                    template: template.clone(),
                    format: Some(fmt),
                };

                // Per-input effective plan (auto-quality search + optional
                // `--max-size` downscale). A failure here is per-input → exit 6.
                let plan =
                    resolve_effective_quality(quality, &auto, fmt, &out_img, global, &label)?;
                let write_img = plan.image.as_ref().unwrap_or(&out_img);

                let sink_input = SinkInput {
                    stem: input.stem(),
                    path: input.path(),
                };
                sink.write(
                    write_img,
                    &sink_input,
                    overwrite,
                    plan.quality,
                    &mut std::io::stdout().lock(),
                )?;
                Ok(())
            })();

            if let Err(e) = result {
                eprintln!("error: {label}: {e}");
                failed += 1;
            }
        }

        if failed > 0 {
            return Err(CliError::PartialBatch { failed, total });
        }
    }

    Ok(())
}

// ── Metadata lane (container lane, SPEC-026) ──────────────────────────────────

/// The lowercase extension to preserve for one container-lane input (`{ext}` in
/// a `--out-dir` template). The format is never transcoded, so we keep the
/// input's own extension: a path's extension, or — for stdin / a missing
/// extension — sniff it from the bytes (`jpg`/`png`).
pub(super) fn metadata_output_ext(input: &crate::source::Input, bytes: &[u8]) -> String {
    if let crate::source::Input::Path(p) = input {
        if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            if !ext.is_empty() {
                return ext.to_ascii_lowercase();
            }
        }
    }
    // Stdin or no extension: sniff from the bytes (no decode).
    match ::image::guess_format(bytes) {
        Ok(::image::ImageFormat::Png) => "png".to_owned(),
        _ => "jpg".to_owned(),
    }
}

/// Read the raw container bytes for one resolved input WITHOUT decoding pixels.
///
/// `Input::Path` → `std::fs::read` (an I/O error after `source::resolve` already
/// confirmed the path maps to exit 3 via [`ImageError::Io`]); `Input::Stdin` →
/// the already-buffered bytes.
pub(super) fn read_raw_bytes(input: &crate::source::Input) -> Result<Vec<u8>, CliError> {
    match input {
        crate::source::Input::Path(p) => Ok(std::fs::read(p).map_err(ImageError::Io)?),
        crate::source::Input::Stdin { bytes, .. } => Ok(bytes.clone()),
    }
}

/// The shared container-lane fan-out (SPEC-026), mirroring [`run_pixel_op`] but
/// reading RAW bytes and transforming via a byte→byte metadata `transform`
/// (no pixel decode — `metadata-not-via-pixel-encode`).
///
/// - Resolves every `inputs` arg via `source::resolve`; a resolution error
///   (missing path / empty glob) is a HARD error (exit 3/2), not partial-batch.
/// - 1 input: write to `-o PATH`, else `--out-dir` (templated), else stdout
///   (raw bytes). A failure keeps its natural code (3/4/5/1).
/// - More than 1 input: REQUIRE `--out-dir` (else exit 2); per-input failures
///   print to stderr and yield exit 6 (DEC-015). Format is always preserved.
fn run_metadata_lane(
    inputs: &[String],
    global: &GlobalArgs,
    transform: impl Fn(&[u8]) -> Result<Vec<u8>, crate::metadata::MetadataError>,
) -> Result<(), CliError> {
    // Resolve + flatten every input arg (resolution errors are hard errors).
    let mut all: Vec<crate::source::Input> = Vec::new();
    let mut stdin_lock = std::io::stdin().lock();
    for arg in inputs {
        let resolved = source::resolve(arg, &mut stdin_lock)?;
        all.extend(resolved);
    }
    if all.is_empty() {
        return Err(CliError::Source(SourceError::NotFound(inputs.join(" "))));
    }

    let overwrite = if global.yes {
        Overwrite::Allow
    } else {
        Overwrite::Forbid
    };

    if all.len() == 1 {
        let input = &all[0];
        let raw = read_raw_bytes(input)?;
        let out_bytes = transform(&raw)?;
        let ext = metadata_output_ext(input, &raw);

        let sink = if let Some(ref out) = global.output {
            if out == "-" {
                Sink::Stdout { format: None }
            } else {
                Sink::File {
                    path: PathBuf::from(out),
                    format: None,
                }
            }
        } else if let Some(ref dir) = global.out_dir {
            let template = global
                .name_template
                .clone()
                .unwrap_or_else(|| "{stem}.{ext}".to_owned());
            Sink::Dir {
                dir: PathBuf::from(dir),
                template,
                format: None,
            }
        } else {
            // No output flag: default to stdout (raw container bytes).
            Sink::Stdout { format: None }
        };

        let sink_input = SinkInput {
            stem: input.stem(),
            path: input.path(),
        };
        sink.write_bytes(
            &out_bytes,
            &sink_input,
            &ext,
            overwrite,
            &mut std::io::stdout().lock(),
        )?;
    } else {
        // Multi-input: require --out-dir.
        let out_dir = global
            .out_dir
            .as_ref()
            .ok_or_else(|| CliError::Usage("multiple inputs require --out-dir".into()))?;
        let template = global
            .name_template
            .clone()
            .unwrap_or_else(|| "{stem}.{ext}".to_owned());

        let total = all.len();
        let mut failed: usize = 0;

        for input in &all {
            let label = match input {
                crate::source::Input::Path(p) => p.display().to_string(),
                crate::source::Input::Stdin { stem, .. } => stem.clone(),
            };

            let result = (|| -> Result<(), CliError> {
                let raw = read_raw_bytes(input)?;
                let out_bytes = transform(&raw)?;
                let ext = metadata_output_ext(input, &raw);

                let sink = Sink::Dir {
                    dir: PathBuf::from(out_dir),
                    template: template.clone(),
                    format: None,
                };
                let sink_input = SinkInput {
                    stem: input.stem(),
                    path: input.path(),
                };
                sink.write_bytes(
                    &out_bytes,
                    &sink_input,
                    &ext,
                    overwrite,
                    &mut std::io::stdout().lock(),
                )?;
                Ok(())
            })();

            if let Err(e) = result {
                eprintln!("error: {label}: {e}");
                failed += 1;
            }
        }

        if failed > 0 {
            return Err(CliError::PartialBatch { failed, total });
        }
    }

    Ok(())
}

/// Wire `meta strip`: remove ALL container metadata via the container lane
/// (DEC-003). Format is preserved; no pixel re-encode (`metadata-not-via-pixel-encode`).
pub(super) fn run_strip(inputs: &[String], global: &GlobalArgs) -> Result<(), CliError> {
    run_metadata_lane(inputs, global, crate::metadata::strip_all)
}

/// Wire `meta clean --gps`: remove ONLY GPS/location metadata via the container
/// lane. `--gps` is required in v1; `meta clean` without it is a usage error (exit
/// 2), leaving room for future selective flags.
pub(super) fn run_clean(inputs: &[String], gps: bool, global: &GlobalArgs) -> Result<(), CliError> {
    if !gps {
        return Err(CliError::Usage("clean requires --gps".into()));
    }
    run_metadata_lane(inputs, global, crate::metadata::clean_gps)
}

/// Wire `meta set`: write the given EXIF attribution tags into the container
/// via the container lane (DEC-003), preserving every other tag and the
/// pixels exactly (no re-encode, `metadata-not-via-pixel-encode`).
///
/// At least one of `--artist`/`--copyright`/`--description` is required; none is
/// a usage error (exit 2). Format is preserved; `-q`/`--format` are ignored.
pub(super) fn run_set(
    inputs: &[String],
    artist: Option<String>,
    copyright: Option<String>,
    description: Option<String>,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    if artist.is_none() && copyright.is_none() && description.is_none() {
        return Err(CliError::Usage(
            "meta set requires at least one of --artist/--copyright/--description".into(),
        ));
    }
    let tags = crate::metadata::TagSet {
        artist,
        copyright,
        description,
    };
    run_metadata_lane(inputs, global, |bytes| {
        crate::metadata::set_tags(bytes, &tags)
    })
}

/// Wire `meta copy --from SRC --to DST`: graft SRC's container EXIF + ICC
/// onto DST via the container lane (DEC-003, DEC-030), preserving DST's pixels
/// exactly (no re-encode, `metadata-not-via-pixel-encode`). JPEG only in v1; a
/// non-JPEG `--from`/`--to` is a [`MetadataError::UnsupportedFormat`] → exit 4.
///
/// This is NOT a fan-out: `--from`/`--to` are each a SINGLE literal path (read
/// directly with `std::fs::read`, no globbing; a missing/unreadable path → exit
/// 3). The output is a single fixed target:
/// - `-o PATH` → write the grafted result there (DST untouched);
/// - `-o -` → write to stdout (raw bytes);
/// - default (no `-o`) → write back to DST IN PLACE, which already exists, so it
///   is refused without `--yes` (exit 5) and overwrites with it.
pub(super) fn run_copy_metadata(from: &str, to: &str, global: &GlobalArgs) -> Result<(), CliError> {
    // Read both inputs directly (no source::resolve glob fan-out). An I/O error
    // (missing/unreadable path) maps to exit 3 via ImageError::Io.
    let from_bytes = std::fs::read(from).map_err(ImageError::Io)?;
    let to_bytes = std::fs::read(to).map_err(ImageError::Io)?;

    let out_bytes = crate::metadata::copy_metadata(&from_bytes, &to_bytes)?;

    // The output extension: DST's own extension, or sniff (format is preserved).
    let ext = {
        let p = Path::new(to);
        match p.extension().and_then(|e| e.to_str()) {
            Some(e) if !e.is_empty() => e.to_ascii_lowercase(),
            _ => match ::image::guess_format(&to_bytes) {
                Ok(::image::ImageFormat::Png) => "png".to_owned(),
                _ => "jpg".to_owned(),
            },
        }
    };

    // Build the output sink: -o PATH → File, -o - → Stdout, else in-place (DST).
    let sink = match global.output.as_deref() {
        Some("-") => Sink::Stdout { format: None },
        Some(path) => Sink::File {
            path: PathBuf::from(path),
            format: None,
        },
        None => Sink::File {
            path: PathBuf::from(to),
            format: None,
        },
    };

    let overwrite = if global.yes {
        Overwrite::Allow
    } else {
        Overwrite::Forbid
    };

    // `to` is the naming context; stem only matters for Dir sinks (unused here).
    let to_path = Path::new(to);
    let sink_input = SinkInput {
        stem: to_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("output"),
        path: Some(to_path),
    };
    sink.write_bytes(
        &out_bytes,
        &sink_input,
        &ext,
        overwrite,
        &mut std::io::stdout().lock(),
    )?;
    Ok(())
}

// ── edit helpers (SPEC-032) ───────────────────────────────────────────────────

/// Build the ordered op list for `edit` from the active op flags (SPEC-032).
///
/// Ops are appended in the canonical order regardless of which flags are set:
///
/// 1. `auto-orient` (orientation normalization → pixels)
/// 2. `resize`      (geometry — `--resize-max N`)
/// 3. `invert`      (color)
///
/// Each op is built through `OperationRegistry::with_builtins` (DEC-005) so the
/// returned ops carry the same `name()` + `params()` that `Recipe::from_ops`
/// records and `apply` can replay.
///
/// Errors:
/// - Zero flags set → `CliError::Usage` (exit 2).
/// - `resize` params rejected → `CliError::Usage` (exit 2).
/// - Any other `RegistryError` (defensive) → `CliError::Usage` (exit 2).
fn build_edit_ops(
    auto_orient: bool,
    resize_max: Option<u32>,
    invert: bool,
) -> Result<Vec<Box<dyn crate::operation::Operation>>, CliError> {
    let registry = OperationRegistry::with_builtins();
    let mut ops: Vec<Box<dyn crate::operation::Operation>> = Vec::new();

    if auto_orient {
        let op = registry
            .build("auto-orient", &OperationParams::empty())
            .map_err(|e| match e {
                RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason),
                RegistryError::Unknown { name } => {
                    CliError::Usage(format!("unknown operation '{name}'"))
                }
            })?;
        ops.push(op);
    }

    if let Some(n) = resize_max {
        let params = resize_params(Some(n), None, None, None, None, None)?;
        let op = registry.build("resize", &params).map_err(|e| match e {
            RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason),
            RegistryError::Unknown { name } => {
                CliError::Usage(format!("unknown operation '{name}'"))
            }
        })?;
        ops.push(op);
    }

    if invert {
        let op = registry
            .build("invert", &OperationParams::empty())
            .map_err(|e| match e {
                RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason),
                RegistryError::Unknown { name } => {
                    CliError::Usage(format!("unknown operation '{name}'"))
                }
            })?;
        ops.push(op);
    }

    if ops.is_empty() {
        return Err(CliError::Usage(
            "edit requires at least one operation flag (--auto-orient, --resize-max, --invert)"
                .into(),
        ));
    }

    Ok(ops)
}

/// Wire the `edit` subcommand: build an ordered op pipeline from the active
/// flags, run it on `input`, write the result, and — when `--save-recipe` is
/// given — serialize the op chain to a TOML recipe file (SPEC-032, DEC-005).
///
/// Flow (order matters for the round-trip + write-after-success guarantee):
///
/// 1. Build the ordered ops via `build_edit_ops` (canonical order; DEC-005).
/// 2. Capture the recipe object NOW before moving ops into the pipeline:
///    `Recipe::from_ops(&ops)` borrows `&[Box<dyn Operation>]`; this must
///    precede the `into_iter().fold` that consumes `ops`.
/// 3. Fold ops into a `Pipeline`.
/// 4. Delegate to `run_pixel_op` for the full load→run→sink fan-out (DEC-015).
/// 5. On success, if `--save-recipe` was given, serialize + write the recipe.
///    A serialization failure → `CliError::Recipe` (exit 1); an I/O write
///    failure → `CliError::Sink(SinkError::Io)` (exit 5). An orphan recipe is
///    never written when the edit itself fails.
pub(super) fn run_edit(
    input: &str,
    auto_orient: bool,
    resize_max: Option<u32>,
    invert: bool,
    save_recipe: Option<&str>,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    // 1. Build ordered ops (canonical order; fails on zero flags or bad params).
    let ops = build_edit_ops(auto_orient, resize_max, invert)?;

    // 2. Capture the recipe before consuming ops (from_ops borrows &[Box<dyn Op>]).
    let recipe = save_recipe.map(|_| Recipe::from_ops(&ops));

    // 3. Fold ops into a Pipeline.
    let pipeline = ops.into_iter().fold(Pipeline::new(), |p, op| p.push(op));

    // 4. Load → run → sink via the established single/multi fan-out helper.
    //    `input` is a &str; wrap it in a one-element Vec<String> slice.
    let input_vec = [input.to_owned()];
    run_pixel_op(pipeline, &input_vec, global, global.quality, None, None)?;

    // 5. On success, write the recipe file if requested.
    if let (Some(path), Some(r)) = (save_recipe, recipe) {
        // Guard: refuse to write through a symlink (DEC-035, SPEC-037).
        crate::sink::reject_symlink_destination(std::path::Path::new(path))?;
        let toml = r.to_toml()?;
        std::fs::write(path, toml).map_err(|e| CliError::Sink(SinkError::Io(e)))?;
    }

    Ok(())
}

// ── thumbnail helpers ─────────────────────────────────────────────────────────

/// The default long-edge bound when `--size` is omitted.
const DEFAULT_THUMBNAIL_SIZE: u32 = 256;

/// Map thumbnail args to the `Resize` OperationParams the registry expects
/// (SPEC-010's PINNED schema). `thumbnail` is a convenience over `resize`:
///
/// - `--square` → resize `fill` N×N  (cover + center-crop to exactly N×N)
/// - else       → resize `max`  N     (bound the long edge to N, no upscale)
///
/// `size` defaults to `DEFAULT_THUMBNAIL_SIZE` (256). Infallible: the mapping
/// is total; the op validates the dims.
fn thumbnail_params(size: Option<u32>, square: bool) -> OperationParams {
    use std::collections::BTreeMap;

    let n = size.unwrap_or(DEFAULT_THUMBNAIL_SIZE);
    let mut map: BTreeMap<String, toml::Value> = BTreeMap::new();

    if square {
        map.insert("mode".into(), toml::Value::String("fill".into()));
        map.insert("width".into(), toml::Value::Integer(n as i64));
        map.insert("height".into(), toml::Value::Integer(n as i64));
    } else {
        map.insert("mode".into(), toml::Value::String("max".into()));
        map.insert("width".into(), toml::Value::Integer(n as i64));
    }

    OperationParams::from_map(map)
}

// ── thumbnail handler ─────────────────────────────────────────────────────────

/// Wire the `thumbnail` subcommand: map `(size, square)` to `Resize` params,
/// build the op via the registry, and delegate to `run_pixel_op` for the
/// full multi-input fan-out (DEC-015).
///
/// - `--size N` (default 256) bounds the longest edge to N, aspect preserved.
/// - `--square` produces an exactly N×N output via cover+center-crop (`fill`).
/// - `--size 0` → op rejects width 0 → `CliError::Usage` (exit 2).
pub(super) fn run_thumbnail(
    inputs: &[String],
    size: Option<u32>,
    square: bool,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    let params = thumbnail_params(size, square);

    let op = OperationRegistry::with_builtins()
        .build("resize", &params)
        .map_err(|e| match e {
            RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason),
            RegistryError::Unknown { name } => {
                CliError::Usage(format!("unknown operation '{name}'"))
            }
        })?;

    let pipeline = Pipeline::new().push(op);
    run_pixel_op(pipeline, inputs, global, global.quality, None, None)
}

// ── auto-orient handler ───────────────────────────────────────────────────────

/// Wire the `auto-orient` subcommand: build the `AutoOrient` op via the
/// registry and delegate to `run_pixel_op` for the full multi-input fan-out.
///
/// The op is parameterless; no forced format (source format is preserved);
/// quality is threaded from `global.quality` with no forced default (DEC-016).
/// Images with no EXIF, no orientation tag, or orientation 1 are returned
/// unchanged (no-op, exit 0 — not an error). After baking, the metadata bundle
/// is dropped by the op itself (DEC-017).
pub(super) fn run_auto_orient(inputs: &[String], global: &GlobalArgs) -> Result<(), CliError> {
    let op = OperationRegistry::with_builtins()
        .build("auto-orient", &OperationParams::empty())
        .map_err(|e| match e {
            RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason),
            RegistryError::Unknown { name } => {
                CliError::Usage(format!("unknown operation '{name}'"))
            }
        })?;

    let pipeline = Pipeline::new().push(op);
    run_pixel_op(pipeline, inputs, global, global.quality, None, None)
}

// ── watermark handler (SPEC-029, DEC-031) ─────────────────────────────────────

/// Wire the `watermark` subcommand — the IO boundary for the first multi-image
/// `Operation` (DEC-031).
///
/// The overlay is loaded ONCE here via `Image::load` (a missing/unreadable/
/// undecodable logo → exit 3) and handed to the op as in-memory pixels, so
/// `Watermark::apply` never touches a file. `--opacity`/`--scale`/`--gravity`
/// are validated BEFORE constructing the op (out-of-range → `CliError::Usage`,
/// exit 2). The op is then run through the standard `run_pixel_op` fan-out
/// (single → stdout/`-o`/`--out-dir`, multi → `--out-dir`, exit 6 on per-input
/// failure — DEC-015), reusing the GLOBAL `-o`/`--out-dir`/`-q`/`-y` flags.
///
/// `watermark` is NOT registered in `with_builtins()` (recipe round-trip is
/// STAGE-005, DEC-031), so the op is constructed directly rather than via the
/// registry.
#[allow(clippy::too_many_arguments)]
/// The watermark overlay source: an image path (SPEC-029) OR text + its rendering
/// flags (SPEC-030). clap enforces `--image` XOR `--text` (exactly one); the other
/// fields are the text-mode flags. Bundled so `run_watermark` stays within clippy's
/// argument-count limit and the mode set travels as one value.
pub(super) struct WatermarkSource<'a> {
    pub(super) image: Option<&'a str>,
    pub(super) text: Option<&'a str>,
    pub(super) font: Option<&'a str>,
    pub(super) size: Option<f32>,
    pub(super) color: Option<&'a str>,
}

/// Map a `text::TextError` to a CLI error. Color/empty/font-parse are usage errors
/// (exit 2); the `--font` *file* read failure is handled separately at the IO
/// boundary as an `Image`/IO load error (exit 3).
fn text_error(e: crate::text::TextError) -> CliError {
    CliError::Usage(e.to_string())
}

/// Build the watermark overlay (`DynamicImage`) + its label from the source mode.
///
/// - Image mode (`--image PATH`): load the overlay once at the IO boundary
///   (→ exit 3 on failure, DEC-031); label is the path (for `params()` round-trip).
/// - Text mode (`--text STR`): read `--font PATH` at the IO boundary (→ exit 3) or
///   fall back to the bundled font; parse `--color` (default `ffffff`) and `--size`
///   (default 32.0, `≤0` → exit 2); rasterize via `text::render_text` (pure) into a
///   transparent RGBA overlay (→ exit 2 on a text error). The label is the text.
fn watermark_overlay(
    src: &WatermarkSource<'_>,
) -> Result<(::image::DynamicImage, String), CliError> {
    if let Some(image) = src.image {
        // Image mode: load the overlay once at the IO boundary (DEC-031).
        let overlay = Image::load(image)?;
        return Ok((overlay.pixels().clone(), image.to_owned()));
    }

    // Text mode. clap guarantees `--text` is present when `--image` is not.
    let text = src.text.unwrap_or("");

    // Load the font at the IO boundary (--font → exit 3) or use the bundled default.
    let font_owned: Option<Vec<u8>> = match src.font {
        Some(path) => Some(std::fs::read(path).map_err(ImageError::Io)?),
        None => None,
    };
    let font_bytes: &[u8] = match font_owned.as_deref() {
        Some(b) => b,
        None => crate::text::DEFAULT_FONT,
    };

    // Color (default white) and size (default 32.0; ≤0 → exit 2).
    let color = match src.color {
        Some(s) => crate::text::parse_color(s).map_err(text_error)?,
        None => [255, 255, 255, 255],
    };
    let size = src.size.unwrap_or(32.0);
    if size <= 0.0 {
        return Err(CliError::Usage(format!("--size must be > 0, got {size}")));
    }

    // Rasterize the text into a transparent RGBA overlay (pure; no file IO).
    let rendered = crate::text::render_text(font_bytes, text, size, color).map_err(text_error)?;
    Ok((::image::DynamicImage::ImageRgba8(rendered), text.to_owned()))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run_watermark(
    inputs: &[String],
    src: &WatermarkSource<'_>,
    gravity: Option<&str>,
    opacity: Option<f32>,
    scale: Option<f32>,
    margin: Option<u32>,
    tile: bool,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    // Build the overlay (image load OR text rasterization) + its label.
    let (overlay, label) = watermark_overlay(src)?;

    // Validate placement params BEFORE constructing the op (→ Usage, exit 2).
    let gravity: Gravity = gravity
        .unwrap_or("southeast")
        .parse()
        .map_err(CliError::Usage)?;

    let opacity = opacity.unwrap_or(1.0);
    if !(0.0..=1.0).contains(&opacity) {
        return Err(CliError::Usage(format!(
            "--opacity must be in 0.0..=1.0, got {opacity}"
        )));
    }

    if let Some(s) = scale {
        if s <= 0.0 {
            return Err(CliError::Usage(format!("--scale must be > 0, got {s}")));
        }
    }

    let margin = margin.unwrap_or(0);

    // Build the op directly (NOT via the registry — DEC-031) with the decoded /
    // rendered overlay pixels; the text/image label is kept for `params()`.
    let op = Watermark::new(overlay, label, gravity, opacity, scale, margin, tile);

    let pipeline = Pipeline::new().push(Box::new(op));
    run_pixel_op(pipeline, inputs, global, global.quality, None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── parse_wxh tests ───────────────────────────────────────────────────────

    #[test]
    fn parse_wxh_parses_valid() {
        assert_eq!(parse_wxh("800x600").unwrap(), (800, 600));
        // Uppercase X accepted.
        assert_eq!(parse_wxh("1920X1080").unwrap(), (1920, 1080));
    }

    #[test]
    fn parse_wxh_rejects_malformed() {
        let bad = [
            "abc",
            "800x",
            "x600",
            "800",
            "800x600x1",
            "0x10",
            "800x0",
            "-1x10",
            "",
        ];
        for s in bad {
            let result = parse_wxh(s);
            assert!(result.is_err(), "expected Err for '{s}', got Ok");
            assert_eq!(
                result.unwrap_err().code(),
                2,
                "parse_wxh error for '{s}' should have code 2"
            );
        }
    }

    // ── resize_params tests ───────────────────────────────────────────────────

    #[test]
    fn resize_params_max_minimal() {
        let p = resize_params(Some(20), None, None, None, None, None).unwrap();
        assert_eq!(p.get_str("mode"), Some("max"));
        assert_eq!(p.get_u32("width"), Some(20));
        // Must NOT have height or percent.
        assert!(
            p.get_u32("height").is_none(),
            "max mode must not have height"
        );
        assert!(
            p.get_f32("percent").is_none(),
            "max mode must not have percent"
        );
    }

    #[test]
    fn resize_params_exact_has_both_dims() {
        let p = resize_params(None, Some("33x77"), None, None, None, None).unwrap();
        assert_eq!(p.get_str("mode"), Some("exact"));
        assert_eq!(p.get_u32("width"), Some(33));
        assert_eq!(p.get_u32("height"), Some(77));
    }

    #[test]
    fn resize_params_percent() {
        let p = resize_params(None, None, Some(50.0), None, None, None).unwrap();
        assert_eq!(p.get_str("mode"), Some("percent"));
        // get_f32 accepts Float TOML values.
        let pct = p.get_f32("percent").expect("percent key must be present");
        assert!((pct - 50.0).abs() < 0.001, "percent value mismatch: {pct}");
    }

    #[test]
    fn resize_params_fit_fill_cover() {
        for (mode, flag_fit, flag_fill, flag_cover) in [
            ("fit", Some("40x40"), None, None),
            ("fill", None, Some("40x40"), None),
            ("cover", None, None, Some("40x40")),
        ] {
            let p = resize_params(None, None, None, flag_fit, flag_fill, flag_cover).unwrap();
            assert_eq!(p.get_str("mode"), Some(mode), "mode mismatch for {mode}");
            assert_eq!(p.get_u32("width"), Some(40), "width mismatch for {mode}");
            assert_eq!(p.get_u32("height"), Some(40), "height mismatch for {mode}");
        }
    }

    #[test]
    fn resize_params_bad_wxh_is_usage() {
        let result = resize_params(None, Some("nope"), None, None, None, None);
        assert!(result.is_err(), "malformed WxH should be an error");
        assert_eq!(
            result.unwrap_err().code(),
            2,
            "malformed WxH should be code 2"
        );
    }

    // ── output_format_for tests ───────────────────────────────────────────────

    fn make_global(format: Option<&str>) -> GlobalArgs {
        GlobalArgs {
            output: None,
            out_dir: None,
            name_template: None,
            jobs: None,
            format: format.map(|s| s.to_owned()),
            quality: None,
            verbose: 0,
            quiet: false,
            yes: false,
            keep_gps: false,
            no_cache: false,
            check: false,
            strict: false,
            watch: false,
        }
    }

    #[test]
    fn output_format_for_format_flag_wins() {
        let global = make_global(Some("png"));
        let result = output_format_for(
            &global,
            Some(Path::new("/x/a.jpg")),
            ::image::ImageFormat::Jpeg,
        )
        .unwrap();
        assert_eq!(
            result,
            ::image::ImageFormat::Png,
            "--format png must win over .jpg path and Jpeg source"
        );
    }

    #[test]
    fn output_format_for_path_ext() {
        let global = make_global(None);
        let result = output_format_for(
            &global,
            Some(Path::new("/x/a.png")),
            ::image::ImageFormat::Jpeg,
        )
        .unwrap();
        assert_eq!(
            result,
            ::image::ImageFormat::Png,
            ".png path ext should override Jpeg source"
        );
    }

    #[test]
    fn output_format_for_preserves_source() {
        let global = make_global(None);
        let result = output_format_for(&global, None, ::image::ImageFormat::Jpeg).unwrap();
        assert_eq!(
            result,
            ::image::ImageFormat::Jpeg,
            "source format should be preserved when no override"
        );
    }

    // ── thumbnail_params tests ────────────────────────────────────────────────

    #[test]
    fn thumbnail_params_max_default() {
        let p = thumbnail_params(None, false);
        assert_eq!(p.get_str("mode"), Some("max"));
        assert_eq!(p.get_u32("width"), Some(256));
        assert!(
            p.get_u32("height").is_none(),
            "max mode must not have height"
        );
    }

    #[test]
    fn thumbnail_params_max_sized() {
        let p = thumbnail_params(Some(64), false);
        assert_eq!(p.get_str("mode"), Some("max"));
        assert_eq!(p.get_u32("width"), Some(64));
        assert!(
            p.get_u32("height").is_none(),
            "max mode must not have height"
        );
    }

    #[test]
    fn thumbnail_params_square_default() {
        let p = thumbnail_params(None, true);
        assert_eq!(p.get_str("mode"), Some("fill"));
        assert_eq!(p.get_u32("width"), Some(256));
        assert_eq!(p.get_u32("height"), Some(256));
    }

    #[test]
    fn thumbnail_params_square_sized() {
        let p = thumbnail_params(Some(64), true);
        assert_eq!(p.get_str("mode"), Some("fill"));
        assert_eq!(p.get_u32("width"), Some(64));
        assert_eq!(p.get_u32("height"), Some(64));
    }

    // ── SPEC-032: edit + --save-recipe ────────────────────────────────────────

    /// `build_edit_ops` with all three flags set returns ops in the canonical
    /// order: auto-orient → resize → invert, regardless of arg order.
    #[test]
    fn edit_ops_canonical_order() {
        let ops = build_edit_ops(true, Some(8), true).expect("should succeed");
        assert_eq!(ops.len(), 3, "expected 3 ops");
        assert_eq!(ops[0].name(), "auto-orient");
        assert_eq!(ops[1].name(), "resize");
        assert_eq!(ops[2].name(), "invert");
    }

    /// Only resize + invert flags set → two ops in canonical order (no auto-orient).
    #[test]
    fn edit_ops_subset_order() {
        let ops = build_edit_ops(false, Some(8), true).expect("should succeed");
        assert_eq!(ops.len(), 2, "expected 2 ops");
        assert_eq!(ops[0].name(), "resize");
        assert_eq!(ops[1].name(), "invert");
    }

    /// No flags set → `CliError::Usage` ("requires at least one operation flag").
    #[test]
    fn edit_ops_requires_at_least_one() {
        let result = build_edit_ops(false, None, false);
        assert!(result.is_err(), "expected Err");
        // Extract the error without relying on Debug on dyn Operation.
        let err = result.err().expect("is_err was just confirmed");
        assert_eq!(err.code(), 2, "usage error must be code 2");
        assert!(
            err.to_string()
                .contains("requires at least one operation flag"),
            "message must mention 'requires at least one operation flag': {err}"
        );
    }

    /// The resize op built by `build_edit_ops(resize_max=Some(16))` carries the
    /// same params as `run_resize` builds for `--max 16`: `{mode:"max", width:16}`.
    /// This pins the round-trip equivalence between `edit` and `apply`.
    #[test]
    fn edit_ops_resize_params_match_resize_command() {
        // Build the op via build_edit_ops (edit path).
        let edit_ops = build_edit_ops(false, Some(16), false).expect("should succeed");
        assert_eq!(edit_ops.len(), 1);
        let edit_params = edit_ops[0].params();

        // Build the equivalent op the way run_resize does it (registry path).
        let resize_params_val =
            resize_params(Some(16), None, None, None, None, None).expect("resize_params ok");
        let registry_op = OperationRegistry::with_builtins()
            .build("resize", &resize_params_val)
            .expect("build resize ok");
        let registry_params = registry_op.params();

        // Both must carry identical TOML params.
        assert_eq!(
            edit_params, registry_params,
            "edit resize params must match run_resize params for --max 16"
        );
    }
}
