//! Helpers shared across ≥2 `cli` submodules: the encode/write seam
//! (`build`/`apply` both replay a recipe onto bytes), recipe resolution, the
//! output-sink builder, and small formatting helpers. Split out of
//! `cli/mod.rs` (SPEC-097) — no behavior change.

use std::path::{Path, PathBuf};

use crate::image::Image;
use crate::operation::OperationRegistry;
use crate::recipe::{Recipe, RecipeError};
use crate::sink::{Overwrite, Sink, SinkInput};

use super::{CliError, GlobalArgs};

/// A known-valid `ProgressStyle` template for the batch progress bar.
///
/// Kept as a const so we can use `.unwrap_or_else(|_| ProgressStyle::default_bar())`
/// in non-test code rather than an `unwrap` on an arbitrary user-supplied string.
pub(super) const BATCH_PROGRESS_TEMPLATE: &str = "{bar:40.cyan/blue} {pos}/{len} {msg}";

/// The decode→pipeline→encode half of [`apply_one`]: everything up to, but not
/// including, the write. Returns the output's extension and its encoded bytes.
///
/// This is the *worker* both callers share. `apply_one` writes its result
/// straight through; `run_build`'s cache-miss path writes it AND stores it under
/// the input's cache key (SPEC-064). Extracting it means the cached and
/// uncached paths cannot drift into producing different bytes.
///
/// Rebuilds the pipeline from `recipe` + `registry` on every call — `Operation`
/// is NOT `Send`, so no pipeline may cross a thread boundary (SPEC-031).
/// The output format preserves the source format (no `--format` in the batch
/// path, DEC-015), which is why the extension is a *result* rather than an input.
pub(super) fn encode_one(
    recipe: &Recipe,
    registry: &OperationRegistry,
    input: &crate::source::Input,
    quality: Option<u8>,
) -> Result<(&'static str, Vec<u8>), CliError> {
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
    let bytes = crate::sink::encode_to_bytes(&out_img, fmt, quality)?;

    Ok((crate::sink::extension_for_format(fmt), bytes))
}

/// Apply one input through the recipe and write the result to `out_dir`.
///
/// Extracted from `run_apply` so it is unit-testable. [`encode_one`] does the
/// decode→pipeline→encode; this adds the `Sink::Dir` write, which is where the
/// name-template expansion, traversal, symlink, and overwrite guards live.
pub(super) fn apply_one(
    recipe: &Recipe,
    registry: &OperationRegistry,
    input: &crate::source::Input,
    out_dir: &Path,
    template: &str,
    overwrite: Overwrite,
    quality: Option<u8>,
) -> Result<(), CliError> {
    let (ext, bytes) = encode_one(recipe, registry, input, quality)?;
    write_encoded(&bytes, ext, input, out_dir, template, overwrite)
}

/// Write already-encoded output `bytes` into `out_dir` under `template`.
///
/// The single write seam for the batch paths: `apply_one` hands it freshly
/// encoded bytes, `run_build`'s cache-hit path hands it bytes read from the
/// store. Both inherit the sink's create-dir, traversal, symlink, and overwrite
/// guards — a cached byte reaches disk through exactly the guards a fresh one does.
pub(super) fn write_encoded(
    bytes: &[u8],
    ext: &str,
    input: &crate::source::Input,
    out_dir: &Path,
    template: &str,
    overwrite: Overwrite,
) -> Result<(), CliError> {
    // `format` is unused by `write_bytes` (the extension is passed explicitly).
    let sink = Sink::Dir {
        dir: out_dir.to_owned(),
        template: template.to_owned(),
        format: None,
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

/// Guard: multi-input without `--out-dir` is a usage error (exit 2).
///
/// Returns `Ok(dir_path)` when `global.out_dir` is `Some`, else `CliError::Usage`.
pub(super) fn require_out_dir_for_batch(global: &GlobalArgs) -> Result<&str, CliError> {
    global
        .out_dir
        .as_deref()
        .ok_or_else(|| CliError::Usage("multiple inputs require --out-dir".into()))
}

/// Resolve `--recipe <arg>` to a [`Recipe`]: a file path OR a bundled name (SPEC-085).
///
/// **Precedence — a real file on disk ALWAYS wins.** `<arg>` is treated as a path
/// first; only when no such file exists does it fall back to the bundled registry
/// (`web`/`gallery`/`product`). So a local `web.toml` (or a file literally named
/// `web`) unambiguously shadows the bundled `web`, and every existing file-path
/// recipe keeps working exactly as before.
///
/// The on-disk size is checked via `std::fs::metadata` BEFORE reading, so a multi-GB
/// "recipe" is never loaded into memory (DEC-036, SPEC-035). Bundled recipes are
/// trusted compile-time strings — no size guard needed. A missing file AND unknown
/// name is `RecipeIo` (exit 3); bad content is `Recipe` (exit 1). Shared by
/// `run_apply` and `run_build` (one recipe per target).
pub(super) fn load_recipe(recipe_arg: &str) -> Result<Recipe, CliError> {
    if Path::new(recipe_arg).is_file() {
        let meta = std::fs::metadata(recipe_arg).map_err(CliError::RecipeIo)?;
        if meta.len() > crate::recipe::RECIPE_MAX_BYTES as u64 {
            return Err(CliError::Recipe(RecipeError::TooLarge {
                size: meta.len() as usize,
                max: crate::recipe::RECIPE_MAX_BYTES,
            }));
        }
        let recipe_text = std::fs::read_to_string(recipe_arg).map_err(CliError::RecipeIo)?;
        return Ok(Recipe::from_toml(&recipe_text)?);
    }

    // Not a file: try the bundled registry by name.
    if let Some(text) = crate::recipe::bundled::resolve(recipe_arg) {
        return Ok(Recipe::from_toml(text)?);
    }

    // Neither a readable file nor a known bundled name → not found (exit 3); name both
    // what we looked for and the bundled recipes available.
    Err(CliError::RecipeIo(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!(
            "no recipe file '{recipe_arg}' and no bundled recipe by that name \
             (bundled: {})",
            crate::recipe::bundled::names().join(", ")
        ),
    )))
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
pub(super) fn build_sink(global: &GlobalArgs) -> Result<Sink, CliError> {
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
pub(super) fn resolve_format(fmt: Option<&str>) -> Result<Option<::image::ImageFormat>, CliError> {
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

/// Render a byte count as a short human string, e.g. `512 B`, `6.0 KB`, `1.5 MB`
/// (decimal units, matching `parse_size`). Used in the `--max-size` warnings.
pub(super) fn fmt_bytes(n: u64) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_bytes_renders_units() {
        assert_eq!(fmt_bytes(512), "512 B");
        assert_eq!(fmt_bytes(6_000), "6.0 KB");
        assert_eq!(fmt_bytes(1_500_000), "1.5 MB");
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
            no_cache: false,
            check: false,
            strict: false,
            watch: false,
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
}
