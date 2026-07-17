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
- [ ] build — worktree session.
- [ ] verify — independent worktree session.
- [ ] ship — orchestrator.
