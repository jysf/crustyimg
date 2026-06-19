//! clap subcommand surface, dispatch, and exit-code mapping (SPEC-007).
//!
//! This module is the **binary boundary**: it owns the clap derive types
//! (`Cli`, `GlobalArgs`, `Commands`), the `CliError` enum with its exit-code
//! mapping, the `run()` entry point the thin `main.rs` calls, and the real
//! end-to-end `apply` path (DEC-012, DEC-007).
//!
//! Every other subcommand is a **stub**: it parses its documented args and
//! returns `CliError::NotImplemented` (exit 1). Real implementations land in
//! STAGE-002 through STAGE-005.
//!
//! Layering: the pixel core (`src/image/`, `src/operation/`) must NOT depend
//! on `clap`; clap is isolated here and in `main.rs` (DEC-012).
//!
//! SPEC-007 adds the [`cli`] module: the clap subcommand surface + dispatch +
//! exit-code mapping (DEC-012, DEC-007).

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use crate::error::ImageError;
use crate::image::Image;
use crate::operation::RegistryError;
use crate::operation::{Gravity, OperationError, OperationParams, OperationRegistry, Watermark};
use crate::pipeline::Pipeline;
use crate::quality::{self, LossyFormat, QualityError, SearchConfig};
use crate::recipe::{Recipe, RecipeError};
use crate::sink::{Overwrite, Sink, SinkError, SinkInput};
use crate::source::{self, SourceError};

// ── Parser types (clap derive) ───────────────────────────────────────────────

/// crustyimg — fast Rust image CLI.
///
/// View and transform images in the terminal via a load-once pipeline
/// and reusable TOML recipes.
#[derive(Parser, Debug)]
#[command(name = "crustyimg", version, about)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,
    #[command(subcommand)]
    pub command: Commands,
}

/// Global options, available on every subcommand.
///
/// Flattened into [`Cli`] and marked `global = true` so clap propagates each
/// flag into subcommand contexts — allowing them to appear before OR after
/// the subcommand name on the command line.
#[derive(Args, Debug)]
pub struct GlobalArgs {
    /// Output file for single-input commands; `-` writes to stdout.
    #[arg(short = 'o', long, global = true)]
    pub output: Option<String>,

    /// Output directory for multi-input / batch commands.
    #[arg(long, global = true)]
    pub out_dir: Option<String>,

    /// Output name template, e.g. `{stem}_web.{ext}`.
    #[arg(long, global = true)]
    pub name_template: Option<String>,

    /// Parallel workers for batch (placeholder; honored in STAGE-005, DEC-006).
    #[arg(short = 'j', long, global = true)]
    pub jobs: Option<usize>,

    /// Force output format (else inferred from `-o` extension or kept).
    #[arg(long, global = true)]
    pub format: Option<String>,

    /// Encoder quality 0-100 (where the format supports it, e.g. JPEG).
    #[arg(short = 'q', long, global = true)]
    pub quality: Option<u8>,

    /// Increase verbosity; repeatable (`-vv` for more). Logs to stderr.
    #[arg(short = 'v', long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Suppress non-error output.
    #[arg(short = 'Q', long, global = true)]
    pub quiet: bool,

    /// Assume "yes" to overwrite prompts (non-interactive).
    #[arg(short = 'y', long, global = true)]
    pub yes: bool,

    /// Opt out of the default drop-GPS policy on pixel-lane encodes.
    #[arg(long, global = true)]
    pub keep_gps: bool,
}

/// A perceptual auto-quality preset for `shrink --target` (SPEC-016, DEC-019).
///
/// Each preset maps to a target SSIMULACRA2 score (higher = closer to the
/// original). clap renders the variants in kebab-case on the command line:
/// `visually-lossless`, `high`, `medium`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum QualityTarget {
    /// Indistinguishable from the original under normal viewing (score ≈ 90).
    VisuallyLossless,
    /// High quality; artifacts not noticeable at normal viewing (score ≈ 70).
    High,
    /// Medium quality; artifacts visible on close inspection (score ≈ 50).
    Medium,
}

impl QualityTarget {
    /// The target SSIMULACRA2 score this preset aims for (DEC-019 anchors).
    fn target_score(self) -> f64 {
        match self {
            QualityTarget::VisuallyLossless => 90.0,
            QualityTarget::High => 70.0,
            QualityTarget::Medium => 50.0,
        }
    }
}

/// An opt-in auto-quality mode for `shrink`/`convert`: the encoder quality is
/// searched per output instead of fixed (SPEC-016 / SPEC-017). Both modes run
/// only for a format with a lossy quality knob (JPEG today; ignored otherwise,
/// DEC-019) — see [`LossyFormat::supports_lossy_quality`]. The search lives in
/// `crate::quality`.
#[derive(Debug, Clone)]
pub enum AutoQuality {
    /// Lowest quality whose decoded round-trip scores ≥ the SSIMULACRA2 target
    /// (`--target`/`--ssim`, SPEC-016).
    Perceptual(SearchConfig),
    /// Highest quality whose encoded size ≤ the byte budget (`--max-size`,
    /// SPEC-017). The `u64` is the budget in bytes.
    SizeBudget(u64),
}

/// The full MVP subcommand surface from `docs/api-contract.md`.
///
/// Each variant carries that command's documented positional and named args.
/// Stub commands parse their args and return `CliError::NotImplemented`.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Display an image in the terminal via viuer (STAGE-002; stub in STAGE-001).
    View {
        input: String,
        #[arg(long)]
        width: Option<u32>,
        #[arg(long)]
        height: Option<u32>,
    },

    /// Print image info: dimensions, format, byte size, color type, EXIF/ICC (STAGE-002).
    Info {
        input: String,
        #[arg(long)]
        exif: bool,
        #[arg(long)]
        json: bool,
    },

    /// Perceptual comparison: SSIMULACRA2 score of <b> vs <a> (STAGE-009, DEC-025).
    /// `--fail-under <N>` exits 7 when the score is below N — a CI visual-regression
    /// gate. `--json` emits a machine-readable result.
    Diff {
        a: String,
        b: String,
        #[arg(long, value_name = "N")]
        fail_under: Option<f64>,
        #[arg(long)]
        json: bool,
    },

    /// Resize one or more images (STAGE-003).
    #[command(group = clap::ArgGroup::new("mode")
        .required(true)
        .args(["max", "exact", "percent", "fit", "fill", "cover"]))]
    Resize {
        inputs: Vec<String>,
        #[arg(long)]
        max: Option<u32>,
        #[arg(long)]
        exact: Option<String>,
        #[arg(long)]
        percent: Option<f32>,
        #[arg(long)]
        fit: Option<String>,
        #[arg(long)]
        fill: Option<String>,
        #[arg(long)]
        cover: Option<String>,
    },

    /// Convenience resize to a small bounded size (STAGE-003).
    Thumbnail {
        inputs: Vec<String>,
        #[arg(long)]
        size: Option<u32>,
        #[arg(long)]
        square: bool,
    },

    /// Optimize-for-web: resize + quality encode + strip metadata (STAGE-003).
    /// `--target`/`--ssim` auto-tune the JPEG quality to a perceptual target
    /// (SPEC-016, DEC-019).
    Shrink {
        inputs: Vec<String>,
        #[arg(long)]
        max: Option<u32>,
        /// Auto-tune the JPEG quality to a perceptual preset (SSIMULACRA2).
        #[arg(long, value_enum)]
        target: Option<QualityTarget>,
        /// Auto-tune the JPEG quality to a specific SSIMULACRA2 score (0-100).
        #[arg(long, conflicts_with = "target")]
        ssim: Option<f64>,
        /// Auto-tune the JPEG quality to fit a byte budget, e.g. `200KB`
        /// (SPEC-017).
        #[arg(long, value_name = "SIZE", conflicts_with_all = ["target", "ssim"])]
        max_size: Option<String>,
    },

    /// Re-encode to another core format (STAGE-003).
    Convert {
        inputs: Vec<String>,
        /// Target format (required for this command).
        #[arg(long)]
        format: String,
        /// Auto-tune the JPEG quality to fit a byte budget, e.g. `200KB`
        /// (SPEC-017; JPEG target only).
        #[arg(long, value_name = "SIZE")]
        max_size: Option<String>,
    },

    /// One-button web-good: auto-orient + strip metadata + perceptual re-encode,
    /// visually-lossless by default, format/size-preserving (STAGE-009, DEC-024).
    /// `--target`/`--ssim`/`--max-size` override the outcome; `--max` optionally
    /// bounds the long edge; `-o`/`--format` pick the output format.
    Optimize {
        inputs: Vec<String>,
        /// Optional long-edge bound (no resize by default).
        #[arg(long)]
        max: Option<u32>,
        /// Override the default visually-lossless target with a preset.
        #[arg(long, value_enum)]
        target: Option<QualityTarget>,
        /// Override with a specific SSIMULACRA2 score (0-100).
        #[arg(long, conflicts_with = "target")]
        ssim: Option<f64>,
        /// Re-encode to fit a byte budget instead, e.g. `200KB`.
        #[arg(long, value_name = "SIZE", conflicts_with_all = ["target", "ssim"])]
        max_size: Option<String>,
    },

    /// Generate a responsive image set: width-scaled variants per format + a
    /// paste-ready <picture>/srcset snippet on stdout (STAGE-009, DEC-026).
    /// Uses the global `--out-dir` (created if missing); resizes by target width,
    /// never upscaling.
    Responsive {
        input: String,
        /// Comma-separated target widths in px, e.g. `320,640,1280`.
        #[arg(long, value_name = "W1,W2,...")]
        widths: String,
        /// Comma-separated output formats (default: the input's format), e.g. `webp,jpeg`.
        #[arg(long, value_name = "F1,F2,...")]
        formats: Option<String>,
        /// Suppress the <picture>/srcset snippet on stdout.
        #[arg(long)]
        no_snippet: bool,
    },

    /// Apply EXIF orientation to pixels, then clear the orientation tag (STAGE-003).
    #[command(name = "auto-orient")]
    AutoOrient { inputs: Vec<String> },

    /// Overlay an image OR text watermark at a gravity anchor (STAGE-004).
    ///
    /// Exactly one of `--image` (SPEC-029) or `--text` (SPEC-030) is required;
    /// they are mutually exclusive. `--scale`/`--tile` are image-only;
    /// `--font`/`--size`/`--color` are text-only.
    Watermark {
        inputs: Vec<String>,
        /// Overlay image path (image mode; XOR `--text`).
        #[arg(long, conflicts_with = "text")]
        image: Option<String>,
        /// Text to render (text mode; XOR `--image`).
        #[arg(long, required_unless_present = "image")]
        text: Option<String>,
        /// Font file (TTF/OTF) for `--text`; defaults to the bundled Go font.
        #[arg(long, conflicts_with = "image")]
        font: Option<String>,
        /// Font size in px for `--text` (default 32.0).
        #[arg(long, conflicts_with = "image")]
        size: Option<f32>,
        /// Text color hex (RRGGBB / #RRGGBB / RRGGBBAA) for `--text` (default ffffff).
        #[arg(long, conflicts_with = "image")]
        color: Option<String>,
        #[arg(long)]
        gravity: Option<String>,
        #[arg(long)]
        opacity: Option<f32>,
        #[arg(long, conflicts_with = "text")]
        scale: Option<f32>,
        #[arg(long)]
        margin: Option<u32>,
        #[arg(long, conflicts_with = "text")]
        tile: bool,
    },

    /// Remove all metadata at the container level (STAGE-004).
    Strip { inputs: Vec<String> },

    /// Remove only GPS/location metadata (STAGE-004).
    Clean {
        inputs: Vec<String>,
        #[arg(long)]
        gps: bool,
    },

    /// Write specific EXIF tags; pixels untouched (STAGE-004).
    Set {
        inputs: Vec<String>,
        #[arg(long)]
        artist: Option<String>,
        #[arg(long)]
        copyright: Option<String>,
        #[arg(long)]
        description: Option<String>,
    },

    /// Copy metadata from one image's container to another's (STAGE-004).
    #[command(name = "copy-metadata")]
    CopyMetadata {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
    },

    /// One-shot multi-op on a single image; optionally saves the recipe (STAGE-005).
    Edit {
        input: String,
        #[arg(long)]
        save_recipe: Option<String>,
    },

    /// Run a saved recipe over one image or a batch (STAGE-005; single-input wired here).
    Apply {
        #[arg(long)]
        recipe: String,
        inputs: Vec<String>,
    },
}

// ── CliError + exit-code mapping ─────────────────────────────────────────────

/// Typed CLI error: wraps each library error module plus a `NotImplemented`
/// stub variant. The `code()` method maps each variant to the api-contract
/// exit code (DEC-007).
///
/// Only ONE place keeps the mapping — here — so unit tests catch any drift.
#[derive(Debug, thiserror::Error)]
pub enum CliError {
    /// A source-resolution error: file not found, glob empty, stdin I/O.
    #[error(transparent)]
    Source(#[from] SourceError),

    /// An image load / decode error.
    #[error(transparent)]
    Image(#[from] ImageError),

    /// A recipe parse / version / unknown-op error.
    #[error(transparent)]
    Recipe(#[from] RecipeError),

    /// A pipeline operation failed.
    #[error(transparent)]
    Operation(#[from] OperationError),

    /// A sink write error.
    #[error(transparent)]
    Sink(#[from] SinkError),

    /// The subcommand is not yet implemented (STAGE-002+ stubs).
    #[error("{0} is not yet implemented")]
    NotImplemented(&'static str),

    /// Recipe file I/O error (reading the recipe path itself).
    #[error("could not read recipe file")]
    RecipeIo(std::io::Error),

    /// One or more inputs in a multi-input batch failed (others may have
    /// succeeded). A per-failure summary is printed to stderr before this is
    /// returned. (api-contract exit code 6.)
    #[error("{failed} of {total} inputs failed")]
    PartialBatch { failed: usize, total: usize },

    /// A usage error detected at runtime (malformed WxH, multi-input without
    /// --out-dir). Mirrors clap's exit 2. Diagnostics go to stderr.
    #[error("{0}")]
    Usage(String),

    /// A perceptual scoring / quality-search failure (SPEC-016). Generic runtime
    /// error → exit 1.
    #[error(transparent)]
    Quality(#[from] QualityError),

    /// A check/gate computed successfully but was NOT satisfied — e.g. `diff
    /// --fail-under` scored below the threshold (SPEC-023, DEC-025). Mapped to exit
    /// 7, distinct from a runtime error so CI can tell "regression detected" from
    /// "couldn't run". Reusable by the future EXIF audit-linter.
    #[error("check not satisfied")]
    CheckFailed,

    /// A container-lane metadata error (`strip` / `clean --gps`, SPEC-026).
    /// `UnsupportedFormat` → exit 4; `Container`/`Exif` → exit 1 (DEC-029).
    #[error(transparent)]
    Metadata(#[from] crate::metadata::MetadataError),
}

impl CliError {
    /// Map this error to its api-contract exit code (DEC-007, `docs/api-contract.md`).
    ///
    /// | Code | Meaning |
    /// |------|---------|
    /// | 1 | Generic runtime error |
    /// | 2 | Usage error (clap owns this; also returned by Usage/InvalidPattern) |
    /// | 3 | Input not found / unreadable |
    /// | 4 | Unsupported or undeterminable format |
    /// | 5 | Output write failed / refused |
    /// | 6 | Partial batch failure (multi-input; some/all inputs failed) |
    /// | 7 | A check/gate was not satisfied (e.g. `diff --fail-under`, DEC-025) |
    pub fn code(&self) -> u8 {
        match self {
            // Source errors
            CliError::Source(SourceError::NotFound(_)) => 3,
            CliError::Source(SourceError::Stdin(_)) => 3,
            CliError::Source(SourceError::InvalidPattern { .. }) => 2,
            // Image errors
            CliError::Image(ImageError::Io(_)) => 3,
            CliError::Image(ImageError::Decode(_)) => 1,
            CliError::Image(ImageError::UnsupportedFormat) => 4,
            // Recipe / operation errors → generic runtime error
            CliError::Recipe(_) => 1,
            CliError::Operation(_) => 1,
            // Sink errors: format/codec errors → 4; everything else → 5
            CliError::Sink(SinkError::UnsupportedExtension(_)) => 4,
            CliError::Sink(SinkError::UnknownFormat) => 4,
            CliError::Sink(SinkError::CodecNotBuilt { .. }) => 4,
            CliError::Sink(_) => 5,
            // Stub commands → generic runtime error
            CliError::NotImplemented(_) => 1,
            // Recipe file read I/O → input not found / unreadable
            CliError::RecipeIo(_) => 3,
            // Partial batch failure → 6 (DEC-015)
            CliError::PartialBatch { .. } => 6,
            // Runtime usage error → 2 (mirrors clap)
            CliError::Usage(_) => 2,
            // Perceptual scoring / quality-search failure → 1 (generic runtime)
            CliError::Quality(_) => 1,
            // A check/gate was not satisfied (diff --fail-under) → 7 (DEC-025)
            CliError::CheckFailed => 7,
            // Container-lane metadata errors (SPEC-026, DEC-029):
            // unsupported format → 4; container/exif parse/rewrite → 1.
            CliError::Metadata(crate::metadata::MetadataError::UnsupportedFormat(_)) => 4,
            CliError::Metadata(crate::metadata::MetadataError::Container(_)) => 1,
            CliError::Metadata(crate::metadata::MetadataError::Exif(_)) => 1,
        }
    }
}

// ── Entry point ──────────────────────────────────────────────────────────────

/// Binary entry point: parse args, dispatch, map errors to `ExitCode`.
///
/// clap owns exit code 2 (usage errors) via `Cli::parse()`. This function
/// maps typed library errors to codes 1/3/4/5 and prints diagnostics to
/// stderr, keeping stdout clean for `-o -` pipe use.
pub fn run() -> ExitCode {
    let cli = Cli::parse(); // clap exits 2 on parse errors
    match dispatch(&cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(e.code())
        }
    }
}

// ── Dispatch ─────────────────────────────────────────────────────────────────

/// Route the parsed CLI to the real handler or a stub.
///
/// Only `Commands::Apply` is wired to a real end-to-end path; every other
/// variant calls `Err(CliError::NotImplemented(...))` (exit 1).
fn dispatch(cli: &Cli) -> Result<(), CliError> {
    match &cli.command {
        Commands::Apply { recipe, inputs } => run_apply(recipe, inputs, &cli.global),

        Commands::View {
            input,
            width,
            height,
        } => run_view(input, *width, *height, &cli.global),
        Commands::Info { input, exif, json } => run_info(input, *exif, *json, &cli.global),
        Commands::Diff {
            a,
            b,
            fail_under,
            json,
        } => run_diff(a, b, *fail_under, *json, &cli.global),
        Commands::Resize {
            inputs,
            max,
            exact,
            percent,
            fit,
            fill,
            cover,
        } => run_resize(
            inputs,
            &ResizeModes {
                max: *max,
                exact: exact.as_deref(),
                percent: *percent,
                fit: fit.as_deref(),
                fill: fill.as_deref(),
                cover: cover.as_deref(),
            },
            &cli.global,
        ),
        Commands::Thumbnail {
            inputs,
            size,
            square,
        } => run_thumbnail(inputs, *size, *square, &cli.global),
        Commands::Shrink {
            inputs,
            max,
            target,
            ssim,
            max_size,
        } => run_shrink(
            inputs,
            *max,
            *target,
            *ssim,
            max_size.as_deref(),
            &cli.global,
        ),
        Commands::Convert {
            inputs,
            format,
            max_size,
        } => run_convert(inputs, format, max_size.as_deref(), &cli.global),
        Commands::Optimize {
            inputs,
            max,
            target,
            ssim,
            max_size,
        } => run_optimize(
            inputs,
            *max,
            *target,
            *ssim,
            max_size.as_deref(),
            &cli.global,
        ),
        Commands::Responsive {
            input,
            widths,
            formats,
            no_snippet,
        } => run_responsive(input, widths, formats.as_deref(), *no_snippet, &cli.global),
        Commands::AutoOrient { inputs } => run_auto_orient(inputs, &cli.global),
        Commands::Watermark {
            inputs,
            image,
            text,
            font,
            size,
            color,
            gravity,
            opacity,
            scale,
            margin,
            tile,
        } => run_watermark(
            inputs,
            &WatermarkSource {
                image: image.as_deref(),
                text: text.as_deref(),
                font: font.as_deref(),
                size: *size,
                color: color.as_deref(),
            },
            gravity.as_deref(),
            *opacity,
            *scale,
            *margin,
            *tile,
            &cli.global,
        ),
        Commands::Strip { inputs } => run_strip(inputs, &cli.global),
        Commands::Clean { inputs, gps } => run_clean(inputs, *gps, &cli.global),
        Commands::Set {
            inputs,
            artist,
            copyright,
            description,
        } => run_set(
            inputs,
            artist.clone(),
            copyright.clone(),
            description.clone(),
            &cli.global,
        ),
        Commands::CopyMetadata { from, to } => run_copy_metadata(from, to, &cli.global),
        Commands::Edit { .. } => Err(CliError::NotImplemented("edit")),
    }
}

// ── Real apply path ───────────────────────────────────────────────────────────

/// A known-valid `ProgressStyle` template for the batch progress bar.
///
/// Kept as a const so we can use `.unwrap_or_else(|_| ProgressStyle::default_bar())`
/// in non-test code rather than an `unwrap` on an arbitrary user-supplied string.
const BATCH_PROGRESS_TEMPLATE: &str = "{bar:40.cyan/blue} {pos}/{len} {msg}";

/// Apply one input through the recipe and write the result to `out_dir`.
///
/// Extracted from `run_apply` so it is unit-testable. Rebuilds the pipeline
/// from `recipe` + `registry` on every call — `Operation` is NOT `Send`, so
/// no pipeline may cross a thread boundary (SPEC-031, Parallel design).
///
/// - Loads the image from `input`.
/// - Builds a local pipeline via `recipe.build_pipeline(registry)`.
/// - Runs the pipeline.
/// - Writes the result to `Sink::Dir { dir, template, format }`.
/// - `format`: `None` → preserve the source format.
fn apply_one(
    recipe: &Recipe,
    registry: &OperationRegistry,
    input: &crate::source::Input,
    out_dir: &Path,
    template: &str,
    overwrite: Overwrite,
    quality: Option<u8>,
) -> Result<(), CliError> {
    // Load.
    let img = match input {
        crate::source::Input::Path(p) => Image::load(p)?,
        crate::source::Input::Stdin { bytes, .. } => Image::from_bytes(bytes)?,
    };

    // Build a fresh pipeline (Operation is not Send; each task rebuilds its own).
    let pipeline = recipe.build_pipeline(registry)?;

    // Run.
    let out_img = pipeline.run(img.clone())?;

    // Preserve the source format (no --format override in batch path v1).
    let fmt = img.source_format();

    let sink = Sink::Dir {
        dir: out_dir.to_owned(),
        template: template.to_owned(),
        format: Some(fmt),
    };

    let sink_input = SinkInput {
        stem: input.stem(),
        path: input.path(),
    };

    sink.write(
        &out_img,
        &sink_input,
        overwrite,
        quality,
        &mut std::io::stdout().lock(),
    )?;

    Ok(())
}

/// Guard: multi-input without `--out-dir` is a usage error (exit 2).
///
/// Returns `Ok(dir_path)` when `global.out_dir` is `Some`, else `CliError::Usage`.
fn require_out_dir_for_batch(global: &GlobalArgs) -> Result<&str, CliError> {
    global
        .out_dir
        .as_deref()
        .ok_or_else(|| CliError::Usage("multiple inputs require --out-dir".into()))
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
fn run_apply(recipe_path: &str, inputs: &[String], global: &GlobalArgs) -> Result<(), CliError> {
    // Step 1: read recipe file text (map io error → exit 3).
    let recipe_text = std::fs::read_to_string(recipe_path).map_err(CliError::RecipeIo)?;

    // Step 2: parse + validate recipe TOML (SPEC-006 reused).
    let recipe = Recipe::from_toml(&recipe_text)?;

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

/// The `view` path: resolve the single input, load the image, and render it
/// via the display Sink. Resolves the FIRST input when a directory/glob yields
/// many (single-image command). A non-tty stdout refuses with
/// `SinkError::NotATty` → exit 5.
fn run_view(
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

/// Build a `Sink` from the global output options.
///
/// Priority:
/// - `-o -`          → `Sink::Stdout { format }` (format from `--format`; `None` → `UnknownFormat` on write).
/// - `-o <PATH>`     → `Sink::File { path, format: optional from --format }`.
/// - `--out-dir DIR` → `Sink::Dir { dir, template, format }`.
/// - No output flag  → `Sink::File` with an empty path (caller's error if path is needed).
///
/// `--format` is a lowercase extension string (e.g. `"png"`, `"jpg"`); it is
/// converted to `image::ImageFormat` via `ImageFormat::from_extension`. An
/// unrecognised format string results in `SinkError::UnsupportedExtension → exit 4`.
fn build_sink(global: &GlobalArgs) -> Result<Sink, CliError> {
    // Convert optional `--format` string to `Option<ImageFormat>`.
    let format_opt = resolve_format(global.format.as_deref())?;

    if let Some(ref out) = global.output {
        if out == "-" {
            // Stdout sink: format must be known at write time (Sink handles None → UnknownFormat).
            return Ok(Sink::Stdout { format: format_opt });
        }
        // File sink.
        return Ok(Sink::File {
            path: PathBuf::from(out),
            format: format_opt,
        });
    }

    if let Some(ref dir) = global.out_dir {
        let template = global
            .name_template
            .clone()
            .unwrap_or_else(|| "{stem}.{ext}".to_owned());
        return Ok(Sink::Dir {
            dir: PathBuf::from(dir),
            template,
            format: format_opt,
        });
    }

    // No output specified: default to stdout (format required separately).
    // In practice the integration tests always pass -o; returning Stdout here
    // makes the error surface cleanly as UnknownFormat rather than a panic.
    Ok(Sink::Stdout { format: format_opt })
}

/// Convert an optional format string (e.g. `"png"`) to `Option<ImageFormat>`.
///
/// A `None` input returns `Ok(None)`. A non-empty string that is not a
/// recognised extension maps to `Err(CliError::Sink(SinkError::UnsupportedExtension))`.
fn resolve_format(fmt: Option<&str>) -> Result<Option<::image::ImageFormat>, CliError> {
    match fmt {
        None => Ok(None),
        Some(s) => {
            // Build a synthetic path `"_.{s}"` and reuse the sink's helper.
            let path_str = format!("_.{s}");
            let synthetic = Path::new(&path_str);
            crate::sink::format_from_extension(synthetic)
                .map(Some)
                .map_err(CliError::Sink)
        }
    }
}

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
fn format_label(fmt: ::image::ImageFormat) -> String {
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
fn escape_json(s: &str) -> String {
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
fn run_info(input: &str, exif: bool, json: bool, _global: &GlobalArgs) -> Result<(), CliError> {
    let resolved = source::resolve(input, &mut std::io::stdin().lock())?;
    let first = resolved
        .into_iter()
        .next()
        .ok_or(CliError::Source(SourceError::NotFound(input.to_owned())))?;

    // Read the raw bytes ONCE: they give the file size, the decoded
    // image, and the EXIF source. (For a path, std::fs::read io-error
    // maps to ImageError::Io → exit 3, consistent with Image::load.)
    let (raw, label): (Vec<u8>, String) = match &first {
        crate::source::Input::Path(p) => {
            let bytes = std::fs::read(p).map_err(ImageError::Io)?;
            (bytes, p.display().to_string())
        }
        crate::source::Input::Stdin { bytes, .. } => (bytes.clone(), "-".to_owned()),
    };
    let img = Image::from_bytes(&raw)?;
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
fn run_diff(
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
struct ResizeModes<'a> {
    max: Option<u32>,
    exact: Option<&'a str>,
    percent: Option<f32>,
    fit: Option<&'a str>,
    fill: Option<&'a str>,
    cover: Option<&'a str>,
}

/// Wire the `resize` subcommand: parse flags, build op via registry, fan-out.
///
/// Single-input: uses the `-o`/`-o -`/`--out-dir` sink from global flags.
/// Multi-input: requires `--out-dir`; fan-out is SEQUENTIAL (no rayon, DEC-006).
/// Partial failures in multi-input → continue + print to stderr + exit 6 (DEC-015).
fn run_resize(
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

/// Render a byte count as a short human string, e.g. `512 B`, `6.0 KB`, `1.5 MB`
/// (decimal units, matching `parse_size`). Used in the `--max-size` warnings.
fn fmt_bytes(n: u64) -> String {
    const KB: f64 = 1000.0;
    const MB: f64 = 1_000_000.0;
    let f = n as f64;
    if f >= MB {
        format!("{:.1} MB", f / MB)
    } else if f >= KB {
        format!("{:.1} KB", f / KB)
    } else {
        format!("{n} B")
    }
}

/// The encode plan for ONE output: the quality to write at, and an optional
/// replacement image when `--max-size` had to downscale the dimensions to fit the
/// byte budget (SPEC-021, DEC-023). `image: None` means write the pipeline's output
/// unchanged; `Some(_)` means write these (smaller) pixels instead.
struct EncodePlan {
    quality: Option<u8>,
    image: Option<Image>,
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
fn resolve_effective_quality(
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
        None => Ok(EncodePlan {
            quality,
            image: None,
        }),
    }
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
///   (`--max-size`, SPEC-017). Used by `run_shrink` and `run_convert`.
fn run_pixel_op(
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
fn metadata_output_ext(input: &crate::source::Input, bytes: &[u8]) -> String {
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
fn read_raw_bytes(input: &crate::source::Input) -> Result<Vec<u8>, CliError> {
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

/// Wire `strip`: remove ALL container metadata via the container lane (DEC-003).
/// Format is preserved; no pixel re-encode (`metadata-not-via-pixel-encode`).
fn run_strip(inputs: &[String], global: &GlobalArgs) -> Result<(), CliError> {
    run_metadata_lane(inputs, global, crate::metadata::strip_all)
}

/// Wire `clean --gps`: remove ONLY GPS/location metadata via the container lane.
/// `--gps` is required in v1; `clean` without it is a usage error (exit 2),
/// leaving room for future selective flags.
fn run_clean(inputs: &[String], gps: bool, global: &GlobalArgs) -> Result<(), CliError> {
    if !gps {
        return Err(CliError::Usage("clean requires --gps".into()));
    }
    run_metadata_lane(inputs, global, crate::metadata::clean_gps)
}

/// Wire `set`: write the given EXIF attribution tags into the container via the
/// container lane (DEC-003), preserving every other tag and the pixels exactly
/// (no re-encode, `metadata-not-via-pixel-encode`).
///
/// At least one of `--artist`/`--copyright`/`--description` is required; none is
/// a usage error (exit 2). Format is preserved; `-q`/`--format` are ignored.
fn run_set(
    inputs: &[String],
    artist: Option<String>,
    copyright: Option<String>,
    description: Option<String>,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    if artist.is_none() && copyright.is_none() && description.is_none() {
        return Err(CliError::Usage(
            "set requires at least one of --artist/--copyright/--description".into(),
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

/// Wire `copy-metadata --from SRC --to DST`: graft SRC's container EXIF + ICC
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
fn run_copy_metadata(from: &str, to: &str, global: &GlobalArgs) -> Result<(), CliError> {
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
fn run_thumbnail(
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

// ── shrink helpers ────────────────────────────────────────────────────────────

/// The default long-edge bound when `--max` is omitted for `shrink`.
const DEFAULT_SHRINK_MAX: u32 = 1600;

/// The default JPEG encode quality when `-q` is omitted for `shrink` (DEC-016).
const DEFAULT_SHRINK_QUALITY: u8 = 80;

/// Map shrink's `max` arg to the `Resize` OperationParams the registry expects
/// (SPEC-010's PINNED schema). `shrink` always uses `mode=max` to bound the
/// long edge; `max` defaults to `DEFAULT_SHRINK_MAX`. Infallible — the mapping
/// is total; the op validates the dim.
fn shrink_params(max: u32) -> OperationParams {
    use std::collections::BTreeMap;

    let mut map: BTreeMap<String, toml::Value> = BTreeMap::new();
    map.insert("mode".into(), toml::Value::String("max".into()));
    map.insert("width".into(), toml::Value::Integer(max as i64));
    OperationParams::from_map(map)
}

// ── shrink handler ────────────────────────────────────────────────────────────

/// Reject combining a fixed `-q/--quality` with an auto-quality mode — they are
/// mutually exclusive (one pins a quality, the other searches for it). `-q` is a
/// GLOBAL arg, so this can't be expressed as a clap `conflicts_with` against the
/// subcommand args; both `shrink` and `convert` enforce it here at runtime
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

/// Resolve `shrink`'s auto-quality target from `--target`/`--ssim` (SPEC-016).
///
/// Returns `Ok(None)` when neither flag is set (fixed-quality behavior). A
/// `--ssim` score outside `0.0..=100.0` is a usage error (exit 2). `--target`
/// and `--ssim` together are rejected by clap (`conflicts_with`) before this is
/// reached; the `(Some, Some)` arm is a defensive fallback.
fn shrink_auto_config(
    target: Option<QualityTarget>,
    ssim: Option<f64>,
) -> Result<Option<SearchConfig>, CliError> {
    match (target, ssim) {
        (Some(_), Some(_)) => Err(CliError::Usage(
            "--target and --ssim are mutually exclusive".into(),
        )),
        (Some(t), None) => Ok(Some(SearchConfig::for_target(t.target_score()))),
        (None, Some(s)) => {
            if !(0.0..=100.0).contains(&s) {
                return Err(CliError::Usage(format!(
                    "--ssim must be a score in 0..=100, got {s}"
                )));
            }
            Ok(Some(SearchConfig::for_target(s)))
        }
        (None, None) => Ok(None),
    }
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

/// Wire the `shrink` subcommand: resize to a long-edge bound + quality-aware
/// JPEG encode + inherent metadata drop (from the pixel-lane re-encode).
///
/// - `--max N` (default 1600) bounds the longest edge to N (no upscale via the
///   `resize max` mode).
/// - `--target <preset>` / `--ssim <score>` / `--max-size <SIZE>` auto-tune the
///   JPEG quality per input (SPEC-016 / SPEC-017): perceptual (lowest quality
///   clearing an SSIMULACRA2 target) or a byte budget (highest quality ≤ SIZE).
///   Opt-in; mutually exclusive with each other and with `-q`. Ignored for
///   non-JPEG outputs (encoder default), mirroring `-q` on lossless formats.
/// - `-q Q` (default 80, only when no auto mode) sets the JPEG encode quality;
///   ignored for lossless formats (DEC-016).
/// - Multi-input fan-out and partial-batch exit 6 are inherited via
///   `run_pixel_op` (DEC-015).
fn run_shrink(
    inputs: &[String],
    max: Option<u32>,
    target: Option<QualityTarget>,
    ssim: Option<f64>,
    max_size: Option<&str>,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    // Resolve the auto mode: perceptual (--target/--ssim) or byte budget
    // (--max-size). clap `conflicts_with_all` makes these mutually exclusive; the
    // `(Some, Some)` arm is a defensive fallback.
    let auto: Option<AutoQuality> = match (shrink_auto_config(target, ssim)?, max_size) {
        (Some(cfg), None) => Some(AutoQuality::Perceptual(cfg)),
        (None, Some(sz)) => Some(AutoQuality::SizeBudget(parse_size(sz)?)),
        (None, None) => None,
        (Some(_), Some(_)) => {
            return Err(CliError::Usage(
                "--max-size is mutually exclusive with --target/--ssim".into(),
            ))
        }
    };

    // `-q` pins a quality; the auto modes search for one — reject combining them.
    reject_quality_with_auto(&auto, global)?;

    let effective_max = max.unwrap_or(DEFAULT_SHRINK_MAX);
    let params = shrink_params(effective_max);

    let op = OperationRegistry::with_builtins()
        .build("resize", &params)
        .map_err(|e| match e {
            RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason),
            RegistryError::Unknown { name } => {
                CliError::Usage(format!("unknown operation '{name}'"))
            }
        })?;

    let pipeline = Pipeline::new().push(op);
    // With an auto target, the per-input search supplies the quality (pass None as
    // the fixed quality). Without it, keep today's fixed default (80).
    let fixed_quality = if auto.is_some() {
        None
    } else {
        Some(global.quality.unwrap_or(DEFAULT_SHRINK_QUALITY))
    };
    run_pixel_op(pipeline, inputs, global, fixed_quality, None, auto)
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
fn run_auto_orient(inputs: &[String], global: &GlobalArgs) -> Result<(), CliError> {
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
struct WatermarkSource<'a> {
    image: Option<&'a str>,
    text: Option<&'a str>,
    font: Option<&'a str>,
    size: Option<f32>,
    color: Option<&'a str>,
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
fn run_watermark(
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

// ── convert handler ───────────────────────────────────────────────────────────

/// Wire the `convert` subcommand: resolve the REQUIRED target format ONCE up
/// front (exit 4 for unsupported/unbuilt codec — DEC-004), then pure re-encode
/// every input to that format via an empty `Pipeline` (no-op pixel transform)
/// and the shared `run_pixel_op` fan-out with `forced_format` (DEC-015 / SPEC-014).
///
/// Quality threading: pass `global.quality` as-is; `convert` has NO forced
/// default (only `shrink` defaults quality to 80, per DEC-016). `--max-size`
/// auto-tunes the JPEG quality to a byte budget (SPEC-017; JPEG target only —
/// ignored with a warning for a lossless target format); mutually exclusive with
/// `-q`.
///
/// NOTE: the convert-local `--format` arg shadows the global `--format`, so
/// `global.format` is `None` inside `convert`; read the target from `format: &str`.
fn run_convert(
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

/// Resolve `optimize`'s auto-quality mode (SPEC-022, DEC-024).
///
/// Unlike [`shrink_auto_config`], `optimize` is ALWAYS in an auto mode: with no
/// flag the default is the **visually-lossless** perceptual target (score 90,
/// `QualityTarget::VisuallyLossless`). `--target`/`--ssim` pick a different
/// perceptual target; `--max-size` switches to a byte budget. The three are
/// mutually exclusive — clap enforces it on the subcommand args, so the trailing
/// `_` arm is a defensive runtime fallback (usage error, exit 2).
fn optimize_auto_config(
    target: Option<QualityTarget>,
    ssim: Option<f64>,
    max_size: Option<&str>,
) -> Result<AutoQuality, CliError> {
    match (target, ssim, max_size) {
        // Default: visually-lossless perceptual target.
        (None, None, None) => Ok(AutoQuality::Perceptual(SearchConfig::for_target(
            QualityTarget::VisuallyLossless.target_score(),
        ))),
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

/// Wire the `optimize` subcommand: the one-button "web-good" command (DEC-024).
///
/// Pipeline (PINNED order): `auto-orient` (bake EXIF orientation, then drop the
/// metadata bundle — DEC-017) then, iff `--max N`, a `resize max N` long-edge
/// bound (built like [`shrink_params`]). The output is re-encoded to a perceptual
/// target (visually-lossless by default) in the input's own format (DEC-015
/// precedence: `--format` > `-o` ext > preserve source — i.e. `forced_format =
/// None`). The pixel-lane re-encode drops ALL metadata (privacy incl. GPS); this is
/// NOT the selective-preserve container lane (DEC-003), which is unbuilt (STAGE-004).
///
/// `optimize` always auto-tunes quality, so a fixed `-q` conflicts (exit 2);
/// `--target`/`--ssim`/`--max-size` are mutually exclusive. Multi-input fan-out +
/// partial-batch exit 6 are inherited via [`run_pixel_op`] (DEC-015).
fn run_optimize(
    inputs: &[String],
    max: Option<u32>,
    target: Option<QualityTarget>,
    ssim: Option<f64>,
    max_size: Option<&str>,
    global: &GlobalArgs,
) -> Result<(), CliError> {
    let auto = Some(optimize_auto_config(target, ssim, max_size)?);
    // optimize always auto-tunes quality; a fixed -q conflicts.
    reject_quality_with_auto(&auto, global)?;

    let registry = OperationRegistry::with_builtins();
    let map_registry_err = |e| match e {
        RegistryError::InvalidParams { reason, .. } => CliError::Usage(reason),
        RegistryError::Unknown { name } => CliError::Usage(format!("unknown operation '{name}'")),
    };

    // Always auto-orient first so any `--max` bound applies to the visually-correct
    // dimensions; the op also drops the metadata bundle after baking (DEC-017).
    let orient = registry
        .build("auto-orient", &OperationParams::empty())
        .map_err(map_registry_err)?;
    let mut pipeline = Pipeline::new().push(orient);

    if let Some(n) = max {
        let resize = registry
            .build("resize", &shrink_params(n))
            .map_err(map_registry_err)?;
        pipeline = pipeline.push(resize);
    }

    // No fixed quality and no forced format: the auto mode drives quality, and the
    // per-input format is preserved / honored from -o/--format (DEC-015).
    run_pixel_op(pipeline, inputs, global, None, None, auto)
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
        Some(q.unwrap_or(DEFAULT_SHRINK_QUALITY))
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
fn run_responsive(
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

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── cli_parses_global_and_apply ──────────────────────────────────────────

    #[test]
    fn cli_parses_global_and_apply() {
        let cli = Cli::try_parse_from([
            "crustyimg",
            "apply",
            "--recipe",
            "r.toml",
            "in.png",
            "-o",
            "out.png",
        ])
        .expect("should parse without error");

        // The subcommand must be the Apply variant.
        match &cli.command {
            Commands::Apply { recipe, inputs } => {
                assert_eq!(recipe, "r.toml");
                assert_eq!(inputs, &["in.png"]);
            }
            other => panic!("expected Apply variant, got {other:?}"),
        }

        // The global `-o` must be captured.
        assert_eq!(cli.global.output.as_deref(), Some("out.png"));
    }

    // ── cli_unknown_subcommand_is_err ────────────────────────────────────────

    #[test]
    fn cli_unknown_subcommand_is_err() {
        let result = Cli::try_parse_from(["crustyimg", "frobnicate"]);
        assert!(
            result.is_err(),
            "expected a parse error for unknown subcommand"
        );
        let err = result.unwrap_err();
        // clap signals usage errors with exit code 2.
        assert_eq!(
            err.exit_code(),
            2,
            "clap should signal exit code 2 for usage errors"
        );
    }

    // ── exit_code_mapping_is_total ────────────────────────────────────────────

    #[test]
    fn exit_code_mapping_is_total() {
        // Source variants.
        assert_eq!(
            CliError::Source(SourceError::NotFound("x".into())).code(),
            3
        );
        assert_eq!(
            CliError::Source(SourceError::Stdin(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "pipe"
            )))
            .code(),
            3
        );
        assert_eq!(
            CliError::Source(SourceError::InvalidPattern {
                pattern: "x".into(),
                reason: "bad".into()
            })
            .code(),
            2
        );

        // Image variants.
        assert_eq!(
            CliError::Image(ImageError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no"
            )))
            .code(),
            3
        );
        assert_eq!(CliError::Image(ImageError::Decode("bad".into())).code(), 1);
        assert_eq!(CliError::Image(ImageError::UnsupportedFormat).code(), 4);

        // Recipe error → 1.
        assert_eq!(CliError::Recipe(RecipeError::Parse("bad".into())).code(), 1);

        // Operation error → 1.
        assert_eq!(
            CliError::Operation(OperationError::Apply {
                op: "x",
                reason: "fail".into()
            })
            .code(),
            1
        );

        // Sink UnsupportedExtension / UnknownFormat → 4; others → 5.
        assert_eq!(
            CliError::Sink(SinkError::UnsupportedExtension("xyz".into())).code(),
            4
        );
        assert_eq!(CliError::Sink(SinkError::UnknownFormat).code(), 4);
        // CodecNotBuilt (a recognized but feature-gated codec, e.g. AVIF without
        // the feature) → exit 4 (DEC-004 / SPEC-018).
        assert_eq!(
            CliError::Sink(SinkError::CodecNotBuilt {
                codec: "avif",
                feature: "avif"
            })
            .code(),
            4
        );
        assert_eq!(
            CliError::Sink(SinkError::AlreadyExists("f".into())).code(),
            5
        );

        // NotImplemented → 1.
        assert_eq!(CliError::NotImplemented("view").code(), 1);

        // RecipeIo → 3.
        assert_eq!(
            CliError::RecipeIo(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "no recipe"
            ))
            .code(),
            3
        );

        // PartialBatch → 6 (DEC-015).
        assert_eq!(
            CliError::PartialBatch {
                failed: 1,
                total: 3
            }
            .code(),
            6
        );

        // Usage → 2 (mirrors clap exit 2).
        assert_eq!(CliError::Usage("bad".into()).code(), 2);

        // Quality (scoring/search failure) → 1 (generic runtime).
        assert_eq!(
            CliError::Quality(QualityError::Score("scoring failed".into())).code(),
            1
        );

        // CheckFailed (diff --fail-under gate not met) → 7 (DEC-025).
        assert_eq!(CliError::CheckFailed.code(), 7);
    }

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

    // ── SPEC-017: parse_size / fmt_bytes ──────────────────────────────────────

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

    #[test]
    fn fmt_bytes_renders_units() {
        assert_eq!(fmt_bytes(512), "512 B");
        assert_eq!(fmt_bytes(6_000), "6.0 KB");
        assert_eq!(fmt_bytes(1_500_000), "1.5 MB");
    }

    // ── SPEC-022: optimize ────────────────────────────────────────────────────

    #[test]
    fn optimize_parses_args() {
        let cli = Cli::try_parse_from([
            "crustyimg",
            "optimize",
            "a.jpg",
            "--max",
            "800",
            "-o",
            "out.jpg",
        ])
        .expect("should parse");
        match &cli.command {
            Commands::Optimize { inputs, max, .. } => {
                assert_eq!(inputs, &["a.jpg"]);
                assert_eq!(*max, Some(800));
            }
            other => panic!("expected Optimize variant, got {other:?}"),
        }
        assert_eq!(cli.global.output.as_deref(), Some("out.jpg"));
    }

    #[test]
    fn optimize_default_auto_is_visually_lossless() {
        match optimize_auto_config(None, None, None).expect("default mode") {
            AutoQuality::Perceptual(cfg) => assert_eq!(cfg.target, 90.0),
            other => panic!("expected Perceptual(90), got {other:?}"),
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

    // ── SPEC-023: diff ────────────────────────────────────────────────────────

    #[test]
    fn diff_parses_args() {
        let cli =
            Cli::try_parse_from(["crustyimg", "diff", "a.png", "b.png", "--fail-under", "90"])
                .expect("should parse");
        match &cli.command {
            Commands::Diff {
                a,
                b,
                fail_under,
                json,
            } => {
                assert_eq!(a, "a.png");
                assert_eq!(b, "b.png");
                assert_eq!(*fail_under, Some(90.0));
                assert!(!*json);
            }
            other => panic!("expected Diff variant, got {other:?}"),
        }
    }

    #[test]
    fn diff_passes_gate() {
        assert!(diff_passes(95.0, Some(90.0)), "95 ≥ 90 passes");
        assert!(!diff_passes(85.0, Some(90.0)), "85 < 90 fails");
        assert!(diff_passes(12.0, None), "no gate always passes");
        // Boundary: equal to the threshold passes.
        assert!(diff_passes(90.0, Some(90.0)), "90 ≥ 90 passes");
    }

    // ── SPEC-024: responsive ──────────────────────────────────────────────────

    #[test]
    fn responsive_parses_args() {
        let cli = Cli::try_parse_from([
            "crustyimg",
            "responsive",
            "in.jpg",
            "--widths",
            "320,640",
            "--out-dir",
            "d",
            "--formats",
            "webp,jpeg",
        ])
        .expect("should parse");
        match &cli.command {
            Commands::Responsive {
                input,
                widths,
                formats,
                no_snippet,
            } => {
                assert_eq!(input, "in.jpg");
                assert_eq!(widths, "320,640");
                assert_eq!(formats.as_deref(), Some("webp,jpeg"));
                assert!(!*no_snippet);
            }
            other => panic!("expected Responsive variant, got {other:?}"),
        }
        // --out-dir is the GLOBAL flag.
        assert_eq!(cli.global.out_dir.as_deref(), Some("d"));
    }

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

    // ── SPEC-031: apply batch helpers ─────────────────────────────────────────

    /// Helper: a `GlobalArgs` with NO out_dir.
    fn global_no_out_dir() -> GlobalArgs {
        GlobalArgs {
            output: None,
            out_dir: None,
            name_template: None,
            jobs: None,
            format: None,
            quality: None,
            verbose: 0,
            quiet: false,
            yes: true,
            keep_gps: false,
        }
    }

    /// `require_out_dir_for_batch` returns `CliError::Usage` (exit 2) when
    /// `--out-dir` is absent. (Tests the guard helper directly.)
    #[test]
    fn apply_batch_requires_out_dir_for_multi() {
        let global = global_no_out_dir();
        let result = require_out_dir_for_batch(&global);
        assert!(result.is_err(), "expected Usage error");
        assert_eq!(
            result.unwrap_err().code(),
            2,
            "missing --out-dir must be code 2"
        );
    }

    /// `apply_one` on a fixture PNG with a `resize max 8` recipe produces
    /// an output no larger than 8×8.
    #[test]
    fn apply_worker_applies_recipe_to_one() {
        use std::io::Cursor;

        use image::{DynamicImage, ImageFormat, RgbImage};

        let dir = tempfile::tempdir().unwrap();

        // Write a 32×32 solid PNG.
        let img = RgbImage::from_pixel(32, 32, image::Rgb([100u8, 150u8, 200u8]));
        let mut buf = Cursor::new(Vec::new());
        DynamicImage::ImageRgb8(img)
            .write_to(&mut buf, ImageFormat::Png)
            .unwrap();
        let src_path = dir.path().join("in.png");
        std::fs::write(&src_path, buf.into_inner()).unwrap();

        // Recipe: resize max 8.
        let recipe_toml = r#"
version = "1"

[[step]]
op = "resize"
mode = "max"
width = 8
"#;
        let recipe = Recipe::from_toml(recipe_toml).unwrap();
        let registry = OperationRegistry::with_builtins();
        let out_dir = dir.path().join("out");
        std::fs::create_dir_all(&out_dir).unwrap();

        let input = crate::source::Input::Path(src_path.clone());
        apply_one(
            &recipe,
            &registry,
            &input,
            &out_dir,
            "{stem}.{ext}",
            Overwrite::Allow,
            None,
        )
        .expect("apply_one should succeed");

        let out_path = out_dir.join("in.png");
        assert!(out_path.exists(), "output file must be created");

        // Verify dimensions are ≤ 8.
        let out_img = image::open(&out_path).unwrap();
        assert!(
            out_img.width() <= 8 && out_img.height() <= 8,
            "resized image must be ≤ 8×8, got {}×{}",
            out_img.width(),
            out_img.height()
        );
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
