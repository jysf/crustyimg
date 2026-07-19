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

use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::generate;

use crate::error::ImageError;
use crate::image::Image;
use crate::operation::RegistryError;
use crate::operation::{Gravity, OperationError, OperationParams, OperationRegistry, Watermark};
use crate::pipeline::Pipeline;
use crate::quality::{QualityError, SearchConfig};
use crate::recipe::{Recipe, RecipeError};
use crate::sink::{Overwrite, Sink, SinkError, SinkInput};
use crate::source::{self, SourceError};

mod build;
mod common;
mod optimize;
// `pub(crate)` (not private): `lint::report` reuses `report::escape_json`
// across the `cli`/`lint` module boundary (SPEC-097 dedup).
pub(crate) mod report;

// `optimize::WEB_DEFAULT_LONG_EDGE` was `pub` before the split; re-export it so
// `crustyimg::cli::WEB_DEFAULT_LONG_EDGE` keeps resolving.
pub use optimize::WEB_DEFAULT_LONG_EDGE;

use build::{run_build, run_build_watching};
use common::resolve_format;
use optimize::{
    resolve_effective_quality, run_apply, run_convert, run_optimize, run_responsive, run_web,
};
use report::{run_diff, run_info, run_lint, LintFlags};

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

    /// Number of parallel workers for batch operations.
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

    /// Bypass the `build` cache: read no entries, write none, rebuild every
    /// input (SPEC-064). The cache is on by default; this is the opt-out.
    #[arg(long, global = true)]
    pub no_cache: bool,

    /// Verify the build against the committed `crustyimg.build.lock` instead of
    /// refreshing it: exit 7 on drift, and never modify the lockfile (SPEC-066).
    ///
    /// `--frozen` and `--locked` are aliases — crustyimg has no network, so the
    /// cargo distinction between "don't update" and "don't hit the registry"
    /// collapses into one assert mode (DEC-059).
    #[arg(
        long,
        global = true,
        visible_alias = "frozen",
        visible_alias = "locked"
    )]
    pub check: bool,

    /// With `--check`: promote cross-environment output-byte variance from a
    /// note to a failure. For shops on a pinned toolchain and arch that want
    /// byte-identity enforced (DEC-059).
    #[arg(long, global = true)]
    pub strict: bool,

    /// `build` only: re-run the build whenever a source, recipe, or the manifest
    /// changes (SPEC-067). Debounced (an editor's save burst = one rebuild) and
    /// self-trigger-proof (the build's own outputs / `.crustyimg` / lockfile are
    /// ignored). Ctrl-C exits. Incompatible with `--check`/`--frozen`/`--locked`.
    /// A build cycle under `--watch` does not rewrite the committed lockfile.
    #[arg(long, global = true)]
    pub watch: bool,
}

/// A perceptual auto-quality preset for `optimize --target` (SPEC-016, DEC-019).
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

/// `optimize --profile`: the format auto-decision bias (SPEC-048, DEC-048).
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum ProfileArg {
    /// Auto-decide the output format, modern-format-first (the default).
    Web,
    /// Auto-decide with a crisp-text / lossless bias for documents.
    Docs,
    /// Engine off: reproduce today's format-preserving `optimize` exactly.
    Preserve,
}

impl ProfileArg {
    /// Map to the engine's pure [`crate::analysis::decide::Profile`].
    fn to_decide(self) -> crate::analysis::decide::Profile {
        use crate::analysis::decide::Profile;
        match self {
            ProfileArg::Web => Profile::Web,
            ProfileArg::Docs => Profile::Docs,
            ProfileArg::Preserve => Profile::Preserve,
        }
    }
}

/// `optimize --explain[=json]`: render the auto-decision trace (SPEC-049).
#[derive(Clone, Copy, Debug, PartialEq, Eq, clap::ValueEnum)]
pub enum ExplainFmt {
    /// Human-readable trace to stderr (the default for a bare `--explain`).
    Human,
    /// Machine-readable JSON to stdout.
    Json,
}

/// An opt-in auto-quality mode for `optimize`/`convert`: the encoder quality is
/// searched per output instead of fixed (SPEC-016 / SPEC-017). Both modes run
/// only for a format with a lossy quality knob (JPEG today; ignored otherwise,
/// DEC-019) — see [`LossyFormat::supports_lossy_quality`]. The search lives in
/// `crate::quality`.
#[derive(Debug, Clone)]
pub enum AutoQuality {
    /// The **default** `optimize` decision (SPEC-084): no search. Each candidate is
    /// encoded **once** at a fixed generous quality ([`crate::sink::FAST_LOSSY_QUALITY`]
    /// for lossy formats), AVIF is admitted for photographic content, and the
    /// smallest that beats the source wins. The perceptual/byte-budget searches
    /// below are the opt-in modes (`--target`/`--ssim`/`--max-size`).
    Fast,
    /// Lowest quality whose decoded round-trip scores ≥ the SSIMULACRA2 target
    /// (`--target`/`--ssim`, SPEC-016).
    Perceptual(SearchConfig),
    /// Highest quality whose encoded size ≤ the byte budget (`--max-size`,
    /// SPEC-017). The `u64` is the budget in bytes.
    SizeBudget(u64),
}

/// The full subcommand surface (see `docs/cli-reference.md`).
///
/// Each variant carries that command's documented positional and named args.
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Display an image in the terminal.
    View {
        input: String,
        #[arg(long)]
        width: Option<u32>,
        #[arg(long)]
        height: Option<u32>,
    },

    /// Print image info: dimensions, format, byte size, color type, EXIF/ICC.
    Info {
        input: String,
        #[arg(long)]
        exif: bool,
        #[arg(long)]
        json: bool,
    },

    /// Perceptual comparison: SSIMULACRA2 score of <b> vs <a>.
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

    /// Resize one or more images.
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

    /// Convenience resize to a small bounded size.
    Thumbnail {
        inputs: Vec<String>,
        #[arg(long)]
        size: Option<u32>,
        #[arg(long)]
        square: bool,
    },

    /// Re-encode to another core format.
    Convert {
        inputs: Vec<String>,
        /// Target format (required for this command).
        #[arg(long)]
        format: String,
        /// Auto-tune the JPEG quality to fit a byte budget, e.g. `200KB`
        /// (JPEG target only).
        #[arg(long, value_name = "SIZE")]
        max_size: Option<String>,
    },

    /// The flagship: make an image web-ready. Downscale the long edge to a
    /// web-friendly default (never upscaling), bake orientation + strip metadata,
    /// pick the smallest modern format that beats the downscaled image (AVIF for
    /// photos, lossless WebP/PNG for graphics), and report its SSIMULACRA2 score.
    /// The downscale to a dimension bound is the contract, so an already-small
    /// source above that bound can come back larger than the original — reported
    /// honestly ("N% larger", plus a `larger_than_source` flag in `--json`), never
    /// hidden. For an unconditional never-bigger guarantee that keeps dimensions,
    /// use `optimize`. Size-insensitive: a 24 MP photo finishes as fast as a small
    /// one because it downscales first.
    ///
    /// Equivalent to `apply --recipe web`. `--max` overrides the downscale bound;
    /// `-o`/`--format` pin the output format (bypassing the auto-decision); the
    /// global `--out-dir`/`--name-template`/`-j` drive batch output as elsewhere.
    Web {
        inputs: Vec<String>,
        /// Override the downscale long-edge bound (default 2048; never upscales).
        #[arg(long)]
        max: Option<u32>,
        /// Emit the machine-readable audit report (the `optimize.explain/v1`
        /// schema) to stdout — the `--json` shared across `optimize`/`web`/`apply`.
        #[arg(long)]
        json: bool,
        /// Report decode/encode/total timing per image (human to stderr; folded
        /// into `--json`).
        #[arg(long)]
        timing: bool,
    },

    /// One-button web-good: auto-orient + strip metadata + a fast fixed-quality
    /// re-encode (high quality by default), picking the smallest modern format that
    /// beats the source — and never shipping a larger file.
    /// `--target`/`--ssim` opt into a perceptual search; `--max-size` into a byte
    /// budget; `--max` optionally bounds the long edge; `-o`/`--format` pick the
    /// output format.
    Optimize {
        inputs: Vec<String>,
        /// Optional long-edge bound (no resize by default).
        #[arg(long)]
        max: Option<u32>,
        /// Opt into a perceptual quality search at a preset target (instead of the
        /// fast fixed-quality default).
        #[arg(long, value_enum)]
        target: Option<QualityTarget>,
        /// Override with a specific SSIMULACRA2 score (0-100).
        #[arg(long, conflicts_with = "target")]
        ssim: Option<f64>,
        /// Re-encode to fit a byte budget instead, e.g. `200KB`.
        #[arg(long, value_name = "SIZE", conflicts_with_all = ["target", "ssim"])]
        max_size: Option<String>,
        /// Format auto-decision bias: `web` (default) auto-picks the smallest
        /// format; `preserve` keeps the input format (today's behaviour).
        #[arg(long, value_enum, default_value_t = ProfileArg::Web)]
        profile: ProfileArg,
        /// Compute and report the winner's SSIMULACRA2 score for this run (off by
        /// default — the keep-dimensions default stays lean; `web` scores always).
        #[arg(long)]
        verify: bool,
        /// Explain the auto-decision: `--explain` (human, to stderr) or
        /// `--explain=json` (machine-readable, to stdout).
        #[arg(long, value_enum, num_args = 0..=1, default_missing_value = "human")]
        explain: Option<ExplainFmt>,
        /// Emit the machine-readable audit report to stdout — the `--json` shared
        /// across `optimize`/`web`/`apply` (equivalent to `--explain=json`).
        #[arg(long, conflicts_with = "explain")]
        json: bool,
        /// Report decode/encode/total timing per image (human to stderr; folded
        /// into `--json`).
        #[arg(long)]
        timing: bool,
    },

    /// Generate a responsive image set: width-scaled variants per format + a
    /// paste-ready <picture>/srcset snippet on stdout.
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

    /// Apply EXIF orientation to pixels, then clear the orientation tag.
    #[command(name = "auto-orient")]
    AutoOrient { inputs: Vec<String> },

    /// Overlay an image OR text watermark at a gravity anchor.
    ///
    /// Exactly one of `--image` or `--text` is required;
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

    /// Container-level metadata operations: `strip` (remove all), `clean --gps`
    /// (remove location), `copy` (graft one file's metadata onto another), `set`
    /// (write artist/copyright/description tags).
    ///
    /// A grouped surface so the top-level verbs read by *job*: `meta` owns the
    /// container-lane metadata ops, none of which re-decode pixels (SPEC-087,
    /// SPEC-089). `auto-orient` is NOT here — it bakes orientation into pixels,
    /// an image op, so it stays top-level (DEC-017).
    #[command(subcommand_required = true, arg_required_else_help = true)]
    Meta {
        #[command(subcommand)]
        command: MetaCommand,
    },

    /// One-shot multi-op on a single image; optionally saves the recipe.
    ///
    /// Op flags are applied in canonical order regardless of CLI position:
    /// `auto-orient` → `resize` → `invert`. At least one op flag is required.
    Edit {
        input: String,
        /// Normalize EXIF orientation to pixels, then clear the tag.
        #[arg(long)]
        auto_orient: bool,
        /// Resize to a max edge of N pixels (canonical order: after auto-orient,
        /// before invert).
        #[arg(long, value_name = "N")]
        resize_max: Option<u32>,
        /// Invert pixel colors.
        #[arg(long)]
        invert: bool,
        /// Write the op chain as a TOML recipe to FILE after a successful edit.
        #[arg(long)]
        save_recipe: Option<String>,
    },

    /// Run a saved recipe over one image or a batch.
    Apply {
        #[arg(long)]
        recipe: String,
        inputs: Vec<String>,
        /// Emit the machine-readable audit report to stdout for a recipe that ends
        /// in the terminal `optimize` step (the bundled web/gallery/product flows) —
        /// the `--json` shared across `optimize`/`web`/`apply`.
        #[arg(long)]
        json: bool,
        /// Report decode/encode/total timing per image (human to stderr; folded
        /// into `--json`). Requires a terminal-`optimize` recipe.
        #[arg(long)]
        timing: bool,
    },

    /// Run a declared build: every `[[target]]` in a build manifest (SPEC-063, DEC-057).
    ///
    /// FILE defaults to `./crustyimg.build.toml`. Each target binds sources (a
    /// glob / dir / path, or a list) to a recipe file, an output directory, and
    /// an optional name template. Unlike `apply`, `build` overwrites its own
    /// declared outputs without `--yes` — a build must be re-runnable.
    Build { file: Option<String> },

    /// Lint an image asset tree: report problems (each with a runnable fix) and
    /// a CI-native exit code. Read-only — never writes an image (SPEC-050, DEC-050).
    ///
    /// Resolves PATHS via the shared source fan-out (globs/dirs/files, non-images
    /// skipped), runs the default rule set per image, and exits `0` clean · `7`
    /// on any error-severity finding · `2` usage · `3` no inputs resolved.
    Lint {
        /// Image files, directories, or globs to lint (default: the current dir).
        paths: Vec<String>,
        /// Use this config file instead of auto-discovering `.crustyimg-lint.toml`.
        #[arg(long, value_name = "PATH")]
        config: Option<String>,
        /// Ignore any discovered config; use built-in defaults + CLI flags only.
        #[arg(long)]
        no_config: bool,
        /// Only run rules matching these ids/prefixes (repeatable; ruff-style).
        #[arg(long, value_name = "PREFIX")]
        select: Vec<String>,
        /// Exclude rules matching these ids/prefixes (repeatable; ruff-style).
        #[arg(long, value_name = "PREFIX")]
        ignore: Vec<String>,
        /// Fail (exit 7) when the number of `warn` findings exceeds N.
        #[arg(long, value_name = "N")]
        max_warnings: Option<usize>,
        /// Declared intended display width; enables `dims/oversized-dimensions`.
        #[arg(long, value_name = "W")]
        max_intended_width: Option<u32>,
        /// Savings gate for "could be smaller" rules, as `BYTES:PERCENT`
        /// (default `4096:10`).
        #[arg(long, value_name = "BYTES:PERCENT")]
        savings_threshold: Option<String>,
    },

    /// Generate a shell-completion script (bash, zsh, fish, powershell, elvish) to stdout.
    Completions { shell: clap_complete::Shell },
}

/// Subcommands of the `meta` group: the container-lane metadata operations
/// (SPEC-087). Each carries the same args its old top-level verb did and
/// dispatches to the identical handler — a pure surface move, no behavior change.
///
/// `arg_required_else_help` makes a bare `meta` print this group's help (listing
/// strip/clean/copy) instead of a terse "subcommand required" error.
#[derive(Subcommand, Debug)]
pub enum MetaCommand {
    /// Remove all metadata at the container level.
    Strip { inputs: Vec<String> },

    /// Remove only GPS/location metadata.
    Clean {
        inputs: Vec<String>,
        #[arg(long)]
        gps: bool,
    },

    /// Copy metadata from one image's container to another's.
    Copy {
        #[arg(long)]
        from: String,
        #[arg(long)]
        to: String,
    },

    /// Write specific EXIF tags; pixels untouched.
    Set {
        inputs: Vec<String>,
        #[arg(long)]
        artist: Option<String>,
        #[arg(long)]
        copyright: Option<String>,
        #[arg(long)]
        description: Option<String>,
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

    /// A build-manifest parse / version / invalid-target error (SPEC-063).
    /// Every variant is a manifest *content* error → exit 2 (usage).
    #[error(transparent)]
    Build(#[from] crate::build::BuildError),

    /// Build-manifest file I/O (including "no `crustyimg.build.toml` here").
    /// Names the path, since `build` discovers a default file (exit 3).
    #[error("could not read build manifest {path}: {source}")]
    BuildManifestIo {
        path: String,
        source: std::io::Error,
    },

    /// A build-lockfile parse / version / oversize error (SPEC-066, DEC-059).
    /// A lockfile *content* error → exit 2, like a malformed manifest.
    #[error(transparent)]
    Lock(#[from] crate::build::lock::LockError),

    /// The committed lockfile exists but could not be read (exit 3). A *missing*
    /// lockfile under `--check` is not this: it is drift → `CheckFailed`.
    #[error("could not read build lockfile {path}: {source}")]
    LockIo {
        path: String,
        source: std::io::Error,
    },

    /// The lockfile could not be written (exit 5 — an output write was refused,
    /// the same code the sink and the cache store use).
    #[error("could not write build lockfile {path}: {source}")]
    LockWrite {
        path: String,
        source: std::io::Error,
    },

    /// The build cache could not be opened or written (SPEC-064, DEC-058).
    /// Only `Cache::open` reaches here — a lookup degrades to a miss, and a
    /// failed `store` is warned about, not raised. Exit 5 (write refused);
    /// `--no-cache` is the way past it.
    #[error(transparent)]
    Cache(#[from] crate::build::cache::CacheError),

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

    /// A container-lane metadata error (`meta strip` / `meta clean --gps`, SPEC-026).
    /// `UnsupportedFormat` → exit 4; `Container`/`Exif` → exit 1 (DEC-029).
    #[error(transparent)]
    Metadata(#[from] crate::metadata::MetadataError),

    /// A `build --watch` setup failure: the OS filesystem watcher could not be
    /// created or a root could not be registered (SPEC-067, DEC-060). A generic
    /// runtime failure of the watch machinery → exit 1.
    #[error(transparent)]
    Watch(#[from] crate::build::watch::WatchError),

    /// A build's resolved targets would write two inputs to one output path
    /// (SPEC-065, DEC-057). A *config* error found before execution → exit 2,
    /// deliberately not the per-output partial-batch 6 (DEC-015).
    ///
    /// `output` may carry a literal `{ext}` — the output extension is only known
    /// after a decode (DEC-058), so the prepare-phase check leaves it unexpanded.
    ///
    /// Paths are quoted literally, not with `{:?}`: `Debug` escapes a Windows
    /// separator into `"a\\logo.png"`, which is not a path the user can act on.
    #[error(
        "output collision: \"{output}\" written by both \"{first}\" and \"{second}\" \
         — two inputs map to one output (disambiguate the name template, e.g. {{parent}}_{{stem}})"
    )]
    OutputCollision {
        /// The shared output path; `{ext}` is left unexpanded (pre-decode).
        output: String,
        /// The earlier of the two colliding sources.
        first: String,
        /// The later of the two colliding sources.
        second: String,
    },
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
            CliError::Image(ImageError::LimitsExceeded(_)) => 1,
            // A recognized format whose DECODER is feature-gated and off (HEIC
            // without `--features heic`, DEC-052) → 4, like the sink-side twin.
            CliError::Image(ImageError::CodecNotBuilt { .. }) => 4,
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
            // A malformed build manifest is a usage error, like a bad flag → 2.
            // (Reading the manifest file is CliError::BuildManifestIo → 3.)
            CliError::Build(_) => 2,
            // A non-injective source→output mapping is a manifest *config* error
            // caught before execution → 2, like a malformed manifest (SPEC-065).
            CliError::OutputCollision { .. } => 2,
            // A watcher setup failure is a generic runtime error → 1.
            CliError::Watch(_) => 1,
            CliError::BuildManifestIo { .. } => 3,
            // A malformed lockfile is a broken committed contract, not a failed
            // check → 2, like a malformed manifest (SPEC-066).
            CliError::Lock(_) => 2,
            CliError::LockIo { .. } => 3,
            CliError::LockWrite { .. } => 5,
            // The build cache could not be created/written → an output write was
            // refused → 5, the same code the sink's write failures use.
            CliError::Cache(_) => 5,
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
    // `--watch` is a GLOBAL clap flag, but only `build` has a rebuild loop to run.
    // On any other subcommand it used to be a silent no-op — the user asks to watch
    // and gets one quiet one-shot run instead. Reject it as a usage error (exit 2)
    // rather than ignoring it (SPEC-071 fix 4; `ergonomic-defaults`). Same shape as
    // the `--watch` × verify-mode guard in `run_build_watching`.
    if cli.global.watch && !matches!(cli.command, Commands::Build { .. }) {
        return Err(CliError::Usage(
            "--watch is only valid with `build`: there is no rebuild loop to run for \
             this subcommand"
                .to_owned(),
        ));
    }

    match &cli.command {
        Commands::Web {
            inputs,
            max,
            json,
            timing,
        } => run_web(inputs, *max, *json, *timing, &cli.global),
        Commands::Apply {
            recipe,
            inputs,
            json,
            timing,
        } => run_apply(recipe, inputs, *json, *timing, &cli.global),
        Commands::Build { file } => {
            if cli.global.watch {
                run_build_watching(file.as_deref(), &cli.global)
            } else {
                run_build(file.as_deref(), &cli.global)
            }
        }

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
            profile,
            verify,
            explain,
            json,
            timing,
        } => run_optimize(
            inputs,
            *max,
            *target,
            *ssim,
            max_size.as_deref(),
            *profile,
            *verify,
            *explain,
            *json,
            *timing,
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
        Commands::Meta { command } => match command {
            MetaCommand::Strip { inputs } => run_strip(inputs, &cli.global),
            MetaCommand::Clean { inputs, gps } => run_clean(inputs, *gps, &cli.global),
            MetaCommand::Copy { from, to } => run_copy_metadata(from, to, &cli.global),
            MetaCommand::Set {
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
        },
        Commands::Edit {
            input,
            auto_orient,
            resize_max,
            invert,
            save_recipe,
        } => run_edit(
            input,
            *auto_orient,
            *resize_max,
            *invert,
            save_recipe.as_deref(),
            &cli.global,
        ),
        Commands::Lint {
            paths,
            config,
            no_config,
            select,
            ignore,
            max_warnings,
            max_intended_width,
            savings_threshold,
        } => run_lint(
            paths,
            &LintFlags {
                config: config.as_deref(),
                no_config: *no_config,
                select,
                ignore,
                max_warnings: *max_warnings,
                max_intended_width: *max_intended_width,
                savings_threshold: savings_threshold.as_deref(),
            },
            &cli.global,
        ),
        Commands::Completions { shell } => run_completions(*shell),
    }
}

// ── Shell completions ─────────────────────────────────────────────────────────

/// Print a clap-generated completion script for `shell` to stdout (SPEC-040, DEC-039).
///
/// Reflects over the full `Cli` command tree so completions stay in sync with
/// the subcommand surface automatically. Takes no input path and touches no
/// file system — the user (or a packager) redirects the output.
fn run_completions(shell: clap_complete::Shell) -> Result<(), CliError> {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "crustyimg", &mut std::io::stdout());
    Ok(())
}

// ── Build --watch (SPEC-067, DEC-060) ────────────────────────────────────────

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

/// Wire `meta strip`: remove ALL container metadata via the container lane
/// (DEC-003). Format is preserved; no pixel re-encode (`metadata-not-via-pixel-encode`).
fn run_strip(inputs: &[String], global: &GlobalArgs) -> Result<(), CliError> {
    run_metadata_lane(inputs, global, crate::metadata::strip_all)
}

/// Wire `meta clean --gps`: remove ONLY GPS/location metadata via the container
/// lane. `--gps` is required in v1; `meta clean` without it is a usage error (exit
/// 2), leaving room for future selective flags.
fn run_clean(inputs: &[String], gps: bool, global: &GlobalArgs) -> Result<(), CliError> {
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
fn run_set(
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
fn run_edit(
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
            Commands::Apply { recipe, inputs, .. } => {
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
        assert_eq!(
            CliError::Image(ImageError::LimitsExceeded("big".into())).code(),
            1
        );
        // Decode-side CodecNotBuilt (HEIC without `--features heic`) → 4 (SPEC-062).
        assert_eq!(
            CliError::Image(ImageError::CodecNotBuilt {
                codec: "HEIC",
                feature: "heic"
            })
            .code(),
            4
        );

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

        // Build manifest: content errors → 2 (usage); reading the file → 3 (SPEC-063).
        assert_eq!(
            CliError::Build(crate::build::BuildError::UnsupportedVersion {
                found: 999,
                supported: crate::build::SUPPORTED_VERSION,
            })
            .code(),
            2
        );
        assert_eq!(
            CliError::Build(crate::build::BuildError::Parse("bad".into())).code(),
            2
        );
        assert_eq!(
            CliError::BuildManifestIo {
                path: "crustyimg.build.toml".into(),
                source: std::io::Error::new(std::io::ErrorKind::NotFound, "no manifest"),
            }
            .code(),
            3
        );

        // A `build --watch` watcher setup failure → 1 (generic runtime, SPEC-067).
        assert_eq!(
            CliError::Watch(crate::build::watch::WatchError::Watcher(
                "no backend".into()
            ))
            .code(),
            1
        );

        // The build cache: only `Cache::open` reaches the boundary → 5, the code
        // the sink's write failures use (SPEC-064; this assertion is SPEC-066's,
        // closing a gap the SPEC-064 build note wrongly believed closed).
        assert_eq!(
            CliError::Cache(crate::build::cache::CacheError::Open {
                path: ".crustyimg/cache".into(),
                source: std::io::Error::other("read-only"),
            })
            .code(),
            5
        );
        assert_eq!(
            CliError::Cache(crate::build::cache::CacheError::Io(std::io::Error::other(
                "full"
            )))
            .code(),
            5
        );

        // The build lockfile (SPEC-066): content error → 2, read → 3, write → 5.
        // A *missing* lockfile under `--check` is none of these — it is drift,
        // reported as CheckFailed → 7.
        assert_eq!(
            CliError::Lock(crate::build::lock::LockError::Parse("bad".into())).code(),
            2
        );
        assert_eq!(
            CliError::Lock(crate::build::lock::LockError::UnsupportedVersion {
                found: 999,
                supported: crate::build::lock::SUPPORTED_LOCK_VERSION,
            })
            .code(),
            2
        );
        assert_eq!(
            CliError::Lock(crate::build::lock::LockError::TooLarge {
                size: crate::build::lock::LOCK_MAX_BYTES + 1,
                max: crate::build::lock::LOCK_MAX_BYTES,
            })
            .code(),
            2
        );
        assert_eq!(
            CliError::Lock(crate::build::lock::LockError::Serialize("no".into())).code(),
            2
        );
        assert_eq!(
            CliError::LockIo {
                path: "crustyimg.build.lock".into(),
                source: std::io::Error::other("unreadable"),
            }
            .code(),
            3
        );
        assert_eq!(
            CliError::LockWrite {
                path: "crustyimg.build.lock".into(),
                source: std::io::Error::other("read-only"),
            }
            .code(),
            5
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

        // A non-injective build is a config error → 2, not partial-batch 6 (SPEC-065).
        assert_eq!(
            CliError::OutputCollision {
                output: "dist/logo.{ext}".into(),
                first: "a/logo.png".into(),
                second: "b/logo.png".into(),
            }
            .code(),
            2
        );

        // Metadata variants (SPEC-026, DEC-029): unsupported format → 4; a
        // container/EXIF parse or rewrite failure → 1 (generic runtime).
        assert_eq!(
            CliError::Metadata(crate::metadata::MetadataError::UnsupportedFormat(
                "x".into()
            ))
            .code(),
            4
        );
        assert_eq!(
            CliError::Metadata(crate::metadata::MetadataError::Container(
                "bad chunk".into()
            ))
            .code(),
            1
        );
        assert_eq!(
            CliError::Metadata(crate::metadata::MetadataError::Exif("bad tag".into())).code(),
            1
        );
    }

    // ── SPEC-065: injective source→output ────────────────────────────────────

    #[test]
    fn output_collision_maps_to_exit_2() {
        let err = CliError::OutputCollision {
            output: "dist/logo.{ext}".into(),
            first: "a/logo.png".into(),
            second: "b/logo.png".into(),
        };
        assert_eq!(err.code(), 2);

        // The message names the shared output and both sources, and points at the fix.
        let msg = err.to_string();
        assert!(msg.contains("dist/logo.{ext}"), "{msg}");
        assert!(
            msg.contains("a/logo.png") && msg.contains("b/logo.png"),
            "{msg}"
        );
        assert!(msg.contains("{parent}_{stem}"), "{msg}");

        // Paths are quoted literally. `{:?}` would render a Windows separator as
        // `a\\logo.png` — not a path the user can copy back into the manifest.
        let windows = CliError::OutputCollision {
            output: r"dist\logo.{ext}".into(),
            first: r"a\logo.png".into(),
            second: r"b\logo.png".into(),
        };
        let msg = windows.to_string();
        assert!(msg.contains(r#""a\logo.png""#), "{msg}");
        assert!(
            !msg.contains(r"a\\logo.png"),
            "must not double-escape: {msg}"
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

    // ── SPEC-033 exit-code mapping for LimitsExceeded ────────────────────────

    /// A `LimitsExceeded` image error must map to exit code 1 (generic runtime
    /// error, DEC-034 / DEC-007 — same as an ordinary decode failure, distinct
    /// from format-not-found exit 4 and file-not-found exit 3).
    #[test]
    fn limits_exceeded_maps_to_exit_1() {
        assert_eq!(
            CliError::Image(ImageError::LimitsExceeded("x".into())).code(),
            1
        );
    }
}
