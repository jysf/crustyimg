//! The auto-decide format-optimization engine and its entry points: `apply`,
//! `convert`, `optimize`, `web`, and `responsive` (SPEC-014, SPEC-016/017,
//! SPEC-022, SPEC-024, SPEC-048/049, SPEC-084/085/086, SPEC-088). Split out of
//! `cli/mod.rs` (SPEC-097) — no behavior change.

use std::path::{Path, PathBuf};

use crate::image::Image;
use crate::operation::{OperationParams, OperationRegistry, RegistryError};
use crate::pipeline::Pipeline;
use crate::quality::{self, LossyFormat, SearchConfig};
use crate::recipe::Recipe;
use crate::sink::{Overwrite, Sink, SinkError, SinkInput};
use crate::source::{self, SourceError};

use super::common::{
    apply_one, build_sink, fmt_bytes, load_recipe, require_out_dir_for_batch, resolve_format,
    BATCH_PROGRESS_TEMPLATE,
};
use super::report::format_label;
use super::{
    metadata_output_ext, read_raw_bytes, run_pixel_op, AutoQuality, CliError, ExplainFmt,
    GlobalArgs, ProfileArg, QualityTarget,
};

// ── Real apply path ───────────────────────────────────────────────────────────

/// The reserved terminal recipe step that encodes via the fast AVIF-aware decision
/// (`Mode::Fast`: modernize format + never-bigger + score) instead of a plain
/// format-preserving sink write (SPEC-085). This is what makes `apply --recipe web`
/// == the `web` verb — the bundled flows end with it. It is NOT a registry
/// operation (it produces bytes + a format choice, not a transformed `Image`), so it
/// is handled here in the apply path and stripped before `build_pipeline`.
const OPTIMIZE_STEP_OP: &str = "optimize";

/// If `recipe` ends with the terminal [`OPTIMIZE_STEP_OP`] step, return a copy with
/// that step removed — the pixel pipeline to run before the fast decision. `None`
/// when the recipe has no terminal `optimize` step (a plain pixel recipe). An
/// `optimize` step anywhere but last is left in place, so `build_pipeline` surfaces
/// it as a typed `UnknownOperation` error rather than silently reordering intent.
fn split_terminal_optimize(recipe: &Recipe) -> Option<Recipe> {
    match recipe.steps.last() {
        Some(step) if step.op == OPTIMIZE_STEP_OP => {
            let mut pixel = recipe.clone();
            pixel.steps.pop();
            Some(pixel)
        }
        _ => None,
    }
}

/// The `apply --recipe` path: recipe → batch fan-out via rayon + indicatif.
///
/// Single resolved input: preserves the original single-input behavior exactly
/// (writes to `-o`/`--out-dir`/stdout; no progress bar needed).
///
/// Multiple resolved inputs: requires `--out-dir` (else exit 2); replays the
/// recipe in parallel (rayon, `--jobs`); indicatif progress on stderr (hidden
/// when `--quiet`); per-input errors → exit 6 on any failure (DEC-015).
///
/// The `Operation` trait is NOT `Send`, so each rayon task rebuilds its own
/// pipeline from the shared `&recipe` + `&registry` (both `Sync`).
pub(super) fn run_apply(
    recipe_path: &str,
    inputs: &[String],
    json: bool,
    timing: bool,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    use indicatif::{ProgressBar, ProgressStyle};
    use rayon::prelude::*;

    // Steps 0-2: resolve (file path OR bundled name), size-guard, read, and parse.
    let recipe = load_recipe(recipe_path)?;

    // A recipe ending in the terminal `optimize` step (the bundled web/gallery/product
    // flows, SPEC-085) encodes via the fast AVIF-aware decision instead of a plain
    // format-preserving write — so `apply --recipe web` == the `web` verb. Run the
    // preceding pixel steps as the pipeline, then dispatch to the SAME auto-decide
    // fan-out `web` uses (always scoring the downscaled winner). This path is
    // sequential (like `optimize`/`web`), not the rayon batch below.
    if let Some(pixel_recipe) = split_terminal_optimize(&recipe) {
        let registry = OperationRegistry::with_builtins();
        let pipeline = pixel_recipe.build_pipeline(&registry)?;

        // A pinned format (`--format` or a recognized `-o` extension) is an explicit
        // override: honor it and skip the auto-decision (and the score), exactly like
        // the `web`/`optimize` verbs do. Without this diversion the terminal-`optimize`
        // path would auto-decide to AVIF and write those bytes to a `.png` path — so
        // `apply --recipe web hero.jpg -o hero.png` must match `web hero.jpg -o hero.png`
        // (a real PNG of the downscaled image, not AVIF-in-a-`.png`).
        let pinned = resolve_format(global.format.as_deref())?.is_some()
            || global.output.as_deref().is_some_and(|o| {
                o != "-" && crate::sink::format_from_extension(Path::new(o)).is_ok()
            });
        if pinned {
            // No auto-decision to report on the pinned path (SPEC-088).
            reject_audit_without_autodecide(json, timing)?;
            return run_pixel_op(
                pipeline,
                inputs,
                global,
                None,
                None,
                Some(AutoQuality::Fast),
            );
        }

        return run_optimize_autodecide(
            &pipeline,
            inputs,
            &AutoQuality::Fast,
            crate::analysis::decide::Profile::Web,
            // `--json` opts into the machine-readable report, matching the `web` verb.
            if json { Some(ExplainFmt::Json) } else { None },
            timing,
            global,
            true,
        );
    }

    // A plain pixel recipe (no terminal `optimize` step) has no auto-decision to
    // report — `--json`/`--timing` here is a usage error, not a silent no-op (SPEC-088).
    reject_audit_without_autodecide(json, timing)?;

    // Step 3: build registry ONCE; shared via & across rayon tasks (fn ptrs → Sync).
    let registry = OperationRegistry::with_builtins();

    // Step 4: probe the pipeline now so a bad recipe/op fails BEFORE we touch any
    // inputs (exit 1 rather than exit 6 per-input).
    recipe.build_pipeline(&registry)?;

    // Step 5: resolve every positional input via source::resolve, flattening to
    // one Vec<Input>. Resolution errors (missing path / empty glob) are HARD errors
    // (exit 3/2), NOT partial-batch.
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
    drop(stdin_lock);

    let overwrite = if global.yes {
        Overwrite::Allow
    } else {
        Overwrite::Forbid
    };

    // ── Single-input: preserve existing behavior exactly ─────────────────────
    if all.len() == 1 {
        let input = &all[0];
        let img = match input {
            crate::source::Input::Path(p) => Image::load(p)?,
            crate::source::Input::Stdin { bytes, .. } => Image::from_bytes(bytes)?,
        };
        let pipeline = recipe.build_pipeline(&registry)?;
        let out_img = pipeline.run(img)?;
        let sink = build_sink(global)?;
        let sink_input = SinkInput {
            stem: input.stem(),
            path: input.path(),
        };
        sink.write(
            &out_img,
            &sink_input,
            overwrite,
            global.quality,
            &mut std::io::stdout().lock(),
        )?;
        return Ok(());
    }

    // ── Multi-input: require --out-dir, parallel fan-out ─────────────────────
    let out_dir_str = require_out_dir_for_batch(global)?;
    let out_dir = PathBuf::from(out_dir_str);
    let template = global
        .name_template
        .clone()
        .unwrap_or_else(|| "{stem}.{ext}".to_owned());
    let total = all.len();

    // Build indicatif progress bar on stderr (hidden when --quiet or non-TTY).
    let bar = if global.quiet {
        ProgressBar::hidden()
    } else {
        let pb = ProgressBar::new(total as u64);
        let style = ProgressStyle::with_template(BATCH_PROGRESS_TEMPLATE)
            .unwrap_or_else(|_| ProgressStyle::default_bar());
        pb.set_style(style);
        pb
    };

    // Run the batch — with or without a bounded thread pool.
    let results: Vec<Result<(), CliError>> = if let Some(n) = global.jobs {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build()
            .map_err(|e| CliError::Usage(format!("could not build thread pool: {e}")))?;
        pool.install(|| {
            all.par_iter()
                .map(|input| {
                    let r = apply_one(
                        &recipe,
                        &registry,
                        input,
                        &out_dir,
                        &template,
                        overwrite,
                        global.quality,
                    );
                    if let Err(ref e) = r {
                        let label = match input {
                            crate::source::Input::Path(p) => p.display().to_string(),
                            crate::source::Input::Stdin { stem, .. } => stem.clone(),
                        };
                        eprintln!("error: {label}: {e}");
                    }
                    bar.inc(1);
                    r
                })
                .collect()
        })
    } else {
        all.par_iter()
            .map(|input| {
                let r = apply_one(
                    &recipe,
                    &registry,
                    input,
                    &out_dir,
                    &template,
                    overwrite,
                    global.quality,
                );
                if let Err(ref e) = r {
                    let label = match input {
                        crate::source::Input::Path(p) => p.display().to_string(),
                        crate::source::Input::Stdin { stem, .. } => stem.clone(),
                    };
                    eprintln!("error: {label}: {e}");
                }
                bar.inc(1);
                r
            })
            .collect()
    };

    bar.finish_and_clear();

    let failed = results.iter().filter(|r| r.is_err()).count();
    if failed > 0 {
        return Err(CliError::PartialBatch { failed, total });
    }

    Ok(())
}

/// The encode plan for ONE output: the quality to write at, and an optional
/// replacement image when `--max-size` had to downscale the dimensions to fit the
/// byte budget (SPEC-021, DEC-023). `image: None` means write the pipeline's output
/// unchanged; `Some(_)` means write these (smaller) pixels instead.
pub(super) struct EncodePlan {
    pub(super) quality: Option<u8>,
    pub(super) image: Option<Image>,
}

/// Resolve the effective encode plan for ONE output (SPEC-016 / SPEC-017 / SPEC-021).
///
/// - No `auto` mode → return the fixed `quality` unchanged (today's behavior).
/// - `Perceptual` + a perceptually-scorable format → run the SSIMULACRA2 search;
///   warn (unless `--quiet`) if the target was unreachable (best-effort highest
///   quality / largest file).
/// - `SizeBudget` + a byte-budget-drivable format → run the byte-budget search;
///   warn (unless `--quiet`) if even minimum quality exceeds the budget
///   (best-effort smallest file).
/// - `Perceptual` + a format with a knob but NO decoder (AVIF, output-only —
///   DEC-020) → cannot score round-trips; warn and use the encoder default.
/// - any auto mode + a format without a quality knob → ignore it (encoder
///   default); a `SizeBudget` on such a format additionally warns. (Mirrors how
///   `-q` is ignored for lossless formats, DEC-016.)
///
/// The two seams are [`LossyFormat::supports_lossy_quality`] (byte budget;
/// JPEG + AVIF-with-feature) and [`LossyFormat::supports_perceptual_quality`]
/// (perceptual; JPEG only — AVIF perceptual defers with AVIF decode, DEC-020).
///
/// `label` names the input in the warnings.
pub(super) fn resolve_effective_quality(
    quality: Option<u8>,
    auto: &Option<AutoQuality>,
    fmt: ::image::ImageFormat,
    out_img: &Image,
    global: &GlobalArgs,
    label: &str,
) -> Result<EncodePlan, CliError> {
    let supports_perceptual = fmt.supports_perceptual_quality();
    let supports_lossy = fmt.supports_lossy_quality();
    match auto {
        Some(AutoQuality::Perceptual(cfg)) if supports_perceptual => {
            let choice = quality::auto_quality(out_img.pixels(), fmt, cfg)?;
            if !choice.met_target && !global.quiet {
                eprintln!(
                    "warning: {label}: could not reach the requested quality target \
                     (best effort at quality {}); the output may be larger than expected",
                    choice.quality
                );
            }
            Ok(EncodePlan {
                quality: Some(choice.quality),
                image: None,
            })
        }
        // Byte budget — works for ANY output format now (SPEC-021, DEC-023): for a
        // lossy format the quality search runs at full size first and only downscales
        // if even min quality overflows; for a lossless format (PNG, lossless WebP)
        // it is a pure scale search. A chosen downscale is threaded back as a
        // replacement image and the user is warned (unless --quiet).
        Some(AutoQuality::SizeBudget(budget)) => {
            let fit = quality::fit_under_size(out_img.pixels(), fmt, *budget)?;
            let image = fit
                .image
                .map(|pixels| Image::from_parts(pixels, out_img.source_format(), None));
            if !global.quiet {
                if !fit.met_budget {
                    let smallest = if fit.bytes > 0 {
                        fmt_bytes(fit.bytes)
                    } else {
                        "the smallest available size".to_owned()
                    };
                    eprintln!(
                        "warning: {label}: could not meet the {} budget even at the \
                         smallest size (best effort {})",
                        fmt_bytes(*budget),
                        smallest
                    );
                } else if let Some(img) = image.as_ref() {
                    eprintln!(
                        "warning: {label}: scaled to {}x{} to fit the {} budget",
                        img.width(),
                        img.height(),
                        fmt_bytes(*budget)
                    );
                }
            }
            Ok(EncodePlan {
                quality: fit.quality,
                image,
            })
        }
        // Perceptual target on a format with a knob but no decoder (AVIF): the
        // SSIMULACRA2 search must decode each candidate to score it, and AVIF
        // decode is not built (output-only v1, DEC-020). Fall back to the encoder
        // default and warn so the silent downgrade is visible; --max-size still
        // works on AVIF (encode-only).
        Some(AutoQuality::Perceptual(_)) if supports_lossy => {
            if !global.quiet {
                eprintln!(
                    "warning: {label}: --target/--ssim need to decode the re-encoded \
                     image to score it, but no {} decoder is built; wrote it at the \
                     encoder default quality (use --max-size for a byte budget)",
                    format_label(fmt)
                );
            }
            Ok(EncodePlan {
                quality: None,
                image: None,
            })
        }
        // Format without a quality knob: perceptual target ignored (encoder default).
        Some(AutoQuality::Perceptual(_)) => Ok(EncodePlan {
            quality: None,
            image: None,
        }),
        // `Fast` is the `optimize`/`web` default decision (SPEC-084); it runs through
        // `optimize_decide_one`, never this convert quality planner. Handled
        // defensively as the encoder default so the match stays exhaustive.
        Some(AutoQuality::Fast) => Ok(EncodePlan {
            quality: None,
            image: None,
        }),
        None => Ok(EncodePlan {
            quality,
            image: None,
        }),
    }
}

// ── resize/quality helpers ────────────────────────────────────────────────────

/// The default JPEG encode quality when `-q` is omitted for a lossy re-encode that
/// defaults its quality (DEC-016) — `responsive`'s per-variant default.
const DEFAULT_LOSSY_QUALITY: u8 = 80;

/// The `web` verb's default downscale long-edge bound, in pixels.
///
/// STAGE-030's benchmark measured this as the flagship sweet spot: downscaling the
/// long edge to 2048 before the AVIF-aware modernize gave ~98% median savings in
/// ~2.7 s and made the flow size-insensitive (a 24 MP photo finishes as fast as a
/// small one). It is a *max* bound applied via `resize mode=max`, so a source
/// already smaller than 2048 keeps its dimensions (never upscaled). `web --max N`
/// overrides it. This is `web`'s opinion, NOT `optimize`'s (which keeps dimensions
/// by default — SPEC-086); the two bundled variants `gallery`/`product` carry their
/// own bounds.
///
/// Stays `pub` (unchanged from before the split, not a widening) and is
/// re-exported from `cli::mod` (`pub use optimize::WEB_DEFAULT_LONG_EDGE;`) so
/// the external path `crustyimg::cli::WEB_DEFAULT_LONG_EDGE` keeps resolving.
pub const WEB_DEFAULT_LONG_EDGE: u32 = 2048;

/// Map a `--max` long-edge bound to the `Resize` OperationParams the registry
/// expects (SPEC-010's PINNED schema): always `mode=max` (bound the long edge, no
/// upscale). Used by `optimize`/`web`'s pipeline. Infallible — the mapping is
/// total; the op validates the dim.
fn resize_max_params(max: u32) -> OperationParams {
    use std::collections::BTreeMap;

    let mut map: BTreeMap<String, toml::Value> = BTreeMap::new();
    map.insert("mode".into(), toml::Value::String("max".into()));
    map.insert("width".into(), toml::Value::Integer(max as i64));
    OperationParams::from_map(map)
}

// ── auto-quality helpers ──────────────────────────────────────────────────────

/// Reject combining a fixed `-q/--quality` with an auto-quality mode — they are
/// mutually exclusive (one pins a quality, the other searches for it). `-q` is a
/// GLOBAL arg, so this can't be expressed as a clap `conflicts_with` against the
/// subcommand args; both `convert` and `optimize` enforce it here at runtime
/// (`CliError::Usage`, exit 2).
fn reject_quality_with_auto(
    auto: &Option<AutoQuality>,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    if auto.is_some() && global.quality.is_some() {
        return Err(CliError::Usage(
            "-q/--quality cannot be combined with --target/--ssim/--max-size \
             (they auto-tune quality)"
                .into(),
        ));
    }
    Ok(())
}

/// Parse a `--max-size` value into a byte count (SPEC-017).
///
/// Accepts an optional decimal number followed by an optional unit suffix
/// (case-insensitive): none/`B` = bytes, `K`/`KB` = ×1000, `M`/`MB` = ×1_000_000,
/// `KiB` = ×1024, `MiB` = ×1_048_576. The result must be a positive whole number
/// of bytes. Empty / non-numeric / zero / negative / overflow / unknown unit → a
/// typed usage error (`CliError::Usage`, exit 2).
fn parse_size(s: &str) -> Result<u64, CliError> {
    let t = s.trim();
    if t.is_empty() {
        return Err(CliError::Usage("--max-size must not be empty".into()));
    }
    // Split the leading numeric part (digits + a decimal point) from the unit.
    let split = t
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(t.len());
    let (num_str, unit_str) = t.split_at(split);
    let num: f64 = num_str
        .parse()
        .map_err(|_| CliError::Usage(format!("invalid --max-size '{s}': not a number")))?;
    if !(num.is_finite() && num > 0.0) {
        return Err(CliError::Usage(format!(
            "invalid --max-size '{s}': must be a positive size"
        )));
    }
    let mult: f64 = match unit_str.trim().to_ascii_lowercase().as_str() {
        "" | "b" => 1.0,
        "k" | "kb" => 1_000.0,
        "m" | "mb" => 1_000_000.0,
        "kib" => 1_024.0,
        "mib" => 1_048_576.0,
        other => {
            return Err(CliError::Usage(format!(
                "invalid --max-size '{s}': unknown unit '{other}' (use B/KB/MB/KiB/MiB)"
            )))
        }
    };
    let bytes = num * mult;
    if !bytes.is_finite() || bytes < 1.0 || bytes > u64::MAX as f64 {
        return Err(CliError::Usage(format!(
            "invalid --max-size '{s}': out of range"
        )));
    }
    Ok(bytes.round() as u64)
}

// ── convert handler ───────────────────────────────────────────────────────────

/// Wire the `convert` subcommand: resolve the REQUIRED target format ONCE up
/// front (exit 4 for unsupported/unbuilt codec — DEC-004), then pure re-encode
/// every input to that format via an empty `Pipeline` (no-op pixel transform)
/// and the shared `run_pixel_op` fan-out with `forced_format` (DEC-015 / SPEC-014).
///
/// Quality threading: pass `global.quality` as-is; `convert` has NO forced
/// default quality (the encoder default unless `-q`, per DEC-016). `--max-size`
/// auto-tunes the JPEG quality to a byte budget (SPEC-017; JPEG target only —
/// ignored with a warning for a lossless target format); mutually exclusive with
/// `-q`.
///
/// NOTE: the convert-local `--format` arg shadows the global `--format`, so
/// `global.format` is `None` inside `convert`; read the target from `format: &str`.
pub(super) fn run_convert(
    inputs: &[String],
    format: &str,
    max_size: Option<&str>,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    // Resolve the REQUIRED target format ONCE, up front.
    // An unsupported/unbuilt codec (e.g. avif, webp) → SinkError → exit 4 (DEC-004),
    // BEFORE any input is loaded — so a multi-input convert to an unbuilt codec
    // is a single exit 4, never a per-input partial-batch exit 6.
    let fmt = resolve_format(Some(format))?
        .ok_or_else(|| CliError::Usage("convert requires a target --format".into()))?;

    // Fail UP FRONT for a recognized-but-feature-gated codec that is not built
    // (e.g. AVIF without `--features avif`): a single exit 4 (DEC-004) before any
    // input is loaded, so a multi-input convert is never a partial-batch exit 6.
    // (An unrecognized extension already fails at `resolve_format` above with
    // UnsupportedExtension → exit 4.)
    crate::sink::ensure_codec_built(fmt).map_err(CliError::Sink)?;

    // Optional byte budget (--max-size). `-q` pins a quality, --max-size searches
    // for one → reject both.
    let auto: Option<AutoQuality> = match max_size {
        Some(sz) => Some(AutoQuality::SizeBudget(parse_size(sz)?)),
        None => None,
    };
    reject_quality_with_auto(&auto, global)?;
    // With a byte budget, the per-input search supplies the quality (pass None).
    let fixed_quality = if auto.is_some() { None } else { global.quality };

    // Pure re-encode: an empty pipeline returns the pixels unchanged.
    let pipeline = Pipeline::new();

    // Force `fmt` for every input; thread the quality / byte-budget search.
    run_pixel_op(pipeline, inputs, global, fixed_quality, Some(fmt), auto)
}

// ── optimize handler ──────────────────────────────────────────────────────────

/// Resolve `optimize`'s auto-quality mode (SPEC-022, DEC-024; SPEC-084).
///
/// `optimize` is ALWAYS in an auto mode: with no flag
/// the default is the **fast decision** ([`AutoQuality::Fast`], SPEC-084) — a
/// fixed-quality single-encode compare that admits AVIF for photographic content and
/// never emits a larger file. `--target`/`--ssim` opt into the perceptual search;
/// `--max-size` opts into the byte-budget search. The three flags are mutually
/// exclusive — clap enforces it on the subcommand args, so the trailing `_` arm is a
/// defensive runtime fallback (usage error, exit 2).
fn optimize_auto_config(
    target: Option<QualityTarget>,
    ssim: Option<f64>,
    max_size: Option<&str>,
) -> Result<AutoQuality, CliError> {
    match (target, ssim, max_size) {
        // Default: the fast decision (SPEC-084) — fixed-quality single-encode compare,
        // AVIF-aware, never-bigger. The perceptual/byte-budget searches are opt-in.
        (None, None, None) => Ok(AutoQuality::Fast),
        (Some(t), None, None) => Ok(AutoQuality::Perceptual(SearchConfig::for_target(
            t.target_score(),
        ))),
        (None, Some(s), None) => {
            if !(0.0..=100.0).contains(&s) {
                return Err(CliError::Usage(format!(
                    "--ssim must be a score in 0..=100, got {s}"
                )));
            }
            Ok(AutoQuality::Perceptual(SearchConfig::for_target(s)))
        }
        (None, None, Some(sz)) => Ok(AutoQuality::SizeBudget(parse_size(sz)?)),
        _ => Err(CliError::Usage(
            "--target/--ssim/--max-size are mutually exclusive".into(),
        )),
    }
}

/// Wire the `optimize` subcommand: the keep-dimensions byte-primitive (DEC-024,
/// SPEC-084/086).
///
/// Pipeline (PINNED order): `auto-orient` (bake EXIF orientation, then drop the
/// metadata bundle — DEC-017) then, iff `--max N`, a `resize max N` long-edge
/// bound (built via [`resize_max_params`]). By default (no flags) the output is the
/// **fast fixed-quality decision** ([`AutoQuality::Fast`], SPEC-084): a single-encode
/// AVIF-aware compare that keeps dimensions and never ships a larger file. The
/// perceptual (`--target`/`--ssim`) and byte-budget (`--max-size`) searches are
/// opt-in. Format follows DEC-015 precedence (`--format` > `-o` ext > the
/// auto-decision, unless `--profile preserve`). The pixel-lane re-encode drops ALL
/// metadata (privacy incl. GPS); this is NOT the selective-preserve container lane
/// (DEC-003), which is unbuilt (STAGE-004).
///
/// The default stays **score-free** (scoring a full-resolution winner costs too much
/// to run unconditionally — SPEC-084 acceptance #4); `--verify` opts into a single
/// [`crate::quality::score_winner_once`] readout for this run (`web` scores always —
/// SPEC-085). `optimize` always auto-tunes quality, so a fixed `-q` conflicts
/// (exit 2); `--target`/`--ssim`/`--max-size` are mutually exclusive. Multi-input
/// fan-out + partial-batch exit 6 are inherited via [`run_pixel_op`] (DEC-015).
// A CLI command handler mirroring its clap-destructured args (outcome flags +
// profile + verify + explain); bundling them would just re-wrap the same scalars.
#[allow(clippy::too_many_arguments)]
pub(super) fn run_optimize(
    inputs: &[String],
    max: Option<u32>,
    target: Option<QualityTarget>,
    ssim: Option<f64>,
    max_size: Option<&str>,
    profile: ProfileArg,
    verify: bool,
    explain: Option<ExplainFmt>,
    json: bool,
    timing: bool,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    let auto = optimize_auto_config(target, ssim, max_size)?;
    // optimize always auto-tunes quality; a fixed -q conflicts.
    reject_quality_with_auto(&Some(auto.clone()), global)?;

    // `--json` is the consistent audit spelling; it maps to the JSON explain
    // channel (equivalent to `--explain=json`, which clap forbids combining with).
    let explain = if json {
        Some(ExplainFmt::Json)
    } else {
        explain
    };

    // Always auto-orient first so any `--max` bound applies to the visually-correct
    // dimensions; the op also drops the metadata bundle after baking (DEC-017).
    let pipeline = optimize_pipeline(max)?;

    // The engine only fires when the user has NOT pinned a format (`--format` or a
    // recognized `-o` extension) and the profile is not `preserve` (DEC-048). A pin
    // or `preserve` reproduces today's format-preserving behaviour exactly.
    let pinned = resolve_format(global.format.as_deref())?.is_some()
        || global
            .output
            .as_deref()
            .is_some_and(|o| o != "-" && crate::sink::format_from_extension(Path::new(o)).is_ok());

    if profile == ProfileArg::Preserve || pinned {
        // Preserve / pinned: auto quality, per-input format preserved / honored
        // from -o/--format (DEC-015). This is the strict regression anchor.
        // (--explain has no decision to describe here; it is silently ignored.)
        // The audit report needs the auto-decision, so `--json`/`--timing` here is a
        // usage error rather than a silent no-op (SPEC-088).
        reject_audit_without_autodecide(json, timing)?;
        return run_pixel_op(pipeline, inputs, global, None, None, Some(auto));
    }

    run_optimize_autodecide(
        &pipeline,
        inputs,
        &auto,
        profile.to_decide(),
        explain,
        timing,
        global,
        // `optimize` keeps dimensions, so scoring the full-resolution winner is too
        // costly to run by default (SPEC-084 acceptance #4); `--verify` opts in for
        // this run (SPEC-086), and `web` scores always (SPEC-085).
        verify,
    )
}

/// The audit report (`--json`/`--timing`) is produced by the auto-decision path
/// (`optimize`/`web`/`apply --recipe web`). On a format-pinned (`-o`/`--format`),
/// `--profile preserve`, or plain-pixel-recipe run there is no decision to report,
/// so passing `--json`/`--timing` is a usage error (exit 2) — an honest rejection
/// rather than a silently ignored flag (SPEC-088).
fn reject_audit_without_autodecide(json: bool, timing: bool) -> Result<(), CliError> {
    if json || timing {
        return Err(CliError::Usage(
            "--json/--timing report the auto-decision and are unavailable with a pinned \
             -o/--format, --profile preserve, or a plain pixel recipe"
                .to_owned(),
        ));
    }
    Ok(())
}

/// Does the image sink resolve to stdout? Two spellings reach it: an explicit
/// `-o -`, and the bare default (no `-o`, no `--out-dir`) — mirroring the sink
/// construction in [`run_optimize_autodecide`]. Keying the JSON guard on this
/// state rather than on the `-o -` spelling closes both doors with one rule.
fn image_sink_is_stdout(global: &GlobalArgs) -> bool {
    global.out_dir.is_none() && global.output.as_deref().is_none_or(|o| o == "-")
}

/// The JSON audit report goes to stdout, and so do the image bytes whenever the
/// sink resolves there — interleaving the two corrupts both (the report is
/// unparseable, the image undecodable). Reject the combination rather than emit a
/// poisoned stream, so stdout stays pipe-clean (SPEC-088, DEC-074).
///
/// This covers `optimize --json`, `web --json`, `apply --recipe web --json` **and**
/// the pre-existing `optimize --explain=json`, which reaches the same writer — one
/// rule for one surface, on both the explicit `-o -` and the default-stdout path.
/// `--timing` alone is unaffected: it renders to stderr. The human `--explain` is
/// likewise fine (stderr).
fn reject_json_report_on_stdout_sink(
    explain: Option<ExplainFmt>,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    if matches!(explain, Some(ExplainFmt::Json)) && image_sink_is_stdout(global) {
        return Err(CliError::Usage(
            "--json/--explain=json writes the report to stdout, which the image is \
             already using (an explicit `-o -`, or the default with no -o/--out-dir); \
             send the image elsewhere (-o FILE or --out-dir DIR) to keep stdout \
             pipe-clean"
                .to_owned(),
        ));
    }
    Ok(())
}

/// Wire the `web` flagship verb (SPEC-085): the measured downscale-then-modernize
/// flow. `web <inputs>` == `apply --recipe web <inputs>` — both reach the identical
/// engine (this verb builds the flow in memory; the bundled `web` recipe reaches it
/// through the terminal-`optimize` apply path).
///
/// The flow: bake EXIF orientation + strip metadata (`auto-orient`, DEC-017) →
/// downscale the long edge to [`WEB_DEFAULT_LONG_EDGE`] (or `--max`, never upscaling)
/// → the fast AVIF-aware decision (`Mode::Fast`, never-bigger — SPEC-084) →
/// unconditionally score the (downscaled) winner and report it. Reuses
/// [`optimize_pipeline`] + [`run_optimize_autodecide`]; it does NOT re-implement the
/// engine.
///
/// `-o`/`--format` pin the output format, which (as with `optimize`) bypasses the
/// auto-decision and reproduces a plain format-honored re-encode of the downscaled
/// image. The global `--out-dir`/`--name-template`/`-j` drive batch output.
pub(super) fn run_web(
    inputs: &[String],
    max: Option<u32>,
    json: bool,
    timing: bool,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    // `web` always auto-tunes (the fast decision), so a fixed `-q` conflicts (exit 2),
    // exactly like `optimize`.
    let auto = AutoQuality::Fast;
    reject_quality_with_auto(&Some(auto.clone()), global)?;

    // The default downscale is `web`'s whole point; `--max` overrides the bound. It is
    // a `resize mode=max` long-edge cap, so a smaller source is never upscaled.
    let long_edge = max.unwrap_or(WEB_DEFAULT_LONG_EDGE);
    let pipeline = optimize_pipeline(Some(long_edge))?;

    // A pinned format (`--format` or a recognized `-o` extension) is an explicit
    // override: honor it and skip the auto-decision (and the score), mirroring
    // `optimize`'s pin behaviour.
    let pinned = resolve_format(global.format.as_deref())?.is_some()
        || global
            .output
            .as_deref()
            .is_some_and(|o| o != "-" && crate::sink::format_from_extension(Path::new(o)).is_ok());
    if pinned {
        // No auto-decision to report on the pinned path (SPEC-088).
        reject_audit_without_autodecide(json, timing)?;
        return run_pixel_op(pipeline, inputs, global, None, None, Some(auto));
    }

    run_optimize_autodecide(
        &pipeline,
        inputs,
        &auto,
        crate::analysis::decide::Profile::Web,
        // `web` has no `--explain`; `--json` opts into the machine-readable report,
        // else the default one-line summary (with the score) carries it.
        if json { Some(ExplainFmt::Json) } else { None },
        timing,
        global,
        // The downscaled winner is cheap to score — always do it (SPEC-085).
        true,
    )
}

/// Build the shared `optimize` pipeline: auto-orient (bake EXIF orientation, drop
/// the metadata bundle, DEC-017), then an optional `--max` long-edge bound.
fn optimize_pipeline(max: Option<u32>) -> Result<Pipeline, CliError> {
    let registry = OperationRegistry::with_builtins();
    let map_registry_err = |e| match e {
        RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason),
        RegistryError::Unknown { name } => CliError::Usage(format!("unknown operation '{name}'")),
    };
    let orient = registry
        .build("auto-orient", &OperationParams::empty())
        .map_err(map_registry_err)?;
    let mut pipeline = Pipeline::new().push(orient);
    if let Some(n) = max {
        let resize = registry
            .build("resize", &resize_max_params(n))
            .map_err(map_registry_err)?;
        pipeline = pipeline.push(resize);
    }
    Ok(pipeline)
}

/// One solved candidate: its measured outcome plus the encoded bytes to ship if
/// it wins (encoded exactly once, via the sink's own encoder — DEC-016).
struct SolvedCandidate {
    outcome: crate::analysis::decide::CandidateOutcome,
    encoded: Vec<u8>,
    ext: String,
    /// The encoder quality used (`None` for a lossless candidate) — for `explain`.
    quality: Option<u8>,
}

/// The fixed encoder quality a lossy candidate is encoded at in the default (fast)
/// decision (SPEC-084). One generous value ([`crate::sink::FAST_LOSSY_QUALITY`])
/// across AVIF / JPEG / lossy WebP — AVIF is the eyeball-validated anchor (DEC-069);
/// the others are conventional high-quality on their own scales, reached only as
/// fallbacks when AVIF is not built. Distinct from `convert`'s
/// [`crate::sink::AVIF_DEFAULT_QUALITY`], which the fast path never uses.
fn fast_lossy_quality(_fmt: ::image::ImageFormat) -> u8 {
    crate::sink::FAST_LOSSY_QUALITY
}

/// The compact LOSSY re-encode format for the metadata-forced fallback (SPEC-084
/// never-bigger), used only when a **lossy-family source** landed in a bucket whose
/// shortlist offers only lossless candidates (a graphic-classified photo, or any
/// lossy source with an ICC profile). A lossless re-encode of a lossy source blows up
/// several-fold; a lossy re-encode in the source's own family (or JPEG) stays
/// ≈ source size, so we ship that instead of the blow-up.
///
/// Prefers the source's own lossy codec when it is built (AVIF → AVIF, WebP → lossy
/// WebP), else a baseline JPEG — but JPEG cannot carry alpha, so an alpha source with
/// no built alpha-capable lossy codec has no compact lossy option and returns `None`
/// (the lossless candidates stay, honestly reported). Returns `None` for a lossless
/// source (PNG/lossless-WebP): its lossless candidates are the right fallback, and a
/// lossy re-encode would be an unwanted quality loss.
fn fast_fallback_lossy_entry(
    source_format: ::image::ImageFormat,
    has_alpha: bool,
    built: crate::analysis::decide::BuiltCodecs,
) -> Option<crate::analysis::decide::ShortlistEntry> {
    use crate::analysis::decide::{Disposition, ShortlistEntry};
    use ::image::ImageFormat::{Avif, Jpeg, WebP};

    let fmt = match source_format {
        Avif if built.avif => Avif,
        WebP if built.webp_lossy => WebP,
        // A lossy source (JPEG, or WebP/AVIF whose lossy encoder is not built): a
        // baseline JPEG is the universal compact lossy fallback — but only without
        // alpha (JPEG has none).
        Jpeg | WebP | Avif if !has_alpha => Jpeg,
        _ => return None,
    };
    Some(ShortlistEntry {
        fmt,
        disposition: Disposition::Lossy,
    })
}

/// Solve one shortlisted candidate against the oriented output image: run the
/// existing quality search for its disposition/mode and encode it once.
fn solve_candidate(
    out_img: &Image,
    entry: crate::analysis::decide::ShortlistEntry,
    auto: &AutoQuality,
) -> Result<SolvedCandidate, CliError> {
    use crate::analysis::decide::{CandidateOutcome, Disposition};
    let fmt = entry.fmt;
    let ext = crate::sink::extension_for_format(fmt).to_owned();

    let (encoded, met_target, quality) = match (entry.disposition, auto) {
        // Fast (the default, SPEC-084): a single fixed-quality encode — no search.
        // Lossy candidates use the generous fast quality; lossless has no knob. A
        // fixed encode has no target to miss, so it always "meets" — `pick_winner`
        // still requires it to beat the source (never-bigger).
        (disposition, AutoQuality::Fast) => {
            let quality = match disposition {
                Disposition::Lossy => Some(fast_lossy_quality(fmt)),
                Disposition::Lossless => None,
            };
            let bytes = crate::sink::encode_to_bytes(out_img, fmt, quality)?;
            (bytes, true, quality)
        }
        // Lossy + perceptual: search the lowest quality meeting the target, then
        // encode once at it. (Shortlist guarantees fmt is perceptually scorable.)
        (Disposition::Lossy, AutoQuality::Perceptual(cfg)) => {
            let qc = quality::auto_quality(out_img.pixels(), fmt, cfg)?;
            let bytes = crate::sink::encode_to_bytes(out_img, fmt, Some(qc.quality))?;
            (bytes, qc.met_target, Some(qc.quality))
        }
        // Lossless + perceptual: a single encode; lossless always meets a
        // perceptual target.
        (Disposition::Lossless, AutoQuality::Perceptual(_)) => {
            let bytes = crate::sink::encode_to_bytes(out_img, fmt, None)?;
            (bytes, true, None)
        }
        // Byte-budget (either disposition): fit_under_size handles quality-then-scale
        // for lossy and the scale search for lossless, and reports whether it fit.
        (_, AutoQuality::SizeBudget(budget)) => {
            let fit = quality::fit_under_size(out_img.pixels(), fmt, *budget)?;
            let bytes = match fit.image {
                Some(px) => {
                    let img = Image::from_parts(px, out_img.source_format(), None);
                    crate::sink::encode_to_bytes(&img, fmt, fit.quality)?
                }
                None => crate::sink::encode_to_bytes(out_img, fmt, fit.quality)?,
            };
            (bytes, fit.met_budget, fit.quality)
        }
    };

    let outcome = CandidateOutcome {
        fmt,
        disposition: entry.disposition,
        bytes: encoded.len() as u64,
        met_target,
    };
    Ok(SolvedCandidate {
        outcome,
        encoded,
        ext,
        quality,
    })
}

/// What `optimize`'s auto-decision produced for one input.
enum OptimizeOutput {
    /// A winning re-encode: ship these bytes (extension names the chosen format).
    Encoded { bytes: Vec<u8>, ext: String },
    /// No candidate beat the source — pass the original file through unchanged.
    Passthrough { raw: Vec<u8>, ext: String },
}

/// Decode → orient → auto-decide the output format for ONE input (SPEC-048/049).
/// Runs the decision engine and returns the bytes to ship (or a passthrough)
/// plus the `ExplainTrace` for reporting (`None` only for a degenerate image).
/// Does NOT print — the caller renders the summary or `--explain`.
fn optimize_decide_one(
    input: &crate::source::Input,
    pipeline: &Pipeline,
    auto: &AutoQuality,
    profile: crate::analysis::decide::Profile,
    // Always score the winner (the `web` verb / a terminal-`optimize` recipe, on
    // their downscaled output — cheap, SPEC-085). The keep-dimensions default
    // (`optimize`) passes `false`: scoring a full-resolution image costs too much
    // to run unconditionally (SPEC-084 acceptance #4).
    always_score: bool,
    // Measure decode/encode/total for the `--timing` audit readout (SPEC-088). When
    // false, no `Instant`s are taken and the trace's `timing` stays `None`, so a
    // non-`--timing` run's report is byte-identical.
    timing: bool,
) -> Result<
    (
        OptimizeOutput,
        Option<crate::analysis::decide::ExplainTrace>,
        // The winner's achieved SSIMULACRA2 score, for the default (fast) path's
        // report — `Some` only for a lossy fast winner that could be decoded and
        // scored; `None` for lossless, passthrough, or the opt-in search modes.
        Option<f64>,
    ),
    CliError,
> {
    use crate::analysis::decide::{
        self, BuiltCodecs, CandidateTrace, Disposition, ExplainTrace, Mode,
    };
    use std::time::Instant;

    // Whole-run clock (SPEC-088). Only started under `--timing`; the sub-spans
    // (decode, encode) accumulate into these locals when it is set.
    let run_start = timing.then(Instant::now);
    let mut decode_ms = 0.0_f64;
    let mut encode_ms = 0.0_f64;

    // Raw source bytes: the "beats source" reference AND the passthrough payload.
    let raw = read_raw_bytes(input)?;
    let source_bytes = raw.len() as u64;

    let decode_start = timing.then(Instant::now);
    let img = match input {
        crate::source::Input::Path(p) => Image::load(p)?,
        crate::source::Input::Stdin { bytes, .. } => Image::from_bytes(bytes)?,
    };
    if let Some(t) = decode_start {
        decode_ms = t.elapsed().as_secs_f64() * 1000.0;
    }
    let source_format = img.source_format();
    // Capture the source's shape + metadata BEFORE the pipeline consumes it: a raw
    // passthrough is only faithful when the pipeline changed nothing `optimize`
    // promised to change (see `pipeline_altered` below).
    let source_info = img.info();
    let source_dims = (source_info.width, source_info.height);
    let source_had_metadata = source_info.has_exif || source_info.has_icc;
    let out_img = pipeline.run(img)?;

    // Did the pipeline alter the image in a way that makes the RAW source an invalid
    // output? `optimize` bakes EXIF orientation and strips ALL metadata (privacy,
    // incl. GPS — DEC-017). So the raw bytes are a faithful passthrough only when the
    // source carried no metadata AND the pixels were untouched (dims unchanged; an
    // orientation flip that keeps dims still carries an EXIF tag, so it trips the
    // metadata check). Otherwise a "passthrough" must ship the PROCESSED, stripped
    // image instead of leaking metadata / a wrong orientation.
    let out_info = out_img.info();
    let pipeline_altered = source_had_metadata || (out_info.width, out_info.height) != source_dims;

    let built = BuiltCodecs {
        webp_lossy: cfg!(feature = "webp-lossy"),
        avif: cfg!(feature = "avif"),
    };
    let mode = match auto {
        AutoQuality::Fast => Mode::Fast,
        AutoQuality::Perceptual(_) => Mode::Perceptual,
        AutoQuality::SizeBudget(_) => Mode::SizeBudget,
    };

    // Compute the analysis verdict; on a degenerate image (no verdict) pass the
    // source through unchanged rather than guessing (no trace to explain).
    let analysis = match crate::analysis::Analysis::compute(&out_img) {
        Ok(a) => a,
        Err(_) => {
            let output = OptimizeOutput::Passthrough {
                raw,
                ext: metadata_output_ext(input, &[]),
            };
            return Ok((output, None, None));
        }
    };
    let has_alpha = out_info.has_alpha;

    let shortlist =
        decide::format_shortlist(analysis.opt_bucket(), has_alpha, profile, mode, built);

    let mut solved: Vec<SolvedCandidate> = Vec::with_capacity(shortlist.len());
    let encode_start = timing.then(Instant::now);
    for entry in shortlist {
        solved.push(solve_candidate(&out_img, entry, auto)?);
    }
    if let Some(t) = encode_start {
        encode_ms += t.elapsed().as_secs_f64() * 1000.0;
    }
    let outcomes: Vec<_> = solved.iter().map(|s| s.outcome).collect();
    let winner = decide::pick_winner(&outcomes, source_bytes, source_format);

    // When nothing beats the source but the pipeline altered the image, the raw
    // source is NOT a valid output (wrong orientation / un-stripped metadata) — we
    // must ship a processed, stripped result. Ship the SMALLEST CORRECT one.
    //
    // A graphic bucket offers only lossless candidates, and a lossless re-encode of a
    // *lossy* source (a photo that classified as a graphic, or any lossy source with
    // an ICC profile) blows up several-fold. So for a lossy-family source with no
    // lossy candidate in the shortlist, add one compact lossy re-encode (its own
    // family, or JPEG) and let it compete — never ship a lossless blow-up
    // (SPEC-084 never-bigger). If even the smallest correct output still exceeds the
    // source (a genuine case: stripping metadata forces a re-encode that can't beat an
    // already-tight source), we ship it anyway — but the report tells the truth
    // ("N% larger"), it is never clamped to a break-even "0% smaller".
    let winner = match winner {
        Some(i) => Some(i),
        None if pipeline_altered => {
            let has_lossy = solved
                .iter()
                .any(|s| s.outcome.disposition == Disposition::Lossy);
            if !has_lossy {
                if let Some(entry) = fast_fallback_lossy_entry(source_format, has_alpha, built) {
                    let fallback_start = timing.then(Instant::now);
                    solved.push(solve_candidate(&out_img, entry, auto)?);
                    if let Some(t) = fallback_start {
                        encode_ms += t.elapsed().as_secs_f64() * 1000.0;
                    }
                }
            }
            (0..solved.len()).min_by_key(|&i| (solved[i].outcome.bytes, i))
        }
        None => None,
    };

    let output = match winner {
        Some(i) => {
            let win = &solved[i];
            OptimizeOutput::Encoded {
                bytes: win.encoded.clone(),
                ext: win.ext.clone(),
            }
        }
        None => OptimizeOutput::Passthrough {
            raw,
            ext: metadata_output_ext(input, &[]),
        },
    };
    let out_bytes = match winner {
        Some(i) => solved[i].outcome.bytes,
        None => source_bytes,
    };

    // The winner is scored only when `always_score` is set (SPEC-085's `web` verb
    // and terminal-`optimize` recipes, on their DOWNSCALED output — cheap, ~0.2–0.35 s
    // at 2–3 MP). The keep-dimensions default (`optimize`) passes `false`: scoring a
    // full-resolution image costs ~107 ms/MP (measured; ~5 s at 47 MP), too much to run
    // unconditionally on the verb everyone runs (SPEC-084 acceptance #4). `optimize
    // --verify` (SPEC-086) will opt in on request.
    //
    // Scoring decodes the shipped winner bytes with the default decoder (AVIF via
    // re_rav1d, no feature needed — SPEC-058) and compares against the oriented,
    // downscaled reference `out_img`. It is best-effort: a decode/score failure
    // degrades to "no score", never failing the command. A passthrough (no winner)
    // and a degenerate image have nothing to score.
    let winner_score: Option<f64> = if always_score {
        winner.and_then(|i| {
            Image::from_bytes(&solved[i].encoded)
                .ok()
                .and_then(|decoded| {
                    crate::quality::score_winner_once(out_img.pixels(), Some(decoded.pixels())).ok()
                })
                .flatten()
        })
    } else {
        None
    };

    let uc = analysis.unique_colors();
    let trace = ExplainTrace {
        source_format,
        class: analysis.class(),
        entropy: analysis.entropy(),
        edge_ratio: analysis.edge_ratio(),
        flat_ratio: analysis.flat_ratio(),
        unique_colors: uc.count(),
        unique_saturated: uc.is_saturated(),
        has_alpha,
        profile,
        mode,
        source_bytes,
        candidates: solved
            .iter()
            .map(|s| CandidateTrace {
                fmt: s.outcome.fmt,
                disposition: s.outcome.disposition,
                quality: s.quality,
                bytes: s.outcome.bytes,
                met_target: s.outcome.met_target,
            })
            .collect(),
        winner,
        out_bytes,
        verify_score: winner_score,
        // `total` is the whole decide→encode→(score) span, so it is >= encode and
        // >= decode by construction (SPEC-088). `None` unless `--timing` was set.
        timing: run_start.map(|start| crate::analysis::decide::Timing {
            decode_ms,
            encode_ms,
            total_ms: start.elapsed().as_secs_f64() * 1000.0,
        }),
    };

    Ok((output, Some(trace), winner_score))
}

/// Render one input's report: `--explain` (json→stdout, human→stderr), else the
/// default one-line summary to stderr (unless `--quiet`).
///
/// `score` is the fast path's achieved SSIMULACRA2 (SPEC-084) — appended to the
/// default summary and the human trace as proof of the quality kept. `None` (a
/// lossless winner, a passthrough, or the opt-in search modes) prints "lossless"
/// only when a re-encode actually happened, otherwise nothing.
fn emit_optimize_report(
    label: &str,
    trace: Option<&crate::analysis::decide::ExplainTrace>,
    score: Option<f64>,
    explain: Option<ExplainFmt>,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    use std::io::Write as _;
    // The score suffix shown on the human summary lines: " · ssim 88.4" for a scored
    // lossy winner, empty otherwise (the JSON channel is schema-pinned and untouched).
    let score_suffix = match score {
        Some(s) => format!(" \u{b7} ssim {s:.1}"),
        None => String::new(),
    };
    match explain {
        Some(ExplainFmt::Json) => {
            if let Some(t) = trace {
                let mut out = std::io::stdout().lock();
                let render = (|| -> std::io::Result<()> {
                    t.write_json(&mut out)?;
                    writeln!(out)
                })();
                render
                    .map_err(|e| CliError::Usage(format!("failed to write explain output: {e}")))?;
            }
        }
        Some(ExplainFmt::Human) => {
            let mut err = std::io::stderr().lock();
            match trace {
                Some(t) => {
                    let _ = writeln!(err, "{label}:");
                    let _ = t.render_human(&mut err);
                    if !score_suffix.is_empty() {
                        let _ = writeln!(err, " {}", score_suffix.trim_start());
                    }
                }
                None => {
                    let _ = writeln!(err, "{label}: no analysis (degenerate input); kept source");
                }
            }
        }
        None => {
            if !global.quiet {
                match trace {
                    Some(t) => {
                        eprintln!("{label}: {}{score_suffix}", t.summary_line());
                        // The `--timing` readout rides the default summary on stderr
                        // (folded into `--json` on the JSON channel above) — SPEC-088.
                        if let Some(tm) = t.timing {
                            eprintln!(
                                "  timing: decode {:.1} ms \u{b7} encode {:.1} ms \u{b7} total {:.1} ms",
                                tm.decode_ms, tm.encode_ms, tm.total_ms,
                            );
                        }
                    }
                    None => eprintln!("{label}: kept source (degenerate input)"),
                }
            }
        }
    }
    // SPEC-090 / DEC-075: when the shipped output ends up LARGER than the source, say
    // so explicitly on stderr — on EVERY channel, including `--json` (whose report
    // goes to stdout, leaving the user no heads-up otherwise). `web` downscales to a
    // dimension bound, so an already-small large-dimension source can re-encode
    // larger; `optimize` hits this only on the rare metadata-forced re-encode. Either
    // way the source could not ship unchanged, so the smallest correct output was
    // kept — and the never-bigger wording must not hide it. Respects `--quiet` like
    // the other stderr diagnostics; the `--json` flag still carries the machine signal.
    if !global.quiet {
        if let Some(t) = trace {
            if t.exceeds_source() {
                eprintln!(
                    "{label}: note: shipped {} B, larger than the {} B source ({}% larger) — the \
                     source could not ship unchanged (metadata stripped / orientation baked / \
                     resized to the requested bound), so the smallest correct output was kept",
                    t.out_bytes,
                    t.source_bytes,
                    -t.savings_percent(),
                );
            }
        }
    }
    Ok(())
}

/// Write one auto-decided output through the appropriate sink (bytes are already
/// encoded — DEC-016 — so this never re-encodes).
fn write_optimize_output(
    output: &OptimizeOutput,
    input: &crate::source::Input,
    sink: &Sink,
    overwrite: Overwrite,
) -> Result<(), CliError> {
    let (bytes, ext) = match output {
        OptimizeOutput::Encoded { bytes, ext } => (bytes.as_slice(), ext.as_str()),
        OptimizeOutput::Passthrough { raw, ext } => (raw.as_slice(), ext.as_str()),
    };
    let sink_input = SinkInput {
        stem: input.stem(),
        path: input.path(),
    };
    sink.write_bytes(
        bytes,
        &sink_input,
        ext,
        overwrite,
        &mut std::io::stdout().lock(),
    )?;
    Ok(())
}

/// The format auto-decision fan-out for `optimize` (SPEC-048) — mirrors
/// [`run_pixel_op`]'s resolve + single/multi structure, but per input it decides
/// the output format via the engine instead of preserving the source format.
#[allow(clippy::too_many_arguments)]
fn run_optimize_autodecide(
    pipeline: &Pipeline,
    inputs: &[String],
    auto: &AutoQuality,
    profile: crate::analysis::decide::Profile,
    explain: Option<ExplainFmt>,
    // Measure + report decode/encode/total per image (`--timing`, SPEC-088).
    timing: bool,
    global: &GlobalArgs,
    // Always score the decided winner (SPEC-085 `web` / terminal-`optimize` recipes,
    // on downscaled output). `optimize` (keep-dimensions default) passes `false`.
    always_score: bool,
) -> Result<(), CliError> {
    // The JSON report and `-o -`'s image bytes would both land on stdout (SPEC-088).
    reject_json_report_on_stdout_sink(explain, global)?;

    let mut all: Vec<crate::source::Input> = Vec::new();
    let mut stdin_lock = std::io::stdin().lock();
    for arg in inputs {
        all.extend(source::resolve(arg, &mut stdin_lock)?);
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
        let label = input
            .path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| input.stem().to_owned());
        let (output, trace, score) =
            optimize_decide_one(input, pipeline, auto, profile, always_score, timing)?;
        emit_optimize_report(&label, trace.as_ref(), score, explain, global)?;

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
            Sink::Stdout { format: None }
        };
        write_optimize_output(&output, input, &sink, overwrite)?;
    } else {
        let out_dir = global
            .out_dir
            .as_ref()
            .ok_or_else(|| CliError::Usage("multiple inputs require --out-dir".into()))?;
        let template = global
            .name_template
            .clone()
            .unwrap_or_else(|| "{stem}.{ext}".to_owned());
        let total = all.len();
        let mut failed = 0usize;

        for input in &all {
            let label = match input {
                crate::source::Input::Path(p) => p.display().to_string(),
                crate::source::Input::Stdin { stem, .. } => stem.clone(),
            };
            let result = (|| -> Result<(), CliError> {
                let (output, trace, score) =
                    optimize_decide_one(input, pipeline, auto, profile, always_score, timing)?;
                emit_optimize_report(&label, trace.as_ref(), score, explain, global)?;
                let sink = Sink::Dir {
                    dir: PathBuf::from(out_dir),
                    template: template.clone(),
                    format: None,
                };
                write_optimize_output(&output, input, &sink, overwrite)
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

// ── responsive command (SPEC-024, DEC-026) ────────────────────────────────────

/// Parse a comma-separated `--widths` list into sorted, deduped positive widths.
/// Empty / zero / non-integer entries are a usage error (exit 2).
fn parse_widths(s: &str) -> Result<Vec<u32>, CliError> {
    let mut out: Vec<u32> = Vec::new();
    for part in s.split(',') {
        let t = part.trim();
        if t.is_empty() {
            return Err(CliError::Usage(format!(
                "invalid --widths '{s}': empty width entry"
            )));
        }
        let w: u32 = t.parse().map_err(|_| {
            CliError::Usage(format!(
                "invalid --widths '{s}': '{t}' is not a positive integer"
            ))
        })?;
        if w == 0 {
            return Err(CliError::Usage(format!(
                "invalid --widths '{s}': width must be > 0"
            )));
        }
        out.push(w);
    }
    if out.is_empty() {
        return Err(CliError::Usage(
            "--widths must list at least one width".into(),
        ));
    }
    out.sort_unstable();
    out.dedup();
    Ok(out)
}

/// Resolve `--formats` (a comma list) to ordered `ImageFormat`s, defaulting to the
/// input's `source` format. An unknown format string is a `SinkError` (exit 4) via
/// [`resolve_format`], mirroring `convert`.
fn parse_formats(
    s: Option<&str>,
    source: ::image::ImageFormat,
) -> Result<Vec<::image::ImageFormat>, CliError> {
    match s {
        None => Ok(vec![source]),
        Some(list) => {
            let mut out = Vec::new();
            for part in list.split(',') {
                let t = part.trim();
                if t.is_empty() {
                    return Err(CliError::Usage(format!(
                        "invalid --formats '{list}': empty format entry"
                    )));
                }
                let fmt = resolve_format(Some(t))?
                    .ok_or_else(|| CliError::Usage("empty --formats entry".into()))?;
                out.push(fmt);
            }
            Ok(out)
        }
    }
}

/// The HTML `type="…"` MIME for an output format.
fn mime_for_format(fmt: ::image::ImageFormat) -> String {
    match fmt {
        ::image::ImageFormat::Jpeg => "image/jpeg".to_owned(),
        ::image::ImageFormat::Png => "image/png".to_owned(),
        ::image::ImageFormat::WebP => "image/webp".to_owned(),
        ::image::ImageFormat::Avif => "image/avif".to_owned(),
        ::image::ImageFormat::Gif => "image/gif".to_owned(),
        other => format!("image/{}", crate::sink::extension_for_format(other)),
    }
}

/// The default lossy encode quality for a `responsive` variant: an explicit `-q`,
/// else 80 for a lossy format; `None` (lossless) for formats without a quality knob.
fn responsive_quality(fmt: ::image::ImageFormat, q: Option<u8>) -> Option<u8> {
    if fmt.supports_lossy_quality() {
        Some(q.unwrap_or(DEFAULT_LOSSY_QUALITY))
    } else {
        None
    }
}

/// Build the resize op params for a width-target `fit` (preserve aspect, no upscale):
/// `fit W × BIG` where BIG is large enough that width always binds.
fn fit_width_params(width: u32) -> OperationParams {
    use std::collections::BTreeMap;
    let mut map: BTreeMap<String, toml::Value> = BTreeMap::new();
    map.insert("mode".into(), toml::Value::String("fit".into()));
    map.insert("width".into(), toml::Value::Integer(width as i64));
    map.insert("height".into(), toml::Value::Integer(1_000_000));
    OperationParams::from_map(map)
}

/// Build the `<picture>`/srcset HTML for the generated variants (pure; unit-tested).
///
/// `variants` is one entry per output format (in emission order), each with its
/// `(actual_width, filename)` rows. A single format emits a bare `<img srcset>`;
/// multiple formats emit `<picture>` with one `<source>` per format + an `<img>`
/// fallback (`fallback_file` at `fallback_w`×`fallback_h`).
fn build_picture_html(
    variants: &[(::image::ImageFormat, Vec<(u32, String)>)],
    fallback_file: &str,
    fallback_w: u32,
    fallback_h: u32,
) -> String {
    fn srcset(rows: &[(u32, String)]) -> String {
        rows.iter()
            .map(|(w, f)| format!("{f} {w}w"))
            .collect::<Vec<_>>()
            .join(", ")
    }

    if variants.len() == 1 {
        format!(
            "<img srcset=\"{}\" src=\"{fallback_file}\" width=\"{fallback_w}\" height=\"{fallback_h}\" alt=\"\">",
            srcset(&variants[0].1)
        )
    } else {
        let mut s = String::from("<picture>\n");
        for (fmt, rows) in variants {
            s.push_str(&format!(
                "  <source type=\"{}\" srcset=\"{}\">\n",
                mime_for_format(*fmt),
                srcset(rows)
            ));
        }
        s.push_str(&format!(
            "  <img src=\"{fallback_file}\" width=\"{fallback_w}\" height=\"{fallback_h}\" alt=\"\">\n</picture>"
        ));
        s
    }
}

/// Wire the `responsive` subcommand (SPEC-024, DEC-026): decode once, write one
/// width-scaled variant per (width × format) into the global `--out-dir`, and print
/// a paste-ready `<picture>`/srcset snippet to stdout (unless `--no-snippet`).
///
/// Resizes by target WIDTH via the resize `fit` mode (preserve aspect, NEVER
/// upscale); widths above the source width are skipped with a warning; variants
/// dedupe by actual width. Output formats default to the input's; a feature-gated
/// unbuilt codec exits 4 up front (DEC-004), before any file is written.
pub(super) fn run_responsive(
    input: &str,
    widths: &str,
    formats: Option<&str>,
    no_snippet: bool,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    use std::collections::BTreeSet;
    use std::io::Write;

    let out_dir = global
        .out_dir
        .as_ref()
        .ok_or_else(|| CliError::Usage("responsive requires --out-dir".into()))?;

    let widths = parse_widths(widths)?;

    // Decode ONCE (DEC-002).
    let img = Image::load(input)?;
    let src_w = img.width();

    // Resolve formats (default = source) and fail up front for an unbuilt codec.
    let formats = parse_formats(formats, img.source_format())?;
    for &fmt in &formats {
        crate::sink::ensure_codec_built(fmt).map_err(CliError::Sink)?;
    }

    // Surviving widths: ≤ source width (skip larger — no upscaling).
    let mut surviving: Vec<u32> = Vec::new();
    for &w in &widths {
        if w > src_w {
            if !global.quiet {
                eprintln!(
                    "warning: width {w} exceeds source width {src_w}; skipped (no upscaling)"
                );
            }
        } else {
            surviving.push(w);
        }
    }
    if surviving.is_empty() {
        return Err(CliError::Usage(format!(
            "no requested width is ≤ the source width ({src_w}px); nothing to generate"
        )));
    }

    // NOTE: responsive builds Sink::File paths via safe_join (not Sink::Dir),
    // so it cannot rely on the Sink::Dir auto-create (DEC-044). Kept explicit.
    std::fs::create_dir_all(out_dir).map_err(|e| CliError::Sink(SinkError::Io(e)))?;

    let stem = Path::new(input)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("image")
        .to_owned();
    let overwrite = if global.yes {
        Overwrite::Allow
    } else {
        Overwrite::Forbid
    };

    let registry = OperationRegistry::with_builtins();
    let map_registry_err = |e| match e {
        RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason),
        RegistryError::Unknown { name } => CliError::Usage(format!("unknown operation '{name}'")),
    };

    let mut per_format: Vec<(::image::ImageFormat, Vec<(u32, String)>)> =
        formats.iter().map(|&f| (f, Vec::new())).collect();
    let mut seen_widths: BTreeSet<u32> = BTreeSet::new();
    let mut fallback: (u32, u32) = (0, 0); // (actual_width, actual_height) of the widest variant

    for &w in &surviving {
        // Resize to width w (preserve aspect, no upscale).
        let op = registry
            .build("resize", &fit_width_params(w))
            .map_err(map_registry_err)?;
        let pipeline = Pipeline::new().push(op);
        let out = pipeline.run(img.clone())?;
        let aw = out.width();
        let ah = out.height();
        // Dedupe by actual width (multiple requested widths can clamp to the same).
        if !seen_widths.insert(aw) {
            continue;
        }
        if aw >= fallback.0 {
            fallback = (aw, ah);
        }

        for (i, &fmt) in formats.iter().enumerate() {
            let ext = crate::sink::extension_for_format(fmt);
            let name = format!("{stem}-{aw}w.{ext}");
            let path = crate::sink::safe_join(Path::new(out_dir), &name).map_err(CliError::Sink)?;
            let sink = Sink::File {
                path,
                format: Some(fmt),
            };
            let sink_input = SinkInput {
                stem: &stem,
                path: Some(Path::new(input)),
            };
            // File sink writes to the path; the `out` writer is unused (discard it).
            sink.write(
                &out,
                &sink_input,
                overwrite,
                responsive_quality(fmt, global.quality),
                &mut std::io::sink(),
            )?;
            per_format[i].1.push((aw, name));
        }
    }

    if !no_snippet {
        let last_fmt = formats[formats.len() - 1];
        let fallback_file = format!(
            "{stem}-{}w.{}",
            fallback.0,
            crate::sink::extension_for_format(last_fmt)
        );
        let html = build_picture_html(&per_format, &fallback_file, fallback.0, fallback.1);
        let mut out = std::io::stdout().lock();
        writeln!(out, "{html}").map_err(crate::sink::SinkError::Io)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── SPEC-084: metadata-forced never-bigger fallback format ────────────────

    /// The compact lossy fallback picks the source's own lossy family when built,
    /// else a baseline JPEG — and refuses (keeps lossless) only when there is no
    /// alpha-capable lossy codec for an alpha source, or the source is lossless.
    #[test]
    fn fast_fallback_lossy_entry_prefers_a_compact_lossy_format() {
        use crate::analysis::decide::{BuiltCodecs, Disposition};
        use ::image::ImageFormat::{Avif, Jpeg, Png, WebP};

        let all = BuiltCodecs {
            webp_lossy: true,
            avif: true,
        };
        let none = BuiltCodecs {
            webp_lossy: false,
            avif: false,
        };

        // A JPEG source → JPEG (no alpha), regardless of what else is built.
        let j = fast_fallback_lossy_entry(Jpeg, false, none).expect("jpeg fallback");
        assert_eq!((j.fmt, j.disposition), (Jpeg, Disposition::Lossy));

        // Source's own family when built: AVIF → AVIF, WebP → lossy WebP.
        assert_eq!(
            fast_fallback_lossy_entry(Avif, false, all).unwrap().fmt,
            Avif
        );
        assert_eq!(
            fast_fallback_lossy_entry(WebP, false, all).unwrap().fmt,
            WebP
        );

        // Own codec NOT built → fall back to JPEG (no alpha).
        assert_eq!(
            fast_fallback_lossy_entry(Avif, false, none).unwrap().fmt,
            Jpeg
        );
        assert_eq!(
            fast_fallback_lossy_entry(WebP, false, none).unwrap().fmt,
            Jpeg
        );

        // Alpha + only JPEG available → no compact lossy option → keep lossless.
        assert!(fast_fallback_lossy_entry(Jpeg, true, none).is_none());
        assert!(fast_fallback_lossy_entry(WebP, true, none).is_none());
        // Alpha with an alpha-capable lossy codec built is fine.
        assert_eq!(
            fast_fallback_lossy_entry(WebP, true, all).unwrap().fmt,
            WebP
        );
        assert_eq!(
            fast_fallback_lossy_entry(Avif, true, all).unwrap().fmt,
            Avif
        );

        // A LOSSLESS source (PNG) never gets a lossy fallback — its lossless
        // candidates are the correct family.
        assert!(fast_fallback_lossy_entry(Png, false, all).is_none());
    }

    // ── SPEC-017: parse_size ───────────────────────────────────────────────────

    #[test]
    fn parse_size_units() {
        assert_eq!(parse_size("200000").unwrap(), 200_000);
        assert_eq!(parse_size("200KB").unwrap(), 200_000);
        assert_eq!(parse_size("200k").unwrap(), 200_000);
        assert_eq!(parse_size("1.5MB").unwrap(), 1_500_000);
        assert_eq!(parse_size("1KiB").unwrap(), 1_024);
        assert_eq!(parse_size("2MiB").unwrap(), 2_097_152);
        assert_eq!(parse_size("512B").unwrap(), 512);
        // Whitespace + case tolerance.
        assert_eq!(parse_size(" 200 kb ").unwrap(), 200_000);
    }

    #[test]
    fn parse_size_rejects_junk() {
        for bad in ["", "abc", "0", "0KB", "-5KB", "12GB", "1.2.3MB", "KB"] {
            let result = parse_size(bad);
            assert!(result.is_err(), "expected Err for '{bad}', got Ok");
            assert_eq!(
                result.unwrap_err().code(),
                2,
                "parse_size error for '{bad}' should be a usage error (code 2)"
            );
        }
    }

    // ── SPEC-022: optimize_auto_config ────────────────────────────────────────

    #[test]
    fn optimize_default_auto_is_fast() {
        // SPEC-084: the flagless default is the fast decision (fixed-quality,
        // AVIF-aware, never-bigger), NOT the perceptual search.
        match optimize_auto_config(None, None, None).expect("default mode") {
            AutoQuality::Fast => {}
            other => panic!("expected Fast, got {other:?}"),
        }
    }

    #[test]
    fn optimize_target_preset_sets_score() {
        for (t, want) in [
            (QualityTarget::VisuallyLossless, 90.0),
            (QualityTarget::High, 70.0),
            (QualityTarget::Medium, 50.0),
        ] {
            match optimize_auto_config(Some(t), None, None).unwrap() {
                AutoQuality::Perceptual(cfg) => {
                    assert_eq!(cfg.target, want, "preset {t:?} should map to {want}")
                }
                other => panic!("expected Perceptual, got {other:?}"),
            }
        }
    }

    #[test]
    fn optimize_ssim_sets_and_validates() {
        match optimize_auto_config(None, Some(85.0), None).unwrap() {
            AutoQuality::Perceptual(cfg) => assert_eq!(cfg.target, 85.0),
            other => panic!("expected Perceptual(85), got {other:?}"),
        }
        // Out-of-range scores are usage errors (exit 2).
        assert_eq!(
            optimize_auto_config(None, Some(150.0), None)
                .unwrap_err()
                .code(),
            2
        );
        assert_eq!(
            optimize_auto_config(None, Some(-1.0), None)
                .unwrap_err()
                .code(),
            2
        );
    }

    #[test]
    fn optimize_max_size_is_size_budget() {
        match optimize_auto_config(None, None, Some("200KB")).unwrap() {
            AutoQuality::SizeBudget(b) => assert_eq!(b, 200_000),
            other => panic!("expected SizeBudget(200000), got {other:?}"),
        }
    }

    #[test]
    fn optimize_conflicting_modes_are_usage() {
        // Defensive runtime arm behind clap's conflicts_with — every multi-Some
        // combination is a usage error (exit 2).
        assert_eq!(
            optimize_auto_config(Some(QualityTarget::High), None, Some("8KB"))
                .unwrap_err()
                .code(),
            2
        );
        assert_eq!(
            optimize_auto_config(Some(QualityTarget::High), Some(70.0), None)
                .unwrap_err()
                .code(),
            2
        );
        assert_eq!(
            optimize_auto_config(None, Some(70.0), Some("8KB"))
                .unwrap_err()
                .code(),
            2
        );
    }

    // ── SPEC-024: responsive ──────────────────────────────────────────────────

    #[test]
    fn parse_widths_ok_and_dedup_sorted() {
        assert_eq!(parse_widths("640, 320,640").unwrap(), vec![320, 640]);
        assert_eq!(parse_widths("100").unwrap(), vec![100]);
    }

    #[test]
    fn parse_widths_rejects_junk() {
        for bad in ["", "0", "abc", "320,0", "-5", "320,,640"] {
            let r = parse_widths(bad);
            assert!(r.is_err(), "expected Err for '{bad}'");
            assert_eq!(r.unwrap_err().code(), 2, "'{bad}' should be a usage error");
        }
    }

    #[test]
    fn parse_formats_defaults_and_resolves() {
        // None → the source format.
        assert_eq!(
            parse_formats(None, ::image::ImageFormat::Jpeg).unwrap(),
            vec![::image::ImageFormat::Jpeg]
        );
        // Explicit list, order preserved.
        assert_eq!(
            parse_formats(Some("webp,jpeg"), ::image::ImageFormat::Png).unwrap(),
            vec![::image::ImageFormat::WebP, ::image::ImageFormat::Jpeg]
        );
        // Unknown format → exit 4 (SinkError::UnsupportedExtension via resolve_format).
        assert_eq!(
            parse_formats(Some("xyz"), ::image::ImageFormat::Jpeg)
                .unwrap_err()
                .code(),
            4
        );
    }

    #[test]
    fn mime_for_format_maps_core() {
        assert_eq!(mime_for_format(::image::ImageFormat::Jpeg), "image/jpeg");
        assert_eq!(mime_for_format(::image::ImageFormat::Png), "image/png");
        assert_eq!(mime_for_format(::image::ImageFormat::WebP), "image/webp");
        assert_eq!(mime_for_format(::image::ImageFormat::Avif), "image/avif");
    }

    #[test]
    fn build_picture_html_single_vs_multi() {
        let jpeg_rows = vec![
            (320u32, "p-320w.jpg".to_owned()),
            (640u32, "p-640w.jpg".to_owned()),
        ];

        // Single format → a bare <img srcset>, no <picture>/<source>.
        let single = build_picture_html(
            &[(::image::ImageFormat::Jpeg, jpeg_rows.clone())],
            "p-640w.jpg",
            640,
            427,
        );
        assert!(
            single.contains("<img srcset="),
            "single → bare img: {single}"
        );
        assert!(single.contains("p-320w.jpg 320w"), "srcset rows: {single}");
        assert!(single.contains("640w"), "srcset rows: {single}");
        assert!(
            single.contains("src=\"p-640w.jpg\""),
            "fallback src: {single}"
        );
        assert!(single.contains("width=\"640\""), "fallback width: {single}");
        assert!(
            !single.contains("<picture>"),
            "single must not wrap in <picture>"
        );

        // Multi-format → <picture> with one <source> per format + an <img> fallback.
        let webp_rows = vec![
            (320u32, "p-320w.webp".to_owned()),
            (640u32, "p-640w.webp".to_owned()),
        ];
        let multi = build_picture_html(
            &[
                (::image::ImageFormat::WebP, webp_rows),
                (::image::ImageFormat::Jpeg, jpeg_rows),
            ],
            "p-640w.jpg",
            640,
            427,
        );
        assert!(multi.contains("<picture>"), "multi → picture: {multi}");
        assert!(
            multi.contains("type=\"image/webp\""),
            "webp source: {multi}"
        );
        assert!(
            multi.contains("type=\"image/jpeg\""),
            "jpeg source: {multi}"
        );
        assert!(
            multi.contains("<img src=\"p-640w.jpg\""),
            "fallback img: {multi}"
        );
    }
}
