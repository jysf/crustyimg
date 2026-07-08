---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-053
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

created_at: 2026-07-07
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
  - avif
  - pure-rust
---

# DEC-053: AVIF decode dependency — `re_rav1d` (no-asm) + `avif-parse` on the default path

## Decision

AVIF **decode** on the default, pure-Rust build is provided by two permissive CODEC
dependencies feeding the canonical `Image` (the webp-lossy precedent, DEC-021/DEC-022 —
**not** a second pixel library, so `single-image-library` is not tripped):

- **`re_rav1d` = "=0.1.3"** (BSD-2-Clause, rerun's combined `rav1d` + `dav1d-rs` fork),
  built **no-asm** (`default-features = false`, features `bitdepth_8`, `bitdepth_16`) →
  pure-Rust, **zero build-tool deps (no nasm/meson)**, clean on Linux/macOS/Windows. We
  use its re-exported **safe `dav1d` Rust API** (`re_rav1d::dav1d::{Decoder, Picture, …}` —
  `Decoder::new`/`send_data`/`get_picture`/plane access), not the raw C ABI.
- **`avif-parse` = "=2.1.0"** (MPL-2.0, kornelski / Firefox `mp4parse`-derived) parses the
  ISOBMFF/MIAF container to the primary-item + optional alpha AV1 OBU streams and **rejects
  grid/tiled collages cleanly** (`Error::Unsupported`).

Our glue (`src/image/avif.rs`, kept thin so the fork is swappable) converts the decoded YUV
planes to 8-bit RGB(A), honoring bit depth (8/10/12 via u8/u16 samples), chroma subsampling
(4:2:0 / 4:2:2 / 4:4:4), YUV range (full/limited), matrix coefficients (BT.601/709/2020 +
GBR identity), and premultiplied alpha (MIAF `prem`). Decode is dispatched in
`decode_with_limits` by ISOBMFF `ftyp` brand **before** the generic `ImageReader` path;
`image`'s own AVIF decoder (dav1d/C) is never used (DEC-004). This also satisfies
`no-new-top-level-deps-without-decision` for both crates.

AVIF/AV1 is **patent royalty-free** (AOMedia grant), so — unlike HEIC (DEC-052) — it belongs
on the default path, not behind a feature.

## Context

crustyimg could *write* AVIF (off-by-default `avif`/`ravif`, DEC-020) but could not *read* it
on the default build: `image` 0.25's AVIF decode depends on **dav1d, a C system library**,
violating `pure-rust-codecs-default` (DEC-004). SPEC-058 (STAGE-016, PROJ-009 headline) adds a
permissive, pure-Rust AVIF decoder so the default binary reads `.avif` end to end.

The design-time deep dive (2026-07-07) concluded that no *clean drop-in* pure-Rust decoder
exists (every mature drop-in is C-backed or AGPL), but that `re_rav1d` + `avif-parse` + glue is
viable. The build-cycle probe (throwaway project, this machine) **confirmed the stack
end-to-end**: encode via `ravif` → parse via `avif-parse` → decode via `re_rav1d` no-asm →
YUV→RGBA round-tripped 10-bit/8-bit, RGB/RGBA, and alpha within ≤1/255 error, with **no nasm
installed**. One design claim was corrected: `re_rav1d` 0.1.3 exposes the ergonomic safe API in
its `pub mod dav1d` (a dav1d-rs fork) — the raw C ABI is present but not what we call.

## Alternatives Considered

- **Option A: `image`'s built-in AVIF decode (`avif-native` → dav1d).**
  - What it is: enable `image/avif` decode.
  - Why rejected: pulls **dav1d (C system dep)** onto the default path — breaks DEC-004 and the
    zero-system-dep, trivial-multi-OS-CI promise. This is exactly what we route around.

- **Option B: `zenavif` / `rav1d-safe` (imazen).**
  - What it is: a pure-Rust AVIF decode stack.
  - Why rejected: **AGPL** — relicenses our statically-linked `MIT OR Apache-2.0` binary
    (`no-agpl-default-deps`, DEC-018).

- **Option C: in-house MIT ISOBMFF/AVIF container parser + `re_rav1d`.**
  - What it is: hand-roll the container parse (reusing the HEIC-spike box-parse tech) to avoid
    the MPL `avif-parse` dep; compounds for RAW-CR3 + a future HEIC path.
  - Why rejected (for now): `avif-parse` is battle-hardened against malformed input (this is an
    untrusted parser derived from Firefox's `mp4parse`); rewriting that hardening from scratch
    is a larger, riskier surface than accepting one file-level-copyleft dep behind a documented
    `deny.toml` exception. Kept as a future option (tracked in the watchlist) if we want to shed
    MPL and share container code with RAW/HEIC.

- **Option D (chosen): `re_rav1d` (no-asm, BSD-2) + `avif-parse` (MPL-2.0) + thin glue.**
  - Why selected: pure-Rust, zero build-tool deps, permissive licenses (`deny` green with one
    documented MPL exception), patent-clean, proven on real files by the probe, and the same
    decoder serves the Wave-3 WASM demo. The `re_rav1d` glue surface is kept thin so we can
    migrate to `image`'s built-in decode once image-rs #2621 lands.

## Consequences

- **Positive:** the default binary reads `.avif` (optimize/convert/info/batch) with **no
  system/build-tool deps**; `just deny` green; lean `--no-default-features` build unaffected
  (the deps are non-optional, not gated by `display`); the AV1 decoder is reusable for the
  WASM demo.
- **Negative / costs:**
  - Two new default deps and a heavier build (`re_rav1d` is a large crate). `re_rav1d` is
    rerun's self-described "messy" fork with an uncertain maintenance future → **pinned** and
    kept behind a thin glue module so it is swappable.
  - `avif-parse` adds an **MPL-2.0** (file-level weak copyleft) dep — accepted via a documented
    per-crate `deny.toml` exception (does not relicense our binary); `to_method` (a `re_rav1d`
    transitive) is **CC0-1.0**, also excepted per-crate.
  - `paste` (RUSTSEC-2024-0436, unmaintained) is now reached on the **default** path via
    `re_rav1d` (not only the `avif` encode feature); it is a compile-time proc-macro with no
    runtime surface — the existing advisory ignore covers it (reason updated).
  - **MSRV floor rises 1.89 → 1.90** (`avif-parse` 2.1.0).
  - Output is **8-bit** RGB(A): 10/12-bit HDR is down-converted without tone-mapping (transfer
    function / HDR handling is out of scope for SPEC-058). Grid/tiled and animated AVIF are
    rejected cleanly (single primary image only).
- **Neutral:** decode is dispatched by `ftyp` brand, independent of `image`'s avif feature; the
  AVIF **output** feature (DEC-020) is untouched.

## Validation

Right if: default `cargo build` (and `--no-default-features`) decode a real `.avif` with no
nasm/meson/system libs on all three CI OSes, `just deny` stays green, and the fuzz target finds
no panics. Revisit when: (a) **image-rs #2621** lands a pure-Rust AVIF decode in `image` — then
migrate and shed the direct `re_rav1d` dep + hand glue; or (b) the **OxideAV** MIT stack
(`oxideav-avif` + `oxideav-av1`) matures to a single permissive stack for AVIF **and** HEIC.
Also revisit if `re_rav1d` is abandoned (pin + thin glue make the swap cheap).

## References

- Related specs: SPEC-058 (this), SPEC-018/DEC-020 (AVIF output), SPEC-019/020 (WebP decode
  default + native-encode-gated precedent), SPEC-059 (future AVIF container spec, if the
  in-house parser path is taken).
- Related decisions: DEC-004 (pure-Rust default), DEC-034 (decode caps), DEC-018
  (`no-agpl-default-deps`), DEC-052 (HEIC feature-gated — the patent contrast).
- Constraints: `pure-rust-codecs-default`, `no-agpl-default-deps`,
  `no-new-top-level-deps-without-decision`, `untrusted-input-hardening`, `single-image-library`.
- Watchlist: `guidance/license-watchlist.yaml` → `avif-decode` (resolved by SPEC-058/DEC-053).
- Upstream: image-rs #2621 (pure-Rust AVIF decode PoC); rerun `re_rav1d`; kornelski `avif-parse`.
