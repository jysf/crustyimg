---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-056
  type: decision
  confidence: 0.9
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
  - src/image/heic.rs
  - src/image/mod.rs
  - src/error.rs
  - src/cli/mod.rs
  - src/lint/mod.rs
  - src/source/mod.rs
  - dist-workspace.toml
  - .github/workflows/ci.yml
  - fuzz/**

tags:
  - codecs
  - heic
  - dependencies
  - licensing
  - patents
  - feature-gate
  - input-reach
---

# DEC-056: HEIC decode via `libheif-rs` → **system** libheif, behind the off-by-default `heic` feature

## Decision

Adopt **`libheif-rs = "=2.7.0"`** (MIT) → `libheif-sys = "=5.3.0+1.23.0"` (MIT) as an
**optional** dependency behind `heic = ["dep:libheif-rs"]`, to implement the policy DEC-052
already set (HEIC decode is feature-gated, never in a distributed artifact). Five choices
inside that adoption:

1. **System-linked, not vendored.** `default-features = false`, so `libheif-sys` finds the
   system libheif via `pkg-config`/`system-deps`. We do **not** enable `embedded-libheif`,
   which would cmake-build a vendored copy and statically link it — that pulls LGPLv3 §4's
   relink obligation onto our binary and adds a large C build. Dynamic/system linking
   satisfies the LGPL by construction.
2. **`v1_17`, the lowest API floor.** `libheif-rs`'s default `latest` feature means `v1_21`,
   which makes `system-deps` demand a system libheif ≥ 1.21 — more than Ubuntu 24.04's apt
   package (1.17.6) provides. Pinning `v1_17` builds against everything from 1.17 to 1.23+
   and keeps the CI job on stock `ubuntu-latest`. The cost: `HeifContext::set_security_limits`
   is gated on `v1_19` and is therefore unavailable to us (see Consequences).
3. **Decode only.** No HEVC encoder is built or exposed (x265 is GPL, and encoder-side patent
   tiers are separate). `crustyimg` never writes `.heic`.
4. **Detection is unconditional; decode is not.** `heic::is_heic` (an ISOBMFF `ftyp` HEVC-brand
   scan) compiles into **both** builds. That is what lets the default binary answer a `.heic`
   with the new `ImageError::CodecNotBuilt { codec: "HEIC", feature: "heic" }` → **exit 4**,
   naming the feature, instead of a vague `UnsupportedFormat`.
5. **Never distributed.** cargo-dist, the Homebrew formula, and `cargo install`/`cargo publish`
   defaults all build without `heic` (guarded by a comment in `dist-workspace.toml`). A dedicated
   CI job — the project's first needing a real system library — is the only place it is built.

`libheif-rs` is a **codec feeding the canonical `Image`**, not a second pixel library, so
`single-image-library` is not tripped (the AVIF/webp-lossy precedent, DEC-053/DEC-022).

## Context

STAGE-019/SPEC-062 is the last stage of PROJ-009 (input reach). AVIF (DEC-053), SVG (DEC-054),
and RAW (DEC-055) all landed on the **default** path: pure-Rust, permissive, patent-clean. HEIC
cannot, for two independent reasons pinned in **DEC-052** — the mature pure-Rust HEIC decoders
are AGPL (`no-agpl-default-deps`), and HEVC is patent-encumbered (Access Advance) on every decode
path regardless of code license. DEC-052 decided *that* HEIC is gated; this decision records
*how*, and which crate.

A design-time probe (2026-07-08, Homebrew libheif 1.23.1 + a real `.heic` produced by macOS
`sips`) decoded the fixture to 64×48 with correct pixels, and confirmed the licenses. A second
probe at build time corrected two design assumptions (see Validation).

### The licensing nuance that keeps `just deny` green

`cargo deny` sees **crates**, and both Rust crates here are **MIT**. The LGPL belongs to the
**system C library**, which is not in the cargo graph — so `just deny` passes with **no new
exception**. Contrast `ansi_colours`, an LGPL *crate*, which needed one. The LGPL obligation is
real but is discharged by **attribution** (`docs/licensing.md`, per DEC-052), not by `deny.toml`.
This is a general lesson: a green `cargo deny` says nothing about system libraries you link.

## Alternatives Considered

- **`embedded-libheif` (vendor + cmake-build libheif).** Rejected: static linking triggers
  LGPLv3 §4 relink obligations on our artifact, and a vendored C build is a large, slow,
  cross-platform liability. System linking is the clean path DEC-052 calls for.
- **`libheif-rs` default features (`latest` = `v1_21`).** Rejected: raises the system-libheif
  floor above the stock `ubuntu-latest` apt package, breaking the CI job for no gain — we use
  none of the ≥1.18 API. Also drags the crate's own `image` feature onto a *second* `image`
  crate version (`single-image-library`).
- **`use-bindgen`.** Not needed: `libheif-sys` ships pre-generated bindings per API version,
  so the feature adds **no** bindgen/libclang/cmake build-tool dependency. (The design context
  assumed bindgen would be present and might raise MSRV; it is not, and does not.)
- **A pure-Rust HEIC decoder** (`imazen/heic`, `ente/heic_decoder`, `rust_h265`). Rejected by
  DEC-052: AGPL, or immature — and the patent blocker is orthogonal to code license, so *no*
  decoder choice un-gates HEIC.
- **`image`'s own HEIF support / no HEIC at all.** `image` has none; "no HEIC at all" leaves the
  most-requested modern input unreachable even for users who accept the terms locally.

## Consequences

- **Default build unchanged.** `cargo tree -e normal` shows zero libheif; the pure-Rust,
  zero-system-dep promise (DEC-004) holds. `.heic` → exit 4 with an actionable message.
- **`just deny` stays green with no new exception** (verified: `advisories ok, bans ok,
  licenses ok, sources ok` with `[graph] all-features = true`, which *does* pull libheif-rs in).
- **MSRV is unmoved.** The `--features heic` tree's highest `rust-version` is 1.90 (`avif-parse`,
  already the floor); `libheif-rs` needs 1.82. No `ci.yml` / `rust-version` bump.
- **We cannot tighten libheif's internal security limits** at the `v1_17` floor. Acceptable:
  libheif ≥ 1.19 applies its own defaults regardless, and the load-bearing bound is ours — the
  DEC-034 dimension/alloc cap is checked from the image **handle**, before `decode` allocates a
  plane. Revisit `v1_19` when the CI runner's libheif floor moves.
- **Windows `heic` is unsupported** (libheif via vcpkg is fiddly). The default Windows binary is
  unaffected — it exits 4 on `.heic` like every other released binary. The CI job runs on
  ubuntu + macOS.
- **A C decoder with a long CVE history is now reachable** under the feature. Mitigations live in
  `src/image/heic.rs`: caps before decode, a checked stride-honoring row copy, storage-depth
  validation, typed errors on every libheif failure, and `fuzz/fuzz_targets/heic_decode.rs`.
- **`source_format` for a decoded HEIC is `Png`.** There is no `ImageFormat::Heic`; this follows
  the SVG→`Png` / RAW→`Jpeg` materialized-raster convention. The shared `SourceFormat` enum
  remains the standing follow-up.

## Validation

Probed firsthand on this machine (Homebrew libheif 1.23.1, macOS), then re-run at build:

- **Decode.** `HeifContext::read_from_bytes` → `primary_image_handle()` → `LibHeif::new().decode(
  &handle, ColorSpace::Rgb(RgbChroma::Rgb), None)` → `planes().interleaved` returned a 64×48
  plane, `stride = 192`, `storage_bits_per_pixel = 24`, first pixel `[200,100,50]` — the fixture's
  color. Truncated input → `InvalidInput ... Insufficient input data`, mapped to `ImageError::Decode`.
- **Two design assumptions corrected.** (a) `bindgen` is **not** in the graph — bindings are
  pre-generated, so no libclang and no MSRV pressure. (b) `security_limits`/`set_security_limits`
  are `#[cfg(feature = "v1_19")]`, so they are unreachable at the `v1_17` floor we chose for
  distro compatibility.
- **Gates.** `cargo test` (default, 582), `cargo test --features heic` (588, incl. 12 HEIC unit +
  4 integration), `cargo test --no-default-features` (lean, 582), `clippy --all-targets -D warnings`
  on all three, `cargo fmt --check`, `just deny` — all green.
- **End to end.** Default binary: `optimize photo.heic` → `error: HEIC support is not built;
  rebuild with --features heic`, exit 4, no output file. `--features heic` binary: `info photo.heic`
  → `64x48`; `optimize photo.heic -o out.webp` → exit 0.

## References

- `decisions/DEC-052` — the governing policy (AGPL wall + HEVC patents). This decision implements it.
- `decisions/DEC-004` — pure-Rust default; native codecs behind off-by-default features.
- `decisions/DEC-022` — `webp-lossy`, the C-dependency feature-gate + dedicated-CI-job precedent.
- `decisions/DEC-034` — decode resource caps (applied from the libheif handle, before decode).
- `decisions/DEC-018` — `no-agpl-default-deps`, and `deny.toml`'s enforcement.
- `docs/licensing.md` — the LGPL attribution + HEVC patent notice for `--features heic` builds.
- `docs/research/heic-input-reach-spike.md` — the spike behind DEC-052.
- `projects/PROJ-009-input-reach/specs/SPEC-062-heic-decode-feature-gated.md`
