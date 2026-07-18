# SPEC-091 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-091-<cycle>.md`.

## Instructions

- [x] design — framed build-ready 2026-07-17. Origin: SPEC-088's fix pass reported a PRE-EXISTING
  `--features avif` flake (re_rav1d `DisjointMut` overlap panic, `disjoint_mut.rs:837`). Maintainer chose
  "investigate first, then decide"; the orchestrator's design-time investigation found: (1) severity is
  BOUNDED — upstream's contract says provenanceless targets ⇒ a release race yields wrong results, NOT
  memory unsafety (so NOT a security fix); (2) the likely mechanism — `decode_obus` uses `Settings::new()`
  (dav1d default `n_threads = 0` = ALL CORES) and never caps threads, so every decoder spawns its own pool,
  inside a rayon batch that already parallelizes across files; `set_n_threads` exists and is unused;
  (3) capping is plausibly a perf WIN (a still frame can't use frame-threading); (4) NOT reproduced locally
  in 3 runs — the build must establish a repro FIRST. Lands before SPEC-083 so BENCHMARKS decode timings
  aren't measured under oversubscription. DEC at build.
- [x] build — `n_threads=1` (DEC-077), branch `spec-091-avif-threads`, PR #95. Repro
  established, flake killed, pixels byte-identical, throughput measured. See Build Completion.
- [x] verify — 2026-07-18, Opus 4.8, primary checkout. ⚠ **PUNCH LIST → returns to build.**
  Core fix independently verified sound (flake gone 0/1200 + negative control cap→0 panics
  return + n_threads=4 still flakes; pixel golden re-derived byte-exact; single-image ~3.2×
  slower, flagship parallel a wash-to-9%-faster; DEC-077 honest; fixture license-clean; all
  macOS gates green). **BLOCKER:** `n_threads=1` runs the decode inline on the caller's
  thread, which overflows Windows' ~1 MB main-thread stack → `optimize_avif_input_writes_webp`
  crashes ("main has overflowed its stack") on the **default** build, RED on both PR #95
  Windows CI runs (parent was green). Every AVIF decode on Windows now stack-overflows.
  Fix: run the inline decode on a thread with an adequate stack, re-validate on Windows CI.
- [x] build (round 2) — 8 MiB scoped decode thread (DEC-077), commit d87d389. Windows CI green;
  flake gone + pixels identical re-proven. See Build Completion — Round 2.
- [x] verify (round 2) — 2026-07-18, Opus 4.8, primary checkout. **CLEAN.** Focused re-verify of
  the scoped-thread change: (1) hostile OBU bytes → typed Err across the `join` (nonempty junk
  driven directly; five hostile fixtures typed-error through the CLI; DEC-034 frame-size guard
  still inside the thread); (2) scoped-thread panic → join Err, no deadlock; (3) **negative
  control** — reverting the wrap in place aborts the small-caller-stack test with SIGABRT, so the
  regression test bites; (4) pixel golden unchanged; (5) rayon composition — no oversubscription,
  full avif suite clean; (6) PR #95 matrix green at HEAD d1d901d (27/0, both windows legs). All
  local gates green. One out-of-scope P3 note: an **empty** OBU stream hits re_rav1d's debug-only
  `debug_abort()` (uncatchable, pre-existing, not round-2); primary path guarded by the metadata
  pre-check, alpha path unguarded (plausible/unconfirmed reachability) — cheap one-line follow-up.
  See Verify Completion — Round 2.
- [x] ship — squash-merged PR #95 (**f5d3859**) 2026-07-18 after CI settled CLEAN on the verify commit
  (ddcc3ac) with no flake recurrence. Full three-OS matrix green throughout. Bookkeeping: cycle→ship,
  5 cost sessions (build $2.35 / verify $1.80 / build-r2 $1.70 / verify-r2 $2.05 / ship $0.75 ≈ **$8.65**,
  all `model:`-tagged), DEC-077, timeline, STAGE-030, archive, memory + brag. **Lessons banked:**
  [[a-green-gate-on-one-os-is-not-the-required-matrix]] (the round-1 Windows miss) + verify's
  [[a-thread-boundary-does-not-catch-abort]]. Three follow-ups filed: upstream re_rav1d report,
  empty-OBU debug_abort guard, `par_iter run_pixel_op`.
