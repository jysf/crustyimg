---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-036
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-19
supersedes: null
superseded_by: null

affected_scope:
  - src/recipe/mod.rs
  - src/cli/mod.rs

tags:
  - security
  - hardening
  - recipe
  - untrusted-input
  - resource-limits
---

# DEC-036: recipe resource limits (size + step caps)

## Decision

Bound an untrusted recipe's resource footprint at the single recipe load choke
point (`Recipe::from_toml`, STAGE-006), on top of the existing version +
unknown-operation + invalid-param validation (SPEC-006):

- **Recipe text size cap:** `RECIPE_MAX_BYTES = 64 KiB`. `Recipe::from_toml`
  rejects input whose length exceeds the cap **before parsing** (typed
  `RecipeError::TooLarge`), so a giant recipe can't drive a parse-time
  memory/CPU blowup. Recipes are tiny (a few hundred bytes); 64 KiB is generous
  (thousands of steps) yet bounds the parser.
- **Step count cap:** `RECIPE_MAX_STEPS = 1024`. After parsing, a recipe with
  more steps is rejected (`RecipeError::TooManySteps`), so a recipe can't build a
  pathologically long pipeline.
- **CLI pre-read guard:** `run_apply` checks the recipe **file size**
  (`fs::metadata().len()`) against `RECIPE_MAX_BYTES` *before*
  `read_to_string`, so a multi-GB file is never read into memory in the first
  place (same `RecipeError::TooLarge`, exit 1).

Both new errors are typed `RecipeError` variants and map to the existing CLI
exit **1** (no new exit code). `RECIPE_MAX_BYTES` is exported from the recipe
module so the CLI shares the one constant. No new dependency.

This is the recipe analog of the decode resource limits (DEC-034): reject an
untrusted artifact that would exhaust resources, with a typed error rather than
a crash/OOM.

## Context

STAGE-006 backlog item #3 is "security-grade recipe validation." The functional
baseline already rejects an unsupported `version` and unknown operations with
typed errors (SPEC-006: `Recipe::{from_toml, build_pipeline}`), and rejects
invalid op params. The residual untrusted-recipe risk is **resource
exhaustion**: a huge recipe file (parse DoS) or a recipe with an enormous number
of steps (pipeline-build DoS). These two caps close that, at the same single
load point every caller (`apply`) already funnels through.

## Alternatives Considered

- **Rely on the size cap alone (skip the explicit step cap)** — the 64 KiB size
  cap already bounds steps (~4 K max), but an explicit `TooManySteps` is clearer,
  cheap, and documents the pipeline-length invariant. Keep both.
- **`#[serde(deny_unknown_fields)]` for strict parsing** — deferred: it is
  incompatible with the `#[serde(flatten)]` on `RecipeStep.params`, and the
  resource caps are the higher-value hardening. The threat-model pass (backlog
  #5) can revisit strict fields.
- **No CLI pre-read guard (size-check only inside `from_toml`)** — rejected:
  `from_toml` runs *after* `read_to_string` has already pulled the whole file
  into a `String`; the metadata pre-check avoids reading a multi-GB file at all.
- **A dedicated exit code for resource-limit rejection** — rejected; reuses the
  recipe error → exit 1 mapping (consistent with DEC-034's choice).

## Consequences

- **Positive:** An untrusted recipe (huge file or huge step list) is refused
  with a typed error, never an OOM/hang, on the one shared load path. Cheap,
  `std`-only, no new dependency.
- **Negative:** A *legitimately* enormous recipe (> 64 KiB or > 1024 steps) is
  refused — not a realistic case (recipes are op chains of a handful of steps).
- **Neutral / known gap:** This does NOT bound an individual op's *parameters* —
  e.g. `resize` has no upper dimension bound, so a recipe step `resize
  exact 100000x100000` is an upscale bomb. That is an op-param-validation concern
  (it also affects the `resize`/`edit`/`shrink` CLI, and percent mode needs an
  apply-time check), tracked separately for the STAGE-006 threat-model pass /
  a dedicated op-bounds spec — NOT this DEC.

## Validation

Right if: a recipe file > 64 KiB and a recipe with > 1024 steps are both rejected
with their typed `RecipeError` (exit 1), the file-size pre-read guard fires
before `read_to_string`, and every normal recipe loads unchanged (SPEC-035
tests). Revisit if: a real recipe legitimately needs more headroom (raise the
caps), or strict-field parsing / op-param bounds are added.

## References

- Related specs: SPEC-035 (this hardening); SPEC-006 (the recipe loader +
  version/unknown-op validation it extends); SPEC-031 (`apply --recipe`, the
  consumer)
- Related decisions: DEC-034 (decode resource limits — the sibling input bound),
  DEC-005 (recipe round-trip via the registry), DEC-007 (typed errors)
- Constraints: `untrusted-input-hardening` (blocking), `no-unwrap-on-recoverable-paths`
