---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-062
  type: story
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # module + decode-side CodecNotBuilt + a system-lib CI job + LGPL/distribution discipline

project:
  id: PROJ-009
  stage: STAGE-019
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-08

references:
  decisions: [DEC-052, DEC-004, DEC-034, DEC-018, DEC-022, DEC-056]
  constraints:
    - pure-rust-codecs-default
    - no-agpl-default-deps
    - no-new-top-level-deps-without-decision
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - single-image-library
  related_specs: [SPEC-058, SPEC-060, SPEC-061, SPEC-018, SPEC-020]

value_link: "STAGE-019's 'read HEIC only under --features heic; default binary exits 4' capability."

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
        Included a firsthand probe (brew libheif 1.23.1 + a real .heic made via `sips`): decoded it to
        64×48 with correct pixels via libheif-rs 2.7.0; confirmed libheif-rs/-sys are MIT (deny green,
        no exception; LGPL is the system C lib) and system-linked via pkg-config.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-062: HEIC decode behind an off-by-default `heic` feature

## Context

HEIC/HEIF is the one common modern format crustyimg **cannot** put on the default
path. Per **DEC-052**, two independent blockers force a gate: the mature pure-Rust
HEIC decoders are **AGPL** (`no-agpl-default-deps`, DEC-018), and **HEVC is patent-
encumbered** (Access Advance pool) regardless of code license — so the exposure
attaches to every decode path, and even a future permissive decoder would not
un-gate it. This spec implements DEC-052: HEIC decode behind an off-by-default
`heic` cargo feature backed by **system libheif** (decode-only), so a user who
opts in and has libheif installed reads `.heic` end to end, while the DEFAULT
distributed binary stays pure-Rust, zero-system-dep, and returns a clear **exit 4**
on `.heic`. It closes PROJ-009 (roadmap Wave 1) as the deliberate NON-default
counterpart to AVIF (SPEC-058), SVG (SPEC-060), and RAW (SPEC-061). See the parent
`STAGE-019-heic-decode-feature-gated.md` for the probe result and framing.

## Goal

Add a `heic` cargo feature that decodes `.heic`/`.heif` via system libheif
(decode-only) to the canonical `Image`, DEC-034-capped and typed-error-safe; make
the **default** (no-`heic`) build detect `.heic` and exit 4 with a clear "rebuild
with --features heic"; keep the feature out of every distributed artifact and
`just deny` green.

## Inputs

- **Files to read:**
  - `src/image/mod.rs` — `decode_with_limits` (~L279) + the AVIF ftyp-brand dispatch
    (`if avif::is_avif(bytes) { … }`), `decode_limits()` (DEC-034 caps), `mod avif;`, the
    `#[cfg(test)] mod tests` layout.
  - `src/image/avif.rs` — the **pattern to mirror**: `is_avif` (ftyp-brand scan), `check_caps`
    (cap dims before allocation), typed-error mapping, the `## Security` doc-comment.
  - `src/error.rs` — `ImageError` (`Decode`/`UnsupportedFormat`/`LimitsExceeded`/`Io`); add
    `CodecNotBuilt` here.
  - `src/sink/mod.rs` — `SinkError::CodecNotBuilt { codec, feature }` (~L145) + `ensure_codec_built`
    (~L247) + the exit-4 mapping — the **encode-side precedent** to mirror on the decode side.
  - `src/cli/mod.rs` — the `SinkError::CodecNotBuilt => 4` exit mapping (~L539) + `UnsupportedFormat →
    4` (~L505); wire the new `ImageError::CodecNotBuilt` to exit 4 too.
  - `src/source/mod.rs` — `IMAGE_EXTENSIONS` (~L97) + `has_image_extension`.
  - `Cargo.toml` — `[features]` (avif/webp-lossy/display) + the optional-dep pattern (`webp`); add
    `libheif-rs` optional + `heic = ["dep:libheif-rs"]`.
  - `.github/workflows/ci.yml` — the `avif` and `webp-lossy` feature jobs (~L60–100) to mirror for a
    system-libheif job; the `lean` job.
  - `deny.toml` — confirm no change needed (libheif-rs/-sys are MIT; `[graph] all-features = true`
    pulls them in but they need no exception).
- **External APIs:** `libheif-rs` 2.7.0 (MIT) → `libheif-sys` 5.3.0+1.23.0 (MIT) → **system** libheif
  (LGPL, decode-only). Docs: https://docs.rs/libheif-rs . Verified API in Implementation Context.
- **Related code paths:** `docs/` (LGPL attribution / feature docs), cargo-dist config + Homebrew
  formula (confirm `heic` excluded).

## Outputs

- **Files created:**
  - `src/image/heic.rs` — `is_heic(bytes) -> bool` (ftyp HEVC-brand scan, always compiled) +
    `#[cfg(feature = "heic")] decode_heic(bytes, &Limits) -> Result<DynamicImage>` (libheif-rs →
    interleaved RGB(A) → `DynamicImage`, DEC-034-capped). Private/`pub(crate)` to `src/image/`.
  - `tests/input_heic.rs` — integration tests (default-build exit-4 + `#[cfg(feature="heic")]` decode).
  - `tests/fixtures/heic/solid_64x48.heic` — a small committed HEIC fixture (generated once via `sips`;
    document regen). Only used by `#[cfg(feature="heic")]` tests + the default-build exit-4 test.
  - `fuzz/fuzz_targets/heic_decode.rs` — a `#[cfg(feature="heic")]` cargo-fuzz target (needs libheif+nightly).
  - `decisions/DEC-056-*.md` — the libheif dependency + feature-gate decision (emitted during build).
- **Files modified:**
  - `src/error.rs` — add `ImageError::CodecNotBuilt { codec: &'static str, feature: &'static str }`
    (mirror `SinkError::CodecNotBuilt`).
  - `src/image/mod.rs` — `mod heic;`; in `decode_with_limits`, after the AVIF branch: `if
    heic::is_heic(bytes) { #[cfg(feature="heic")] return Ok((heic::decode_heic(bytes, limits)?,
    ImageFormat::Png)); #[cfg(not(feature="heic"))] return Err(ImageError::CodecNotBuilt{codec:"HEIC",
    feature:"heic"}); }`. Add `#[cfg(test)]` unit tests for `is_heic` + the exit-4 path.
  - `src/cli/mod.rs` — map `ImageError::CodecNotBuilt` → exit 4 (with the clear message).
  - `src/source/mod.rs` — add `"heic"`, `"heif"` to `IMAGE_EXTENSIONS` (update the comment).
  - `Cargo.toml` — `libheif-rs = { version = "=2.7.0", optional = true, default-features = false }`
    (or with the version feature the build confirms) + `heic = ["dep:libheif-rs"]`.
  - `.github/workflows/ci.yml` — a `heic` job (install system libheif, build/test/clippy `--features heic`).
  - Distribution config (cargo-dist / brew) — confirm `heic` is NOT enabled; document local-build-only.
  - `docs/` — LGPL attribution note for `--features heic` builds that redistribute.
- **New exports:** none required (decode flows through `decode_with_limits`); keep `is_heic`/`decode_heic`
  `pub(crate)` in `src/image/`. `ImageError::CodecNotBuilt` is a new public error variant.

## Acceptance Criteria

- [ ] With `--features heic` (system libheif present), `Image::from_bytes(heic_bytes)` /
  `Image::load("*.heic")` decode to the canonical `Image` with correct dimensions; `optimize
  <fixture>.heic -o out.webp` and `convert <fixture>.heic --format png -o out.png` exit 0 with correct dims.
- [ ] In the **default** build (no `heic`), a `.heic` input yields `ImageError::CodecNotBuilt{codec:"HEIC",
  feature:"heic"}` → **exit 4** with "HEIC support is not built; rebuild with --features heic" — NOT a
  generic "unsupported format" or a panic. `is_heic` detection is compiled in both builds.
- [ ] The DEC-034 caps are honored under `--features heic`: a HEIC over the dimension/alloc cap →
  `ImageError::LimitsExceeded` (checked from the libheif handle dims BEFORE decode); a corrupt HEIC →
  typed `ImageError::Decode`, never a panic.
- [ ] `.heic`/`.heif` are in `IMAGE_EXTENSIONS`; a directory source discovers them (they then decode
  under the feature, or surface the exit-4 codec-not-built in the default build).
- [ ] `is_heic` distinguishes HEVC brands (`heic/heix/heim/heis/hevc/hevx`) from AVIF (`avif/avis`) and
  generic `mif1` (AVIF is dispatched first); a `.avif` is NOT mis-detected as HEIC.
- [ ] **No system/C dependency on the default path**; `cargo build --no-default-features` (lean) still
  succeeds; `just deny` green with **no new license exception** (libheif-rs/-sys are MIT).
- [ ] A CI job builds+tests `--features heic` against an installed system libheif; the feature is
  **excluded from distributed artifacts** (cargo-dist/brew build without it).
- [ ] `cargo clippy --all-targets -- -D warnings` (default + lean + `--features heic`) and `cargo fmt
  --check` clean.

## Failing Tests

Written during **design**, BEFORE build. Build makes these pass.

> **Fixture:** `tests/fixtures/heic/solid_64x48.heic` — a 64×48 solid HEIC, generated once via
> `sips -s format heic solid.png --out solid_64x48.heic` (macOS OS encoder; document the regen in a
> comment, mirroring the AVIF fixture note). Committed as a static asset (HEIC cannot be produced by a
> permissive encoder — encode needs x265/GPL, out of scope). Small; it is test data, not code.

- **`src/image/heic.rs`** (in a new `#[cfg(test)] mod tests`)
  - `"is_heic_detects_hevc_brands"` — ftyp with major/compat brand `heic`/`heix`/`hevc` → `true`;
    a PNG signature, an AVIF `ftyp…avif`, and a bare `mif1`-only ftyp → `false`.
  - `#[cfg(feature="heic")] "decode_heic_solid_dimensions"` — `decode_heic(include_bytes!(fixture),
    &Limits::default())` → `Ok`, `64×48`.
  - `#[cfg(feature="heic")] "heic_respects_dimension_cap"` — a tiny `Limits` (max dim < 64) →
    `Err(ImageError::LimitsExceeded(_))` (from the handle dims, before decode).
  - `#[cfg(feature="heic")] "corrupt_heic_is_decode_error_not_panic"` — the fixture truncated → `Err(Decode)`.
- **`src/image/mod.rs`** (in the existing `#[cfg(test)] mod tests`, a `── SPEC-062 HEIC` section)
  - `#[cfg(not(feature="heic"))] "heic_without_feature_is_codec_not_built"` —
    `Image::from_bytes(include_bytes!(fixture))` → `Err(ImageError::CodecNotBuilt{ feature: "heic", .. })`.
  - `#[cfg(feature="heic")] "heic_decodes_to_expected_dimensions"` — `Image::from_bytes(fixture)` →
    `Ok`, 64×48, `source_format == Png`.
- **`tests/input_heic.rs`**
  - `#[cfg(not(feature="heic"))] "optimize_heic_exits_4_codec_not_built"` — `optimize <fixture>.heic -o
    out.webp` → **exit 4**, stderr names `--features heic`; no panic, no output file.
  - `#[cfg(feature="heic")] "optimize_heic_input_writes_webp"` — same command → exit 0, valid WebP 64×48.
  - `"directory_source_discovers_heic"` — a temp dir with `a.heic` (+ a `.txt`) → `source::resolve`
    returns exactly the `.heic` (extension discovery works regardless of the feature).

## Implementation Context

*Read this section (and the files it points to) before starting build. The "PROBE" block was verified
firsthand during design (brew libheif 1.23.1 + libheif-rs 2.7.0 + a real `.heic`) — trust it, but
re-confirm the exact API + version feature against the pinned crate at build.*

### Decisions that apply
- **`DEC-052`** — the governing policy: HEIC decode is feature-gated, never in a distributed artifact
  (AGPL wall + HEVC patents, two independent blockers). Do NOT relitigate; this spec implements it.
- `DEC-004` — pure-Rust default + native codecs behind off-by-default features; HEIC (system libheif, C)
  is exactly such a feature. Default build exits 4 (the AVIF-encode precedent).
- `DEC-034` — decode caps; cap from the libheif handle dims BEFORE `decode`, mirror `avif::check_caps`.
- `DEC-018` / `no-agpl-default-deps` — the pure-Rust HEIC decoders are AGPL (excluded); libheif-rs/-sys
  are MIT so the crates are fine; the system libheif is LGPL (attribution, not a deny exception).
- `DEC-022` — the `webp-lossy` C-dep feature (first C dependency, vendored libwebp): the feature-gate +
  optional-dep + dedicated-CI-job precedent to mirror (but heic uses a SYSTEM lib, not vendored).
- **`DEC-056` (NEW — emit during build)** — the libheif-rs dependency + `heic` feature-gate: crate
  versions + MIT licenses, decode-only (no x265), **system-linked** (not `embedded-libheif`), the
  never-distributed constraint, and the CI approach. Satisfies `no-new-top-level-deps-without-decision`.

### PROBE — verified firsthand (2026-07-08), libheif-rs decodes real HEIC; crates are MIT
- **Licenses (authoritative):** `libheif-rs` = 2.7.0 **MIT**, `libheif-sys` = 5.3.0+1.23.0 **MIT**,
  `four-cc`/`libc` MIT/Apache. `cargo deny check licenses` needs **NO new exception** — the LGPL is the
  *system* libheif C lib, which cargo-deny does not see (contrast `ansi_colours`, an LGPL *crate*). The
  LGPL obligation (attribution + license text; dynamic/system link is clean, `embedded-libheif` static
  linking carries LGPLv3 §4 relink) is documented per DEC-052, not enforced by deny.
- **Link model:** `libheif-sys` (`links = "heif"`) uses `system-deps`/`pkg-config` → **system** libheif;
  build-deps include `bindgen` (needs libclang — system clang sufficed on macOS), `cmake`, `vcpkg`. An
  `embedded-libheif` feature vendors+builds libheif via cmake — **do not use it** (static LGPL §4 + a
  large C build); system-link is the DEC-052 clean path.
- **Decode API (compiled + ran; decoded the fixture to 64×48, first pixel [200,100,50]):**
```rust
#[cfg(feature = "heic")]
pub(crate) fn decode_heic(bytes: &[u8], limits: &Limits) -> Result<DynamicImage> {
    use libheif_rs::{LibHeif, HeifContext, ColorSpace, RgbChroma};
    let ctx = HeifContext::read_from_bytes(bytes).map_err(map_heif_err)?;
    let handle = ctx.primary_image_handle().map_err(map_heif_err)?;
    check_caps(handle.width(), handle.height(), limits)?;      // DEC-034, BEFORE decode (reuse avif::check_caps shape)
    let has_alpha = handle.has_alpha_channel();
    let chroma = if has_alpha { RgbChroma::Rgba } else { RgbChroma::Rgb };
    let lib = LibHeif::new();
    let img = lib.decode(&handle, ColorSpace::Rgb(chroma), None).map_err(map_heif_err)?;
    let iv = img.planes().interleaved.ok_or_else(|| ImageError::Decode("heic: no interleaved plane".into()))?;
    // iv.data is row-padded: copy row-by-row honoring iv.stride (stride >= width*channels).
    // Build RgbImage / RgbaImage from the tightly-packed rows → DynamicImage.
}
```
  `map_heif_err` maps `libheif_rs::HeifError` → `ImageError::Decode(...)`. Honor `iv.stride` (row
  padding) — do NOT assume `stride == width*channels`. For 10/12-bit HEIC, libheif can down-convert to
  8-bit RGB(A) (as here); leave HDR/bit-depth precision out of scope (mirror the AVIF 8-bit decision).

### Detection (`is_heic`, always compiled — mirror `avif::is_avif`)
Scan the leading `ftyp` box for HEVC major/compatible brands: `heic`, `heix`, `heim`, `heis`, `hevc`,
`hevx`. Do NOT match generic `mif1`/`msf1` (AVIF also carries `mif1`). Dispatch **AVIF first** in
`decode_with_limits`, then HEIC, so an AVIF-in-HEIF (`ftyp…avif…mif1`) routes to AVIF. Keep the bounds
checks from `is_avif` (len/box-size clamps). `is_heic` compiles in BOTH builds — it is what lets the
default build return the clear exit-4 instead of a generic error.

### Default-build behavior (exit 4, not "unsupported")
Add `ImageError::CodecNotBuilt { codec: &'static str, feature: &'static str }` (mirror
`SinkError::CodecNotBuilt`, same `#[error("{codec} support is not built; rebuild with --features
{feature}")]`) and map it to **exit 4** in `src/cli/mod.rs` (alongside the sink one). In
`decode_with_limits`, the `is_heic` branch returns it under `#[cfg(not(feature="heic"))]`. This is the
DEC-004/052 promise: the shipped binary tells the user exactly how to get HEIC, rather than failing vaguely.

### CI + distribution
- **CI job (`heic`):** install system libheif (`brew install libheif` on macOS; `apt-get install -y
  libheif-dev` on ubuntu), then `cargo build/test/clippy --all-targets --features heic`. Mirror the
  `webp-lossy` job's single-runner shape. Windows (libheif via vcpkg) is fiddly — scope the job to
  macOS + Linux and document Windows-`heic` as unsupported-for-now (the default Windows binary is
  unaffected — it just exits 4 on `.heic`).
- **Distribution (DEC-052):** confirm cargo-dist config, the Homebrew formula, and `cargo publish`
  default do NOT enable `heic`. The released binary on `.heic` → exit 4. Document `heic` as
  local-build-only (needs libheif + accepts the LGPL/patent terms).

### Constraints that apply
- `pure-rust-codecs-default` (HEIC/C stays behind the feature; default untouched), `no-agpl-default-deps`
  (pure-Rust HEIC decoders are AGPL, excluded; libheif-rs is MIT), `no-new-top-level-deps-without-decision`
  (DEC-056), `untrusted-input-hardening` (HEIC is hostile binary + a C decoder — cap before decode, honor
  stride, typed errors, fuzz target), `no-unwrap-on-recoverable-paths`, `every-public-fn-tested`,
  `clippy-fmt-clean` (incl. `--features heic`), `single-image-library` (libheif feeds the canonical
  `Image`; not a second pixel library — the AVIF/webp-lossy precedent).

### Prior related work
- `SPEC-058`/`DEC-053` (AVIF) — the ftyp-brand `is_*` + `check_caps` + `decode_with_limits` dispatch to
  mirror exactly (HEIC is the ISOBMFF sibling); AVIF is default, HEIC is gated — the patent contrast.
- `SPEC-018`/`DEC-020` (AVIF encode) + `SPEC-020`/`DEC-022` (webp-lossy) — the feature-gate + optional-dep
  + `CodecNotBuilt`/exit-4 + dedicated-CI-job precedents.
- `SPEC-060` (SVG) / `SPEC-061` (RAW) — the `source_format` = materialized-format wrinkle (HEIC → `Png`).

### Out of scope (for this spec specifically)
- HEIC in any **distributed artifact**; HEIC **encode**; a **pure-Rust** HEIC decoder; HEIF **image
  sequences** / multi-image (single primary only); **10/12-bit HDR** precision (8-bit RGB(A), mirror
  AVIF); Windows `heic` CI (document as later); AVIF/SVG/RAW (shipped).

## Notes for the Implementer

- Do the **real decode probe first** (the design probe already proved it: `brew install libheif`, a
  `sips`-made `.heic`, `cargo add libheif-rs`, decode) so the API + version feature are confirmed on the
  pinned crate before wiring — then keep `src/image/heic.rs` thin (mirror `avif.rs`).
- Verify BOTH the **default** build (HEIC → exit 4) AND `--features heic` (decode) AND the **lean**
  `--no-default-features` build; run `just deny` (must stay green, no new exception) and confirm the
  `heic` feature adds `libheif-rs`/`libheif-sys` (MIT) to the graph without tripping licenses.
- Honor the interleaved plane **stride** when copying pixels (row padding) — a `width*channels`
  assumption will corrupt non-aligned widths.
- Cap dims from the libheif **handle** BEFORE `decode` (reuse `avif::check_caps` or a shared helper) —
  do not decode first.
- MSRV: `bindgen`/`libheif-sys` may raise the floor — let the CI `msrv` job confirm and bump `ci.yml` +
  `rust-version` if needed (the AVIF lesson). Note the `msrv` job does not build `--features heic`, so
  the heic-only floor is checked by the heic job, not msrv.
- Fixture regen is `sips` (macOS) — document it; the committed `.heic` is tiny test data.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-056` — <title>
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
