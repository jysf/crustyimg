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
- [ ] build (round 2) — address the Windows stack overflow; keep flake gone + pixels identical.
- [ ] verify (round 2) — re-check on the three-OS matrix.
- [ ] ship — orchestrator.
