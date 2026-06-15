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

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Args, Parser, Subcommand};

use crate::error::ImageError;
use crate::image::Image;
use crate::operation::OperationError;
use crate::operation::OperationRegistry;
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
    Shrink {
        inputs: Vec<String>,
        #[arg(long)]
        max: Option<u32>,
    },

    /// Re-encode to another core format (STAGE-003).
    Convert {
        inputs: Vec<String>,
        /// Target format (required for this command).
        #[arg(long)]
        format: String,
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
}

impl CliError {
    /// Map this error to its api-contract exit code (DEC-007, `docs/api-contract.md`).
    ///
    /// | Code | Meaning |
    /// |------|---------|
    /// | 1 | Generic runtime error |
    /// | 2 | Usage error (clap owns this; not returned here) |
    /// | 3 | Input not found / unreadable |
    /// | 4 | Unsupported or undeterminable format |
    /// | 5 | Output write failed / refused |
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
        Commands::Info { .. } => Err(CliError::NotImplemented("info")),
        Commands::Resize { .. } => Err(CliError::NotImplemented("resize")),
        Commands::Thumbnail { .. } => Err(CliError::NotImplemented("thumbnail")),
        Commands::Shrink { .. } => Err(CliError::NotImplemented("shrink")),
        Commands::Convert { .. } => Err(CliError::NotImplemented("convert")),
        Commands::AutoOrient { .. } => Err(CliError::NotImplemented("auto-orient")),
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
    }
}
