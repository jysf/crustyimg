---
# Maps to ContextCore task.* semantic conventions.
task:
  id: SPEC-067
  type: story
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: M                    # a debounced watch loop + one new dep + cross-platform + self-trigger exclusion; the logic is small, the dep/testability add weight

project:
  id: PROJ-007
  stage: STAGE-023
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-09

references:
  decisions: [DEC-057, DEC-058, DEC-006, DEC-027, DEC-004, DEC-060]
  constraints:
    - no-new-top-level-deps-without-decision
    - no-async-runtime
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - every-public-fn-tested
    - clippy-fmt-clean
    - ergonomic-defaults
  related_specs: [SPEC-063, SPEC-064, SPEC-066]

value_link: "STAGE-023's 'a debounced rebuild inner loop' — the dev ergonomics leg of the build."

cost:
  sessions:
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-07-09
      notes: >
        Framing/design cycle — main-loop, not separately metered → null-with-note per AGENTS §4.
        Grounded in a firsthand read of the shipped executor (`run_build(file, &GlobalArgs)`, the
        `Commands::Build` dispatch, `GlobalArgs` where `--check`/`--no-cache` live). Key finding:
        the STAGE-021 cache makes a full re-run incremental, so `--watch` is a thin loop over
        `run_build`, not a dependency graph. The file-watch dep (notify) + license probe is a build
        concern (DEC-060). Testability strategy (extract pure logic, thin driven loop) set here.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-067: `crustyimg build --watch`

## Context

PROJ-007's "verifiable" leg shipped: `crustyimg build` runs a declared build (STAGE-020),
does only changed work via a content-addressed cache (STAGE-021), and pins/verifies via a
lockfile (STAGE-022). This spec is the last stage — the **dev ergonomics** leg: `--watch`,
a debounced loop that re-runs the build whenever a source, recipe, or the manifest changes,
so an author sees outputs refresh without re-invoking the tool.

The design's elegance is what it doesn't need: the STAGE-021 cache already makes a full
re-run incremental, so `--watch` re-runs the **whole** build each cycle and lets the cache
prune it to the affected outputs — no change→target dependency graph. `--watch` is a thin,
resilient loop over the shipped `run_build`. See the parent `STAGE-023-watch.md`.

## Goal

Add `--watch` to the build path: run an initial build, then watch the manifest, each target's
recipe, and each target's source roots; on any change — debounced so an editor's write burst
is one rebuild, and excluding the build's own outputs / cache / lockfile so it never
self-triggers — re-run `run_build`. A failing cycle is reported and the loop continues;
`Ctrl-C` exits. One new (permissive, non-async) file-watch dependency, recorded in DEC-060.

## Inputs

- **Files to read:**
  - `src/cli/mod.rs` — `run_build` (~L1558, the "build once" op the loop wraps — call it
    verbatim per cycle), the `Commands::Build` dispatch (~L703), `GlobalArgs` (~L58, add
    `--watch`; `--no-cache`/`--check`/`--quiet`/`--jobs` already live here), `load_manifest`,
    `prepare_target` (how targets resolve — the watch set is derived from the same `Target`s).
  - `src/build/mod.rs` — `BuildManifest`/`Target` (`source: SourceSpec`, `recipe`, `out`); the
    watch set = each target's source roots + recipe + the manifest; the exclusion set = each
    target's `out`, `.crustyimg/` (`cache::DEFAULT_CACHE_DIR`), and `lock::DEFAULT_LOCK_FILE`.
  - `src/source/mod.rs` — glob/dir/path resolution (a source root is the directory a glob/dir/path
    lives under, watched recursively; globs are re-resolved by `run_build` each cycle).
  - `Cargo.toml` — add the one file-watch dep (DEC-060); decide the `watch` feature gate.
  - `decisions/DEC-006` (no async — the watcher is threads+channel), `DEC-027` (the `display`
    default-feature precedent to mirror for `watch`), `DEC-057`/`DEC-058` (the executor + cache).
- **External APIs:** the file-watch crate (recommended `notify`) — a background thread delivering
  events over `std::sync::mpsc`. **New dependency → DEC-060 + a `just deny` license check.**
- **Related code paths:** `tests/build_cache.rs` for the temp-project harness shape (fixtures,
  driving the binary as a child process).

## Outputs

- **Files created:**
  - `src/build/watch.rs` — the **pure, unit-tested** logic (no real watcher, no blocking):
    - `pub fn watch_roots(manifest: &BuildManifest, manifest_path: &Path) -> WatchSet` →
      `WatchSet { roots: Vec<PathBuf>, excluded: Vec<PathBuf> }`: the source-root dirs + recipes +
      manifest to watch, and the `out` dirs + cache dir + lockfile to exclude.
    - `pub fn is_excluded(path: &Path, excluded: &[PathBuf]) -> bool` — the event filter (an event
      under any excluded prefix is dropped, so a build never wakes itself).
    - `pub fn debounce(...)` — coalesce a burst into one rebuild signal over a quiet window,
      written as a function over an event source + a timeout so it is testable with a synthetic
      channel (no wall-clock flakiness in the unit test).
    - a typed `WatchError` if any setup can fail.
  - `tests/build_watch.rs` — a few timing-tolerant integration tests driving `build --watch` as a
    child process (see Failing Tests); may be `#[ignore]`-gated if CI proves flaky.
  - `decisions/DEC-060-*.md` — the file-watch dep + debounce + watch-loop semantics + the
    self-trigger exclusion + the feature-gate decision (emitted at build).
- **Files modified:**
  - `src/build/mod.rs` — `pub mod watch;` (feature-gated with `watch` if chosen).
  - `src/cli/mod.rs` — `--watch` on `GlobalArgs`; the `Commands::Build` path runs the watch loop
    when `global.watch` (initial `run_build`, then watch `WatchSet.roots`, debounce, filter via
    `is_excluded`, re-run `run_build` per debounced change, catch+print per-cycle errors, loop).
  - `Cargo.toml` — the one file-watch dep + optional `watch` feature.
- **New exports:** `crustyimg::build::watch::{watch_roots, is_excluded, WatchSet, WatchError}`.

## Acceptance Criteria

- [ ] `crustyimg build --watch` runs an initial build, then blocks watching; editing a source
  file triggers a rebuild in which only the affected output changes (the cache prunes the rest),
  reported per cycle.
- [ ] Editing a **recipe** or the **manifest**, or adding/removing a source under a watched glob
  root, triggers a rebuild (globs re-resolved each cycle by `run_build`).
- [ ] A single save that fires a **burst** of filesystem events (incl. an editor's atomic
  temp-write + rename) produces exactly **one** rebuild (debounced).
- [ ] The loop **does not self-trigger**: after the initial build settles, the build's own writes
  to `out` / `.crustyimg/` / the lockfile do NOT cause another rebuild (no infinite loop). `is_excluded`
  compares **normalized** paths so an absolute/canonical event path from the watcher still matches a
  manifest-relative excluded entry (see Implementation Context — this is the load-bearing correctness detail).
- [ ] In watch mode, a build cycle **does not rewrite the committed lockfile** (a dev loop is pre-commit
  iteration; `run_build` suppresses the lock write under `--watch`). `--watch` combined with a verify mode
  (`--check`/`--frozen`/`--locked`) is a **usage error** (exit 2) with a clear message — the two are
  incompatible (write-mode loop vs one-shot assert); do not leak check-mode exit codes into the loop.
- [ ] **Startup vs cycle failure:** an unparseable/missing manifest at start is a hard exit (there is
  nothing to derive a watch set from), but a *build* failure (bad recipe, undecodable input) with a valid
  manifest **still enters the watch loop** — it prints the error and waits, so the user can fix it and see
  it recover. (Consistent with the failing-cycle rule below; the first cycle is not special.)
- [ ] A **failing cycle** (a mid-watch broken recipe or undecodable input) is printed to stderr
  and the loop keeps watching; a subsequent good change still rebuilds. `Ctrl-C` exits.
- [ ] **No async runtime** (DEC-006 holds); **one** new dependency, recorded in **DEC-060**,
  permissive with `just deny` green and **no new exception** — if the watcher's license (or a transitive
  dep's, e.g. Artistic-2.0) is not on the deny allowlist, that is a real decision point in DEC-060 (justify
  an exception, or pick another watcher), not a silent addition; if `--watch` is feature-gated, the default
  binary includes it and `--no-default-features` drops it (mirroring `display`, DEC-027).
- [ ] `watch_roots` / `is_excluded` / `debounce` are pure, unit-tested; `cargo clippy --all-targets
  -- -D warnings` + `cargo fmt --check` clean; no `unwrap` on recoverable paths.

## Failing Tests

Written during **design**, BEFORE build. Build makes these pass.

- **`src/build/watch.rs`** (`#[cfg(test)] mod tests`)
  - `"watch_roots_covers_sources_recipes_manifest"` — a 2-target manifest → `roots` contains each
    target's source-root dir + recipe + the manifest path.
  - `"watch_roots_excludes_outputs_cache_and_lock"` — `excluded` contains each target's `out` dir,
    `.crustyimg/`, and `crustyimg.build.lock` (the anti-self-trigger set).
  - `"is_excluded_drops_output_and_cache_events_keeps_source"` — a path under an `out` dir /
    `.crustyimg/` / the lockfile → excluded; a path under a source root → not excluded.
  - `"is_excluded_matches_across_absolute_and_relative_paths"` — the excluded set holds a
    manifest-relative `dist/` while the event arrives as an **absolute** `.../dist/a.png` (as `notify`
    reports): still excluded. The normalization is the whole point — a prefix check on raw strings
    would miss it and the build would self-trigger.
  - `"debounce_coalesces_a_burst_into_one_signal"` — feed a synthetic burst of N events within the
    window into `debounce` (with an injected short timeout) → exactly one rebuild signal; a later
    event after the quiet window → a second signal.
- **`tests/build_watch.rs`** (integration; spawn the binary as a child, generous timeouts; may be
  `#[ignore]`-gated if CI flakes)
  - `"watch_rebuilds_on_source_change"` — spawn `build --watch` in a temp project; wait for the
    initial outputs; modify one source; poll (timeout) until that output's bytes change; kill the
    child. The other output is unchanged (cache).
  - `"watch_does_not_self_trigger"` — spawn, wait for the initial build to settle, then observe a
    quiet period: the outputs are NOT rebuilt again on their own (the build's writes didn't wake it).
  - `"watch_survives_a_failing_cycle"` — mid-watch, write a broken recipe → the loop reports an
    error and stays alive; then a valid edit → a normal rebuild.
  - `"watch_does_not_write_the_lockfile"` — after a watch cycle rebuilds an output, the committed
    `crustyimg.build.lock` is unchanged (absent if none was committed; byte-identical if one was) —
    a watch cycle suppresses the lock write.
  - `"watch_rejects_verify_modes"` — `build --watch --check` (and `--frozen`) exit **2** immediately
    with a usage error, before watching begins. (Non-blocking to test; no child to kill.)
  - `"watch_starts_despite_a_broken_initial_build"` — a project whose recipe is broken at start:
    `build --watch` does **not** hard-exit; it prints the error and enters the loop (then a fixed recipe
    rebuilds). A *missing manifest*, by contrast, is a hard exit (nothing to watch).

## Implementation Context

*Read this section (and the files it points to) before starting build. The seam was read
firsthand during design against the current post-lockfile tree — re-confirm signatures.*

### Decisions that apply
- `DEC-058` — the cache is why `--watch` needs no affected-target analysis: re-run `run_build`
  and the cache skips unchanged inputs. Do NOT reimplement invalidation.
- `DEC-057` — `run_build(file, &GlobalArgs)` is the "build once" op; call it (almost) verbatim per
  cycle. Manifest/source/recipe paths are cwd-relative; the watch set derives from the same `Target`s.
  **One deliberate change:** `run_build` must **suppress the lockfile write when `global.watch`** — a
  dev-loop cycle is not a commit, and a user iterating doesn't want `crustyimg.build.lock` silently
  modified in their working tree mid-edit. This is a one-line branch in `run_build`'s existing
  global-flag plumbing (the same place `--check`/`--no-cache` already branch), so "call `run_build`"
  stays true and "don't rewrite the lock" is enforced inside it.
- `DEC-006` — **no async runtime.** `notify` runs on its own thread and delivers events over an
  `std::sync::mpsc` channel; the main thread runs the debounce/rebuild loop. Confirm the chosen
  crate needs no tokio.
- `DEC-027` — the `display`/viuer precedent: a default-on feature the released binary ships and
  `--no-default-features` drops. Mirror it for `watch` if the dep should be gated (recommended).
- **`DEC-060` (NEW — emit at build)** — the file-watch dep (`notify` recommended; permissive,
  `just deny` green with no new exception — run deny right after `cargo add`, the transitive tail
  is the real check), the debounce (hand-rolled `recv_timeout` window, ~150–250 ms; fallback
  `notify-debouncer-mini`), the watch-loop semantics (below), the self-trigger exclusion **and its
  path normalization**, the lockfile suppression under `--watch`, the rejection of `--watch` + verify
  modes (a `--watch --check` verify-on-change mode is possible *future* scope, explicitly not built
  here), and the feature-gate.

### The watch loop (thin, over the shipped executor)
```
if global.watch {
    if global.check || global.frozen || global.locked {      // write-loop vs one-shot assert
        return Err(CliError::Usage("--watch cannot be combined with --check/--frozen".into())); // exit 2
    }
    let manifest = load_manifest(file)?;      // HARD exit if unparseable/missing — no watch set without it
    let set = watch::watch_roots(&manifest, manifest_path);
    // notify::RecommendedWatcher over set.roots (recursive); events → mpsc channel
    // Initial build: run it, but a build FAILURE enters the loop anyway (print, don't exit) —
    // the first cycle is not special. run_build suppresses the lock write because global.watch.
    if let Err(e) = run_build(file, global) { eprintln!("build error: {e}"); }
    loop {
        let batch = watch::debounce(&rx, DEBOUNCE);           // block until a quiet burst
        // is_excluded NORMALIZES both sides: notify reports absolute/canonical paths, the excluded
        // set is manifest-relative — a raw string prefix check would miss them and self-trigger.
        if batch.iter().all(|p| watch::is_excluded(p, &set.excluded)) { continue; } // own writes
        if let Err(e) = run_build(file, global) { eprintln!("build error: {e}"); }  // stay alive
    }
}
```
**`is_excluded` is the correctness crux and the SPEC-066 lesson repeated:** the watcher's event paths
and the manifest-derived excluded set come from different sources and won't string-match. Normalize
both (make absolute against cwd + lexically clean, or canonicalize the existing roots once and compare
against the event's canonical form) so the exclusion actually fires. Write this + its test first.
Rely on default SIGINT for `Ctrl-C` (exit 130) — do NOT add a `ctrlc` dependency (that would be a
second new dep for a graceful message the OS already delivers). Re-read the manifest each cycle so
a manifest edit is honored (and re-derive `watch_roots`; re-registering the watcher on a manifest
change is acceptable).

### Constraints that apply
- `no-new-top-level-deps-without-decision` (one dep, DEC-060), `no-async-runtime` (threads+channel),
  `untrusted-input-hardening` (watch only the declared source roots + recipes + manifest, all under
  the user's tree; the exclusion set prevents watching/echoing the output tree), `no-unwrap-on-recoverable-paths`,
  `every-public-fn-tested` (the pure `watch_roots`/`is_excluded`/`debounce`), `clippy-fmt-clean`,
  `ergonomic-defaults` (one `--watch` flag; sensible debounce default; no required config).

### Testability strategy (call this out — file watching is timing-sensitive)
Put ALL logic in pure functions (`watch_roots`, `is_excluded`, `debounce` over an injected event
source + timeout) and unit-test those deterministically. Keep the blocking `notify` loop itself as
thin as possible — it is the one part not unit-tested. The integration tests drive the real binary
as a child process with generous timeouts and a poll-until-changed loop; if CI proves flaky, gate
them `#[ignore]` (documented) rather than weakening the unit coverage.

### Out of scope (for this spec specifically)
- An affected-target dependency graph; a dev HTTP server / live-preview / WASM demo (Wave 3);
  hot-reloading the binary; a `ctrlc` graceful-shutdown dep; rewriting the lockfile per cycle; the
  full `--watch --frozen` hard-gate matrix (note `--watch --check` only); the DEC-059 pre-decode
  format-sniff carry.

## Notes for the Implementer

- Run `just deny` **immediately** after `cargo add notify` — a watcher pulls platform backends and
  `windows-sys`/`mio`/`kqueue`/`fsevent-sys`; confirm permissive + no new deny exception before
  writing code. Confirm no async runtime is dragged in.
- Keep `watch_roots`/`is_excluded`/`debounce` pure and in `src/build/watch.rs`; keep the `notify`
  wiring + the loop in `cli`. Re-use `run_build` verbatim — do not fork the executor.
- The self-trigger exclusion is the correctness crux: write `is_excluded` + its test first, and add
  the `watch_does_not_self_trigger` integration test so an infinite rebuild loop can't ship.
- If `--watch` is feature-gated, ensure the flag still parses under `--no-default-features` and
  gives a clear "built without watch support" error (mirror the HEIC `CodecNotBuilt`→exit 4 shape),
  or gate the flag itself — decide in DEC-060 and keep the lean build clean.
- Emit `DEC-060` with `affected_scope` covering `src/build/watch.rs`, `src/cli/mod.rs`, `Cargo.toml`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-060` — file-watch dep + debounce + watch-loop semantics + self-trigger exclusion + feature gate
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

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
