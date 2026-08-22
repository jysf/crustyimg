---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-037                     # stable, zero-padded within the project
  status: on_hold                   # proposed | active | shipped | cancelled | on_hold
  # PARKED 2026-08-10, by its own criterion rather than a new one. This stage has always
  # said "pull it when there's an adoption signal or a maintainer decision to broaden the
  # surface — not on the launch clock." 0.7.0 has shipped and no adoption signal exists
  # yet, so the condition is unchanged and the honest status is on_hold rather than
  # proposed — `proposed` reads as queued work. Revive when someone other than the
  # maintainer asks for `convert --to` or for more bundled recipes.
  priority: low                     # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-010                      # parent project
repo:
  id: crustyimg

created_at: 2026-07-20
shipped_at: null

# Re-homed 2026-07-26 from PROJ-008 STAGE-032, where it was framed but never
# started. Content unchanged; the ID and parent project moved. No spec shipped
# under the old number, so nothing is split across projects.

value_contribution:
  advances: >
    Adds convenience on top of the frozen CLI surface without reopening it — post-launch quality-of-life
    verbs and recipes that the STAGE-030 freeze deliberately left out so 1.0 could ship on a stable core.
  delivers:
    - "A `convert --to <fmt>` convenience surface (explicit one-shot format conversion), alongside the
      existing engine/`web`/`optimize` verbs."
    - "Additional bundled recipes (social/archive presets) on top of the shipped web/gallery/product set."
  explicitly_does_not:
    - "Change, rename, or remove any of the ~14 verbs frozen in STAGE-030 — this is additive only."
    - "Add a new codec, engine capability, backend/service, or ML — the PROJ-008 territory guardrails stand."
    - "Block the launch — this stage is post-launch, pulled only when there's a reason to."
---

# STAGE-037: post-launch CLI surface enhancements

## What This Stage Is

The home for additive CLI conveniences that STAGE-030's surface freeze deliberately deferred. The
taxonomy froze at ~14 one-intent verbs so 1.0 could launch on a surface with no dependents but the
maintainer and no relaunch risk. This stage holds the optional-but-nice surface work that can land
*after* launch without touching that frozen core: an explicit `convert --to` conversion verb and extra
bundled recipes (social/archive presets). It is strictly additive — nothing here renames, removes, or
re-specs a frozen verb.

## Why Now

Not now — **proposed and deferred by design.** STAGE-030 shipped SPEC-092 out of scope because a
convenience rename plus extra recipes are exactly the kind of surface that should not gate a launch or
churn a just-frozen CLI. Captured here as a real stage (rather than an out-of-scope brief bullet) so the
work has a spec backlog to grow into when pulled. Pull it when there's an adoption signal or a maintainer
decision to broaden the surface — not on the launch clock.

## Success Criteria

- `convert --to <fmt>` exists as an explicit one-shot conversion, consistent with the frozen verb set,
  with no change to any existing verb's behavior or output (byte-identity for unchanged paths).
- Any new bundled recipes follow the shipped recipe registry conventions (file-path-wins precedence,
  plain behavior-first headers per the recipe-header guard, SPEC-096).
- All gates green (native + `--features avif` + lean).

## Scope

### In scope
- The `convert --to` convenience verb; additional social/archive bundled recipes; the in-repo doc/test
  updates those require.

### Explicitly out of scope
- Any change to the STAGE-030 frozen verbs; new codecs/engine features/backends/ML; anything that would
  force a CLI relaunch.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [ ] (not yet written) — [M] ⚡ **`watermark` cannot be expressed in a recipe, and neither can the
  output format. Recipe coverage measured on `main` at `92a60b7`, 2026-08-21.** **Maintainer decided
  2026-08-21 that watermark must be recipe-expressible** — which is also this stage's own unpark
  criterion (*"a maintainer decision to broaden the surface"*), so **the `on_hold` status above is
  now stale and wants a ruling.**

  **What a recipe can express — the whole list.** The built-in registry
  (`src/operation/registry.rs:80-83`) holds **four** constructors: `identity`, `invert`, `resize`,
  `auto-orient`. A fifth name, `optimize`, works in the bundled recipes but is **not a registered
  operation** — it is a terminal marker (`OPTIMIZE_STEP_OP`, `src/recipe/mod.rs:302`) special-cased
  by the recipe machinery. Driven: every other name returns `error: unknown operation '<x>'`.

  | CLI capability | in a recipe? |
  |---|---|
  | `resize`, `auto-orient`, `edit --invert` | ✅ |
  | `optimize` (and `web`, as resize + the marker) | ✅ via the terminal marker |
  | **`watermark`** — image/text, font, size, colour, gravity, opacity, scale, margin, tile | ❌ **the whole verb, ~10 params** |
  | **output format and quality** | ❌ **structural — see below** |
  | `thumbnail --size --square` | ❌ no `thumbnail` op; `--square`'s crop semantics may not reduce to a `resize` mode |
  | `meta` strip / clean / set | ❌ container lane (DEC-003) — possibly by design, but a recipe cannot say "strip GPS" |
  | `responsive` | ❌ output multiplicity, not an operation |

  ⚠ **The output-side gap is the one nobody asked about and it is structural.** `Recipe`
  (`src/recipe/mod.rs:176`) has exactly four fields — `version`, `name`, `description`, `steps`.
  **There is no format field and no quality field.** So `convert --format webp -q 80` is not
  expressible as a recipe at all; the only way a recipe reaches an encoder decision is the
  `optimize` marker, which *chooses* the format automatically. A user who wants "always WebP at
  q80, applied to a directory" cannot write that recipe today. **This is a bigger hole than
  watermark and should be scoped with it, not after it.**

  📌 **Only `edit` has `--save-recipe`** (driven across all 12 verbs). So the one path that
  *emits* a recipe covers only the three ops it can perform — which is why the gap has stayed
  invisible: nothing that can save a recipe can do anything a recipe cannot express.

  ⚠ Not a gap, but a documentation error found while checking: **AGENTS.md:452's glossary defines
  Gravity as anchoring "a watermark or crop region"** — there is **no crop capability** anywhere in
  the CLI (`crop` is not a verb, not an `edit` flag, and not a registry op). The glossary describes
  something that does not exist.

- [ ] SPEC-092 (deferred from STAGE-030 2026-07-20) — `convert --to` rename + social/archive recipes.
  Optional convenience surface; additive on top of the frozen ~14-verb core.

**Count:** 0 shipped / 0 active / 1 pending (SPEC-092)

## Design Notes

- **Additive-only discipline.** Unlike STAGE-030 (a hard cutover), this stage may not rename or remove a
  shipped verb — the surface is frozen. New verbs/recipes sit alongside the existing set.
- Recipe headers must stay plain and behavior-first (enforced by `bundled_recipe_headers_are_plain`,
  SPEC-096).

## Dependencies

### Depends on
- STAGE-030 (PROJ-008, shipped 2026-07-20) — the frozen ~14-verb surface and recipe registry this
  stage extends.

### Enables
- Nothing blocks on this; it is pure post-launch convenience.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
