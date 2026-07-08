---
# Maps to ContextCore epic-level conventions.
stage:
  id: STAGE-017
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-009
repo:
  id: crustyimg

created_at: 2026-07-08
shipped_at: null

value_contribution:
  advances: >
    Extends the project's "read the modern-format assets developers actually
    have" thesis to VECTOR input: rasterize `.svg` to the canonical raster
    `Image` on the default, pure-Rust, zero-system-dep build, so every shipped
    command (optimize / convert / info / resize / batch) applies to SVG sources —
    the second default input format after AVIF (STAGE-016).
  delivers:
    - "`.svg` rasterizes to the canonical `Image` in the DEFAULT build (resvg, no system libs, no C/nasm)"
    - "`.svg` is discovered by directory/glob sources and flows through optimize/convert/info/resize"
    - "A hostile SVG (external file/URL ref, script, decompression-bomb viewBox) is handled safely — external refs refused, output dimensions capped (DEC-034), typed errors not panics"
    - "A recorded rasterizer-dependency decision (DEC-054) proving the pure-Rust, permissive resvg stack"
  explicitly_does_not:
    - "Emit SVG *output* (this wave reads; no SVG encoder)"
    - "Rasterize SVGZ (gzip `.svgz`), animated SMIL, or CSS/script-driven dynamic SVG beyond what usvg statically resolves"
    - "Add a `--size`/`--scale`/`--dpi` render-override flag (v1 rasterizes at the SVG's intrinsic size at 1x; override is a possible follow-up)"
    - "Load system fonts (deterministic bundled-font-only text; no OS font enumeration)"
    - "Pull dav1d or any C/system dependency onto the default path"
---

# STAGE-017: SVG rasterize as a default, pure-Rust input

## What This Stage Is

The stage that lets the default crustyimg binary **read SVG**. Today `.svg` is an
unknown input (the `image` crate has no SVG support — SVG is XML/text, not a
magic-byte raster). This stage adds a **permissive, pure-Rust rasterizer**
(`resvg`/`usvg`/`tiny-skia`, Apache-2.0 OR MIT / BSD-3-Clause) so
`crustyimg optimize icon.svg -o icon.png` (and `convert`, `info`, `resize`,
batch) just work in the default build with no system libraries and no C/nasm.
SVG is an **untrusted, hostile input surface** (external-file/URL refs, scripts,
billion-laughs/decompression-bomb viewBoxes), so the load-bearing work is
**security configuration** (disable external resource loading, refuse script/
network, cap output dimensions via DEC-034 *before* rasterizing) as much as the
render glue. This is the second default input of PROJ-009, mirroring STAGE-016
(AVIF): explicit content-sniff dispatch in `decode_with_limits`, `.svg` added to
`IMAGE_EXTENSIONS`, typed errors, DEC-034 caps, a cargo-fuzz target, and a dep
DEC emitted at build.

## Why Now

- **Second-highest-leverage default input.** After AVIF (STAGE-016), SVG is the
  most common modern asset in a web/content developer's tree (icons, logos,
  charts). Rasterizing it lets the shipped optimize/convert engine turn vector
  assets into raster derivatives with zero setup — a concrete "it just works"
  win with a fully permissive, patent-clean, pure-Rust stack.
- **Cleaner than AVIF on licensing.** The design-time probe (2026-07-08)
  confirmed the resvg stack is **entirely permissive** (Apache-2.0 OR MIT;
  tiny-skia BSD-3-Clause) — **no deny.toml license exception needed** (the
  framing's MPL-2.0 assumption was outdated: RazrFalcon relicensed
  resvg/usvg/tiny-skia off MPL). The one cost is an *advisory* tail, not a
  license one (see Design Notes).
- **Foundational for the demo wave.** The same rasterizer serves the Wave-3
  in-browser demo (SVG→PNG/WebP client-side).

## Success Criteria

- `Image::load("x.svg")` / `Image::from_bytes(svg_bytes)` rasterize a real SVG to
  the canonical raster `Image` in the **default** build (no system libs, no
  C/nasm), at the SVG's intrinsic dimensions, honoring the DEC-034 decode caps.
- `optimize`/`convert`/`info`/`resize` operate on `.svg` inputs end to end;
  directory/glob sources discover `.svg`; a malformed SVG surfaces a typed
  `ImageError::Decode` (not a panic).
- **Hostile-input safety:** an SVG with an external file/URL reference does not
  read local files or reach the network; a decompression-bomb width/viewBox is
  rejected with `ImageError::LimitsExceeded` before pixel allocation; a
  cargo-fuzz target exists on the SVG decode path.
- **No C/system dependency on the default path**; `just deny` green (all-permissive
  tree, no new license exception; one *advisory* ignore for the ttf-parser
  unmaintained-status if text is enabled — see Design Notes); the lean
  `--no-default-features` build still succeeds.
- A **DEC-054** records the rasterizer-dependency choice (crate set, licenses,
  pure-Rust verification, the security configuration, and the advisory trade-off).

## Scope

### In scope
- Pick + wire the pure-Rust resvg stack; content-sniff SVG (`<svg`/`<?xml…<svg`)
  and dispatch in `decode_with_limits` before the generic `ImageReader`; admit
  `.svg` to `IMAGE_EXTENSIONS`; harden usvg (no external resources, no scripts,
  data-URI-only image hrefs); cap output dimensions via DEC-034 before raster;
  typed-error coverage; cargo-fuzz target. Text via the bundled Go font only.
  **(SPEC-060)**

### Explicitly out of scope
- SVG **output** / an SVG encoder; SVGZ (`.svgz` gzip); SMIL animation; a
  `--size`/`--scale`/`--dpi` override; system-font loading; AVIF/RAW/HEIC inputs
  (other stages).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-060 (design) — SVG **rasterize + security + wiring**: `resvg`/`usvg`/`tiny-skia`
  (all permissive) → tiny-skia Pixmap → straight RGBA8 → canonical `Image`; content-sniff
  dispatch in `decode_with_limits`, `.svg` in `IMAGE_EXTENSIONS`; usvg hardened
  (no external resources/scripts/network, data-URI-only hrefs), DEC-034 dimension cap before
  raster, typed errors, cargo-fuzz target, bundled-Go-font text; DEC-054; deny.toml advisory
  ignore (RUSTSEC-2026-0192, ttf-parser unmaintained) — **no license exception**.

**Count:** 0 shipped / 1 active / 0 pending — single-spec stage (mirrors STAGE-016's shape).

## Design Notes

- **PROBE RESULT (2026-07-08) — the resvg stack is fully permissive; the framing's
  MPL-2.0 assumption is OUTDATED.** A firsthand probe (`cargo add resvg` v0.47.0 in
  a throwaway crate + `cargo deny` against crustyimg's real allow-list) confirmed
  **every crate in the tree is permissive**: `resvg`/`usvg` = `Apache-2.0 OR MIT`,
  `tiny-skia`/`tiny-skia-path` = `BSD-3-Clause`, `fontdb`/`rustybuzz`/`ttf-parser`/
  `roxmltree`/`svgtypes`/`kurbo`/`zune-jpeg` = MIT or MIT/Apache. **No deny.toml
  *license* exception is required** — a genuine simplification versus the AVIF stack
  (which needed MPL/CC0 exceptions). RazrFalcon relicensed resvg/usvg/tiny-skia off
  MPL-2.0; the brief's "likely MPL → another exception" expectation no longer holds.
- **DEC-054 (at build):** adopt `resvg = { version = "=0.47.0", default-features =
  false, features = ["text"] }` (drops `system-fonts`, `memmap-fonts`,
  `raster-images`, gif/webp decoders — a lean, security-reduced tree). `usvg` and
  `tiny-skia` come transitively; we call `usvg`/`tiny-skia`/`resvg` directly. This
  is the webp-lossy/AVIF precedent (codec crates feeding the canonical `Image`), so
  `single-image-library` is **not** tripped — resvg is a rasterizer, not a second
  pixel library.
- **The one real cost is an ADVISORY tail, not a license one.** Enabling `text`
  re-introduces `ttf-parser 0.25.1` (via `usvg`→`fontdb`/`rustybuzz`), which trips
  **RUSTSEC-2026-0192** (ttf-parser is *unmaintained* — the exact advisory the repo
  deleted after moving watermark text to `skrifa`/`zeno`, DEC-045). resvg's text
  stack is built on ttf-parser and `skrifa` cannot be substituted, so the build
  must **re-add a `deny.toml` advisory ignore** (RUSTSEC-2026-0192; unmaintained
  status, not a vulnerability; parse-time only). *Verified:* dropping the `text`
  feature removes ttf-parser entirely and needs no ignore — see the fonts/text
  decision in SPEC-060 (v1 keeps text ON with a bundled font, because
  text-without-fonts renders text **silently to nothing**, a correctness footgun).
  This is the direct analog of AVIF's "a big decoder drags an advisory tail onto the
  default path" lesson (paste/RUSTSEC-2024-0436).
- **Maintenance status (checked 2026-07-08) — the stack is well-maintained; only a
  leaf is flagged, and it is being retired upstream.** resvg/usvg/tiny-skia were handed
  off from RazrFalcon to **Linebender** (the funded Rust 2D-graphics org — Vello/Kurbo/
  Xilem) and are **actively released** (v0.47.0, Feb 2026). Only the transitive
  `ttf-parser` *leaf* (harfbuzz org) carries the unmaintained flag, and the ecosystem is
  consolidating font parsing onto **fontations** (`read-fonts`/`skrifa`, via **HarfRust**)
  — which crustyimg **already uses via `skrifa`** for watermark text (DEC-045). So we are
  not building on a dead stack: we carry one stable, feature-complete leaf that upstream is
  actively replacing, and the revisit trigger is concrete — **drop the advisory ignore when
  resvg's text stack migrates off ttf-parser** (track: Linebender resvg font issue #200;
  fontations#956; HarfRust). Refs verified this session.
- **DECISION (v1): Option 1 — text ON + the advisory ignore.** A `skrifa` text→path
  pre-pass (own the text stack, no ttf-parser) was weighed against text-ON and **deferred**:
  `skrifa` provides glyph outlines but does NOT shape, and faithful SVG text layout
  (`text-anchor`, `x/y/dx/dy` lists, nested `<tspan>`, `letter-spacing`, writing-mode) would
  reimplement a large slice of usvg's text module at *lower* fidelity than the upstream
  migration hands us for free. Build the pre-pass ONLY if the upstream migration stalls AND
  an independent reason appears (e.g. WASM binary-size, Wave 3). Unlike AVIF (no upstream
  pure-Rust path existed, so owning the glue was right), owning SVG text now would be
  soon-throwaway work.
- **Security is the load-bearing work, and resvg is safe-by-construction for the
  worst vectors.** Verified firsthand: usvg does **not** execute scripts (strips
  them), has **no HTTP client** (SSRF is a non-issue), and its XML parser
  (`roxmltree`) does not expand external entities (XXE is a non-issue). The residual
  vectors are (a) **local-file read** via `<image href="…">`/`resources_dir` — closed
  by `resources_dir = None` + an `image_href_resolver` whose `resolve_string`
  returns `None` (refuse file/URL hrefs; keep data: URIs); and (b) **decompression
  bomb** via a huge declared `width`/`height`/`viewBox` — a 100000×100000 SVG
  *parses* fine (usvg allocates no pixels) so we **cap `tree.size()` against the
  DEC-034 limits BEFORE `tiny_skia::Pixmap::new`**, exactly like AVIF caps container
  dimensions before decode.
- **Wiring is small once the rasterizer exists (mirror AVIF):** SVG is NOT
  magic-byte sniffable by `image` — add a content sniff (`is_svg`: `<svg` or an XML
  prolog followed by `<svg`) and dispatch it in `decode_with_limits` **before** the
  generic `ImageReader` path; add `"svg"` to `IMAGE_EXTENSIONS`. The deltas are the
  `Cargo.toml` dep, one `src/image/svg.rs` module, the dispatch line, one
  `IMAGE_EXTENSIONS` entry, a `deny.toml` advisory ignore, and tests + a fuzz target.
- **`source_format` wrinkle (no `ImageFormat::Svg`):** `image::ImageFormat` has no
  SVG variant, and one would never "preserve" SVG as an output format anyway. After
  rasterization the `Image` genuinely *is* an RGBA8 raster, so SPEC-060 sets
  `source_format = ImageFormat::Png` for a rasterized SVG (natural lossless-RGBA
  target; also the sensible default output when `convert`/`optimize` gets no explicit
  format). Consequence: `info x.svg` reports the rasterized format, not "svg" — an
  accepted wart; a faithful `SourceFormat` enum that can name non-`image` inputs is a
  possible follow-up refactor, out of scope here.

## Dependencies

### Depends on
- Shipped decode seam (`src/image/mod.rs` `decode_with_limits` + the AVIF dispatch
  precedent at ~L288, `src/source/mod.rs` `IMAGE_EXTENSIONS` at ~L97, `src/error.rs`).
- DEC-004 (pure-Rust default), DEC-034 (decode caps), DEC-018 / `no-agpl-default-deps`.
- The bundled BSD-3 Go font already in the tree (`assets/fonts/Go-Regular.ttf`,
  DEC-045) for SVG `<text>`.

### Enables
- Richer optimize/convert/lint coverage over SVG asset trees; the Wave-3 in-browser
  demo's SVG→raster conversion; a shared "text-based/hostile input" hardening pattern
  reused by future importers.

## Stage-Level Reflection

*Filled in when status moves to shipped.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
