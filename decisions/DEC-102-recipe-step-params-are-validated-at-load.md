---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-102
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-10-25
supersedes: null
superseded_by: null

affected_scope:
  - src/operation/registry.rs
  - src/operation/mod.rs
  - src/recipe/mod.rs
  - docs/data-model.md

tags:
  - recipe
  - registry
  - validation
  - untrusted-input-hardening
  - dec-061
  - dec-014
---

# DEC-102: Recipe step params are validated at load against a schema each op publishes through the registry

## Decision

Every operation publishes the step keys it accepts — a `&'static [ParamSpec]` table of
`(key, ParamKind)` — when it registers, and `OperationRegistry::build` checks a step's map against
that table **before** the op's constructor runs: a key the op does not take, a value of the wrong
TOML type, or a value out of range (every number must be finite, also as the `f32` the ops read)
is refused with one line naming the key. The constructors keep only the rules that depend on other
keys — required keys per mode, `image` XOR `text`, keys the step's mode never reads, and `color`'s
hex syntax. The terminal `optimize` marker, which the registry never sees, refuses any key in
`Recipe::from_toml`. A refused step surfaces as `RecipeError::InvalidOperation { step, name,
reason }`, displayed as `recipe step {step} ('{name}'): {reason}` with steps numbered from 0; exit
code 1 is unchanged on `apply` and `build`.

This **supersedes DEC-061's accepted risk #1** ("Recipe step params stay tolerant") and its
matching "`deny_unknown_fields`-equivalent for recipe step params" alternative: an unknown step key
is now refused at load. The rest of DEC-061 stands. It also makes DEC-014's sentence that the
accessors "return typed errors on missing/wrong-type" true — at the registry, not in the
`Option`-returning accessors, which now only ever see an absent or a valid value.

The schema is the **read side** of the seam, not a new storage form: `OperationParams` stays the
ordered `BTreeMap<String, toml::Value>` it is (DEC-014), checking never rewrites it, and so
`to_toml()` — and the cache key and lockfile hash built from it (DEC-058, DEC-101) — is
byte-identical for every recipe that still loads. `register(name, ctor)` now means "takes no
keys"; `register_with_assets(name, ctor, keys)` is shorthand for a table of those keys as
`ParamKind::Asset`; `register_with_params(name, ctor, table)` is the general form; `param_specs`
returns the table; `asset_keys` is **derived** from it (every `Asset` key, in table order), so the
asset seam of DEC-100 and strictness come from one declaration. Adding a recipe key is one line in
the op's table (`RESIZE_PARAMS`, `WATERMARK_PARAMS` in `src/operation/mod.rs`) plus the
constructor's read.

## Context

Measured on `main` before the change (`crustyimg 0.7.1`): an unknown or misspelt key on any step
was ignored at exit 0; a wrong-typed **optional** watermark key (`opacity = "0.5"`, `margin = 5.0`,
`tile = "yes"`, …) silently took its default at exit 0, because `get_*` returns `None` for absent
and wrong-typed alike and the constructor did `.unwrap_or(default)`; `margin = -5` became 0 through
`u32::try_from(..).ok()`; `size = nan|inf` dropped the watermark entirely and `percent = nan` wrote
1×1 images, both at exit 0; `percent = inf` failed per input (exit 6); a key the mode never reads
(`height` under `mode = "max"`, `size` on an image watermark) was ignored; and errors that did fire
named neither the step nor the offending key. The watermark recipe surface (DEC-100) was merged but
unreleased, so fixing it before 0.8.0 means no user meets the silent-default path.

## Alternatives Considered

- **Typed per-op structs as the stored and serialized form.**
  - Why rejected: re-emitting from structs changes `to_toml()` bytes (integer-for-float,
    default emission, key order), which re-pins the cache key and every lockfile line, for no
    user-visible gain over validating the map in place.
- **Hand-written key checks in each constructor, with no published table.**
  - Why rejected: nothing for a future wasm `validate()` or the docs to read, and a new op could
    forget it; the registry check cannot be forgotten.
- **Validating inside `Recipe::from_toml`.**
  - Why rejected: it has no registry, and a custom registry (tests, future ops) would lose its
    ops. Only the terminal marker, which is recipe syntax rather than an op, is checked there.
- **Warning instead of refusing.** Rejected by the maintainer: a warning is ignored in CI.

## Consequences

- A recipe that carried an unknown, wrong-typed or mode-unused key used to load and build; it is
  now refused. This is the one intended break, recorded under **Breaking** in the CHANGELOG, and
  it lands inside 0.8.0's single lockfile migration. No `CACHE_SCHEMA_VERSION` change.
- `OperationRegistry::asset_keys` returns `&[&'static str]` borrowed from the registry (was
  `&'static`); every caller copies the key out, so none changed.
- `Resize::from_params` / `Watermark::from_params` run the same table check themselves, so a
  direct caller gets the registry's guarantees.
- An upper bound on watermark `size` is **not** set here (a resource-limit question in DEC-036's
  family); nor a "did you mean…?" suggestion (the `expected one of` list shows the spelling).

## Revisit if

- An op needs a range that depends on another key (it belongs in the constructor, not the table),
  or a key whose type depends on the mode — `ParamKind` would need a richer form.
- A wasm `validate()` export lands and wants structured errors rather than one line.
