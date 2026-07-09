---
# Maps to ContextCore epic-level conventions.
stage:
  id: STAGE-019
  status: shipped                   # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-009
repo:
  id: crustyimg

created_at: 2026-07-08
shipped_at: 2026-07-08

value_contribution:
  advances: >
    Closes the input-reach wave with the honest, opt-in slice: HEIC/HEIF decode
    behind an off-by-default `heic` cargo feature (system libheif, decode-only),
    so a user who accepts the terms and has libheif installed can read `.heic`
    end to end — while the DEFAULT distributed binary stays pure-Rust, zero-system-dep,
    and free of both the AGPL and the HEVC-patent exposure, surfacing a clear
    "codec not built" (exit 4) on `.heic`. This is the deliberate NON-default
    counterpart to AVIF/SVG/RAW, per DEC-052.
  delivers:
    - "`--features heic` builds decode `.heic`/`.heif` (system libheif, decode-only) to the canonical `Image`"
    - "The DEFAULT binary DETECTS `.heic` and exits 4 with a clear 'HEIC support is not built; rebuild with --features heic' — not a vague 'unsupported format'"
    - "HEIC decode is bounded by the DEC-034 caps + libheif's own security limits; typed errors, no panics"
    - "A recorded decision (DEC-056) for the libheif-rs dependency + the feature-gate + why HEIC is never in a distributed artifact (DEC-052)"
  explicitly_does_not:
    - "Ship HEIC in ANY distributed artifact (cargo-dist release / Homebrew bottle / crates.io default) — feature is local-build-only (DEC-052: AGPL wall + HEVC patents)"
    - "Encode HEIC (that adds x265/GPL + encoder-side patents) — decode-only (WITH_X265=OFF)"
    - "Put HEIC decode on the default path or add any system/C dependency to the default build"
    - "Adopt a pure-Rust HEIC decoder (the mature ones are AGPL; permissive ones are immature AND the HEVC patent blocker is orthogonal to code license — DEC-052)"
---

# STAGE-019: HEIC decode behind an off-by-default `heic` feature

## What This Stage Is

The stage that lets crustyimg read **HEIC/HEIF** — but **only** as an explicit,
local opt-in, never in a shipped binary. Per **DEC-052**, HEIC decode is gated
behind an off-by-default `heic` cargo feature backed by **system libheif**
(decode-only: libde265, no x265), because two independent blockers keep it out of
the default: the mature pure-Rust decoders are **AGPL**, and **HEVC is patent-
encumbered** regardless of code license. So `cargo build --features heic` (with
libheif installed) reads `.heic` end to end, while the default distributed binary
stays pure-Rust and zero-system-dep and returns a clear **exit 4 "codec not built"**
on a `.heic`. This is the deliberate counterpart to the AVIF/SVG/RAW default inputs
— same input-format wiring (ftyp-brand detection before the generic decoder, typed
errors, DEC-034 caps), but feature-gated and honest about the wall. It closes
PROJ-009 (roadmap Wave 1).

## Why Now

- **Closes the wave honestly.** AVIF/SVG/RAW gave the default binary permissive,
  patent-clean input reach; HEIC is the one remaining common format that *cannot*
  be default. Shipping it as a clearly-labeled opt-in (rather than not at all)
  gives Mac/Linux users who accept the terms an in-tool path, matching the
  whole-industry posture (Firefox/Fedora/libvips/image-rs all gate HEVC out of
  the default). DEC-052 already made the call; this stage implements it.
- **The wiring is de-risked.** The design probe (2026-07-08) confirmed the
  libheif-rs stack decodes a real HEIC on this machine and that the Rust crates
  are permissive (MIT) — so the remaining work is feature-gating, the exit-4
  default path, a system-lib CI job, and untrusted-input hardening.

## Success Criteria

- `cargo build --features heic` (system libheif present) decodes `.heic`/`.heif` to
  the canonical `Image`; `optimize`/`convert`/`info` operate on `.heic` end to end
  under the feature, honoring the DEC-034 caps; a corrupt/oversize HEIC → typed
  `ImageError` (never a panic).
- The **default** build (`cargo build`, no `heic`) detects `.heic` by container brand
  and returns a clear **exit 4** "HEIC support is not built; rebuild with --features
  heic" — not a generic "unsupported format" — mirroring DEC-004's AVIF-encode behavior.
- **No system/C dependency on the default path**; `cargo build --no-default-features`
  (lean) still succeeds; `just deny` green (the libheif-rs/-sys crates are MIT — no
  new license exception; the LGPL is the *system* C lib, documented per DEC-052).
- A CI job builds+tests `--features heic` against an installed system libheif; the
  feature is **excluded from every distributed artifact** (cargo-dist / brew / publish).
- A **DEC-056** records the libheif-rs dependency + the feature-gate + the
  decode-only/system-link/never-distributed constraints (implementing DEC-052).

## Scope

### In scope
- A `src/image/heic.rs` module: `is_heic(bytes)` (ftyp HEVC-brand sniff) + a
  `#[cfg(feature = "heic")]` `decode_heic(bytes, limits)` (libheif-rs → interleaved
  RGB(A) → canonical `Image`, DEC-034-capped). Dispatch in `decode_with_limits`
  before the generic decoder; a decode-side `ImageError::CodecNotBuilt { codec,
  feature }` (→ exit 4) for the feature-off path. `.heic`/`.heif` in
  `IMAGE_EXTENSIONS`. The `heic = ["dep:libheif-rs"]` feature, a system-lib CI job,
  LGPL attribution docs, tests, DEC-056. **(SPEC-062)**

### Explicitly out of scope
- HEIC **in any distributed artifact** (local-build-only); HEIC **encode**; a
  **pure-Rust** HEIC decoder (AGPL/immature + patent-orthogonal, DEC-052); AVIF/SVG/
  RAW inputs (shipped); image sequences / multi-image HEIF (single primary image only).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-062 (shipped on 2026-07-08) — HEIC decode behind `--features heic`: `src/image/heic.rs`
  (ftyp-brand `is_heic` in both builds + `#[cfg(feature="heic")]` libheif-rs decode → canonical `Image`,
  cap-before-decode, stride-honoring), new `ImageError::CodecNotBuilt`→exit-4 for the default build,
  `.heic`/`.heif` in `IMAGE_EXTENSIONS`, `heic = ["dep:libheif-rs"]` (MIT crates, **no deny exception**),
  a system-libheif CI job (`libheif-plugin-libde265` on ubuntu), excluded from distributed artifacts, LGPL
  attribution (docs/licensing.md), DEC-056. PR #68 (b2f370a), 24/24 CI green; also fixed a lint
  false-positive this change introduced (valid `.heic` reported "corrupt").

**Count:** 1 shipped / 0 active / 0 pending — single-spec stage complete.

## Design Notes

- **PROBE RESULT (2026-07-08) — the libheif-rs stack decodes real HEIC here, and the Rust crates are
  PERMISSIVE (MIT).** A firsthand probe (brew `libheif` 1.23.1 + a real `.heic` made via `sips`, macOS
  OS encoder) confirmed: `libheif-rs` = **2.7.0 (MIT)**, `libheif-sys` = **5.3.0+1.23.0 (MIT)**, linking
  **system** libheif via `system-deps`/pkg-config (an `embedded-libheif` cmake/vendored option also
  exists). Decoded the fixture to 64×48 with correct pixels via `LibHeif::new().decode(&handle,
  ColorSpace::Rgb(RgbChroma::Rgb), None)` → `img.planes().interleaved`. **Key licensing nuance:** unlike
  `ansi_colours` (an LGPL *crate* needing a `deny.toml` exception), here the LGPL is the **system C
  library** — invisible to `cargo deny`, which sees only the MIT Rust crates → **`just deny` stays green
  with NO new exception**. The LGPL obligation (attribution + license text; dynamic/system link is the
  clean path, static/`embedded-libheif` carries LGPLv3 §4 relink) is real and documented per DEC-052,
  not enforced by deny.
- **DEC-056 (at build):** adopt `libheif-rs` (optional, MIT) behind `heic = ["dep:libheif-rs"]`,
  decode-only, **system-linked** (not `embedded-libheif` — keeps the LGPL clean + no giant vendored C
  build). Records the dependency, the feature-gate, decode-only (no x265), system-vs-embedded choice,
  and the never-distributed constraint (implementing DEC-052; satisfies
  `no-new-top-level-deps-without-decision`).
- **Two independent reasons this is NOT default (DEC-052, do not relitigate):** (1) the AGPL wall on
  mature pure-Rust HEIC decoders; (2) HEVC patents (Access Advance pool) attach to *every* decode path
  regardless of code license. Even a future permissive pure-Rust decoder would NOT un-gate HEIC — the
  patent question is separate (counsel, not code). So `heic` uses system libheif (LGPL, the cleanest
  in-tool path) and never ships in a distributed artifact.
- **The default build must EXIT 4 with a clear message, not "unsupported".** `ImageError` has no
  decode-side "codec not built" today (`Decode`/`UnsupportedFormat`/`LimitsExceeded`/`Io`). Add
  `ImageError::CodecNotBuilt { codec, feature }` (→ exit 4), mirroring `SinkError::CodecNotBuilt`
  (sink/mod.rs). In `decode_with_limits`: detect `heic::is_heic(bytes)` and, with the feature OFF,
  return `CodecNotBuilt{ codec: "HEIC", feature: "heic" }` — the DEC-004/052 behavior (the AVIF-encode
  exit-4 precedent, but on the decode side).
- **Detection mirrors AVIF, on HEVC brands.** `is_heic` scans the `ftyp` box for HEVC-specific major/
  compatible brands (`heic`, `heix`, `heim`, `heis`, `hevc`, `hevx`) — NOT the generic `mif1`/`msf1`
  (which AVIF also carries). Dispatch **AVIF first** (`avif`/`avis`) then HEIC, so an AVIF-in-HEIF isn't
  mis-routed. Both are ISOBMFF/`ftyp`; reuse the `avif::is_avif` shape.
- **CI needs a system library (a first).** `avif` is pure-Rust and `webp-lossy` vendors libwebp via
  `cc`; `heic` is the first feature needing an actual **system** lib + `pkg-config` + `bindgen`
  (libclang). Add a dedicated job: install libheif (`brew install libheif` on macOS / `apt-get install
  libheif-dev` on ubuntu), then `cargo build/test/clippy --features heic`. Likely macOS + Linux only
  (Windows libheif via vcpkg is fiddly — document as unsupported-for-now, or a stretch). Single-runner,
  like the webp-lossy job.
- **Distribution excludes `heic` (DEC-052).** cargo-dist release builds, the Homebrew formula, and
  `cargo publish`/install default all build WITHOUT `heic`. Verify the dist config does not enable it;
  document that `heic` is local-build-only. Running the released binary on `.heic` → exit 4.
- **`source_format` wrinkle (no `ImageFormat::Heic`).** Same as SVG→`Png` / RAW→`Jpeg`: report a
  materialized raster format for a decoded HEIC (`Png`). Accepted wart; the shared `SourceFormat` enum
  is the standing follow-up.
- **Security (untrusted binary; C decoder).** libheif is C and historically CVE-prone: cap dims from
  `handle.width()/height()` BEFORE `decode` (DEC-034), honor the interleaved plane **stride** (row
  padding) when copying to `RgbImage`/`RgbaImage`, set libheif's own security limits if the binding
  exposes them, and check `handle.has_alpha_channel()` → decode `Rgba` vs `Rgb`. Typed errors on every
  libheif failure. A `cargo-fuzz` target needs libheif + nightly → carry as a pre-1.0 gate (parity with
  avif/svg/raw).

## Dependencies

### Depends on
- Shipped decode seam (`src/image/mod.rs` `decode_with_limits` + the AVIF ftyp-brand precedent),
  `src/image/avif.rs` (`is_avif` shape + `check_caps`), `src/source/mod.rs` (`IMAGE_EXTENSIONS`),
  `src/error.rs`, `src/sink/mod.rs` (`SinkError::CodecNotBuilt`/exit-4 pattern to mirror on decode).
- DEC-052 (the HEIC-feature-gate policy — the governing decision), DEC-004 (pure-Rust default +
  feature-gated native), DEC-034 (decode caps), DEC-018 (`no-agpl-default-deps`), DEC-022 (the
  `webp-lossy` C-dep feature precedent).
- External: system libheif (LGPL, decode-only) at build+run of the `heic` feature.

### Enables
- Closes PROJ-009 (input reach). The `is_heic` brand detection + the ISOBMFF familiarity also inform any
  future permissive HEIC path if the patent question is ever cleared (DEC-052).

## Stage-Level Reflection

- **Did we deliver the outcome in "What This Stage Is"?** Yes — `--features heic` decodes `.heic`/`.heif`
  via system libheif into the canonical `Image`, and the DEFAULT binary stays pure-Rust/zero-system-dep,
  detecting `.heic` and returning a clear exit-4 ("rebuild with --features heic") instead of a vague
  error. PR #68, 24/24 CI green; `just deny` green with no new exception (MIT Rust crates; LGPL is the
  system C lib). The honest opt-in per DEC-052 (AGPL wall + HEVC patents).
- **How many specs did it actually take?** 1 (SPEC-062) — as planned; a single-spec stage like the other
  three, but the heaviest wiring (a new error variant + a system-lib CI job + distribution discipline).
- **What changed between starting and shipping?** The design probe under-specified the *system-library*
  reality: no bindgen (pre-generated bindings, so no MSRV move), the API-version feature had to be pinned
  to `v1_17` (the crate default `v1_21` outran Ubuntu's apt libheif and would silently break CI — at the
  cost of `set_security_limits`, so our DEC-034 pre-check is the load-bearing bound), and Ubuntu's
  `libheif-dev` links+parses but cannot *decode* without `libheif-plugin-libde265` (Homebrew bundles it,
  so mac testing couldn't catch it). Verify also caught nothing new — the build had already found + fixed
  the lint false-positive this change introduced.
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - **A new `IMAGE_EXTENSIONS` entry OR a new `ImageError` variant needs an audit of every decode caller
    and `Err(_)` catch-all, not just the exit-code map.** This bit SPEC-061 (`info <raw>`) and again here
    (lint called a valid `.heic` "truncated or corrupt … re-export a valid image", exit 7, on any
    iPhone-photo directory). The tripwire is `decisions-audit --changed`, so the DEC's `affected_scope`
    must list the caller files (cli/lint), not just the codec module — added at ship.
  - **For a feature depending on a versioned system library:** pin the API-version feature to the OLDEST
    distro-shipped lib, install the decoder *backend* (not just the dev headers) in CI, and expect the
    first CI run — not local mac testing — to surface the environment truth.
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - Yes — the decode-caller-audit lesson generalizes to every input format (it has now recurred twice)
    and belongs in the shared `image-extensions-expose-every-decode-caller` note, not just this stage.
  - <one-line items>
