---
insight:
  id: DEC-004
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

created_at: 2026-06-13
supersedes: null
superseded_by: null

affected_scope:
  - Cargo.toml
  - src/image/**
  - src/sink/**

tags:
  - codecs
  - dependencies
  - ci
  - portability
---

# DEC-004: Pure-Rust codecs by default; native codecs behind cargo features

## Decision

Default builds use **only pure-Rust** codec crates (`image` and its
pure-Rust decoders/encoders, `fast_image_resize`). Native-dependency codecs
(`mozjpeg`/libjpeg-turbo, `rexiv2`/gexiv2, AVIF/libavif) are **opt-in cargo
features**, off by default. Core formats in the MVP are JPEG, PNG, GIF, BMP,
TIFF, ICO. **WebP** output is a fast-follow (pure-Rust via `image`). **AVIF**
is feature-gated and lands later; invoking it without the feature exits 4
("codec not built").

## Context

The prototype shelled out to ImageMagick and pulled native deps that make
cross-platform builds painful. We want `cargo build` and CI to be trivially
green on Linux, macOS, and Windows (DEC-009) with no system libraries.
Native codecs do offer real wins (mozjpeg's size/quality), but they cost
build complexity and platform-specific breakage. Gating them behind
features lets power users opt in without taxing the default path
(feature-exploration.md § "Technical considerations" / "Decisions to
formalize" #3).

## Alternatives Considered

- **Option A: Native codecs by default (mozjpeg, libavif)**
  - Why rejected: breaks the trivial multi-OS CI promise; Windows builds in
    particular become fragile. Contradicts a core success signal.

- **Option B: Pure-Rust only, no native ever**
  - Why rejected: forecloses mozjpeg's real web-size win and AVIF entirely;
    PROJ-001 explicitly flags mozjpeg quality as a thesis risk worth a
    feature escape hatch.

- **Option C (chosen): pure-Rust default + feature-gated native**
  - Why selected: default builds stay trivial and portable; advanced
    encoders remain reachable behind `--features mozjpeg` / `avif` / `rexiv2`.

## Consequences

- **Positive:** Zero system deps by default; trivial CI; clean release
  artifacts. Native wins available on demand.
- **Negative:** Pure-Rust JPEG encode may not match mozjpeg on size/quality
  (PROJ-001 thesis risk). Feature combinations multiply the test matrix
  (mitigated: CI tests default; a single extra job builds with native
  features).
- **Neutral:** AVIF deferral is intentional; `convert --format avif` returns
  exit 4 when not built.

## Validation

Right if: default `cargo build`/`cargo test` are green on all three OSes
with no system libraries, and `--features mozjpeg` builds and improves JPEG
output where available. Revisit if: pure-Rust encoders prove inadequate for
the "better for web" claim — then consider promoting mozjpeg to a default
on platforms where it builds cleanly.

## References

- Related specs: SPEC-001 (CI), STAGE-003 (convert/shrink)
- Related decisions: DEC-003 (rexiv2 as optional metadata feature), DEC-009 (CI matrix)
- External docs: https://docs.rs/image, https://crates.io/crates/mozjpeg, https://crates.io/crates/ravif
