---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-060
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-007
repo:
  id: crustyimg

created_at: 2026-07-10
supersedes: null
superseded_by: null

affected_scope:
  - src/build/watch.rs
  - src/cli/mod.rs
  - Cargo.toml
  - deny.toml

tags:
  - build
  - watch
  - dependency
  - debounce
  - hardening
  - feature-gate
---

# DEC-060: `build --watch` — the file-watch dep, debounce, self-trigger exclusion, and loop semantics

## Decision

`crustyimg build --watch` runs an initial build, then re-runs the **whole** shipped
`run_build` on any debounced change to a source, recipe, or the manifest — letting the
STAGE-021 cache (DEC-058) prune each re-run to the affected outputs. There is **no**
change→target dependency graph; `--watch` is a thin, resilient loop over the executor.

The specifics that make it correct and permissive:

- **Dependency:** one new crate, **`notify` 8.2.0** (recommended in the spec), behind a
  new **default-on `watch`** cargo feature (mirroring `display`, DEC-027). It runs an OS
  watcher on its **own thread** and delivers events over `std::sync::mpsc` — threads +
  a channel, **not** an async runtime, so **DEC-006 holds** (`cargo tree -i tokio` /
  `async-std` both empty; `mio` is a synchronous epoll/kqueue wrapper used by notify's
  thread, not tokio). Default features on = the per-OS backend (inotify / FSEvents /
  ReadDirectoryChangesW); the `serde`/`crossbeam` extras stay off.
- **License:** `just deny` (full graph, `all-features = true`) stays green via **three
  new per-crate exceptions** — `notify` (**CC0-1.0**, a public-domain dedication, same
  tier as the excepted `to_method` and the globally-allowed `0BSD`/`Unlicense`), and
  `inotify` + `inotify-sys` (**ISC**, notify's Linux backend; ISC is an OSI/FSF-approved
  permissive license functionally equivalent to the already-allowed `BSD-2-Clause`).
  None is copyleft; none touches the `MIT OR Apache-2.0` binary. Excepted per-crate
  (not widened into the global `allow` list) so a future unexpected ISC/CC0 crate still
  triggers a fresh review — the same discipline as the `ansi_colours`/`libfuzzer-sys`/
  `avif-parse`/`to_method` exceptions.
- **Debounce:** a hand-rolled `recv_timeout` quiet-window (**200 ms**), no
  `notify-debouncer-mini`. `debounce(rx, window)` blocks for the first event, then
  drains the channel until a quiet `window` elapses, returning the batch — coalescing an
  editor's atomic save burst (temp-write + rename + metadata touches) into one rebuild.
- **Self-trigger exclusion + path normalization (the correctness crux):** the watcher
  reports **absolute, OS-canonical** event paths; the manifest-derived excluded set
  (each target's `out` dir, the `.crustyimg` metadata dir, the lockfile) is
  **manifest-relative**. `is_excluded` **normalizes both sides** — absolutize a relative
  path against the CWD, clean it lexically (no `canonicalize`, which would fail on a
  just-deleted path), and compare by path **components** (`Path::starts_with`, so `dist`
  never matches `distortion`). Without this the build's own writes to `out`/`.crustyimg`
  wake it forever. Its unit test was written first, and a `watch_does_not_self_trigger`
  integration test guards against regression.
- **Lockfile suppression under `--watch`:** a `--watch` build cycle **does not rewrite**
  the committed `crustyimg.build.lock` — a dev loop is pre-commit iteration, not a
  commit. One-line branch inside `run_build`'s existing lock-write step (`None if
  global.watch => {}`).
- **`--watch` + a verify mode is a usage error (exit 2):** `--watch --check` (and its
  `--frozen`/`--locked` aliases, which are the *same* clap field) is rejected before any
  watching begins, with a clear message — a write-mode loop and a one-shot assertion are
  incompatible. A verify-on-change `--watch --check` mode is possible *future* scope,
  explicitly not built here.
- **Startup vs cycle failure:** a missing/unparseable manifest at start is a **hard
  exit** (nothing to derive a watch set from); a **build** failure with a valid manifest
  (bad recipe, undecodable input) **enters the loop** — print the error and wait, so the
  user can fix it and watch it recover. The first cycle is not special.
- **Ctrl-C** exits via the default **SIGINT** (exit 130) — **no `ctrlc` dependency**
  (that would be a second new dep for a message the OS already delivers).
- **Feature gate:** `watch` is default-on, so the released binary ships `--watch`.
  `--no-default-features` drops `notify` and its backend; the `--watch` flag still parses
  and returns a clear *"built without watch support"* usage error (exit 2), not a panic
  or a parse failure (the `display`/DEC-027 shape). The pure logic in
  `src/build/watch.rs` compiles **unconditionally** (it uses no `notify` type), so its
  unit tests run even in the lean build.

## Context

STAGE-023 is the last stage of PROJ-007 and the **dev-ergonomics** leg of the build:
`build` is declared (STAGE-020), incremental (STAGE-021), and verifiable (STAGE-022);
`--watch` closes the edit→see-output loop. The design's elegance is what it doesn't
need — because the cache already makes a full re-run incremental, `--watch` re-runs the
whole build and lets the cache do the pruning, so there is no invalidation logic to
reimplement and no dependency graph to keep correct.

The load-bearing risk is a **self-trigger loop**: watch the tree, the build writes to
that same tree, the writes wake the watcher, forever. The SPEC-065/066 lesson — green
exit-code tests miss whole defect classes (hostile serialized input, cross-platform path
handling) — applies directly: a naive string-prefix exclusion passes a happy-path test
and self-triggers in production because the watcher's absolute paths never string-match
the manifest's relative excluded set. So the normalization is the whole point, and it is
tested first and driven end-to-end against the real binary.

## Alternatives Considered

### The debounce: hand-rolled `recv_timeout` vs `notify-debouncer-mini`

`notify-debouncer-full`/`-mini` exist and handle rename-coalescing well, but they are an
**additional** dependency (and `-full` pulls `file-id` + more), for a quiet-window that
is ~10 lines of `recv_timeout` over the channel we already own. Re-running the whole
build each cycle means we don't need per-path event *semantics* (created/modified/
removed) at all — only "something under a watched, non-excluded root changed" — so the
simplest debounce that coalesces a burst is sufficient. Chosen: hand-rolled. If real
editors on some platform defeat the 200 ms window, `notify-debouncer-mini` is the
documented fallback.

### License: per-crate exceptions vs widening the global `allow` list

ISC and CC0-1.0 are both squarely permissive, and one could argue ISC belongs in the
global `allow` list beside `BSD-2-Clause`. But the repo's revealed discipline (NCSA and
CC0 both got per-crate exceptions rather than global widening, "keeping the surface
minimal") is to spotlight every non-`allow` license at the crate that introduces it.
Followed here: three scoped exceptions, global `allow` unchanged, so a *future*
unexpected ISC/CC0 dependency still fails `deny` and forces a decision. This was a real
decision point (deny failed on first `cargo add`), not a silent addition.

### Watch roots: lexical derivation vs a filesystem probe

`watch_roots` derives each source's watch **directory** purely lexically (a glob's
literal prefix dir; a trailing-slash dir; else a bare path's parent), touching no disk,
so it is deterministic in a unit test. Every root is a **directory** watched — never a
bare file — because a single-file watch misses an editor's atomic save when the original
inode is replaced by a rename. Over-watching is the safe direction (the cache turns a
redundant rebuild into a no-op); under-watching would silently miss a real edit. A bare
slash-less directory source (e.g. `photos`, indistinguishable lexically from a file)
watches its parent — broader, but rare, since directory sources are spelled with a
trailing slash or as globs.

### Two watch tiers: recursive source roots vs shallow manifest/recipe dirs

The manifest and recipes live **beside** the build's own `dist/` and `.crustyimg/` cache
trees at the project root. A first, obvious implementation watched *every* root
recursively — including the manifest's `.` — which meant the watcher recursively covered
`.crustyimg/cache/**`. That shipped green on macOS but **failed on Linux CI**: on a
*fresh* build, the burst of cache writes right after the watcher registers floods inotify
and degrades detection of the source watches, so a subsequent source edit is never seen
(reproduced: 22 re-issued edits over 45 s, still no rebuild — a real detection failure,
not a dropped-event flake; the discriminator was that a *pre-built* project, whose cache
already existed, detected edits fine). So `WatchSet` has **two tiers**: source roots
watched **recursively** (they contain only inputs), and the manifest/recipe directories
watched **non-recursively** (we only care about those files; a shallow watch never covers
the deep cache/output churn). A dir that is already a recursive root is de-duplicated out
of the shallow set (a recursive watch is a superset, and registering the same inode twice
can share/clobber a Linux inotify watch descriptor). This is the load-bearing
cross-platform lesson of the wave, caught by the 3-OS CI exactly as intended.

### `is_excluded`: lexical-absolutize vs `canonicalize`

Comparing canonical forms on both sides would also work, but `canonicalize` fails on a
path the build just deleted and resolves symlinks the watcher may not have applied.
`current_dir()` is already symlink-resolved on Unix, so absolutize-then-lexical-clean
aligns with the watcher's absolute event paths without any of `canonicalize`'s failure
modes. Verified on macOS (FSEvents, `/private/var/…` temp dirs) via the integration
suite; the 3-OS CI covers Linux/Windows.

## Consequences

- `crustyimg build --watch` is a first-class dev loop: edit a source → exactly one
  rebuild of only the affected output (`(N cached, 1 rebuilt)`), the rest cache-pruned.
- One new default dependency (`notify`) with three documented, scoped license
  exceptions; `deny` green, DEC-006 intact.
- The lean `--no-default-features` build stays watcher-free and gives a clear error.
- **Follow-ups (out of scope here):** a verify-on-change `--watch --check` mode; a dev
  HTTP/live-preview server (Wave 3); hot-reloading the binary; if the 200 ms window
  proves too tight on some editor/OS, adopt `notify-debouncer-mini`.
