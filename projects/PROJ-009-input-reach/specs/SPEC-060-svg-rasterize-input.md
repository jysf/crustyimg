---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-060
  type: story
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # rasterizer wiring is small; the security config + text/advisory decision + the source_format wrinkle carry the weight

project:
  id: PROJ-009
  stage: STAGE-017
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-08

references:
  decisions: [DEC-004, DEC-034, DEC-018, DEC-045, DEC-054]
  constraints:
    - pure-rust-codecs-default
    - no-agpl-default-deps
    - no-new-top-level-deps-without-decision
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - single-image-library
  related_specs: [SPEC-058, SPEC-044]

value_link: "STAGE-017's 'rasterize `.svg` to the canonical raster Image from the default pure-Rust build' capability."

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-08
      notes: >
        Framing/design cycle — main-loop, not separately metered → null-with-note per AGENTS §4.
        Included a firsthand load-bearing probe (`cargo add resvg` v0.47.0 + `cargo deny` +
        compiling the render/security/font API in a throwaway crate) to verify licenses, the
        dep tree, the render pipeline, the hardening options, and the text/advisory trade-off.
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 185000
      estimated_usd: 1.67
      duration_minutes: 30
      recorded_at: 2026-07-08
      notes: >
        Build cycle run in the main loop (not a separately-metered subagent), so tokens_total is
        an ORDER-OF-MAGNITUDE ESTIMATE, not a null (per the autonomous-run-cost practice + AGENTS §4).
        estimated_usd = 185k tokens × Opus 4.8 list rate ($5/$25 per MTok, ~80/20 input/output, no
        cache discount) ≈ $1.67. Wired src/image/svg.rs + dispatch + IMAGE_EXTENSIONS + deny advisory
        ignore + fixture/tests/fuzz + DEC-054; all gates (default test, lean build, deny, clippy, fmt,
        MSRV) verified green firsthand.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-060: SVG rasterize as a default, pure-Rust input

## Context

crustyimg cannot read `.svg` today: the `image` crate has no SVG support, because
SVG is XML/text, not a magic-byte raster format. This spec adds a **permissive,
pure-Rust rasterizer** (`resvg`/`usvg`/`tiny-skia`) so the default binary
rasterizes `.svg` to the canonical raster `Image` end to end
(`optimize`/`convert`/`info`/`resize`/batch). It is the second default input of
PROJ-009 (roadmap Wave 1), directly mirroring **SPEC-058** (AVIF decode): explicit
content-sniff dispatch in `decode_with_limits`, `.svg` added to `IMAGE_EXTENSIONS`,
typed `ImageError`s, DEC-034 caps enforced *before* allocation, a `cargo-fuzz`
target, and a rasterizer-dependency decision (**DEC-054**) emitted at build.

SVG is a **hostile, untrusted input surface** (external file/URL refs → local-file
read / SSRF, `<script>`, billion-laughs / decompression-bomb viewBoxes). The
load-bearing work of this spec is therefore **security configuration** as much as
the render glue: configure usvg to refuse external resources and scripts, and cap
output dimensions via DEC-034 before rasterizing. See the parent
`STAGE-017-svg-rasterize-input.md` for the stage-level framing and the probe
result.

## Goal

Make the **default** crustyimg build rasterize `.svg` inputs to the canonical
raster `Image` — pure-Rust, zero system deps, `just deny` green — safely against
hostile SVG (external refs refused, output dimensions capped, typed errors not
panics), and admit `.svg` to the source allow-list so batch commands see it.

## Inputs

- **Files to read:**
  - `src/image/mod.rs` — the decode seam: `decode_with_limits` (~L279) and the
    **AVIF dispatch precedent** at ~L288 (`if avif::is_avif(bytes) { … }`), `mod avif;`
    at L29, `decode_limits()` (DEC-034 caps: `MAX_IMAGE_DIMENSION` = 65_535,
    `MAX_ALLOC_BYTES` = 512 MiB), and the `#[cfg(test)] mod tests` AVIF section (~L635).
  - `src/image/avif.rs` — the **pattern to mirror**: `is_avif` (brand sniff),
    `decode_avif`, `check_caps` (cap dims/alloc from metadata *before* allocation),
    premultiplied-alpha handling (`unpremultiply`), typed `map_parse_err`, and the
    module's `## Security` doc-comment.
  - `src/source/mod.rs` — `IMAGE_EXTENSIONS` (~L97, already has `avif`) + `has_image_extension`.
  - `src/error.rs` — `ImageError` (`Decode`, `UnsupportedFormat`, `LimitsExceeded`).
  - `Cargo.toml` — the AVIF dep block (L90+) as the shape to mirror; `[features]` (L106+).
  - `deny.toml` — the `[advisories] ignore` list (L84+) and the per-crate `[licenses] exceptions`
    (L64+); the AVIF entries are the templates to mirror (add an ADVISORY ignore, **not** a license
    exception).
  - `fuzz/Cargo.toml` + `fuzz/fuzz_targets/avif_decode.rs` — the cargo-fuzz target to mirror.
  - `assets/fonts/Go-Regular.ttf` (+ `LICENSE-Go`) — the bundled BSD-3 default font (DEC-045).
- **External APIs:** `resvg` 0.47.0 / `usvg` 0.47.0 / `tiny-skia` 0.12.0 (verified — see
  Implementation Context). Docs: https://docs.rs/resvg , https://docs.rs/usvg , https://docs.rs/tiny-skia .
- **Related code paths:** `src/cli/mod.rs` `output_format_for` (~L1754 — the "preserve
  source_format" default), `src/analysis/decide.rs` (consumes `source_format`), `src/sink/mod.rs`
  (`encode_to_bytes`).

## Outputs

- **Files created:**
  - `src/image/svg.rs` — the rasterize module (`is_svg`, `decode_svg`, cap check, usvg
    hardening, YUV-free straight-RGBA conversion), private to `src/image/`.
  - `tests/input_svg.rs` — integration tests (see Failing Tests).
  - `tests/fixtures/svg/rect_text_40x30.svg` — a tiny plain-text SVG fixture (a rect + a
    `<text>` element; committed as text — no encoder feature or ImageMagick needed, unlike AVIF).
  - `fuzz/fuzz_targets/svg_decode.rs` — a cargo-fuzz target on the SVG decode path.
  - `decisions/DEC-054-*.md` — the rasterizer-dependency decision (emitted during build).
- **Files modified:**
  - `Cargo.toml` — add `resvg = { version = "=0.47.0", default-features = false, features = ["text"] }`
    (usvg + tiny-skia arrive transitively). Non-optional (default path); no new feature flag.
  - `src/image/mod.rs` — `mod svg;`; dispatch `if svg::is_svg(bytes) { return … }` in
    `decode_with_limits` before the generic `ImageReader` path; add the `#[cfg(test)]` SVG unit tests.
  - `src/source/mod.rs` — add `"svg"` to `IMAGE_EXTENSIONS` (update the block comment).
  - `deny.toml` — add a `RUSTSEC-2026-0192` **advisory ignore** (ttf-parser unmaintained, via the
    usvg text stack). **No `[licenses] exceptions` change** — the whole resvg tree is permissive.
  - `fuzz/Cargo.toml` — add the `svg_decode` `[[bin]]` + a `Seed:` line to `tests/fixtures/svg`.
- **New exports:** none required — SVG flows through the existing `Image::from_bytes`/`load`.
  Keep `is_svg`/`decode_svg` `pub(crate)` inside `src/image/` (mirror `avif`).

## Acceptance Criteria

- [ ] In the **default** build (`cargo build`, no extra features), `Image::load("*.svg")` and
  `Image::from_bytes(svg_bytes)` return a decoded raster `Image` whose dimensions equal the SVG's
  intrinsic size (its `width`/`height`, else its `viewBox`) at 1x.
- [ ] The decode honors the DEC-034 caps: an SVG declaring a `width`/`height`/`viewBox` above the
  dimension or allocation cap yields `ImageError::LimitsExceeded` — checked from `usvg::Tree::size()`
  **before** `tiny_skia::Pixmap::new` (a 100000×100000 SVG must not attempt a ~38 GiB allocation).
- [ ] A malformed / truncated / non-SVG-but-SVG-sniffed byte stream yields `ImageError::Decode(_)`
  (typed, no panic / `unwrap`).
- [ ] **Hostile-input safety:** an SVG referencing an external file or URL
  (`<image href="file:///etc/passwd">` / `xlink:href="/etc/passwd"` / `href="http://…"`) rasterizes
  WITHOUT reading that file or making a network request — the external reference resolves to nothing
  (transparent), and decode still returns `Ok`.
- [ ] `.svg` is in `IMAGE_EXTENSIONS`; a directory/glob source containing an `.svg` includes it (and
  a non-image `.txt` sibling is excluded).
- [ ] `optimize <fixture>.svg -o out.png` exits 0 and writes a valid PNG with the fixture's intrinsic
  dimensions in the default build; `convert <fixture>.svg -o out.webp` exits 0 and writes a valid WebP.
- [ ] SVG `<text>` renders using the **bundled Go font** (no system fonts loaded); a text fixture
  decodes to a non-empty raster (text is not silently dropped).
- [ ] **No C/system dependency on the default path**; `cargo build --no-default-features` (lean) still
  succeeds; `just deny` is green — the resvg tree is all-permissive (**no new license exception**),
  with a single documented `RUSTSEC-2026-0192` advisory ignore for the ttf-parser unmaintained status.
- [ ] A `fuzz/svg_decode` cargo-fuzz target compiles against the public API (`Image::from_bytes`).
- [ ] `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` clean.

## Failing Tests

Written during **design**, BEFORE build. Build makes these pass.

> **Fixture:** `tests/fixtures/svg/rect_text_40x30.svg` — a plain-text SVG with `width='40'
> height='30' viewBox='0 0 40 30'`, a filled `<rect>`, a semi-transparent `<circle>`, and a `<text>`
> element (to exercise the bundled-font path). Committed verbatim as text — no encoder feature needed
> (contrast AVIF's binary fixture). Small inline `const` SVG byte slices are also used in unit tests.

- **`src/image/svg.rs`** (in a new `#[cfg(test)] mod tests`)
  - `"is_svg_detects_svg_and_xml_prolog"` — `<svg …>`, `  <svg>` (leading ws), and
    `<?xml version='1.0'?><svg>` → `true`; a PNG signature, an AVIF `ftyp`, and a non-`<svg>` XML
    doc (`<html>`) → `false`.
  - `"svg_dimension_cap_rejects_before_raster"` — a synthetic SVG declaring `width='100000'
    height='100000'` → `decode_svg(bytes, &tiny_limits)` (or production limits) → `Err(ImageError::LimitsExceeded(_))`,
    never an allocation attempt / panic.
  - `"corrupt_svg_bytes_are_decode_error_not_panic"` — `<svg` truncated / `not an svg` sniffed as SVG →
    `Err(ImageError::Decode(_))`.
- **`src/image/mod.rs`** (in the existing `#[cfg(test)] mod tests`, an `── SPEC-060 SVG` section)
  - `"svg_decodes_to_intrinsic_dimensions"` — `Image::from_bytes(b"<svg … width='40' height='30' …>")`
    → `Ok`, `width()==40 && height()==30`.
  - `"svg_uses_viewbox_when_no_width_height"` — an SVG with only `viewBox='0 0 100 50'` → `100×50`.
  - `"oversize_svg_is_limits_exceeded"` — an SVG declaring `width='70000'` (> `MAX_IMAGE_DIMENSION`)
    → `Image::from_bytes` → `Err(ImageError::LimitsExceeded(_))` (via `decode_with_limits` production caps).
  - `"malformed_svg_is_decode_error_not_panic"` — `b"<svg xmlns='…'><rect"` (unclosed) → `Err(ImageError::Decode(_))`.
  - `"svg_external_file_ref_is_ignored"` — an SVG with `<image href="file:///etc/hostname" width='10'
    height='10'/>` over a colored background → `Image::from_bytes` is `Ok`, dimensions match, and the
    referenced file is NOT loaded (the region is background/transparent, proving `resolve_string`
    refused the external href — no local-file read).
- **`tests/input_svg.rs`** (integration, default build)
  - `"optimize_svg_input_writes_png"` — run `optimize` on the fixture to a temp `.png` → exit 0, output
    decodes as PNG with the fixture's dimensions (40×30).
  - `"convert_svg_to_webp"` — run `convert`/`optimize` on the fixture to a temp `.webp` → exit 0, output
    decodes as WebP with the fixture's dimensions.
  - `"directory_source_discovers_svg"` — a temp dir with `a.svg` (+ a `.txt`) → `source::resolve` returns
    exactly the `.svg`.
  - `"svg_text_uses_bundled_font"` — rasterize the text fixture and assert the output is a non-empty
    raster of the expected dims (the `<text>` path is exercised with the bundled Go font; no system fonts).

## Implementation Context

*Read this section (and the files it points to) before starting build. Everything below the "PROBE"
heading was verified firsthand during design in a throwaway crate (resvg 0.47.0, `cargo deny`, and a
compiling render/security/font program) — trust it, but re-confirm the exact API against the pinned
version at build.*

### Decisions that apply
- `DEC-004` — pure-Rust codecs by default; the rasterizer **must be pure-Rust** on the default path
  (resvg/usvg/tiny-skia are pure-Rust, no C/nasm). Verified.
- `DEC-034` — decode resource caps; enforce dimension/alloc caps from `usvg::Tree::size()` **before**
  `Pixmap::new`, mirroring `avif::check_caps`. Reuse the existing `Limits` passed into `decode_with_limits`.
- `DEC-018` / `no-agpl-default-deps` — the resvg tree is entirely permissive (verified); **no AGPL/LGPL,
  no license exception needed**. The only deny.toml delta is an *advisory* ignore.
- `DEC-045` — the bundled BSD-3 Go font (`assets/fonts/Go-Regular.ttf`) used for watermark text; reuse
  it as the SVG default font (embed via `include_bytes!`).
- **`DEC-054` (NEW — emit during build)** — records the rasterizer choice: the crate set + pinned
  versions, that the whole tree is permissive (no license exception), the `default-features = false,
  features = ["text"]` lean config, the security configuration (resources_dir/href-resolver/caps), the
  RUSTSEC-2026-0192 advisory trade-off, and why resvg beats a browser-style external-loading rasterizer.
  Satisfies `no-new-top-level-deps-without-decision`.

### PROBE — verified firsthand (2026-07-08), the resvg 0.47 render + security + font pipeline

**Dependency + license (authoritative — corrects the framing's MPL-2.0 expectation):**
- `resvg = { version = "=0.47.0", default-features = false, features = ["text"] }`. This drops the
  default `system-fonts`, `memmap-fonts`, and `raster-images` features (and the gif/image-webp/zune-jpeg
  decoders they pull), leaving a lean tree of **all-permissive** crates: `resvg`/`usvg` `Apache-2.0 OR
  MIT`; `tiny-skia`/`tiny-skia-path` `BSD-3-Clause`; `fontdb` MIT, `rustybuzz` MIT, `ttf-parser` `MIT OR
  Apache-2.0`, `roxmltree` MIT/Apache, `svgtypes`/`kurbo`/`simplecss`/`data-url`/`base64`/`png`/`flate2`
  all MIT/Apache/BSD/Zlib. **`cargo deny check licenses` against crustyimg's real allow-list passes with
  NO new exception.**
- **Advisory:** the `text` feature pulls `ttf-parser 0.25.1` (via `usvg`→`fontdb`+`rustybuzz`), which
  `cargo deny check advisories` flags as **RUSTSEC-2026-0192** (*unmaintained*; author declared EOL;
  recommended alt `skrifa` — which the repo already uses for watermarks, but resvg's stack cannot be
  swapped to it without an upstream migration). Add to `deny.toml [advisories] ignore`:
  `{ id = "RUSTSEC-2026-0192", reason = "ttf-parser unmaintained; transitive via usvg/resvg text stack (SPEC-060/DEC-054); parse-time only, no vuln. Revisit + DROP when resvg's text stack migrates off ttf-parser to fontations/read-fonts (HarfRust) — we already sit on fontations via skrifa" }`.
  *Verified:* dropping the `text` feature removes ttf-parser entirely (no ignore needed) — see the
  fonts/text decision below.
- **Maintenance context (checked 2026-07-08 — matters for the decision):** the rasterizer stack itself
  is **actively maintained by Linebender** (Vello/Kurbo/Xilem org; v0.47.0 Feb 2026) — NOT abandoned.
  Only the `ttf-parser` *leaf* is flagged, and the ecosystem is migrating font parsing onto **fontations**
  (`read-fonts`/`skrifa`, via HarfRust — fontations#956, resvg font issue #200), which crustyimg already
  uses. So the advisory ignore is a *temporary* carry on a stable leaf that upstream is deleting for us —
  it is NOT a bet on a dead stack.

**Render pipeline (compiles + runs):**
```rust
use resvg::{tiny_skia, usvg};
// 1. Hardened parse options
let mut opt = usvg::Options::default();
opt.resources_dir = None;                       // no filesystem resolution of relative paths
opt.image_href_resolver = usvg::ImageHrefResolver {
    resolve_data: usvg::ImageHrefResolver::default_data_resolver(), // keep data: URIs
    resolve_string: Box::new(|_href, _opts| None),                  // refuse file/URL hrefs
};
// (text) load ONLY the bundled font, no system fonts:
let mut db = usvg::fontdb::Database::new();
db.load_font_data(include_bytes!("../../assets/fonts/Go-Regular.ttf").to_vec());
opt.font_family = "Go".to_string();
opt.fontdb = std::sync::Arc::new(db);
// 2. Parse (typed error, never panics on malformed input — verified on garbage/truncated/empty)
let tree = usvg::Tree::from_data(bytes, &opt).map_err(|e| ImageError::Decode(format!("svg: {e}")))?;
// 3. Intrinsic size (width/height, else viewBox) — CAP against `limits` BEFORE allocating
let size = tree.size();
let w = size.width().ceil() as u32;
let h = size.height().ceil() as u32;
check_caps(w, h, limits)?;                       // mirror avif::check_caps (dim + w*h*4 alloc)
// 4. Rasterize
let mut pixmap = tiny_skia::Pixmap::new(w, h)
    .ok_or_else(|| ImageError::Decode("svg: zero/invalid raster size".into()))?;
resvg::render(&tree, tiny_skia::Transform::identity(), &mut pixmap.as_mut());
// 5. tiny-skia stores PREMULTIPLIED RGBA8 — demultiply to STRAIGHT alpha (the AVIF unpremultiply analog)
let straight: Vec<u8> = pixmap.pixels().iter()
    .flat_map(|p| { let c = p.demultiply(); [c.red(), c.green(), c.blue(), c.alpha()] })
    .collect();
let buf = ::image::RgbaImage::from_raw(w, h, straight)
    .ok_or_else(|| ImageError::Decode("svg: raster buffer size mismatch".into()))?;
Ok(DynamicImage::ImageRgba8(buf))
```
Notes proven by the probe: malformed/truncated/empty → typed `Err` (never panic); a 100000×100000 SVG
**parses fine** (usvg allocates no pixels), so the cap in step 3 is what stops the bomb — 100000 >
`MAX_IMAGE_DIMENSION` (65_535), so the dimension cap alone rejects it. `resolve_string = |_,_| None`
refuses external hrefs. Text with an empty fontdb renders to **nothing** (`root().has_children() == false`);
loading the bundled Go font + `font_family = "Go"` makes it render (`has_children == true`).

### Fonts / text decision (v1)
- **v1 = text ON, bundled Go font only, no system fonts.** Rationale: SVG `<text>` is common (icons,
  logos, charts) and rendering it to *nothing* is a silent-wrong-output footgun (verified). The repo
  already bundles `Go-Regular.ttf` (BSD-3, DEC-045); embed it via `include_bytes!` and set it as the
  default family. Do **not** call `Database::load_system_fonts()` — keep the render deterministic and
  free of filesystem/font-enumeration surface. Cost: the RUSTSEC-2026-0192 advisory ignore above
  (unmaintained ttf-parser; parse-time; well-precedented by the paste/RUSTSEC-2024-0436 ignore).
- **Alternatives weighed and NOT chosen for v1 (recorded so the trade-off is explicit and reversible):**
  - *Text OFF* (`resvg = { default-features = false }`, no `text`) removes ttf-parser + the advisory
    ignore entirely, but silently drops all SVG text. Rejected on correctness grounds.
  - *`skrifa` text→path pre-pass* (own the text stack, no ttf-parser, keep text) — **deferred, do NOT
    build now.** `skrifa` gives glyph outlines but does NOT shape; faithful SVG text layout
    (`text-anchor`, `x/y/dx/dy` lists, nested `<tspan>`, `letter-spacing`, writing-mode, bidi) means
    reimplementing a large slice of usvg's text module at *lower* fidelity than the upstream
    fontations/HarfRust migration will hand us for free (see the maintenance context above). The
    asymmetry: text-ON costs one informational advisory-ignore line for as long as the migration takes;
    the pre-pass costs weeks of engineering + a worse text path + ongoing maintenance. Build it ONLY if
    the upstream migration stalls AND an independent reason appears (e.g. WASM binary-size in Wave 3) —
    at which point it may warrant its own spec (and a possible upstream contribution, mirroring the
    image-rs AVIF one). This is the opposite of the AVIF call, where no upstream pure-Rust path existed
    so owning the glue was correct.

### `source_format` wrinkle (no `ImageFormat::Svg`)
`image::ImageFormat` has no `Svg` variant, and "preserving" SVG as an output format is nonsensical.
After rasterization the `Image` *is* an RGBA8 raster, so set `source_format = ImageFormat::Png` for a
rasterized SVG (lossless-RGBA target; also the sensible default when `convert`/`optimize` gets no
explicit `-o`/`--format`). Return it from the `svg` dispatch arm just like the AVIF arm returns
`ImageFormat::Avif`. Consequence: `info x.svg` reports `png`, not `svg` — an accepted wart. A faithful
`SourceFormat` enum that can name non-`image` inputs is a possible follow-up refactor (out of scope).

### Wiring (mirror AVIF exactly)
- `is_svg(bytes)`: skip a leading UTF-8/UTF-16 BOM + ASCII whitespace, then return `true` if the stream
  starts with `<svg` (case-insensitive) OR starts with an XML prolog `<?xml` whose following non-ws
  content reaches a `<svg` root within a bounded window (cap the scan, e.g. first ~1 KiB — do not scan
  unbounded). Reject anything else (PNG/JPEG/AVIF/`<html>`). Keep it allocation-free.
- In `decode_with_limits`, add `if svg::is_svg(bytes) { return Ok((svg::decode_svg(bytes, limits)?, ImageFormat::Png)); }`
  alongside the existing AVIF branch, **before** the generic `ImageReader` path (which cannot detect SVG).
- Add `"svg"` to `IMAGE_EXTENSIONS` in `src/source/mod.rs` and update the block comment (SPEC-060/DEC-054).
- **SVGZ (`.svgz`, gzip) is OUT of scope for v1:** usvg *can* auto-decompress gzip, but our content sniff
  keys on `<svg`/`<?xml` and gzip magic (`1f 8b`) won't match at the byte-sniff seam. Note it as a
  follow-up (sniff gzip magic → hand to usvg).

### Constraints that apply
- `pure-rust-codecs-default`, `no-agpl-default-deps`, `no-new-top-level-deps-without-decision`,
  `untrusted-input-hardening` (SVG is hostile untrusted input — refuse external resources/scripts/network,
  cap dimensions before raster, no-panic typed errors, add the `cargo-fuzz` target),
  `no-unwrap-on-recoverable-paths`, `every-public-fn-tested`, `clippy-fmt-clean`,
  `single-image-library` (resvg is a rasterizer feeding the canonical `Image`, NOT a second pixel
  library — the AVIF/webp-lossy precedent; do not route any pixel op through it).

### Prior related work
- `SPEC-058` / `DEC-053` — AVIF decode as a default input: the **exact pattern to mirror** (content
  dispatch in `decode_with_limits`, `IMAGE_EXTENSIONS`, `check_caps`-before-alloc, premultiplied-alpha
  handling, typed errors, cargo-fuzz target, dep DEC, deny.toml delta). `src/image/avif.rs`.
- `SPEC-044` / `DEC-045` — the bundled Go font + `skrifa`/`zeno` watermark text stack (why ttf-parser
  was dropped, and why re-adding it via resvg needs the advisory ignore).

### Out of scope (for this spec specifically)
- SVG **output** / an SVG encoder; SVGZ (`.svgz`); SMIL animation; a `--size`/`--scale`/`--dpi`
  render override (v1 rasterizes at intrinsic 1x); system-font loading; a faithful `SourceFormat` enum;
  AVIF/RAW/HEIC inputs (other stages).

## Notes for the Implementer

- Verify the **lean build** (`cargo build --no-default-features`) AND `just deny` as part of build, not
  just at verify — resvg is non-optional (default path), so both must stay green (`verify-includes-lean-build`).
- Run `just deny` **immediately after `cargo add resvg`** (the AVIF lesson): confirm the license check is
  clean (it should need no exception) and surface the RUSTSEC-2026-0192 advisory up front so the ignore
  lands before you write the module.
- Keep `src/image/svg.rs` thin and off the pixel core's public surface (mirror `avif`); the rasterizer
  stays inside `src/image/`.
- Cap dimensions from `tree.size()` **before** `Pixmap::new` — a huge viewBox parses fine and would
  otherwise attempt a multi-GiB allocation. Reuse `avif::check_caps`'s shape (dim caps + `w*h*4` alloc).
- Confirm the exact resvg 0.47 API against docs.rs at build (field names on `usvg::Options`, the
  `ImageHrefResolver` fn signature, `tiny_skia::PremultipliedColorU8::demultiply`) — the probe compiled
  against 0.47.0 but pin + re-verify.
- MSRV: check whether resvg/usvg raise the floor above the current 1.90 (avif-parse) — let the CI `msrv`
  job confirm and bump `rust-version` + the hardcoded `ci.yml` msrv toolchain if needed (the AVIF lesson).
- Fixture is plain text — no `--features`/regen dance. Commit `tests/fixtures/svg/rect_text_40x30.svg`
  directly and seed the fuzz corpus from `tests/fixtures/svg`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-060-svg-rasterize`
- **PR (if applicable):** #66
- **All acceptance criteria met?** yes
  - Default-build `.svg` rasterize (intrinsic width/height, else viewBox) → canonical RGBA `Image`,
    `source_format = Png`. DEC-034 caps enforced from `usvg::Tree::size()` **before** `Pixmap::new`.
    Malformed/truncated → typed `Decode`. External file/URL hrefs refused (no local-file read/network),
    decode still `Ok`. `.svg` in `IMAGE_EXTENSIONS`; dir/glob discovery works. `optimize .svg → .png`
    and `convert .svg → .webp` exit 0 with correct dims. `<text>` renders with the bundled Go font
    (no system fonts). `just deny` green — **no license exception**, one `RUSTSEC-2026-0192` advisory
    ignore. `fuzz/svg_decode` target added. Lean `--no-default-features` build + clippy + fmt clean.
- **New decisions emitted:**
  - `DEC-054` — SVG rasterize dependency (`resvg`/`usvg`/`tiny-skia`, text ON + bundled font, hardened)
- **Deviations from spec:**
  - `convert` requires an explicit `--format` (it does not infer from the `-o` extension), so the
    `convert_svg_to_webp` integration test passes `--format webp` (the spec's example omitted it). No
    behavior change — purely the test invocation.
  - `is_svg` gained a small tag-boundary check (`<svg` must be followed by whitespace/`>`/`/`/EOF) so
    `<svgfoo>` is not mis-sniffed — a tightening within the spec's "starts with `<svg`" intent.
- **Follow-up work identified:**
  - SVGZ (`.svgz`, gzip) input: sniff gzip magic (`1f 8b`) → hand to usvg (auto-decompresses). Noted
    out-of-scope in DEC-054; a small follow-up spec.
  - A faithful `SourceFormat` enum that can name non-`image` inputs (so `info x.svg` reports `svg`,
    not `png`) — cross-cutting refactor, out of scope here.
  - A `--size`/`--scale`/`--dpi` SVG render override (v1 rasterizes at intrinsic 1x).

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Almost nothing — the Implementation Context's verified probe (the exact render/security/font code,
   the licensing correction, the deny.toml delta) was a near-drop-in handoff. The one small gap was the
   `convert` CLI needing an explicit `--format` (surfaced by a failing test, fixed in seconds).
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The referenced set (DEC-004/034/018/045/054 + the untrusted-input + single-image-library
   constraints) was complete. The probe having already resolved the MPL-vs-permissive question up front
   is what made the license step a non-event.
3. **If you did this task again, what would you do differently?**
   — Run `just deny` immediately after `cargo add` (I did) — that front-loaded the advisory before any
   module code, exactly as the AVIF lesson prescribes. Nothing I'd change; the mirror-AVIF discipline
   made this fast.

---

## Reflection (Ship)

1. **What would I do differently next time?**
   — <answer>
2. **Does any template, constraint, or decision need updating?**
   — <answer>
3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
