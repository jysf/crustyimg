# SPEC-067 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — `crustyimg build --watch` (STAGE-023, the last spec of PROJ-007; the dev inner loop).
  A `--watch` flag + a debounced loop over the shipped `run_build`: initial build → watch the manifest +
  each target's recipe + source roots (recursive) → debounce a burst into one rebuild → re-run `run_build`
  (cache prunes to affected outputs — **no dependency graph, the STAGE-021 cache makes a full re-run
  incremental**) → loop-resilient (a failing cycle prints + continues) → Ctrl-C exits (default SIGINT, no
  `ctrlc` dep). **Correctness crux: do NOT self-trigger** — exclude each target's `out`, `.crustyimg/`, and
  the lockfile from the watch triggers, or the build's own writes loop forever. **Lockfile: watch does NOT
  rewrite it** (pre-commit iteration, not a commit). Testability: pure `watch_roots`/`is_excluded`/`debounce`
  (unit-tested deterministically) + a thin `notify` loop + timing-tolerant child-process integration tests
  (may `#[ignore]` if CI flakes). **One new dep = `notify`** (threads+mpsc, NOT async → DEC-006 holds;
  permissive, `just deny` right after `cargo add`) → **DEC-060** (dep + debounce + loop semantics + exclusion
  + feature-gate, mirror `display`/DEC-027). Failing Tests: watch-roots-covers / excludes-outputs-cache-lock /
  is-excluded / debounce-coalesces (unit) + rebuilds-on-source-change / does-not-self-trigger /
  survives-failing-cycle (integration). Framing, 2026-07-09.
- [ ] **build** — `src/build/watch.rs` (pure `watch_roots`/`is_excluded`/`debounce` + `WatchError`) + wire the
  `Commands::Build` path to the watch loop under `--watch`; `cargo add notify` + `just deny` FIRST; DEC-060;
  decide the `watch` feature gate. Make all Failing Tests pass. Verify default + lean + `just deny` (permissive,
  no new exception) + clippy + fmt; confirm no async runtime dragged in.
- [ ] **verify** — fresh session. Re-run gates; drive the real binary: a source edit rebuilds only the affected
  output; a recipe/manifest edit rebuilds; a burst debounces to one rebuild; **the build does not wake itself**
  (no infinite loop); a failing cycle keeps the loop alive; Ctrl-C exits. Confirm one permissive dep, no async,
  lean build clean.
- [ ] **ship** — merge PR; verify + ship cost sessions + totals + reflection; archive to done/. STAGE-023
  backlog: SPEC-067 shipped → **STAGE-023 SHIPPED** → **PROJ-007 COMPLETE** (build+cache+lockfile+watch). Run
  the Project Ship reflection + update the brief; note the roadmap moves to the next wave.
