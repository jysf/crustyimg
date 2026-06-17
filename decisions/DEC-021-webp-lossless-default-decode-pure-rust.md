---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-021
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
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-17
supersedes: null
superseded_by: null

affected_scope:
  - Cargo.toml
  - src/sink/**
  - docs/api-contract.md

tags:
  - webp
  - codec
  - image-webp
  - pure-rust
  - default-format
  - modern-formats
---

# DEC-021: WebP lossless output + WebP decode (input) as a pure-Rust DEFAULT format

## Decision

crustyimg gains **WebP** support as a **default, pure-Rust** format via the `image`
crate's own `webp` feature (→ `image-webp`, MIT/Apache):

1. **WebP DECODE (input) — default, both lossy and lossless.** `image-webp` decodes
   both WebP variants in pure Rust, so `.webp` becomes a readable INPUT everywhere
   (`view`, `info`, `resize`, `thumbnail`, `convert in.webp --format png`, etc.).
2. **WebP LOSSLESS ENCODE (output) — default, pure-Rust.** `convert --format webp`
   / `-o x.webp` produce a **lossless** WebP (VP8L) via `image`'s
   `WebPEncoder::new_lossless` (reached through the existing `DynamicImage::write_to`
   path). Lossless WebP is typically smaller than PNG at the same fidelity.
3. **It is a DEFAULT format, not feature-gated (DEC-004).** `image-webp` is
   pure-Rust, small, and permissive (`just deny` green with **no new exception**),
   so per DEC-004 ("pure-Rust codecs ship by default; native/heavy codecs behind
   off-by-default features") WebP is added to the default `image` feature list. This
   **changes the default build**: `convert --format webp` flips from exit 4
   ("codec not built") to a successful lossless encode, and `.webp` input now decodes.
4. **No quality knob on lossless WebP — `-q`/auto-quality do not apply.** Lossless
   WebP has no 0–100 quality parameter, so `-q` is **ignored** for WebP output (like
   PNG, DEC-016), and the auto-quality searches (`--target`/`--ssim`, `--max-size`)
   do **not** drive it — `supports_lossy_quality(WebP)` / `supports_perceptual_quality(WebP)`
   are **false** in the default build. They become available only with the
   deferred lossy encoder (point 5).
5. **LOSSY WebP is DEFERRED to SPEC-020 (a separate decision/feature).** Pure-Rust
   `image-webp` does **not** do lossy encode; lossy WebP needs **`libwebp` (a C
   system dependency)**. That is a materially different risk profile (a first C dep,
   a new top-level crate, a CI C-toolchain, and `single-image-library` tension), so
   it lands behind an **off-by-default `webp-lossy` feature** in its own spec
   (SPEC-020, DEC-022). This spec (SPEC-019) is the pure-Rust foundation; SPEC-020
   layers lossy on top and is what lets the auto-quality searches drive WebP.

## Context

STAGE-008 is "Modern Formats & Quality." SPEC-016/017 built the perceptual-target
and byte-budget quality core; SPEC-018 added AVIF output (the first modern format,
feature-gated, output-only — it has no pure-Rust decoder). WebP is the natural next
format and is the **inverse of AVIF's capability profile**, verified empirically at
design time:

- `image` 0.25.10 `webp` feature → `image-webp` 0.2.4 (`MIT OR Apache-2.0`):
  **builds pure-Rust, no nasm, no system libs** (confirmed by a clean build with the
  feature enabled).
- `image-webp` **decodes lossy + lossless** WebP, but **encodes lossless only**
  (`WebPEncoder::new_lossless`; the `image` docs state: *"If you need lossy encoding,
  you'll have to use libwebp."*).
- `cargo deny check licenses` stays **green with no new exception** when `webp` is
  enabled (image-webp + its transitives are all permissive).

So the pure-Rust, zero-system-deps, permissive promise is fully preserved by adding
WebP decode + lossless encode to the default build. The only thing that is NOT
pure-Rust — lossy encode — is the one thing held back (SPEC-020).

The AVIF contrast is instructive (see DEC-020 and the perceptual-search finding):
AVIF has a pure-Rust ENCODER but no decoder, so it is output-only and only the
byte-budget search can drive it. WebP has a pure-Rust DECODER but only a lossless
encoder, so it is input + lossless-output and **no** search drives it until the
lossy (libwebp) encoder lands — at which point BOTH searches drive it (the decoder
already exists, so the perceptual search can score WebP round-trips).

## Alternatives Considered

- **Feature-gate WebP (like AVIF) instead of defaulting it.**
  - Why rejected: WebP decode + lossless encode are pure-Rust, small, and permissive
    — exactly the DEC-004 "default" category. Gating would hide `.webp` *input*
    (which users expect to just work) behind a flag for no real cost saving
    (`image-webp` is tiny next to `rav1e`). Reserve gating for the C-dep lossy
    encoder (SPEC-020).

- **Do lossless + lossy WebP in one spec.**
  - Why rejected: lossy needs `libwebp` (a C dep) — a new top-level dependency, a CI
    C-toolchain, and `single-image-library` tension. Bundling it with the trivial
    pure-Rust part makes an L spec and couples a risky decision to a safe one.
    Splitting keeps SPEC-019 shippable in pure Rust and isolates the C-dep review in
    SPEC-020.

- **Skip lossless WebP; wait for lossy (libwebp) and ship only that.**
  - Why rejected: lossless WebP (smaller-than-PNG) and WebP **input** are real,
    free, pure-Rust wins available today. Shipping them now unblocks `.webp` reading
    immediately and gives the responsive-set / `optimize` waves a WebP target.

- **Use a pure-Rust lossy WebP encoder.**
  - Why rejected: none exists that is mature/permissive; `image-webp` explicitly
    points to `libwebp` for lossy. (Revisit if one appears.)

## Consequences

- **Positive:** `.webp` files are readable everywhere by default; lossless WebP is a
  new default output (smaller than PNG); the pure-Rust / zero-system-deps / permissive
  promise is intact (`just deny` green, no new exception); the WebP wiring (format
  recognition, decode, the encode arm) is in place for SPEC-020 to add lossy on top.
- **Negative / default-build change:** the default binary grows by `image-webp`;
  `convert --format webp` now succeeds (lossless) instead of exiting 4, and a couple
  of existing tests that asserted "webp → exit 4" must flip. `-q`/`--max-size`/
  `--target` on a WebP output are ignored with a warning (no quality knob) until
  SPEC-020 — potentially surprising to users who expect WebP to be lossy; the
  warning and docs must say "lossless WebP; build --features webp-lossy for lossy."
- **Neutral:** `extension_for_format` already returned `"webp"`; this fixes the
  long-standing one-way asymmetry (`format_from_extension` now recognizes it too).

## Validation

Right if: the default build reads `.webp` input (lossy + lossless) and writes
lossless WebP via `convert --format webp` / `-o x.webp` (`image::guess_format ==
WebP`, and the output decodes back); `-q` on a WebP output is ignored like PNG;
`cargo deny check licenses` is green with **no** new exception; the rest of the
default suite + all gates stay green. Revisit if: a mature pure-Rust lossy WebP
encoder appears (fold it in, drop the libwebp plan); or WebP's default inclusion is
judged too heavy for the base binary (then gate it behind a pure-Rust `webp`
feature — but keep decode available).

## References
- Related specs: SPEC-019 (this — WebP lossless + decode), SPEC-020 (lossy WebP via
  the `webp-lossy` libwebp feature), SPEC-018 (AVIF — the opposite capability
  profile), SPEC-017 (the format-agnostic search lossy WebP will plug into),
  SPEC-014 (`convert` + exit-4 for an unbuilt codec — WebP no longer hits it).
- Related decisions: DEC-004 (codec policy / pure-Rust default), DEC-018 (the
  license gate — green with no new exception), DEC-016 (`-q` ignored on lossless
  formats), DEC-015 (format precedence), DEC-020 (AVIF — the contrast).
- Related constraints: `pure-rust-codecs-default`, `no-agpl-default-deps`,
  `no-new-top-level-deps-without-decision` (satisfied — WebP arrives via the existing
  `image` dep's feature, no new top-level crate), `single-image-library` (satisfied
  — WebP encode/decode is the `image` crate's own backend).
- External docs: https://docs.rs/image-webp , https://docs.rs/image/0.25.10/image/codecs/webp/struct.WebPEncoder.html
