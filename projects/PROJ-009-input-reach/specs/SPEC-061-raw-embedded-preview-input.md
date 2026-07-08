---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-061
  type: story
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # one module + a load() branch; the weight is security bounds + the extension-routing wrinkle + corpus honesty

project:
  id: PROJ-009
  stage: STAGE-018
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-08

references:
  decisions: [DEC-004, DEC-034, DEC-018, DEC-055]
  constraints:
    - pure-rust-codecs-default
    - no-agpl-default-deps
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - single-image-library
  related_specs: [SPEC-058, SPEC-060]

value_link: "STAGE-018's 'extract the embedded full-res JPEG preview from common RAW on the default build' capability."

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
        Included a firsthand probe (the repo's pinned `image` =0.25.10) proving the scan-for-largest-
        embedded-JPEG mechanism: load_from_memory tolerates trailing bytes, and decode-from-each-SOI +
        pick-largest-by-pixels extracts the full preview over the thumbnail — no IFD/ISOBMFF parsing,
        no new dep.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-061: RAW Tier-1 embedded-preview extraction as a default input

## Context

crustyimg cannot read camera RAW today. Full RAW *development* (demosaic +
white-balance + color-matrix) is Tier-2 — LGPL (`rawler`) or a multi-month
from-scratch effort — and is deliberately out of scope (watchlist
`raw-camera-decode`, DEC-018). But nearly every RAW file embeds a **full-res
JPEG preview** (what the camera's screen shows), and extracting *that* is
permissive, pure-Rust, patent-free, and needs no RAW codec. This spec adds
**Tier-1 embedded-preview extraction** so the default binary turns a
`.nef`/`.cr2`/`.cr3`/`.arw`/`.dng`/… into a quick web derivative end to end
(`optimize`/`convert`/`info`/`resize`/batch). It is the third default input of
PROJ-009 (roadmap Wave 1), mirroring **SPEC-058** (AVIF) and **SPEC-060** (SVG):
explicit routing before the generic decoder, RAW extensions in the source
allow-list, typed `ImageError`s, DEC-034 caps, a `cargo-fuzz` target, and a
decision (**DEC-055**) emitted at build. See the parent
`STAGE-018-raw-embedded-preview-input.md` for the probe result and framing.

## Goal

Make the **default** crustyimg build extract the largest embedded JPEG preview
from common RAW files and return it as the canonical raster `Image` — pure-Rust,
no new dependency, bounded against hostile input (DEC-034 caps, capped candidate
count, typed errors) — and admit RAW extensions to the source allow-list so batch
commands see them.

## Inputs

- **Files to read:**
  - `src/image/mod.rs` — `Image::load` (~L61: reads bytes → `from_bytes`), `from_bytes`,
    `decode_with_limits` (~L279) + the AVIF/SVG dispatch precedent, `decode_limits()` (DEC-034 caps:
    `MAX_IMAGE_DIMENSION` = 65_535, `MAX_ALLOC_BYTES` = 512 MiB), `map_image_decode_error`, and the
    `#[cfg(test)] mod tests` layout.
  - `src/image/avif.rs` + `src/image/svg.rs` — the **pattern to mirror**: a small private
    `src/image/<fmt>.rs` module with a detector + a `decode_*` that enforces the caps and maps to typed
    `ImageError`, plus the module's `## Security` doc-comment.
  - `src/source/mod.rs` — `IMAGE_EXTENSIONS` (~L97, has `jpg…avif svg`) + `has_image_extension`.
  - `src/error.rs` — `ImageError` (`Decode`, `UnsupportedFormat`, `LimitsExceeded`).
  - `src/metadata/tiff.rs` — the in-house TIFF IFD parser (SPEC-045). **Read it, then note it is NOT
    needed** for v1 (the byte-scan approach sidesteps IFD walking); listed so build doesn't re-derive
    the "should I parse IFDs?" question. It follows only EXIF/GPS/Interop pointers + the IFD1
    thumbnail, not the SubIFDs where full previews live — which is exactly why the scan is simpler.
  - `fuzz/Cargo.toml` + `fuzz/fuzz_targets/{avif,svg}_decode.rs` — the cargo-fuzz target to mirror.
- **External APIs:** none new — `image`'s existing JPEG decoder (`image::load_from_memory_with_format`
  / the `ImageReader` + `Limits` path already in `decode_with_limits`).
- **Related code paths:** `src/cli/mod.rs` `output_format_for` (the "preserve source_format" default),
  `src/sink/mod.rs` (`encode_to_bytes`), `src/pipeline/` (loads file inputs via `Image::load`).

## Outputs

- **Files created:**
  - `src/image/raw.rs` — `is_raw_extension(&Path) -> bool`, `extract_preview(bytes, &Limits) -> Result<DynamicImage>`
    (public byte entry for routing + fuzz), the JPEG-SOI scan + capped candidate decode + largest-wins
    logic, and typed-error mapping. Private/`pub(crate)` to `src/image/` (mirror `avif`/`svg`).
  - `tests/input_raw.rs` — integration tests (see Failing Tests).
  - `tests/fixtures/raw/synthetic_preview.nef` — a **hand-built synthetic** RAW-preview fixture
    (a TIFF header + a small embedded JPEG thumbnail + a larger embedded JPEG preview), generated
    natively from `image`'s own JPEG encoder (no camera, no ImageMagick — AGENTS §12). Document the
    construction in a comment / a `tests/`-local helper or an `examples/gen_raw_fixture.rs`.
  - `fuzz/fuzz_targets/raw_preview.rs` — a cargo-fuzz target on `raw::extract_preview` (via a public entry).
  - `decisions/DEC-055-*.md` — the RAW-input approach decision (emitted during build).
- **Files modified:**
  - `src/image/mod.rs` — `mod raw;`; in `Image::load`, branch on `raw::is_raw_extension(path)` →
    `raw::extract_preview(&bytes, &decode_limits())` → build `Image` with `source_format = Jpeg` +
    best-effort metadata from the preview JPEG; else fall through to `from_bytes`. Add `#[cfg(test)]` unit tests.
  - `src/source/mod.rs` — add the RAW extensions to `IMAGE_EXTENSIONS` (update the block comment).
  - `fuzz/Cargo.toml` — add the `raw_preview` `[[bin]]` + a `Seed:` line to `tests/fixtures/raw`.
- **New exports:** a public byte entry for RAW preview extraction (e.g. `crustyimg::image::raw_preview(&[u8]) -> Result<Image>`
  or a `pub(crate)` fn the fuzz target reaches) — enables the `load` branch, the fuzz target, and a
  future stdin/`--format raw` path. Keep the scan/decode internals private.
- **No `Cargo.toml` / `deny.toml` change** — no new crate.

## Acceptance Criteria

- [ ] In the **default** build, `Image::load("<fixture>.nef")` returns the embedded **full-res** JPEG
  preview (the larger of the two embedded JPEGs) as the canonical `Image`, with the preview's dimensions
  and `source_format == Jpeg`.
- [ ] Extraction is format-agnostic: the same path handles a TIFF-based container (`.nef`/`.cr2`/`.arw`/`.dng`)
  and a non-TIFF container (a CR3-style ISOBMFF blob / a RAF-style blob) — all via the JPEG-SOI scan, no
  per-vendor parsing. (At minimum: a unit test proves the scan finds the largest JPEG regardless of the
  surrounding container bytes.)
- [ ] Decoding honors the DEC-034 caps: a candidate preview whose declared dimensions exceed the cap is
  rejected; if the only preview present is oversize → `ImageError::LimitsExceeded`; no OOM/panic.
- [ ] A RAW with **no decodable embedded JPEG** → a typed `ImageError` (`Decode`/`UnsupportedFormat`),
  never a panic; false `FF D8 FF` matches in compressed data are skipped and do not error the whole file.
- [ ] The number of full candidate **decode attempts is bounded** (a file stuffed with fake SOIs cannot
  cause unbounded work) — assert via a unit test with many fake SOIs.
- [ ] RAW extensions are in `IMAGE_EXTENSIONS`; a directory/glob source containing a `.nef` includes it
  (and excludes a non-image sibling).
- [ ] `optimize <fixture>.nef -o out.webp` exits 0 and writes a valid WebP with the preview's dimensions;
  `convert <fixture>.nef --format png -o out.png` exits 0 and writes a valid PNG.
- [ ] **No new dependency**, no C/system dep; `cargo build --no-default-features` (lean) still succeeds;
  `just deny` unchanged and green (no new crate/license/advisory).
- [ ] A `fuzz/raw_preview` target compiles against the public API.
- [ ] `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` clean.

## Failing Tests

Written during **design**, BEFORE build. Build makes these pass.

> **Fixture:** `tests/fixtures/raw/synthetic_preview.nef` — a hand-built blob: a `II*\0` TIFF header,
> a small (~16×12) embedded JPEG thumbnail, then a larger (~64×48) embedded JPEG preview, plus filler
> bytes. Built natively from `image`'s JPEG encoder (no camera/ImageMagick). Unit tests also build
> equivalent blobs inline. Document the regen in a comment / `examples/gen_raw_fixture.rs`.

- **`src/image/raw.rs`** (in a new `#[cfg(test)] mod tests`)
  - `"is_raw_extension_matches_known_exts"` — `.nef/.cr2/.cr3/.arw/.dng/.raf/.rw2/.orf/.pef/.srw`
    (any case) → `true`; `.jpg/.png/.tif/.tiff/.svg/.avif/.webp` → `false`.
  - `"extract_preview_picks_largest_decodable_jpeg"` — a blob `[tiff hdr][16×12 jpeg][junk][64×48 jpeg][junk]`
    → `Ok`, dims `64×48` (the full preview, not the thumbnail).
  - `"extract_preview_skips_false_soi_matches"` — the same blob with extra `FF D8 FF xx` junk sequences
    that are not valid JPEGs → still `Ok` at `64×48`; the whole file is not errored by a bad candidate.
  - `"extract_preview_bounds_decode_attempts"` — a blob with many (> the cap) fake `FF D8 FF` markers →
    completes quickly and does not attempt more than the capped number of decodes (assert via a counter
    hook or by construction), returning the valid preview if present or a typed error if not.
  - `"oversize_only_preview_is_limits_exceeded"` — a blob whose single embedded JPEG declares dims above
    a tiny `Limits` → `extract_preview(bytes, &tiny_limits)` → `Err(ImageError::LimitsExceeded(_))`.
  - `"no_embedded_jpeg_is_typed_error_not_panic"` — a blob with no decodable JPEG (just a TIFF header +
    noise) → `Err(ImageError::Decode(_) | ImageError::UnsupportedFormat)`.
- **`src/image/mod.rs`** (in the existing `#[cfg(test)] mod tests`, a `── SPEC-061 RAW` section)
  - `"raw_extension_routes_to_preview_extraction"` — write the fixture to a temp `photo.nef`,
    `Image::load(path)` → `Ok`, dims == preview dims, `source_format == ImageFormat::Jpeg`.
  - `"non_raw_extension_still_uses_generic_decoder"` — a real PNG written to `x.png` still loads via the
    generic path (`source_format == Png`), proving the RAW branch is extension-gated and side-effect-free.
- **`tests/input_raw.rs`** (integration, default build)
  - `"optimize_raw_input_writes_webp"` — `optimize <fixture>.nef -o out.webp` → exit 0, output decodes as
    WebP with the preview's dims.
  - `"convert_raw_to_png"` — `convert <fixture>.nef --format png -o out.png` → exit 0, PNG with the dims.
  - `"directory_source_discovers_raw"` — a temp dir with `a.nef` (+ a `.txt`) → `source::resolve` returns
    exactly the `.nef`.

## Implementation Context

*Read this section (and the files it points to) before starting build. The "PROBE" block was verified
firsthand during design against the repo's pinned `image` =0.25.10 — trust it, but re-confirm the exact
API at build.*

### Decisions that apply
- `DEC-004` — pure-Rust codecs by default; RAW preview uses only `image`'s pure-Rust JPEG decoder + a
  byte scanner. No new dep, no C.
- `DEC-034` — decode caps; every candidate preview decode routes through the same `Limits`
  (`decode_with_limits`/`decode_limits()`), so a bomb preview is rejected before allocation.
- `DEC-018` / `no-agpl-default-deps` — Tier-2 development (`rawler`, LGPL-2.1) stays out; this spec adds
  no crate at all.
- **`DEC-055` (NEW — emit during build)** — records: RAW input = Tier-1 largest-embedded-JPEG preview via
  a format-agnostic byte scan + capped `image` JPEG decode; extension-routed in `load`; covered
  extensions; **no new dependency**; security bounds; and that Tier-2 development is out (watchlist
  `raw-camera-decode`). No new top-level dep, so `no-new-top-level-deps-without-decision` is not tripped —
  but DEC-055 still records the capability + approach.

### PROBE — verified firsthand (2026-07-08), the scan-and-decode-largest mechanism

Against the repo's pinned `image` =0.25.10 (default-features off, `jpeg` on):
- **`image::load_from_memory` tolerates trailing bytes after a JPEG's EOI** — decoding a 64×48 JPEG with
  200 random trailing bytes returned `Ok(64×48)`. So we do NOT need to find each JPEG's exact end; we can
  decode *from* each SOI and let the decoder stop at the real EOI.
- **Scan + pick-largest works:** given a blob `[TIFF header][16×12 jpeg][junk][64×48 jpeg][trailing junk]`,
  scanning for `FF D8 FF`, calling `load_from_memory_with_format(&buf[i..], ImageFormat::Jpeg)` at each
  match, and keeping the largest by pixel count returned the **64×48 preview** at the correct offset —
  skipping the 16×12 thumbnail. Junk `FF` bytes produced no false decodes.
- **Therefore CR3 (ISOBMFF) and Fuji RAF are covered by the same scan** — their previews are embedded
  baseline JPEGs, found without box/IFD parsing. This corrects the brief's "reuses ISOBMFF glue for CR3"
  and the watchlist's "parse the TIFF/EXIF IFDs" — both are unnecessary for Tier-1.

Sketch (re-verify names at build):
```rust
pub(crate) fn extract_preview(bytes: &[u8], limits: &Limits) -> Result<DynamicImage> {
    let mut best: Option<DynamicImage> = None;
    let mut best_px = 0usize;
    let mut saw_oversize = false;
    let mut attempts = 0usize;
    let mut i = 0;
    while i + 3 <= bytes.len() {
        // Cheap prune: SOI (FF D8) + a plausible next marker (FF E0..EF / FF DB / FF C0..CF / FF FE).
        if bytes[i] == 0xFF && bytes[i + 1] == 0xD8 && bytes[i + 2] == 0xFF
            && is_plausible_jpeg_marker(bytes.get(i + 3)) {
            if attempts >= MAX_PREVIEW_CANDIDATES { break; }   // bound the work (untrusted input)
            attempts += 1;
            match decode_jpeg_with_limits(&bytes[i..], limits) {   // routes through DEC-034 caps
                Ok(img) => { let px = (img.width() as usize) * (img.height() as usize);
                             if px > best_px { best_px = px; best = Some(img); } }
                Err(ImageError::LimitsExceeded(_)) => saw_oversize = true,
                Err(_) => {}   // false SOI / not a JPEG → skip
            }
            i += 3;
        } else { i += 1; }
    }
    match best {
        Some(img) => Ok(img),
        None if saw_oversize => Err(ImageError::LimitsExceeded("raw: embedded preview exceeds caps".into())),
        None => Err(ImageError::Decode("raw: no decodable embedded JPEG preview".into())),
    }
}
```
Notes: `MAX_PREVIEW_CANDIDATES` bounds decode attempts (pick a small N, e.g. 16 — real RAW has 1–3 real
previews; the prune keeps the count low anyway). To favor the true full preview cheaply you MAY sort
candidate offsets by span-to-next-SOI descending and try the biggest first, but the "decode all pruned
candidates, keep max pixels" loop above is already correct and bounded — measure before optimizing.

### Detection / routing decision (extension-driven)
- TIFF-based RAW starts with TIFF magic (`II*\0`/`MM\0*`), byte-indistinguishable from a plain `.tif`, so
  a content sniff would risk mis-routing legitimate TIFFs. **Route by file extension in `Image::load`**
  (which has the `Path`): `if raw::is_raw_extension(path) { return Image from raw::extract_preview(...) }`
  before the generic `from_bytes`. Keep `from_bytes`/`decode_with_limits` generic (AVIF/SVG/standard).
- **RAW via stdin (`-`) is a v1 non-goal** — `from_bytes` has no path. Document it; a `--format raw` hint
  or a TIFF-`Make`/CR3-`ftyp 'crx '` sniff is a later option. The fuzz target still reaches the byte-level
  `extract_preview` directly, so the untrusted-input path is fuzzed regardless of routing.
- **`source_format = ImageFormat::Jpeg`** — the preview is a JPEG; report the materialized raster format
  (the SVG→`Png` precedent). `info x.nef` will report `jpeg`; accepted wart. Metadata: best-effort from
  the preview JPEG's own APP1 (reuse `MetadataBundle::capture(preview_bytes, ImageFormat::Jpeg)`), or
  `None` — do NOT parse the RAW container's EXIF in v1.

### RAW extension set (v1)
`nef, nrw` (Nikon) · `cr2, cr3` (Canon) · `arw, srf, sr2` (Sony) · `dng` (Adobe/Leica/Pixel/…) ·
`raf` (Fuji) · `rw2` (Panasonic) · `orf` (Olympus) · `pef` (Pentax) · `srw` (Samsung) · `rwl` (Leica) ·
`raw` (generic). Case-insensitive (reuse `has_image_extension`'s comparison). Note `.x3f` (Sigma Foveon)
is deliberately omitted (no standard baseline-JPEG preview) — it would just yield the typed "no preview"
error if named directly; fine.

### Constraints that apply
- `pure-rust-codecs-default`, `no-agpl-default-deps` (Tier-2 out), `untrusted-input-hardening` (RAW is
  hostile untrusted binary — cap every candidate decode via DEC-034, bound the candidate count, no-panic
  typed errors, add the `cargo-fuzz` target), `no-unwrap-on-recoverable-paths`, `every-public-fn-tested`,
  `clippy-fmt-clean`, `single-image-library` (the preview decodes through the existing `image` decoder
  into the canonical `Image`; no second pixel library, no pixel op routed elsewhere).

### Prior related work
- `SPEC-058`/`DEC-053` (AVIF) and `SPEC-060`/`DEC-054` (SVG) — the input-format pattern to mirror
  (routing before the generic decoder, `IMAGE_EXTENSIONS`, caps-before-alloc, typed errors, fuzz target,
  dep/approach DEC, `source_format` = materialized format).
- `SPEC-045`/`DEC-046` (`src/metadata/tiff.rs`) — the in-house TIFF IFD parser; **not needed here** (the
  scan sidesteps IFD walking), but read it so the "should I parse IFDs?" question is pre-answered: no.

### Out of scope (for this spec specifically)
- RAW **development** (demosaic/white-balance/color — Tier-2, LGPL `rawler`, watchlist); RAW via **stdin**;
  RAW-container **EXIF/metadata** passthrough beyond the preview JPEG's own; a faithful `SourceFormat`
  enum; guaranteeing every vendor/model; AVIF/SVG/HEIC inputs (other stages).

## Notes for the Implementer

- Verify the **lean build** (`cargo build --no-default-features`) AND `just deny` as part of build — no
  new crate, so `deny` should be unchanged, but confirm (the RAW path is non-optional / default).
- Keep `src/image/raw.rs` thin and off the pixel core's public surface (mirror `avif`/`svg`); expose only
  the byte entry needed for routing + fuzz.
- Route every candidate decode through the DEC-034 `Limits` — do NOT call an uncapped `load_from_memory`;
  reuse/parametrize the `decode_with_limits` machinery so the caps are enforced identically.
- Bound `MAX_PREVIEW_CANDIDATES` and prune with a plausible-marker check before decoding — untrusted input
  can stuff the file with `FF D8 FF`. A fuzz seed from `tests/fixtures/raw` should not time out.
- The fixture is a hand-built synthetic blob (TIFF header + two embedded JPEGs) — build it from `image`'s
  JPEG encoder (no camera/ImageMagick, AGENTS §12). Consider an `examples/gen_raw_fixture.rs` for regen,
  and reuse the same construction inline in unit tests so they are self-contained.
- MSRV: no new dep, so the floor should be unchanged (1.90) — confirm via the CI `msrv` job.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-055` — <title>
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — <answer>
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>
3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

1. **What would I do differently next time?**
   — <answer>
2. **Does any template, constraint, or decision need updating?**
   — <answer>
3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
