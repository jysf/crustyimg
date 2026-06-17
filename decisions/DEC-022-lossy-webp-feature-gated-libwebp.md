---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-022
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
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-17
supersedes: null
superseded_by: null

affected_scope:
  - Cargo.toml
  - .github/workflows/ci.yml
  - src/sink/**
  - src/quality/**

tags:
  - webp
  - codec
  - libwebp
  - c-dependency
  - feature-gate
  - modern-formats
---

# DEC-022: Lossy WebP encode behind an off-by-default `webp-lossy` (libwebp) feature

## Decision

crustyimg gains **lossy WebP encode** behind an **off-by-default `webp-lossy` cargo
feature** that pulls the **`webp` crate** (→ `libwebp-sys` → **vendored libwebp**,
compiled at build time with `cc`). This layers onto the pure-Rust WebP foundation
from SPEC-019/DEC-021 (lossless encode + decode, default):

1. **It is the project's FIRST C dependency — and it is OPT-IN (DEC-004).** libwebp
   is C; `libwebp-sys` **vendors** it and builds it with `cc` (a C compiler — present
   on CI runners and dev machines — but **no system library install** is needed). The
   default build stays **pure-Rust with zero C**; only `--features webp-lossy` compiles
   libwebp. This is the same "heavy/native codec behind an off-by-default feature"
   policy as AVIF (DEC-020), except AVIF happened to be pure-Rust and this is not.
2. **License gate stays green with NO new exception.** `webp` is `MIT OR Apache-2.0`,
   `libwebp-sys` is `MIT`, and vendored libwebp is BSD-3 (+ an additional Google
   patent grant) — all permissive. `cargo deny check licenses` (all-features) is
   **green with no new exception** (verified empirically in design).
3. **WebP gains a quality knob ⇒ BOTH auto-quality searches drive it.** With the
   feature, `-q` maps to WebP quality (1–100), and — because the pure-Rust DECODER
   already exists (SPEC-019) — **both** the byte-budget (`--max-size`) AND the
   perceptual (`--target`/`--ssim`) searches drive WebP (the perceptual search can
   decode WebP candidates to score them). This is the **contrast with AVIF**, which
   has no decoder and so supports only the byte-budget search (DEC-020). So WebP, with
   this feature, sets BOTH `LossyFormat` predicates true.
4. **Lossy is selected by the presence of a quality; otherwise lossless.** WebP
   encodes **lossy** when an effective quality is determined (an explicit `-q`, or a
   quality chosen by an auto-quality search); with no quality it stays **lossless**
   (the SPEC-019 default). `shrink` (which defaults `-q` to 80) therefore writes lossy
   WebP with the feature on; a bare `convert --format webp` stays lossless. Without
   the feature, `-q`/auto on WebP are ignored (lossless), exactly as SPEC-019 shipped.
5. **The `webp` crate is used as an encode-only codec binding, not a second image
   library.** We feed it raw bytes from the existing `image` `DynamicImage`
   (`to_rgba8()`/`to_rgb8()` → `Encoder::from_rgba`/`from_rgb`), and do NOT enable the
   `webp` crate's `image` feature (which would pull a second, possibly version-skewed
   `image` crate). So `single-image-library` holds: one pixel library (`image`); `webp`
   is a single-format encoder sink, analogous to `ssimulacra2` being a metric-only crate.

## Context

STAGE-008 is "Modern Formats & Quality." SPEC-019 shipped WebP as a pure-Rust default
(lossless encode + decode) but explicitly deferred lossy encode because pure-Rust
`image-webp` does not do it (the `image` docs point to libwebp). Lossy WebP is the
headline "smaller than JPEG" WebP story and the thing that lets the SPEC-016/017
auto-quality searches drive WebP. This spec adds it the only way available — libwebp —
behind an opt-in feature so the default build keeps its zero-C, zero-system-deps
promise.

Design-time empirical verification (the "pin the dep in design" discipline):
- `webp` 0.3.1 → `libwebp-sys` 0.9.6 **builds via `cc`** (vendored libwebp; clean
  build, no system lib install).
- Licenses: `webp` MIT/Apache, `libwebp-sys` MIT, vendored libwebp BSD-3 — `cargo deny
  check licenses` **green, no new exception**.
- API pinned: `webp::Encoder::from_rgba(&[u8], w, h).encode(quality: f32) ->
  WebPMemory` (and `.encode_lossless()`); `WebPMemory: Deref<[u8]>`.

## Alternatives Considered

- **Wait for a pure-Rust lossy WebP encoder.**
  - Why rejected: none mature/permissive exists; `image-webp` explicitly defers to
    libwebp. Waiting indefinitely blocks the headline WebP value. (Revisit if one
    appears — it would let us drop the C dep.)
- **Make lossy WebP a DEFAULT (no feature gate).**
  - Why rejected: it would put C code (libwebp + `cc` at build time) in every build,
    breaking the "default build is pure-Rust, zero C, zero system deps" promise that
    even AVIF preserved. Opt-in keeps the default lean and honest.
- **Enable the `webp` crate's `image` feature for `Encoder::from_image`.**
  - Why rejected: it pulls the `webp` crate's own `image` dependency, risking a SECOND
    `image` crate version in the tree (binary bloat + `single-image-library` tension).
    Feeding raw `to_rgba8()` bytes to `from_rgba` avoids it entirely.
- **Use raw `libwebp-sys` directly (skip the `webp` wrapper).**
  - Why rejected: `webp` is a thin, permissive, safe wrapper with the exact
    `from_rgba(...).encode(q)` shape we need; raw FFI would add unsafe surface for no
    gain. (`webp` pulls `libwebp-sys` anyway.)

## Consequences

- **Positive:** the headline WebP story — lossy WebP, typically smaller than JPEG at
  equal quality — and the auto-quality searches (BOTH, perceptual + byte-budget) drive
  WebP, because SPEC-019 already gave us the decoder. License gate green, no new
  exception.
- **Negative:** introduces the project's **first C dependency** (vendored libwebp via
  `cc`); a `--features webp-lossy` build needs a C compiler and is slower to compile.
  AVIF quality numbers / WebP quality numbers are not comparable; use the
  perceptual/byte targets. A second cross-sync contract (the sink encode and the
  search probe must encode WebP identically) now also covers WebP.
- **Neutral:** a new CI `webp-lossy` job (single ubuntu runner; `cc` is preinstalled).
  WebP's lossy-vs-lossless choice is driven by "is a quality set," which is a small but
  real behavior rule to document.

## Validation

Right if: the default build is unchanged (lossless WebP only; `-q` ignored); a
`--features webp-lossy` build encodes lossy WebP for `-q`/`--target`/`--ssim`/
`--max-size` (output `image::guess_format == WebP`, smaller at low quality than
lossless for a photo), and a bare `convert --format webp` stays lossless; `cargo deny
check licenses` is green with no new exception; the feature build needs only `cc` (no
system libwebp install). Revisit if: a pure-Rust lossy WebP encoder appears (drop the
C dep); or libwebp's vendoring/license posture changes; or the `cc` build proves
problematic on a target (then consider a system-libwebp link option).

## References
- Related specs: SPEC-020 (this — lossy WebP), SPEC-019 (the pure-Rust WebP
  foundation this layers onto), SPEC-018 (AVIF — the byte-budget-only contrast),
  SPEC-017 (the format-agnostic search lossy WebP plugs into), SPEC-016 (the
  perceptual search, now usable for WebP because the decoder exists).
- Related decisions: DEC-021 (WebP lossless+decode default), DEC-004 (codec policy /
  feature-gating), DEC-018 (license gate — green, no new exception), DEC-019 (the
  auto-quality search reused), DEC-016 (`-q` → encoder quality), DEC-020 (AVIF — the
  perceptual-needs-a-decoder finding, which WebP satisfies).
- Related constraints: `pure-rust-codecs-default` (default stays pure-Rust; the C dep
  is opt-in), `no-agpl-default-deps` (all permissive), `no-new-top-level-deps-without-decision`
  (this DEC authorizes the `webp` top-level dep), `single-image-library` (webp is an
  encode-only codec sink, not a second pixel library).
- External docs: https://docs.rs/webp , https://chromium.googlesource.com/webm/libwebp/+/refs/heads/main/COPYING
