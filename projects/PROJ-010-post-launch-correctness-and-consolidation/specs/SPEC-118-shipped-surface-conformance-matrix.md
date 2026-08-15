---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-118
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-010
  stage: STAGE-042
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5     # build on Sonnet: the matrix is mechanical
                                   # once the shape is settled, and it is settled
                                   # below. Verify stays Opus.
  created_at: 2026-08-15

references:
  decisions:
    - DEC-087
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-111
    - SPEC-112
    - SPEC-115

value_link: >
  STAGE-042's central instrument. Every PROJ-010 defect so far has been an
  UNENUMERATED CELL — a verb × input × entry-point combination nobody listed, so
  nobody tested. This spec builds the enumeration itself, and it would have caught
  SPEC-111 and SPEC-112 before either shipped.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Read `recipe::bundled`,
        the three recipe entry points, and `Commands`; settled the wasm-leg
        coupling and the exhaustiveness mechanism rather than deferring either.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-118: the shipped-surface conformance matrix

## Context

Every defect PROJ-010 has fixed shares one shape: **a cell nobody enumerated.**

- **SPEC-111** — `build` gained an auto-decide path; no bundled recipe was ever run through it.
- **SPEC-112** — `wasm::transform` was the last call site handing a terminal `optimize` to
  `build_pipeline`, so none of the three bundled recipes ran through it, while the README told
  readers they did.
- **SPEC-115** — every input-format test drove `optimize` with a **pinned** `-o`; not one drove
  the decide path, so three input families were untested on the flagship verb's default.
- **SPEC-113** — the pinned branch of a two-branch fork; every prior fix landed on the other one.

In each case the individual pieces were tested and the *combination* was not. Fixing them one at
a time treats the symptom. **This spec builds the enumeration**, so a missing combination fails
the build instead of reaching a user.

STAGE-042's backlog states the payoff plainly: *"This single test would have caught SPEC-111 and
SPEC-112 before either shipped."*

## Goal

A test that iterates the shipped surface — every bundled recipe × every entry point that accepts
one — and asserts each combination produces valid output for the requested format. Plus a second
test that fails when the surface grows without the matrix growing with it.

## Inputs

- **Files to read:**
  - `src/recipe/bundled.rs` — `names()` (`:61`) and `resolve()` (`:56`); three recipes today:
    `web`, `gallery`, `product` (`:41,45,49`).
  - `src/cli/common.rs:160-171` — where a recipe **name** is resolved for the CLI.
  - `src/wasm.rs:171` — `transform(input, recipe_toml, out_format)`. **Note it takes TOML, not a
    name** — the wasm leg must resolve the bundled recipe itself.
  - `src/cli/mod.rs:229` — `Commands`, the enum the verb list must stay exhaustive against.
  - `src/cli/build.rs` — how a manifest target names a recipe.
- **Prior art for the assertion style:** `tests/input_svg.rs` (SPEC-115) — sniff the written
  bytes, never the extension or the summary line.

## The design calls — settled here

**1. Two dimensions, not one.** The matrix is `bundled_recipe × entry_point`:

| | `apply --recipe <name>` | `build` manifest target | `wasm::transform` |
|---|---|---|---|
| `web` | ✔ | ✔ | ✔ |
| `gallery` | ✔ | ✔ | ✔ |
| `product` | ✔ | ✔ | ✔ |

Nine assertions today, and it **extends by itself** when a fourth recipe is added — which is the
whole point. Iterate `bundled::names()`; do not hard-code the three.

**2. The verb half needs an exhaustiveness guard to be real.** A `PIXEL_LANE_VERBS` list plus a
test asserting it is exhaustive against `Commands`. Adding a verb without classifying it must
**fail the build**, not silently skip coverage. Without this the matrix is a snapshot that rots;
with it, the matrix is a gate.

**3. The wasm leg is real coverage, and it is currently unrun.** `just wasm-test` is executed by
**no CI leg** — `ci.yml` has no wasm32 step and `pages.yml`'s browser smoke drives only the demo's
markerless path. So the wasm third of this matrix runs only on a maintainer's machine.
**This spec still includes the wasm leg**, and is explicitly coupled to STAGE-042's wasm32 CI
chore: *a matrix nobody runs is worth nothing*. If the CI chore has not landed when this builds,
say so in Build Completion and state plainly that one third of the matrix is unenforced.

**4. Assert on decoded output, not exit status.** Each cell must decode the written bytes and
confirm the format matches what the recipe requested. `exit 0` is what SPEC-111 and SPEC-112 both
returned while producing nothing valid.

## Outputs

- **Files modified/created:** a new `tests/conformance.rs` (or similar — say which and why);
  `src/cli/mod.rs` if `PIXEL_LANE_VERBS` lives there.
- **New fixtures:** reuse existing ones. If a recipe needs an input none of the committed
  fixtures satisfy, **say which and why** before adding one.
- **New exports:** possibly `PIXEL_LANE_VERBS`; keep it as tight as the tests allow.
- **New DEC:** none expected. If the exhaustiveness check needs a macro or a build-time
  reflection trick, that IS a decision — record it.

## Acceptance Criteria

- [ ] **AC-1.** **The recipe × entry-point matrix exists and passes**, iterating
      `bundled::names()` rather than a hard-coded list. Nine cells today.
- [ ] **AC-2.** **Each cell asserts decoded output**, not exit status: the written bytes decode,
      and their format is the one the recipe asked for.
- [ ] **AC-3.** **Adding a bundled recipe extends the matrix automatically.** Driven: add a
      fourth recipe locally, confirm the test count rises without editing the test, remove it.
- [ ] **AC-4.** **`PIXEL_LANE_VERBS` is asserted exhaustive against `Commands`.** Driven: add a
      dummy variant locally, confirm the test **FAILS**, remove it. This is the criterion that
      makes the guard a gate rather than a snapshot. [[a-guards-advertised-reach-is-a-claim]]
- [ ] **AC-5.** **The matrix would have caught SPEC-111 and SPEC-112.** Driven, not asserted:
      revert each fix in turn (`git revert --no-commit` the relevant hunk, or re-introduce the
      original defect), confirm the matrix goes RED, restore. **This is the spec's central
      claim and the reason it is worth building — if the matrix does not catch them, it does not
      do what STAGE-042 says it does.**
- [ ] **AC-6.** **The wasm leg is included** and passes under `just wasm-test`. If no CI leg runs
      it, Build Completion must say so explicitly and name what is therefore unenforced.
- [ ] **AC-7.** **No false green from an empty matrix.** Assert the test actually ran N cells,
      with N derived from `names().len() × entry_points`. A matrix that iterates an empty list
      passes silently. [[a-harness-that-exercises-nothing-reports-green]]
- [ ] **AC-8.** Clean **full matrix**, fresh per-leg `CARGO_TARGET_DIR`, sequential, through
      `rtk proxy`: default, `--no-default-features`, `--features webp-lossy`, **and
      `--features heic`**. Clippy and `fmt --check` each. Establish your own `main` baseline.
      **Then read the CI legs individually.**

## Failing Tests

Written during **design**, BEFORE build.

- **`tests/conformance.rs`**
  - `"every_bundled_recipe_runs_through_every_entry_point"` — AC-1/AC-2/AC-7. **Passes today**
    on `main`, because SPEC-111/112/115 already fixed the cells it covers. AC-5's reverts are
    what prove it can fail.
  - `"pixel_lane_verbs_is_exhaustive_against_commands"` — AC-4. **Passes today**; fails the
    moment a verb is added without classification.
- **Negative controls** (AC-5 and AC-4, run and recorded, not committed)
  - Revert SPEC-111's fix → matrix RED. Revert SPEC-112's fix → matrix RED. Add a dummy
    `Commands` variant → exhaustiveness test RED.

## Implementation Context

### Decisions that apply
- **DEC-087** — `build`'s recipe handling and the `encode_one_optimize_decided` seam.

### Constraints that apply
- `test-before-implementation` — **applies unusually**, as in SPEC-117: the matrix passes on
  `main` by construction. **AC-5 is the substitute**, and it is stronger than a red test because
  it demonstrates the matrix catches two real, historical defects.
- `clippy-fmt-clean`, `one-spec-per-pr` (**blocking**).

### Prior related work
- **SPEC-111 / SPEC-112** — the two defects this matrix must retro-catch. Read both Build
  Completions; SPEC-112's records that its design offered a **false choice** (a `pub(crate)`
  widening that cannot work across `cfg(target_arch)` trees), which is worth knowing before
  touching the wasm leg.
- **SPEC-115** — the assertion style to copy.

### Out of scope
- Fixing anything the matrix finds. **If a cell fails on `main`, that is a live defect: stop,
  report it, and let it be framed as its own spec.** Do not fix it inside this one — that is how
  a test-building spec turns into an unbounded one.
- The wasm32 CI leg itself (STAGE-042's separate chore).
- Non-bundled, user-supplied recipes.

## Notes for the Implementer

- **The value is in AC-5.** A matrix that passes proves little; a matrix demonstrated to catch
  two historical defects proves it is load-bearing. Budget for it.
- **Do not hard-code three recipes.** The self-extension is the feature.
- **A piped command reports the pipe's exit code.** Redirect and read `$?`.
- macOS has no `timeout(1)`. `git commit -s` (DCO). **Own git worktree.** **Do not merge the PR.
  Do not bump the version.**
- Follow `projects/_templates/prompts/closing-steps-snippet.md` at the end, including
  `just advance-cycle SPEC-118 verify` — and confirm the `cycle:` line actually moved.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
- **Deviations from spec:**
- **Follow-up work identified:**

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
2. **Was there a constraint or decision that should have been listed but wasn't?**
3. **If you did this task again, what would you do differently?**

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
