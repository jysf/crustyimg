---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-127
  type: story                      # epic | story | task | bug | chore
  cycle: design                    # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-050
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5     # usually same Claude, different session
  created_at: 2026-09-04

references:
  decisions:
    - DEC-005
    - DEC-015
    - DEC-087
    - DEC-098
    - DEC-058
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
    - every-public-fn-tested
    - ergonomic-defaults
  related_specs:
    - SPEC-126
    - SPEC-111

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-050's <capability>". Optional; null is acceptable.
value_link: >
  STAGE-050's thesis item. A recipe that cannot name its own output format is
  not a portable description of a job — the caller has to supply half of it.
  SPEC-126 made `apply` and `build` agree on what a format means, which is the
  precondition for the schema naming one.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-127: recipes carry format and quality

## Context

A recipe describes a job, and today it can describe every part of that job **except what the
output should be**. `Recipe` carries `version`, `name`, `description` and `steps` — the pixel
operations — and nothing about the encode. So the same recipe run two ways produces two different
files, and the caller has to supply the missing half every time:

```
crustyimg apply --recipe web.toml hero.png -o out.jpg --format jpeg -q 82
                └─ the portable part ─┘   └──── the part that is not ────┘
```

⛔ **This was gated on STAGE-049 and is now unblocked.** SPEC-126 made `apply` and `build` resolve
output format by one rule (`--format` > `-o` ext > preserve source, DEC-015/DEC-098). Before that,
adding `format` to the schema would have baked a disagreement into it: the two paths would have
read the same field and produced different bytes.

**Driven on `main` at `7181eed` (0.7.1 + SPEC-126), with a valid recipe emitted by
`edit --save-recipe` rather than hand-written:**

| recipe | exit | message |
|---|---|---|
| valid `version = "1"` (control) | **0** | — |
| v1 **plus an unknown `format` key** | **1** | `could not parse recipe TOML: TOML parse error at line 2, column 1` |
| `version = "2"`, no unknown key | **1** | `unsupported recipe version '2' (supported: 1)` |

`Recipe` is `#[serde(deny_unknown_fields)]`, so **today's binary rejects any recipe carrying a
`format` key, and the error it gives is a TOML parse failure pointing at line 2** — which tells a
user nothing about needing a newer crustyimg. The version check, given a clean recipe, produces
exactly the right message. That asymmetry is measured, and it decides Call 1 below.

⚠ **Read the exit codes from `$?` after a redirect.** A first pass at this table piped through
`head` and reported `exit=0` for all three rows.

## Goal

`Recipe` can name its own `format` and `quality`; every path that runs a recipe honours them by
one precedence rule; and a recipe that uses them is rejected by older binaries with a message
that says so.

## Inputs

- `src/recipe/mod.rs` — `Recipe`, `RecipeStep`, `SUPPORTED_VERSION`, `from_toml`/`to_toml`,
  `split_terminal_optimize`.
- `src/cli/common.rs` — `encode_one(recipe, registry, input, format_override, quality)`, the seam
  every recipe-running path already funnels through.
- `src/cli/optimize.rs` — `run_apply`; `src/cli/build.rs` — the manifest path.
- `src/wasm.rs:171` — `transform(input, recipe_toml, out_format)`.
- DEC-015 (format precedence), DEC-098 (`apply` preserves at every arity), DEC-087 (a name
  template's literal extension names the file, it does not pin the format), DEC-005 (recipes
  round-trip through the registry), DEC-058 (the build cache key).

## Outputs

- `Recipe.format: Option<String>` and `Recipe.quality: Option<u8>`, round-tripping losslessly.
- `SUPPORTED_VERSION` accepts both `"1"` and `"2"`; a recipe using either new field must declare
  `version = "2"`.
- One precedence rule, applied by `apply`, `build` and `wasm::transform` alike.
- DEC-099 recording the calls below.

## The design calls — settled here

### Call 1 — a recipe using `format` or `quality` MUST declare `version = "2"`

Settled by the table in Context, not by preference. The naive answer is an optional field on v1:
`skip_serializing_if = "Option::is_none"` means existing recipes serialize unchanged and nothing
breaks. **That answer is wrong, and the measurement is why.** `deny_unknown_fields` means an older
binary handed a v1-plus-`format` recipe fails with a **TOML parse error at line 2**. The same
binary handed `version = "2"` says *"unsupported recipe version '2' (supported: 1)"* — the actual
problem, in the user's language.

crustyimg is a published crate with two shipped releases in the wild. A recipe is a file people
commit and share, so a forward recipe **will** meet an old binary. Gate the new fields behind the
version and that meeting produces a sentence someone can act on.

⚠ **`"1"` stays valid and unchanged.** This is not a migration: v1 recipes parse, run and
round-trip exactly as they do today, and `to_toml` must keep emitting `version = "1"` for a recipe
that uses neither new field. **Emitting `"2"` unconditionally would strand every existing recipe
on the next `--save-recipe`.**

### Call 2 — precedence: the CLI wins, the recipe beats the source, and `-o` sits between

Extend DEC-015's chain rather than inventing a second one:

```
--format (explicit CLI)  >  -o extension  >  recipe.format  >  preserve the source
```

The reasoning is DEC-098's, applied one level down: **the more specific and more immediate wins.**
A flag typed at the call site is a deliberate override of a file's default; a recipe's `format` is
a default the file carries. `-o hero.png` stays above the recipe because DEC-087 already rules a
recognised `-o` extension a pin, and reopening that here would change `web`/`optimize` behaviour
this spec has no business touching.

`quality` takes the same shape: `-q` > `recipe.quality` > the format's own default.

⚠ **The terminal `optimize` step is the carve-out.** A recipe ending in the reserved `optimize`
marker runs the auto-decide engine, which *chooses* a format and *searches* for a quality. A
recipe that both ends in `optimize` and declares `format`/`quality` is asking for two contradictory
things. **Rule: the explicit field wins and the decision is skipped** — that is already what a
`--format` pin does on that path (DEC-087), so this keeps one behaviour rather than adding a
second. It must be **tested**, not assumed.

### Call 3 — `wasm::transform` grows a source of truth, so make the parameter the override

`transform(input, recipe_toml, out_format)` takes the format as its own argument and parses it
itself (`parse_format` rejects `"auto"` and empty, so it is always a concrete pinned format). Give
the recipe a `format` and there are two answers to one question.

**Rule: `out_format` is the CLI-flag equivalent and wins; `recipe.format` is the fallback.** That
makes the wasm surface obey the same sentence as the native one, which is the whole point of
having a precedence rule at all.

📌 STAGE-053 raised this and correctly said it needs deciding **whether or not** the wasm CI leg
ever gets built. It is decided here. ⚠ But note what that means for evidence: `just wasm-test`
runs in no CI job today, so **a wasm assertion added by this spec is not covered by the required
matrix.** Say so in Build Completion rather than implying the leg is guarded.

### Call 4 — typed per-operation param structs are NOT in this spec

STAGE-050's backlog entry bundles "plus typed per-operation parameter structs" with this work.
**Split.** `OperationParams` is a `BTreeMap<String, toml::Value>` with hand-rolled `get_str` /
`get_u32` / `get_f32`, and typing it touches every operation in the registry — a refactor with its
own blast radius, its own round-trip risk under DEC-005, and no dependency on this schema change
in either direction. Bundling turns an **M** into an **L**, and AGENTS §2 says an L should be
split.

Filed back to STAGE-050 as its own item by this spec's design cycle.

## Acceptance Criteria

- [ ] **AC-1.** A recipe declaring `version = "2"` with `format` and `quality` **round-trips
      losslessly** — `from_toml(to_toml(r)) == r` — and a v1 recipe using neither field still
      round-trips and still serialises as `version = "1"`.
- [ ] **AC-2.** `apply` honours `recipe.format` at **1 input and at N inputs**, identically —
      driven for at least two target formats, asserted on the written **bytes** via
      `image::guess_format`, never the filename extension (SPEC-126, Call 4).
- [ ] **AC-3.** **Precedence holds in both directions:** `--format` overrides `recipe.format`, and
      with no `--format` the recipe's value beats preserve-source. Same for `-q` vs
      `recipe.quality`.
- [ ] **AC-4.** **`apply` and `build` produce byte-identical output** for the same v2 recipe and
      input — the SPEC-126 property, re-asserted against the new field. This is the one that
      catches a `build` path that reads the recipe differently.
- [ ] **AC-5.** A recipe using `format` or `quality` **without** `version = "2"` is rejected with
      a typed error, and a `version = "2"` recipe is rejected by a binary built from `main` before
      this spec with **`unsupported recipe version`** — the message, not just a non-zero exit.
- [ ] **AC-6.** The terminal `optimize` carve-out (Call 2): a bundled recipe with an explicit
      `format` **skips the auto-decision and honours the pin**; the same recipe without one still
      auto-decides and its output is **byte-identical to `main`**.
- [ ] **AC-7.** **A negative control, one revert per independent condition** (AGENTS §15). Call 1
      (version gate), Call 2 (precedence) and Call 3 (wasm) are independent; reverting each must
      flip only the tests that exercise it. The evidence is the **behavioural flip**, never a hash.
- [ ] **AC-8.** **Nothing else changes bytes.** `resize`, `thumbnail`, `watermark`, `optimize`,
      `web`, `convert`, `responsive` and a **v1** recipe through `apply`/`build` all produce
      byte-identical output to `main` on the corpus. ⚠ State the corpus boundary in the DEC —
      SPEC-126's verify had to add that.
- [ ] **AC-9.** Clean full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequential: default,
      `--no-default-features`, `--features webp-lossy`. Clippy and `fmt --check` each. Plus
      `just wasm-check` — Call 3 touches an engine module.

## Failing Tests

Written during design, made to pass during build (constraint `test-before-implementation`). All
must be confirmed **RED against `main`** before any implementation, and the baseline recorded.

- `tests/recipe_v2.rs::v2_round_trips_format_and_quality` — AC-1.
- `tests/recipe_v2.rs::v1_still_round_trips_and_stays_v1` — AC-1's other half; the strand guard.
- `tests/recipe_v2.rs::new_field_without_v2_is_rejected` — AC-5.
- `tests/apply_batch.rs::apply_honours_recipe_format_at_every_arity` — AC-2. **Two tests, one per
  arity**, per SPEC-126's re-approve finding: a combined test dies at the first failing arity and
  the suite cannot say which regressed.
- `tests/apply_batch.rs::cli_format_overrides_recipe_format` — AC-3.
- `tests/apply_batch.rs::apply_and_build_agree_on_v2_recipe` — AC-4.
- `tests/optimize.rs::terminal_optimize_honours_an_explicit_recipe_format` — AC-6.

## Implementation Context

**Read before writing code:** this section, then DEC-015, DEC-087, DEC-098, DEC-005, DEC-058, the
parent `STAGE-050-recipe-reach.md`, the project `brief.md`, and `/guidance/constraints.yaml`.

**Where the seams are.** `encode_one` in `src/cli/common.rs` already takes
`format_override: Option<ImageFormat>` and `quality: Option<u8>` and is the funnel **both** `apply`
and `build` reach the encoder through — SPEC-126 confirmed `build.rs` calls it directly with an
unchanged signature. So the recipe's values should be resolved into those two parameters by each
caller, **not** read inside `encode_one`: that keeps the precedence decision at the call site where
the CLI flags also live, and it is why `build` cannot silently diverge.

`ops::output_format_for` (`src/cli/ops.rs`, `pub(super)` since SPEC-126) is the existing
`--format` > `-o` ext > source resolution. **Call 2 inserts `recipe.format` as a new lowest-but-one
rung.** Widen that function rather than reimplementing the chain beside it — reimplementation is
exactly what SPEC-126 avoided and the reason `apply` and `build` now agree.

**The version gate.** `SUPPORTED_VERSION` is a single `&str`. It becomes a set, and `from_toml`
grows a rule: the new fields require `"2"`. `to_toml` must emit `"1"` when neither field is set —
that is `v1_still_round_trips_and_stays_v1`, and it is the difference between an additive change
and stranding every recipe in the wild.

**The wasm target.** `src/recipe/` is an **engine** module (CLAUDE.md, DEC-064): it compiles for
`wasm32-unknown-unknown`. No `std::fs`, no `clap`. Run `just wasm-check`.

**Cost.** Follow `projects/_templates/prompts/cost-snippet.md`. Build and verify are metered
cycles and must not be left null (AGENTS §4, DEC-083 — price per component, never a flat rate).

**⛔ Byte-changing on a shipped verb? Only for recipes that opt in.** A v1 recipe must produce
identical bytes, which is AC-8. But the surface changes, so this still batches into PROJ-011's
single lockfile migration with the rest of STAGE-050 — **do not bump the version, do not cut a
release.**

## Notes for the Implementer

- 📌 **DEC-099 is reserved for this spec.** `next_id` scans only the working tree, so a record on
  an unmerged branch is invisible and the id can collide. Highest on `main` is DEC-098.
- ⚡ **Write `docs/api-contract.md` in the same change, not after.** SPEC-126 shipped without it
  and verify caught it; DEC-015's own `affected_scope` names that file, and the decisions audit
  **cannot** tell you so — it silently drops inline-array scopes (filed, PROJ-013 STAGE-047).
  The recipe section and the `apply`/`build` entries all describe format resolution.
- ⚠ **`cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal.** Redirect
  stdout; do not "fix" it. And a piped command reports the **pipe's** exit code — redirect and
  read `$?`. This spec's own Context table got that wrong on the first pass.
- **Never poll CI.** Background `gh pr checks --watch` and read a direct snapshot at the true head
  SHA when it exits.
- **Budget ~150 exchanges**, and push a WIP commit once it compiles, before the matrix.
