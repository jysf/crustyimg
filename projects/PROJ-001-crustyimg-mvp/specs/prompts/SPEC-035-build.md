# SPEC-035 build prompt — recipe resource limits

Start a **fresh session**. You are the IMPLEMENTER for SPEC-035 in the `crustyimg`
repo (cwd is the repo root). The architect (Opus) wrote the spec, its failing tests,
and **DEC-036** (the caps). **No new dependency** — `std` + the existing `toml`. Make
the spec's `## Failing Tests` pass with the smallest correct change, then open a PR and
STOP. Follow this prompt exactly.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-035-security-grade-recipe-validation-and-resource-limits.md`
   — especially `## Limits policy (PINNED)`, `## Failing Tests`, `## Notes for the Implementer`.
2. `decisions/DEC-036` (the caps), `DEC-005`, `DEC-007`.
3. `src/recipe/mod.rs` — `Recipe::from_toml` (the choke point), `RecipeError`,
   `SUPPORTED_VERSION`. `src/cli/mod.rs` — `run_apply` (the `read_to_string` path).

## What to build
- **`src/recipe/mod.rs`:**
  - Add `pub const RECIPE_MAX_BYTES: usize = 64 * 1024;` and
    `pub const RECIPE_MAX_STEPS: usize = 1024;`.
  - Add two `RecipeError` variants:
    `#[error("recipe is too large ({size} bytes; max {max})")] TooLarge { size: usize, max: usize }`
    and `#[error("recipe has too many steps ({count}; max {max})")] TooManySteps { count: usize, max: usize }`.
  - In `Recipe::from_toml(s: &str)`, in THIS order:
    1. `if s.len() > RECIPE_MAX_BYTES { return Err(RecipeError::TooLarge { size: s.len(), max: RECIPE_MAX_BYTES }); }` — BEFORE `toml::from_str`.
    2. parse (existing).
    3. version check (existing).
    4. `if recipe.steps.len() > RECIPE_MAX_STEPS { return Err(RecipeError::TooManySteps { count: recipe.steps.len(), max: RECIPE_MAX_STEPS }); }`
- **`src/cli/mod.rs`** (`run_apply`): before `std::fs::read_to_string(recipe_path)`,
  add: `let meta = std::fs::metadata(recipe_path).map_err(CliError::RecipeIo)?; if
  meta.len() > crate::recipe::RECIPE_MAX_BYTES as u64 { return
  Err(CliError::Recipe(RecipeError::TooLarge { size: meta.len() as usize, max:
  crate::recipe::RECIPE_MAX_BYTES })); }` then the existing read. (Import the const via
  the existing `crate::recipe` path / `use`.)

## Hard rules
- **Smallest correct change.** Do NOT touch the version/unknown-op/invalid-param
  validation, `build_pipeline`, or the round-trip — these caps are ADDITIVE. The new
  variants reuse the existing `CliError::Recipe(_) => 1` mapping (no new `code()` arm).
  **No new dependency.** No `unwrap`/`expect`/`panic!` on the new non-test paths.
- **Order matters:** size check BEFORE parse; step check AFTER the version check.
  **Boundaries inclusive:** reject only on `>` (a recipe exactly at a cap is accepted).
- Build the oversized / many-step fixtures programmatically in the tests as `String`s
  (do NOT commit a 64 KiB file). For the step-cap test, 1025 `identity` steps (~18 KB)
  stays under the byte cap, so it exercises the step gate, not the size gate.
- Every named test in `## Failing Tests` must EXIST and PASS:
  - `src/recipe/mod.rs`: `from_toml_rejects_oversized_recipe`,
    `from_toml_accepts_recipe_at_size_cap`, `from_toml_rejects_too_many_steps`,
    `from_toml_accepts_recipe_at_step_cap`, `from_toml_normal_recipe_still_round_trips`,
    `from_toml_unsupported_version_still_rejected`.
  - integration (`tests/recipe_round_trip.rs` or `tests/apply_batch.rs`):
    `apply_oversized_recipe_file_exits_1`, `apply_normal_recipe_still_works`.
- Run clippy right after writing doc comments (the SPEC-031 `doc_lazy_continuation` lesson).

## Gates (all must pass — INCLUDING the lean build)
```
cargo fmt && git add -u
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --no-default-features
cargo deny check licenses
```

## Git / PR
- Branch `feat/spec-035-recipe-limits` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked `reports/*.md`
  and `TESTING-WITH-YOUR-PHOTOS.md` (do NOT stage them).
- PR title: `feat(recipe): recipe resource limits (SPEC-035)`.
- PR body per AGENTS.md §13 (Decisions referenced — DEC-036, DEC-005, DEC-007 /
  Constraints — `untrusted-input-hardening` / New decisions — "DEC-036 at design").
- Fill the spec's `## Build Completion` + 3 reflection answers; append the build cost
  session (numerics null; agent `claude-sonnet-4-6`).

## Cost
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-19
  notes: "recipe resource limits: RECIPE_MAX_BYTES/RECIPE_MAX_STEPS + RecipeError::TooLarge/TooManySteps enforced in from_toml (size before parse, steps after version) + CLI run_apply pre-read metadata guard; reuse Recipe(_) exit 1; std-only, no new dep"
```

## When done
`just advance-cycle SPEC-035 verify` (if it mis-globs or doesn't update the spec's
`cycle:` field, set `cycle: verify` in the spec frontmatter by hand), open the PR with
`gh`, and **stop** — the orchestrator pauses for the user before any merge.
