---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-162
  type: chore                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-063
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: claude-sonnet-5-5   # usually same Claude, different session
  created_at: 2026-11-24

references:
  decisions: [DEC-009, DEC-020, DEC-094]
  constraints: []
  related_specs: [SPEC-124, SPEC-154]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-063's <capability>". Optional; null is acceptable.
value_link: "infrastructure enabling STAGE-063's fixes: every PR gets the same gates for about half the runner time"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: build
      date: 2026-11-24
      model: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 4954980
      tokens_breakdown:
        input: 98
        cache_creation: 107045
        cache_read: 4826722
        output: 21115
      estimated_usd: 2.17
      note: >
        Measured from the session JSONL (found by the prompt's title), deduped by message.id, max output_tokens
        per id, no subagent transcripts. Taken before the last few edits, so it slightly undercounts.
        Rates: Sonnet $3 input, $3.75 cache_creation, $0.30 cache_read, $15 output per MTok.
  totals:
    tokens_total: 4954980
    estimated_usd: 2.17
    session_count: 1
---

# SPEC-162: CI runs each test once

## Context

The maintainer asked on 2026-11-24 whether the build could be faster and whether CI runs duplicate
tests. The orchestrator measured the last full PR run, and CI does run duplicates. There was no
frame or design session: that measurement is the frame, and the decisions below are the design.
Verify still runs in full.

- **Stage:** STAGE-063, next to SPEC-154 (the CI path classifier). The project is PROJ-011.
- **Baseline run:** `45501714010`, PR #270 at `afbee0c`, on 2026-11-23.
  - **About 103 runner-minutes** in all. The slowest legs were heic on macOS (17.9 min) and Windows
    (16.5 min).
  - **The Ubuntu `check` leg:**
    - Build took 2.5 min.
    - Test took 8.3 min: about 3.5 min compiling the tests and 4.7 min running them.
    - Clippy took 0.5 min.
  - **Test run time on Ubuntu** was 281 s over 63 binaries and 1,445 tests:
    - `avif_tile_pin`: **149.9 s for 2 tests**;
    - `recipe_corpus_output`: 54.6 s;
    - `audit_bench`: 30.4 s.

### What the run showed

1. **The `avif` job repeats the Ubuntu `check` leg exactly.**
   - `avif` has been a default feature since DEC-081. So `cargo build/test/clippy --features avif`
     runs the default feature set again.
   - The job's comment says it checks avif "without the other defaults". But `--features` adds to
     the defaults and never removes them, so that combination was never tested.
   - The job costs 11.7 runner-minutes per run, and checks nothing the `check` leg does not.
2. **`avif_tile_pin` builds a second crustyimg from cold inside the test.**
   - `probe_binary()` runs `cargo build --features image/rayon` into
     `std::env::temp_dir()/crustyimg-spec124-probe-target-<pid>`.
   - Because of the PID, the folder is new on every run: nothing is reused, and nothing is ever
     deleted. On 2026-11-24 the maintainer's temp folder held 179 of them, 229 GB in all. They have
     been deleted.
   - The test is `cfg(feature = "avif")`, so it runs on 6 legs: `check` × 3, `webp-lossy` and
     `heic` × 2. Before this spec, it ran on a 7th, the `avif` job.
   - It checks that ravif's tile count ignores the core count (DEC-094), and that does not depend on
     the webp-lossy or heic features.
3. **Only the two wasm jobs cache their builds**, with `Swatinem/rust-cache@v2`. Every other job
   compiles the whole dependency tree from cold.

## Goal

CI runs each check once, with the same coverage:
- the `avif` job is removed;
- the probe builds incrementally, into a folder inside the cargo target directory, and runs only on
  the three `check` legs;
- a build cache is kept only where a measured run shows it pays.

## Inputs

- **`.github/workflows/ci.yml`:** the jobs this spec changes.
- **`tests/avif_tile_pin.rs`:** `probe_binary()` (lines 151–181) and the test that uses it.
- **`src/sink/mod.rs:782` and `src/quality/mod.rs:426`:** the two
  `with_num_threads(Some(AVIF_TILE_THREADS))` pins. These are the mutation targets.
- **DEC-020** (its line 146 names the `avif` job) and **DEC-094**.
- **The baseline:** `gh api repos/jysf/crustyimg/actions/runs/45501714010/jobs`.

## Outputs

- **Modified: `.github/workflows/ci.yml`.**
  - The `avif` job is removed.
  - In `webp-lossy` and `heic`, the Test step adds
    `-- --skip avif_output_is_identical_across_ambient_core_counts`.
  - In `check`, `Swatinem/rust-cache@v2` is added, but **only if** AC-7's measurement keeps it.
  - Every changed comment says why in plain, behaviour-first words, with no spec IDs.
- **Modified: `tests/avif_tile_pin.rs`.**
  - `probe_binary()` builds into
    `PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("avif-probe-{:016x}", h))`, where `h`
    hashes `env!("CARGO_MANIFEST_DIR")` with `std::hash::DefaultHasher`.
  - Cargo sets `CARGO_TARGET_TMPDIR` for integration tests, as `<target>/tmp`. The folder stays the
    same from run to run, so the nested build is incremental, and `cargo clean` deletes it.
  - The hash gives each checkout its own probe folder, even when worktrees share one
    `CARGO_TARGET_DIR`. A shared target folder has served a stale library across worktrees before.
  - The doc comment is updated to match.
- **Not changed:** `src/`, every other test, `justfile`, `scripts/`, the DECs and `docs/`.
  - DEC-020 stays as written, because a DEC records the decision as of its date.
  - Build searches `decisions/`, `docs/`, `AGENTS.md` and `scripts/` for anything that names the
    `avif` job as current. It lists each hit in its report and changes none of them.

## Acceptance Criteria

- [ ] **AC-1: the `avif` job is gone, and nothing else changed.**
  - No job is named `avif`.
  - The required checks are still the three `check` legs:
    `gh api repos/jysf/crustyimg/branches/main/protection/required_status_checks --jq .contexts`.
  - `git diff --stat origin/main` shows only `ci.yml`, `tests/avif_tile_pin.rs` and the SPEC-162
    files.
- [ ] **AC-2: the probe is incremental.** Run `cargo test --test avif_tile_pin` twice on one target
  folder.
  - Quote each run's wall time.
  - Quote the nested build's `Compiling` and `Fresh` lines from the second run.
  - The second run compiles nothing in the probe folder: neither crustyimg nor any dependency.
- [ ] **AC-3: nothing is left in the temp folder.**
  - Count `${TMPDIR}crustyimg-spec124-probe-target-*` and `${TMPDIR}avif-probe-*` before and after
    both runs.
  - Both counts are unchanged.
- [ ] **AC-4: the probe still sees a source change** (mutants M1 and M2 below).
  - After a green run on the same probe folder, remove the pin.
  - The probe must rebuild, and the test must go red.
  - Restore the pin, and the test goes green again.
- [ ] **AC-5: two checkouts never share a probe binary** (mutant M3).
  - Use two worktrees with **one shared** `CARGO_TARGET_DIR`.
  - Run the test green in worktree A.
  - In worktree B, with the pin removed, run it: it must be red.
  - Run it in A again: it must be green.
  - Each checkout gets its own `avif-probe-<hash>` folder. List both.
- [ ] **AC-6: two concurrent runs don't corrupt each other.**
  - Start two `cargo test --test avif_tile_pin` at once, in one checkout with one target folder.
  - Both must pass.
  - The second must wait on cargo's lock. Quote the "Blocking waiting for file lock" line, or
    explain why it didn't appear.
- [ ] **AC-7: the build cache is kept only if it pays.**
  - Add `Swatinem/rust-cache@v2` to the three `check` legs and push twice. The first run saves the
    cache; the second restores it, and that second run is the measurement.
  - Keep the cache only if both of these hold:
    - The second run's Build + Test time on **each** of the three legs is at least 20% below that
      leg's baseline.
    - The repository's total cache stays under 6 GB. Check with
      `gh api repos/jysf/crustyimg/actions/cache/usage`, and
      `gh api repos/jysf/crustyimg/actions/caches --jq '.actions_caches[] | "\(.key) \(.size_in_bytes)"'`.
  - Otherwise, remove the step in a later commit and report the numbers.
  - Either way, record the numbers, and whether the probe's nested build was warm on the second run
    (its time in the log).
- [ ] **AC-8: the probe test runs where it should, and only there.** On the final CI run at the PR
  head:
  - each `check` leg's log shows `avif_output_is_identical_across_ambient_core_counts ... ok`;
  - the `webp-lossy` log and both `heic` logs show the `avif_tile_pin` binary with `1 filtered out`,
    and no line for that test;
  - across the whole suite, `cargo test -- --list` shows exactly one test that the skip pattern
    matches.
- [ ] **AC-9: the saving is measured, not estimated.**
  - Sum each job's `completed_at - started_at` for the final run
    (`gh api repos/jysf/crustyimg/actions/runs/<id>/jobs`).
  - Next to the baseline, give:
    - the total runner-minutes;
    - the slowest leg's wall time;
    - each leg's Test step.
  - The orchestrator's estimate was 103 → 55–60 runner-minutes. Report what was measured, even if it
    misses that.
- [ ] **AC-10: every gate is green on the PR head:** the local matrix and the CI legs.

## Failing Tests

There are no new tests. The existing `avif_tile_pin` tests are what this spec changes, and the
mutants below are the controls.

### Mutation table

Apply each mutant alone, then restore. Record every test that goes red.

- **M1.**
  - Mutant: at `src/sink/mod.rs:782`, change `with_num_threads(Some(AVIF_TILE_THREADS))` to
    `with_num_threads(None)`.
  - Expected red:
    - `avif_output_is_identical_across_ambient_core_counts`, because the probe rebuilt and the
      hashes differ;
    - `both_encode_paths_set_the_thread_count` only if this arm reaches it.
  - Record what actually goes red.
- **M2.**
  - Mutant: the same change at `src/quality/mod.rs:426`.
  - Expected red: `both_encode_paths_set_the_thread_count`, because the two arms drift.
  - Record whether the probe test goes red too.
- **M3.**
  - Mutant: M1 in worktree B only, with A and B sharing one `CARGO_TARGET_DIR` (AC-5).
  - Expected: red in B, and green in A both before and after.
- **M4.**
  - Mutant: take the hash out of `probe_binary()`'s folder name, leaving a plain `avif-probe` folder.
    Then repeat M3's drive.
  - Expected: it shows the risk the hash guards against, with B's red or A's green coming out wrong
    in some order.
  - If nothing ever goes wrong, say so. The hash then stays as cheap insurance, and the report says
    it was not proven necessary.

Before M1, check on the unmutated tree that `RAYON_NUM_THREADS` reaches the probe. The test's doc
says that without the pin, 1, 4 and 14 threads give three different hashes. Quote what M1 shows.

## Implementation Context

### Decisions that apply

- **`DEC-094`:** ravif reads the core count through `current_num_threads` unless the tile thread
  count is pinned. The probe test guards this.
- **`DEC-020`:** it introduced the `avif` job when avif was opt-in. DEC-081 made avif a default,
  which turned the job into a duplicate.
- **`DEC-009`:** the three-OS matrix stays. Only advisory jobs change.

### Constraints that apply

- **`no-secrets-in-code`:** unchanged. The cache step needs no secret.
- **DCO:** every commit is signed off.

### Overlap with other open work (taken 2026-11-24 20:34 EST)

- **`PR #237`** (the heifer swap, a draft, last updated 2026-11-07) rewrites the `heic` job's install
  steps. This spec adds `--skip` to the same job's Test step. The hunks are close but don't touch,
  so whichever PR merges second rebases.
- **`PR #240`** (wasm32 clippy, a draft, last updated 2026-11-02) adds steps to `ddphotos-parity`.
  It doesn't overlap.
- At Step 0, the build re-runs `gh pr list --state open --json number,title,files` and records any
  change.

### Prior related work

- **`SPEC-124`** added the probe, which is where the `spec124` folder name comes from.
- **`SPEC-154`** added the path classifier. Docs-only pushes already skip every heavy job.

### Out of scope

- **Skipping `recipe_corpus_output` and `audit_bench` on the feature legs.**
  - `webp-lossy` and `heic` change encode and decode paths that those tests may pin.
  - Nobody has shown which rows don't depend on the feature, so both tests keep running everywhere.
- **The rerun on push to `main` after each merge.** It records the result of every `main` commit, by
  design; see the `concurrency` comment.
- **A `--no-default-features --features avif` check.**
  - The `avif` job claimed to cover this but never did, so removing the job loses nothing that was
    being checked.
  - If it's wanted, it becomes a new idea entry.
- **`opt-level`, linker or profile changes.** A local speedup was a CI slowdown once (19 → 29 min).

## Notes for the Implementer

- **Measure CI from the API, not the UI.**
  - For the baseline, use run `45501714010`'s jobs.
  - For each new run, get its ID with `gh run list --branch <branch> --limit 3`.
- **Never poll CI.**
  - After each push, run `gh pr checks <PR> --watch` once in the background.
  - If it returns before the checks register, run it once more.
- **The nested `cargo build` takes the outer environment.** Don't add `CARGO_TARGET_DIR` or features
  to the outer command.
- **`CARGO_TARGET_TMPDIR` is a compile-time `env!`.** It exists only for integration tests and
  benches; `tests/avif_tile_pin.rs` is an integration test.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** chore/spec-162-ci-runs-each-test-once
- **PR (if applicable):** #271 (draft)
- **All acceptance criteria met?** yes, with two findings on AC-5 and M4 (below)
- **New decisions emitted:** none
- **Files this diff touches:**
  - `.github/workflows/ci.yml` — `avif` job removed; `--skip avif_output_is_identical_across_ambient_core_counts` on the `webp-lossy` and `heic` Test steps; `Swatinem/rust-cache@v2` (key `check`) on the `check` legs; the `changes` comment no longer names `avif`.
  - `tests/avif_tile_pin.rs` — the probe builds into `<target>/tmp/avif-probe-<hash of CARGO_MANIFEST_DIR>`; doc comment updated.
  - `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-162-ci-runs-each-test-once.md` — this section and the cost entry.
- **Acceptance criteria** (local drives used fresh `CARGO_TARGET_DIR`s):
  - AC-1: met. No `avif` job; `git diff --name-only origin/main...HEAD` lists only the three files above. The required checks are the three `check` legs, whose job and `name:` are unchanged.
  - AC-2: met. Run 1 55 s (435 `Compiling` lines, outer plus nested); run 2 1 s, 0 `Compiling` lines (`Finished dev ... in 0.12s`). A verbose third run showed `Fresh crustyimg`, `Fresh ravif`, `Fresh image` and 0 `Compiling`/`Dirty`. Baseline on `main`: 57 s.
  - AC-3: met. Before: 0 old (`crustyimg-spec124-probe-target-*`) and 0 new (`avif-probe-*`) in `$TMPDIR`. After both runs: 0 and 0. The baseline run on `main` left 1 old folder, which I deleted.
  - AC-4: met. M1 and M2 below: each went red on a probe folder that had just been green, and green again after the restore.
  - AC-5: partly as written. B (pin removed, shared target) went red; A went green before. A's second run went RED, not green (see findings). With A's sources touched, A went green again; the folders were `avif-probe-26f05cfb9e2c46e9` (A) and `avif-probe-66c2fddb20c8b56c` (B).
  - AC-6: met. Two concurrent runs, one checkout, one target: both `2 passed`. The second printed `Blocking waiting for file lock on build directory`.
  - AC-7: met, cache kept. Numbers below.
  - AC-8: met, from run `45539781614` at `ecf0dc7`. All three `check` legs show `avif_output_is_identical_across_ambient_core_counts ... ok`. `webp-lossy`, `heic` ubuntu and `heic` macOS show the `avif_tile_pin` binary with `1 passed; ... 1 filtered out` and only `both_encode_paths_set_the_thread_count`. `cargo test -- --list` has exactly 1 test matching the pattern.
  - AC-9: met. Numbers below.
  - AC-10: met. Local matrix (default, `--no-default-features`, `--features webp-lossy`): fmt, clippy `-D warnings` and full test all green. Every CI leg of run `45539781614` is green. The commits after it are docs-only.
- **Mutants:**
  - Before M1: `RAYON_NUM_THREADS` reaches the probe. With the pin removed the hashes for 1, 2 and 4 threads differ; 4, 8 and 14 give the same hash (the size term caps the tile count at 256x256).
  - M1 (`src/sink/mod.rs:782` -> `None`): RED `avif_output_is_identical_across_ambient_core_counts` and `both_encode_paths_set_the_thread_count`. The table expected the second only "if this arm reaches it", and it does. Restored: green.
  - M2 (`src/quality/mod.rs:426` -> `None`): RED `both_encode_paths_set_the_thread_count` only. The probe test stayed green: the probe runs `convert`, which uses only the sink arm. Matches the table.
  - M3: A green, B red, then A RED (not green). Mismatch, see findings.
  - M4: no mismatch in the order I ran (A green, B red, A touched then green). The probe folder is shared and was rebuilt because the mutated source was newer. The hash is not proven necessary here; it stays as cheap insurance.
- **Cache decision (AC-7): kept.** Run 2 (`45539781614`, cache restored) against the baseline run `45501714010`, Build + Test:
  - ubuntu: 645 s -> 368 s (-42.9%).
  - macOS: 560 s -> 434 s (-22.5%). Tightest margin against the 20% bar.
  - windows: 932 s -> 498 s (-46.6%).
  - Cache total 3.48 GB (9 entries) < 6 GB. The `check` entries: 641 MB (Linux), 634 MB (macOS), 563 MB (Windows).
  - The probe's nested build was warm: `avif_tile_pin` took 20.5 s on ubuntu (baseline 149.9 s), 20.1 s on macOS and 35.7 s on Windows; the nested build compiled only the crustyimg crate.
- **Baseline vs final (runner-minutes):**
  - Total: 102.6 -> 62.3 (run 2). Run 1 (cache cold, 3cfa32d) was 86.9.
  - Slowest leg: 17.9 min (heic macOS) -> 14.5 min (run 1; windows `check` 20.1 min, cold, saving the cache) / 10.7 min heic macOS in run 2; windows `check` 9.0 min in run 2.
  - Test step, baseline -> run 2: ubuntu check 496 -> 349 s; macOS 424 -> 394 s; windows 678 -> 454 s; webp-lossy 396 -> 257 s; heic ubuntu 523 -> 281 s; heic macOS 745 -> 395 s.
  - The 55-60 runner-minute estimate was missed by 2-7 minutes: 62.3 measured. Note that the `heic` and `webp-lossy` legs do not use the cache.
- **Docs sweep (hits listed, none changed):**
  - `decisions/DEC-020-avif-output-feature-gated-ravif.md:146` — names the CI `avif` job.
  - `decisions/DEC-081-avif-encode-moves-into-the-default-feature-set.md:188-190` — says the `avif` job is "kept ... not removed".
  - `docs/sessions/2026-06-16-spec-018-avif-build-handoff.md:60` and `docs/sessions/2026-06-16-stage-004-formats-quality-build-handoff.md:98` — historical handoffs that plan the `avif` job.
- **Overlap check (after the last rebase; no rebase was needed, branch at `origin/main`):** #237 (heic job, rewrites install steps; hunks near but not touching), #229 (docs/roadmap.md) and #226 (AGENTS.md, src/cli/mod.rs) are open. #240 is no longer open (merged as `39e68e0`). No open PR touches the files this PR changes except #237's `ci.yml`.
- **Deviations from spec / findings:**
  - A docs-only commit does not measure the cache: the path classifier skips the heavy steps. The build prompt suggested a Build Completion edit; that first push (`4270619`) was cancelled by the next one. The second measured commit is a one-line `ci.yml` comment (`ecf0dc7`).
  - AC-5 / M3: with two worktrees sharing one `CARGO_TARGET_DIR`, cargo gives both the same unit hash, so A's run after B's reused B's library and B's test binary without compiling. That test binary has B's manifest path baked in, so it probed B's mutated folder, and `both_encode_paths_set_the_thread_count` (which does not use the probe) went red too. The probe hash protects only the nested probe build. It cannot protect the outer artifacts, which is the known stale-library problem for shared target folders. Touching A's sources restores green and A's own `avif-probe-<hash>` folder was never touched.
  - The local matrix's first webp pass failed because zsh did not word-split `$F`; re-run with explicit flags, green.
- **Follow-up work identified:**
  - DEC-020 and DEC-081 still describe the `avif` job; a DEC records its date, so left alone. A note in a future decision or `docs/` is the orchestrator's call.
  - `heic` and `webp-lossy` legs could take the same cache: not measured here.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Which commit measures the cache. The prompt's "fill in Build Completion" suggestion is docs-only, so the classifier skips the heavy steps and measures nothing.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — The shared-target-dir stale-artifact behaviour: AC-5 expects A green after B's red, but cargo reuses B's outer test binary.

3. **If you did this task again, what would you do differently?**
   — Push a `ci.yml` change as the second commit from the start, and touch A's sources before the AC-5 re-run.

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
