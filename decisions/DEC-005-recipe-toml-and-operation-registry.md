---
insight:
  id: DEC-005
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

created_at: 2026-06-13
supersedes: null
superseded_by: null

affected_scope:
  - src/recipe/**
  - src/operation/**

tags:
  - recipes
  - serde
  - registry
  - extensibility
---

# DEC-005: Recipe format is TOML (serde); operations resolve through a registry

## Decision

A **recipe** is a versioned, ordered list of operation steps serialized as
**TOML** via `serde`. Each step is `op = "<name>"` plus that operation's
params. An **operation registry** maps `name -> constructor(params)`, and
**both** the CLI and the recipe loader construct operations through it. This
guarantees recipes **round-trip**: a recipe serialized from a run
deserializes back into the identical ordered operation list.

## Context

The thesis ("tune once, replay across many") requires a serializable
representation of the operation chain that survives save/load exactly. The
operation set grows every stage, so adding an op must not require editing
the recipe parser — the registry is the single seam new ops register at
(feature-exploration.md § "Workflow model" / "Decisions to formalize" #4).

## Alternatives Considered

- **Option A: YAML or JSON recipes**
  - Why rejected: TOML is the Rust ecosystem default, is the friendliest to
    hand-edit for an ordered list of tables (`[[step]]`), and matches
    `Cargo.toml` familiarity. JSON is noisy to hand-write; YAML's
    whitespace footguns aren't worth it. (A future format adapter is
    possible but not MVP.)

- **Option B: No registry — match op names in a giant `match` in the parser**
  - Why rejected: every new op edits the parser, coupling recipe code to the
    op catalog. The registry decouples them.

- **Option C (chosen): TOML + serde + operation registry, with a version field**
  - Why selected: ergonomic, idiomatic, round-trips cleanly, and additive
    for new operations. The `version` field lets the loader reject
    incompatible recipes instead of misreading them.

## Consequences

- **Positive:** Recipes are readable and hand-editable; new operations are
  additive; round-trip is testable in STAGE-001 before any real op exists.
- **Negative:** Each op must define serde-serializable params and register
  itself; forgetting to register makes a recipe fail to load (mitigated: a
  test asserts every registered op round-trips). Params typed as a generic
  value lose some compile-time checking until constructed.
- **Neutral:** Recipe `version` starts at `"1"`; bumping is a deliberate
  breaking-change signal.

## Validation

Right if: SPEC-005's round-trip test passes (serialize → file → parse →
identical op list), and later stages add operations without touching the
recipe parser. Revisit if: users demand a second recipe format, or params
typing proves too loose (then introduce per-op typed param enums behind the
registry).

## References

- Related specs: SPEC-003 (Operation trait), SPEC-005 (recipe + registry round-trip)
- Related decisions: DEC-002 (Operation trait this serializes)
- External docs: https://docs.rs/serde, https://docs.rs/toml
- Data model: `docs/data-model.md` § "Recipe Schema"
