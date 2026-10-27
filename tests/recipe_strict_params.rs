//! Recipe step params are checked when the recipe is loaded.
//!
//! A step key the op does not take, a value of the wrong TOML type, a value
//! out of range (including `nan`/`inf`), or a key the step's mode never reads
//! is refused before any input is touched, with one error naming the step
//! index, the op and the key. Recipes that were valid before keep loading.
//!
//! The library tables go through `Recipe::from_toml` + `build_pipeline` and
//! assert on the error's `reason` (never on its Display), so each table tests
//! one rule. The CLI tests drive the real binary, as `tests/apply_batch.rs`
//! does, because "nothing was written" is an observable of the process.

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::Command;

use image::{DynamicImage, ImageFormat, RgbImage, RgbaImage};
use tempfile::TempDir;

use crustyimg::operation::{
    AutoOrient, Gravity, Identity, Invert, Operation, OperationParams, OperationRegistry, Watermark,
};
use crustyimg::pipeline::Pipeline;
use crustyimg::recipe::{Recipe, RecipeError};

/// Path to the compiled binary, provided by Cargo.
const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

// ── Library helpers ─────────────────────────────────────────────────────────

/// A tiny decodable PNG standing in for a watermark `image` file.
fn tiny_png() -> Vec<u8> {
    let mut out = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(4, 4, image::Rgba([9, 9, 9, 255])))
        .write_to(&mut out, ImageFormat::Png)
        .expect("encode tiny png");
    out.into_inner()
}

/// Attach stand-in bytes for every string-valued asset key, as the CLI's
/// resolver does before building the pipeline.
fn attach_assets(recipe: &mut Recipe) {
    let png = tiny_png();
    for step in &mut recipe.steps {
        if step.params.get_str("image").is_some() {
            step.params.set_resolved_bytes("image", png.clone());
        }
        if step.params.get_str("font").is_some() {
            step.params
                .set_resolved_bytes("font", crustyimg::text::DEFAULT_FONT.to_vec());
        }
    }
}

/// Parse and build a recipe the way `apply` does (assets resolved first).
fn load(toml: &str) -> Result<Pipeline, RecipeError> {
    let mut recipe = Recipe::from_toml(toml)?;
    attach_assets(&mut recipe);
    recipe.build_pipeline(&OperationRegistry::with_builtins())
}

/// A one-step version-1 recipe: `op` plus the step body lines.
fn one_step(op: &str, body: &str) -> String {
    format!("version = \"1\"\n\n[[step]]\nop = \"{op}\"\n{body}\n")
}

/// The refusal for `toml`: `(name, reason)`. Panics if it loads, or if it
/// fails any other way than a refused step.
fn refusal(toml: &str) -> (String, String) {
    match load(toml) {
        Ok(_) => panic!("expected the recipe to be refused, but it loaded:\n{toml}"),
        Err(RecipeError::InvalidOperation { name, reason, .. }) => (name, reason),
        Err(other) => panic!("expected a refused step, got {other:?} for:\n{toml}"),
    }
}

/// The step index an `InvalidOperation` carries, read from its Debug form so
/// the assertion is about the field, not about the Display wording.
fn step_index(err: &RecipeError) -> Option<usize> {
    let debug = format!("{err:?}");
    let rest = debug.split("step: ").nth(1)?;
    rest.split(|c: char| !c.is_ascii_digit())
        .next()?
        .parse()
        .ok()
}

const RESIZE_KEYS: &str = "expected one of: mode, width, height, percent";

// ── AC-1: unknown key, every op ─────────────────────────────────────────────

#[test]
fn unknown_key_is_refused_on_every_op() {
    // (op, body, unknown key, the phrase that must follow it)
    let rows: &[(&str, &str, &str, &str)] = &[
        (
            "resize",
            "mode = \"max\"\nwidth = 8\nbogus = 1",
            "bogus",
            RESIZE_KEYS,
        ),
        (
            "watermark",
            "text = \"hi\"\nbogus = 1",
            "bogus",
            "expected one of: image, text, font, size, color, gravity, opacity, scale, margin, tile",
        ),
        ("identity", "bogus = 1", "bogus", "'identity' takes no keys"),
        ("invert", "bogus = 1", "bogus", "'invert' takes no keys"),
        (
            "auto-orient",
            "bogus = 1",
            "bogus",
            "'auto-orient' takes no keys",
        ),
        // A misspelt required key is reported as unknown, not as a missing `width`.
        ("resize", "mode = \"max\"\nwidht = 100", "widht", RESIZE_KEYS),
    ];
    assert_eq!(rows.len(), 6, "5 ops + the misspelt-required-key row");

    for (op, body, key, tail) in rows {
        let (name, reason) = refusal(&one_step(op, body));
        assert_eq!(&name, op);
        assert!(
            reason.contains(&format!("unknown key '{key}'")) && reason.contains(tail),
            "{op}: reason must name '{key}' and say `{tail}`, got: {reason}"
        );
    }
}

// ── AC-2: the terminal `optimize` marker takes no keys ──────────────────────

#[test]
fn terminal_optimize_marker_with_a_key_is_refused() {
    let toml = "version = \"1\"\n\n[[step]]\nop = \"auto-orient\"\n\n[[step]]\nop = \"resize\"\n\
                mode = \"max\"\nwidth = 8\n\n[[step]]\nop = \"optimize\"\nquality = \"max\"\n";
    let err = Recipe::from_toml(toml).expect_err("a keyed terminal optimize must be refused");
    match &err {
        RecipeError::InvalidOperation { name, reason, .. } => {
            assert_eq!(name, "optimize");
            assert!(
                reason.contains("'quality'"),
                "reason must name the key, got: {reason}"
            );
        }
        other => panic!("expected InvalidOperation, got {other:?}"),
    }
    assert_eq!(step_index(&err), Some(2), "the marker is step 2: {err:?}");
}

/// The marker check must also run on a recipe whose only step is the marker.
#[test]
fn a_one_step_recipe_with_a_keyed_optimize_marker_is_refused() {
    let toml = "version = \"1\"\n\n[[step]]\nop = \"optimize\"\nquality = \"max\"\n";
    let err = Recipe::from_toml(toml).expect_err("a keyed lone optimize must be refused");
    match &err {
        RecipeError::InvalidOperation { name, reason, .. } => {
            assert_eq!(name, "optimize");
            assert!(
                reason.contains("'quality'"),
                "reason must name the key, got: {reason}"
            );
        }
        other => panic!("expected InvalidOperation, got {other:?}"),
    }
    assert_eq!(step_index(&err), Some(0), "the marker is step 0: {err:?}");
}

#[test]
fn keyless_terminal_optimize_marker_still_parses() {
    let bundled = [
        include_str!("../recipes/web.toml"),
        include_str!("../recipes/gallery.toml"),
        include_str!("../recipes/product.toml"),
    ];
    for toml in bundled {
        let recipe = Recipe::from_toml(toml).expect("a bundled recipe must parse");
        assert_eq!(recipe.steps.last().map(|s| s.op.as_str()), Some("optimize"));
    }
}

// ── AC-3: wrong type, every key ─────────────────────────────────────────────

#[test]
fn wrong_type_is_refused_for_every_key() {
    // (op, body, key) — the body gives `key` a value of the wrong TOML type and
    // is otherwise a valid step.
    let rows: &[(&str, &str, &str)] = &[
        ("resize", "mode = 5\nwidth = 8", "mode"),
        ("resize", "mode = \"max\"\nwidth = 100.0", "width"),
        (
            "resize",
            "mode = \"exact\"\nwidth = 8\nheight = \"8\"",
            "height",
        ),
        ("resize", "mode = \"percent\"\npercent = \"50\"", "percent"),
        ("watermark", "image = 5", "image"),
        ("watermark", "text = 5", "text"),
        ("watermark", "text = \"hi\"\nfont = 5", "font"),
        ("watermark", "text = \"hi\"\nsize = \"big\"", "size"),
        ("watermark", "text = \"hi\"\ncolor = 255", "color"),
        ("watermark", "text = \"hi\"\ngravity = 3", "gravity"),
        ("watermark", "text = \"hi\"\nopacity = \"0.5\"", "opacity"),
        ("watermark", "text = \"hi\"\nscale = true", "scale"),
        ("watermark", "text = \"hi\"\nmargin = 5.0", "margin"),
        ("watermark", "text = \"hi\"\ntile = \"yes\"", "tile"),
    ];
    assert_eq!(rows.len(), 14, "one row per step key");

    for (op, body, key) in rows {
        let (name, reason) = refusal(&one_step(op, body));
        assert_eq!(&name, op);
        assert!(
            reason.contains(&format!("'{key}' must be")),
            "{op}.{key}: reason must say `'{key}' must be`, got: {reason}"
        );
    }
}

#[test]
fn integer_for_a_number_key_is_accepted() {
    let rows = [
        one_step("watermark", "text = \"hi\"\nopacity = 1"),
        one_step("watermark", "text = \"hi\"\nsize = 24"),
        one_step("resize", "mode = \"percent\"\npercent = 50"),
    ];
    assert_eq!(rows.len(), 3);
    for toml in &rows {
        load(toml).unwrap_or_else(|e| panic!("must build, got {e}:\n{toml}"));
    }
}

// ── AC-4: out of range, every ranged key ────────────────────────────────────

#[test]
fn out_of_range_is_refused_for_every_ranged_key() {
    let rows: &[(&str, &str, &str)] = &[
        ("resize", "mode = \"max\"\nwidth = 0", "width"),
        (
            "resize",
            "mode = \"exact\"\nwidth = 8\nheight = 0",
            "height",
        ),
        ("resize", "mode = \"percent\"\npercent = 0", "percent"),
        ("resize", "mode = \"percent\"\npercent = nan", "percent"),
        ("resize", "mode = \"percent\"\npercent = inf", "percent"),
        ("watermark", "text = \"hi\"\nsize = 0", "size"),
        ("watermark", "text = \"hi\"\nsize = nan", "size"),
        ("watermark", "text = \"hi\"\nsize = inf", "size"),
        ("watermark", "text = \"hi\"\nscale = 0", "scale"),
        ("watermark", "text = \"hi\"\nscale = nan", "scale"),
        ("watermark", "text = \"hi\"\nopacity = 1.5", "opacity"),
        ("watermark", "text = \"hi\"\nopacity = -0.1", "opacity"),
        ("watermark", "text = \"hi\"\nopacity = nan", "opacity"),
        ("watermark", "text = \"hi\"\nmargin = -5", "margin"),
        ("resize", "mode = \"big\"\nwidth = 8", "mode"),
        ("watermark", "text = \"hi\"\ngravity = \"up\"", "gravity"),
        ("watermark", "text = \"hi\"\ncolor = \"xyz\"", "color"),
    ];
    assert_eq!(rows.len(), 17, "every ranged key, non-finite rows included");

    for (op, body, key) in rows {
        let (name, reason) = refusal(&one_step(op, body));
        assert_eq!(&name, op);
        assert!(
            reason.contains(&format!("'{key}' must be")),
            "{op}: `{body}` must be refused naming '{key}', got: {reason}"
        );
    }
}

/// A TOML integer is 64-bit; an integer key is 32-bit unsigned. A value past
/// `u32::MAX` must be refused, not wrapped or dropped.
#[test]
fn an_integer_past_u32_max_is_refused() {
    let rows: &[(&str, &str, &str)] = &[
        ("resize", "mode = \"max\"\nwidth = 4294967296", "width"),
        ("watermark", "text = \"hi\"\nmargin = 4294967296", "margin"),
    ];
    assert_eq!(rows.len(), 2);
    for (op, body, key) in rows {
        let (_, reason) = refusal(&one_step(op, body));
        assert!(
            reason.contains(&format!("'{key}' must be an integer")),
            "{op}: got: {reason}"
        );
    }
}

/// Number keys are read as 32-bit floats. A finite TOML float past `f32::MAX`
/// would become `inf` there, so it is refused like `inf` itself.
#[test]
fn a_number_past_f32_range_is_refused() {
    let rows: &[(&str, &str, &str)] = &[
        ("watermark", "text = \"hi\"\nsize = 1e300", "size"),
        ("resize", "mode = \"percent\"\npercent = 1e300", "percent"),
    ];
    assert_eq!(rows.len(), 2);
    for (op, body, key) in rows {
        let (_, reason) = refusal(&one_step(op, body));
        assert!(
            reason.contains(&format!("'{key}' must be a finite number")),
            "{op}: got: {reason}"
        );
    }
}

/// A tiny positive float passes `> 0` as an `f64` but reads as `0.0` as the
/// `f32` the ops use, so the range is checked on the value as it is read.
#[test]
fn a_positive_number_that_reads_as_zero_is_refused() {
    let rows: &[(&str, &str, &str)] = &[
        ("resize", "mode = \"percent\"\npercent = 1e-50", "percent"),
        ("watermark", "text = \"hi\"\nsize = 1e-50", "size"),
        ("watermark", "text = \"hi\"\nscale = 1e-50", "scale"),
    ];
    assert_eq!(rows.len(), 3);
    // Collect every failing row so one run shows which of the three load.
    let mut failures = Vec::new();
    for (op, body, key) in rows {
        match load(&one_step(op, body)) {
            Ok(_) => failures.push(format!("{op} {key}: the recipe loaded")),
            Err(RecipeError::InvalidOperation { reason, .. })
                if reason.contains(&format!("'{key}' must be")) && reason.contains("got 1e-50") => {
            }
            Err(other) => failures.push(format!("{op} {key}: wrong refusal: {other:?}")),
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}

// ── AC-5: a key the step's mode never reads ─────────────────────────────────

#[test]
fn key_unused_by_the_mode_is_refused() {
    // (op, body, key, phrase)
    let rows: &[(&str, &str, &str, &str)] = &[
        (
            "resize",
            "mode = \"max\"\nwidth = 8\nheight = 8",
            "height",
            "is not used by mode 'max'",
        ),
        (
            "resize",
            "mode = \"percent\"\npercent = 50\nwidth = 10",
            "width",
            "is not used by mode 'percent'",
        ),
        (
            "resize",
            "mode = \"exact\"\nwidth = 8\nheight = 8\npercent = 50",
            "percent",
            "is not used by mode 'exact'",
        ),
        (
            "watermark",
            "image = \"logo.png\"\nsize = 64.0",
            "size",
            "only applies to a text watermark",
        ),
        (
            "watermark",
            "image = \"logo.png\"\ncolor = \"ff0000\"",
            "color",
            "only applies to a text watermark",
        ),
        (
            "watermark",
            "image = \"logo.png\"\nfont = \"f.ttf\"",
            "font",
            "only applies to a text watermark",
        ),
    ];
    assert_eq!(rows.len(), 6, "3 resize modes + 3 text-only keys");

    for (op, body, key, phrase) in rows {
        let (name, reason) = refusal(&one_step(op, body));
        assert_eq!(&name, op);
        assert!(
            reason.contains(&format!("'{key}' {phrase}")),
            "{op}: `{body}` must say `'{key}' {phrase}`, got: {reason}"
        );
    }
}

#[test]
fn documented_interplay_is_not_refused() {
    let rows = [
        // `tile` ignores gravity and margin, as the CLI verb documents.
        one_step(
            "watermark",
            "image = \"logo.png\"\ntile = true\ngravity = \"north\"\nmargin = 5",
        ),
        // `center` ignores margin.
        one_step(
            "watermark",
            "text = \"hi\"\ngravity = \"center\"\nmargin = 5",
        ),
    ];
    assert_eq!(rows.len(), 2);
    for toml in &rows {
        load(toml).unwrap_or_else(|e| panic!("must build, got {e}:\n{toml}"));
    }
}

// ── AC-6: the error names the step index, op and key ────────────────────────

#[test]
fn error_display_names_step_op_and_key() {
    let toml = "version = \"1\"\n\n[[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 64\n\n\
                [[step]]\nop = \"resize\"\nmode = \"max\"\nwidth = 32\nbogus = 1\n";
    let err = match load(toml) {
        Ok(_) => panic!("step 1's unknown key must be refused"),
        Err(e) => e,
    };
    assert_eq!(
        err.to_string(),
        "recipe step 1 ('resize'): unknown key 'bogus'; expected one of: mode, width, height, percent"
    );
}

// ── AC-9: versions 1 and 2 are validated identically ────────────────────────

#[test]
fn versions_1_and_2_validate_identically() {
    let body = "[[step]]\nop = \"watermark\"\ntext = \"hi\"\nopactiy = 0.5\n";
    let errs: Vec<RecipeError> = ["1", "2"]
        .iter()
        .map(|v| match load(&format!("version = \"{v}\"\n\n{body}")) {
            Ok(_) => panic!("version {v}: the misspelt key must be refused"),
            Err(e) => e,
        })
        .collect();
    assert_eq!(errs.len(), 2);
    match &errs[0] {
        RecipeError::InvalidOperation { reason, .. } => {
            assert!(reason.contains("unknown key 'opactiy'"), "got: {reason}")
        }
        other => panic!("expected InvalidOperation, got {other:?}"),
    }
    // Debug carries every field (step, name, reason).
    assert_eq!(format!("{:?}", errs[0]), format!("{:?}", errs[1]));
}

// ── AC-11(b): a recipe crustyimg itself writes is never refused ─────────────

fn overlay() -> DynamicImage {
    DynamicImage::ImageRgba8(RgbaImage::from_pixel(4, 4, image::Rgba([1, 2, 3, 255])))
}

fn resize_op(registry: &OperationRegistry, pairs: &[(&str, toml::Value)]) -> Box<dyn Operation> {
    let map = pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), v.clone()))
        .collect();
    registry
        .build("resize", &OperationParams::from_map(map))
        .expect("a CLI-shaped resize must build")
}

#[test]
fn recipes_crustyimg_writes_are_never_refused() {
    use toml::Value::{Float, Integer, String as S};
    let registry = OperationRegistry::with_builtins();
    let mut ops: Vec<Box<dyn Operation>> = Vec::new();

    ops.push(resize_op(
        &registry,
        &[("mode", S("max".into())), ("width", Integer(64))],
    ));
    for mode in ["exact", "fit", "fill", "cover"] {
        ops.push(resize_op(
            &registry,
            &[
                ("mode", S(mode.into())),
                ("width", Integer(64)),
                ("height", Integer(48)),
            ],
        ));
    }
    ops.push(resize_op(
        &registry,
        &[("mode", S("percent".into())), ("percent", Float(37.5))],
    ));
    let resize_count = ops.len();
    assert_eq!(resize_count, 6, "all six resize modes");

    // Image mode, with and without `scale`; centered with a margin, as the CLI writes it.
    ops.push(Box::new(Watermark::new_image(
        overlay(),
        "logo.png".into(),
        Gravity::SouthEast,
        0.5,
        None,
        8,
        false,
    )));
    ops.push(Box::new(Watermark::new_image(
        overlay(),
        "logo.png".into(),
        Gravity::Center,
        1.0,
        Some(0.25),
        0,
        true,
    )));
    // Text mode, with and without `font` (a text mark never has `scale`).
    ops.push(Box::new(Watermark::new_text(
        overlay(),
        "hi".into(),
        None,
        32.0,
        [255, 255, 255, 255],
        Gravity::NorthWest,
        1.0,
        None,
        0,
        false,
    )));
    ops.push(Box::new(Watermark::new_text(
        overlay(),
        "hi".into(),
        Some("font.ttf".into()),
        24.5,
        [255, 0, 0, 128],
        Gravity::Center,
        0.75,
        None,
        4,
        false,
    )));
    ops.push(Box::new(Identity));
    ops.push(Box::new(Invert));
    ops.push(Box::new(AutoOrient));
    assert_eq!(ops.len(), 13, "6 resize + 4 watermark + 3 paramless");

    let toml = Recipe::from_ops(&ops).to_toml().expect("to_toml");
    load(&toml).unwrap_or_else(|e| panic!("crustyimg's own recipe was refused: {e}\n{toml}"));
}

// ── CLI helpers ─────────────────────────────────────────────────────────────

fn write_png(dir: &Path, name: &str) -> PathBuf {
    let img = RgbImage::from_pixel(24, 16, image::Rgb([42, 100, 200]));
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    DynamicImage::ImageRgb8(img).save(&path).unwrap();
    path
}

fn files_in(dir: &Path) -> usize {
    match std::fs::read_dir(dir) {
        Ok(entries) => entries.count(),
        Err(_) => 0,
    }
}

fn error_lines(stderr: &str) -> Vec<&str> {
    stderr.lines().filter(|l| l.starts_with("error:")).collect()
}

// ── AC-7: `apply --recipe` refuses before writing anything ──────────────────

#[test]
fn apply_single_input_refuses_a_wrong_typed_watermark_key() {
    let dir = TempDir::new().unwrap();
    let input = write_png(dir.path(), "in.png");
    let recipe = dir.path().join("r.toml");
    std::fs::write(
        &recipe,
        one_step("watermark", "text = \"hi\"\nopacity = \"0.5\""),
    )
    .unwrap();
    let out = dir.path().join("out.png");

    let output = Command::new(BIN)
        .arg("apply")
        .arg("--recipe")
        .arg(&recipe)
        .arg(&input)
        .arg("-o")
        .arg(&out)
        .arg("-y")
        .output()
        .expect("run apply");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert!(!out.exists(), "no output may be written");
    let errors = error_lines(&stderr);
    assert_eq!(errors.len(), 1, "one error line, got: {stderr}");
    assert!(
        errors[0].contains("recipe step 0 ('watermark')") && errors[0].contains("'opacity'"),
        "the error must name the step, op and key, got: {stderr}"
    );
}

#[test]
fn apply_batch_refuses_an_unknown_key_before_any_output() {
    let dir = TempDir::new().unwrap();
    let inputs: Vec<PathBuf> = ["a.png", "b.png", "c.png"]
        .iter()
        .map(|n| write_png(dir.path(), n))
        .collect();
    assert_eq!(inputs.len(), 3);
    let recipe = dir.path().join("r.toml");
    std::fs::write(
        &recipe,
        one_step("resize", "mode = \"max\"\nwidth = 8\nbogus = \"x\""),
    )
    .unwrap();
    let out_dir = dir.path().join("out");
    std::fs::create_dir_all(&out_dir).unwrap();

    let output = Command::new(BIN)
        .arg("apply")
        .arg("--recipe")
        .arg(&recipe)
        .args(&inputs)
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("-y")
        .output()
        .expect("run apply");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert_eq!(files_in(&out_dir), 0, "no output may be written");
    assert!(
        stderr.contains("recipe step 0 ('resize')") && stderr.contains("'bogus'"),
        "the error must name the step, op and key, got: {stderr}"
    );
}

// ── AC-8: `build` refuses before writing anything ───────────────────────────

#[test]
fn build_refuses_a_non_finite_watermark_size_before_any_output() {
    let dir = TempDir::new().unwrap();
    let root = dir.path();
    for name in ["src/a.png", "src/b.png", "src/c.png"] {
        write_png(root, name);
    }
    std::fs::write(
        root.join("r.toml"),
        one_step("watermark", "text = \"hi\"\nsize = nan"),
    )
    .unwrap();
    std::fs::write(
        root.join("crustyimg.build.toml"),
        "version = 1\n\n[[target]]\nsource = \"src/*.png\"\nrecipe = \"r.toml\"\nout = \"dist\"\n",
    )
    .unwrap();

    let output = Command::new(BIN)
        .arg("build")
        .current_dir(root)
        .output()
        .expect("run build");
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert_eq!(output.status.code(), Some(1), "stderr: {stderr}");
    assert_eq!(files_in(&root.join("dist")), 0, "no output may be written");
    assert!(
        stderr.contains("error: target #0 ("),
        "the target line must stay, got: {stderr}"
    );
    assert!(
        stderr
            .lines()
            .any(|l| l.contains("recipe step 0 ('watermark')") && l.contains("'size'")),
        "a line must name the step, op and key, got: {stderr}"
    );
}
