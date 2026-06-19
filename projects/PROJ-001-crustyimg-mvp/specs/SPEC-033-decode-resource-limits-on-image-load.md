---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-033
  type: story                      # epic | story | task | bug | chore
  cycle: build                     # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-006
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet (prescriptive prompt)
  created_at: 2026-06-19

references:
  decisions: [DEC-034, DEC-002, DEC-007]
  constraints:
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - clippy-fmt-clean
    - every-public-fn-tested
  related_specs: [SPEC-002]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-006's <capability>". Optional; null is acceptable.
value_link: >
  First STAGE-006 hardening item: bounds decoder dimensions/allocation on the
  one canonical load path so an untrusted file can't trigger a decompression
  bomb — closing a known gap before the MVP ships.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-033: decode resource limits on image load

## Context

**The first STAGE-006 hardening item, and a known gap closed.** STAGE-006 is the
MVP exit gate: harden every untrusted-input surface and verify it. The most
fundamental surface is image decode itself — the binary is run on arbitrary
files, and until now the canonical load path set **no decoder dimension limits**.
A crafted file declaring enormous dimensions (a "decompression bomb") could drive
a large allocation before any pixels exist. The `image` crate supports a direct
defense — `image::Limits` on the decode reader — which `ImageReader::new` already
initialises to a 512 MiB allocation default but with **no dimension ceiling**, and
that default is implicit (the crate documents it "may be changed in future major
version increases").

This spec sets explicit `image::Limits` (dimension caps + an affirmed allocation
cap) on the **single canonical decode choke point** (`decode_with_format` in
`src/image/mod.rs`, reached by `Image::load` / `from_bytes` / `from_reader`), and
maps the decoder's limit error to a new typed `ImageError::LimitsExceeded` so an
over-limit input is **rejected with a typed error, never a panic or OOM**, on
every command that loads an image. Parent: `STAGE-006` (backlog item #1).
Governing: **DEC-034** (the limits policy — caps, reject-not-clamp, typed error,
exit 1), **DEC-002** (the canonical load path), **DEC-007** (typed errors). No new
dependency (`image::Limits` is in the already-vendored `image` crate).

## Goal

Bound decoder dimensions and allocation on the one canonical decode path via
`image::Limits` so an over-limit (decompression-bomb / forged-dimension) input is
rejected with a typed `ImageError::LimitsExceeded` (CLI exit 1) — never a panic or
OOM — while every realistic image still decodes unchanged.

## Inputs

- **Files to read:**
  - `src/image/mod.rs` — `decode_with_format` (the single decode choke point to
    harden), `Image::{load, from_bytes, from_reader}` (all route through it), the
    `#[cfg(test)] mod tests` fixture helpers (`solid_png`).
  - `src/error.rs` — `ImageError` (add the `LimitsExceeded` variant).
  - `src/cli/mod.rs` — `CliError::code()` (the `CliError::Image(ImageError::…)`
    exit-code arms — add the `LimitsExceeded → 1` mapping).
  - `decisions/DEC-034` (the limits policy this implements), `DEC-002`, `DEC-007`.
- **External APIs:** `image::Limits` (`=0.25.10`, already a dependency) —
  `Limits::default()` + public fields `max_image_width`/`max_image_height:
  Option<u32>`, `max_alloc: Option<u64>` (struct is `#[non_exhaustive]` → build via
  `default()` + field assignment, NOT a struct literal); `ImageReader::limits(&mut
  self, Limits)`; the decoder returns `image::ImageError::Limits(_)` on exceed.
  Docs: https://docs.rs/image/0.25.10/image/struct.Limits.html
- **Related code paths:** `src/image/`, `src/error.rs`, `src/cli/mod.rs`, `tests/`.

## Outputs

- **Files modified:**
  - `src/error.rs` — add `ImageError::LimitsExceeded(String)` (typed, distinct from
    `Decode`).
  - `src/image/mod.rs` — add `decode_limits() -> image::Limits` (the production
    caps, DEC-034) and `map_image_decode_error(::image::ImageError) -> ImageError`
    (limit→`LimitsExceeded`, else `Decode`); apply the limits in `decode_with_format`
    before `decode()`. Optionally a `decode_with_limits(bytes, &Limits)` seam so a
    custom `Limits` is unit-testable. All unit-tested in the module's
    `#[cfg(test)]`.
  - `src/cli/mod.rs` — `CliError::code()`: add
    `CliError::Image(ImageError::LimitsExceeded(_)) => 1`.
  - `docs/api-contract.md` — note that image load is bounded by decode limits and an
    over-limit input exits 1 (DEC-034). (Done at design.)
  - `SECURITY.md` — record decode limits as a mitigation in the threat model (the
    decompression-bomb row). (Done at design.)
- **New exports:** `ImageError::LimitsExceeded` (public enum variant). The
  `decode_limits` / `map_image_decode_error` helpers stay private (module-internal),
  unit-tested in-module.
- **Database changes:** none.

## Limits policy (PINNED — DEC-034)

- **Constants** (module consts in `src/image/mod.rs`):
  - `MAX_IMAGE_DIMENSION: u32 = 65_535` → `max_image_width` and `max_image_height`.
  - `MAX_ALLOC_BYTES: u64 = 512 * 1024 * 1024` → `max_alloc`.
- **Construction:** `let mut limits = ::image::Limits::default(); limits.max_image_width
  = Some(MAX_IMAGE_DIMENSION); limits.max_image_height = Some(MAX_IMAGE_DIMENSION);
  limits.max_alloc = Some(MAX_ALLOC_BYTES);` — field assignment (the struct is
  `#[non_exhaustive]`; a struct literal will NOT compile).
- **Application:** in `decode_with_format`, make the reader `mut`, call
  `reader.limits(decode_limits())` AFTER `with_guessed_format()` and the
  `reader.format()` check, BEFORE `reader.decode()`.
- **Error mapping:** `reader.decode().map_err(map_image_decode_error)?` where
  `map_image_decode_error` returns `ImageError::LimitsExceeded(e.to_string())` for
  `::image::ImageError::Limits(_)` and `ImageError::Decode(e.to_string())` for every
  other decode error. (Match on the `::image::ImageError` variant.)
- **Reject, do not clamp.** No downscale-to-fit; an over-limit input is refused.
- **Exit code:** `LimitsExceeded → 1` (generic runtime rejection; no new exit code).
- **Scope of effect:** because this is the one shared decode point, the limit
  applies uniformly to `view` / `info` / `resize` / `convert` / `edit` / `apply` /
  every load — no per-command wiring.

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] A `70_000 × 1` PNG (cheap to create; width > `MAX_IMAGE_DIMENSION`) fed to a
  load entry is rejected with `ImageError::LimitsExceeded` — NOT a panic, OOM, or
  `Decode` — and at the CLI it exits **1** (e.g. `crustyimg info bomb.png`).
- [ ] Decoding any normal small image with the production `decode_limits()` succeeds
  unchanged (no regression for realistic images).
- [ ] A deliberately tiny `Limits` (`max_image_width = Some(1)`) makes a normal PNG
  decode return `LimitsExceeded` via the test seam (proves enforcement, not just the
  constant value).
- [ ] A deliberately tiny `max_alloc` (e.g. `Some(16)`) makes a normal PNG **whose
  decoded buffer exceeds the cap** (e.g. 64×64) decode return `LimitsExceeded`
  (proves the allocation/`reserve` path, not only dimensions).
- [ ] `map_image_decode_error` maps `::image::ImageError::Limits(_)` →
  `LimitsExceeded`; an ordinary decode failure (truncated/corrupt-but-recognized
  bytes) stays `Decode` (limits don't swallow normal decode errors).
- [ ] `CliError::Image(ImageError::LimitsExceeded(_)).code() == 1` (unit-tested
  alongside the other exit-code-mapping tests).
- [ ] The limit applies to `Image::load`, `from_bytes`, AND `from_reader` (they all
  route through the hardened `decode_with_format`).
- [ ] `cargo deny` green; the **lean build** (`--no-default-features`) compiles; no
  new dependency added; no `unwrap`/`expect`/`panic!` on the new non-test paths.

## Failing Tests

Written during **design**, BEFORE build. Generate fixtures natively (small PNGs via
the `image` crate; the bomb fixture is a real but cheap oversized-dimension PNG — NO
CRC forgery, NO large allocation).

- **`src/image/mod.rs` (unit, `#[cfg(test)] mod tests`)**
  - `"oversized_dimension_png_is_limits_exceeded"` — `RgbImage::new(70_000, 1)`
    encoded to PNG, then `Image::from_bytes(&png)` → `Err(ImageError::LimitsExceeded(_))`
    (the production limits reject it at header time; the buffer is only ~210 KB so the
    test is cheap and never OOMs).
  - `"normal_image_decodes_under_production_limits"` — a small `solid_png` decodes to
    `Ok` under `decode_limits()` (no regression).
  - `"tiny_dimension_limit_rejects_via_seam"` — `decode_with_limits(small_png,
    &limits_with_max_width_1)` → `Err(LimitsExceeded)` (enforcement, not the constant).
  - `"tiny_alloc_limit_rejects_via_seam"` — `decode_with_limits(png_64x64,
    &limits_with_max_alloc_16)` → `Err(LimitsExceeded)` (allocation/`reserve` path;
    the 64×64 decoded buffer is well over 16 bytes).
  - `"map_limit_error_to_limits_exceeded"` — `map_image_decode_error(
    ::image::ImageError::Limits(::image::error::LimitError::from_kind(
    ::image::error::LimitErrorKind::DimensionError)))` returns `LimitsExceeded`.
  - `"truncated_png_is_decode_not_limits"` — a valid PNG signature/IHDR with a
    truncated IDAT (recognized as PNG, undecodable) → `Image::from_bytes` returns
    `Err(ImageError::Decode(_))` (the else branch; limits don't mask ordinary
    decode failures).
  - `"from_reader_is_also_limited"` — `Image::from_reader(Cursor::new(&oversized_png))`
    → `Err(LimitsExceeded)` (proves all entries route through the choke point).
- **`src/cli/mod.rs` (unit, exit-code mapping tests)**
  - `"limits_exceeded_maps_to_exit_1"` —
    `CliError::Image(ImageError::LimitsExceeded("x".into())).code() == 1`.
- **`src/error.rs` (unit)**
  - `"limits_exceeded_carries_message"` — `ImageError::LimitsExceeded("too big".into())
    .to_string()` contains the message (mirrors the existing `decode_error_carries_message`).
- **`tests/cli.rs` or `tests/image_load.rs` (integration, drives the binary / public API)**
  - `"info_on_oversized_image_exits_1_not_panic"` — write a `70_000×1` PNG to a
    tempdir, run `crustyimg info bomb.png`, assert exit code **1** and a non-empty
    stderr naming the limit/decode rejection (no panic, no hang).

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-034` — the limits policy: `MAX_IMAGE_DIMENSION = 65_535`, `MAX_ALLOC_BYTES =
  512 MiB`, reject-not-clamp, typed `LimitsExceeded`, exit 1, set explicitly (don't
  inherit the crate default). **Implement exactly these caps.**
- `DEC-002` — the canonical single-image load path; harden the one shared
  `decode_with_format`, not each call site.
- `DEC-007` — typed, matchable errors; no `unwrap`/`expect`/`panic!` on the new
  non-test code.

### Constraints that apply

- `untrusted-input-hardening` (blocking) — decode limits surfaced as a typed error,
  never a panic/OOM. This spec is the decode-limits half of that constraint.
- `no-unwrap-on-recoverable-paths`, `clippy-fmt-clean`, `every-public-fn-tested`.

### Prior related work

- `SPEC-002` (shipped) — built `Image::load`/`from_bytes`/`from_reader` +
  `decode_with_format` + `ImageError`. This spec hardens that path; do NOT change the
  metadata-capture behavior or the decode-once model.

### Out of scope (for this spec specifically)

- **Per-run configurability** (`--max-pixels` / env override) — fixed caps for v1
  (DEC-034); a documented override is an additive follow-up.
- **Path/symlink traversal hardening** (Source/Sink) — STAGE-006 backlog item #2, a
  separate spec.
- **Security-grade recipe validation**, **`cargo audit`/`deny` in CI**, the
  **threat-model verification pass** — later STAGE-006 backlog items.
- A dedicated exit code for resource-limit rejection — reuses exit 1 (DEC-034).

## Notes for the Implementer

- **One choke point.** Every load entry (`load`/`from_bytes`/`from_reader`) already
  funnels through `decode_with_format`. Harden there only; the entries need no change
  beyond inheriting it. A test pins that `from_reader` is also limited.
- **`Limits` is `#[non_exhaustive]`** — you MUST build it as `Limits::default()` then
  assign the public fields; a struct literal won't compile. `ImageReader::limits`
  takes `&mut self` and returns `()`, so make `reader` `mut` and call it before
  `decode()` (after the `reader.format()` check). `decode()` consumes the reader.
- **Test seam.** Factor the decode so a custom `Limits` is testable, e.g.
  `decode_with_limits(bytes: &[u8], limits: &image::Limits) -> Result<(DynamicImage,
  ImageFormat)>` and have `decode_with_format(bytes)` call it with `decode_limits()`.
  `ImageReader::limits` takes `Limits` by value, so clone the borrowed limits
  (`limits.clone()`) inside the seam (`Limits: Clone`).
- **The bomb fixture is cheap and real.** `RgbImage::new(70_000, 1)` is ~210 KB and
  PNG-encodes fine; the decoder reads the IHDR (70 000 × 1), checks it against
  `max_image_width = 65_535`, and returns `ImageError::Limits(DimensionError)` BEFORE
  decoding image data. No CRC forgery, no multi-GB allocation — keep it this way.
- **Mapping helper.** `map_image_decode_error` matches on `::image::ImageError`:
  `::image::ImageError::Limits(_) => ImageError::LimitsExceeded(e.to_string())`, `_ =>
  ImageError::Decode(e.to_string())`. To unit-test it, construct a limit error via
  `::image::error::{LimitError, LimitErrorKind}` (`LimitError::from_kind(
  LimitErrorKind::DimensionError)`), wrapped in `::image::ImageError::Limits(_)`.
- Run clippy right after the doc comments (the SPEC-031 `doc_lazy_continuation`
  lesson) and **run the lean build** (`cargo build --no-default-features`) before
  finishing — it's CI-only otherwise.
- No new dependency, and do NOT touch the codec feature set or `with_builtins`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
