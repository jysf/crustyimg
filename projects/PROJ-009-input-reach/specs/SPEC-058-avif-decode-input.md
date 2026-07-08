---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-058
  type: story
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # deep-dive (2026-07-07): viable path = re_rav1d(no-asm)+avif-parse+glue, ~1-1.5 wk; split into container (SPEC-059) + decode (this) specs

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
  sessions:
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 480000
      estimated_usd: 4.30
      duration_minutes: null
      recorded_at: 2026-07-07
      notes: >
        Main-loop build (NOT a separately-metered subagent), so tokens_total is
        an order-of-magnitude ESTIMATE per AGENTS §4 — Opus 4.8 $5/$25 per MTok,
        ~80/20 in/out, no cache discount. Input-heavy: read the decode/sink/source
        modules + several DECs, and ran a throwaway re_rav1d+avif-parse decode
        probe (encode→parse→decode round-trip) before wiring. Replace with the
        real /cost number if this is re-run as a metered cycle.
  totals:
    tokens_total: 480000
    estimated_usd: 4.30
    session_count: 1
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

### The load-bearing probe — RESULT (deep-dived 2026-07-07): a viable permissive pure-Rust path IS confirmed
No *clean drop-in* exists (every mature drop-in is C-backed or AGPL), **but** a viable permissive,
pure-Rust, zero-build-tool path is confirmed — AVIF-decode stays the Wave-1 default headline.

**SHIP PATH (DEC-053) — `re_rav1d` + `avif-parse` + our glue (~1–1.5 person-weeks):**
- **`re_rav1d`** (BSD-2, rerun) is a combined `rav1d`+`dav1d-rs` fork that re-exports the **safe,
  ergonomic `dav1d-rs` Rust API** (`Decoder::new` / `send_data` / `get_picture` / plane access),
  backed by pure-Rust rav1d — *not* just a C ABI (the first quick probe was wrong on this). Build
  **no-asm** (`--no-default-features --features bitdepth_8,bitdepth_16`) → pure-Rust, **zero
  build-tool deps (no nasm)**, and perf is fine for one-shot stills.
- **`avif-parse`** (MPL-2.0, kornelski, Firefox-mp4parse-derived, hardened) parses the ISOBMFF/MIAF
  container and hands you the primary-item + alpha AV1 OBUs ready to decode. MPL-2.0 is file-level
  copyleft — fine alongside MIT/Apache; add a `deny.toml` note.
- **Our glue:** feed OBUs → `re_rav1d` → YUV planes → RGB(A), honoring bit depth (8/10/12),
  `pixel_layout` (4:2:0/2:2/4:4:4), and color (nclx/CICP matrix+range) + premultiplied alpha.

**Split accordingly:** SPEC-058 (this) = decode integration + color/alpha; SPEC-059 = the AVIF
container parse (or a thin wrap of `avif-parse`). The same decoder also serves the Wave-3 WASM demo.

**Caveats to test against real files:** (1) `re_rav1d` is rerun's self-described "messy" fork with an
uncertain maintenance future → **pin versions, keep the FFI/glue surface thin** so we can swap later;
(2) **grid/tiled** AVIF may be unsupported by `avif-parse` → reject cleanly if so; (3) **color
correctness** is the usual AVIF footgun, not a decoder bug.

**Parallel / future (not blocking this spec):** contribute **image-rs #2621** upstream (trivial — a
working PoC exists) and/or a **native Rust API into `rav1d`**, so crustyimg can later migrate to
`image`'s built-in pure-Rust decode and shed the direct `re_rav1d` dep + glue. **Watchlist** the
**OxideAV** MIT stack (`oxideav-avif` + `oxideav-av1`, pure-Rust, no C/nasm, intra-only today, sub-1.0)
as the potentially-cleanest future — one MIT stack for both AVIF and HEIC — and re-probe as it matures.
See `guidance/license-watchlist.yaml` → `avif-decode` for the full landscape.

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

- **Branch:** `feat/spec-058-avif-decode`
- **PR (if applicable):** (opened after coordinating a push of origin/main — see below)
- **All acceptance criteria met?** yes
  - [x] default build decodes `.avif` (`Image::load`/`from_bytes`) with correct dims + `source_format == Avif`
  - [x] DEC-034 caps honored (dimension/alloc cap → `LimitsExceeded`, checked from container metadata **before** allocation)
  - [x] truncated/corrupt AVIF → typed `ImageError::Decode` (no panic/`unwrap`)
  - [x] `.avif` in `IMAGE_EXTENSIONS`; directory source discovers it
  - [x] `optimize <fixture>.avif -o out.webp` exits 0, writes a valid 16×16 WebP (default build)
  - [x] no C/system dep on the default path; `--no-default-features` builds; `just deny` green (no new copyleft in the global allow list — MPL/CC0 excepted per-crate)
  - [x] `#[cfg(feature="avif")]` round-trip: 32×32 gradient encode→decode, dims match, SSIMULACRA2 well above threshold
  - [x] `cargo clippy --all-targets -- -D warnings` (default + lean + avif) and `cargo fmt --check` clean
- **New decisions emitted:**
  - `DEC-053` — AVIF decoder-dependency choice: `re_rav1d` (no-asm, BSD-2) + `avif-parse` (MPL-2.0) + thin YUV→RGB glue.
- **Deviations from spec:**
  - **The decoder API:** the spec/probe framing said `re_rav1d` re-exports the *dav1d-rs* ergonomic API. Confirmed TRUE for 0.1.3 — it lives in `re_rav1d::dav1d` (a dav1d-rs fork). No C-ABI FFI was needed; the glue uses the safe `Decoder`/`Picture` API. (The raw C ABI is also present but unused.)
  - **8-bit output:** 10/12-bit AVIF is down-converted to 8-bit RGB(A) without HDR tone-mapping (transfer-function/HDR handling left out of scope). Noted in DEC-053.
  - **SPEC-059 not needed:** `avif-parse` covered the container (primary + alpha OBUs, clean grid rejection) with no substantial in-house ISOBMFF glue, so the decode stayed a single spec. SPEC-059 remains a *future option* only if we later drop MPL for an in-house MIT parser.
  - **`IMAGE_EXTENSIONS`:** added only `avif` (noticed `webp` is also absent from the list, but leaving that to a separate spec — `one-spec-per-pr`).
  - **MSRV:** bumped 1.89 → 1.90 (avif-parse 2.1.0 floor); CI `msrv` job confirms.
  - **deny.toml:** added per-crate exceptions for `avif-parse` (MPL-2.0) and `to_method` (CC0-1.0, a re_rav1d transitive); updated the `paste` advisory-ignore reason (now also reached on the default path via re_rav1d).
- **Follow-up work identified:**
  - Run the `fuzz/avif_decode` target under nightly + `cargo-fuzz` (not installed in the build env) — the target compiles against the public API; seed from `tests/fixtures/avif`.
  - Optional: migrate to `image`'s built-in pure-Rust AVIF decode when image-rs #2621 lands (sheds the direct `re_rav1d` dep + hand glue); watchlist the OxideAV MIT stack.
  - Optional: preserve 10/12-bit precision (Rgb16/Rgba16) + HDR transfer handling; add `.webp` to `IMAGE_EXTENSIONS`; a decide-engine "don't re-encode an AVIF input" bias.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** — Whether `re_rav1d` exposed a safe API or only a C ABI. The design section hedged ("first quick probe was wrong"), so I inspected the crate source directly; it does expose the safe `re_rav1d::dav1d` API, which made the glue clean. The load-bearing probe the spec demanded was the right call — it resolved this in minutes and validated color correctness on real files.
2. **Was there a constraint or decision that should have been listed but wasn't?** — The transitive-license fallout wasn't fully anticipated: beyond the expected MPL exception for `avif-parse`, `re_rav1d` pulls `to_method` (CC0-1.0) and `paste` (an existing advisory) onto the **default** path. Worth a spec note that adding a large decoder often drags a small license/advisory tail that `just deny` surfaces.
3. **If you did this task again, what would you do differently?** — Run `just deny` immediately after `cargo add` (before writing any glue) to surface the CC0/MPL/advisory tail up front, rather than after the whole module was written. Everything else — probe-first, thin glue, cap-before-decode — I'd keep.

---

## Reflection (Ship)

1. **What would I do differently next time?** —
2. **Does any template, constraint, or decision need updating?** —
3. **Is there a follow-up spec I should write now before I forget?** —
