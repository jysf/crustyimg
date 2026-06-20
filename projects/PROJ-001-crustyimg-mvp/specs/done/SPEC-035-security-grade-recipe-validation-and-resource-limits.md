---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-035
  type: story                      # epic | story | task | bug | chore
  cycle: ship  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-006
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet (prescriptive prompt)
  created_at: 2026-06-19

references:
  decisions: [DEC-036, DEC-005, DEC-007]
  constraints:
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - clippy-fmt-clean
    - every-public-fn-tested
  related_specs: [SPEC-006, SPEC-031, SPEC-033]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-006's <capability>". Optional; null is acceptable.
value_link: >
  Third STAGE-006 hardening item: caps an untrusted recipe's size and step
  count at the single load choke point so a hostile recipe can't exhaust
  memory/CPU on parse or pipeline-build — the recipe analog of decode limits.

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
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: >
        Main-loop orchestrator work, not separately metered. Read the recipe
        loader + the resize op; established that the functional validation
        (version/unknown-op/param) already exists and the real delta is resource
        bounding. Authored the spec (Limits policy PINNED, Failing Tests,
        Implementation Context) + DEC-036 + the Sonnet build prompt. Pinned the
        load-bearing order (size before parse, steps after version) and inclusive
        boundaries, and recorded the resize-upscale-bomb (op-param bound) as an
        explicit out-of-scope follow-up. std-only, no new dep. Third STAGE-006 spec.
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: 86548
      estimated_usd: 0.47
      duration_minutes: 4
      recorded_at: 2026-06-19
      notes: >
        Real metered subagent on Sonnet 4.6. subagent_tokens=86548,
        duration_ms=258217. estimated_usd at Sonnet list ($3/$15 per MTok,
        ~80/20). recipe resource limits: RECIPE_MAX_BYTES/RECIPE_MAX_STEPS +
        RecipeError::TooLarge/TooManySteps enforced in from_toml (size before
        parse, steps after version) + CLI run_apply pre-read metadata guard;
        reuse Recipe(_) exit 1; std-only, no new dep. 404 tests green (6 unit + 2
        integration new); clippy/fmt/lean/deny clean. No deviations.
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 55000
      estimated_usd: 0.50
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: >
        ORDER-OF-MAGNITUDE ESTIMATE (~55k) — read-only Explore subagent on Opus
        (no metered usage block) + orchestrator main-loop gate re-runs (cargo
        test 404 ok / clippy / fmt / deny / lean). Explore verdict: APPROVED, no
        concerns; verified the caps match DEC-036, the load-bearing check
        ordering (size→parse, version→steps), inclusive boundaries, the CLI
        pre-read guard precedes read_to_string, and ran a gap-hunt confirming
        from_toml is the single load path with no bypass.
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: "Main-loop ship bookkeeping (merge dance + cost totals + reflection + archive); not separately metered."
  totals:
    tokens_total: 141548
    estimated_usd: 0.97
    session_count: 4
---

# SPEC-035: security-grade recipe validation and resource limits

## Context

**The third STAGE-006 hardening item.** SPEC-033 bounded decode (input pixels)
and SPEC-034 bounded paths; this spec bounds the **recipe** — the other untrusted
artifact `apply --recipe` consumes. The recipe loader already does the
*functional* validation: `Recipe::from_toml` rejects malformed TOML
(`RecipeError::Parse`) and an unsupported `version` (`UnsupportedVersion`), and
`build_pipeline` rejects an unknown op (`UnknownOperation`) or invalid params
(`InvalidOperation`) — all typed (SPEC-006). What's missing is **resource
bounding**: a hostile recipe can still be arbitrarily large (parse-time
memory/CPU DoS) or carry an enormous number of steps (pipeline-build DoS).

This spec adds two caps at the **single recipe load choke point**
(`Recipe::from_toml`, through which every caller — `apply` — funnels): a recipe
**text-size cap** (reject before parsing) and a **step-count cap** (reject after
parsing), plus a CLI **pre-read file-size guard** so a multi-GB recipe file is
never read into memory. Both surface as typed `RecipeError` variants → exit 1.
This is the recipe analog of SPEC-033's decode limits. Parent: `STAGE-006`
(backlog item #3). Governing: **DEC-036** (the caps), **DEC-005** (recipe
round-trip), **DEC-007** (typed errors). No new dependency.

## Goal

Reject an untrusted recipe whose text exceeds `RECIPE_MAX_BYTES` or whose step
count exceeds `RECIPE_MAX_STEPS` with a typed `RecipeError` (CLI exit 1) — at the
`Recipe::from_toml` choke point, plus a CLI file-size pre-read guard — so a
hostile recipe can't exhaust memory/CPU on parse or pipeline build, while every
normal recipe loads unchanged.

## Inputs

- **Files to read:**
  - `src/recipe/mod.rs` — `Recipe::from_toml` (the choke point to harden; add the
    size + step checks), `RecipeError` (add the two variants), `SUPPORTED_VERSION`.
  - `src/cli/mod.rs` — `run_apply` (the `std::fs::read_to_string(recipe_path)`
    read path — add the metadata pre-read size guard), `CliError` (already maps
    `Recipe(_) => 1`).
  - `decisions/DEC-036` (the policy this implements), `DEC-005`, `DEC-007`.
- **External APIs:** none new (`std::fs::metadata`, already-present `toml`).
- **Related code paths:** `src/recipe/`, `src/cli/mod.rs`, `tests/`.

## Outputs

- **Files modified:**
  - `src/recipe/mod.rs` — add `pub const RECIPE_MAX_BYTES: usize = 64 * 1024;`
    and `pub const RECIPE_MAX_STEPS: usize = 1024;`; add `RecipeError::TooLarge {
    size: usize, max: usize }` and `RecipeError::TooManySteps { count: usize, max:
    usize }`; enforce both in `Recipe::from_toml` (size before parse, step count
    after the version check). Unit-tested.
  - `src/cli/mod.rs` — in `run_apply`, before `read_to_string`, check
    `std::fs::metadata(recipe_path).len()` against `RECIPE_MAX_BYTES` and return
    `CliError::Recipe(RecipeError::TooLarge { .. })` when over.
  - `docs/api-contract.md` — note recipe size/step caps (exit 1). (Done at design.)
  - `SECURITY.md` — mark the untrusted-recipe row's resource-DoS as bounded
    (SPEC-035 / DEC-036). (Done at design.)
- **New exports:** `RECIPE_MAX_BYTES`, `RECIPE_MAX_STEPS` (pub consts);
  `RecipeError::{TooLarge, TooManySteps}` (pub variants).
- **Database changes:** none.

## Limits policy (PINNED — DEC-036)

- **Constants** (in `src/recipe/mod.rs`, `pub` so the CLI shares them):
  - `RECIPE_MAX_BYTES: usize = 64 * 1024` (64 KiB).
  - `RECIPE_MAX_STEPS: usize = 1024`.
- **`Recipe::from_toml(s: &str)` order:**
  1. `if s.len() > RECIPE_MAX_BYTES` → `Err(RecipeError::TooLarge { size: s.len(),
     max: RECIPE_MAX_BYTES })` — **before** `toml::from_str` (don't parse an
     oversized string).
  2. parse (existing) → `Parse` on failure.
  3. version check (existing) → `UnsupportedVersion`.
  4. `if recipe.steps.len() > RECIPE_MAX_STEPS` → `Err(RecipeError::TooManySteps {
     count: recipe.steps.len(), max: RECIPE_MAX_STEPS })`.
- **CLI pre-read guard** (`run_apply`, before `read_to_string`):
  `let meta = std::fs::metadata(recipe_path).map_err(CliError::RecipeIo)?; if
  meta.len() > RECIPE_MAX_BYTES as u64 { return Err(CliError::Recipe(
  RecipeError::TooLarge { size: meta.len() as usize, max: RECIPE_MAX_BYTES })); }`
  then the existing `read_to_string`. (Avoids reading a huge file into memory.)
- **Exit code:** both new variants map to **1** via the existing
  `CliError::Recipe(_) => 1` arm (no new exit code).
- **Reject, do not truncate.** An over-limit recipe is refused, not clipped.
- **Existing validation unchanged:** version + unknown-op + invalid-param
  rejection (SPEC-006) stays exactly as is; these caps are additive.

## Acceptance Criteria

- [ ] `Recipe::from_toml` on a `> 64 KiB` string → `Err(RecipeError::TooLarge { ..
  })`, and the oversized string is NOT parsed (the size check precedes parsing).
- [ ] `Recipe::from_toml` on a valid recipe with `> 1024` steps →
  `Err(RecipeError::TooManySteps { count, max: 1024 })`.
- [ ] A recipe at exactly the caps (`== RECIPE_MAX_BYTES`, `== RECIPE_MAX_STEPS`)
  is accepted (boundary is inclusive; only `>` is rejected).
- [ ] A normal small recipe still loads, round-trips, and builds its pipeline
  unchanged (no regression to SPEC-006 behavior).
- [ ] The existing `UnsupportedVersion` / `UnknownOperation` / `Parse` paths are
  unchanged (a malformed recipe is still `Parse`; a bad version still
  `UnsupportedVersion`).
- [ ] `run_apply` with a recipe file larger than `RECIPE_MAX_BYTES` exits **1**
  (the metadata guard fires before `read_to_string`); a normal recipe file works.
- [ ] `CliError::Recipe(RecipeError::TooLarge { .. }).code() == 1` and likewise for
  `TooManySteps` (the existing `Recipe(_) => 1` arm covers them; confirm).
- [ ] `cargo deny` green; the **lean build** compiles; no new dependency; no
  `unwrap`/`expect`/`panic!` on the new non-test paths.

## Failing Tests

Written during **design**, BEFORE build. Build recipe TOML as inline strings;
generate the oversized/many-step fixtures programmatically (a `String` built in
the test — do NOT commit a 64 KiB fixture file).

- **`src/recipe/mod.rs` (unit, `#[cfg(test)] mod tests`)**
  - `"from_toml_rejects_oversized_recipe"` — a string of length `RECIPE_MAX_BYTES
    + 1` (e.g. `"#".repeat(RECIPE_MAX_BYTES + 1)`, a TOML comment so it would
    otherwise parse) → `Err(RecipeError::TooLarge { .. })`.
  - `"from_toml_accepts_recipe_at_size_cap"` — a valid recipe whose text length is
    `<= RECIPE_MAX_BYTES` loads `Ok` (boundary inclusive).
  - `"from_toml_rejects_too_many_steps"` — programmatically build a recipe TOML
    with `RECIPE_MAX_STEPS + 1` `[[step]]` (op = "identity") entries (well under
    the byte cap is impossible at 1025 steps — so this test must also stay under
    `RECIPE_MAX_BYTES`; `identity` steps are ~18 bytes, 1025×18 ≈ 18 KB < 64 KB,
    OK) → `Err(RecipeError::TooManySteps { count: 1025, max: 1024 })`.
  - `"from_toml_accepts_recipe_at_step_cap"` — exactly `RECIPE_MAX_STEPS`
    `identity` steps → `Ok` (boundary inclusive).
  - `"from_toml_normal_recipe_still_round_trips"` — a small resize recipe loads,
    `to_toml`/`from_toml` round-trips, and `build_pipeline(with_builtins)` succeeds
    (no regression).
  - `"from_toml_unsupported_version_still_rejected"` — a `version = "2"` recipe is
    still `UnsupportedVersion` (existing behavior intact).
- **`tests/recipe_round_trip.rs` or `tests/apply_batch.rs` (integration)**
  - `"apply_oversized_recipe_file_exits_1"` — write a recipe file `> RECIPE_MAX_BYTES`
    to a tempdir, run `crustyimg apply --recipe big.toml img.png -o out.png`, assert
    exit **1** (the CLI pre-read guard) and a non-empty stderr.
  - `"apply_normal_recipe_still_works"` — a normal recipe file applies successfully
    (exit 0) — regression guard for the pre-read change.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-036` — the caps: `RECIPE_MAX_BYTES = 64 KiB`, `RECIPE_MAX_STEPS = 1024`,
  reject-not-truncate, typed errors, exit 1, enforced at `from_toml` + a CLI
  pre-read guard. **Implement exactly these.**
- `DEC-005` — recipes round-trip through the registry; do NOT change the
  round-trip or `build_pipeline` behavior — these caps are additive gates.
- `DEC-007` — typed, matchable errors; no `unwrap`/`expect`/`panic!` on the new
  non-test paths.

### Constraints that apply

- `untrusted-input-hardening` (blocking) — the recipe half (decode = SPEC-033,
  paths = SPEC-034).
- `no-unwrap-on-recoverable-paths`, `clippy-fmt-clean`, `every-public-fn-tested`.

### Prior related work

- `SPEC-006` (shipped) — `Recipe::{from_toml, build_pipeline}` + the version /
  unknown-op / invalid-param validation these caps sit on top of. Do NOT
  reimplement or change that validation.
- `SPEC-031` (shipped) — `run_apply` (the recipe read + batch consumer); the
  pre-read guard goes here.
- `SPEC-033` (shipped) — the sibling input-resource-limit (decode); same
  reject-with-typed-error posture.

### Out of scope (for this spec specifically)

- **Op-parameter bounds** — e.g. `resize` has no upper dimension cap, so a recipe
  step `resize exact 100000x100000` is an upscale bomb (≈40 GB output). That is an
  op-param-validation concern: it also affects the `resize`/`edit`/`shrink` CLI,
  and `percent` mode needs an **apply-time** check (output depends on input size).
  **Tracked for the STAGE-006 threat-model pass (backlog #5) / a dedicated
  op-bounds spec — NOT here.** (Recorded in DEC-036's Consequences.)
- `#[serde(deny_unknown_fields)]` strict parsing — incompatible with the
  `#[serde(flatten)]` on `RecipeStep.params`; deferred (DEC-036).
- Decode limits (SPEC-033) and path/symlink hardening (SPEC-034) — already shipped.

## Notes for the Implementer

- **One choke point.** Both `apply` and any future recipe loader call
  `Recipe::from_toml`; put the size + step checks there so every path inherits
  them. The CLI metadata guard is a cheap addition so a huge *file* isn't read
  into a `String` in the first place — it uses the SAME exported
  `RECIPE_MAX_BYTES`.
- **Order matters:** size check **before** `toml::from_str` (a test asserts an
  oversized string is rejected without parsing); step check **after** the version
  check (so an over-version recipe is still `UnsupportedVersion`, not
  `TooManySteps`).
- **Boundaries are inclusive** — reject only on `>` (`len() > MAX`,
  `steps.len() > MAX`); a recipe exactly at a cap is accepted (tests pin both).
- Reuse the existing `CliError::Recipe(_) => 1` mapping — the new variants need no
  new `code()` arm (confirm with a unit test).
- The step-count fixture: build the TOML in the test as a `String`
  (`"version = \"1\"\n" + &"[[step]]\nop = \"identity\"\n".repeat(n)`); 1025
  `identity` steps is ~18 KB, comfortably under the 64 KiB byte cap, so the
  step-cap test exercises the step gate, not the size gate.
- Run clippy right after doc comments (the SPEC-031 `doc_lazy_continuation`
  lesson) and **run the lean build** (`cargo build --no-default-features`).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-035-recipe-limits`
- **PR (if applicable):** opened (see PR URL)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none (DEC-036 was written at design; no new decisions from build)
- **Deviations from spec:**
  - none
- **Follow-up work identified:**
  - none beyond what DEC-036 already captures (op-parameter bounds for upscale-bomb, strict TOML parsing)

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing significant. The spec was precise: exact const names, exact error variant shapes, exact ordering (size before parse, steps after version). The note that 1025 identity steps ≈ 18 KB (well under 64 KiB) pre-answered the only potential confusion about which gate fires in the many-steps test.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. The `CliError::Recipe(_) => 1` coverage was called out explicitly; confirmed it already covers the new variants without a new arm.

3. **If you did this task again, what would you do differently?**
   — Nothing material. The spec was self-contained and the build was straightforward additive work with no surprises.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Reading the recipe loader AND the resize op before writing the spec is what
   kept the scope honest: it showed the *functional* validation (version/
   unknown-op/param) already existed — so "security-grade recipe validation" was
   really about **resource bounding** — and it surfaced the genuinely scary
   recipe vector (a `resize exact 100000x100000` upscale bomb) that this spec
   deliberately does NOT fix (op-param bounds, percent needs apply-time checks,
   and it spans the CLI resize commands too). Pinning the load-bearing order
   (size before parse; steps after version) and inclusive boundaries in the spec
   meant the build hit them first try, and the verify gap-hunt confirmed
   `from_toml` is the single choke point. Three-for-three clean Sonnet builds this
   stage at ~$0.45 each.

2. **Does any template, constraint, or decision need updating?**
   — No template/constraint change; DEC-036 records the policy. The recurring
   STAGE-006 pattern is now explicit and worth carrying into the threat-model
   pass: **harden at the single choke point, reuse the existing typed-error +
   exit-code mapping, and have verify run an adversarial bypass-grep** — it both
   proves completeness and keeps surfacing the next item (here, the resize
   upscale-bomb; on SPEC-034, the `--save-recipe` raw write).

3. **Is there a follow-up spec I should write now before I forget?**
   — Two now-accumulated op/path follow-ups belong to the **threat-model
   verification pass (backlog #5)**, not standalone specs yet: (a) **op-parameter
   bounds** — the `resize` upscale bomb (incl. `percent` apply-time) — and (b) the
   **`edit --save-recipe` raw write** symlink-guard parity (from SPEC-034). The
   immediate next item is **backlog #4: wire `cargo audit`/`cargo deny` into CI**
   (a CI/tooling change). Both tracked on the stage backlog.
