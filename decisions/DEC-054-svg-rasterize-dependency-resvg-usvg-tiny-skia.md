---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-054
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent
    - operator

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-009
repo:
  id: crustyimg

created_at: 2026-07-08
supersedes: null
superseded_by: null

affected_scope:
  - Cargo.toml
  - deny.toml
  - src/image/**
  - src/source/**
  - fuzz/**

tags:
  - codecs
  - dependencies
  - licensing
  - svg
  - pure-rust
  - fonts
---

# DEC-054: SVG rasterize dependency — `resvg`/`usvg`/`tiny-skia` (text ON, bundled font) on the default path

## Decision

SVG **rasterize** on the default, pure-Rust build is provided by the permissive
`resvg` stack, feeding the canonical `Image` (the AVIF/webp-lossy precedent,
DEC-053/DEC-021 — a rasterizer, **not** a second pixel library, so
`single-image-library` is not tripped):

- **`resvg` = "=0.47.0"**, `default-features = false, features = ["text"]`
  (Apache-2.0 OR MIT). Drops the default `system-fonts`/`memmap-fonts`/
  `raster-images` features (and the gif/webp/jpeg raster decoders they pull),
  leaving a lean, **all-permissive** tree: `usvg` (Apache-2.0 OR MIT) parses the
  SVG XML to a resolved render tree; `tiny-skia`/`tiny-skia-path` (BSD-3-Clause)
  rasterize it; `fontdb`/`rustybuzz`/`roxmltree`/`svgtypes`/`kurbo`/`data-url`/
  `png`/`flate2`/… are all MIT/Apache/BSD/Zlib. `usvg` + `tiny-skia` arrive
  transitively — **no new top-level dep beyond `resvg`.**

Our glue (`src/image/svg.rs`, kept thin so the stack is swappable) rasterizes at
intrinsic (1x) size and converts the premultiplied RGBA8 pixmap to a straight-
alpha RGBA8 `DynamicImage` (the AVIF `unpremultiply` analog). Rasterization is
dispatched in `decode_with_limits` by a bounded `<svg`/`<?xml` content sniff
**before** the generic `ImageReader` path (which has no SVG support). There is no
`ImageFormat::Svg`, so a rasterized SVG reports `source_format = Png` (its pixels
are now a lossless RGBA raster; `info x.svg` prints `png` — an accepted wart, a
faithful `SourceFormat` enum is a possible follow-up). This also satisfies
`no-new-top-level-deps-without-decision`.

**Text is ON, using the bundled BSD-3 Go font only** (`assets/fonts/Go-Regular.ttf`,
DEC-045), loaded into an explicit `fontdb`; system fonts are **never** enumerated
(`load_system_fonts` is not called), keeping the render deterministic and free of
filesystem/font-enumeration surface. SVG `<text>` is common (icons, logos,
charts) and rendering it to *nothing* is a silent-wrong-output footgun (verified),
so the default family is set to the bundled font.

**Hostile-input hardening** (SVG is untrusted — external refs → local-file read /
SSRF, decompression-bomb viewBoxes): `usvg::Options::resources_dir = None` and the
`image_href_resolver` string resolver returns `None` for every href, so external
`file:`/`http:` references resolve to nothing (data: URIs kept) — no local-file
read, no network. Output dimensions are capped from `usvg::Tree::size()` against
the DEC-034 `Limits` **before** `tiny_skia::Pixmap::new`, so a huge viewBox is
rejected without ever attempting the multi-GiB allocation. A `cargo-fuzz` target
(`fuzz/fuzz_targets/svg_decode.rs`) exercises the parse + raster path.

## Context

crustyimg could not read `.svg` at all: the `image` crate has no SVG support
because SVG is XML/text, not a magic-byte raster format, so it never flows through
the generic decode path. SPEC-060 (STAGE-017, PROJ-009 Wave 1, the second default
input after AVIF) adds a permissive, pure-Rust rasterizer so the default binary
rasterizes `.svg` end to end (optimize/convert/info/resize/batch).

The build-cycle probe (throwaway crate, this machine, 2026-07-08) **corrected the
framing's licensing assumption**: the framing expected an MPL-2.0 exception (à la
`avif-parse`), but the whole `resvg` tree with `default-features = false,
features = ["text"]` is **entirely permissive** — `cargo deny check licenses`
against crustyimg's real allow-list passes with **NO new exception**. The only
supply-chain cost is a single **advisory** ignore: the `text` feature re-adds
`ttf-parser 0.25.1` (via `usvg` → `fontdb` + `rustybuzz`), flagged
**RUSTSEC-2026-0192** (unmaintained; author declared EOL, recommended alt
`skrifa`). The render + security + font API compiled and ran against resvg 0.47.0
(hardened parse, external-ref refusal, cap-before-raster, bundled-font text all
verified); the build cycle re-confirmed the exact API on the pinned version.

## Alternatives Considered

- **Option A: a browser-style external-loading rasterizer (or resvg with default
  features / system fonts).**
  - Why rejected: enabling `system-fonts` enumerates the host's font directories
    (non-deterministic renders, a font-enumeration surface) and `raster-images`
    drags a second set of raster decoders into the default tree. Resolving
    external `href`s at all would open local-file-read / SSRF on untrusted input.
    We deliberately run the *hardened* lean config.

- **Option B: text OFF (`resvg = { default-features = false }`, no `text`).**
  - What it is: removes `ttf-parser` + the RUSTSEC-2026-0192 advisory ignore
    entirely.
  - Why rejected: it silently drops **all** SVG `<text>` — a correctness footgun
    (verified: an empty fontdb renders text to nothing). The asymmetry is stark:
    text-ON costs one *informational* advisory-ignore line; text-OFF costs silent
    wrong output on a common SVG feature.

- **Option C: own the text stack with a `skrifa` text→path pre-pass (no
  ttf-parser, keep text).**
  - Why rejected **for now** (deferred, not built): `skrifa` gives glyph outlines
    but does **not** shape; faithful SVG text (`text-anchor`, `x/y/dx/dy` lists,
    nested `<tspan>`, `letter-spacing`, writing-mode, bidi) means reimplementing a
    large slice of usvg's text module at *lower* fidelity than the upstream
    fontations/HarfRust migration will hand us for free. The advisory ignore is a
    *temporary* carry on a stable leaf upstream is already deleting. This is the
    **opposite** of the AVIF call (DEC-053), where no upstream pure-Rust path
    existed so owning the glue was correct. Build it only if the upstream
    migration stalls AND an independent reason appears (e.g. WASM binary size in
    Wave 3) — then it earns its own spec.

- **Option D (chosen): `resvg`/`usvg`/`tiny-skia` lean + `text`, bundled font,
  hardened, thin glue.**
  - Why selected: pure-Rust, zero system/build-tool deps, **all-permissive** (no
    license exception, one advisory ignore), hostile-input-hardened, text renders
    correctly, and the stack is actively maintained by Linebender (Vello/Kurbo/
    Xilem org) — not abandoned. Only the `ttf-parser` *leaf* is flagged, and the
    ecosystem is migrating font parsing onto fontations (which crustyimg already
    uses via `skrifa`), so the ignore is self-liquidating.

## Consequences

- **Positive:** the default binary rasterizes `.svg` (optimize/convert/info/
  resize/batch) with **no system/build-tool deps**; `just deny` green with **no
  new license exception**; lean `--no-default-features` build unaffected (resvg is
  non-optional, not gated by `display`); SVG `<text>` renders with the bundled
  font. The rasterizer is reusable for the Wave-3 WASM demo.
- **Negative / costs:**
  - A large new dependency tree (`resvg` + `usvg` + `tiny-skia` + font stack) and
    a heavier build. Kept behind a thin glue module so the stack is swappable.
  - **`ttf-parser` (RUSTSEC-2026-0192, unmaintained)** re-enters the tree via the
    text stack — accepted via a documented, dated `deny.toml [advisories] ignore`
    with a revisit trigger (drop when resvg migrates off ttf-parser to
    fontations/HarfRust). Parse-time only, no known vuln. Note this is a *carry*:
    SPEC-044/DEC-045 had *removed* ttf-parser (skrifa/zeno for watermark text);
    SVG text re-adds it transitively.
  - `info x.svg` reports `source_format = png` (no `ImageFormat::Svg`); "preserve
    the source format" is nonsensical for SVG since the output *is* a raster.
  - v1 rasterizes at **intrinsic 1x** — no `--size`/`--scale`/`--dpi` override.
  - **SVGZ (`.svgz`, gzip) is out of scope** for v1: the content sniff keys on
    `<svg`/`<?xml`, and gzip magic (`1f 8b`) won't match at the byte seam. A
    follow-up can sniff gzip magic and hand to usvg (which auto-decompresses).
- **Neutral:** rasterize is dispatched by a content sniff, independent of any
  `image` feature; the metadata lane is untouched (rasterized SVG has no captured
  EXIF/ICC).

## Validation

Right if: default `cargo build` (and `--no-default-features`) rasterize a real
`.svg` with no system libs on all three CI OSes; `just deny` stays green (licenses
clean, only the RUSTSEC-2026-0192 advisory ignored); external refs are refused
(no local-file read / network); a huge viewBox is `LimitsExceeded` before
allocation; the fuzz target finds no panics. Revisit when: (a) resvg's text stack
**migrates off `ttf-parser`** to fontations/read-fonts (HarfRust) — then DROP the
advisory ignore; (b) SVGZ / a `--size`/`--dpi` render override / a faithful
`SourceFormat` enum is wanted (each a follow-up spec); or (c) resvg is abandoned
(pin + thin glue make the swap cheap).

## References

- Related specs: SPEC-060 (this), SPEC-058/DEC-053 (AVIF decode — the default-input
  pattern mirrored here), SPEC-044/DEC-045 (bundled Go font + skrifa/zeno watermark
  text stack that dropped ttf-parser, now re-added transitively).
- Related decisions: DEC-004 (pure-Rust default), DEC-034 (decode caps), DEC-018
  (`no-agpl-default-deps`), DEC-045 (bundled BSD-3 Go font), DEC-053 (AVIF decode
  dependency — the licensing/hardening contrast: MPL exception there, none here).
- Constraints: `pure-rust-codecs-default`, `no-agpl-default-deps`,
  `no-new-top-level-deps-without-decision`, `untrusted-input-hardening`,
  `single-image-library`.
- Advisory: RUSTSEC-2026-0192 (`ttf-parser` unmaintained), ignored in `deny.toml`.
- Upstream: Linebender `resvg`/`usvg`/`tiny-skia`; fontations#956 / resvg#200
  (font-parsing migration onto HarfRust/read-fonts).
