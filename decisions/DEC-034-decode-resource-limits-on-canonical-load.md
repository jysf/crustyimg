---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-034
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-19
supersedes: null
superseded_by: null

affected_scope:
  - src/image/mod.rs
  - src/error.rs
  - src/cli/mod.rs

tags:
  - security
  - hardening
  - decode
  - untrusted-input
  - decompression-bomb
---

# DEC-034: decode resource limits on the canonical `Image` load path

## Decision

Apply **`image::Limits`** on the single canonical decode choke point
(`src/image/mod.rs::decode_with_format`, reached by every load entry —
`Image::load` / `from_bytes` / `from_reader`) to bound a decoder's dimensions and
allocation, so an untrusted file cannot trigger a decompression bomb (OOM / huge
allocation) before the pixels are ever produced. Concretely:

- **Strict per-dimension cap:** `max_image_width = max_image_height = 65_535`
  (`MAX_IMAGE_DIMENSION`). Any single declared dimension above this is rejected at
  header time, before allocation. 65 535 is the JPEG hard limit and a sane ceiling
  for every supported format; legitimate images effectively never exceed it.
- **Allocation cap:** `max_alloc = 512 MiB` (`MAX_ALLOC_BYTES`) — the real
  decompression-bomb defense (it bounds the decoded buffer `W×H×channels`). This is
  numerically equal to the `image` crate's *current* default, but we set it
  **explicitly**: the crate documents that its default "may be changed in future
  major version increases", and a security limit must not silently drift with a dep
  bump.
- **Reject, do not clamp.** Exceeding a limit is a hard rejection of the input, not a
  silent downscale. The decoder's `image::ImageError::Limits(_)` is mapped to a new
  typed **`crate::ImageError::LimitsExceeded(String)`** (DEC-007), distinct from
  `Decode` so callers/tests can tell "rejected for safety" from "corrupt data". At
  the CLI boundary it maps to **exit 1** (generic runtime rejection — no new
  api-contract exit code).

`image::Limits` is part of the already-vendored `image` crate (`=0.25.10`); **no new
dependency**, so `no-new-top-level-deps-without-decision` is not triggered. This DEC
exists to record the *security policy* (the caps, reject-not-clamp, the typed error,
the exit code) that later specs build on, per constraint `untrusted-input-hardening`.

## Context

STAGE-006 is the MVP exit gate: harden every untrusted-input surface and verify it.
Decode limits are its first item and close a known gap — until now the load path set
no dimension limits, so a crafted file declaring enormous dimensions could drive a
large allocation (the 512 MiB default alloc cap was inherited implicitly but
undocumented, and no dimension ceiling existed). The `image` API supports this
directly: `ImageReader::new` initialises `Limits::default()`, and
`ImageReader::limits(&mut self, Limits)` overrides it before `decode()`. `Limits` is
`#[non_exhaustive]` with public fields, so it is built via `Limits::default()` +
field assignment (no struct literal). A probe (`RgbImage::new(70_000, 1)` → PNG → load)
confirmed the width cap rejects an oversized-but-cheap fixture with
`ImageError::Limits(DimensionError)` before allocation — a deterministic adversarial
fixture that needs no CRC forgery and no large buffer.

## Alternatives Considered

- **Clamp/auto-downscale instead of reject** — rejected: silently resizing an
  untrusted input is surprising and hides the attack; the safe default is to refuse.
- **Inherit the crate's implicit `Limits::default()` (do nothing)** — rejected: no
  dimension ceiling, and the alloc default can change between `image` versions; a
  security limit must be explicit and version-stable.
- **Per-run configurability now (`--max-pixels` / env override)** — deferred (out of
  scope): v1 uses fixed, generous caps that pass every realistic image; a documented
  override is an additive follow-up if a user has a legitimately enormous image.
- **A dedicated exit code for resource-limit rejection** — rejected for v1: reuses
  exit 1 (generic runtime) to avoid api-contract churn; the typed `LimitsExceeded`
  message conveys the cause. Revisit if scripts need to distinguish it.

## Consequences

- **Positive:** A decompression-bomb / forged-dimension file is rejected with a typed
  error, never a panic or OOM, on every command that loads an image (`view`/`info`/
  `resize`/`convert`/`edit`/`apply`/…), because the limit sits at the one shared
  decode point. The policy is explicit and version-stable.
- **Negative:** A *legitimately* gigantic image (a single dimension > 65 535, or a
  decoded buffer > 512 MiB) is now refused. This is the intended trade-off for
  untrusted-input safety; a future `--max-pixels`/env override can re-admit such
  inputs deliberately.
- **Neutral:** No new dependency; no behavior change for the small images the test
  suite and normal use exercise (the 512 MiB alloc cap already applied by default).

## Validation

Right if: a `70_000×1` (or > 512 MiB decoded) fixture is rejected with
`ImageError::LimitsExceeded` (exit 1) — not a panic/OOM — while every existing image
decodes unchanged, on all load entries. Revisit if: users need configurable limits
(add `--max-pixels`/env), a new format needs a different ceiling, or a dedicated exit
code is wanted.

## References

- Related specs: SPEC-033 (decode resource limits on image load); SPEC-002 (the
  canonical `Image` load path this hardens)
- Related decisions: DEC-002 (single image model / load-once), DEC-007 (typed errors,
  no panics on recoverable paths)
- Constraints: `untrusted-input-hardening` (blocking), `no-unwrap-on-recoverable-paths`
- External docs: https://docs.rs/image/0.25.10/image/struct.Limits.html
