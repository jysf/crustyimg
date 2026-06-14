---
insight:
  id: DEC-007
  type: decision
  confidence: 0.9
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
  - src/error.rs
  - src/**
  - src/main.rs

tags:
  - error-handling
  - conventions
---

# DEC-007: Typed library errors (`thiserror`) + `anyhow` at the binary boundary

## Decision

The **library** (everything under the crate's modules) returns typed error
enums via `thiserror` — e.g. `ImageError`, `RecipeError`, `MetadataError`,
unified under a crate `Error`. The **binary** (`main.rs` / `cli`) uses
`anyhow` to add human context and maps the typed errors to the documented
**exit codes** (see `docs/api-contract.md`). The library does **not** depend
on `anyhow`.

## Context

The prototype used `anyhow` everywhere, including `.expect("File should
open")` panics in library-ish code — opaque to callers and crash-prone.
A reusable library wants typed, matchable errors; a CLI wants friendly
messages and meaningful exit codes. Splitting the two is the idiomatic Rust
pattern.

## Alternatives Considered

- **Option A: `anyhow` everywhere (prototype)**
  - Why rejected: callers can't match on error kinds; encourages `.unwrap()`
    /`.expect()` panics; can't map cleanly to exit codes.

- **Option B: hand-rolled error enums, no helper crate**
  - Why rejected: lots of boilerplate `From`/`Display` impls `thiserror`
    generates for free.

- **Option C (chosen): `thiserror` in lib, `anyhow` in bin**
  - Why selected: matchable typed errors for the library, ergonomic context
    + exit-code mapping at the boundary, no panics on expected failures.

## Consequences

- **Positive:** Library errors are matchable and stable; exit codes are
  derivable from error kinds; no `unwrap`/`expect` on recoverable paths.
- **Negative:** A small amount of mapping glue at the binary boundary.
- **Neutral:** Tests assert on typed error variants, not string messages.

## Validation

Right if: every documented exit code (1–6) maps from a distinct error kind,
and no `unwrap`/`expect`/`panic!` appears on recoverable paths (enforced by
the `no-unwrap-on-recoverable-paths` constraint + clippy). Revisit if: the
error enum sprawls — then split per-module errors more aggressively.

## References

- Related specs: SPEC-002 (ImageError), SPEC-005 (RecipeError), SPEC-006 (exit-code mapping)
- Related decisions: DEC-002
- External docs: https://docs.rs/thiserror, https://docs.rs/anyhow
- CLI contract: `docs/api-contract.md` § "Exit Codes"
