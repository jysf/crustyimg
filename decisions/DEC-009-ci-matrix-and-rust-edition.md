---
insight:
  id: DEC-009
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent
    - operator

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
  - .github/workflows/**
  - Cargo.toml

tags:
  - ci
  - toolchain
  - edition
---

# DEC-009: Rust edition 2021, stable toolchain, three-OS CI matrix

## Decision

Target Rust **edition 2021** on the **stable** toolchain. CI runs on
**Linux, macOS, and Windows** and must pass `cargo build`, `cargo test`,
`cargo clippy -- -D warnings`, and `cargo fmt --check` on each. The default
(pure-Rust) feature set is the matrix; one extra job builds with native
features (`mozjpeg`) to catch breakage without taxing the main matrix.

## Context

A core success signal is "multi-OS CI is green," and the rebuild's whole
point is testability from spec one. Edition 2021 is the current widely-
supported stable edition; edition 2024 is newer and raises the minimum
toolchain with no feature we need today — not worth the MSRV cost. Pure-Rust
defaults (DEC-004) are what make a trivial three-OS matrix realistic.

## Alternatives Considered

- **Option A: Edition 2024**
  - Why rejected: bumps the minimum stable toolchain for no MVP-required
    feature; edition 2021 is safer for broad install/CI compatibility today.
    (Cheap to bump later if a 2024 feature becomes worthwhile.)

- **Option B: Linux-only CI**
  - Why rejected: directly contradicts the multi-OS success signal;
    Windows path/codec issues must surface in CI, not on a user's machine.

- **Option C (chosen): edition 2021, stable, three-OS matrix + one native-feature job**
  - Why selected: meets the success signal, keeps MSRV reasonable, and
    guards the native-feature path cheaply.

## Consequences

- **Positive:** Cross-platform regressions caught early; lint/format
  enforced uniformly; clean release artifacts per OS.
- **Negative:** Windows CI is the slowest/most finicky leg (path
  separators, line endings); three OSes triple CI minutes.
- **Neutral:** Edition can be revisited per project; not locked forever.

## Validation

Right if: the three-OS matrix stays green through STAGE-001 and beyond, and
no platform-specific bug ships undetected. Revisit if: an edition-2024
feature becomes compelling, or CI minutes force trimming the matrix.

## References

- Related specs: SPEC-001 (Cargo project + CI)
- Related decisions: DEC-004 (pure-Rust default enables trivial matrix)
- External docs: https://doc.rust-lang.org/edition-guide/
