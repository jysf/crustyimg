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
  implementer: claude-opus-5
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

- [ ] **AC-1.** `with_num_threads(Some(N))` is set on **every** AVIF encode path —
      `sink::encode_to_bytes*` **and** `quality::encode_candidate_bytes_with`, which
      DEC-019/DEC-068 require to stay in lockstep. A probe encode that skips one is
      the defect this spec would otherwise introduce.
- [ ] **AC-2.** **Output is byte-identical across differing ambient core counts.**
      Drive it the way SPEC-123 did — the harness is committed at
      `scripts/spec123_avif_thread_determinism.py`; reuse it, do not rebuild it.
- [ ] **AC-3.** **N is chosen from measurement**, with the serial 1-tile vs N-tile
      wall clock and the compression delta both reported (Call 2).
- [ ] **AC-4.** **The compression win is measured on the branch**, not quoted from
      DEC-094.
- [ ] **AC-5.** **A negative control** — revert the pin; AC-2 goes red. The
      behavioural flip is the evidence, not a binary hash (AGENTS §15).
- [ ] **AC-6.** **The migration driven, not reasoned** — same as SPEC-121's AC-8:
      key changes, `--frozen` fails, regeneration succeeds, no stale cache hit.
- [ ] **AC-7.** **`RELEASING.md` / CHANGELOG say AVIF output bytes change**, and the
      note names all three specs in the wave.
- [ ] **AC-8.** **Call 5 answered** — the STAGE-042 `[env]` item closed or its
      remainder stated.
- [ ] **AC-9.** Clean full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequential:
      default, `--no-default-features`, `--features webp-lossy`. Clippy and
      `fmt --check` each. Then read the CI legs individually.

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

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
- **Deviations from spec:**
- **Follow-up work identified:**

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
2. **Was there a constraint or decision that should have been listed but wasn't?**
3. **If you did this task again, what would you do differently?**

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
