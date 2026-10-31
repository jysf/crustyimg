//! Pins what every existing recipe produces, not just how it serializes.
//!
//! `tests/recipe_round_trip.rs` pins each recipe's canonical TOML, which is a
//! cache-key input. It never runs a recipe, so a change to what a recipe draws
//! (a resize filter, a watermark default) passes it. This test runs all 14
//! corpus and bundled recipes on three fixtures built here from raw samples and
//! compares the pixels before encoding, plus the encoded bytes for the two
//! recipes that end in PNG or lossless WebP. Lossy and AVIF bytes are not
//! pinned: the encoders promise determinism per machine, not across machines.
//!
//! Goldens live in `tests/fixtures/recipe_corpus/<stem>.output`. Regenerate
//! them with `CRUSTYIMG_BLESS=1 cargo test --test recipe_corpus_output --
//! --ignored bless`, and only when an output is meant to change.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::Command;

use ::image::{DynamicImage, ImageBuffer, ImageFormat, Rgb, Rgba, RgbaImage};
use sha2::{Digest as _, Sha256};

use crustyimg::image::Image;
use crustyimg::operation::OperationRegistry;
use crustyimg::recipe::Recipe;
use crustyimg::sink::encode_to_bytes;

// ─── The recipes and the literals the goldens are checked against ───────────

/// Every recipe under the pin: the corpus stems, then the bundled ones
/// (`bundled_<name>`). A literal list, so a recipe dropped from it goes red.
const RECIPES: &[&str] = &[
    "v1_resize_cover",
    "v1_resize_exact",
    "v1_resize_fill",
    "v1_resize_fit",
    "v1_resize_max",
    "v1_resize_percent",
    "v1_watermark_image_placement",
    "v1_watermark_text_font",
    "v2_ddphotos_cli_shape",
    "v2_format_quality_optimize",
    "v2_watermark_integer_numbers",
    "bundled_gallery",
    "bundled_product",
    "bundled_web",
];

const FIXTURES: [&str; 3] = ["rgb8_2600x1730", "rgba8_1730x2600", "rgba16_2600x1730"];

/// 14 recipes x 3 fixtures, plus 3 PNG and 3 WebP byte rows.
const ROWS: usize = 48;

/// Non-comment golden lines per target: 42 pixel rows, 3 PNG, 3 WebP and 3
/// chosen-format lines.
const GOLDEN_LINES: usize = 51;

/// What each recipe must produce, by fixture. Independent of the goldens, so a
/// golden blessed over a pipeline that ran nothing still fails here.
const EXPECTED_DIMS: [(&str, [&str; 3]); 14] = [
    ("v1_resize_cover", ["721x480", "640x962", "721x480"]),
    ("v1_resize_exact", ["640x480", "640x480", "640x480"]),
    ("v1_resize_fill", ["640x480", "640x480", "640x480"]),
    ("v1_resize_fit", ["640x426", "319x480", "640x426"]),
    ("v1_resize_max", ["800x532", "532x800", "800x532"]),
    ("v1_resize_percent", ["488x325", "325x488", "488x325"]),
    (
        "v1_watermark_image_placement",
        ["2600x1730", "1730x2600", "2600x1730"],
    ),
    (
        "v1_watermark_text_font",
        ["2600x1730", "1730x2600", "2600x1730"],
    ),
    (
        "v2_ddphotos_cli_shape",
        ["1000x665", "665x1000", "1000x665"],
    ),
    (
        "v2_format_quality_optimize",
        ["2600x1730", "1730x2600", "2600x1730"],
    ),
    (
        "v2_watermark_integer_numbers",
        ["1024x681", "681x1024", "1024x681"],
    ),
    ("bundled_gallery", ["2560x1703", "1703x2560", "2560x1703"]),
    ("bundled_product", ["1600x1065", "1065x1600", "1600x1065"]),
    ("bundled_web", ["2048x1363", "1363x2048", "2048x1363"]),
];

/// The bundled recipes, whose `optimize` choice is read through the binary.
#[cfg(all(feature = "avif", not(feature = "webp-lossy")))]
const BUNDLED: [&str; 3] = ["gallery", "product", "web"];

/// The recipe that ends in a PNG byte row, and the one that ends in WebP.
const PNG_RECIPE: &str = "v2_watermark_integer_numbers";
const WEBP_RECIPE: &str = "v2_format_quality_optimize";

/// Steps that cannot change a fixture: it carries no orientation, and
/// `identity` is a no-op by definition.
const INERT_OPS: [&str; 2] = ["auto-orient", "identity"];

// ─── Fixtures ───────────────────────────────────────────────────────────────

struct XorShift(u64);
impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

/// A seeded gradient-plus-noise image, built from raw samples (no encoder or decoder involved).
fn fixture(kind: &str, w: u32, h: u32) -> Image {
    let mut r = XorShift(0x5EED_0141 ^ (u64::from(w) << 20) ^ u64::from(h));
    let px = match kind {
        "rgb8" => DynamicImage::ImageRgb8(ImageBuffer::from_fn(w, h, |x, y| {
            let n = (r.next() & 31) as u32;
            Rgb([
                ((x * 255 / w + n) % 256) as u8,
                ((y * 255 / h + n) % 256) as u8,
                (((x + y) / 7 + n) % 256) as u8,
            ])
        })),
        "rgba8" => DynamicImage::ImageRgba8(RgbaImage::from_fn(w, h, |x, y| {
            let n = (r.next() & 31) as u32;
            Rgba([
                ((x * 255 / w + n) % 256) as u8,
                ((y * 255 / h + n) % 256) as u8,
                90,
                (128 + (x + y) % 128) as u8,
            ])
        })),
        "rgba16" => DynamicImage::ImageRgba16(ImageBuffer::from_fn(w, h, |x, y| {
            let n = (r.next() & 2047) as u32;
            Rgba([
                ((x * 65535 / w + n) % 65536) as u16,
                ((y * 65535 / h + n) % 65536) as u16,
                20000,
                (32768 + (x * 7 + y) % 32768) as u16,
            ])
        })),
        other => panic!("unknown fixture kind {other}"),
    };
    Image::from_parts(px, ImageFormat::Png, None)
}

fn fixtures() -> Vec<(&'static str, Image)> {
    vec![
        (FIXTURES[0], fixture("rgb8", 2600, 1730)),
        (FIXTURES[1], fixture("rgba8", 1730, 2600)),
        (FIXTURES[2], fixture("rgba16", 2600, 1730)),
    ]
}

/// A seeded 64x32 RGBA logo for the image watermark, as PNG bytes. An input to
/// the watermark step (it decodes it), not a fixture.
fn logo_png() -> Vec<u8> {
    let mut r = XorShift(0x1060);
    let img = RgbaImage::from_fn(64, 32, |_, _| {
        let v = r.next();
        Rgba([
            v as u8,
            (v >> 8) as u8,
            (v >> 16) as u8,
            128 | (v >> 24) as u8,
        ])
    });
    let mut out = std::io::Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(img)
        .write_to(&mut out, ImageFormat::Png)
        .expect("encode logo");
    out.into_inner()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn sha_hex(bytes: &[u8]) -> String {
    hex(&Sha256::digest(bytes))
}

fn pixel_sha(img: &Image) -> String {
    sha_hex(img.pixels().as_bytes())
}

fn dims_of(img: &Image) -> String {
    format!("{}x{}", img.pixels().width(), img.pixels().height())
}

fn depth_of(img: &Image) -> u32 {
    let c = img.pixels().color();
    u32::from(c.bytes_per_pixel()) * 8 / u32::from(c.channel_count())
}

// ─── Computing a row ────────────────────────────────────────────────────────

/// What one recipe produced on one fixture, before encoding.
#[derive(Clone, Debug, PartialEq)]
struct Computed {
    dims: String,
    colour: String,
    depth: u32,
    sha: String,
    /// `("png" | "webp", sha256 of the encoded file)` for the byte rows.
    bytes: Option<(&'static str, String)>,
}

impl Computed {
    fn of(img: &Image) -> Computed {
        Computed {
            dims: dims_of(img),
            colour: format!("{:?}", img.pixels().color()),
            depth: depth_of(img),
            sha: pixel_sha(img),
            bytes: None,
        }
    }

    fn pixels_rest(&self) -> String {
        format!(
            "dims={} colour={} depth={} sha256={}",
            self.dims, self.colour, self.depth, self.sha
        )
    }
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn corpus_dir() -> PathBuf {
    root().join("tests/fixtures/recipe_corpus")
}

/// Read text with line endings normalized: a Windows checkout may convert the
/// repo's LF files to CRLF.
fn read_text(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
        .replace("\r\n", "\n")
}

fn recipe_source(stem: &str) -> PathBuf {
    match stem.strip_prefix("bundled_") {
        Some(name) => root().join(format!("recipes/{name}.toml")),
        None => corpus_dir().join(format!("{stem}.toml")),
    }
}

fn load_recipe(stem: &str) -> Recipe {
    Recipe::from_toml(&read_text(&recipe_source(stem)))
        .unwrap_or_else(|e| panic!("{stem} must parse: {e}"))
}

/// The recipe's pixel steps: a terminal `optimize` marker is recipe syntax, not
/// an op, so it is dropped, and asset keys get the bytes the CLI's resolver
/// would attach.
fn pixel_recipe(stem: &str, logo: &[u8]) -> Recipe {
    let mut recipe = load_recipe(stem);
    if recipe.steps.last().is_some_and(|s| s.op == "optimize") {
        recipe.steps.pop();
    }
    for step in &mut recipe.steps {
        if step.params.get_str("image").is_some() {
            step.params.set_resolved_bytes("image", logo.to_vec());
        }
        if step.params.get_str("font").is_some() {
            step.params
                .set_resolved_bytes("font", crustyimg::text::DEFAULT_FONT.to_vec());
        }
    }
    recipe
}

fn label(recipe: &str, fixture: &str) -> String {
    format!("{recipe} × {fixture}")
}

// The control lines: what a run says when the pipeline did not do its job.

fn dims_control_line(recipe: &str, fixture: &str, actual: &str, expected: &str) -> String {
    format!(
        "{}: dims {actual}, expected {expected} (expected-dims table)",
        label(recipe, fixture)
    )
}

fn equals_fixture_line(recipe: &str, fixture: &str) -> String {
    format!("{}: output equals the fixture", label(recipe, fixture))
}

fn step_unchanged_line(recipe: &str, fixture: &str, index: usize, op: &str) -> String {
    format!(
        "{}: step {index} ('{op}') changed nothing",
        label(recipe, fixture)
    )
}

/// Run `stem` on `fx`. Returns the computed row (when the pipeline ran) and
/// the control lines of "it must be able to fail": expected dimensions, output
/// differs from the input, every acting step acts, and the step-by-step result
/// equals the real pipeline.
fn compute(
    stem: &str,
    fixture_name: &str,
    fx: &Image,
    logo: &[u8],
) -> (Option<Computed>, Vec<String>) {
    let who = label(stem, fixture_name);
    let registry = OperationRegistry::with_builtins();
    let recipe = pixel_recipe(stem, logo);
    let mut lines = Vec::new();

    let pipeline = match recipe.build_pipeline(&registry) {
        Ok(p) => p,
        Err(e) => return (None, vec![format!("{who}: error: {e}")]),
    };
    let out = match pipeline.run(fx.clone()) {
        Ok(img) => img,
        Err(e) => return (None, vec![format!("{who}: error: {e}")]),
    };
    let mut computed = Computed::of(&out);

    // (a) dimensions against a literal table; (b) a recipe that draws
    // something does not return its input. (b) is reported only when (a) held:
    // a wrong size already says the recipe did not run.
    let index = FIXTURES
        .iter()
        .position(|f| *f == fixture_name)
        .expect("known fixture");
    let expected = EXPECTED_DIMS
        .iter()
        .find(|(name, _)| *name == stem)
        .map(|(_, dims)| dims[index])
        .unwrap_or_else(|| panic!("{stem} has no expected dims"));
    if computed.dims != expected {
        lines.push(dims_control_line(
            stem,
            fixture_name,
            &computed.dims,
            expected,
        ));
    } else if computed.sha == pixel_sha(fx) {
        lines.push(equals_fixture_line(stem, fixture_name));
    }
    // (c) + (d) one step at a time, each fed the previous output
    let mut current = fx.clone();
    for (i, step) in recipe.steps.iter().enumerate() {
        let mut single = recipe.clone();
        single.steps = vec![step.clone()];
        let stepped = single
            .build_pipeline(&registry)
            .map_err(|e| e.to_string())
            .and_then(|p| p.run(current.clone()).map_err(|e| e.to_string()));
        match stepped {
            Ok(next) => {
                if !INERT_OPS.contains(&step.op.as_str()) && pixel_sha(&next) == pixel_sha(&current)
                {
                    lines.push(step_unchanged_line(stem, fixture_name, i, &step.op));
                }
                current = next;
            }
            Err(e) => {
                lines.push(format!("{who}: error: step {i} ('{}'): {e}", step.op));
                break;
            }
        }
    }
    if pixel_sha(&current) != computed.sha {
        lines.push(format!(
            "{who}: step-by-step result differs from the whole pipeline"
        ));
    }

    // Encoded bytes for the two recipes pinned past the pixels.
    let byte_row = match stem {
        PNG_RECIPE => Some(("png", ImageFormat::Png)),
        WEBP_RECIPE => Some(("webp", ImageFormat::WebP)),
        _ => None,
    };
    if let Some((kind, format)) = byte_row {
        match encode_to_bytes(&out, format, None) {
            Ok(bytes) => computed.bytes = Some((kind, sha_hex(&bytes))),
            Err(e) => lines.push(format!("{who}: error: {kind} encode: {e}")),
        }
    }
    (Some(computed), lines)
}

// ─── The golden format ──────────────────────────────────────────────────────

/// A parsed `.output` file: `(fixture, kind) -> rest of the line`.
#[derive(Debug, Default)]
struct Golden {
    rows: BTreeMap<(String, String), String>,
    /// Non-comment lines, in file order.
    lines: Vec<String>,
}

const KINDS: [&str; 4] = ["pixels", "png", "webp", "optimize"];

/// Parse a golden. Every non-comment line is `<fixture> <kind> <rest>`; keys
/// are unique and the rest follows the kind's grammar.
fn parse_golden(text: &str) -> Result<Golden, String> {
    let mut golden = Golden::default();
    for raw in text.replace("\r\n", "\n").lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut parts = line.splitn(3, ' ');
        let (fixture, kind, rest) = match (parts.next(), parts.next(), parts.next()) {
            (Some(f), Some(k), Some(r)) => (f, k, r),
            _ => return Err(format!("malformed golden line: {line}")),
        };
        if !FIXTURES.contains(&fixture) {
            return Err(format!("unknown fixture in golden line: {line}"));
        }
        if !KINDS.contains(&kind) {
            return Err(format!("unknown kind in golden line: {line}"));
        }
        let ok = match kind {
            "pixels" => pixel_fields(rest).is_some(),
            "optimize" => rest.starts_with("format=") && !rest.contains(' '),
            _ => is_sha_field(rest),
        };
        if !ok {
            return Err(format!("malformed golden line: {line}"));
        }
        let key = (fixture.to_owned(), kind.to_owned());
        if golden.rows.insert(key, rest.to_owned()).is_some() {
            return Err(format!("duplicate golden key: {fixture} {kind}"));
        }
        golden.lines.push(line.to_owned());
    }
    Ok(golden)
}

fn is_sha_field(s: &str) -> bool {
    s.strip_prefix("sha256=")
        .is_some_and(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// `(dims, colour, depth, sha)` from `dims=… colour=… depth=… sha256=…`.
fn pixel_fields(rest: &str) -> Option<[String; 4]> {
    let mut it = rest.split(' ');
    let dims = it.next()?.strip_prefix("dims=")?;
    let colour = it.next()?.strip_prefix("colour=")?;
    let depth = it.next()?.strip_prefix("depth=")?;
    let sha = it.next()?;
    if it.next().is_some() || !is_sha_field(sha) {
        return None;
    }
    Some([
        dims.to_owned(),
        colour.to_owned(),
        depth.to_owned(),
        sha["sha256=".len()..].to_owned(),
    ])
}

/// The targets with committed goldens. Resize and compositing run on float
/// maths whose last bits differ by OS and CPU, so each target is pinned exactly
/// on its own file rather than loosened to a tolerance.
const TARGETS: &[&str] = &["linux-x86_64", "macos-aarch64", "windows-x86_64"];

/// Where the golden for `stem` on a target lives. A target outside `TARGETS` is
/// an error, never a skip and never another target's file. A known target whose
/// file is missing still gets its path: `check_recipes` reports the file as
/// missing and prints the dump to bless it from.
fn golden_path_for(stem: &str, os: &str, arch: &str) -> Result<PathBuf, String> {
    let target = format!("{os}-{arch}");
    let path = corpus_dir().join(format!("{stem}.{target}.output"));
    if !TARGETS.contains(&target.as_str()) {
        return Err(format!(
            "no recipe golden for {target}: goldens exist for {}; bless on this target and add it to TARGETS",
            TARGETS.join(", ")
        ));
    }
    Ok(path)
}

/// This target's golden path for `stem`; panics when the target is unknown.
fn golden_path(stem: &str) -> PathBuf {
    golden_path_for(stem, std::env::consts::OS, std::env::consts::ARCH)
        .unwrap_or_else(|msg| panic!("{msg}"))
}

/// Compare a `pixels` row. The first field that moved wins, in the order
/// dims, colour, depth, pixels.
fn pixel_mismatch(
    recipe: &str,
    fixture: &str,
    golden: Option<&str>,
    actual: &Computed,
) -> Option<String> {
    let who = label(recipe, fixture);
    let Some(golden) = golden else {
        return Some(format!("{who}: no golden line (bless to add it)"));
    };
    let Some([dims, colour, depth, sha]) = pixel_fields(golden) else {
        return Some(format!("{who}: malformed golden line: {golden}"));
    };
    let depth_now = actual.depth.to_string();
    if dims != actual.dims {
        Some(format!(
            "{who}: dims changed: golden {dims}, actual {}",
            actual.dims
        ))
    } else if colour != actual.colour {
        Some(format!(
            "{who}: colour changed: golden {colour}, actual {}",
            actual.colour
        ))
    } else if depth != depth_now {
        Some(format!(
            "{who}: depth changed: golden {depth}, actual {depth_now}"
        ))
    } else if sha != actual.sha {
        Some(format!(
            "{who}: pixels changed (dims {dims}, colour {colour}, depth {depth} unchanged)"
        ))
    } else {
        None
    }
}

/// Compare a `png`/`webp` byte row. `pixels_changed` says whether the same
/// row's pixels line already moved, so the message tells the two apart.
fn bytes_mismatch(
    recipe: &str,
    fixture: &str,
    kind: &str,
    golden: Option<&str>,
    actual_sha: &str,
    pixels_changed: bool,
) -> Option<String> {
    let who = label(recipe, fixture);
    let Some(golden) = golden else {
        return Some(format!("{who}: no golden line (bless to add it)"));
    };
    if golden == format!("sha256={actual_sha}") {
        return None;
    }
    let also = if pixels_changed {
        "pixels also changed"
    } else {
        "pixels unchanged"
    };
    Some(format!("{who}: {kind} bytes changed ({also})"))
}

/// Compare the format `optimize` chose on a bundled recipe.
fn optimize_mismatch(
    recipe: &str,
    fixture: &str,
    golden: Option<&str>,
    chose: &str,
) -> Option<String> {
    let who = label(recipe, fixture);
    let Some(golden) = golden else {
        return Some(format!("{who}: no golden line (bless to add it)"));
    };
    let want = golden.strip_prefix("format=").unwrap_or(golden);
    (want != chose).then(|| format!("{who}: optimize chose {chose}, golden {want}"))
}

fn render_file(
    stem: &str,
    rows: &[(&str, Computed)],
    optimize: &BTreeMap<String, String>,
) -> String {
    let mut out = format!(
        "# {stem}: what this recipe produces on each fixture.\n\
         # Written by `CRUSTYIMG_BLESS=1 cargo test --test recipe_corpus_output -- --ignored bless`.\n\
         # A changed line means an existing recipe's output changed.\n"
    );
    for (fixture, c) in rows {
        out.push_str(&format!("{fixture} pixels {}\n", c.pixels_rest()));
        if let Some((kind, sha)) = &c.bytes {
            out.push_str(&format!("{fixture} {kind} sha256={sha}\n"));
        }
        if let Some(format) = optimize.get(*fixture) {
            out.push_str(&format!("{fixture} optimize {format}\n"));
        }
    }
    out
}

/// The file as a CI log shows it, named for the running target so it can be
/// copied straight into the repo.
fn dump_block(stem: &str, os: &str, arch: &str, content: &str) -> String {
    format!(
        "----- BEGIN tests/fixtures/recipe_corpus/{stem}.{os}-{arch}.output -----\n{content}----- END -----\n"
    )
}

// ─── Checking every row ─────────────────────────────────────────────────────

struct Report {
    mismatches: Vec<String>,
    /// The rows with at least one mismatch line: `(recipe, fixture, kind)`.
    changed_rows: BTreeSet<(String, String, &'static str)>,
    rows_checked: usize,
    dump: String,
}

impl Report {
    /// Record `line` against the row it belongs to.
    fn flag(&mut self, recipe: &str, fixture: &str, kind: &'static str, line: Option<String>) {
        if let Some(line) = line {
            self.mismatches.push(line);
            self.changed_rows
                .insert((recipe.to_owned(), fixture.to_owned(), kind));
        }
    }
}

/// Check `recipes` against the goldens `golden_text` hands back (`None` when the
/// file does not exist). A missing file is one mismatch line, and the rows
/// still compute against an empty golden so its dump block is still printed.
fn check_recipes(
    recipes: &[&str],
    os: &str,
    arch: &str,
    golden_text: &dyn Fn(&str) -> Option<String>,
) -> Report {
    let logo = logo_png();
    let fxs = fixtures();
    let mut report = Report {
        mismatches: Vec::new(),
        changed_rows: BTreeSet::new(),
        rows_checked: 0,
        dump: String::new(),
    };
    for stem in recipes {
        let golden = match golden_text(stem) {
            Some(text) => match parse_golden(&text) {
                Ok(g) => g,
                Err(e) => {
                    report.mismatches.push(format!("{stem}: {e}"));
                    Golden::default()
                }
            },
            None => {
                report.mismatches.push(format!(
                    "{stem}: no golden file tests/fixtures/recipe_corpus/{stem}.{os}-{arch}.output (bless this target from this run's dump below)"
                ));
                Golden::default()
            }
        };
        let mut rows = Vec::new();
        let mut optimize = BTreeMap::new();
        for (fixture_name, fx) in &fxs {
            let (computed, controls) = compute(stem, fixture_name, fx, &logo);
            for line in controls {
                report.flag(stem, fixture_name, "pixels", Some(line));
            }
            if let Some(rest) = golden
                .rows
                .get(&((*fixture_name).to_owned(), "optimize".to_owned()))
            {
                optimize.insert((*fixture_name).to_owned(), rest.clone());
            }
            let Some(c) = computed else {
                // The error line is already recorded; the row still counts.
                report.rows_checked += 1;
                continue;
            };
            let get = |kind: &str| {
                golden
                    .rows
                    .get(&((*fixture_name).to_owned(), kind.to_owned()))
                    .map(String::as_str)
            };
            report.rows_checked += 1;
            let pixel = pixel_mismatch(stem, fixture_name, get("pixels"), &c);
            let pixels_changed = pixel.is_some();
            report.flag(stem, fixture_name, "pixels", pixel);
            if let Some((kind, sha)) = &c.bytes {
                report.rows_checked += 1;
                let line = bytes_mismatch(stem, fixture_name, kind, get(kind), sha, pixels_changed);
                report.flag(stem, fixture_name, kind, line);
            }
            rows.push((*fixture_name, c));
        }
        report.dump.push_str(&dump_block(
            stem,
            os,
            arch,
            &render_file(stem, &rows, &optimize),
        ));
    }
    report
}

fn check_all() -> Report {
    check_recipes(
        RECIPES,
        std::env::consts::OS,
        std::env::consts::ARCH,
        &|stem| {
            let path = golden_path(stem);
            path.exists().then(|| read_text(&path))
        },
    )
}

#[test]
fn every_recipe_output_matches_its_golden() {
    let report = check_all();
    assert_eq!(
        report.rows_checked, ROWS,
        "rows checked {}, expected {ROWS}",
        report.rows_checked
    );
    assert_eq!(EXPECTED_DIMS.len() * FIXTURES.len(), 42);
    if !report.mismatches.is_empty() {
        panic!(
            "{}\n{} of {ROWS} rows changed\n{}",
            report.mismatches.join("\n"),
            report.changed_rows.len(),
            report.dump
        );
    }
}

#[test]
fn the_fixtures_are_the_ones_the_goldens_were_blessed_on() {
    let want = [
        (
            "rgb8_2600x1730",
            "2600x1730",
            "Rgb8",
            "94daa4afe5bb126411653297dc212b6119033ca9a24a7ada79ba700931835fe9",
        ),
        (
            "rgba8_1730x2600",
            "1730x2600",
            "Rgba8",
            "7df0cef0ac38b40f5fbd98f7063712000c4ca3ad7e697891d4b10558d045d713",
        ),
        (
            "rgba16_2600x1730",
            "2600x1730",
            "Rgba16",
            "75dfcd86a58adb7ed22037cbd98166664a2518594791b8e0ce6eff93b2d8aba6",
        ),
    ];
    for ((name, img), (want_name, dims, colour, sha)) in fixtures().iter().zip(want) {
        assert_eq!(*name, want_name);
        assert_eq!(dims_of(img), dims, "{name} dims");
        assert_eq!(
            format!("{:?}", img.pixels().color()),
            colour,
            "{name} colour"
        );
        assert_eq!(pixel_sha(img), sha, "{name} sha256");
    }
}

// ─── The message format ─────────────────────────────────────────────────────

fn computed(dims: &str, colour: &str, depth: u32, sha: &str) -> Computed {
    Computed {
        dims: dims.to_owned(),
        colour: colour.to_owned(),
        depth,
        sha: sha.repeat(64),
        bytes: None,
    }
}

#[test]
fn a_row_mismatch_names_the_field_that_moved() {
    let golden = |c: &Computed| c.pixels_rest();
    let base = computed("800x532", "Rgb8", 8, "a");
    let rec = "v1_resize_max";
    let fx = "rgb8_2600x1730";
    let check = |g: &Computed, a: &Computed| pixel_mismatch(rec, fx, Some(&golden(g)), a);

    assert_eq!(
        check(&base, &computed("801x532", "Rgb8", 8, "a")).as_deref(),
        Some("v1_resize_max × rgb8_2600x1730: dims changed: golden 800x532, actual 801x532")
    );
    assert_eq!(
        check(&base, &computed("800x532", "Rgba8", 8, "a")).as_deref(),
        Some("v1_resize_max × rgb8_2600x1730: colour changed: golden Rgb8, actual Rgba8")
    );
    assert_eq!(
        check(&base, &computed("800x532", "Rgb8", 16, "a")).as_deref(),
        Some("v1_resize_max × rgb8_2600x1730: depth changed: golden 8, actual 16")
    );
    // Dims unchanged, pixels changed: a message that always said "dims" fails here.
    assert_eq!(
        check(&base, &computed("800x532", "Rgb8", 8, "b")).as_deref(),
        Some(
            "v1_resize_max × rgb8_2600x1730: pixels changed (dims 800x532, colour Rgb8, depth 8 unchanged)"
        )
    );
    assert_eq!(check(&base, &base), None);
    assert_eq!(
        pixel_mismatch(rec, fx, None, &base).as_deref(),
        Some("v1_resize_max × rgb8_2600x1730: no golden line (bless to add it)")
    );

    let sha = "c".repeat(64);
    let line = format!("sha256={sha}");
    let png = |g: Option<&str>, changed| bytes_mismatch(PNG_RECIPE, fx, "png", g, &sha, changed);
    assert_eq!(png(Some(&line), false), None);
    assert_eq!(
        png(Some("sha256=".to_owned().as_str()), false).as_deref(),
        Some("v2_watermark_integer_numbers × rgb8_2600x1730: png bytes changed (pixels unchanged)")
    );
    assert_eq!(
        png(Some("sha256=x"), true).as_deref(),
        Some(
            "v2_watermark_integer_numbers × rgb8_2600x1730: png bytes changed (pixels also changed)"
        )
    );
    assert_eq!(
        optimize_mismatch("bundled_web", fx, Some("format=avif"), "webp").as_deref(),
        Some("bundled_web × rgb8_2600x1730: optimize chose webp, golden avif")
    );
    assert_eq!(
        optimize_mismatch("bundled_web", fx, Some("format=avif"), "avif"),
        None
    );

    // The control lines `compute` records.
    assert_eq!(
        dims_control_line(rec, fx, "1x1", "800x532"),
        "v1_resize_max × rgb8_2600x1730: dims 1x1, expected 800x532 (expected-dims table)"
    );
    assert_eq!(
        equals_fixture_line(rec, fx),
        "v1_resize_max × rgb8_2600x1730: output equals the fixture"
    );
    assert_eq!(
        step_unchanged_line(rec, fx, 2, "resize"),
        "v1_resize_max × rgb8_2600x1730: step 2 ('resize') changed nothing"
    );

    // The dump names the running target and carries an `optimize` line through.
    let mut optimize = BTreeMap::new();
    optimize.insert(fx.to_owned(), "format=avif".to_owned());
    let rows = [(fx, base.clone())];
    let content = render_file("bundled_web", &rows, &optimize);
    let dump = dump_block("bundled_web", "linux", "x86_64", &content);
    assert!(dump.starts_with(
        "----- BEGIN tests/fixtures/recipe_corpus/bundled_web.linux-x86_64.output -----\n# bundled_web:"
    ));
    assert!(dump.ends_with("rgb8_2600x1730 optimize format=avif\n----- END -----\n"));
    assert!(dump.contains(&format!("rgb8_2600x1730 pixels {}\n", base.pixels_rest())));
    // What the dump holds parses back as a golden.
    assert_eq!(parse_golden(&content).expect("dump parses").lines.len(), 2);
}

// ─── The binary agrees with the library ─────────────────────────────────────

fn write_fixture_png(dir: &Path, name: &str, fx: &Image) -> PathBuf {
    let path = dir.join(format!("{name}.png"));
    fx.pixels()
        .save_with_format(&path, ImageFormat::Png)
        .expect("write fixture png");
    // The binary must see the pixels the goldens were blessed on.
    let back = Image::load(&path).expect("load fixture png");
    assert_eq!(
        pixel_sha(&back),
        pixel_sha(fx),
        "{name}: transport changed the pixels"
    );
    path
}

fn golden_row(stem: &str, fixture: &str, kind: &str) -> String {
    let golden = parse_golden(&read_text(&golden_path(stem))).expect("golden parses");
    golden
        .rows
        .get(&(fixture.to_owned(), kind.to_owned()))
        .unwrap_or_else(|| panic!("{stem} has no `{fixture} {kind}` golden line"))
        .clone()
}

fn crustyimg(dir: &Path) -> Command {
    // The binary Cargo built for this test, never one on PATH.
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_crustyimg"));
    cmd.current_dir(dir);
    cmd
}

/// Run `apply` on `stem` and compare what it wrote with the golden bytes row.
fn seam_row(fx: &Image, stem: &str, ext: &str, kind: &str) -> Result<(), String> {
    let dir = tempfile::tempdir().expect("tempdir");
    write_fixture_png(dir.path(), FIXTURES[0], fx);
    std::fs::write(dir.path().join("logo.png"), logo_png()).expect("write logo");
    std::fs::copy(recipe_source(stem), dir.path().join("recipe.toml")).expect("copy recipe");

    let out = crustyimg(dir.path())
        .args(["apply", "--recipe", "recipe.toml"])
        .arg(format!("{}.png", FIXTURES[0]))
        .args(["--out-dir", "out"])
        .output()
        .expect("run crustyimg");
    if !out.status.success() {
        return Err(format!(
            "{stem}: apply failed: {}",
            String::from_utf8_lossy(&out.stderr)
        ));
    }

    let mut written: Vec<_> = std::fs::read_dir(dir.path().join("out"))
        .expect("out dir")
        .map(|e| e.expect("entry").file_name().to_string_lossy().into_owned())
        .collect();
    written.sort();
    let name = format!("{}.{ext}", FIXTURES[0]);
    if written != [name.clone()] {
        return Err(format!(
            "{stem}: apply output files {written:?}, expected [{name:?}]"
        ));
    }

    let bytes = std::fs::read(dir.path().join("out").join(&name)).expect("read output");
    let actual = format!("sha256={}", sha_hex(&bytes));
    if actual != golden_row(stem, FIXTURES[0], kind) {
        return Err(format!(
            "{stem}: apply wrote different bytes than the library seam"
        ));
    }
    Ok(())
}

#[test]
fn the_library_seam_writes_the_bytes_apply_writes() {
    let fx = fixture("rgb8", 2600, 1730);
    // Both rows run before the test fails, so one red row cannot hide the other.
    let results: Vec<_> = [(PNG_RECIPE, "png", "png"), (WEBP_RECIPE, "webp", "webp")]
        .into_iter()
        .map(|(stem, ext, kind)| (kind, seam_row(&fx, stem, ext, kind)))
        .collect();
    assert_eq!(results.len(), 2);
    let report: Vec<_> = results
        .iter()
        .map(|(kind, r)| match r {
            Ok(()) => format!("{kind} row green"),
            Err(e) => format!("{kind} row red: {e}"),
        })
        .collect();
    assert!(
        results.iter().all(|(_, r)| r.is_ok()),
        "{}",
        report.join("\n")
    );
}

/// The format `optimize` chose for each bundled recipe on the 8-bit fixture.
/// Only the binary can say: the decision is private to the CLI.
#[cfg(all(feature = "avif", not(feature = "webp-lossy")))]
fn chosen_formats() -> BTreeMap<&'static str, String> {
    let fx = fixture("rgb8", 2600, 1730);
    std::thread::scope(|scope| {
        let handles: Vec<_> = BUNDLED
            .iter()
            .map(|name| {
                let fx = &fx;
                scope.spawn(move || {
                    let dir = tempfile::tempdir().expect("tempdir");
                    write_fixture_png(dir.path(), FIXTURES[0], fx);
                    let out = crustyimg(dir.path())
                        .args(["apply", "--recipe", name, "--json"])
                        .arg(format!("{}.png", FIXTURES[0]))
                        .args(["--out-dir", "out"])
                        .output()
                        .expect("run crustyimg");
                    assert!(
                        out.status.success(),
                        "bundled {name}: apply failed: {}",
                        String::from_utf8_lossy(&out.stderr)
                    );
                    let report: serde_json::Value =
                        serde_json::from_slice(&out.stdout).expect("--json report");
                    let winner = report["winner"].as_u64().expect("winner index") as usize;
                    let format = report["candidates"][winner]["format"]
                        .as_str()
                        .expect("winner format")
                        .to_owned();
                    let ext = std::fs::read_dir(dir.path().join("out"))
                        .expect("out dir")
                        .map(|e| e.expect("entry").path())
                        .next()
                        .and_then(|p| p.extension().map(|x| x.to_string_lossy().into_owned()))
                        .expect("an output file");
                    assert_eq!(ext, format, "bundled {name}: output extension vs winner");
                    (*name, format)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("thread"))
            .collect()
    })
}

#[cfg(all(feature = "avif", not(feature = "webp-lossy")))]
#[test]
fn optimize_picks_the_pinned_format_for_each_bundled_recipe() {
    let chosen = chosen_formats();
    let mut rows_checked = 0;
    let mut mismatches = Vec::new();
    for name in BUNDLED {
        let stem = format!("bundled_{name}");
        let golden = golden_row(&stem, FIXTURES[0], "optimize");
        mismatches.extend(optimize_mismatch(
            &stem,
            FIXTURES[0],
            Some(&golden),
            &chosen[name],
        ));
        rows_checked += 1;
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    assert_eq!(rows_checked, 3);
}

// ─── The goldens themselves ─────────────────────────────────────────────────

#[test]
fn every_golden_line_is_checked_and_no_golden_is_stray() {
    let mut on_disk: Vec<String> = std::fs::read_dir(corpus_dir())
        .expect("corpus dir")
        .filter_map(|e| {
            let name = e.ok()?.file_name().to_string_lossy().into_owned();
            name.strip_suffix(".output").map(str::to_owned)
        })
        .collect();
    on_disk.sort();
    let mut want: Vec<String> = RECIPES
        .iter()
        .flat_map(|stem| TARGETS.iter().map(move |t| format!("{stem}.{t}")))
        .collect();
    want.sort();
    assert_eq!(
        on_disk, want,
        "the .output files must be exactly the recipes under the pin, once per target"
    );

    for target in TARGETS {
        check_target_goldens(target);
    }
}

fn check_target_goldens(target: &str) {
    let mut total = 0;
    for stem in RECIPES {
        let golden = parse_golden(&read_text(
            &corpus_dir().join(format!("{stem}.{target}.output")),
        ))
        .unwrap_or_else(|e| panic!("{stem}.{target}: {e}"));
        let count = |kind: &str| golden.rows.keys().filter(|(_, k)| k == kind).count();
        assert_eq!(count("pixels"), 3, "{stem}: pixels lines");
        assert_eq!(
            count("png"),
            if *stem == PNG_RECIPE { 3 } else { 0 },
            "{stem}: png lines"
        );
        assert_eq!(
            count("webp"),
            if *stem == WEBP_RECIPE { 3 } else { 0 },
            "{stem}: webp lines"
        );
        assert_eq!(
            count("optimize"),
            if stem.starts_with("bundled_") { 1 } else { 0 },
            "{stem}: optimize lines"
        );
        total += golden.lines.len();
    }
    assert_eq!(total, GOLDEN_LINES, "{target}: golden lines");
}

// ─── Blessing ───────────────────────────────────────────────────────────────

/// Whether a bless may write. `Ok(false)` means the switch is off and nothing
/// should happen; an `Err` is a refusal that must fail loudly.
fn bless_gate(bless: Option<&OsStr>, ci: Option<&OsStr>) -> Result<bool, String> {
    if bless.is_none() {
        return Ok(false);
    }
    if ci.is_some_and(|v| !v.is_empty()) {
        return Err("refusing to bless recipe goldens in CI (CI is set)".to_owned());
    }
    Ok(true)
}

#[test]
fn bless_refuses_in_ci_and_without_the_switch() {
    let one = OsStr::new("1");
    assert_eq!(bless_gate(None, None), Ok(false));
    assert_eq!(bless_gate(Some(one), None), Ok(true));
    assert_eq!(bless_gate(Some(one), Some(OsStr::new(""))), Ok(true));
    let err = bless_gate(Some(one), Some(OsStr::new("true"))).expect_err("CI refuses");
    assert!(err.contains("refusing to bless"), "{err}");
}

/// Writes the `.output` files. Run it with
/// `CRUSTYIMG_BLESS=1 cargo test --test recipe_corpus_output -- --ignored bless`
/// only when the goldens should describe the current code.
#[test]
#[ignore]
fn bless_recipe_corpus_outputs() {
    let bless = std::env::var_os("CRUSTYIMG_BLESS");
    let ci = std::env::var_os("CI");
    match bless_gate(bless.as_deref(), ci.as_deref()) {
        Err(msg) => panic!("{msg}"),
        Ok(false) => {
            eprintln!("CRUSTYIMG_BLESS is not set; not writing goldens");
            return;
        }
        Ok(true) => {}
    }
    bless_all();
}

#[cfg(not(all(feature = "avif", not(feature = "webp-lossy"))))]
fn bless_all() {
    panic!("bless needs the default codec set (avif on, webp-lossy off)");
}

#[cfg(all(feature = "avif", not(feature = "webp-lossy")))]
fn bless_all() {
    let logo = logo_png();
    let fxs = fixtures();
    let chosen = chosen_formats();
    for stem in RECIPES {
        let mut optimize = BTreeMap::new();
        if let Some(name) = stem.strip_prefix("bundled_") {
            optimize.insert(FIXTURES[0].to_owned(), format!("format={}", chosen[name]));
        }
        let rows: Vec<_> = fxs
            .iter()
            .map(|(name, fx)| {
                let (computed, _) = compute(stem, name, fx, &logo);
                (
                    *name,
                    computed.unwrap_or_else(|| panic!("{stem} × {name} failed to run")),
                )
            })
            .collect();
        let path = corpus_dir().join(format!(
            "{stem}.{}-{}.output",
            std::env::consts::OS,
            std::env::consts::ARCH
        ));
        std::fs::write(path, render_file(stem, &rows, &optimize)).expect("write golden");
    }
}

#[test]
fn a_target_without_goldens_fails() {
    let err = golden_path_for("v1_resize_max", "linux", "riscv64").expect_err("no riscv64 golden");
    assert_eq!(
        err,
        "no recipe golden for linux-riscv64: goldens exist for linux-x86_64, macos-aarch64, windows-x86_64; bless on this target and add it to TARGETS"
    );
    let ok = golden_path_for("v1_resize_max", "macos", "aarch64").expect("mac golden");
    assert!(ok.ends_with("v1_resize_max.macos-aarch64.output"), "{ok:?}");
}

#[test]
fn a_missing_golden_fails_and_still_prints_its_dump() {
    let (os, arch) = (std::env::consts::OS, std::env::consts::ARCH);
    let recipes = ["v1_resize_max", "v1_resize_exact", "v1_resize_fit"];
    // Only the first recipe's golden is absent. The others are present but
    // empty, so what the test counts does not depend on what the pipeline draws.
    let report = check_recipes(&recipes, os, arch, &|stem| {
        (stem != "v1_resize_max").then(String::new)
    });

    let missing: Vec<_> = report
        .mismatches
        .iter()
        .filter(|l| l.contains("no golden file"))
        .collect();
    let want = format!(
        "v1_resize_max: no golden file tests/fixtures/recipe_corpus/v1_resize_max.{os}-{arch}.output (bless this target from this run's dump below)"
    );
    assert_eq!(missing, [&want]);
    // The failure count includes the missing-file line, plus a "no golden line"
    // for each of the 3 pixels rows of each of the 3 recipes.
    let no_line = report
        .mismatches
        .iter()
        .filter(|l| l.contains("no golden line"))
        .count();
    assert_eq!(no_line, 9);
    assert!(report.mismatches.len() > 9, "{:#?}", report.mismatches);
    assert!(report.changed_rows.len() >= 9);

    // Every recipe's block is printed, complete, including the one with no golden.
    for stem in recipes {
        let begin =
            format!("----- BEGIN tests/fixtures/recipe_corpus/{stem}.{os}-{arch}.output -----\n");
        let start = report
            .dump
            .find(&begin)
            .unwrap_or_else(|| panic!("no block for {stem}"));
        let block = &report.dump[start + begin.len()..];
        let end = block.find("----- END -----\n").expect("block is closed");
        let content = &block[..end];
        assert!(content.starts_with(&format!("# {stem}:")), "{stem}: header");
        assert_eq!(
            content.lines().filter(|l| l.contains(" pixels ")).count(),
            3,
            "{stem}: one pixels line per fixture"
        );
    }
    assert_eq!(report.dump.matches("----- BEGIN").count(), recipes.len());
}
