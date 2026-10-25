---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-132
  type: bug                        # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
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
  implementer: null                # chosen at build dispatch
  created_at: 2026-10-24

references:
  decisions:
    - DEC-060
    - DEC-061
    - DEC-100
    - DEC-101
    - DEC-057
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

## Inputs

*Design fills this in.*

## Outputs

*Design fills this in.*

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in
**build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-NNN` — <one-line summary of why this matters here>
- `DEC-MMM` — <one-line summary>

### Constraints that apply

These constraints apply to the paths touched by this task (see
`/guidance/constraints.yaml` for full text):

- `constraint-id-1` — <one-line summary>
- `constraint-id-2` — <one-line summary>

### Prior related work

- `SPEC-YYY` (shipped) — <one-line summary, if relevant>
- `PR #NNN` — <link, if relevant>

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

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

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Files this diff touches** — **list every one**, with a phrase on what changed
  in each. Not "src/foo and tests": the enumeration is what a verify cycle diffs
  the real change against, and what tells you whether every affected `DEC-*` has
  the path in its `affected_scope`. SPEC-122's Deviations claimed "`src/operation`
  and `tests/` only" and was wrong by two `scripts/` files, which left
  `affected_scope` blind; SPEC-124's build added this section by hand after that
  lesson and still listed 8 of its 9 files. It is a checklist because it is easy
  to get wrong from memory — build it from `git diff --name-only`, not recall.
  - `path/to/file` — <what changed>
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

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
