---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-062
  type: story
  cycle: ship  # frame | design | build | verify | ship
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
    - cycle: build
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 260000
      estimated_usd: 2.34
      duration_minutes: 18
      recorded_at: 2026-07-08
      notes: >
        Build ran in the main loop (interactive, not a separately-metered subagent), so `/cost` was
        not readable programmatically — tokens_total is an ORDER-OF-MAGNITUDE ESTIMATE per the
        autonomous-run-cost practice (labelled estimate, not null, so `just cost-audit` passes at
        ship). estimated_usd = 260k tokens × Opus 4.8 list ($5/$25 per MTok, ~80/20 in/out) ≈ $2.34.
        Work: re-ran the libheif probe (which corrected two design assumptions — no bindgen; the
        v1_17 API floor), src/image/heic.rs (is_heic in both builds + feature-gated capped,
        stride-honoring decode), ImageError::CodecNotBuilt → exit 4, decode_with_limits dispatch after
        AVIF, .heic/.heif extensions, `heic` feature, sips-made fixture, unit + integration tests,
        fuzz/heic_decode, a system-libheif CI job, docs/licensing.md (LGPL + patents), dist guard,
        DEC-056. All gates green: test/clippy on default + lean + --features heic, fmt, `just deny`
        (no new exception), end-to-end exit-4 and decode runs.
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 230000
      estimated_usd: 2.07
      duration_minutes: null
      recorded_at: 2026-07-08
      notes: >
        Fresh verify session — ✅ APPROVED. Re-ran all three builds independently (default 582,
        --features heic 588, lean 582, clippy×3, fmt, just deny green with git diff main -- deny.toml
        empty), re-audited all 8 Err(_) catch-alls in src/ (only the lint site needed the fix, confirmed),
        proved cap-before-decode on the handle, and — beyond the tests — made a 67×45 HEIC to prove the
        stride-padding path (stride 208 vs row_bytes 201, unsheared). Verified the ubuntu heic job really
        DECODES (decode_heic_solid_dimensions passed on ubuntu-latest with libheif-plugin-libde265), the
        plugin gotcha via the actual failed CI run (140f26b2), and distribution excludes heic. Pulled CI
        rows via gh api (24 success / 5 skipped-dist / 0 fail). Main-loop → ORDER-OF-MAGNITUDE ESTIMATE
        per §4: ~230k × Opus 4.8 list ($5/$25, ~80/20) ≈ $2.07. 3 non-blocking ship items filed.
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-08
      notes: >
        Ship bookkeeping (squash-merge #68, cost/reflection/totals/archive/stage-ship + PROJ-009
        project-ship, roadmap gates, the 3 verify ship-items) — main-loop, not separately metered →
        null-with-note per AGENTS §4.
  totals:
    tokens_total: 490000
    estimated_usd: 4.41
    session_count: 4
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

- [x] With `--features heic` (system libheif present), `Image::from_bytes(heic_bytes)` /
  `Image::load("*.heic")` decode to the canonical `Image` with correct dimensions; `optimize
  <fixture>.heic -o out.webp` and `convert <fixture>.heic --format png -o out.png` exit 0 with correct dims.
- [x] In the **default** build (no `heic`), a `.heic` input yields `ImageError::CodecNotBuilt{codec:"HEIC",
  feature:"heic"}` → **exit 4** with "HEIC support is not built; rebuild with --features heic" — NOT a
  generic "unsupported format" or a panic. `is_heic` detection is compiled in both builds.
- [x] The DEC-034 caps are honored under `--features heic`: a HEIC over the dimension/alloc cap →
  `ImageError::LimitsExceeded` (checked from the libheif handle dims BEFORE decode); a corrupt HEIC →
  typed `ImageError::Decode`, never a panic.
- [x] `.heic`/`.heif` are in `IMAGE_EXTENSIONS`; a directory source discovers them (they then decode
  under the feature, or surface the exit-4 codec-not-built in the default build).
- [x] `is_heic` distinguishes HEVC brands (`heic/heix/heim/heis/hevc/hevx`) from AVIF (`avif/avis`) and
  generic `mif1` (AVIF is dispatched first); a `.avif` is NOT mis-detected as HEIC.
- [x] **No system/C dependency on the default path**; `cargo build --no-default-features` (lean) still
  succeeds; `just deny` green with **no new license exception** (libheif-rs/-sys are MIT).
- [x] A CI job builds+tests `--features heic` against an installed system libheif; the feature is
  **excluded from distributed artifacts** (cargo-dist/brew build without it).
- [x] `cargo clippy --all-targets -- -D warnings` (default + lean + `--features heic`) and `cargo fmt
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

- **Branch:** `feat/spec-062-heic-decode`
- **PR (if applicable):** #68
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - `DEC-056` — HEIC decode via `libheif-rs` → **system** libheif, behind the off-by-default `heic` feature
- **Deviations from spec:**
  - **`default-features = false, features = ["v1_17"]`, not bare `default-features = false`.**
    `libheif-sys` selects its pre-generated bindings by version feature and `compile_error!`s if none
    is set; its default (`latest` = `v1_21`) would make `system-deps` demand a system libheif ≥ 1.21,
    more than `ubuntu-latest`'s apt package (1.17.6) provides. `v1_17` builds against 1.17 → 1.23+.
  - **No `bindgen`/libclang in the graph** (the spec's Implementation Context expected it). Bindings
    are pre-generated; `use-bindgen` is off by default. So no build-tool dependency and no MSRV
    pressure — the `--features heic` tree's floor is 1.90 (`avif-parse`, already ours). **No
    `ci.yml`/`rust-version` bump**, contrary to the spec's "MSRV may move" note.
  - **libheif's own security limits are NOT set.** `set_security_limits` is `#[cfg(feature = "v1_19")]`,
    unreachable at the `v1_17` floor. The stage's "set them if the binding exposes them" note is
    therefore unmet by choice; libheif ≥ 1.19 applies its defaults regardless, and the DEC-034
    handle-dim pre-check is the load-bearing bound. Recorded in DEC-056 with a revisit trigger.
  - **CI job runs on ubuntu + macOS (a 2-OS matrix)**, not a single runner as the spec suggested —
    the two libheif install paths (apt/brew) are exactly what the job needs to prove.
  - **Fixed a defect this spec introduced in `lint`** (not in the spec's scope, but caused by it).
    Adding `.heic` to `IMAGE_EXTENSIONS` made `lint` discover HEIC files, and its
    `size/truncated-or-corrupt` rule matched `Err(_)` — so in the DEFAULT build a valid `.heic`
    was reported as *"image is truncated or corrupt; re-export a valid image"* (Error → exit 7).
    A false diagnosis with a destructive remedy, firing on any directory of iPhone photos.
    `TruncatedOrCorrupt::check` now skips `ImageError::CodecNotBuilt`, with a regression test.
    This is the SPEC-061 `info <raw>` lesson recurring: **adding an extension to `IMAGE_EXTENSIONS`
    exposes it to every caller that decodes, and a new `ImageError` variant needs an audit of
    every `Err(_)` catch-all**, not just the exit-code map.
  - **Ubuntu's `libheif-dev` cannot decode HEVC.** The first CI run failed on ubuntu-latest with
    `UnsupportedFeature(UnsupportedCodec)` — Debian/Ubuntu split libheif's codec backends into
    separate plugin packages, so the job also installs `libheif-plugin-libde265`. Linking, the
    `v1_17` floor, and container parsing were all fine; only decode failed. Documented in README
    + `docs/licensing.md`. (Homebrew's libheif bundles its backends, which is why local macOS
    testing could not have caught this — the CI matrix did.)
  - **Extra tests beyond the spec's list:** `info_heic_exits_4_codec_not_built` and
    `info_heic_reports_dimensions` (the SPEC-061 lesson: extension/byte path splits break `info`),
    `avif_is_not_mis_detected_as_heic`, an alloc-cap test, and an `is_heic` oversized-box-size bounds
    test. Added `docs/licensing.md` (the spec said "docs/" without naming a file).
- **Follow-up work identified:**
  - Windows `heic` (libheif via vcpkg) — documented as unsupported; a stretch spec if demand appears.
  - Move to `v1_19` + `set_security_limits` once the CI runner's stock libheif floor reaches 1.19.
  - HEIC alpha (`RgbChroma::Rgba`) is implemented but untested — `sips` produces no alpha HEIC. A
    fixture from another encoder would close it.
  - The `source_format` wart (HEIC/SVG → `Png`, RAW → `Jpeg`) — the shared `SourceFormat` enum remains
    the standing follow-up, now with a third instance.
  - `fuzz/heic_decode` needs libheif + nightly → carry as a pre-1.0 hardening gate (parity with avif/svg/raw).
  - **A `meta/not-inspected` Info lint finding.** `lint` now stays silent on a `.heic` it cannot
    decode, which is better than a false "corrupt" error but still reports the file as scanned-and-
    clean. An Info-severity "not inspected (codec not built)" finding would be the honest answer;
    `size/truncated-or-corrupt` cannot carry it because its severity is fixed at Error.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing blocked me; the spec was unusually build-ready. The one thing it got *wrong* rather than
   unclear was the dependency shape: it predicted `bindgen`/libclang and an MSRV bump, and left the
   version feature as "or with the version feature the build confirms". Re-running the probe first (as
   the spec instructed) surfaced the `v1_17`-vs-`latest` question — which turned out to be the single
   most consequential decision in the build, because it silently determines whether the CI job can use
   stock `ubuntu-latest`. A design that had probed `apt-cache policy libheif-dev` would have named it.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No missing constraint. But the spec's Security note ("set libheif's own security limits if the
   binding exposes them") and its CI note ("apt-get install libheif-dev") are in latent conflict: the
   ubuntu package's version is exactly what makes those limits unreachable. Worth stating as a rule:
   when a feature links a *system* library, the design should pin the minimum system version, because
   it constrains both the API surface and the CI runner.
3. **If you did this task again, what would you do differently?**
   — Probe the *distro package versions*, not just the local Homebrew one, before writing the CI job.
   I wired `--features heic` against my 1.23.1 and only then worked backwards to the floor that
   `ubuntu-latest` could satisfy. Ten seconds of `apt-cache policy` (or reading libheif-sys's
   `[package.metadata.system-deps]` table) would have set the version feature first, in one pass.

---

## Reflection (Ship)

1. **What would I do differently next time?**
   — Two probe assumptions in the spec were wrong and cost CI round-trips: it predicted bindgen/libclang
   + a possible MSRV bump (neither — libheif-sys ships pre-generated bindings), and it did not pin the
   libheif API version, so libheif-rs's `latest` (v1_21) silently demanded a system libheif newer than
   Ubuntu's apt package. **Lesson: when a feature depends on a versioned system library, the design probe
   must pin the API-version feature to the OLDEST distro-shipped lib (here `v1_17` = Ubuntu 1.17.6), not
   the crate default.** And the biggest bite was environmental, not code: macOS Homebrew bundles the HEVC
   backend, so no amount of local mac testing catches that Ubuntu's `libheif-dev` links + parses but
   cannot decode without `libheif-plugin-libde265`. Probe on the target distro, or expect the first CI run
   to teach you.
2. **Does any template, constraint, or decision need updating?**
   — No template/constraint change. DEC-056 emitted (libheif dep + system-link + never-distributed +
   the v1_17/`set_security_limits` trade-off with a revisit trigger). The reusable lesson —
   **[[image-extensions-expose-every-decode-caller]]** — is the one to keep: a new `IMAGE_EXTENSIONS`
   entry OR a new `ImageError` variant needs an audit of *every* decode caller and `Err(_)` catch-all,
   not just the exit-code map. It bit SPEC-061 (`info <raw>`) and again here (lint called a valid `.heic`
   "corrupt"). At ship I added `src/cli/mod.rs` + `src/lint/mod.rs` to DEC-056's `affected_scope` so
   `just decisions-audit --changed` will warn the next editor of those files (verify item a).
3. **Is there a follow-up spec I should write now before I forget?**
   — No new spec now, but tracked follow-ups (in `docs/roadmap.md`): run `cargo +nightly fuzz run
   heic_decode` (pre-1.0 gate, parity with avif/svg/raw); a **stride-padding test with an odd-width
   fixture** (the current 64px fixture can't exercise row padding — proven correct at verify but untested;
   verify item b); Windows `heic` (vcpkg) + the `v1_19` `set_security_limits` upgrade; HEIC alpha
   coverage; and the shared `SourceFormat` enum so `info x.heic` need not report `png`. This is the LAST
   spec of PROJ-009 — shipping it project-ships the input-reach wave.
