#![cfg(not(target_arch = "wasm32"))]
//! SVG `<text>` in any `font-family` draws in the bundled Go font.
//!
//! Every test drives the real binary (`convert X.svg --format png`) and reads
//! the output PNG with the upstream `image` decoder, so no crustyimg decode
//! code decides what counts as ink. SVGs are generated inline.

use std::path::Path;
use std::process::Command;
use std::sync::Arc;

use resvg::{tiny_skia, usvg};
use sha2::{Digest, Sha256};

const BIN: &str = env!("CARGO_BIN_EXE_crustyimg");

/// SHA-256 over width (LE u32), height (LE u32) and the RGBA8 samples of the
/// "Hello" render in the bundled font. Computed outside crustyimg.
const GO_HELLO_PIXELS_SHA256: &str =
    "6e4d6b080052173623550a30e4e877658f5da28ddb711fbfad7bc46df2d46ad6";

/// Pixels with alpha > 0 in the "Hello" render.
const GO_HELLO_INK: usize = 738;

/// Attributes every test `<text>` carries.
const T: &str = r#"x="10" y="40" font-size="32""#;

fn svg_of(body: &str) -> String {
    format!(r#"<svg xmlns="http://www.w3.org/2000/svg" width="200" height="60">{body}</svg>"#)
}

/// `<text>` carrying `attrs`, with `Hello` inside.
fn hello(attrs: &str) -> String {
    svg_of(&format!("<text {T} {attrs}>Hello</text>"))
}

struct Render {
    status: Option<i32>,
    stderr: String,
    rgba: image::RgbaImage,
}

impl Render {
    fn ink(&self) -> usize {
        self.rgba.pixels().filter(|p| p.0[3] > 0).count()
    }

    fn pixel_hash(&self) -> String {
        let mut h = Sha256::new();
        h.update(self.rgba.width().to_le_bytes());
        h.update(self.rgba.height().to_le_bytes());
        h.update(self.rgba.as_raw());
        h.finalize().iter().map(|b| format!("{b:02x}")).collect()
    }
}

/// Run `convert` on `svg` and decode the PNG it wrote.
fn render(dir: &Path, name: &str, svg: &str) -> Render {
    let input = dir.join(format!("{name}.svg"));
    let output = dir.join(format!("{name}.png"));
    std::fs::write(&input, svg).unwrap();
    let out = Command::new(BIN)
        .args([
            "convert",
            input.to_str().unwrap(),
            "--format",
            "png",
            "-o",
            output.to_str().unwrap(),
            "-y",
        ])
        .output()
        .expect("failed to run convert");
    let rgba = match image::open(&output) {
        Ok(img) => img.to_rgba8(),
        Err(e) => panic!(
            "{name}: no readable PNG ({e}); status {:?}, stderr {}",
            out.status.code(),
            String::from_utf8_lossy(&out.stderr)
        ),
    };
    Render {
        status: out.status.code(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        rgba,
    }
}

fn assert_clean_exit(name: &str, r: &Render) {
    assert_eq!(r.status, Some(0), "{name}: exit code");
    assert!(r.stderr.is_empty(), "{name}: stderr was {:?}", r.stderr);
}

/// Every family form that previously drew nothing now draws exactly what
/// `font-family="Go"` draws.
#[test]
fn every_family_form_draws_the_go_render() {
    let dir = tempfile::tempdir().unwrap();
    let go = render(dir.path(), "go_reference", &hello(r#"font-family="Go""#));
    assert_eq!(
        go.ink(),
        GO_HELLO_INK,
        "the Go reference render lost its ink"
    );

    let rows: Vec<(&str, String)> = vec![
        ("sans-serif", hello(r#"font-family="sans-serif""#)),
        ("serif", hello(r#"font-family="serif""#)),
        ("monospace", hello(r#"font-family="monospace""#)),
        ("cursive", hello(r#"font-family="cursive""#)),
        ("fantasy", hello(r#"font-family="fantasy""#)),
        ("Helvetica", hello(r#"font-family="Helvetica""#)),
        ("Arial", hello(r#"font-family="Arial""#)),
        ("DejaVu Sans", hello(r#"font-family="DejaVu Sans""#)),
        ("NoSuchFontXyz", hello(r#"font-family="NoSuchFontXyz""#)),
        (
            "list without Go",
            hello(r#"font-family="Helvetica, sans-serif""#),
        ),
        (
            "quoted list",
            hello(r#"font-family="'Helvetica Neue', Arial, sans-serif""#),
        ),
        (
            "style attribute",
            hello(r#"style="font-family: sans-serif""#),
        ),
        (
            "style element",
            svg_of(&format!(
                "<style>text{{font-family:Arial}}</style><text {T}>Hello</text>"
            )),
        ),
        ("lowercase go", hello(r#"font-family="go""#)),
        (
            "inherited",
            svg_of(&format!(
                r#"<g font-family="sans-serif"><text {T}>Hello</text></g>"#
            )),
        ),
        (
            "sans-serif bold",
            hello(r#"font-family="sans-serif" font-weight="bold""#),
        ),
        (
            "serif italic",
            hello(r#"font-family="serif" font-style="italic""#),
        ),
        (
            "mixed tspan",
            svg_of(&format!(
                r#"<text {T} font-family="Arial">Hel<tspan font-family="Go">lo</tspan></text>"#
            )),
        ),
    ];
    assert_eq!(rows.len(), 18, "the table lost or gained a row");

    for (name, svg) in &rows {
        let r = render(dir.path(), &name.replace(' ', "_"), svg);
        assert_clean_exit(name, &r);
        assert_eq!(r.ink(), GO_HELLO_INK, "{name}: ink");
        assert!(
            r.rgba.as_raw() == go.rgba.as_raw(),
            "{name}: pixels differ from the Go render"
        );
        assert_eq!(
            r.pixel_hash(),
            GO_HELLO_PIXELS_SHA256,
            "{name}: pinned hash"
        );
    }
}

/// The forms that drew before the fix keep their exact pixels.
#[test]
fn working_family_forms_keep_their_pinned_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let rows: Vec<(&str, String)> = vec![
        ("no family", hello("")),
        ("Go", hello(r#"font-family="Go""#)),
        ("list with Go", hello(r#"font-family="Helvetica, Go""#)),
        ("Go bold", hello(r#"font-family="Go" font-weight="bold""#)),
        (
            "Go italic",
            hello(r#"font-family="Go" font-style="italic""#),
        ),
        (
            "Go tspan in Go text",
            svg_of(&format!(
                r#"<text {T} font-family="Go">Hel<tspan font-family="Go">lo</tspan></text>"#
            )),
        ),
    ];
    assert_eq!(rows.len(), 6, "the table lost or gained a row");

    for (name, svg) in &rows {
        let r = render(dir.path(), &name.replace(' ', "_"), svg);
        assert_clean_exit(name, &r);
        assert_eq!(r.ink(), GO_HELLO_INK, "{name}: ink");
        assert_eq!(
            r.pixel_hash(),
            GO_HELLO_PIXELS_SHA256,
            "{name}: pinned hash"
        );
    }
}

/// The ink counter's negative control: the same SVGs with no text draw nothing.
#[test]
fn text_free_svgs_draw_no_ink() {
    let dir = tempfile::tempdir().unwrap();
    let rows: Vec<(&str, String)> = vec![
        ("empty text", svg_of(&format!("<text {T}></text>"))),
        (
            "empty text with family",
            svg_of(&format!(r#"<text {T} font-family="sans-serif"></text>"#)),
        ),
        (
            "space only",
            svg_of(&format!(r#"<text {T} font-family="sans-serif"> </text>"#)),
        ),
    ];
    assert_eq!(rows.len(), 3, "the table lost or gained a row");

    for (name, svg) in &rows {
        let r = render(dir.path(), &name.replace(' ', "_"), svg);
        assert_clean_exit(name, &r);
        assert_eq!(r.ink(), 0, "{name}: ink");
    }
}

/// A font file the host OS ships, with the family name it answers to.
/// `None` means no candidate exists on this host.
fn host_font() -> Option<(&'static str, &'static str)> {
    let candidates: &[(&str, &str)] = if cfg!(target_os = "macos") {
        &[("Helvetica", "/System/Library/Fonts/Helvetica.ttc")]
    } else if cfg!(target_os = "windows") {
        &[("Arial", r"C:\Windows\Fonts\arial.ttf")]
    } else {
        &[
            (
                "DejaVu Sans",
                "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            ),
            (
                "Liberation Sans",
                "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            ),
            (
                "Noto Sans",
                "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            ),
        ]
    };
    let found = candidates
        .iter()
        .copied()
        .find(|(_, path)| Path::new(path).exists());
    if found.is_none() && (cfg!(target_os = "macos") || cfg!(target_os = "windows")) {
        panic!("expected {candidates:?} to exist on this OS");
    }
    found
}

/// Alpha channel of `svg` rendered by resvg with only `font` loaded.
fn alpha_with_only_font(svg: &str, font: Vec<u8>, family: &str) -> Vec<u8> {
    let mut db = usvg::fontdb::Database::new();
    db.load_font_data(font);
    let opt = usvg::Options {
        font_family: family.to_string(),
        fontdb: Arc::new(db),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_data(svg.as_bytes(), &opt).expect("parse test svg");
    let mut pixmap = tiny_skia::Pixmap::new(200, 60).unwrap();
    resvg::render(
        &tree,
        tiny_skia::Transform::identity(),
        &mut pixmap.as_mut(),
    );
    pixmap.data().chunks_exact(4).map(|px| px[3]).collect()
}

/// A font installed on the host is never read: naming it still draws Go.
#[test]
fn an_installed_system_font_is_never_read() {
    let Some((family, path)) = host_font() else {
        eprintln!(
            "SKIP an_installed_system_font_is_never_read: no known font file on this host; \
             checked: DejaVu Sans, Liberation Sans, Noto Sans"
        );
        return;
    };
    eprintln!("an_installed_system_font_is_never_read: using {family} from {path}");

    let dir = tempfile::tempdir().unwrap();
    let svg = hello(&format!(r#"font-family="{family}""#));
    let go = render(dir.path(), "go_reference", &hello(r#"font-family="Go""#));
    let go_alpha: Vec<u8> = go.rgba.pixels().map(|p| p.0[3]).collect();

    // Positive control: the installed face draws ink that differs from Go's,
    // so a leak of the system font would be visible below.
    let font_bytes = std::fs::read(path).unwrap();
    let own_alpha = alpha_with_only_font(&svg, font_bytes, family);
    assert!(
        own_alpha.iter().any(|&a| a > 0),
        "{family}: the control render drew no ink"
    );
    assert!(
        own_alpha != go_alpha,
        "{family}: draws the same alpha as Go, so this host cannot tell them apart"
    );

    let r = render(dir.path(), "installed", &svg);
    assert_clean_exit(family, &r);
    assert_eq!(
        r.pixel_hash(),
        GO_HELLO_PIXELS_SHA256,
        "{family}: pinned hash"
    );
}
