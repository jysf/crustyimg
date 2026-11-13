//! Single-image and pixel-transform command handlers: `view`, `resize`,
//! `thumbnail`, `auto-orient`, `watermark`, `edit`, and the `meta`
//! container-lane group (`strip`/`clean`/`set`/`copy`) — plus the shared
//! `run_pixel_op` fan-out every pixel-lane handler (here and in `optimize.rs`)
//! replays through (DEC-015). Split out of `cli/mod.rs` (SPEC-097) — no
//! behavior change.

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::error::ImageError;
use crate::image::Image;
use crate::operation::registry::param_spec;
use crate::operation::{
    Crop, Gravity, IntroducedAlpha, InvisibleMark, Operation, OperationError, OperationParams,
    OperationRegistry, Pad, Refusal, RegistryError, Watermark, MAX_AREA, MAX_EDGE,
    WATERMARK_PARAMS,
};
use crate::pipeline::{OnRefusal, Pipeline, SkippedStep};
use crate::recipe::{Recipe, RecipeStep};
use crate::sink::{Overwrite, Sink, SinkError, SinkInput};
use crate::source::{self, OnEmpty, SourceError};

use super::common::{
    name_pinned_format, refuse_folder, report_skipped_images, resolve_format, warn_skipped_steps,
};
use super::optimize::{auto_orient_prefix, pipeline_altered_source, resolve_effective_quality};
use super::{AutoQuality, CliError, EditOp, GlobalArgs};

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
    let resolved = source::resolve_all(&[input], &mut std::io::stdin().lock(), OnEmpty::Fail)?;
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

/// Decide the output `ImageFormat` for one input (DEC-015, extended by
/// SPEC-127 Call 2):
///   1. `--format FMT`       → that format (force; FMT via resolve_format).
///   2. else the output name's recognised extension: `-o <path>` when given,
///      else the `--out-dir` name `template` (`{stem}.png` pins PNG; a
///      template with `{ext}` never pins).
///   3. else `recipe.format` → the recipe's own pin, if any (SPEC-127).
///   4. else                 → PRESERVE the input's source_format().
///
/// An unrecognized `--format` is a typed `SinkError` (exit 4) surfaced via
/// `resolve_format`. An unrecognized `-o` or template extension names the file
/// only and falls through to rung 3. An unrecognized `recipe_format` string is
/// a typed `SinkError`, but ONLY reached when rungs 1-2 are both absent
/// (matching how an output-name pin already short-circuits past whatever
/// `recipe.format` says, unvalidated, when it wins).
///
/// `template` is the name template the output is written under, or `None`
/// when the output is not written into `--out-dir` (with `-o` given, the
/// template is not the output's name and is never consulted).
///
/// `pub(super)` (SPEC-126): `apply`'s single-input path (`optimize::run_apply`)
/// reuses this exact resolution rather than the `Sink::Dir`
/// default-to-PNG fallback it used to fall through to — so `apply` at one
/// input resolves format identically to `resize`/`thumbnail`/`watermark`.
/// Those three callers (via `run_pixel_op`, below) have no recipe at all and
/// pass `recipe_format: None`, leaving rungs 1/2/4 exactly as before SPEC-127.
pub(super) fn output_format_for(
    global: &GlobalArgs,
    output_path: Option<&Path>,
    template: Option<&str>,
    recipe_format: Option<&str>,
    source_format: ::image::ImageFormat,
) -> Result<::image::ImageFormat, CliError> {
    // 1-2. The format the flags and the output name fix.
    if let Some(fmt) = format_fixed_by_flags(global, output_path, template)? {
        return Ok(fmt);
    }

    // 3. The recipe's own `format` (SPEC-127, Call 2): a default the file
    // carries, beaten by anything more specific/immediate above.
    if let Some(fmt) = resolve_format(recipe_format)? {
        return Ok(fmt);
    }

    // 4. Preserve source format.
    Ok(source_format)
}

/// The output format that `--format` or the output name fixes, or `None` when
/// neither does (rungs 1 and 2 of [`output_format_for`]).
///
/// Reads no input, so a batch whose format is the same for every image can ask
/// it once, before any image is read.
pub(super) fn format_fixed_by_flags(
    global: &GlobalArgs,
    output_path: Option<&Path>,
    template: Option<&str>,
) -> Result<Option<::image::ImageFormat>, CliError> {
    // 1. Explicit --format wins.
    if let Some(fmt) = resolve_format(global.format.as_deref())? {
        return Ok(Some(fmt));
    }

    // 2. The output name's extension: -o <path>, else the --out-dir template.
    // An unrecognized extension names the file only and falls through.
    if let Some(path) = output_path {
        if let Ok(fmt) = crate::sink::format_from_extension(path) {
            return Ok(Some(fmt));
        }
    } else if let Some(fmt) = template.and_then(crate::sink::template_literal_format) {
        return Ok(Some(fmt));
    }
    Ok(None)
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
///
/// Bakes EXIF orientation first (SPEC-110), so `--max`/etc. bound the
/// visually-correct dimensions rather than the stored (possibly sideways) ones
/// — this is the verb where a stale orientation bound the wrong AXIS, not just
/// mis-tagged the output.
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

    // Build the pipeline: bake orientation first, then this single resize op.
    let pipeline = auto_orient_prefix()?.push(op);

    run_pixel_op(pipeline, inputs, global, global.quality, None, None, false)
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
/// - `never_bigger`: when `true`, apply `optimize`'s pinned-path never-bigger guard
///   (SPEC-113) — see [`write_pixel_output`]. `false` for every caller except
///   `run_optimize`'s pinned (non-`preserve`) branch; a pin has no candidate
///   shortlist, only "the re-encode" versus "the source".
#[allow(clippy::too_many_arguments)]
pub(super) fn run_pixel_op(
    pipeline: Pipeline,
    inputs: &[String],
    global: &GlobalArgs,
    quality: Option<u8>,
    forced_format: Option<::image::ImageFormat>,
    auto: Option<AutoQuality>,
    never_bigger: bool,
) -> Result<(), CliError> {
    // A forced format is the same for every input, so a step that writes alpha
    // it can't store is refused once, before any input is read. The only
    // caller whose pipeline can carry such a step forces its format; a caller
    // that resolves the format per input must check there.
    if let Some(refused) = forced_format.and_then(|fmt| pipeline.unstorable_alpha(fmt)) {
        return Err(refused.into());
    }

    // Resolve every input arg, flattening into one Vec<Input>.
    // Resolution errors (missing path / empty glob) are hard errors (exit 3/2),
    // NOT partial-batch (exit 6). Partial-batch applies only to per-input
    // load/run/write failures AFTER successful resolution.
    let all = source::resolve_all(inputs, &mut std::io::stdin().lock(), OnEmpty::Fail)?;

    // A caller that resolves the format per input checks a step's alpha against
    // it here, but only when a step adds alpha. Every other pipeline keeps its
    // exit codes: resolving the format before the pipeline runs would turn a
    // pipeline failure, such as a crop past the edge, into a format error
    // whenever `--format` is also wrong.
    let alpha_check = forced_format.is_none() && pipeline.introduces_alpha();

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
        let label = input
            .path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| input.stem().to_owned());
        // F1 (SPEC-107, DEC-085): warn on a truncated JPEG; still proceed (exit
        // stays 0). Unconditional, not gated on `--quiet` — see `report.rs`'s
        // `run_info` for why.
        if img.is_truncated_jpeg() {
            eprintln!("warning: {label}: {}", crate::image::TRUNCATED_JPEG_WARNING);
        }
        // SPEC-119: unconditional, not gated on `--quiet` — same reasoning as
        // the truncated-JPEG warning immediately above (DEC-085's sibling).
        if img.is_animated_input() {
            eprintln!("warning: {label}: {}", crate::image::ANIMATED_INPUT_WARNING);
        }

        // The output name and the template that fix the format. The template
        // names the output only when it goes into --out-dir.
        let output_path = global.output.as_ref().map(|s| Path::new(s.as_str()));
        let template = if output_path.is_none() && global.out_dir.is_some() {
            global.name_template.as_deref()
        } else {
            None
        };

        // Before the pipeline runs, so one input is refused with nothing written.
        if alpha_check {
            let fmt = output_format_for(global, output_path, template, None, img.source_format())?;
            if let Some(refused) = pipeline.unstorable_alpha(fmt) {
                return Err(refused.into());
            }
        }

        // Captured BEFORE the pipeline (which may replace the pixels) consumes
        // `img` — only needed when `never_bigger` is set, but cheap to always take
        // (a snapshot of already-decoded fields, no re-decode).
        let source_info = img.info();

        let out_img = pipeline.run(img.clone())?;

        // Resolve per-input output format (DEC-015): forced_format wins if Some.
        let fmt = match forced_format {
            Some(f) => f,
            // No recipe on this path (`resize`/`thumbnail`/`watermark`/`edit`/
            // `auto-orient` build their pipeline directly from CLI flags).
            None => output_format_for(global, output_path, template, None, img.source_format())?,
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
        let plan = resolve_effective_quality(quality, &auto, fmt, &out_img, global, &label)?;
        // A `--max-size` downscale (SPEC-021) replaces the pixels we write.
        let write_img = plan.image.as_ref().unwrap_or(&out_img);

        let sink_input = SinkInput {
            stem: input.stem(),
            path: input.path(),
        };
        write_pixel_output(
            &sink,
            write_img,
            fmt,
            plan.quality,
            input,
            &sink_input,
            overwrite,
            never_bigger,
            &source_info,
            &label,
            global,
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

        // A format the flags fix for every input is checked once, before any
        // image is read: refusing it per input would fail every input one by one.
        if alpha_check {
            if let Some(fmt) = format_fixed_by_flags(global, None, Some(&template))? {
                if let Some(refused) = pipeline.unstorable_alpha(fmt) {
                    return Err(refused.into());
                }
            }
        }

        let total = all.len();
        let mut failed: usize = 0;
        let mut skipped_images: usize = 0;

        for input in &all {
            // Label for error messages.
            let label = match input {
                crate::source::Input::Path(p) => p.display().to_string(),
                crate::source::Input::Stdin { stem, .. } => stem.clone(),
            };

            // Load, run, resolve format, build sink, write — catch per-input errors.
            let result = (|| -> Result<Vec<SkippedStep>, CliError> {
                let img = match input {
                    crate::source::Input::Path(p) => Image::load(p)?,
                    crate::source::Input::Stdin { bytes, .. } => Image::from_bytes(bytes)?,
                };
                // F1 (SPEC-107, DEC-085): see the single-input branch above.
                if img.is_truncated_jpeg() {
                    eprintln!("warning: {label}: {}", crate::image::TRUNCATED_JPEG_WARNING);
                }
                // SPEC-119: see the single-input branch above.
                if img.is_animated_input() {
                    eprintln!("warning: {label}: {}", crate::image::ANIMATED_INPUT_WARNING);
                }

                // A format that follows this input is known now, and before the
                // pipeline runs: only this input fails.
                if alpha_check {
                    let fmt = output_format_for(
                        global,
                        None,
                        Some(&template),
                        None,
                        img.source_format(),
                    )?;
                    if let Some(refused) = pipeline.unstorable_alpha(fmt) {
                        return Err(refused.into());
                    }
                }

                // See the single-input branch above: captured before the pipeline
                // consumes `img`.
                let source_info = img.info();

                // A watermark this image cannot take is left off; the image
                // is still written (the batch keeps going).
                let run = pipeline.run_with(img.clone(), OnRefusal::Skip)?;
                let out_img = run.image;

                // Per-input format resolution (DEC-015): forced_format wins if Some; no -o path in fan-out.
                let fmt = match forced_format {
                    Some(f) => f,
                    // No recipe on this path either — see the single-input branch above.
                    None => {
                        output_format_for(global, None, Some(&template), None, img.source_format())?
                    }
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
                write_pixel_output(
                    &sink,
                    write_img,
                    fmt,
                    plan.quality,
                    input,
                    &sink_input,
                    overwrite,
                    never_bigger,
                    &source_info,
                    &label,
                    global,
                    &mut std::io::stdout().lock(),
                )?;
                Ok(run.skipped)
            })();

            match result {
                Err(e) => {
                    eprintln!("error: {label}: {e}");
                    failed += 1;
                }
                Ok(skipped) => {
                    warn_skipped_steps(&label, &skipped);
                    if !skipped.is_empty() {
                        skipped_images += 1;
                    }
                }
            }
        }

        // A skipped watermark is not a failure: the image was written.
        report_skipped_images(skipped_images, total, global.quiet);
        if failed > 0 {
            return Err(CliError::PartialBatch { failed, total });
        }
    }

    Ok(())
}

/// Write one pixel-op output through `sink`, applying `optimize`'s pinned-path
/// never-bigger guard (SPEC-113) when `never_bigger` is set.
///
/// `never_bigger: false` (every caller except `run_optimize`'s pinned branch) is
/// BYTE-FOR-BYTE the pre-SPEC-113 behavior: `sink.write` encodes `write_img` and
/// writes it, unconditionally.
///
/// `never_bigger: true`: a pin has no candidate shortlist to compare, only "the
/// re-encode" versus "the source" — so this encodes ONCE (via
/// [`crate::sink::encode_to_bytes`], the same encoder `sink.write` would have
/// used) so the size is known before anything is written. The source ships
/// verbatim instead of the re-encode, with a one-line note on stderr (unless
/// `--quiet`), only when ALL of:
/// - the resolved output format matches the source's format (a cross-format pin
///   — `-o out.png` from a `.jpg` — is a deliberate conversion, untouched here);
/// - the re-encode did NOT beat the source (SPEC-084's own bar: strictly
///   smaller — matching bytes counts as "did not beat it", same as
///   `decide::pick_winner`'s `<` eligibility test);
/// - the pipeline changed nothing `optimize` promised to change: see
///   [`super::optimize::pipeline_altered_source`] — when metadata was stripped or
///   orientation baked or the pixels resized, the raw source is not merely a
///   BIGGER output, it is an INVALID one (leaked EXIF/ICC, wrong orientation, or
///   the wrong dimensions), so this guard must not apply and the re-encode ships
///   even though it is larger (matching what the auto-decide path already does
///   in that situation, `optimize.rs`'s `optimize_decide_one`);
/// - the raw container bytes actually sniff (`::image::guess_format`) as `fmt`.
///   `source_info.format` can be an ADOPTED label rather than the real container
///   format: SVG and HEIC input have no matching `ImageFormat` variant and both
///   report `Png`, and a RAW file's embedded-preview extraction reports `Jpeg`
///   while the bytes on disk are the whole RAW container — for any of those,
///   `read_raw_bytes` is NOT a valid `fmt` file, and writing it under a
///   `.png`/`.jpg` name would silently mislabel non-image (or wrong-format)
///   bytes as the pinned format.
///
/// Otherwise the (already-encoded) re-encode bytes are written.
#[allow(clippy::too_many_arguments)]
fn write_pixel_output(
    sink: &Sink,
    write_img: &Image,
    fmt: ::image::ImageFormat,
    quality: Option<u8>,
    input: &crate::source::Input,
    sink_input: &SinkInput<'_>,
    overwrite: Overwrite,
    never_bigger: bool,
    source_info: &crate::image::ImageInfo,
    label: &str,
    global: &GlobalArgs,
    out: &mut dyn std::io::Write,
) -> Result<(), CliError> {
    if !never_bigger {
        sink.write(write_img, sink_input, overwrite, quality, out)?;
        return Ok(());
    }

    // Encode once ourselves (DEC-016's own encoder) so we can measure it before
    // writing anything — `sink.write` would otherwise encode a second time.
    let encoded = crate::sink::encode_to_bytes(write_img, fmt, quality)?;
    let ext = crate::sink::extension_for_format(fmt);

    if fmt == source_info.format && !pipeline_altered_source(source_info, &write_img.info()) {
        let raw = read_raw_bytes(input)?;
        // `source_info.format` can be an ADOPTED label, not the raw container's
        // real format: SVG and HEIC both decode with no matching `ImageFormat`
        // variant and report `Png`; a RAW file's embedded preview reports
        // `Jpeg` while the file on disk is the whole RAW container. For any of
        // those, `raw` is NOT a valid `fmt` file at all — sniff the actual
        // bytes so the guard only ever treats a truly-matching container as
        // "the source" (never text/RAW-container bytes written into a
        // `.png`/`.jpg` and silently mislabeled).
        let raw_is_really_fmt = ::image::guess_format(&raw).ok() == Some(fmt);
        if raw_is_really_fmt && encoded.len() as u64 >= raw.len() as u64 {
            if !global.quiet {
                eprintln!(
                    "{label}: note: kept the {} B source unchanged \u{2014} the {} \
                     re-encode would have shipped {} B, not smaller",
                    raw.len(),
                    super::report::format_label(fmt),
                    encoded.len(),
                );
            }
            sink.write_bytes(&raw, sink_input, ext, overwrite, out)?;
            return Ok(());
        }
    }

    sink.write_bytes(&encoded, sink_input, ext, overwrite, out)?;
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

/// The container lane writes each input's own bytes, so it cannot honour an
/// output name that asks for a different format. Check every input BEFORE any
/// write, so a refused batch leaves nothing behind (not even the inputs that
/// would have matched).
///
/// The name is `-o` (one input) or, with `--out-dir`, the name template — the
/// same name the write below uses. A container that cannot be identified
/// counts as different: it cannot be shown to match.
fn refuse_a_name_asking_for_another_format(
    all: &[crate::source::Input],
    global: &GlobalArgs,
) -> Result<(), CliError> {
    let via = if all.len() == 1 && global.output.is_some() {
        "-o"
    } else {
        "the name template"
    };
    let asked = name_pinned_format(global, all.len()).map(|fmt| (fmt, via));
    let Some((wanted, via)) = asked else {
        return Ok(());
    };

    for input in all {
        let found = container_format(input)?;
        if found != Some(wanted) {
            let label = input
                .path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| input.stem().to_owned());
            return Err(CliError::MetaFormatMismatch {
                input: label,
                via,
                wanted: format_name(wanted),
                found: found.map_or_else(|| "unrecognised".to_owned(), format_name),
                suggest: wanted
                    .extensions_str()
                    .first()
                    .copied()
                    .unwrap_or("png")
                    .to_owned(),
            });
        }
    }
    Ok(())
}

/// An input's container format from its leading bytes (no pixel decode), with
/// the same sniffer the metadata lane itself uses.
fn container_format(
    input: &crate::source::Input,
) -> Result<Option<::image::ImageFormat>, CliError> {
    match input {
        crate::source::Input::Path(p) => {
            use std::io::Read;
            let mut header = Vec::with_capacity(64);
            std::fs::File::open(p)
                .and_then(|f| f.take(64).read_to_end(&mut header))
                .map_err(ImageError::Io)?;
            Ok(::image::guess_format(&header).ok())
        }
        crate::source::Input::Stdin { bytes, .. } => Ok(::image::guess_format(bytes).ok()),
    }
}

/// A format's name for messages: `PNG`, `JPEG`, `WEBP`.
fn format_name(fmt: ::image::ImageFormat) -> String {
    format!("{fmt:?}").to_ascii_uppercase()
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
    warn_on_manifest_drop: bool,
) -> Result<(), CliError> {
    // Resolve + flatten every input arg (resolution errors are hard errors).
    let all = source::resolve_all(inputs, &mut std::io::stdin().lock(), OnEmpty::Fail)?;

    refuse_a_name_asking_for_another_format(&all, global)?;

    let overwrite = if global.yes {
        Overwrite::Allow
    } else {
        Overwrite::Forbid
    };

    if all.len() == 1 {
        let input = &all[0];
        let raw = read_raw_bytes(input)?;
        let out_bytes = transform(&raw)?;
        if warn_on_manifest_drop
            && crate::metadata::has_manifest(&raw)
            && !crate::metadata::has_manifest(&out_bytes)
        {
            let label = input
                .path()
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| input.stem().to_owned());
            eprintln!(
                "warning: {label}: {}",
                crate::metadata::MANIFEST_DROPPED_WARNING
            );
        }
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
                if warn_on_manifest_drop
                    && crate::metadata::has_manifest(&raw)
                    && !crate::metadata::has_manifest(&out_bytes)
                {
                    eprintln!(
                        "warning: {label}: {}",
                        crate::metadata::MANIFEST_DROPPED_WARNING
                    );
                }
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
///
/// No manifest-drop warning: `strip`'s whole contract is removing all
/// metadata (a C2PA manifest included, same as EXIF/ICC/text chunks), so
/// there is nothing surprising to flag — unlike `clean`/`set`/`copy`, which
/// drop one only as a side effect of an edit that would otherwise silently
/// break it.
pub(super) fn run_strip(inputs: &[String], global: &GlobalArgs) -> Result<(), CliError> {
    run_metadata_lane(inputs, global, crate::metadata::strip_all, false)
}

/// Wire `meta clean --gps`: remove ONLY GPS/location metadata via the container
/// lane. `--gps` is required in v1; `meta clean` without it is a usage error (exit
/// 2), leaving room for future selective flags.
pub(super) fn run_clean(inputs: &[String], gps: bool, global: &GlobalArgs) -> Result<(), CliError> {
    if !gps {
        return Err(CliError::Usage("clean requires --gps".into()));
    }
    run_metadata_lane(inputs, global, crate::metadata::clean_gps, true)
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
    run_metadata_lane(
        inputs,
        global,
        |bytes| crate::metadata::set_tags(bytes, &tags),
        true,
    )
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
    refuse_folder(from)?;
    refuse_folder(to)?;
    let from_bytes = std::fs::read(from).map_err(ImageError::Io)?;
    let to_bytes = std::fs::read(to).map_err(ImageError::Io)?;

    let out_bytes = crate::metadata::copy_metadata(&from_bytes, &to_bytes)?;

    // DST's own manifest, not SRC's, is what can be dropped here: the graft
    // rewrites DST's EXIF/ICC (see copy_metadata's doc comment).
    if crate::metadata::has_manifest(&to_bytes) && !crate::metadata::has_manifest(&out_bytes) {
        eprintln!(
            "warning: {to}: {}",
            crate::metadata::MANIFEST_DROPPED_WARNING
        );
    }

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

// ── edit helpers ──────────────────────────────────────────────────────────────

/// The flags `edit` was given, as the op builders read them.
///
/// Which op flags were typed, and in what order, comes from `order` alone; the
/// value fields are read only for an op that `order` lists. `--auto-orient` is
/// not in `order`: orientation is always baked first, so the flag only counts as
/// an op flag.
#[derive(Debug, Clone, Default)]
pub(crate) struct EditFlags {
    pub(crate) auto_orient: bool,
    pub(crate) resize_max: Option<u32>,
    pub(crate) crop: Option<String>,
    pub(crate) crop_gravity: Option<String>,
    pub(crate) rotate: Option<i64>,
    pub(crate) flip: Option<String>,
    pub(crate) pad: Option<String>,
    pub(crate) pad_color: Option<String>,
    pub(crate) pad_gravity: Option<String>,
    pub(crate) trim_color: Option<String>,
    pub(crate) trim_tolerance: Option<u8>,
    pub(crate) order: Vec<EditOp>,
}

/// What `--save-recipe` writes for one step.
enum StepRecord {
    /// The op's own params, known when it was built.
    Built(OperationParams),
    /// A pixel `--crop` or a border `--pad N`: the fractions it worked out when it
    /// ran, read after the run.
    Resolved(Rc<RefCell<Option<OperationParams>>>),
}

/// One step of an `edit` run: the op to run and what to save for it.
struct EditStep {
    op: Box<dyn Operation>,
    record: StepRecord,
}

/// A `--crop` value: pixels (`WxH+X+Y`) or an aspect (`W:H`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CropFlag {
    Pixels { w: u32, h: u32, x: u32, y: u32 },
    Aspect,
}

/// Digits only, no sign, and it must fit a `u32`.
fn crop_number(text: &str) -> Option<u32> {
    if text.is_empty() || !text.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let mut value: u64 = 0;
    for c in text.bytes() {
        value = value * 10 + u64::from(c - b'0');
        if value > u64::from(u32::MAX) {
            return None;
        }
    }
    Some(value as u32)
}

/// Parse a `--crop` value. A `:` makes it an aspect (checked later by the op);
/// otherwise it must be `WxH+X+Y` with a lowercase ASCII `x`, all four numbers
/// present, and a width and height of at least 1.
fn parse_crop_flag(s: &str) -> Result<CropFlag, CliError> {
    if s.contains(':') {
        return Ok(CropFlag::Aspect);
    }
    let bad_syntax = || {
        CliError::Usage(format!(
            "--crop must be WxH+X+Y in pixels, such as 800x600+10+20, or an aspect W:H, such \
             as 16:9; got {s:?}"
        ))
    };
    let rect = (|| {
        let (w, rest) = s.split_once('x')?;
        let (h, offsets) = rest.split_once('+')?;
        let (x, y) = offsets.split_once('+')?;
        Some((
            crop_number(w)?,
            crop_number(h)?,
            crop_number(x)?,
            crop_number(y)?,
        ))
    })();
    let Some((w, h, x, y)) = rect else {
        return Err(bad_syntax());
    };
    if w == 0 || h == 0 {
        return Err(CliError::Usage(format!(
            "--crop must keep at least 1 px each way; got {s:?}"
        )));
    }
    Ok(CropFlag::Pixels { w, h, x, y })
}

/// The `crop` params for a pixel rectangle in an `iw`×`ih` image, as fractions.
///
/// Each is one `f64` division, written to the recipe untouched: a rounded or
/// shortened fraction can replay as a rectangle that runs past the edge.
fn crop_fractions(w: u32, h: u32, x: u32, y: u32, iw: u32, ih: u32) -> OperationParams {
    use std::collections::BTreeMap;

    let (fw, fh) = (f64::from(iw), f64::from(ih));
    let mut map: BTreeMap<String, toml::Value> = BTreeMap::new();
    map.insert("x".into(), toml::Value::Float(f64::from(x) / fw));
    map.insert("y".into(), toml::Value::Float(f64::from(y) / fh));
    map.insert("width".into(), toml::Value::Float(f64::from(w) / fw));
    map.insert("height".into(), toml::Value::Float(f64::from(h) / fh));
    OperationParams::from_map(map)
}

/// A pixel `--crop`. It works out its fractions from the image it receives,
/// which is not the decoded image when an earlier step resized or turned it,
/// and keeps them for `--save-recipe`.
struct PixelCrop {
    w: u32,
    h: u32,
    x: u32,
    y: u32,
    /// The flag value as typed, for messages.
    typed: String,
    resolved: Rc<RefCell<Option<OperationParams>>>,
}

impl Operation for PixelCrop {
    fn name(&self) -> &'static str {
        "crop"
    }

    fn params(&self) -> OperationParams {
        self.resolved
            .borrow()
            .clone()
            .unwrap_or_else(OperationParams::empty)
    }

    fn apply(&self, img: Image) -> Result<Image, OperationError> {
        let fail = |reason: String| OperationError::Apply { op: "crop", reason };
        let (iw, ih) = (img.pixels().width(), img.pixels().height());
        // Checked in pixels, before any conversion, so the message speaks pixels.
        if u64::from(self.x) + u64::from(self.w) > u64::from(iw) {
            return Err(fail(format!(
                "--crop {} runs past the right edge of the {iw}\u{d7}{ih} px image it receives",
                self.typed
            )));
        }
        if u64::from(self.y) + u64::from(self.h) > u64::from(ih) {
            return Err(fail(format!(
                "--crop {} runs past the bottom edge of the {iw}\u{d7}{ih} px image it receives",
                self.typed
            )));
        }
        let params = crop_fractions(self.w, self.h, self.x, self.y, iw, ih);
        let crop = Crop::from_params(&params).map_err(|e| match e {
            RegistryError::InvalidParams { reason, .. } => fail(reason),
            other => fail(other.to_string()),
        })?;
        let out = crop.apply(img)?;
        *self.resolved.borrow_mut() = Some(params);
        Ok(out)
    }
}

/// A `--pad` value: pixels on every side (`N`) or an aspect (`W:H`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PadFlag {
    Border(u32),
    Aspect,
}

/// Parse a `--pad` value. A `:` makes it an aspect (checked later by the op);
/// otherwise it must be whole pixels, at least 1.
fn parse_pad_flag(s: &str) -> Result<PadFlag, CliError> {
    if s.contains(':') {
        return Ok(PadFlag::Aspect);
    }
    match crop_number(s) {
        Some(n) if n >= 1 => Ok(PadFlag::Border(n)),
        _ => Err(CliError::Usage(format!(
            "--pad must be N, a whole number of pixels of at least 1, or an aspect W:H, such \
             as 4:5; got {s:?}"
        ))),
    }
}

/// A border `--pad N`. It works out `border` (N over the long edge) from the
/// image it receives, which is not the decoded image when an earlier step
/// resized or turned it, and keeps it for `--save-recipe`.
struct PixelPad {
    n: u32,
    /// The `--pad-color` value as typed; it is saved as written.
    color: String,
    /// What a translucent fill adds, carrying the flag that set it. Without
    /// this forward, the alpha checks pass and a JPEG gets a black border.
    alpha: Option<IntroducedAlpha>,
    resolved: Rc<RefCell<Option<OperationParams>>>,
}

impl Operation for PixelPad {
    fn name(&self) -> &'static str {
        "pad"
    }

    fn params(&self) -> OperationParams {
        self.resolved
            .borrow()
            .clone()
            .unwrap_or_else(OperationParams::empty)
    }

    fn apply(&self, img: Image) -> Result<Image, OperationError> {
        let fail = |reason: String| OperationError::Apply { op: "pad", reason };
        let (iw, ih) = (img.pixels().width(), img.pixels().height());
        let long = iw.max(ih);
        // Checked in pixels, before any conversion, so the messages speak pixels.
        // A recipe's `border` is at most 1, so a wider N has no saved form.
        if self.n > long {
            return Err(fail(format!(
                "--pad {} is wider than the {long} px long edge of the {iw}\u{d7}{ih} px image \
                 it receives; a border can be at most as wide as the long edge",
                self.n
            )));
        }
        let (cw, ch) = (
            u64::from(iw) + 2 * u64::from(self.n),
            u64::from(ih) + 2 * u64::from(self.n),
        );
        if cw > u64::from(MAX_EDGE) || ch > u64::from(MAX_EDGE) {
            return Err(fail(format!(
                "--pad {} on this {iw}\u{d7}{ih} px image needs a {cw}\u{d7}{ch} px canvas, \
                 which exceeds the maximum edge limit of {MAX_EDGE} px",
                self.n
            )));
        }
        if cw * ch > MAX_AREA {
            return Err(fail(format!(
                "--pad {} on this {iw}\u{d7}{ih} px image needs a {cw}\u{d7}{ch} px canvas = {} \
                 px, which exceeds the maximum area of {MAX_AREA} px",
                self.n,
                cw * ch
            )));
        }
        // One `f64` division, written to the recipe untouched: `pad` rounds
        // `border * long` back to N px.
        let border = f64::from(self.n) / f64::from(long);
        let params = pad_params([
            ("border", toml::Value::Float(border)),
            ("color", toml::Value::String(self.color.clone())),
        ]);
        let pad = Pad::from_params(&params).map_err(|e| match e {
            RegistryError::InvalidParams { reason, .. } => fail(reason),
            other => fail(other.to_string()),
        })?;
        let out = pad.apply(img)?;
        *self.resolved.borrow_mut() = Some(params);
        Ok(out)
    }

    fn introduced_alpha(&self) -> Option<IntroducedAlpha> {
        self.alpha.clone()
    }
}

/// `pad` or `trim` params from key/value pairs.
fn pad_params<const N: usize>(pairs: [(&str, toml::Value); N]) -> OperationParams {
    OperationParams::from_map(
        pairs
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect::<std::collections::BTreeMap<_, _>>(),
    )
}

/// An aspect `--pad` and `--trim`, wrapped so the messages name the flags the
/// user typed. It forwards every `Operation` method; the alpha it forwards
/// carries the flag that set it, and an apply failure has each `'key' = ` of
/// the op's own wording replaced by its flag.
struct FlagWorded {
    inner: Box<dyn Operation>,
    /// `(recipe key, flag)` pairs.
    keys: &'static [(&'static str, &'static str)],
}

impl FlagWorded {
    fn flag_for(&self, key: &str) -> Option<&'static str> {
        self.keys.iter().find(|(k, _)| *k == key).map(|(_, f)| *f)
    }
}

impl Operation for FlagWorded {
    fn name(&self) -> &'static str {
        self.inner.name()
    }

    fn params(&self) -> OperationParams {
        self.inner.params()
    }

    fn apply(&self, img: Image) -> Result<Image, OperationError> {
        self.inner.apply(img).map_err(|e| match e {
            OperationError::Apply { op, mut reason } => {
                for (key, flag) in self.keys {
                    reason = reason.replace(&format!("'{key}' = "), &format!("{flag} "));
                }
                OperationError::Apply { op, reason }
            }
            other => other,
        })
    }

    fn refusal(&self, img: &Image) -> Option<Refusal> {
        self.inner.refusal(img)
    }

    fn introduced_alpha(&self) -> Option<IntroducedAlpha> {
        self.inner.introduced_alpha().map(|a| IntroducedAlpha {
            flag: self.flag_for(a.key),
            ..a
        })
    }
}

/// `pad`'s keys and the flags that set them.
const PAD_FLAGS: [(&str, &str); 3] = [
    ("aspect", "--pad"),
    ("gravity", "--pad-gravity"),
    ("color", "--pad-color"),
];

/// `trim`'s keys and the flags that set them.
const TRIM_FLAGS: [(&str, &str); 2] =
    [("color", "--trim-color"), ("tolerance", "--trim-tolerance")];

/// Build one op through the registry. A refusal names the flag the user typed
/// instead of the recipe key: each `(key, flag)` swaps the first `'key'` in the
/// op's message for `flag`.
fn build_flag_op(
    op: &'static str,
    params: &OperationParams,
    keys: &[(&str, &str)],
) -> Result<Box<dyn Operation>, CliError> {
    OperationRegistry::with_builtins()
        .build(op, params)
        .map_err(|e| match e {
            RegistryError::InvalidParams { mut reason, .. } => {
                for (key, flag) in keys {
                    reason = reason.replacen(&format!("'{key}'"), flag, 1);
                }
                CliError::Usage(reason)
            }
            RegistryError::Unknown { name } => {
                CliError::Usage(format!("unknown operation '{name}'"))
            }
        })
}

/// A step whose saved params are the op's own.
fn built(op: Box<dyn Operation>) -> EditStep {
    let record = StepRecord::Built(op.params());
    EditStep { op, record }
}

/// A single-key param set.
fn one_param(key: &str, value: toml::Value) -> OperationParams {
    let mut map = std::collections::BTreeMap::new();
    map.insert(key.to_owned(), value);
    OperationParams::from_map(map)
}

/// The error for an op that `order` lists but whose value is missing: a bug,
/// never user input.
fn missing_edit_value(flag: &str) -> CliError {
    CliError::Usage(format!("internal: {flag} was listed but has no value"))
}

/// Build the steps for `edit`, one per op flag, in the order the flags were
/// typed. `--auto-orient` adds no step: orientation is baked first, always.
///
/// Every op is built through `OperationRegistry::with_builtins`, so a saved step
/// loads again in `apply`. A bad value is `CliError::Usage` (exit 2), raised
/// before any file is read and naming the flag.
///
/// Errors:
/// - No op flag and no `--auto-orient` → `CliError::Usage`.
/// - A value the op or the `--crop` syntax rejects → `CliError::Usage`.
fn build_edit_ops(flags: &EditFlags) -> Result<Vec<EditStep>, CliError> {
    if flags.order.is_empty() && !flags.auto_orient {
        return Err(CliError::Usage(
            "edit requires at least one operation flag (--auto-orient, --resize-max, --invert, \
             --crop, --rotate, --flip, --pad, --trim)"
                .into(),
        ));
    }

    let mut steps = Vec::new();
    for edit_op in &flags.order {
        match edit_op {
            EditOp::ResizeMax => {
                let n = flags
                    .resize_max
                    .ok_or_else(|| missing_edit_value("--resize-max"))?;
                let params = resize_params(Some(n), None, None, None, None, None)?;
                steps.push(built(build_flag_op("resize", &params, &[])?));
            }
            EditOp::Invert => {
                steps.push(built(build_flag_op(
                    "invert",
                    &OperationParams::empty(),
                    &[],
                )?));
            }
            EditOp::Rotate => {
                let angle = flags.rotate.ok_or_else(|| missing_edit_value("--rotate"))?;
                let params = one_param("angle", toml::Value::Integer(angle));
                steps.push(built(build_flag_op(
                    "rotate",
                    &params,
                    &[("angle", "--rotate")],
                )?));
            }
            EditOp::Flip => {
                let axis = flags
                    .flip
                    .as_deref()
                    .ok_or_else(|| missing_edit_value("--flip"))?;
                let params = one_param("axis", toml::Value::String(axis.to_owned()));
                steps.push(built(build_flag_op(
                    "flip",
                    &params,
                    &[("axis", "--flip")],
                )?));
            }
            EditOp::Crop => {
                let typed = flags
                    .crop
                    .as_deref()
                    .ok_or_else(|| missing_edit_value("--crop"))?;
                match parse_crop_flag(typed)? {
                    CropFlag::Aspect => {
                        let mut params = one_param("aspect", toml::Value::String(typed.to_owned()));
                        if let Some(g) = &flags.crop_gravity {
                            params = OperationParams::from_map(
                                params
                                    .iter()
                                    .map(|(k, v)| (k.to_owned(), v.clone()))
                                    .chain([("gravity".to_owned(), toml::Value::String(g.clone()))])
                                    .collect(),
                            );
                        }
                        steps.push(built(build_flag_op(
                            "crop",
                            &params,
                            &[("aspect", "--crop"), ("gravity", "--crop-gravity")],
                        )?));
                    }
                    CropFlag::Pixels { w, h, x, y } => {
                        if flags.crop_gravity.is_some() {
                            return Err(CliError::Usage(
                                "--crop-gravity only places an aspect --crop W:H; a pixel --crop \
                                 WxH+X+Y is placed by its +X+Y"
                                    .into(),
                            ));
                        }
                        let resolved = Rc::new(RefCell::new(None));
                        steps.push(EditStep {
                            op: Box::new(PixelCrop {
                                w,
                                h,
                                x,
                                y,
                                typed: typed.to_owned(),
                                resolved: Rc::clone(&resolved),
                            }),
                            record: StepRecord::Resolved(resolved),
                        });
                    }
                }
            }
            EditOp::Pad => {
                let typed = flags
                    .pad
                    .as_deref()
                    .ok_or_else(|| missing_edit_value("--pad"))?;
                let color = flags
                    .pad_color
                    .as_deref()
                    .ok_or_else(|| missing_edit_value("--pad-color"))?;
                match parse_pad_flag(typed)? {
                    PadFlag::Aspect => {
                        let mut params = pad_params([
                            ("aspect", toml::Value::String(typed.to_owned())),
                            ("color", toml::Value::String(color.to_owned())),
                        ]);
                        if let Some(g) = &flags.pad_gravity {
                            params = OperationParams::from_map(
                                params
                                    .iter()
                                    .map(|(k, v)| (k.to_owned(), v.clone()))
                                    .chain([("gravity".to_owned(), toml::Value::String(g.clone()))])
                                    .collect(),
                            );
                        }
                        let inner = build_flag_op("pad", &params, &PAD_FLAGS)?;
                        steps.push(built(Box::new(FlagWorded {
                            inner,
                            keys: &PAD_FLAGS,
                        })));
                    }
                    PadFlag::Border(n) => {
                        if flags.pad_gravity.is_some() {
                            return Err(CliError::Usage(
                                "--pad-gravity only places an aspect --pad W:H; a border --pad N \
                                 adds the same width on every side"
                                    .into(),
                            ));
                        }
                        // A stand-in with `border = 1`: it runs the colour through
                        // `pad`'s own parser before any file is read, and says
                        // what alpha the fill adds. The real border is worked out
                        // from the image the step receives.
                        let stand_in = build_flag_op(
                            "pad",
                            &pad_params([
                                ("border", toml::Value::Float(1.0)),
                                ("color", toml::Value::String(color.to_owned())),
                            ]),
                            &PAD_FLAGS,
                        )?;
                        let alpha = stand_in.introduced_alpha().map(|a| IntroducedAlpha {
                            flag: Some("--pad-color"),
                            ..a
                        });
                        let resolved = Rc::new(RefCell::new(None));
                        steps.push(EditStep {
                            op: Box::new(PixelPad {
                                n,
                                color: color.to_owned(),
                                alpha,
                                resolved: Rc::clone(&resolved),
                            }),
                            record: StepRecord::Resolved(resolved),
                        });
                    }
                }
            }
            EditOp::Trim => {
                // Only what was typed is saved, so the recipe replays what ran.
                let mut map = std::collections::BTreeMap::new();
                if let Some(c) = &flags.trim_color {
                    map.insert("color".to_owned(), toml::Value::String(c.clone()));
                }
                if let Some(t) = flags.trim_tolerance {
                    map.insert("tolerance".to_owned(), toml::Value::Integer(i64::from(t)));
                }
                let inner = build_flag_op("trim", &OperationParams::from_map(map), &TRIM_FLAGS)?;
                steps.push(built(Box::new(FlagWorded {
                    inner,
                    keys: &TRIM_FLAGS,
                })));
            }
        }
    }
    Ok(steps)
}

/// Run `edit`: build the steps from the typed flags, run them on `input`, write
/// the result, and, with `--save-recipe`, save the steps that ran.
///
/// Flow:
///
/// 1. `build_edit_ops` builds every step before any file is read, so a bad flag
///    value exits 2 with nothing written.
/// 2. A pixel `--crop` is converted to fractions of the image it receives, which
///    differs per image; with `--save-recipe` that is only well defined for one
///    input image, so a batch is refused up front (exit 2). Listing the input
///    reads no image.
/// 3. The steps run after the orientation bake, which every `edit` does first
///    wherever `--auto-orient` is typed (the flag is an accepted no-op).
/// 4. `run_pixel_op` does the load → run → sink fan-out.
/// 5. Only after a successful run, `--save-recipe` writes the recipe: an
///    `auto-orient` step, then each step that ran, in run order, with a pixel
///    crop's fractions as it resolved them. It is built after the run because the
///    fractions need each step's image. A failed run writes no recipe. A
///    serialization failure is `CliError::Recipe` (exit 1), a write failure
///    `CliError::Sink(SinkError::Io)` (exit 5).
pub(super) fn run_edit(
    input: &str,
    flags: &EditFlags,
    save_recipe: Option<&str>,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    // 1. Build every step (fails on zero flags or a bad value).
    let steps = build_edit_ops(flags)?;

    // 2. A pixel crop or a border pad saved as fractions needs exactly one image
    // to be about. The first such step, in typed order, picks the message.
    let first_pixel_step = steps
        .iter()
        .find(|s| matches!(s.record, StepRecord::Resolved(_)));
    if let (Some(step), Some(_), true) = (first_pixel_step, save_recipe, input != "-") {
        let listed =
            source::resolve_all(&[input.to_owned()], &mut std::io::empty(), OnEmpty::Fail)?;
        let count = listed.len();
        if count > 1 {
            return Err(CliError::Usage(if step.op.name() == "pad" {
                format!(
                    "--save-recipe with a border --pad N needs one input image, and {input:?} \
                     names {count}: the border is saved as a fraction of the image it ran on; \
                     use an aspect --pad W:H, or edit each image on its own"
                )
            } else {
                format!(
                    "--save-recipe with a pixel --crop needs one input image, and {input:?} \
                     names {count}: the crop is saved as fractions of the image it ran on; use \
                     an aspect --crop W:H, or edit each image on its own"
                )
            }));
        }
    }

    // 3. Fold the steps onto the orientation bake, keeping what to save for each.
    let mut plan: Vec<(&'static str, StepRecord)> = Vec::new();
    let mut pipeline = auto_orient_prefix()?;
    for step in steps {
        plan.push((step.op.name(), step.record));
        pipeline = pipeline.push(step.op);
    }

    // 4. Load → run → sink via the single/multi fan-out helper.
    let input_vec = [input.to_owned()];
    run_pixel_op(
        pipeline,
        &input_vec,
        global,
        global.quality,
        None,
        None,
        false,
    )?;

    // 5. The run succeeded: save what ran.
    if let Some(path) = save_recipe {
        let mut recipe = Recipe::from_ops(&[]);
        recipe.steps.push(RecipeStep {
            op: "auto-orient".to_owned(),
            params: OperationParams::empty(),
        });
        for (name, record) in plan {
            let params = match record {
                StepRecord::Built(p) => p,
                StepRecord::Resolved(cell) => cell.borrow().clone().ok_or_else(|| {
                    CliError::Operation(OperationError::Apply {
                        op: name,
                        reason: format!("internal: the {name} step never ran"),
                    })
                })?,
            };
            recipe.steps.push(RecipeStep {
                op: name.to_owned(),
                params,
            });
        }
        // Refuse to write through a symlink.
        crate::sink::reject_symlink_destination(std::path::Path::new(path))?;
        let toml = recipe.to_toml()?;
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
///
/// Bakes EXIF orientation first (SPEC-110), so the bound applies to the
/// visually-correct dimensions.
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

    let pipeline = auto_orient_prefix()?.push(op);
    run_pixel_op(pipeline, inputs, global, global.quality, None, None, false)
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
    run_pixel_op(pipeline, inputs, global, global.quality, None, None, false)
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

/// The result of resolving a [`WatermarkSource`] into an overlay: the
/// decoded/rendered pixels plus everything the corresponding `Watermark`
/// needs to round-trip via `params()` (SPEC-128, Call 3b) — distinct from the
/// pre-SPEC-128 single flattened "label" string, which conflated the two
/// modes under one key (`image`, even for text).
enum ResolvedOverlay {
    Image {
        overlay: ::image::DynamicImage,
        path: String,
    },
    Text {
        overlay: ::image::DynamicImage,
        text: String,
        font_path: Option<String>,
        size: f32,
        color: [u8; 4],
    },
}

/// Check one watermark flag against the key table a recipe step is checked
/// against, so the CLI and a recipe accept the same values. The message names
/// the flag the user typed and prints the value as it parsed. `None` (flag not
/// given) is always fine.
fn check_watermark_flag(flag: &str, key: &str, value: Option<f32>) -> Result<(), CliError> {
    let Some(v) = value else { return Ok(()) };
    let Some(spec) = param_spec(WATERMARK_PARAMS, key) else {
        return Err(CliError::Usage(format!(
            "internal error: no watermark key '{key}'"
        )));
    };
    // `f64::from(f32)` is exact, so the table's own re-read as f32 is a no-op:
    // this is the check on the value the op will actually see.
    if spec.kind.accepts(&toml::Value::Float(f64::from(v))) {
        return Ok(());
    }
    Err(CliError::Usage(format!(
        "{flag} must be {}, got {}",
        spec.kind.expectation(),
        flag_value_text(v)
    )))
}

/// `nan`, `inf`, `-inf`, or the value as parsed (`1025`, `1.5`).
fn flag_value_text(v: f32) -> String {
    if v.is_nan() {
        "nan".to_owned()
    } else if v.is_infinite() {
        if v > 0.0 { "inf" } else { "-inf" }.to_owned()
    } else {
        v.to_string()
    }
}

/// The refusal for a mark that would draw nothing, naming the flags as typed.
fn invisible_mark_error(
    cause: InvisibleMark,
    src: &WatermarkSource<'_>,
    size: f32,
    opacity: f32,
) -> CliError {
    CliError::Usage(match cause {
        InvisibleMark::OpacityZero => {
            "--opacity 0 draws no mark; to leave an image unmarked, do not run watermark".to_owned()
        }
        InvisibleMark::TooFaint => format!(
            "--opacity {} is too faint: the mark would draw no visible pixel; \
             use a larger --opacity",
            flag_value_text(opacity)
        ),
        InvisibleMark::BlankText => {
            "--text has no visible characters; use text with at least one visible character"
                .to_owned()
        }
        InvisibleMark::TextTooSmall => format!(
            "--text draws no visible pixel at --size {}; use a larger --size",
            flag_value_text(size)
        ),
        InvisibleMark::TransparentColor => format!(
            "--color {} is fully transparent; use an alpha above 00, or RRGGBB",
            src.color.unwrap_or_default()
        ),
        InvisibleMark::TransparentImage => format!(
            "--image {} is fully transparent; use an image with visible pixels",
            src.image.unwrap_or_default()
        ),
    })
}

/// Build the watermark overlay (`DynamicImage`) + its round-trip identity from
/// the source mode. Every flag has been checked by now; this is where files
/// are first read.
///
/// - Image mode (`--image PATH`): load the overlay once at the IO boundary
///   (→ exit 3 on failure, DEC-031); the path is kept for `params()` round-trip.
/// - Text mode (`--text STR`): read `--font PATH` at the IO boundary (→ exit 3) or
///   fall back to the bundled font; rasterize via `text::render_text` (pure)
///   into a transparent RGBA overlay (→ exit 2 on a text error).
fn watermark_overlay(
    src: &WatermarkSource<'_>,
    color: [u8; 4],
    size: f32,
) -> Result<ResolvedOverlay, CliError> {
    if let Some(image) = src.image {
        // Image mode: load the overlay once at the IO boundary (DEC-031).
        // Read and decode separately so an unreadable file names itself, while a
        // decode failure keeps its own error and exit code (SPEC-149).
        let bytes = std::fs::read(image).map_err(|source| CliError::WatermarkAssetUnreadable {
            what: "watermark image",
            path: image.to_owned(),
            source,
        })?;
        let overlay = Image::decode_path(image, &bytes)?;
        return Ok(ResolvedOverlay::Image {
            overlay: overlay.pixels().clone(),
            path: image.to_owned(),
        });
    }

    // Text mode. clap guarantees `--text` is present when `--image` is not.
    let text = src.text.unwrap_or("").to_owned();
    let font_path = src.font.map(str::to_owned);

    // Load the font at the IO boundary (--font → exit 3) or use the bundled default.
    let font_owned: Option<Vec<u8>> = match &font_path {
        Some(path) => {
            Some(
                std::fs::read(path).map_err(|source| CliError::WatermarkAssetUnreadable {
                    what: "font file",
                    path: path.clone(),
                    source,
                })?,
            )
        }
        None => None,
    };
    let font_bytes: &[u8] = match font_owned.as_deref() {
        Some(b) => b,
        None => crate::text::DEFAULT_FONT,
    };

    // Rasterize the text into a transparent RGBA overlay (pure; no file IO).
    let rendered = crate::text::render_text(font_bytes, &text, size, color).map_err(text_error)?;
    Ok(ResolvedOverlay::Text {
        overlay: ::image::DynamicImage::ImageRgba8(rendered),
        text,
        font_path,
        size,
        color,
    })
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
    // Check every flag before any file is read, so a bad flag is the answer
    // whatever the inputs, `--font` or `--image` turn out to be (→ Usage, exit 2).
    check_watermark_flag("--size", "size", src.size)?;
    check_watermark_flag("--scale", "scale", scale)?;
    check_watermark_flag("--opacity", "opacity", opacity)?;
    let gravity: Gravity = gravity
        .unwrap_or("southeast")
        .parse()
        .map_err(CliError::Usage)?;
    let color = match src.color {
        Some(s) => crate::text::parse_color(s).map_err(text_error)?,
        None => [255, 255, 255, 255],
    };
    let size = src.size.unwrap_or(32.0);
    let opacity = opacity.unwrap_or(1.0);
    // The two causes the flags alone decide: both are true whatever the font
    // or overlay turns out to be, so they come before either is read.
    if opacity == 0.0 {
        return Err(invisible_mark_error(
            InvisibleMark::OpacityZero,
            src,
            size,
            opacity,
        ));
    }
    if src.image.is_none() && color[3] == 0 {
        return Err(invisible_mark_error(
            InvisibleMark::TransparentColor,
            src,
            size,
            opacity,
        ));
    }

    // Build the overlay (image load OR text rasterization) + its round-trip identity.
    let resolved = watermark_overlay(src, color, size)?;

    let margin = margin.unwrap_or(0);

    // Build the op directly (NOT via the registry — DEC-031) with the decoded /
    // rendered overlay pixels; the mode-specific identity is kept for `params()`
    // (SPEC-128, Call 3b: image mode's path, text mode's text + rendering flags).
    let op = match resolved {
        ResolvedOverlay::Image { overlay, path } => {
            Watermark::new_image(overlay, path, gravity, opacity, scale, margin, tile)
        }
        ResolvedOverlay::Text {
            overlay,
            text,
            font_path,
            size,
            color,
        } => Watermark::new_text(
            overlay, text, font_path, size, color, gravity, opacity, scale, margin, tile,
        ),
    };

    // The rest need the rendered or loaded mark; still before any input is read.
    if let Some(cause) = op.invisible_mark() {
        return Err(invisible_mark_error(cause, src, size, opacity));
    }

    let pipeline = auto_orient_prefix()?.push(Box::new(op));
    run_pixel_op(pipeline, inputs, global, global.quality, None, None, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_checked_watermark_flag_has_a_table_row() {
        use crate::operation::registry::param_spec;
        use crate::operation::{ParamKind, WATERMARK_PARAMS};
        for key in ["size", "scale", "opacity"] {
            let spec = param_spec(WATERMARK_PARAMS, key).expect("a table row");
            assert!(matches!(spec.kind, ParamKind::Number { .. }), "{key}");
        }
    }

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
            None,
            None,
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
            None,
            None,
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
        let result =
            output_format_for(&global, None, None, None, ::image::ImageFormat::Jpeg).unwrap();
        assert_eq!(
            result,
            ::image::ImageFormat::Jpeg,
            "source format should be preserved when no override"
        );
    }

    // ── SPEC-127: recipe.format is a new rung between -o ext and preserve ────

    #[test]
    fn output_format_for_recipe_format_beats_preserve() {
        let global = make_global(None);
        let result = output_format_for(
            &global,
            None,
            None,
            Some("webp"),
            ::image::ImageFormat::Jpeg,
        )
        .unwrap();
        assert_eq!(
            result,
            ::image::ImageFormat::WebP,
            "recipe.format must beat preserve-source when nothing higher-ranked is set"
        );
    }

    #[test]
    fn output_format_for_o_ext_beats_recipe_format() {
        let global = make_global(None);
        let result = output_format_for(
            &global,
            Some(Path::new("/x/a.png")),
            None,
            Some("webp"),
            ::image::ImageFormat::Jpeg,
        )
        .unwrap();
        assert_eq!(
            result,
            ::image::ImageFormat::Png,
            "-o's recognized extension must beat recipe.format"
        );
    }

    #[test]
    fn output_format_for_cli_flag_beats_recipe_format() {
        let global = make_global(Some("png"));
        let result = output_format_for(
            &global,
            None,
            None,
            Some("webp"),
            ::image::ImageFormat::Jpeg,
        )
        .unwrap();
        assert_eq!(
            result,
            ::image::ImageFormat::Png,
            "--format must beat recipe.format"
        );
    }

    #[test]
    fn output_format_for_template_takes_the_output_name_rung() {
        let jpeg = ::image::ImageFormat::Jpeg;
        let png = ::image::ImageFormat::Png;
        let webp = ::image::ImageFormat::WebP;
        let no_flag = make_global(None);
        // The template's recognised extension beats recipe.format and the source.
        assert_eq!(
            output_format_for(&no_flag, None, Some("{stem}.png"), Some("webp"), jpeg).unwrap(),
            png
        );
        // --format beats the template.
        assert_eq!(
            output_format_for(
                &make_global(Some("webp")),
                None,
                Some("{stem}.png"),
                None,
                jpeg
            )
            .unwrap(),
            webp
        );
        // {ext}, or no recognised extension, names the file only.
        for template in ["{stem}.{ext}", "{stem}.txt", "{stem}", "{name}"] {
            assert_eq!(
                output_format_for(&no_flag, None, Some(template), None, jpeg).unwrap(),
                jpeg,
                "{template}"
            );
        }
        // With -o, the template is not the output's name and is ignored, even
        // when -o's own extension is unrecognised.
        assert_eq!(
            output_format_for(
                &no_flag,
                Some(Path::new("o.jpg")),
                Some("{stem}.png"),
                None,
                jpeg
            )
            .unwrap(),
            jpeg
        );
        assert_eq!(
            output_format_for(
                &no_flag,
                Some(Path::new("o.bin")),
                Some("{stem}.png"),
                None,
                jpeg
            )
            .unwrap(),
            jpeg
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

    // ── edit: typed order and the op builders ─────────────────────────────────

    /// The names of the steps `build_edit_ops` builds, in run order.
    fn step_names(flags: &EditFlags) -> Vec<&'static str> {
        build_edit_ops(flags)
            .expect("should succeed")
            .iter()
            .map(|s| s.op.name())
            .collect()
    }

    /// Steps come out in the order `order` lists them, not in a fixed order, and
    /// `--auto-orient` adds none.
    #[test]
    fn edit_steps_follow_typed_order() {
        let flags = EditFlags {
            auto_orient: true,
            resize_max: Some(8),
            order: vec![EditOp::Invert, EditOp::ResizeMax],
            ..EditFlags::default()
        };
        assert_eq!(step_names(&flags), ["invert", "resize"]);

        let flags = EditFlags {
            resize_max: Some(8),
            rotate: Some(90),
            flip: Some("vertical".into()),
            crop: Some("2:1".into()),
            order: vec![
                EditOp::Flip,
                EditOp::ResizeMax,
                EditOp::Crop,
                EditOp::Rotate,
            ],
            ..EditFlags::default()
        };
        assert_eq!(step_names(&flags), ["flip", "resize", "crop", "rotate"]);

        // A pixel crop is a "crop" step too.
        let flags = EditFlags {
            crop: Some("3x3+0+0".into()),
            order: vec![EditOp::Crop],
            ..EditFlags::default()
        };
        assert_eq!(step_names(&flags), ["crop"]);
    }

    /// No op flag and no `--auto-orient` → `CliError::Usage`; `--auto-orient`
    /// alone is accepted.
    #[test]
    fn edit_steps_require_an_op_flag() {
        let err = build_edit_ops(&EditFlags::default())
            .err()
            .expect("zero flags must be refused");
        assert_eq!(err.code(), 2, "usage error must be code 2");
        assert!(
            err.to_string()
                .contains("requires at least one operation flag"),
            "message must mention 'requires at least one operation flag': {err}"
        );
        let only_orient = EditFlags {
            auto_orient: true,
            ..EditFlags::default()
        };
        assert!(build_edit_ops(&only_orient).is_ok());
        assert!(step_names(&only_orient).is_empty());
    }

    /// The resize step built for `--resize-max 16` carries the same params as
    /// `run_resize` builds for `--max 16`: `{mode:"max", width:16}`.
    #[test]
    fn edit_resize_params_match_resize_command() {
        let flags = EditFlags {
            resize_max: Some(16),
            order: vec![EditOp::ResizeMax],
            ..EditFlags::default()
        };
        let steps = build_edit_ops(&flags).expect("should succeed");
        assert_eq!(steps.len(), 1);
        let edit_params = steps[0].op.params();

        let resize_params_val =
            resize_params(Some(16), None, None, None, None, None).expect("resize_params ok");
        let registry_op = OperationRegistry::with_builtins()
            .build("resize", &resize_params_val)
            .expect("build resize ok");
        assert_eq!(
            edit_params,
            registry_op.params(),
            "edit resize params must match run_resize params for --max 16"
        );
    }

    #[test]
    fn parse_crop_flag_cases() {
        assert_eq!(
            parse_crop_flag("800x600+10+20").ok(),
            Some(CropFlag::Pixels {
                w: 800,
                h: 600,
                x: 10,
                y: 20
            })
        );
        assert_eq!(parse_crop_flag("16:9").ok(), Some(CropFlag::Aspect));
        for bad in [
            "10X10+0+0",
            "10x10",
            "10\u{d7}10+0+0",
            "-1x10+0+0",
            "+10x10+0+0",
            "10x10+0+0+0",
            "99999999999x1+0+0",
            "0x10+0+0",
            "10x0+0+0",
            "",
        ] {
            let err = parse_crop_flag(bad).err();
            assert!(err.is_some(), "{bad:?} must be refused");
            assert_eq!(err.map(|e| e.code()), Some(2));
        }
    }

    /// The converter divides by the size it is given, in `f64`, with no rounding.
    #[test]
    fn crop_fractions_divide_by_the_given_size() {
        let p = crop_fractions(631, 2, 9, 0, 640, 2);
        assert_eq!(p.get_f64("x"), Some(9.0 / 640.0));
        assert_eq!(p.get_f64("width"), Some(631.0 / 640.0));
        assert_eq!(p.get_f64("y"), Some(0.0));
        assert_eq!(p.get_f64("height"), Some(2.0 / 2.0));
        let p = crop_fractions(7, 3, 11, 5, 30, 20);
        assert_eq!(p.get_f64("x"), Some(11.0 / 30.0));
        assert_eq!(p.get_f64("y"), Some(5.0 / 20.0));
        assert_eq!(p.get_f64("width"), Some(7.0 / 30.0));
        assert_eq!(p.get_f64("height"), Some(3.0 / 20.0));
    }
}
