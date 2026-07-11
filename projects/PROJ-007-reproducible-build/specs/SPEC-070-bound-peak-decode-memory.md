---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-070
  type: chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # a shared pre-decode dimension→memory check wired into ~4 decode seams + a cap constant + a budget/factor DECISION (DEC-063) + cross-format tests; each site is small, the breadth + the tradeoff call are the weight

project:
  id: PROJ-007
  stage: STAGE-024
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-10

references:
  decisions: [DEC-034, DEC-035, DEC-062, DEC-004]
  constraints:
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - no-new-top-level-deps-without-decision
    - every-public-fn-tested
    - clippy-fmt-clean
    - ergonomic-defaults
  related_specs: [SPEC-058, SPEC-060, SPEC-061, SPEC-069]

value_link: "STAGE-024's peak-memory hardening — closes the F-RAW-1 / general-decode memory-amplification residual SPEC-069 surfaced; the untrusted-input-hardening posture's missing memory bound."

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-10
      notes: >
        Framing/design cycle — main-loop, not separately metered → null-with-note per AGENTS §4.
        Grounded in a firsthand map of the decode-memory surface: the caps in `src/image/mod.rs`
        (`MAX_IMAGE_DIMENSION`, `MAX_ALLOC_BYTES`, `decode_limits()`), the central `decode_with_limits`
        seam + its `ImageReader` generic branch (no pre-decode dim peek today), the per-format entries
        (AVIF `check_caps` + SVG `check_caps` already gate chosen dims; RAW `decode_jpeg_with_limits`
        + generic path do NOT), and `image` 0.25's `Limits` (only width/height/max_alloc — no
        total-pixel field, so the cap must be crustyimg-side). Confirmed on the real binary in SPEC-069
        verify: a 782 B `.nef` peaks ~1.93 GB via `info`. Scope precision set here: closes the
        DECODE-stage peak (F-RAW-1 + general path), NOT F-AVIF-3's upstream parse-stage over-alloc.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-070: bound peak decode memory

## Context

SPEC-069's fuzz gate surfaced a real, product-facing memory-amplification residual (**F-RAW-1**,
same class as F-AVIF-3): a **< 800-byte** crafted `.nef` whose embedded JPEG's SOF declares
**16384×9776** drives the decoder to a **~1.9 GB peak working set** — confirmed on the shipped
release binary (`crustyimg info` on the 782-byte reproducer → `dimensions: 16384x9776`, `peak
memory footprint 1.93 GB`, ≈2470× amplification). It **passes every DEC-034 cap**: 16384 and 9776
are each < `MAX_IMAGE_DIMENSION` (65535), and the RGB output (~480 MB) is < `MAX_ALLOC_BYTES`
(512 MB). The gap is structural — `image::Limits.max_alloc` bounds a **single allocation**, not the
**cumulative/peak** working set, and `image` 0.25 exposes no total-pixel or peak field. So a
near-max-dimension image (embedded RAW preview **or** a plain `.jpg`/`.png`) can be driven to ~2 GB
from a tiny file, on any decode command (`info`/`convert`/`build`/`lint`) — a real memory DoS in a
CI/build context, against the "trust it in CI on untrusted files" thesis.

This spec adds the missing bound: a **total-pixel / peak-memory cap enforced BEFORE the full pixel
decode**, via a cheap header dimension peek at each decode entry that lacks one today. It is the
peak-memory leg of `untrusted-input-hardening`, and it is user-prioritized ahead of the rest of the
STAGE-024 backlog.

## Goal

Add a crustyimg-side **peak-memory budget** (a total-pixel cap derived from a peak-bytes budget ×
an amplification factor over the RGBA output) and enforce it at **every decode seam before the big
allocation**: reject an image whose *declared* dimensions imply more than the budget with a typed
`ImageError::LimitsExceeded`, before `.decode()` runs. Wire it into the two seams that have no
pre-decode dimension check today — the generic `ImageReader` path (`decode_with_limits`) and the
RAW embedded-JPEG candidate path (`decode_jpeg_with_limits`) — and align the AVIF/SVG `check_caps`
(which already have dims) to the same cap. Emit **DEC-063** recording the budget + factor + the
resulting pixel cap + the tradeoff (the largest legitimate image we support). Close **F-RAW-1** (and
the general-decode peak) with a regression, and — because the reproducer is now rejected cheaply at
the header — move it into the always-on `fuzz_corpus_never_panics` smoke (SPEC-069 had to exclude
it). **Explicitly NOT in scope: F-AVIF-3**, which is an upstream `avif-parse` *parse-stage*
over-allocation *before* frame dimensions are known (needs vendoring; stays its own filed item). No
new default dependency.

## Inputs

- **The surface map (read first — the design handoff):** the Implementation Context below carries
  every anchor. Re-confirm against the current tree.
- **Caps + the central seam:** `src/image/mod.rs` — `MAX_IMAGE_DIMENSION` (`:37`), `MAX_ALLOC_BYTES`
  (`:42`), `decode_limits()` (`:276`), `decode_with_limits` (`:302`) and its generic `ImageReader`
  branch (`:350-355`: `with_guessed_format()` → `reader.format()` → **straight to `reader.decode()`
  with no dim peek**). The two production `decode_limits()` call sites: `:363` (`decode_with_format`)
  and `:381` (`raw_preview`).
- **Per-format entries:**
  - `src/image/avif.rs` — `check_caps(w,h,limits)` (`:220`, already includes `w*h*4 > max_alloc`),
    called on container metadata dims at `:180` **before** the RGBA alloc; `frame_size_limit` (`:260`).
  - `src/image/svg.rs` — `check_caps` (`:201`) on the chosen render-target size at `:151`, **before**
    `Pixmap::new` (`:153`).
  - `src/image/raw.rs` — `scan_for_preview` (`:127`) → per-candidate `decode_jpeg_with_limits`
    (`:190-194`: `ImageReader` + `set_format(Jpeg)` + `reader.decode()`, **no SOF dim peek**);
    `MAX_PREVIEW_CANDIDATES` (`:142`).
- **The `image` crate:** `image = "=0.25.10"` (Cargo.toml `:38`). `Limits` has only
  `max_image_width/height/max_alloc` — **no total-pixel/peak field** (so the cap is ours). Header
  peek: `ImageReader::into_dimensions()` (consumes the reader — see the gotcha) or the decoder's
  `.dimensions()` before `.decode()`.
- **The F-RAW-1 evidence + reproducer:** `docs/research/proj-009-fuzz-run.md` (the F-RAW-1 entry);
  the SPEC-069 verify measured ~1.93 GB on the 782-byte reproducer. If the minimized reproducer bytes
  are recoverable (the run record has the description/sha), commit one under `tests/fixtures/fuzz/`.
- **Tests to mirror:** `src/image/mod.rs` `oversized_dimension_png_is_limits_exceeded` (`:624`),
  `normal_image_decodes_under_production_limits` (`:636`), the `_via_seam` limit tests (`:646`,`:661`);
  `src/image/avif.rs` `check_caps` tests (`:527`, `:605`); `src/image/svg.rs` check_caps tests (`:251`).

## Outputs

- **Files created:**
  - `decisions/DEC-063-*.md` — the peak-memory cap decision: the **peak-bytes budget**, the
    **amplification factor** (over the RGBA output; SPEC-069 measured ~4× for JPEG), the resulting
    **total-pixel cap** (`MAX_IMAGE_PIXELS` or equivalent), the **tradeoff** (the largest legitimate
    image this supports — e.g. a ~1 GiB budget × 4× ⇒ ~64–100 Mpix, rejecting the 160 Mpix bomb while
    keeping ~24–50 MP consumer/prosumer photos), that it **supersedes the implicit 128 Mpix
    single-RGBA-buffer bound** the AVIF/SVG `check_caps` had via `max_alloc/4`, and that it does NOT
    close F-AVIF-3 (upstream parse-stage). `affected_scope` = `src/image/{mod,avif,svg,raw}.rs`.
- **Files modified:**
  - `src/image/mod.rs` — add the pixel/peak cap constant next to `MAX_ALLOC_BYTES`; a shared,
    unit-tested helper (e.g. `check_pixel_budget(w, h) -> Result<()>` returning `LimitsExceeded`);
    wire it into the generic path **before `.decode()`** via a header dimension peek (mind the
    `into_dimensions()`-consumes-the-reader gotcha — peek from the decoder or re-wrap the in-memory
    bytes). Consider threading the cap through `decode_limits()`/a small `DecodeCaps` struct so it has
    one source of truth.
  - `src/image/raw.rs` — peek each candidate JPEG's SOF dimensions before `decode_jpeg_with_limits`
    and apply the same check, so an oversized embedded preview is rejected before the ~2 GB decode.
  - `src/image/avif.rs`, `src/image/svg.rs` — extend `check_caps` (or call the shared helper) so the
    total-pixel cap applies uniformly (dims are already available there).
  - The **hardcoded test mirrors** that bypass `decode_limits()` — `raw.rs:234` (`generous()`) and
    `avif.rs:609` — updated to carry the new cap.
  - `tests/` (or the `#[cfg(test)]` modules) — the regressions below.
  - `tests/fuzz_regressions.rs` / `fuzz_corpus_never_panics` — **move F-RAW-1's reproducer into the
    always-on smoke** now that it's rejected cheaply at the header (update the SPEC-069 note that
    excluded it); `docs/research/proj-009-fuzz-run.md` F-RAW-1 entry → mark closed by SPEC-070.
- **New exports:** possibly `crustyimg::image::check_pixel_budget` (or keep it private with a test
  seam like `compute_key_with_schema` did for SPEC-064 — decide by testability).

## Acceptance Criteria

- [ ] A crafted image whose **declared** dimensions exceed the peak budget (the F-RAW-1 reproducer:
  a tiny `.nef`/`.jpg` declaring 16384×9776) is **rejected with `ImageError::LimitsExceeded` BEFORE
  the full decode**, on the RAW path AND the generic JPEG/PNG path — verified on the **real binary**
  with peak RSS bounded (`/usr/bin/time -l crustyimg info <reproducer>` peaks well under the budget,
  not ~1.9 GB).
- [ ] A **legitimate large image at the supported ceiling** (per DEC-063, e.g. a ~24 MP photo) still
  decodes correctly (dims + pixels intact) — the cap rejects the bomb without rejecting real photos.
  A test drives both sides of the boundary.
- [ ] The cap is enforced **uniformly** across AVIF / SVG / RAW / generic (PNG/JPEG) — no decode
  entry reaches the big allocation with unchecked declared dimensions. The two hardcoded test mirrors
  (`raw.rs`, `avif.rs`) carry the new cap so they don't silently diverge.
- [ ] **F-RAW-1 is closed and its reproducer joins `fuzz_corpus_never_panics`** (it's now rejected at
  the header, so the smoke can run it without OOM risk); the run record's F-RAW-1 entry is marked
  closed-by-SPEC-070. **F-AVIF-3 is explicitly left open** (upstream parse-stage) and the spec/DEC say
  so — no overclaim (the SPEC-068/069 lesson).
- [ ] **DEC-063** records the budget + amplification factor + pixel cap + the largest-supported-image
  tradeoff + that it supersedes the implicit 128 Mpix single-buffer bound + the F-AVIF-3 exclusion.
- [ ] **No new default dependency** (`git diff main -- Cargo.toml Cargo.lock deny.toml` empty). Full
  gate matrix green incl. lean build; no `unwrap` on recoverable paths; the pre-decode check is a
  typed error, never a panic. `cargo clippy --all-targets -- -D warnings` + `cargo fmt --check` +
  `just deny` + `just validate`.

## Failing Tests

Written during design/build, BEFORE the fix. Build makes them pass.

- **`src/image/mod.rs`** (`#[cfg(test)]`)
  - `"declared_oversize_pixels_rejected_before_decode"` — a valid small file whose header declares a
    total-pixel count over the cap (e.g. a JPEG with a 16384×9776 SOF, or a synthetic PNG IHDR) →
    `Err(LimitsExceeded)`, and (assert the *cheapness*) it does not allocate the full buffer — e.g.
    the error arrives without the decode running (a timing/instrumentation-light assertion, or assert
    on a tiny input that couldn't hold the pixels).
  - `"legitimate_large_image_within_budget_decodes"` — an image at/just under the pixel cap decodes
    to the expected dims (the boundary's allowed side).
  - `"pixel_budget_helper_math"` — unit-test the pure `check_pixel_budget(w,h)`: just over the cap →
    Err, just under → Ok, and a `w×h` that overflows `u64` is handled (saturating), not a panic.
- **`src/image/raw.rs`**
  - `"raw_preview_rejects_oversize_embedded_jpeg_before_decode"` — a synthetic RAW carrying an
    embedded JPEG with an over-cap SOF → the candidate is rejected (no ~2 GB decode); a RAW with a
    normal-size preview still extracts it.
- **`tests/` (integration, real binary)**
  - `"info_on_pixel_bomb_is_bounded_and_exit_code"` — drive `crustyimg info` on the F-RAW-1
    reproducer: exits with the limits error code (not 0), and does NOT peak multi-GB (bounded RSS).
  - Add the reproducer to `fuzz_corpus_never_panics` (now safe) — the smoke stays green.
- **AVIF/SVG:** extend the existing `check_caps` tests so an over-pixel-cap (but under old per-side)
  dims case is rejected.

## Implementation Context

*Read this and re-confirm anchors. The map was read firsthand; the caps + seams are current.*

### The gap, precisely
`image::Limits.max_alloc` (512 MB, `mod.rs:42`) is a **per-allocation** budget (the crate decrements
it per `reserve()` and restores on free), so several sub-512 MB buffers sum to ~1.9 GB peak without
tripping it — and the crate marks it "non-strict, some decoders may ignore it." Only
`max_image_width/height` are strict, and per-side caps can't bound `w×h` (a 65535×65535 image =
4.3 **billion** px passes both side caps). There is **no** total-pixel field in `image` 0.25's
`Limits`. So the bound must be a **crustyimg pre-decode check on declared `w×h`**.

### Where the check slots (dims available pre-decode)
- **AVIF** (`avif.rs:180`) and **SVG** (`svg.rs:151`) already call `check_caps` with the dims known
  before allocation — extend/redirect these to the shared cap. (They already reject `w*h*4 >
  max_alloc`, i.e. a 128 Mpix single-RGBA-buffer bound; the new cap is tighter and uniform.)
- **Generic** (`mod.rs:350-355`) and **RAW candidate** (`raw.rs:190-194`) jump straight to
  `.decode()`. Insert a header dimension peek before decode. **Gotcha:** `ImageReader::into_dimensions()`
  consumes the reader — either read dims from the decoder (`with_guessed_format()` →
  decoder `.dimensions()` → then decode via that decoder) or re-wrap the in-memory bytes in a fresh
  `Cursor` for the actual decode (cheap; bytes are already in memory). RAW's reader is JPEG-forced
  (`set_format(Jpeg)`), so a peek reads the SOF only.

### The cap + the decision (DEC-063)
- Compute an estimated peak = `w · h · bpp · factor`. SPEC-069 measured **~4×** the RGB output for
  JPEG (480 MB output → ~1.9 GB). Pick a **peak-bytes budget** and derive `MAX_IMAGE_PIXELS`.
  **Recommendation (finalize in DEC-063):** budget ≈ **1 GiB**, factor **4** over the RGBA (×4 bytes)
  output ⇒ `MAX_IMAGE_PIXELS ≈ 64 Mpix` (~8000×8000) — rejects the 160 Mpix bomb, keeps essentially
  all consumer/prosumer photography (24–50 MP); a >64 MP medium-format image is rejected (raise the
  budget if that matters — that's the tradeoff to state, not hide).
- Keep `MAX_IMAGE_DIMENSION` (65535) as the per-side backstop; the new cap is the product bound.
- Use **saturating** arithmetic for `w×h` (u32×u32 into u64) — never overflow/panic on hostile dims.

### Scope precision (do NOT overclaim)
- **Closes:** F-RAW-1 and the general JPEG/PNG decode-stage peak (the reachable, product-facing case).
- **Does NOT close:** F-AVIF-3 — an `avif-parse` allocation during **container parsing**, *before*
  `check_caps` sees frame dims; not reachable by a dimension peek without vendoring avif-parse. Say so
  in the DEC + run record; it stays the separately-filed upstream item.

### Constraints
- `untrusted-input-hardening` (this is its peak-memory leg), `no-unwrap-on-recoverable-paths` (the
  check is a typed `LimitsExceeded`), `no-new-top-level-deps-without-decision` (none),
  `every-public-fn-tested` (the pure `check_pixel_budget`), `clippy-fmt-clean`, `ergonomic-defaults`
  (a legitimate photo must still decode — the cap is a bomb filter, not a low ceiling).

### Out of scope
- F-AVIF-3 / vendoring avif-parse; a true runtime peak-RSS accounting or custom global allocator
  (the declared-dimension estimate is the pragmatic bound — an XL allocator approach is explicitly
  not taken); raising/lowering `MAX_IMAGE_DIMENSION`; the other STAGE-024 backlog items.

## Notes for the Implementer

- **One source of truth for the cap**, reached by all four decode paths — don't scatter the constant.
  Thread it through `decode_limits()` or a small `DecodeCaps` so a future change is one edit (and the
  two hardcoded test mirrors don't silently diverge again).
- **Drive the real binary** (`/usr/bin/time -l crustyimg info <reproducer>`) to prove the peak is
  bounded — a passing unit test that rejects the dims is necessary but the RSS measurement is the
  real proof (the wave's lesson). Also drive a legitimate large photo to prove no false rejection.
- **The reproducer graduates:** once rejected at the header, add F-RAW-1's input to
  `fuzz_corpus_never_panics` and flip the run-record note — a small, satisfying closure of the
  SPEC-069 residual.
- Emit `DEC-063`; state the F-AVIF-3 exclusion explicitly so the record stays honest.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-063` — peak decode-memory cap (budget + amplification factor + pixel cap + tradeoff; F-AVIF-3 excluded)
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

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
