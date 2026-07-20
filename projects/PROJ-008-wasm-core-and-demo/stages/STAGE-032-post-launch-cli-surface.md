---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-032                     # stable, zero-padded within the project
  status: proposed                  # proposed | active | shipped | cancelled | on_hold
  priority: low                     # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-008                      # parent project
repo:
  id: crustyimg

created_at: 2026-07-20
shipped_at: null

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

# STAGE-032: post-launch CLI surface enhancements

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
- STAGE-030 (shipped 2026-07-20) — the frozen ~14-verb surface and recipe registry this stage extends.

### Enables
- Nothing blocks on this; it is pure post-launch convenience.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
