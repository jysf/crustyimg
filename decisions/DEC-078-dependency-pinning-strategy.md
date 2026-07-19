---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-078
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-19
supersedes: null
superseded_by: null

# Refines AGENTS.md §5 / DEC-011 / DEC-013's exact-pin convention; will govern
# Cargo.toml's library-public dependency rows at the crates.io publish (backlog #5).
affected_scope:
  - Cargo.toml

tags:
  - dependencies
  - semver
  - cargo-toml
  - crates-io
  - publish
  - pinning
---

# DEC-078: dependency pinning — exact for the binary now, caret for the library at publish

## Decision

Exact `=` pins stay the policy for the CLI/binary today. Relaxing the
library-public dependencies to caret (`^x.y`) becomes a mandatory prerequisite
of the crates.io publish (backlog #5), not something done now.

## Context

The pre-launch Rust audit (`docs/research/proj-008-rust-directives-audit.md`,
its D4 section) flagged the 30 exact `=` version pins in `Cargo.toml` against
the general Rust guidance "never exact-pin a manifest; use caret, and get
reproducibility from the committed `Cargo.lock`." That guidance is directionally
correct, but crustyimg's `Cargo.toml` pins every dependency exactly on purpose
(AGENTS.md §5, the DEC-011/DEC-013 pattern) to serve the PROJ-007
reproducible-build thesis: the manifest itself documents the single version
that was built and tested, on top of the lock.

The audit's D4 impact table classifies all 30 pins by reach and finds every
single row harmless today:

- **No downstream Cargo consumer exists.** crustyimg is not on crates.io
  (DEC-040/DEC-041 — a crates.io publish is authorized backlog item #5, not yet
  fired), and the npm package ships a **compiled** `crustyimg_bg.wasm`, not a
  resolvable Cargo tree. Neither path exposes an `=` pin to an external
  resolver.
- **Binary distribution** (GitHub Releases, `cargo install --locked`) consumes
  the committed `Cargo.lock` directly — the manifest pins add nothing and cost
  nothing there.

The one case where the general guidance does bite: a **library published to
crates.io**. `Cargo.lock` is ignored by library consumers, so exact `=` pins on
the published crate's public `[dependencies]` force version-unification
conflicts downstream. That case has no live target in this repo yet — it
starts to matter on the day backlog #5 actually ships.

## Alternatives Considered

- **Relax everything to caret now, rely on `--locked`.** Cleanest semver
  hygiene, and it would already satisfy the day #5 ships. Rejected: it is
  churn against a deliberate, currently-harmless convention, for zero
  downstream benefit today, and it loses the "the manifest documents the exact
  tested version" property AGENTS.md §5 and DEC-011/DEC-013 were written for.

- **Relax only the library-public dependencies now** (the `[dependencies]`
  table + the wasm-target rows), keeping bin-only and dev rows pinned.
  Pre-pays the eventual crates.io cost with less churn than the full relax.
  Rejected for the same reason as the option above, just smaller: there is
  still no consumer to benefit from it today, and it is easy to get wrong
  (mis-classifying a row as library-public vs. bin-only) without the actual
  publish forcing a careful pass.

- **Keep all pins as-is, defer the question to the crates.io-publish spec
  (chosen).** Zero downstream harm today, zero churn, and the question is
  answered exactly once, at the moment it starts to matter — by the same spec
  that has to audit the dependency table for the publish anyway.

## Consequences

- **Positive:** no code change now. The eventual crates.io-publish spec
  inherits a concrete, pre-agreed checklist item — caret-migrate the
  library-public rows (the audit's D4 impact table names them: the shared
  `[dependencies]` table and the wasm-target table) and re-verify
  `cargo update`/the lockfile — instead of re-litigating pinning under
  deadline.
- **Negative:** the pin-relaxation work is still owed; it is deferred, not
  eliminated. Whoever builds the crates.io-publish spec must budget for it.
- **Neutral:** `[[bin]]`-only dependencies and `[dev-dependencies]` never need
  relaxing — they do not constrain a library consumer's resolution, so this
  decision never reaches them.

This refines, not contradicts, AGENTS.md §5 and DEC-011/DEC-013: exact pins
remain correct for the binary; caret becomes required for the library-public
surface at publish time. AGENTS.md §5 carries a pointer to this decision so the
two read as one policy.

## Validation

Right if: the crates.io-publish spec (backlog #5) caret-migrates exactly the
library-public rows the audit's D4 impact table identifies, and no dependency
pin changes before that spec starts. Revisit if a downstream Cargo consumer
appears before #5 (e.g. a workspace member or a git dependency someone takes
on crustyimg) — that would move the "no live target" premise earlier than the
publish itself.

## References

- Related specs: SPEC-098 (this decision), SPEC-097 (the audit's other
  adopted finding, STAGE-031).
- Related decisions: DEC-011 (`viuer` display, the exact-pin pattern's
  origin), DEC-013 (`kamadak-exif`, same pattern), DEC-040 (`cargo-dist`
  release pipeline), DEC-041 (release channels — names backlog #5, the
  crates.io publish, as the gating event for this decision).
- External docs: `docs/research/proj-008-rust-directives-audit.md` (D4
  section — the impact table this decision relies on; not re-derived here).
