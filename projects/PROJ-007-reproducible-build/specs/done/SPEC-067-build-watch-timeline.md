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
- [x] **build** — `src/build/watch.rs` (pure `watch_roots`/`is_excluded`/`debounce` + `WatchSet`/`WatchError`) +
  wired the `Commands::Build` path to the watch loop under `--watch`; `cargo add notify` + `just deny` first;
  DEC-060; `notify` behind a default-on `watch` feature. All Failing Tests pass; default + lean + deny + clippy
  + fmt green; no async runtime (DEC-006 holds). → **PR #74** on `feat/spec-067-build-watch`, 26/26 3-OS CI green.
  `just deny` green via **3 scoped per-crate exceptions** (notify CC0-1.0, inotify/inotify-sys ISC — a real
  decision point in DEC-060, not a silent add). Deviations (all sound, in Build Completion): `--frozen`/`--locked`
  are clap aliases of `--check` (one guard field); a new `CliError::Watch`→exit 1; **two-tier WatchSet** (recursive
  source roots, shallow manifest/recipe dirs) — the 3-OS CI caught a Linux inotify overflow from watching the
  cache tree recursively, fixed exactly as the wave's cross-platform lesson predicts; `is_excluded` lexical-
  absolutizes vs CWD (not `canonicalize`). Est. ~350k tok / ~$3.15 (labelled main-loop estimate §4). 2026-07-10.
- [x] **verify** — fresh session. **CLEAN, ships — empty punch list.** Drove the real release binary on macOS
  against every criterion with adversarial input: self-trigger crux confirmed under symlinked `/tmp`, a
  non-symlinked `$HOME` path, and a whole-cwd `.` glob (strongest confirmation the component-wise CWD-normalized
  exclusion fires); the two-tier watch shown NOT to under-watch (source/recipe/manifest edits + glob add/remove
  all rebuild, cache prunes correctly); 24-write burst → one rebuild; resilience/startup asymmetry; `--watch`×
  verify (all 3 spellings + reversed order) → exit 2; committed lockfile byte-identical across a rebuild; lean-
  build error clear. All gates re-run green (685 + 7 integration, clippy/fmt/deny/validate). Two non-blocking
  observations → STAGE-024 backlog (stray `--watch` on non-build subcommands is a silent no-op; no orphan-output
  prune on source removal). Est. ~140k tok / ~$1.26 (labelled estimate §4). 2026-07-10.
- [x] **ship** — squash-merged **PR #74** → main (**c2ec46a**); filled build/verify/ship cost sessions +
  `cost.totals` (540k tok / ~$4.86, 4 sessions — build/verify/ship are labelled main-loop estimates §4) + ship
  reflection; timeline [x]; **STAGE-023 → shipped** (SPEC-067 was its only spec); brief + stage backlog updated;
  archived spec+timeline to `done/`; `just cost-audit` + `just validate` green; brag entry added. **STAGE-023
  SHIPPED → PROJ-007 has build+cache+lockfile+watch all in.** PROJ-007 does NOT close yet: **STAGE-024**
  (hardening & security sweep) remains, queued last — the Project Ship reflection runs after it. 2026-07-10.
