---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-033
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

created_at: 2026-06-18
supersedes: null
superseded_by: null

affected_scope:
  - src/cli/mod.rs
  - Cargo.toml

tags:
  - dependencies
  - cli
  - batch
  - progress
  - license
---

# DEC-033: `indicatif` for batch progress reporting

## Decision

Add **`indicatif` `=0.18.4`** (MIT, pure-Rust) as the progress-bar dependency for the
batch `apply --recipe` run (STAGE-005, SPEC-031). It renders a live progress bar **to
stderr** while a recipe is replayed in parallel across a source list, so stdout stays
clean for `-o -` pipes. `rayon` (the parallelism itself) is **not** new here — it is
pre-justified by **DEC-006** ("no async; batch parallelism via rayon … landed for
`apply` in STAGE-005"); this DEC only covers the progress dependency.

## Context

STAGE-005's success criteria call for `apply --recipe … --out-dir …` to run in
parallel "with a progress bar". A long batch (a directory of photos) needs feedback;
without it the tool looks hung. `indicatif` is the de-facto Rust progress crate, pure-
Rust and permissive. A probe confirmed its tree is clean: `indicatif` (MIT) → `console`
(MIT), `unicode-width` (MIT OR Apache-2.0), `portable-atomic` (Apache-2.0 OR MIT),
`number_prefix` — all pure-Rust, all permissive; `cargo deny check licenses` stays
green with no new exception. The `ProgressBar` is thread-safe, so each `rayon` task can
`inc(1)` on completion.

`indicatif` is a new top-level dependency, so the `no-new-top-level-deps-without-
decision` constraint requires this record.

## Alternatives Considered

- **Hand-rolled stderr counter (`eprint!("\r{done}/{total}")`)** — no dependency, but
  no ETA/throughput, flickers under parallel writes, and re-implements terminal
  handling. Rejected: indicatif is small, permissive, and correct.
- **No progress at all** — rejected: the stage explicitly wants a progress bar; a
  silent multi-minute batch is poor UX.
- **`linya`/other progress crates** — comparable; `indicatif` is the most widely used
  and battle-tested, and integrates cleanly with `rayon`.

## Consequences

- **Positive:** Clear batch feedback (count + ETA) on stderr; stdout stays pipe-clean;
  thread-safe under `rayon`. Pure-Rust, permissive, `just deny` green.
- **Negative:** +1 dependency (+ its small tree, e.g. `console`). The bar must be
  suppressed when `--quiet` and when stderr is not a TTY (indicatif auto-hides on a
  non-terminal; honor `--quiet` explicitly).
- **Neutral:** Only the batch (multi-input) `apply` path uses it; single-input runs and
  the pixel ops are unaffected.

## Validation

Right if: a multi-input `apply --recipe` shows a progress bar on stderr that advances as
inputs finish, stdout stays clean, `--quiet` suppresses it, and `cargo deny` stays
green (SPEC-031 tests + manual run). Revisit if: we want progress on other long
commands (reuse the same helper) or need to drop the dep for a leaner build (feature-
gate it).

## References

- Related specs: SPEC-031 (parallel batch `apply --recipe`)
- Related decisions: DEC-006 (no async; rayon batch parallelism — the parallelism dep),
  DEC-018 (permissive license / `cargo deny`), DEC-015 (partial-batch exit 6)
- Constraints: `no-new-top-level-deps-without-decision`, `pure-rust-codecs-default`,
  `no-agpl-default-deps`
- External docs: https://docs.rs/indicatif/0.18.4
