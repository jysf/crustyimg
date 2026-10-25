---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-132
  type: bug                        # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-050
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
  implementer: claude-sonnet-5-5
  created_at: 2026-10-24

references:
  decisions:
    - DEC-060
    - DEC-061
    - DEC-100
    - DEC-101
    - DEC-057
    - DEC-058
  constraints:
    - test-before-implementation
    - one-spec-per-pr
    - no-unwrap-on-recoverable-paths
  related_specs:
    - SPEC-067
    - SPEC-128
    - SPEC-129
    - SPEC-131

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-050's <capability>". Optional; null is acceptable.
value_link: >
  STAGE-050's outcome is one `build` that watermarks a whole site from a manifest; `build --watch`
  is how a designer iterates on that watermark, and today it never sees the watermark file change.

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
    - cycle: frame
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 8105124
      tokens_breakdown:
        input: 112
        output: 21927
        cache_creation: 153676
        cache_read: 7929409
      estimated_usd: 5.47
      duration_minutes: null
      recorded_at: 2026-10-24
      note: >
        Interactive frame session in its own worktree (docs/spec-132-frame). Measured from the
        session's transcript (896a1ffd-….jsonl, identified by content), deduped by `.message.id`
        (56 unique ids), input/cache_creation/cache_read once per id, MAX output_tokens per id.
        Priced by component at Opus $5/$25 per MTok, cache_creation x1.25 ($6.25), cache_read
        x0.10 ($0.50). Measured before the commit/push turns, so it slightly under-counts. Ran
        long in wall-clock because the shell auto-mode classifier returned no verdict on ~15
        calls and ended one turn; the release build and three watch repro runs are included.
    - cycle: design
      agent: claude-opus-5-5
      interface: claude-code
      tokens_total: 4249300
      tokens_breakdown:
        input: 68
        output: 40035
        cache_creation: 132689
        cache_read: 4076508
      estimated_usd: 3.87
      duration_minutes: null
      recorded_at: 2026-10-25
      note: >
        Interactive design session in its own worktree (docs/spec-132-design). Measured from the
        session's transcript (9b7be2b7-….jsonl, identified by content), deduped by `.message.id`
        (34 unique ids), input/cache_creation/cache_read once per id, MAX output_tokens per id.
        Priced by component at Opus $5/$25 per MTok, cache_creation x1.25 ($6.25), cache_read
        x0.10 ($0.50). Includes a release build of `main` and 6 driven watch rows. Measured
        before the commit/push turns, so it slightly under-counts.
    - cycle: build
      agent: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 4205527
      tokens_breakdown:
        input: 70
        output: 22707
        cache_creation: 126615
        cache_read: 4056135
      estimated_usd: 2.03
      duration_minutes: null
      recorded_at: 2026-10-25
      note: >
        Build session in its own worktree (feat/spec-132-watch-recipe-assets). Measured from the
        session's transcript (4c7af3cf-….jsonl, identified by content), deduped by `.message.id`
        (32 unique ids), input/cache_creation/cache_read once per id, MAX output_tokens per id.
        Priced by component at Sonnet $3/$15 per MTok, cache_creation x1.25 ($3.75), cache_read
        x0.10 ($0.30). Measured before the final commit/PR turns, so it slightly under-counts.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-132: build --watch watches recipe asset paths

## Context

STAGE-050 backlog item "[S] ⚡ **`build --watch` never watches a recipe's asset paths**", found by
SPEC-129's retroactive verify (2026-09-07). PROJ-011.

A recipe can now name a file: a watermark step's `image` (overlay) or `font` key (SPEC-128). A
designer iterating on the logo runs `crustyimg build --watch`, edits `brand/logo.png`, and
**nothing happens**: no rebuild, no stderr line, the old watermark stays in `dist/`. Only a later
edit to a source, recipe or the manifest starts a build. SPEC-129 already makes that build
correct (the cache key hashes the asset's bytes), so the output is right **once a build runs**.
The watch loop just never starts one.

What the user writes:

```toml
# crustyimg.build.toml
version = 1
[[target]]
source = "src/*.png"
recipe = "recipes/r.toml"
out    = "dist"

# recipes/r.toml
version = "1"
[[step]]
op    = "watermark"
image = "brand/logo.png"   # or: text = "...", font = "brand/Go.ttf"
```

What they see today after three edits to `brand/logo.png`: stderr shows the initial
`built 1 target, 1 output (0 cached, 1 rebuilt)` and nothing more. The process keeps running and
nothing is reported.

## Goal

Under `build --watch`, editing (or creating) a file that a recipe step's `image` or `font` key
names starts a rebuild, the same as editing a source file does. That includes an asset path the
recipe was changed to point at mid-session, and it holds wherever the asset sits.

## Frame

**Verdict: GO. Size S** (upper end of S, see sizing).

### Premise check: confirmed, on a release build of `main`

Release build of `main` at `8471fb1` (`cargo build --release`). Driven with
`repro.sh <bin> <repo> image|font <workdir>` (reproduced in full below), which builds the layout above in a temp
dir, runs `crustyimg build --watch` with stderr going to a log **outside** the project, and counts
`^built ` summary lines. The asset is in `brand/`, which is **outside every watched path**: `.` is
shallow (the manifest's dir), `recipes/` is shallow, and `src/` is recursive. Each asset edit is
re-issued 3× over ~6 s, the same nudge pattern `tests/build_watch.rs` uses.

| row | layout | action | `built` lines (cumulative) |
|---|---|---|---:|
| initial | `image = "brand/logo.png"` | start watch | 1 |
| **negative control** | same | idle 3 s | 1 (no self-trigger) |
| **scenario: image** | same | swap `brand/logo.png` bytes ×3 | **1, no rebuild** |
| positive control | same | edit `src/a.png` | 2 |
| initial | `text` + `font = "brand/Go.ttf"` | start watch | 1 |
| negative control | same | idle 3 s | 1 |
| **scenario: font** | same | rewrite + touch `brand/Go.ttf` ×3 | **1, no rebuild** |
| positive control | same | edit `src/a.png` | 2 |
| **fake-pass trap** | `image = "recipes/logo.png"` (beside the recipe) | swap bytes ×3 | **4: caught by accident** |

So `image` and `font` behave the same: neither is ever watched. The trap row shows why the
failing tests must put the asset **outside** every currently-watched directory. An asset beside
its recipe (or under a source root, or at the manifest's own level) is already caught.

The trap row is the same script with `brand/logo.png` → `recipes/logo.png` (`sed` over the script).

<details><summary><code>repro.sh</code>: re-drive it on the build branch; the scenario rows must flip to 2+</summary>

```sh
#!/bin/sh
# Reproduce: `build --watch` never rebuilds when a recipe's watermark asset changes.
# Usage: repro.sh <bin> <repo> <mode: image|font> <workdir>
set -u
BIN=$1; REPO=$2; MODE=$3; W=$4
rm -rf "$W"; mkdir -p "$W/proj/src" "$W/proj/recipes" "$W/proj/brand" "$W/log"
P=$W/proj
cp "$REPO/tests/fixtures/classify/color_photo_fuji.png" "$P/src/a.png"
cp "$REPO/tests/fixtures/classify/dithered_graphic.png" "$P/brand/logo.png"
cp "$REPO/assets/fonts/Go-Regular.ttf" "$P/brand/Go.ttf"
if [ "$MODE" = image ]; then
  printf 'version = "1"\n\n[[step]]\nop = "watermark"\nimage = "brand/logo.png"\n' > "$P/recipes/r.toml"
else
  printf 'version = "1"\n\n[[step]]\nop = "watermark"\ntext = "(c) x"\nfont = "brand/Go.ttf"\n' > "$P/recipes/r.toml"
fi
printf 'version = 1\n\n[[target]]\nsource = "src/*.png"\nrecipe = "recipes/r.toml"\nout = "dist"\n' > "$P/crustyimg.build.toml"

cd "$P" || exit 1
"$BIN" build --watch 2> "$W/log/err.log" > /dev/null &
PID=$!
n() { grep -c '^built ' "$W/log/err.log"; }
i=0; while [ "$(n)" -lt 1 ] && [ $i -lt 100 ]; do sleep 0.2; i=$((i+1)); done
sleep 2
echo "[$MODE] after initial build:            builds=$(n)"
sleep 3
echo "[$MODE] negative control (idle 3s):     builds=$(n)"

# The scenario: edit the asset, re-issued 3x over ~6s (as the test suite's nudge does).
for k in 1 2 3; do
  if [ "$MODE" = image ]; then
    if [ $((k % 2)) = 1 ]; then cp "$REPO/tests/fixtures/classify/dither_32color.png" brand/logo.png
    else cp "$REPO/tests/fixtures/classify/dithered_graphic.png" brand/logo.png; fi
  else
    cp "$REPO/assets/fonts/Go-Regular.ttf" brand/Go.ttf; touch brand/Go.ttf
  fi
  sleep 2
done
echo "[$MODE] after editing brand/ asset x3:  builds=$(n)"

# Positive control: edit a source file.
cp "$REPO/tests/fixtures/classify/photo_entropy_floor.png" src/a.png
i=0; B=$(n); while [ "$(n)" -le "$B" ] && [ $i -lt 50 ]; do sleep 0.2; i=$((i+1)); done
sleep 1
echo "[$MODE] positive control (edit src/a.png): builds=$(n)"

kill $PID 2>/dev/null; wait $PID 2>/dev/null
echo "--- watch stderr ---"; cat "$W/log/err.log"
```

</details>

**Code, cited:**
- `src/build/watch.rs:98-131` (`watch_roots`) derives roots from the manifest's dir, each
  `target.source` and each `target.recipe`. It never opens a recipe, so it cannot know an asset
  path. It is deliberately filesystem-free (module docs, `watch.rs:1-23`).
- `src/cli/build.rs:1028-1037` (`watch_impl`) recomputes the watch set **only when the manifest
  changed**. A recipe edit that re-points `image` at a new file rebuilds once, because the recipe
  dir is watched, but never adds the new asset's dir. So an "at startup only" fix would still miss
  edits after a re-point. **That makes `src/cli/build.rs` mandatory, not optional.**
- Asset paths are read relative to the **process working directory**:
  `std::fs::read(&path)` in `resolve_recipe_assets` (`src/cli/common.rs:214-216`). That is the
  same base as manifest paths (DEC-057) and the same base `is_excluded` absolutizes against
  (`normalize_abs`, `watch.rs:233`). So `containing_dir(asset)` drops straight into the existing machinery.
- The asset-key list is one query: `registry.asset_keys(op)` (`src/operation/registry.rs:144`),
  `["image", "font"]` for `watermark` (`registry.rs:94-98`).

### Sizing: S, and why not M

The code delta is small (~40–80 lines). The verification matrix is narrow. The work:
1. A way to give `WatchSet` each recipe's asset directories while `watch_roots` stays pure: an
   extra input, or a separate pure helper. The CLI side loads the recipes.
2. Widen `watch_impl`'s recompute trigger from "manifest changed" to "manifest or a recipe
   changed", or simply recompute after every non-excluded batch and re-register only when the set
   differs. The re-create/re-register path already exists (`build.rs:1032-1036`).

Verification: {`image`, `font`} × {asset present at start, recipe re-pointed mid-session}, plus an
idle negative control and a pure unit test of the computed watch set. That is ~4 driven rows plus
unit tests, not an L-shaped matrix. It is one verb (`build --watch`) and one feature-gated path.
It is not M because no new mechanism is needed. Both halves extend code that already exists for
the manifest.

### Can it stay inside `src/build/watch.rs` + tests? **No.**

It **must** touch `src/cli/build.rs` (`watch_impl`): the recompute trigger lives there, and so
does the only place that can load a recipe. `src/cli/common.rs` is **probably avoidable**: the
asset walk is `registry.asset_keys(&step.op)` + `step.params.get_str(key)`, a few lines that can
sit in `build.rs`. But if design extracts a shared "asset paths of a recipe" helper for
`resolve_recipe_assets` to reuse (a reasonable call, so "which keys are assets" has one walk),
it touches `common.rs`.

**Parallelism:** SPEC-131's frame lists no `src/cli/build.rs` or `src/cli/common.rs` edits
(`src/operation/**`, `src/recipe/**`), so there is **no file overlap with SPEC-131** unless its
design routes errors through `common.rs`. There is one **API-level** overlap: SPEC-131 changes
`OperationParams` accessor semantics (absent vs wrong-typed), and this spec's walk calls
`get_str`. If both are in flight, whichever merges second rebases onto the other's `get_str`.
SPEC-133 (`src/operation/mod.rs`) and SPEC-134 (`src/sink/**`, naming): no expected overlap.
⚠ Check SPEC-134's file list at its design: if it changes `build`'s template handling in
`src/cli/build.rs` (its ruling extends to `build`), the two collide in that file, though in
different functions.

### Scope

**In:**
- Every recipe asset (`registry.asset_keys`, today watermark `image` + `font`) gets its
  **containing directory** watched under `build --watch`, for every target.
- The watch set is recomputed when a recipe changes (not only the manifest), so a re-pointed
  asset is picked up mid-session.
- An asset that does not exist yet: its directory is still a watch root if it exists, so
  creating the file triggers a build. If the directory does not exist either, today's
  `warning: cannot watch …` path applies (`build.rs:971-973`). Design decides whether that is
  enough.
- Tests: pure watch-set unit tests (image + font, asset outside every other root) and end-to-end
  `tests/build_watch.rs` tests on the existing `nudge_until` pattern.

**Out:** see "Out of scope" below.

### Rough build file list
- `src/build/watch.rs`: `watch_roots` input / new pure helper; unit tests.
- `src/cli/build.rs`: `watch_impl` (collect asset paths from the loaded recipes; recompute
  trigger); possibly `register_roots` unchanged.
- `src/cli/common.rs`: **only if** design extracts a shared asset-path walk.
- `tests/build_watch.rs`: end-to-end image + font + re-point tests.
- `decisions/DEC-060-*.md`: `affected_scope` is stale (lists `src/cli/mod.rs`; the watch loop
  moved to `src/cli/build.rs` in SPEC-097), plus a short amendment or a DEC-103. Design's call.
- `CHANGELOG.md` (Unreleased / 0.8.0 entry).

### Questions only the maintainer can answer
None. Every open call is answered by code or a DEC, or is a design detail:
- Watch the file or its directory: DEC-060 already rules **directory, never a bare file**
  (atomic-save rename). Shallow, to keep out of the cache tree.
- An asset path outside the manifest dir (`../shared/logo.png`): DEC-061's ruling for sources
  (roots follow the manifest, no clamp) applies the same way. Recommend following it.

## Design (2026-10-25)

### What design measured on top of the frame

Release build of `main` at `5caaffa` (`cargo build --release`, reports `crustyimg 0.7.1`), macOS
(FSEvents). Layout as in the frame: manifest at the root (shallow `.`), `source = "src/*.png"`
(recursive `src`), recipe `recipes/r.toml` (shallow `recipes`), every asset in a directory that
is **none** of those. "Attempts" counts `^built ` + `^error` lines in a stderr log kept outside
the project. Driver: the frame's `repro.sh` plus `drive2.sh` (same shape; scratchpad, not
committed).

| row | layout / action | attempts before → after | verdict on `main` |
|---|---|---|---|
| frame row, `image` | edit `brand/logo.png` ×3 | 1 → 1 | **no rebuild** (re-confirmed) |
| frame row, `font` | rewrite `brand/Go.ttf` ×3 | 2 → 2 | **no rebuild** (re-confirmed) |
| positive control (both) | edit `src/a.png` | +1 | rebuilds |
| negative control (both) | idle 3 s | +0 | no self-trigger |
| **re-point `image`** | rewrite recipe `image = "art/b.png"`, then edit `art/b.png` ×3 | 1 → 2 → **2** | re-point rebuilds (recipe dir is watched); every later edit of the new asset is missed |
| **re-point `font`** | rewrite recipe `font = "art/Go.ttf"`, then corrupt `art/Go.ttf` ×3 | 1 → 2 → **2** | same; and the corrupt font is never reported |
| **asset dir absent at start** | `image = "brand/logo.png"`, no `brand/`; create it, touch recipe, then edit ×3 | 2 → 3 → 4 → **4** | creating `brand/` wakes the shallow `.` watch, the build recovers, but `brand/` is never registered, so every later edit is missed |
| missing root warning | `source = "nope/*.png"` | — | `warning: cannot watch nope: No path was found. about ["nope"]`: FSEvents refuses a missing path, like inotify |
| observable check | corrupt font, one-shot `build` | — | `error: invalid operation 'watermark': could not parse font: …`, exit 1; restoring it → `(1 cached, 0 rebuilt)`; a second run (self-diff) → same, exit 0 |

**Two findings that shape the tests:**

1. ⚠ **A spurious build at startup.** The frame's `font` run logged a second `built … (1 cached,
   0 rebuilt)` line within ~2 s of start, before any edit (attempts = 2 at "settled"; the `image`
   run had 1). FSEvents delivered events from the fixture writes made just before the watcher
   started. So **counting `built` lines from start can fake a pass.** Every end-to-end test below
   (a) waits for the log to *settle* before taking its baseline and (b) asserts an observable only
   the asset edit can produce: changed output bytes for `image`, a new `error:` line from a
   corrupted font for `font`. A cache-hit spurious build changes no bytes and prints no error.
2. **The missing-directory case is a third independent bug**, not the frame's "today's warning
   path applies". A fix that adds asset dirs at startup and recomputes on recipe edits still
   misses it: the dir is in the computed set from the start (lexically), registration fails once,
   the set never *differs* again, so nothing re-registers. Measured on `main` as the last row.
   This spec fixes it (Call 3); it is in the frame's scope ("Design decides whether that is
   enough"). It is not enough.

### Call 1 — the watch set gains asset directories, shallow, as a third input to `watch_roots`

`watch_roots(manifest, manifest_path, assets: &[PathBuf]) -> WatchSet`. `watch_roots` stays pure
and filesystem-free (module docs, `src/build/watch.rs:1-23`): it takes the asset paths as given
and never opens a recipe. For each asset path:

- watch its **containing directory** (`containing_dir`, `watch.rs:210`), never the bare file
  (DEC-060: an editor's atomic save replaces the inode);
- **shallow** (`WatchSet::shallow`), never recursive. An asset's directory can be the project
  root (`image = "logo.png"` → `.`), and a recursive `.` covers `.crustyimg/` and `dist/`, which
  is the inotify flood DEC-060's two-tier rule exists to prevent. A shallow watch on the
  directory sees every save of the file;
- **dropped when a recursive root already covers it** (the asset dir equals a recursive root or
  sits under one, compared by components after `lexical_clean`). A recursive watch is a superset,
  and registering the same inode twice can clobber a Linux inotify descriptor (DEC-060). Today's
  exact-equality dedupe (`watch.rs:120`) stays as it is for manifest/recipe dirs; the
  covered-by check applies to asset dirs only, so no existing root changes;
- **unclamped**: `../shared/logo.png` watches `../shared`, the same rule DEC-061 accepted risk #2
  applies to sources. No warning (that item stays parked with the sources' one);
- de-duplicated with `push_unique`, so two steps naming files in one dir add one root.

`assets` is empty → the result is exactly today's. The five existing `watch_roots` test calls
gain `&[]` and must otherwise pass **unedited**. That is the regression guard.

**Rejected:** a second pure function that patches a `WatchSet` afterwards. Two functions would
each need the recursive/shallow dedupe, and the ordering between them becomes a rule to remember.

### Call 2 — the CLI collects asset paths with `asset_keys` + the string value; never the resolver

New in `src/cli/build.rs`, behind `#[cfg(feature = "watch")]`:

```rust
/// Every file a target's recipe names through an asset key (a watermark's `image` or `font`),
/// spelled as the recipe wrote it. Reads recipes, never the assets themselves. A recipe that
/// does not load contributes nothing: the build reports that error, and the recipe's own
/// directory is watched anyway, so fixing it brings its assets into the watch set.
fn recipe_asset_paths(manifest: &BuildManifest, registry: &OperationRegistry) -> Vec<PathBuf>
```

For each `target`: `load_recipe(&target.recipe)` (`src/cli/common.rs:156`) → on `Err`, skip; for
each `step`, `for &key in registry.asset_keys(&step.op)`, push `PathBuf::from(s)` when
`step.params.get_str(key)` is `Some(s)`. The raw loaded recipe is walked (no terminal-`optimize`
strip needed: `asset_keys("optimize")` is empty, `registry.rs:144-146`). A bundled recipe name
(`recipe = "web"`) loads from the bundle and names no asset.

- **Not `resolve_recipe_assets`** (settled at frame): it reads bytes and fails on a missing
  asset, and a missing asset is exactly a case that must still be watched.
- **No shared helper in `src/cli/common.rs`.** "Which keys are assets" already has one
  definition (`asset_keys`), and "is the value a path" is `get_str` in both walks. The
  duplicated part is a three-line loop. Extracting it would put this build into `common.rs` for
  no behaviour gain.
- Paths stay relative to the process working directory, the base `resolve_recipe_assets` reads
  from (`std::fs::read(&path)`, `common.rs:214`) and the base `is_excluded` absolutizes against
  (`normalize_abs`, `watch.rs:233`). So the watched dir is the dir the build reads.
- One `OperationRegistry::with_builtins()` is built at `watch_impl` start and reused.

A small `fn watch_set_for(manifest, manifest_path, registry) -> WatchSet` (also `watch`-gated)
composes the two, so startup and the loop compute the set the same way.

### Call 3 — recompute after every non-excluded batch; re-register when the set differs or a failed root now exists

Today (`src/cli/build.rs:1028-1037`) the set is recomputed only when the batch touched the
manifest, and then always re-registered. New loop tail, after `run_build`:

```rust
// (illustrative)
if let Ok(m) = load_manifest(path) {
    let next = watch_set_for(&m, Path::new(path), &registry);
    let retry = unwatched.iter().any(|root| root.is_dir());
    if next != set || retry {
        set = next;
        watcher = make_watcher(&tx)?;
        unwatched = register_roots(&mut watcher, &set);
    }
}
```

- **Recompute on every non-excluded batch**, not "did this batch touch a recipe file". It costs
  one manifest parse and one small TOML read per recipe per rebuild, which is nothing next to the
  build that just ran. It cannot miss a trigger: a manifest edit, a recipe edit, a recipe that
  goes from broken to valid (its assets appear) and a newly created directory all go through the
  same path. Matching the batch against recipe paths would need the *current* recipe list and a
  second normalization rule, and would still miss the broken→valid case when the fix was made
  via some other file.
- **Re-register only when the set differs, or a root that failed to register now exists as a
  directory.** `register_roots` (`build.rs:964`) returns the roots it could not watch
  (`Vec<PathBuf>`), keeping its `warning: cannot watch …` line. The retry is what fixes the
  missing-directory row. Re-registering is the existing mechanism (fresh watcher, the receiver
  survives, `make_watcher` `build.rs:938`); events in the swap window can be lost, as they already
  can on a manifest edit (DEC-060 accepted this).
- A manifest edit that changes nothing in the set no longer re-registers. That is safe: the
  watched roots are identical, and the retry still covers a root that was missing.
- A still-missing root does not re-register and does not re-print its warning on each rebuild.
  It warns once per registration.
- A manifest that fails to load mid-session keeps the current set (today's behaviour).
- `manifest_prefix` (`build.rs:1015`) goes away; nothing else uses it.

**Rejected:** re-register after *every* batch. It needs no comparison, but it recreates the
watcher on every save, reprints every missing-root warning on every rebuild, and widens the
lost-event window to every cycle. On FSEvents a fresh stream can also re-deliver the burst that
woke it (finding 1 above), which can turn one edit into two builds.

**What the user sees when the asset's directory does not exist** (AC-5): at start, one
`warning: cannot watch brand: …` line, then the existing build error
`error: recipe step 0 ('watermark'): could not read 'image' file "brand/logo.png": …`. The loop
keeps running (DEC-060: a cycle failure is not a hard exit). When the user creates the file, the
next change the loop sees re-registers `brand/`: creating `brand/` in a watched directory
(here, the shallow `.`), or touching the recipe or a source. From then on edits rebuild. If
nothing watched changes, nothing wakes the loop. That is the same reach a missing source root
has today, and it is stated in the docs line (below), not hidden.

### Call 4 — tests: pure unit tests on the set and the walk, plus end-to-end rows in `tests/build_watch.rs`

- **Unit, `src/build/watch.rs`** (unconditional, run in the lean build too): the set's shape for
  asset paths. Deterministic, no timing.
- **Unit, `src/cli/build.rs`** (`watch`-gated): `recipe_asset_paths` over recipes written to a
  `tempfile::TempDir` and referenced by **absolute** path in a `BuildManifest` built from a TOML
  string, so no test changes the process CWD.
- **End-to-end, `tests/build_watch.rs`**: the file's pattern is reliable. Seven tests use it,
  they run on all three CI OSes, and it was hardened against the Linux inotify flood (DEC-060,
  `nudge_until`). Reuse `spawn_watch`, `poll_until`, `nudge_until`, `write_png`, `bytes`. Add
  two helpers:
  - `fn attempts(log: &Path) -> usize`: count of lines starting `built ` or `error`;
  - `fn settled_attempts(log: &Path) -> usize`: poll until `attempts` has not changed for 2 s,
    return it; **panic** if it never settles within `DEADLINE`. A self-trigger loop never settles,
    so every test that settles also guards against one.
  - a `brand_project(recipe: &str)` fixture: `src/a.png`, `recipes/r.toml` = `recipe`,
    manifest at the root, `out = "dist"`, and **no** asset written (each test writes its own).
- ⚠ **Every end-to-end test puts the asset outside every watched path**: in `brand/` or `art/`,
  never beside the recipe, under `src/`, or at the root. The frame's trap row (asset beside the
  recipe) passes on `main` by accident. Each test's doc comment says so.

### Call 5 — DEC-060 is amended at design, not a new DEC-103

This is DEC-060's rule ("watch the directory, two tiers, re-register on change") applied to a new
input. No new principle, dependency or trade-off. An amendment section in
`decisions/DEC-060-build-watch-dependency-debounce-and-loop-semantics.md` records Calls 1–3, and
its `affected_scope` gains `src/cli/build.rs` (block-list form). `src/cli/mod.rs` stays in scope:
it still holds the `--watch` flag, `CliError::Watch` and the build-only guard
(`src/cli/mod.rs:141-143,697-701,830-834`). Written in this design commit, so
`decisions-audit --changed` flags DEC-060 on the build's diff. **DEC-103 is not used**; the
orchestrator can release the reservation.

### Call 6 — no cache, lockfile or version change

The cache already hashes asset content (SPEC-129/DEC-101). This spec only starts builds.
`CACHE_SCHEMA_VERSION`, the lockfile and every non-watch code path are unchanged. Ships in 0.8.0
with the rest of PROJ-011.

## Inputs

- **Files to read:** `src/build/watch.rs` (all), `src/cli/build.rs:890-1040` (`run_build_watching`,
  `make_watcher`, `register_roots`, `watch_impl`), `src/cli/common.rs:156-228` (`load_recipe`,
  `resolve_recipe_assets`, read only), `src/operation/registry.rs:85-150` (`asset_keys`),
  `tests/build_watch.rs` (all), `decisions/DEC-060-*.md`, DEC-061 accepted risk #2.
- **External APIs:** `notify` 8.2.0 (`Watcher::watch` error on a missing path, confirmed above on
  FSEvents).
- **Related code paths:** SPEC-131's registry change (`asset_keys` return lifetime narrows to
  `&self`; `get_str` unchanged). See Implementation Context.

## Outputs

- **Files created:** none.
- **Files modified:** see Implementation Context → "Files the build touches".
- **New exports:** none public. `watch::watch_roots` gains a third parameter (`pub` in a
  `pub(crate)`-used module; only `src/cli/build.rs` calls it).
- **Database changes:** none. No `CACHE_SCHEMA_VERSION`, lockfile or manifest-format change.

## Acceptance Criteria

Every AC applies to **`build --watch`** only, on a binary with the default `watch` feature. No
other verb has a watch loop (`apply` has none, `build` without `--watch` registers no watcher).
In every end-to-end AC the asset sits in a directory that is **not** the manifest's dir, a
recipe's dir, or under a source root.

- [ ] **AC-1. `image` present at start.** With `image = "brand/logo.png"`, rewriting
      `brand/logo.png` with different pixels makes the running watch loop rebuild, and
      `dist/a.png`'s bytes change from the settled pre-edit output. RED on `main` (measured:
      no rebuild).
- [ ] **AC-2. `font` present at start.** With `text = …` + `font = "brand/Go.ttf"`, overwriting
      `brand/Go.ttf` with bytes that are not a font makes the running loop rebuild and report it: a
      new `error:` line appears in stderr after the settled baseline, and the process is still
      running. RED on `main` (nothing is reported).
- [ ] **AC-3. `image` re-pointed mid-session.** Start on `image = "brand/logo.png"`; rewrite the
      recipe to `image = "art/b.png"` (an existing file, `art/` watched by nothing at start); after
      that rebuild settles, rewriting `art/b.png` with different pixels rebuilds and changes
      `dist/a.png`'s bytes. RED on `main` (measured).
- [ ] **AC-4. `font` re-pointed mid-session.** Start on `font = "brand/Go.ttf"`; rewrite the
      recipe to `font = "art/Go.ttf"` (a valid copy); after that rebuild settles, corrupting
      `art/Go.ttf` produces a new `error:` line. RED on `main` (measured).
- [ ] **AC-5. Asset directory absent at start.** With `image = "brand/logo.png"` and no `brand/`:
      stderr contains a `cannot watch` line naming `brand`, the initial build reports the existing
      unreadable-asset error, and the loop keeps running. After `brand/logo.png` is created and the
      loop has rebuilt successfully (`dist/a.png` exists), rewriting `brand/logo.png` with
      different pixels rebuilds and changes `dist/a.png`'s bytes. RED on `main` (measured: the
      last step never rebuilds).
- [ ] **AC-6. No self-trigger with asset roots registered, including after a re-registration**
      (control; GREEN on `main`, must stay GREEN). With `brand/` watched, after a re-point to
      `art/` has settled, a 3 s idle window adds **zero** attempts and leaves `dist/a.png`
      byte-identical. `settled_attempts` returning at all is the guard that the loop settles.
- [ ] **AC-7. Watch-set shape** (unit, `watch_roots`). For asset paths given to `watch_roots`:
      (a) each asset's containing dir is in `shallow` and not in `recursive`, for an `image`-style
      and a `font`-style path in **two different** dirs; (b) a root-level asset (`logo.png`) gives
      a shallow `.`, never a recursive one; (c) `../shared/logo.png` gives `../shared`
      (unclamped); (d) an asset under a recursive source root (`src/brand/logo.png` with
      `source = "src/**/*.png"`) adds no shallow root; (e) two assets in one dir add one root; (f)
      with `&[]`, every existing `watch_roots` test passes with only the added argument.
- [ ] **AC-8. Asset walk** (unit, `recipe_asset_paths`). (a) A two-target manifest whose recipes
      hold a `watermark` `image` step, a `watermark` `text`+`font` step and a `resize` step yields
      exactly the `image` and `font` paths, in target then step order; (b) a target whose recipe
      file does not exist, and one whose recipe does not parse, contribute nothing and do not
      error; (c) an asset path that does not exist on disk is still listed (the walk never reads
      asset bytes); (d) a non-string value (`image = 5`) is skipped.
- [ ] **AC-9. Negative control per independent condition** (AGENTS §15). Three conditions, each
      reverted alone on the build branch, release binary rebuilt from scratch, behavioural flip
      recorded:
      - **C1, asset dirs reach the set:** make `recipe_asset_paths` return empty → AC-1…AC-5 RED,
        AC-8 RED; AC-6, AC-7 (pure) GREEN.
      - **C2, recompute after every batch:** restore the manifest-only gate → AC-3, AC-4, AC-5
        RED; AC-1, AC-2 GREEN.
      - **C3, retry a failed root that now exists:** drop `|| retry` → AC-5 RED; AC-1…AC-4 GREEN.
- [ ] **AC-10. Nothing outside `build --watch` changes.** Existing `tests/build_watch.rs`,
      `tests/build.rs`, `tests/build_cache.rs`, `tests/build_watermark_cache.rs` and
      `tests/apply_batch.rs` stay green unedited. `src/cli/common.rs`, `src/build/cache.rs`,
      `src/build/lock.rs` and `src/operation/**` are not in the diff.
- [ ] **AC-11. Clean matrix**: default, `--no-default-features` (the new e2e tests and
      `recipe_asset_paths` are compiled out; the new `watch.rs` unit tests still run),
      `--features webp-lossy`; fresh `CARGO_TARGET_DIR` each, run sequentially; clippy
      `-D warnings` and `fmt --check` on each. The 3-OS CI is the gate for the end-to-end rows
      (Linux inotify and Windows are where watch timing differs); read every leg.
- [ ] **AC-12. Docs.** `docs/cli-reference.md`'s `--watch` paragraph says a file a recipe names
      (a watermark's image or font) is watched too, and that a directory that does not exist yet
      is picked up at the next rebuild. `CHANGELOG.md` `[Unreleased] → Fixed` gets one entry
      (wording below). The DEC-060 amendment is already on `main` (this design commit).

## Failing Tests

All new. Each end-to-end test's doc comment states where the asset sits and why that directory is
not otherwise watched.

- **`src/build/watch.rs`** (unit, unconditional)
  - `watch_roots_watches_each_asset_directory_shallow`: AC-7(a)(e). RED: does not compile
    (`watch_roots` takes two arguments on `main`).
  - `watch_roots_keeps_a_root_level_asset_dir_shallow`: AC-7(b). RED: does not compile.
  - `watch_roots_asset_outside_the_manifest_dir_is_unclamped`: AC-7(c). RED: does not compile.
  - `watch_roots_drops_an_asset_dir_a_source_root_covers`: AC-7(d). RED: does not compile.
- **`src/cli/build.rs`** (unit, in a new `#[cfg(all(test, feature = "watch"))] mod watch_tests`)
  - `recipe_asset_paths_lists_image_and_font_in_order`: AC-8(a). RED: no such function.
  - `recipe_asset_paths_skips_a_recipe_that_does_not_load`: AC-8(b). RED: no such function.
  - `recipe_asset_paths_lists_an_asset_that_does_not_exist`: AC-8(c). RED: no such function.
  - `recipe_asset_paths_skips_a_non_string_value`: AC-8(d). RED: no such function.
- **`tests/build_watch.rs`** (end-to-end, `watch` feature)
  - `watch_rebuilds_when_the_watermark_image_changes`: AC-1. RED: `brand/` is never watched,
    the bytes never change (frame row; re-measured above).
  - `watch_rebuilds_when_the_watermark_font_changes`: AC-2. RED: no rebuild, no `error:` line.
  - `watch_follows_a_recipe_repointed_to_a_new_image`: AC-3. RED: the set is recomputed only on
    manifest edits (measured).
  - `watch_follows_a_recipe_repointed_to_a_new_font`: AC-4. RED: same (measured).
  - `watch_picks_up_an_asset_directory_created_mid_session`: AC-5. RED: `brand/` is never
    registered (measured); also RED on a fix without C3.
  - `watch_with_asset_roots_does_not_self_trigger`: AC-6. **Control, GREEN on `main`.** Listed
    so verify runs it under each revert. It is not a failing test.

Noun count: `image` × {at start, re-pointed} = AC-1, AC-3; `font` × {at start, re-pointed} = AC-2,
AC-4; missing dir = AC-5 (`image` only: the missing-dir path is key-independent, the dir is what
fails to register); idle = AC-6. Unit: 4 shape cases + 4 walk cases.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Files the build touches (best estimate; the orchestrator schedules on this)

- `src/build/watch.rs`: `watch_roots` gains `assets: &[PathBuf]` (Call 1); `WatchSet` and
  module docs say "shallow = the manifest's, recipes' and recipe assets' directories"; the five
  existing test calls gain `&[]`; four new unit tests.
- `src/cli/build.rs`: **these functions only**: `register_roots` (returns the roots it could
  not watch), `watch_impl` (registry once; `watch_set_for` at start and in the loop; Call 3's
  re-register rule; `manifest_prefix` removed); new `recipe_asset_paths` and `watch_set_for`
  next to them; new `mod watch_tests` placed **directly after `watch_impl`**, not appended to
  the existing `mod tests` at the end of the file. ⚠ **Shared file with SPEC-134**, which
  edits `prepare_target` and `target_format_plan` (its Implementation Context). Different
  functions, and keeping the new tests out of `mod tests` avoids an end-of-file conflict.
  Whichever merges second rebases trivially.
- `tests/build_watch.rs`: helpers `attempts`, `settled_attempts`, `brand_project`; six new tests.
  No other spec in flight touches this file.
- `docs/cli-reference.md`: the `--watch` paragraph (`:343-349`), one sentence.
- `CHANGELOG.md`: one `[Unreleased] → Fixed` bullet (SPEC-131/133/134 add their own bullets;
  separate entries, trivial merge).
- **Not touched:** `src/cli/common.rs`, `src/cli/mod.rs`, `src/operation/**`, `src/recipe/**`,
  `src/build/{cache,lock,mod}.rs`, `decisions/**` (DEC-060 is amended in this design commit).

### SPEC-131 interplay: designed against its API, so either merge order is trivial

SPEC-131 (designed, `63fbfc0`) changes `OperationRegistry::asset_keys` to return
`&[&'static str]` borrowed from `&self` (was `&'static [&'static str]`) and derives it from a
param schema. It does **not** change `OperationParams::get_str`, and it leaves `src/cli/**`
alone. The walk in Call 2 is written to compile against both:
`for &key in registry.asset_keys(&step.op)` copies each `&'static str` out, and `registry` lives
for the whole of `watch_impl`. Do not store the slice itself anywhere that outlives the registry
borrow. Behaviour is the same on both: SPEC-131 refuses `image = 5` at `build_pipeline`, and
this walk skips it via `get_str` → `None` (AC-8(d)). That keeps the watch set from depending on
validation order. A recipe SPEC-131 refuses at `Recipe::from_toml` (a key on the terminal
`optimize` marker) fails `load_recipe` and contributes no assets, which is AC-8(b)'s rule.

### Decisions that apply

- `DEC-060`: watch semantics; directory never file, two tiers, re-register on change. **Amended
  in this design commit** with Calls 1–3. Read the amendment; it is the build's contract.
- `DEC-061`: accepted risk #2, watch roots follow the manifest unclamped. Applied to assets
  (AC-7(c)).
- `DEC-057`: manifest paths are relative to the working directory, the same base the asset
  resolver reads from.
- `DEC-058`: the cache prunes a full re-run, so there is no per-asset target graph.
- `DEC-100`: the asset seam; `asset_keys` is the one definition of "which keys are files".
- `DEC-101`: the cache key hashes asset bytes, so a rebuild this spec starts is correct.

### Constraints that apply

- `test-before-implementation`: write the 14 tests (13 failing + the AC-6 control), see the 13 RED (or not compile) on `main`, then
  implement.
- `one-spec-per-pr`.
- `no-unwrap-on-recoverable-paths`: a recipe that fails to load and a root that fails to
  register are both recoverable; neither may panic or end the loop.

### Prior related work

- `SPEC-067` (shipped): `build --watch`, the loop and `tests/build_watch.rs`.
- `SPEC-128` (shipped, PR #189): watermark in recipes, `asset_keys`, `resolve_recipe_assets`.
- `SPEC-129` (shipped, PR #190/#191): cache hashes asset content; its verify found this bug.
- `SPEC-131` (designed): see interplay above.

### Out of scope (for this spec specifically)

- **Cache correctness for an edited asset**: shipped in SPEC-129 / DEC-101. This spec only makes
  the watch loop start the build.
- **A per-asset change→target graph** (rebuild only the targets that use the edited asset). DEC-060
  rules out a dependency graph: the loop re-runs the whole build and the cache prunes it.
- **Assets on `apply`**: it has no watch mode.
- **A warning when an asset's watch root escapes the manifest dir**: the same low-severity item
  DEC-061 already parks for sources (`watch.rs` test `watch_root_escaping_source_follows_the_manifest_documented`);
  not filed separately, it belongs with that one.
- **Excluding an asset that sits inside a target's `out` dir or `.crustyimg/`**: a self-referential
  layout nobody writes. The existing exclusion wins; not worth a test. Needs filing only if
  someone hits it.
- **Future asset-bearing ops** (the `.cube` LUT): covered automatically if the walk uses
  `registry.asset_keys`. No op-specific code.
- **Watching a missing directory's nearest existing parent** so its creation always wakes the
  loop. Call 3's retry covers the common layouts (a top-level dir under the shallow root);
  anything deeper waits for the next watched change, as a missing source root does today.
- **Widening the covered-by-a-recursive-root dedupe to recipe/manifest dirs.** Today's
  exact-equality rule stays for them; changing it is not this bug.

## Notes for the Implementer

- **The fake-pass trap is the main risk.** If any end-to-end test puts its asset beside the
  recipe, under `src/`, or at the project root, it passes on `main`. Before implementing, run all
  five failing e2e tests on `main` and confirm each is RED **for the asserted reason** (a
  deadline on the bytes/`error:` observable, not a setup panic).
- **Never baseline on a raw attempt count from start** (finding 1). Take baselines with
  `settled_attempts`, and assert the content observable (bytes for `image`, `error:` for
  `font`). An AC-2/AC-4 assertion of "attempts went up" alone is a fake pass on macOS.
- Write images with `write_png` at a **different colour** for each edit so the bytes really
  differ. `nudge_until` re-issues the same write, which is fine: a repeat is a cache hit.
- For the font, copy `assets/fonts/Go-Regular.ttf` (the repo's only TTF) via
  `env!("CARGO_MANIFEST_DIR")`. The edit is `std::fs::write(path, b"not a font")`. Match only the
  start of an `error` line, not the message text: SPEC-131 may reword constructor messages.
- AC-5's `cannot watch` line comes from `notify`'s error text, which differs per OS (FSEvents:
  `No path was found. about ["brand"]`). Assert only `cannot watch` + `brand` on the same line.
- AC-5 wake-up: after creating `brand/logo.png`, use `nudge_until` that rewrites the recipe with
  identical bytes until `dist/a.png` exists. Don't rely on the `brand/` creation event alone (it
  depends on the shallow `.` watch, which is incidental to the layout).
- `register_roots` keeps printing `warning: cannot watch {root}: {e}` exactly as today; it only
  also collects the root.
- Keep `recipe_asset_paths` and `watch_set_for` behind `#[cfg(feature = "watch")]`, or the lean
  build fails clippy on dead code. `watch::watch_roots` stays unconditional.
- Comments read behaviour-first, with no spec or DEC ids (repo rule).
- CHANGELOG wording (adapt freely, keep it behaviour-first):
  > **`build --watch` now rebuilds when a watermark's image or font changes.** A recipe's
  > `image` or `font` file was never watched, so editing a logo under `--watch` did nothing until
  > a source changed, unless the file happened to sit beside the recipe or among the sources.
  > The file's folder is now watched wherever it is, including a file the recipe is changed to
  > point at mid-session and a folder created after the watch started.
- **Checkpoint:** push a WIP commit once it compiles with the unit tests green, before the
  end-to-end runs and the matrix.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** feat/spec-132-watch-recipe-assets
- **PR (if applicable):** see PR title `fix(build): --watch watches recipe asset paths (SPEC-132)`
- **All acceptance criteria met?** yes (AC-11's 3-OS CI leg is the open gate)
- **New decisions emitted:** none
- **Files this diff touches** (from `git diff --name-only origin/main...HEAD`):
  - `src/build/watch.rs` — `watch_roots` takes `assets`; shallow asset dirs, dropped when a recursive root covers them (`covers`); 4 new unit tests, 5 existing calls gain `&[]`
  - `src/cli/build.rs` — `make_watcher` skips read-only access events (new `is_read_only_access`); `register_roots` returns failed roots; new `recipe_asset_paths`, `watch_set_for`; `watch_impl` recomputes after every non-excluded batch and re-registers on a changed set or a retryable root; `manifest_prefix` removed; new `mod watch_tests`
  - `tests/build_watch.rs` — helpers `attempts`, `settled_attempts`, `brand_project` (+ `start_and_settle`); 6 new tests
  - `docs/cli-reference.md` — `--watch` paragraph
  - `CHANGELOG.md` — one `[Unreleased] → Fixed` entry
- **RED baseline (unmodified `src/`):** the 4 `watch.rs` and 4 `recipe_asset_paths` unit tests fail to compile only on the `watch_roots` arity and the missing function. On the unmodified binary the five end-to-end tests fail on their asserted observable (image bytes unchanged; no new `error:` line; AC-5's final rebuild never happens), not a setup panic. AC-6 passes (control). AC-5's `cannot watch` assertion was moved to the end of the test so the final step's own failure is what `main` reports.
- **Negative controls** (`cargo clean -p crustyimg` before each run, restored with `git checkout`):
  - C1 (`recipe_asset_paths` empty): AC-1..AC-5 RED (5 e2e), AC-8 three of four RED (the non-string case passes on an empty list); AC-6 and the pure AC-7 tests GREEN.
  - C2 (manifest-only recompute gate): AC-3, AC-4, AC-5 RED; AC-1, AC-2, AC-6, AC-8 GREEN.
  - C3 (drop `|| retry`): AC-5 RED only; AC-1..AC-4, AC-6 GREEN.
- **Matrix** (fresh `CARGO_TARGET_DIR` each, sequential, fmt --check + clippy --all-targets -D warnings + test): default, `--no-default-features`, `--features webp-lossy` all green locally. CI is the gate for Linux/Windows timing.
- **Deviations from spec:**
  - AC-10 diff check: `src/cli/common.rs`, `src/build/cache.rs`, `src/build/lock.rs`, `src/operation/**` are not in the diff.
  - **`make_watcher` (outside the spec's function list) now drops read-only access events.** CI
    (Linux) showed `build --watch` waking itself: `notify`'s inotify backend watches `OPEN`, so
    the build's own reads of the manifest, recipe and assets re-triggered it (~216 attempts in
    45 s). Reproduced in Docker on unmodified `origin/main` with the control test, so it predates
    this spec; the existing tests compare output bytes only, and a hot loop of cache hits leaves
    them unchanged, which hid it. `settled_attempts` exposed it. Open / read / read-close events
    are skipped; a write-close, modify, create and rename still wake the loop. Unit test
    `reads_are_not_changes_but_writes_are`; verified green in Docker (Linux) for all 13 tests.
  - Extra helper `start_and_settle` in the e2e file (not in the spec's helper list).
- **Follow-up work identified:** none.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing blocking. AC-5 as written is red on `main` at its `cannot watch` line first, which hides the final step; ordering the assertion last fixed it.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. All six design calls held; none turned out wrong.

3. **If you did this task again, what would you do differently?**
   — Run clippy on the test target before the first matrix leg: one `cmp_owned` lint in a new test cost a full leg rerun.

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
