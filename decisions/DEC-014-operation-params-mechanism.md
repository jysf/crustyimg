---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-014
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

# Decisions are repo-level, but it's useful to track which project
# caused them to be emitted.
project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-15
supersedes: null
superseded_by: null

affected_scope:
  - src/operation/**
  - src/recipe/**

tags:
  - operations
  - serde
  - params
  - extensibility
---

# DEC-014: `OperationParams` is a generic TOML-map newtype; each Operation owns its param parsing

## Decision

`OperationParams` is a **newtype over an ordered map** —
`OperationParams(BTreeMap<String, toml::Value>)` — with hand-written
`Serialize`/`Deserialize` that delegate to the inner map (empty map →
**zero flattened keys**, so parameterless ops like `invert` stay
zero-extra-keys; non-empty map → collected verbatim). **Each `Operation`
owns its own param parsing and validation in its constructor**, reading
from the map via typed accessors (`get_str`/`get_u32`/`get_f32`) that
return typed errors on missing/wrong-type. Per-op validation no longer
lives in the `Deserialize` impl. The recipe TOML round-trip is preserved
(`Recipe::from_toml(recipe.to_toml()?)? == recipe`).

## Context

SPEC-006 widened `OperationParams` from the SPEC-003 placeholder to a
`None`-only enum with a hand-written serde impl: `None` serialized to an
empty map, and **deserializing a non-empty map for a parameterless op was a
hard error**. That impl had no way to carry *real* parameters — it was a
forward-compatibility stub. SPEC-010 introduces the first parameterized
operation (`resize`, with `mode`/`width`/`height`/`percent` keys), which
forces the mechanism to actually carry typed params now.

The hard constraint is the flatten boundary. `RecipeStep` uses
`#[serde(flatten)] params: OperationParams`, so when serde deserializes a
step it hands `OperationParams` a map of *all* the non-`op` keys in the
`[[step]]` table — **with no knowledge of which `op` they belong to**.
There is therefore no op context available at deserialize time to pick a
typed param variant or validate keys per-op. The op identity is only known
later, at `registry.build(&step.op, &step.params)`. This is why validation
must move out of `Deserialize` and into each op's constructor (which *does*
know the op). The map newtype is the representation that survives the
context-free flatten and still round-trips.

## Alternatives Considered

- **Option A: Typed enum variant per op (`OperationParams::Resize { mode, width, .. }`)**
  - What it is: grow the existing enum with a real `Resize` variant (and one
    per future op), deriving or hand-writing serde over the variants.
  - Why rejected: the flatten-deserialize has **no `op` context** — serde
    can't know which variant a bare `{mode=..,width=..}` map is, so an
    untagged enum would guess (and mis-route overlapping key sets), while a
    tagged enum would require a discriminator key the schema doesn't have
    (the discriminator *is* `op`, which lives on `RecipeStep`, not in
    `params`). It also re-couples the params type to the op catalog —
    exactly what the registry seam (DEC-005) exists to avoid: every new op
    would edit the central enum.

- **Option B: `toml::Value` directly as the `params` field (no newtype)**
  - What it is: `RecipeStep { op, #[serde(flatten)] params: toml::Value }`.
  - Why rejected: a bare flattened `toml::Value` is awkward (it would be a
    `toml::Value::Table` but the type doesn't say so), gives no home for the
    typed accessors/`empty()`/validation helpers, and weakens the
    `Operation::params()` return contract. A thin newtype over the map keeps
    a named type with methods while staying just as generic.

- **Option C (chosen): generic ordered-map newtype + per-op validation in the constructor**
  - What it is: `OperationParams(BTreeMap<String, toml::Value>)`; ops parse
    and validate their own keys in their `Constructor`; the registry surfaces
    a typed `InvalidParams` error.
  - Why selected: survives the context-free flatten boundary, round-trips
    losslessly (empty map ↔ zero keys ↔ `invert`; populated map ↔ resize's
    keys), keeps the params type decoupled from the op catalog (DEC-005), and
    locates validation where the op identity is actually known (DEC-002 — the
    op owns its transform *and* its argument contract). `BTreeMap` gives a
    deterministic key order for stable round-trips.

## Consequences

- **Positive:** First-class parameterized operations with no central
  switch; the params type never changes when an op is added; round-trip is
  preserved and testable; validation errors are typed and attributable to
  the op. The `Resize` constructor is the template every later parameterized
  op (`thumbnail`, `shrink`, `convert`) follows.
- **Negative:** Params are dynamically typed until an op's constructor
  extracts them, so a malformed param surfaces at `build_pipeline` time
  (registry construction), not at `from_toml` parse time. This is the
  correct seam (only the op knows its schema) but it splits "is this valid
  TOML?" from "is this valid *for this op*?" across two calls. The old
  enum's compile-time `None`-ness is gone (it was a stub, not real safety).
- **Neutral:** SPEC-003/SPEC-006 unit tests that asserted
  `OperationParams::None` migrate to `OperationParams::empty()`;
  `Identity`/`Invert::params()` return the empty form. `RecipeStep`/`Recipe`
  serde derives are **unchanged**.

## Validation

Right if: the `docs/data-model.md` resize step (`op="resize"`,
`mode="max"`, `width=1200`, height omitted) round-trips via `Recipe`
`PartialEq`; `invert` stays zero-extra-keys through the same round-trip;
a `resize` step with a bad/missing param fails with a typed
`RecipeError::InvalidOperation` (not `UnknownOperation`); and later
parameterized ops register without touching `OperationParams` or the recipe
parser. Revisit if: the per-op validation boilerplate proves repetitive
enough to warrant a derive/macro, or a future op needs a param shape a flat
key→value map cannot express (nested tables) — then extend the accessor set
deliberately rather than re-coupling the type to the catalog.

## References

- Related specs: SPEC-010 (first consumer — `resize` + this mechanism),
  SPEC-003 (the `OperationParams` placeholder), SPEC-006 (the `None`-only
  enum this replaces)
- Related decisions: DEC-005 (recipe round-trip + registry seam this
  preserves), DEC-002 (operation boundary — the op owns its transform and
  now its param contract), DEC-007 (typed errors — `InvalidParams` /
  `InvalidOperation`)
- Data model: `docs/data-model.md` § "Recipe Schema" (the worked `resize`
  step pins the param-key schema)
- External docs: https://docs.rs/toml (`toml::Value`)
