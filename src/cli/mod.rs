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

use std::process::ExitCode;

use clap::{Args, CommandFactory, Parser, Subcommand};
use clap_complete::generate;

use crate::error::ImageError;
use crate::operation::OperationError;
use crate::quality::{QualityError, SearchConfig};
use crate::recipe::RecipeError;
use crate::sink::SinkError;
use crate::source::SourceError;

mod build;
mod common;
mod ops;
mod optimize;
// `pub(crate)` (not private): `lint::report` reuses `report::escape_json`
// across the `cli`/`lint` module boundary (SPEC-097 dedup).
pub(crate) mod report;

// `optimize::WEB_DEFAULT_LONG_EDGE` was `pub` before the split; re-export it so
// `crustyimg::cli::WEB_DEFAULT_LONG_EDGE` keeps resolving.
pub use optimize::WEB_DEFAULT_LONG_EDGE;

use build::{run_build, run_build_watching};
use ops::{
    run_auto_orient, run_clean, run_copy_metadata, run_edit, run_resize, run_set, run_strip,
    run_thumbnail, run_view, run_watermark, ResizeModes, WatermarkSource,
};
use optimize::{run_apply, run_convert, run_optimize, run_responsive, run_web};
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
