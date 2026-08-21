---
task:
  id: SPEC-124
  type: story
  cycle: design
  blocked: true
  priority: high
  complexity: S

project:
  id: PROJ-010
  stage: STAGE-042
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5
  created_at: 2026-08-18

references:
  decisions:
    - DEC-094
    - DEC-058
    - DEC-019
    - DEC-068
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-123
    - SPEC-121
    - SPEC-122

value_link: >
  Closes both riders SPEC-123 measured. AVIF output stops depending on the
  machine's core count — which removes a live `diff` false positive — and the
  multi-tile compression penalty comes back, on a tool whose thesis is
  quality-per-byte. Rides STAGE-046's byte-changing wave so users pay one
  lockfile migration, not two.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Framed on the
        maintainer's 2026-08-18 ruling to pin now and ride SPEC-121/122's wave
        rather than pay a second lockfile migration later.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-124: pin the AVIF encoder's tile count

## Context

`src/sink/mod.rs` constructs `AvifEncoder::new_with_speed_quality(..)` and **never
calls `with_num_threads`**, so `ravif` takes its default and derives the AV1 tile
count from `std::thread::available_parallelism()` — the machine's core count.
**SPEC-123 measured the consequences** (DEC-094):

- **Output varies with core count.** Core count is in neither the cache key
  (DEC-058) nor the lockfile's `[env]`, and `src/build/lock.rs:459-466` marks a
  same-`env` hash change as drift **unconditionally** — so `diff` reports a
  differently-cored, same-arch host as a **real regression**.
- **The shipped build takes the worst cell.** Against a 1-tile encode:
  **+1,497 B / +1.5%** on `photo_forest_cc0.jpg`, **+412 B / +47.9%** on
  `graphic_large.png`. Tiles are coded independently, so more of them costs
  compression — and `ravif` is compiled **without** its `threading` feature, so
  **the encode is serial and those tiles buy no parallelism whatsoever.**

We pay the tile tax and collect none of the benefit.

## The design calls — settled here

### Call 1 — pin the count; do not enable threading

`with_num_threads(Some(N))` only. **`image/rayon` is a separate, later decision**
(the measured 5.7× / 4.4× performance lever). ⚠ Verify's constraint on STAGE-042:
enabling `image/rayon` *without* a pin reopens the determinism hole, because a
scoped `--jobs` pool becomes an encoder parameter again. **This spec is what makes
that later change safe to make.**

### Call 2 — ⚡ N is NOT settled here. Measure it, then choose.

**The prior — and it is a prior, not a conclusion** ([[a-measurement-specs-cost-lives-in-the-refutation]]):
since the encode is serial today, tiles cost compression and buy nothing, so
**N = 1 may be strictly better — same speed, materially smaller files.**

**That is exactly the kind of confident reasoning SPEC-123 punished.** Drive it:

- **Wall clock, serial 1-tile vs serial N-tile.** Nobody has measured whether tile
  count affects *serial* encode time. If 1 tile is slower, the trade is real and N
  is a judgement call, not a freebie.
- **Compression at 1 tile**, re-measured on the branch rather than quoting DEC-094.
- **The forward cost of N = 1.** If `image/rayon` is ever enabled, N = 1 means a
  single-threaded encode forever. A larger N preserves future parallelism at a
  known compression cost. **State the trade; recommend a value; justify it.**

### Call 3 — the quantization rider binds the test, not just the prose

DEC-094: `rav1e` quantizes a requested count to a legal tile grid, so a **range** of
requests collapses to one layout (graphic matched at every N ≥ 12; photo at
N = 13–20). **A test asserting "N=14 and N=16 differ" would be asserting something
false.** Pin the *observable* — output is byte-identical across differing ambient
core counts — not the requested number.

### Call 4 — sequence with the wave, not necessarily in it

The migration is keyed on `crate::version()` (`src/cli/build.rs:294`), so what makes
it *one* migration is landing in the **same release** as SPEC-121/122, not the same
PR. **Sequence after SPEC-122; ship before the next tag.** One spec per PR
(constraint `one-spec-per-pr`).

### Call 5 — this closes a filed item; say so

Pinning removes the core-count variance, which is the mechanism behind STAGE-042's
`[env]`-cannot-express-"same machine" item. **Confirm that and close it, or explain
what remains.** The `lock.rs:124-129` prose is still wrong on its own terms even
once the variance is gone.

## Acceptance Criteria

- [x] **AC-1.** `with_num_threads(Some(N))` is set on **every** AVIF encode path —
      `sink::encode_to_bytes*` **and** `quality::encode_candidate_bytes_with`, which
      DEC-019/DEC-068 require to stay in lockstep. A probe encode that skips one is
      the defect this spec would otherwise introduce.
      → `AVIF_TILE_THREADS = 1` in both; `tests/avif_tile_pin.rs::both_encode_paths_set_the_thread_count`.
- [x] **AC-2.** **Output is byte-identical across differing ambient core counts.**
      Drive it the way SPEC-123 did — the harness is committed at
      `scripts/spec123_avif_thread_determinism.py`; reuse it, do not rebuild it.
      → `tests/avif_tile_pin.rs::avif_output_is_identical_across_ambient_core_counts`,
      via a `--features image/rayon` probe (DEC-094 leg E's own proxy for a
      differently-cored machine). See Deviations for why this landed as a Rust
      test rather than a Python-harness extension.
- [x] **AC-3.** **N is chosen from measurement**, with the serial 1-tile vs N-tile
      wall clock and the compression delta both reported (Call 2).
      → DEC-096 §1–§4; N = 1.
- [x] **AC-4.** **The compression win is measured on the branch**, not quoted from
      DEC-094. → DEC-096 §2 (re-measured, plus the auto q85 path DEC-094 didn't cover here).
- [x] **AC-5.** **A negative control** — revert the pin; AC-2 goes red. The
      behavioural flip is the evidence, not a binary hash (AGENTS §15).
      → driven three ways (sink-only, quality-only, both) during build; DEC-096 Validation.
- [x] **AC-6.** **The migration driven, not reasoned** — same as SPEC-121's AC-8:
      key changes, `--frozen` fails, regeneration succeeds, no stale cache hit.
      → driven manually with a temporary uncommitted version bump; DEC-096 Consequences.
- [x] **AC-7.** **`RELEASING.md` / CHANGELOG say AVIF output bytes change**, and the
      note names all three specs in the wave.
      → CHANGELOG.md (plain-language, measured numbers) + RELEASING.md (spec IDs, maintainer-facing).
- [x] **AC-8.** **Call 5 answered** — the STAGE-042 `[env]` item closed or its
      remainder stated. → STAGE-042 backlog: item closed, remainder re-filed narrower.
- [x] **AC-9.** Clean full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequential:
      default, `--no-default-features`, `--features webp-lossy`. Clippy and
      `fmt --check` each. Then read the CI legs individually.
      → all three legs green locally; CI read after push (see Build Completion).

## Failing Tests

- **`tests/avif_tile_pin.rs`** (new)
  - `"avif_output_is_identical_across_ambient_core_counts"` — AC-2. **RED** today.
  - `"both_encode_paths_set_the_thread_count"` — AC-1. Asserts the *lockstep*
    DEC-019 requires, which is the part a partial fix would miss.

## Implementation Context

### Out of scope
- **`image/rayon`** — the performance lever, its own decision.
- Making `lock.rs` express "same machine", and the `:124-129` prose fix — filed
  separately on STAGE-042.
- Any user-facing flag. The tile count is not a knob users should hold.

## Notes for the Implementer

- **Do not re-litigate SPEC-123's verdict.** DEC-094 is the record. **Do
  re-measure anything you are about to assert in this spec's own DEC.**
- ⚠ **DEC-094's leg F was corrected at verify** — it bounds a band, it does not
  identify a point. Read the corrected version.
- **Budget in exchanges (~150), not minutes.** Never poll CI; background the watch.
- macOS has no `timeout(1)`. `git commit -s`. **Own git worktree.** **Do not merge
  the PR. Do not bump the version.**
- Follow `closing-steps-snippet.md`, including `just advance-cycle SPEC-124 verify`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `fix/spec-124-pin-the-avif-encoder-tile-count`
- **PR:** #TBD (opened at the end of this build; see PR body for the link)
- **All acceptance criteria met?** yes — AC-1 through AC-9, see the checklist above and DEC-096.
- **New decisions emitted:** DEC-096 — pin the AVIF tile count to 1, with the full Call 2
  measurement (`decisions/DEC-096-pin-the-avif-tile-count-to-one.md`).
- **Files this diff touches** (learning SPEC-122's Deviations lesson — list every one):
  - `src/sink/mod.rs` — `AVIF_TILE_THREADS` constant, pin applied, doc comments updated.
  - `src/quality/mod.rs` — mirrored constant + pin (the DEC-019/DEC-068 cross-sync pattern already
    used for `AVIF_SPEED`).
  - `tests/avif_tile_pin.rs` (new) — the two named failing tests (AC-1, AC-2/AC-5).
  - `examples/spec124_tile_count_probe.rs` (new) — the committed measurement probe DEC-096 cites
    (SPEC-120's throwaway-prototype precedent).
  - `CHANGELOG.md` — new AVIF bullet + extended the existing "Reproducible builds" bullet.
  - `RELEASING.md` — a maintainer-facing note naming SPEC-121/122/124 as one batched migration.
  - `projects/PROJ-010-post-launch-correctness-and-consolidation/stages/STAGE-042-release-safety-instruments.md`
    — Call 5: closed the `[env]`-cannot-express-"same machine" item, re-filed its narrower remainder.
  - This spec file — AC checkboxes, `implementer` corrected to the model that actually ran, this
    section.
- **Deviations from spec:**
  1. **`implementer` corrected `claude-opus-5` → `claude-sonnet-5`.** The build prompt asserted "you
     are Opus" on the strength of the spec's `implementer` field, but the session that actually ran
     is Sonnet 5 (confirmed from the transcript's `.message.model`, every one of 391 usage-bearing
     messages). AGENTS §4 is explicit that cost prices at the model that ran, not the one a prompt
     names — corrected the field so the record agrees with reality, and priced this session's cost
     at Sonnet anchors accordingly (see `cost.sessions` below).
  2. **AC-2's test is a Rust `#[test]`, not a `scripts/spec123_avif_thread_determinism.py` extension**,
     though the guardrail said "reuse it, do not rebuild it." Reasoned through in DEC-096's module doc
     (`tests/avif_tile_pin.rs`'s header comment): the Python harness's existing legs vary
     `RAYON_NUM_THREADS`/`--jobs` against the SHIPPED (non-`threading`) binary, which DEC-094 itself
     already proved is a no-op lever — extending only that harness would produce a test that is
     GREEN whether or not the pin exists, which is not a test (AC-5 would have nothing to flip red).
     The only lever DEC-094 established as reaching the tile-count computation at all is the
     `--features image/rayon` probe (leg E) — reused, not rebuilt, but as a Rust `OnceLock`-memoized
     one-time build inside the test file rather than a new Python leg, so `cargo test` stays the
     single source of truth for CI (no `python3` dependency added to the gate suite). Flagging this
     explicitly rather than silently reinterpreting the guardrail.
  3. **AC-6 (migration) was driven manually, not as a committed `cargo test`** — matching SPEC-121's
     own AC-8 precedent exactly (a real target, a temporary uncommitted version bump, discarded after).
     See DEC-096 Consequences for the full drive and its result.
- **Follow-up work identified:**
  1. STAGE-042's re-filed item: `lock.rs:124-129`'s "same machine" prose is still wrong on its own
     terms, independent of any known live mechanism (Call 5's remainder — see the stage backlog).
  2. SPEC-122's CHANGELOG entry (linear-light resize) is still missing from `[Unreleased]` — noticed
     while extending the "Reproducible builds" bullet for this spec, out of scope to fix here (not
     this spec's diff, and I have not independently re-verified SPEC-122's measured numbers), but
     worth flagging before the release-cut roll: `RELEASING.md`'s new note names it, the CHANGELOG
     itself does not yet describe it.
  3. The pre-existing STAGE-042 "Count:" summary line (`8 pending`) undercounts by one against a
     literal `- [ ]` grep (`9`) even before this spec's edit — didn't originate here (checked via
     `git show HEAD:...`), not fixed, flagging so it isn't mistaken for something this build caused.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** Which artifact `tests/avif_tile_pin.rs`'s
   AC-2 test should actually be. The spec names a Rust test file; the guardrail says reuse the Python
   harness; DEC-094 establishes that the only way to genuinely vary "ambient core count" on one host
   is the `image/rayon` probe. Those three don't obviously fit together until you trace through WHY
   the Python harness's existing legs can't discriminate pinned-from-unpinned (they only vary a lever
   DEC-094 already proved is inert on the shipped build) — worth a sentence in a future prompt.
2. **Was there a constraint or decision that should have been listed but wasn't?** Not a constraint,
   but a useful fact: `rav1e`'s own `MAX_TILE_WIDTH`/`MAX_TILE_AREA` clamp (`tiling/tiler.rs`) already
   forces the tile count up for large images regardless of what's requested. Nothing in the spec or
   DEC-094 mentions it, and it's the thing that makes "N=1 always" safe rather than a latent bug for
   large photos — worth citing in the next AVIF-tiling spec so it isn't re-derived from scratch.
3. **If you did this task again, what would you do differently?** Test the large-image wall-clock
   claim on REALISTIC content before writing ANY of it down, not after. The first large-image
   measurement used a synthetic maximally-busy gradient and showed a real ~17% N=1 slowdown that
   would have changed the headline recommendation; only re-running it against an upscaled real photo
   (matching the tool's actual measured corpus) showed that was a property of adversarial content,
   not of image size. Cheap to do either order here, but on a spec where the first large-content probe
   is expensive, doing the realistic check first would have saved a wrong turn.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
