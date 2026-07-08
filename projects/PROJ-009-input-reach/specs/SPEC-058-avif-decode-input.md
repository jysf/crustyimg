---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-058
  type: story
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: L                    # probe (2026-07-07) confirmed: no permissive pure-Rust drop-in → rav1d+container glue; SPLIT into container-parse + decode specs at build

project:
  id: PROJ-009
  stage: STAGE-016
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-07

references:
  decisions: [DEC-004, DEC-034, DEC-018, DEC-020, DEC-053]
  constraints:
    - pure-rust-codecs-default
    - no-agpl-default-deps
    - no-new-top-level-deps-without-decision
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
  related_specs: [SPEC-018, SPEC-019, SPEC-020]

value_link: "STAGE-016's 'read AVIF from the default pure-Rust build' capability."

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-058: AVIF decode as a default, pure-Rust input

## Context

crustyimg can *write* AVIF (off-by-default `avif` feature → `ravif`, DEC-020) but cannot
*read* it in the default build: `image` 0.25's AVIF **decode** path depends on **dav1d, a C
system library**, which violates `pure-rust-codecs-default` (DEC-004). This spec adds a
**permissive, pure-Rust AVIF decoder** so the default binary reads `.avif` end to end
(`optimize`/`convert`/`info`/batch). AVIF/AV1 is **patent royalty-free** (AOMedia grant), so
it belongs on the default path — the key contrast with HEIC, which stays feature-gated for
patent + AGPL reasons (DEC-052). This is the headline spec of STAGE-016 / PROJ-009 (roadmap
Wave 1, `docs/roadmap.md`).

## Goal

Make the **default** crustyimg build decode `.avif` inputs to the canonical `Image` — pure-Rust,
zero system deps, `just deny` green — and admit `.avif` to the source allow-list so batch
commands see it.

## Inputs

- **Files to read:**
  - `src/image/mod.rs` — the decode seam (`decode_with_limits` at ~L271, `from_bytes`, `map_image_decode_error` ~L264, `decode_limits()` DEC-034 caps).
  - `src/source/mod.rs` — `IMAGE_EXTENSIONS` (~L94) + `has_image_extension`.
  - `src/error.rs` — `ImageError` enum (`Decode`, `UnsupportedFormat`, `LimitsExceeded`).
  - `Cargo.toml` — the `image` dep/features (L36) and `[features]` (L89+); existing `avif = ["image/avif"]` (encode) at L102.
  - `docs/research/heic-input-reach-spike.md` — the ISOBMFF box-parsing pattern (if the decoder needs container glue).
- **External APIs:** the chosen AVIF decoder (see Implementation Context / DEC-053). Candidates:
  `re_rav1d` via `image` (image-rs #2621) or `rav1d` (BSD-2-Clause) + AVIF container parse.
- **Related code paths:** `src/sink/mod.rs` (`ensure_codec_built` exit-4 pattern — mirror for behavior, but AVIF decode is DEFAULT so no exit-4 for it), `src/analysis/decide.rs` (unchanged: input format is not a decide variable today).

## Outputs

- **Files modified:**
  - `Cargo.toml` — add the pure-Rust AVIF decoder (new dep or an `image` feature that does NOT pull dav1d); keep default features pure-Rust.
  - `src/image/mod.rs` — ensure `.avif` decodes through `decode_with_limits` (usually automatic once the codec is active; add wiring only if the decoder is a separate crate, in which case dispatch on `ImageFormat::Avif` before the generic `ImageReader` path).
  - `src/source/mod.rs` — add `"avif"` to `IMAGE_EXTENSIONS`.
  - `deny.toml` — only if the chosen crate needs a documented note (it must NOT need a copyleft exception).
- **Files created:**
  - `tests/input_avif.rs` — integration tests (see Failing Tests).
  - `tests/fixtures/avif/solid_16x16.avif` — a tiny static fixture (provenance documented below).
  - `decisions/DEC-053-*.md` — the decoder-dependency decision (emitted during build, after the probe).
- **New exports:** none required (decode flows through existing `Image::from_bytes`); if a
  dedicated decoder needs a helper, keep it private to `src/image/`.

## Acceptance Criteria

- [ ] In the **default** build (`cargo build`, no extra features), `Image::load("*.avif")` and
  `Image::from_bytes(avif_bytes)` return a decoded `Image` with correct dimensions.
- [ ] Decoding honors the DEC-034 caps: an image over the dimension/alloc cap yields
  `ImageError::LimitsExceeded`, not an OOM or panic.
- [ ] A truncated/corrupt AVIF yields `ImageError::Decode(_)` (typed, no panic/`unwrap`).
- [ ] `.avif` is in `IMAGE_EXTENSIONS`; a directory/glob source containing an `.avif` includes it.
- [ ] `optimize <fixture>.avif -o out.webp` exits 0 and writes a valid WebP in the default build.
- [ ] **No C/system dependency on the default path**; `cargo build --no-default-features` (lean)
  still succeeds; `just deny` is green (permissive license, no new copyleft exception).
- [ ] `#[cfg(feature = "avif")]` round-trip: a natively-generated gradient encoded to AVIF and
  decoded back matches dimensions and is perceptually close (SSIM ≥ threshold), proving the
  decode path against our own encoder.
- [ ] `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` clean.

## Failing Tests

Written during **design**, BEFORE build. Build makes these pass.

- **`src/image/mod.rs`** (in the existing `#[cfg(test)] mod tests`)
  - `"avif_decodes_to_expected_dimensions"` — `Image::from_bytes(include_bytes!(".../solid_16x16.avif"))`
    → `Ok`, `width()==16 && height()==16`, `source_format == ImageFormat::Avif`.
  - `"corrupt_avif_is_decode_error_not_panic"` — feed the fixture truncated to 32 bytes →
    `Err(ImageError::Decode(_) | ImageError::UnsupportedFormat)`, never a panic.
  - `"avif_respects_decode_dimension_cap"` — call `decode_with_limits(fixture, tiny_limits)`
    with a dimension/alloc cap below the fixture → `Err(ImageError::LimitsExceeded(_))`.
- **`tests/input_avif.rs`** (integration, default build)
  - `"optimize_avif_input_writes_webp"` — run `optimize` on the fixture to a temp `.webp` →
    exit 0, output decodes as WebP with the fixture's dimensions.
  - `"directory_source_discovers_avif"` — a temp dir with `a.avif` (+ a `.txt`) → `source::resolve`
    returns exactly the `.avif`.
- **`tests/input_avif.rs`** (feature-gated round-trip)
  - `#[cfg(feature = "avif")] "avif_roundtrip_gradient"` — generate a 32×32 gradient natively,
    encode to AVIF bytes via the sink, `Image::from_bytes` it → dims match, SSIM ≥ 0.95 vs source.

> **Fixture provenance:** `tests/fixtures/avif/solid_16x16.avif` is a 16×16 solid image generated
> once via crustyimg's own `--features avif` encoder (ravif) and committed as a static asset,
> because AVIF cannot be produced natively without an encoder feature (AGENTS §12 forbids shelling
> out to ImageMagick). Document the one-line regen command in a comment in `tests/input_avif.rs`.

## Implementation Context

*Read this section (and the files it points to) before starting build.*

### Decisions that apply
- `DEC-004` — pure-Rust codecs by default; the AVIF **decoder must be pure-Rust** on the default
  path (no dav1d/C). Native stays feature-gated.
- `DEC-034` — decode resource caps; the new decoder must route through `image::Limits`
  (`decode_with_limits`) or enforce equivalent dimension/alloc caps itself.
- `DEC-018` — `no-agpl-default-deps`; **`zenavif` (imazen) is AGPL → excluded.** Verify the chosen
  crate's full license tree with `just deny`.
- `DEC-020` — the existing AVIF **output** feature (`ravif`); do not disturb it. This spec is decode.
- **`DEC-053` (NEW — emit during build after the probe)** — records the AVIF-decoder choice:
  the crate, its license (permissive), that it is pure-Rust + patent-clean (AV1 royalty-free),
  its maturity, and why it beats the dav1d path. If it is a new top-level dep, DEC-053 also
  satisfies `no-new-top-level-deps-without-decision`.

### The load-bearing probe — RESULT (done 2026-07-07; this is now an L, not an M)
A design-time probe ran (`docs/roadmap.md` reconciliation session). **Finding: there is NO mature,
permissive, pure-Rust AVIF decoder with a usable Rust API + container handling today**, so this is
real work, not a feature-flip:
- **`image`'s built-in avif decode = dav1d (C).** The pure-Rust path (image-rs #2621) is **open/
  unmerged** — so "AVIF decode via `image`" is NOT available in a pinnable version. (Re-check #2621
  at build; if it has landed in a compatible `image`, that becomes the cheapest path.)
- **`rav1d` / `re_rav1d` (BSD-2)** are permissive pure-Rust AV1 decoders but expose only a **C-style
  API** (Rust API "planned"), lean on asm/nasm (a slower no-asm build exists), and decode only the
  **AV1 bitstream** — they need **AVIF/ISOBMFF container parsing** on top (`ftyp avif`, `meta`/
  `iloc`/`iprp`/`av1C` → primary item — the box-parse pattern proven in the HEIC spike).
- **`rav1d-safe`/`zenavif` (imazen)** have a clean safe-Rust API + container but are **AGPL →
  excluded** (DEC-018). `avif-decode` (kornelski) uses AOM (**C**). `rusty-av1-toolkit` is experimental.

**Three viable paths — pick one as DEC-053:** (a) **`rav1d`/`re_rav1d` (BSD-2) + our own AVIF
container parser** + a verified no-asm pure-Rust build — keeps the default pure-Rust (weeks of work;
**split into a container-parse spec + a decode spec**); (b) **feature-gate a C decoder** (dav1d/aom)
off the default — acceptable for AVIF because it is patent-clean (contrast HEIC/DEC-052), but it is
not the pure-Rust default headline; (c) **wait** for image-rs #2621 / a re_rav1d Rust API, then AVIF
decode is nearly a free `image` version bump. Because none is a drop-in, **the sequencing of this
spec within Wave 1 is an open question** (SVG via `resvg` IS a clean pure-Rust drop-in and may lead
instead) — confirm with the maintainer before building. See the `avif-decode` entry in
`guidance/license-watchlist.yaml` for the full landscape.

### Constraints that apply
- `pure-rust-codecs-default`, `no-agpl-default-deps`, `no-new-top-level-deps-without-decision`,
  `untrusted-input-hardening` (AVIF is untrusted binary input — caps + no-panic + add a
  `cargo-fuzz` target if a new container parser is introduced),
  `no-unwrap-on-recoverable-paths`, `every-public-fn-tested`, `clippy-fmt-clean`.

### Prior related work
- `SPEC-018` / `DEC-020` — AVIF output (ravif) feature: the feature-gating + `ensure_codec_built`
  exit-4 pattern (mirror the *shape*, but AVIF decode is DEFAULT so it has no exit-4).
- `SPEC-019` (WebP) / `SPEC-020` (webp-lossy) — the "pure-Rust decode default, native-encode
  feature-gated, deny-green, no second image crate" precedent to follow.
- The HEIC spike — the ISOBMFF box-parsing approach (`iloc`/`iprp`) if option 2 is chosen.

### Out of scope (for this spec specifically)
- AVIF **animation** and **grid/tiled** multi-image (single primary image only).
- Any change to AVIF **output** / the `avif` encode feature.
- **Format-preservation bias** in `decide.rs` (input format is not a decide variable today; a
  separate follow-up spec if we ever want "don't re-encode an AVIF input").
- SVG / RAW / HEIC inputs (later stages).

## Notes for the Implementer

- Verify the **lean build** (`cargo build --no-default-features`) AND `just deny` as part of build,
  not just at verify — the default-feature gates don't exercise them (`verify-includes-lean-build`).
- Format dispatch is automatic once the codec is active (`ImageReader::with_guessed_format` in
  `decode_with_limits`) — prefer that path; only add explicit `ImageFormat::Avif` dispatch if the
  decoder is a standalone crate `image` can't route to.
- Keep the decoder OFF the pixel core's public surface; decode stays inside `src/image/`.
- If you introduce any new container parser, add a `cargo-fuzz` target and dimension caps up front
  (untrusted input; same discipline as the HEIC spike's security note).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-053` — AVIF decoder-dependency choice (required)
- **Deviations from spec:**
- **Follow-up work identified:**

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** —
2. **Was there a constraint or decision that should have been listed but wasn't?** —
3. **If you did this task again, what would you do differently?** —

---

## Reflection (Ship)

1. **What would I do differently next time?** —
2. **Does any template, constraint, or decision need updating?** —
3. **Is there a follow-up spec I should write now before I forget?** —
