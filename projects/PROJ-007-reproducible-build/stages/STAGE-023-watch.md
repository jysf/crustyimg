---
# Maps to ContextCore epic-level conventions.
stage:
  id: STAGE-023
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: medium
  target_complete: null

project:
  id: PROJ-007
repo:
  id: crustyimg

created_at: 2026-07-09
shipped_at: null

value_contribution:
  advances: >
    Delivers the fast dev inner loop of the "Makefile for images" thesis: `crustyimg build
    --watch` re-runs the declared build whenever a source, recipe, or the manifest changes —
    debounced, resilient to editor atomic-writes — so an author sees outputs refresh without
    re-invoking the tool. It is the last stage of PROJ-007; the "verifiable" leg (build +
    cache + lockfile) already shipped, and this is the ergonomics leg on top of it.
  delivers:
    - "`crustyimg build --watch`: an initial build, then a debounced rebuild on every change to a watched source root / recipe / manifest, until interrupted"
    - "Only-affected rebuilds for free — the STAGE-021 content-addressed cache already skips unchanged inputs, so a watch cycle is incremental with no new invalidation logic"
    - "Loop resilience: a failing cycle (bad recipe, undecodable input) is reported and the loop keeps watching, not aborts"
    - "A recorded decision (DEC-060) fixing the file-watch dependency + debounce + watch-loop semantics (incl. not self-triggering on its own outputs)"
  explicitly_does_not:
    - "Compute an affected-target dependency graph — the cache makes a full re-run incremental, so `--watch` re-runs the whole build and lets the cache do the pruning"
    - "Run a dev HTTP server / live-preview page (Wave 3 WASM/demo territory), or hot-reload the binary"
    - "Rewrite the committed lockfile on every save — a watch cycle is pre-commit iteration, not a commit action (see Design Notes)"
    - "Introduce an async runtime — the file watcher runs on its own thread + channel (DEC-006 holds)"
---

# STAGE-023: `--watch` — the debounced rebuild inner loop

## What This Stage Is

The stage that closes the dev-ergonomics gap in `crustyimg build`. Today you re-run
`crustyimg build` by hand after every edit; this stage adds `--watch`: run an initial build,
then watch the manifest, each target's recipe, and each target's source roots, and on any
change — debounced so an editor's write-temp-then-rename burst is one rebuild — re-run the
build. The elegant part is what it *doesn't* need: STAGE-021's content-addressed cache
already makes a full re-run do only the work that changed, so `--watch` re-runs the **whole**
declared build each cycle and lets the cache prune it to the affected outputs. No
dependency-graph, no affected-target analysis — a thin, resilient loop over the shipped
incremental executor. It is the last stage of PROJ-007 and the one leg the design-feedback
review de-gated from 1.0-blocking, so it completes the wave without being on the critical path.

## Why Now

- **It's the only PROJ-007 stage left, and it sits cleanly on top of what shipped.** The
  build (STAGE-020), the cache (STAGE-021), and the lockfile (STAGE-022) are done; `--watch`
  is a loop around `run_build`, not new build machinery.
- **The cache turned the hard part into a non-problem.** "Rebuild only affected targets" —
  the brief's success signal — would normally need change→target dependency tracking. The
  content-addressed cache already delivers it: re-run everything, and unchanged inputs are
  cache hits. So the stage is small and low-risk.
- **It's the daily-driver ergonomics that make the rest pleasant to use.** A Makefile for
  images that you have to re-invoke by hand is half a tool; the watch loop is what makes the
  build/cache/lockfile machinery feel like a live build.

## Success Criteria

- `crustyimg build --watch` runs an initial build, then blocks watching; editing a **source**
  triggers a rebuild of only the affected output(s) (via the cache), reported per cycle.
- Editing a **recipe** or the **manifest** triggers a rebuild; a **new/deleted** source file
  in a watched glob root is picked up (globs are re-resolved each cycle).
- The loop is **debounced** — one save that fires a burst of filesystem events (incl. an
  editor's atomic temp-write + rename) produces exactly one rebuild.
- The loop **does not self-trigger**: writing the build's own outputs (each target's `out`),
  the `.crustyimg/` cache, or the lockfile must NOT cause another rebuild (no infinite loop).
- A **failing cycle** (bad recipe, undecodable input) is reported to stderr and the loop keeps
  watching; `Ctrl-C` exits cleanly.
- **No async runtime** (DEC-006 holds — the watcher is threads + a channel); the one new
  dependency (a file-watch crate) is recorded in **DEC-060**, permissive, `just deny` green
  with no new exception; the lean build is unaffected (or `--watch` is feature-gated — decide
  in DEC-060).

## Scope

### In scope
- **SPEC-067 — `crustyimg build --watch`**: a `--watch` flag on the build path; a watch module
  (`src/build/watch.rs` or in `cli`) that resolves the watch set (manifest + recipes + source
  roots), watches it, debounces events, and re-runs the shipped `run_build` per debounced
  change, excluding the build's own outputs/cache/lock from triggering; loop resilience +
  clean interrupt; the new file-watch dependency (**DEC-060**). **(SPEC-067)**

### Explicitly out of scope
- An affected-target dependency graph (the cache makes it unnecessary); a dev HTTP server /
  live-preview page or WASM demo (Wave 3); hot-reloading the binary; rewriting the committed
  lockfile on every cycle (Design Notes); `--watch` combined with `--frozen` as a hard gate
  (note `--watch --check` as a possible "verify-on-change" companion, don't build the full
  matrix); the pre-decode format sniff carry (DEC-059 follow-up, separate).

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] SPEC-067 (design) — `crustyimg build --watch`: debounced rebuild loop over the shipped incremental
  executor (`run_build` per cycle; the cache prunes to affected outputs — no dependency graph); watch
  manifest + recipes + source roots (recursive), **exclude own outputs/`.crustyimg/`/lockfile so it never
  self-triggers**; loop-resilient; Ctrl-C via default SIGINT (no `ctrlc` dep); pure `watch_roots`/`is_excluded`/
  `debounce` unit-tested + a thin `notify` loop. **One new dep (`notify`, threads+mpsc not async) → DEC-060.**
  Framed 2026-07-09.

**Count:** 0 shipped / 1 active / 0 pending — single-spec stage; SPEC-067 framed, build-ready.

## Design Notes

- **The cache is why this is small.** `--watch` re-runs `run_build` on each debounced change;
  STAGE-021's cache (DEC-058) skips unchanged inputs, so the full re-run is incremental and
  "only affected outputs rebuild" falls out for free. Do NOT build change→target tracking.
- **The dependency (→ DEC-060, license-probe at build).** Recommended: **`notify`** — the
  standard cross-platform watcher (inotify / FSEvents / ReadDirectoryChangesW). It runs on a
  background thread and delivers events over an `std::sync::mpsc` channel, so it does **not**
  introduce an async runtime (DEC-006 holds). It is a heavier dep than `sha2` (platform
  backends + `mio`/`kqueue`/`fsevent-sys`/`windows-sys`), so run `just deny` **immediately**
  after `cargo add` and confirm permissive licenses with **no new exception** (the recurring
  lesson) — `notify` is CC0-1.0/Artistic-2.0-ish but the transitive tail is the real check.
  Decide at build whether `--watch` is **feature-gated** (keeps the lean/headless build free of
  the watcher) or default-on; lean toward a default `watch` feature that the released binary
  ships but `--no-default-features` drops, mirroring `display` (DEC-027).
- **Debounce (→ DEC-060).** Editors fire bursts (multiple modify events; atomic
  temp-write+rename). Collect events and rebuild once after a short quiet window (~150–250 ms).
  Prefer a **hand-rolled** debounce (a `recv_timeout` loop that drains the channel until quiet)
  to keep the new-dep surface to `notify` alone; `notify-debouncer-mini` is the fallback if the
  hand-roll gets fiddly. Watch **directories recursively** at each source root (not individual
  files), so create/delete/rename all register, and **re-resolve globs each cycle** so new and
  deleted sources are picked up.
- **Do not self-trigger (correctness-critical).** A build writes into each target's `out` dir,
  the `.crustyimg/` cache, and (a plain build) the lockfile. If the watcher sees those writes it
  rebuilds forever. The watch set must **exclude** every target's `out` dir, `.crustyimg/`, and
  `crustyimg.build.lock` — either by not watching them, or by filtering events whose path is
  under an output/cache/lock location. Test this explicitly (a build must not wake itself).
- **Lockfile in watch mode (a deliberate call).** A watch cycle is **pre-commit iteration**, not
  a commit action, so `--watch` should **not** rewrite the committed `crustyimg.build.lock` on
  every save (that is constant git churn and, combined with the self-trigger rule, avoids the
  loop entirely). Watch builds outputs + uses the cache; the lock is refreshed by a normal
  `crustyimg build`. Note `--watch --check` as a plausible "verify against the committed lock on
  each change, report drift, keep watching" companion — record the decision in DEC-060; a hard
  `--frozen`+`--watch` gate is out of scope.
- **Loop resilience + exit.** The initial build's errors surface as normal (a missing manifest
  at start is a hard exit). Once watching, a failing cycle (bad recipe, partial-batch decode
  failure) is printed and the loop continues — a dev loop must survive a broken intermediate
  save. `Ctrl-C`/SIGINT ends the loop with a clean exit. Honor `--quiet` (suppress per-cycle
  summaries) and `--jobs` (the per-build fan-out) as `run_build` already does.

## Dependencies

### Depends on
- STAGE-020 (`run_build`, DEC-057), STAGE-021 (the cache that makes a re-run incremental,
  DEC-058), STAGE-022 (the lockfile — watch coexists with it per the Design Notes). DEC-006
  (no async; the watcher is threads+channel). The manifest/source/recipe resolution already shipped.

### Enables
- The daily-driver ergonomics for the whole build/cache/lockfile surface. Nothing downstream in
  PROJ-007 (it is the last stage); a future Wave-3 demo/dev-server could build on the watch loop.

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
