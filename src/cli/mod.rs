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

use crate::error::ImageError;
use crate::image::Image;
use crate::operation::RegistryError;
use crate::operation::{OperationError, OperationParams, OperationRegistry};
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

    /// Apply EXIF orientation to pixels, then clear the orientation tag (STAGE-003).
    #[command(name = "auto-orient")]
    AutoOrient { inputs: Vec<String> },

    /// Overlay an image watermark at a gravity anchor (STAGE-004).
    Watermark {
        inputs: Vec<String>,
        #[arg(long)]
        image: String,
        #[arg(long)]
        gravity: Option<String>,
        #[arg(long)]
        opacity: Option<f32>,
        #[arg(long)]
        scale: Option<f32>,
        #[arg(long)]
        margin: Option<u32>,
        #[arg(long)]
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
            // Sink errors: format errors → 4; everything else → 5
            CliError::Sink(SinkError::UnsupportedExtension(_)) => 4,
            CliError::Sink(SinkError::UnknownFormat) => 4,
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
        Commands::AutoOrient { inputs } => run_auto_orient(inputs, &cli.global),
        Commands::Watermark { .. } => Err(CliError::NotImplemented("watermark")),
        Commands::Strip { .. } => Err(CliError::NotImplemented("strip")),
        Commands::Clean { .. } => Err(CliError::NotImplemented("clean")),
        Commands::Set { .. } => Err(CliError::NotImplemented("set")),
        Commands::CopyMetadata { .. } => Err(CliError::NotImplemented("copy-metadata")),
        Commands::Edit { .. } => Err(CliError::NotImplemented("edit")),
    }
}

// ── Real apply path ───────────────────────────────────────────────────────────

/// The single-input `apply` path: recipe → pipeline → source → image → sink.
///
/// Steps (per spec Notes §step-by-step):
/// 1. Read recipe text from disk (io error → exit 3 via `CliError::RecipeIo`).
/// 2. Parse recipe TOML → `Recipe` (`RecipeError` → exit 1).
/// 3. Build registry and pipeline.
/// 4. Resolve the first input via `source::resolve` (`SourceError` → exit 3).
/// 5. Load the image (`ImageError` → exit 3/1/4).
/// 6. Run the pipeline (`OperationError` → exit 1).
/// 7. Write via the `Sink` built from global options (`SinkError` → exit 5/4).
fn run_apply(recipe_path: &str, inputs: &[String], global: &GlobalArgs) -> Result<(), CliError> {
    // Step 1: read recipe file text (map io error → exit 3).
    let recipe_text = std::fs::read_to_string(recipe_path).map_err(CliError::RecipeIo)?;

    // Step 2: parse recipe TOML.
    let recipe = Recipe::from_toml(&recipe_text)?;

    // Step 3: build registry and pipeline.
    let registry = OperationRegistry::with_builtins();
    let pipeline = recipe.build_pipeline(&registry)?;

    // Step 4: resolve the first positional input.
    let first_arg = inputs.first().map(|s| s.as_str()).unwrap_or("");
    let resolved = source::resolve(first_arg, &mut std::io::stdin().lock())?;
    let input = resolved
        .into_iter()
        .next()
        .ok_or(CliError::Source(SourceError::NotFound(
            first_arg.to_owned(),
        )))?;

    // Step 5: load image.
    let img = match &input {
        crate::source::Input::Path(p) => Image::load(p)?,
        crate::source::Input::Stdin { bytes, .. } => Image::from_bytes(bytes)?,
    };

    // Step 6: run pipeline.
    let out_img = pipeline.run(img)?;

    // Step 7: build Sink and write.
    let sink = build_sink(global)?;
    let sink_input = SinkInput {
        stem: input.stem(),
        path: input.path(),
    };
    let overwrite = if global.yes {
        Overwrite::Allow
    } else {
        Overwrite::Forbid
    };
    sink.write(
        &out_img,
        &sink_input,
        overwrite,
        global.quality,
        &mut std::io::stdout().lock(),
    )?;

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

/// Resolve the effective encode quality for ONE output (SPEC-016 / SPEC-017).
///
/// - No `auto` mode → return the fixed `quality` unchanged (today's behavior).
/// - `Perceptual` + a lossy-quality format → run the SSIMULACRA2 search; warn
///   (unless `--quiet`) if the target was unreachable (best-effort highest
///   quality / largest file).
/// - `SizeBudget` + a lossy-quality format → run the byte-budget search; warn
///   (unless `--quiet`) if even minimum quality exceeds the budget (best-effort
///   smallest file).
/// - any auto mode + a format without a quality knob → ignore it (encoder
///   default); a `SizeBudget` on such a format additionally warns that a byte
///   budget needs a lossy format. (Mirrors how `-q` is ignored for lossless
///   formats, DEC-016.)
///
/// Which formats support the search is the single seam
/// [`LossyFormat::supports_lossy_quality`] — JPEG today; AVIF/WebP land in
/// SPEC-018/019, at which point this guard generalizes with no change here.
///
/// `label` names the input in the warnings.
fn resolve_effective_quality(
    quality: Option<u8>,
    auto: &Option<AutoQuality>,
    fmt: ::image::ImageFormat,
    out_img: &Image,
    global: &GlobalArgs,
    label: &str,
) -> Result<Option<u8>, CliError> {
    let supports_lossy = fmt.supports_lossy_quality();
    match auto {
        Some(AutoQuality::Perceptual(cfg)) if supports_lossy => {
            let choice = quality::auto_quality(out_img.pixels(), fmt, cfg)?;
            if !choice.met_target && !global.quiet {
                eprintln!(
                    "warning: {label}: could not reach the requested quality target \
                     (best effort at quality {}); the output may be larger than expected",
                    choice.quality
                );
            }
            Ok(Some(choice.quality))
        }
        Some(AutoQuality::SizeBudget(budget)) if supports_lossy => {
            let choice = quality::auto_under_size(out_img.pixels(), fmt, *budget)?;
            if !choice.met_target && !global.quiet {
                // `choice.score` carries the achieved smallest size; guard the rare
                // case where the search returns a fallback whose metric is unknown
                // (NaN) so the message never reads a bogus "0 B".
                let smallest = if choice.score.is_finite() {
                    fmt_bytes(choice.score as u64)
                } else {
                    "the smallest available size".to_owned()
                };
                eprintln!(
                    "warning: {label}: could not meet the {} budget (smallest is {} at \
                     quality {}); dimension reduction not yet supported",
                    fmt_bytes(*budget),
                    smallest,
                    choice.quality
                );
            }
            Ok(Some(choice.quality))
        }
        // Format without a quality knob: no byte-budget search exists for it yet.
        Some(AutoQuality::SizeBudget(_)) => {
            if !global.quiet {
                eprintln!(
                    "warning: {label}: --max-size currently supports only JPEG output; \
                     {} was left at encoder default",
                    format_label(fmt)
                );
            }
            Ok(None)
        }
        // Format without a quality knob: perceptual target ignored (encoder default).
        Some(AutoQuality::Perceptual(_)) => Ok(None),
        None => Ok(quality),
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
        let effective_quality =
            resolve_effective_quality(quality, &auto, fmt, &out_img, global, &label)?;

        let sink_input = SinkInput {
            stem: input.stem(),
            path: input.path(),
        };
        sink.write(
            &out_img,
            &sink_input,
            overwrite,
            effective_quality,
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

                // Per-input effective quality (auto-quality search for JPEG, else
                // fixed). A scoring failure here is a per-input failure → exit 6.
                let effective_quality =
                    resolve_effective_quality(quality, &auto, fmt, &out_img, global, &label)?;

                let sink_input = SinkInput {
                    stem: input.stem(),
                    path: input.path(),
                };
                sink.write(
                    &out_img,
                    &sink_input,
                    overwrite,
                    effective_quality,
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
}
