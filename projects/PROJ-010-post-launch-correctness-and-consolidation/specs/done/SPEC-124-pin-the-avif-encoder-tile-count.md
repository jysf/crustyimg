---
task:
  id: SPEC-124
  type: story
  cycle: ship
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
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 89964179
      duration_minutes: 88
      recorded_at: 2026-08-21
      tokens_breakdown:
        input: 562
        output: 207301
        cache_creation: 466341
        cache_read: 89289975
      estimated_usd: 31.65
      note: >
        MEASURED — full-session transcript sum over all 503 assistant messages
        with usage (session c900b43d-a96a-4a41-8d08-2d2d7dce4d62), first→last
        timestamp. ⚠ CORRECTED BY THE ORCHESTRATOR AT SHIP. The cycle recorded
        150697084 / $53.89 / 87m, which prices EXACTLY right at its own snapshot
        — 479 of 503 messages — but stopped counting before the session did: a
        cycle cannot count the messages that write its own cost block. Delta
        +11,905,515 tokens / +$3.85. Fourth instance of this (SPEC-114,
        SPEC-117, SPEC-123, now this), and SPEC-123 showed the same curve
        three times over ($32.80 at 242 → $40.37 at 285 → $46.17 at 318).
        Priced at Sonnet anchors ($3/$15 per MTok; cache_creation x1.25,
        cache_read x0.10) — the model that actually ran, per AGENTS §4, not
        the Opus anchors the build prompt named (see Build Completion
        Deviations #1). Orchestrator re-derived every component from the
        transcript; all 503 messages report claude-sonnet-5, confirming the
        `implementer` correction. Includes reading CI to completion after push
        (one long background wait via `gh pr checks --watch`, read once after
        settling, not polled).
        ⚠ CORRECTED 2026-09-05 (SPEC-127 verify + orchestrator, independently).
        The original figure summed EVERY transcript line carrying `usage`. Claude
        Code writes one line per CONTENT BLOCK, and lines sharing a `.message.id`
        repeat identical input/cache_creation/cache_read, so the three static
        fields were double-counted once per extra block. Recomputed by deduping on
        `.message.id`, taking those three from the group and MAX output.
        Was $57.74 / 162,602,599 tokens (1.82x over) over the same
        503 transcript lines = 281 real API calls. See STAGE-053.
    - cycle: verify
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 9773993
      duration_minutes: 68
      recorded_at: 2026-08-21
      tokens_breakdown:
        input: 140
        output: 67911
        cache_creation: 186090
        cache_read: 9519852
      estimated_usd: 7.62
      note: >
        MEASURED — full-session transcript sum over all 136 usage-bearing
        assistant messages (session 016c54b7-f9d9-4ac2-8f72-4723216092b5;
        134 claude-opus-5 + 2 synthetic), first→last timestamp. Priced at Opus
        anchors ($5/$25 per MTok; cache_creation x1.25, cache_read x0.10).
        The cycle recorded 17522773 / $13.74 / 65m at 129 messages and SAID SO —
        it flagged its own snapshot and asked for re-derivation at ship, which
        is the first cycle in this project to anticipate the undercount rather
        than fall into it. Orchestrator re-derived: delta +1,479,700 tokens /
        +$1.53. Verify at 26% of build cost returned a 5-item punch list.
        ⚠ CORRECTED 2026-09-05 (SPEC-127 verify + orchestrator, independently).
        The original figure summed EVERY transcript line carrying `usage`. Claude
        Code writes one line per CONTENT BLOCK, and lines sharing a `.message.id`
        repeat identical input/cache_creation/cache_read, so the three static
        fields were double-counted once per extra block. Recomputed by deduping on
        `.message.id`, taking those three from the group and MAX output.
        Was $15.27 / 19,002,473 tokens (2.00x over) over the same
        136 transcript lines = 72 real API calls. See STAGE-053.
    - cycle: ship
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop orchestrator cycle (AGENTS §4) — reflection, cost
        re-derivation, totals, archive. Null-with-note is the sanctioned form
        for design/ship; build and verify above are both MEASURED.
  totals:
    tokens_total: 99738172
    estimated_usd: 39.27
    session_count: 2  # design + build + verify + ship
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
      → all three legs green locally; CI read individually after push (`gh pr checks 184`,
      not the `--watch` summary — see Build Completion): all 16 real checks pass —
      3-OS `build/test/clippy/fmt` matrix, `avif`/`webp-lossy`/`heic`×2-OS features,
      lean build, msrv, DCO, front-matter, cost-capture audit, supply-chain policy.
      Release/deploy jobs show `skipping` (tag-only), as expected on a PR.

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
- **PR:** [#184](https://github.com/jysf/crustyimg/pull/184)
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
  - `decisions/DEC-096-pin-the-avif-tile-count-to-one.md` (new) — the Call 2 measurement record.
    (⚠ **Added at verify**: the enumeration listed 8 of the 9 files, omitting this one even though
    the bullet above names it with its full path. `affected_scope` was not left blind, so SPEC-122's
    lesson held — but the list was still short by one.)
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
  3. ~~The pre-existing STAGE-042 "Count:" summary line (`8 pending`) undercounts by one against a
     literal `- [ ]` grep (`9`).~~ ⚠ **WITHDRAWN at verify (2026-08-21) — a false finding.** The line
     does not undercount: the 9th `- [ ]` is SPEC-118, counted by the same line under "3 framed."
     At the base commit it reconciles exactly — 8 pending + 3 framed + 1 shipped + 1 chore done =
     13 = 9 `- [ ]` + 4 `- [x]`. **What this diff DOES introduce is a 14th item in no category** (the
     newly-closed `[env]` entry), which is the real bookkeeping gap. A grep's scope is also a claim
     [[mechanical-sweeps-need-a-mechanical-check]].
  4. `just wasm-check` fails on this machine — `rust-lld: Library not loaded: @rpath/libLLVM.dylib` —
     and reproduces identically on a clean `main` checkout with none of this diff's changes, so it is
     a pre-existing local rustup/wasm32 toolchain gap, not something this spec broke. Matches
     STAGE-042's own "no CI leg runs `just wasm-test`" finding (still filed, still not this spec's to
     fix) — this build could not locally verify the wasm target compiles, though nothing in the diff
     (a builder-pattern setter call) has wasm-specific implications, and the `avif feature` CI job
     passed, which exercises the same encode call on native.
  5. **`gh pr checks --watch`'s own summary line is not reliable** — it reported `451 pass / 0 fail /
     223 pending` and exited 0, but a direct `gh pr checks 184` immediately after showed every real
     check `pass` and the rest `skipping` (release/deploy jobs, tag-only). Read the direct snapshot,
     not the watcher's tally, for the actual verdict — worth a line in a future prompt's CI-reading
     guardrail.

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

**Shipped 2026-08-21.** PR [#184](https://github.com/jysf/crustyimg/pull/184) → `0107a49`, merged
mid-punch-list; the 5 verify items applied on `main` at `7b9b04d`. Total **$73.01**
(build $57.74 + verify $15.27, both re-derived from transcripts at ship).

1. **What would I do differently next time?**
   — **State a measurement's fixture in terms of what it measures, not what it looks like.** The
   whole spec was built on the right instinct — Call 2 explicitly refused to settle N at design and
   demanded four axes be driven — and the build did drive them. It then described its 24 MP
   fixture as *"per-pixel-random … no redundancy anywhere"* when it was a deterministic sawtooth
   encoding at 0.157 bpp, and that single wrong adjective carried the entire argument dismissing a
   real 17 % regression. **The measurement was sound; the sentence about it was not**, and a
   decision record is read for its sentences. Two cheap habits would have caught it: report the
   fixture's own encoded bits-per-pixel next to its description, and re-read every adjective in a
   record as if it were an assertion — because it is.

2. **Does any template, constraint, or decision need updating?**
   — **Yes, and one is already done.** `projects/_templates/spec.md` gained a required *"Files this
   diff touches"* field: both builds in this wave added one by hand after SPEC-122's lesson and
   SPEC-124's still listed 8 of 9, which is what a field, not a habit, is for.
   ⚠ **Still open — DEC-096's `insight.confidence` is `0.88`, set when §1 claimed "no trend" and
   §4b claimed the regression was explained.** Both weakened at verify. The decision itself did not
   (§2's compression win and §3's structural argument are untouched and were independently
   replicated), so the number is defensible — but it has not been re-examined since the evidence
   under it moved, and AGENTS §17 exists to stop exactly that drift. **A maintainer call, flagged
   not taken.**
   Also unresolved by design: this repo has no written rule for the case that bit twice this wave —
   a cycle's cost block cannot count the messages that write it, so every self-reported figure
   under-reports by 3–7 %. `cost-snippet.md` warns about premature readings; it does not say the
   residual is structural and the orchestrator must re-derive. It should.

3. **Is there a follow-up spec I should write now before I forget?**
   — **Not a spec — a corpus item, and it is filed.** Every open question §4b left resolves the same
   cheap way: `bench/corpus/` has no large real photograph, so nothing with genuine 24 MP detail has
   ever been measured through this encoder. Filed on STAGE-042 with the numbers. Two other filed
   items came out of this spec rather than a spec of their own: `tests/avif_tile_pin.rs`'s in-test
   `cargo build` (~+15–25 min CI per PR, a 1.3 GB dir leaked per test process, and it ships to
   crates.io because `exclude` does not cover `/tests`), and the `lock.rs:124-129` prose remainder.
   **The one thing this spec earned that is not filed anywhere as work:** `image/rayon` is now safe
   to enable, which was the entire point of pinning first. Whoever takes it must re-measure N
   against real parallelism data rather than inherit DEC-096's numbers, which predate threading
   being real.
