# Handoff — build PROJ-004 STAGE-013 (crustyimg lint), CI-gated merge-on-green

*2026-07-06 · continue in a fresh session*

## Orient (read first)

Where things stand, all on `main`:

- **PROJ-001 (MVP)** and **PROJ-002 (optimization engine)** are **shipped**. `main` is at
  **v0.3.0 (untagged)** — do NOT push a release tag / publish; that's the maintainer's step.
- **PROJ-004 (image lint)** is **framed + active** and build-ready. `just status` shows it.
- The active stage is **STAGE-013 (lint core & shipped-capability rules)**. Its four specs are
  fully written with Failing Tests. **First build target: SPEC-050.**
- Design/framing (brief, stages, specs, DEC-050) is already committed + pushed to `main`, so a
  build branch off `main` has everything it needs.

PROJ-002 shipped this same way (framed → built autonomously spec-by-spec, CI-gated, merge on
green). The prompt below repeats that for STAGE-013. It is self-contained — paste it into the new
session.

## The build prompt

```
You are continuing work on crustyimg (a fast, pure-Rust image CLI) in a FRESH session.
Working dir: the crustyimg repo. Read AGENTS.md first, every session.

CURRENT STATE (as of 2026-07-06, all on `main`):
- PROJ-001 (MVP) and PROJ-002 (optimization engine) are SHIPPED. `main` is at v0.3.0
  (untagged — do NOT push a release tag / publish; that's the maintainer's step).
- PROJ-004 (image lint) is FRAMED and ACTIVE and build-ready. `just status` shows it.
- The active stage is STAGE-013 (lint core & shipped-capability rules). Its 4 specs are
  fully written with Failing Tests. First build target: SPEC-050.

YOUR TASK — autonomously build + ship STAGE-013, CI-gated, merge-on-green:
  Build the four STAGE-013 specs IN ORDER (each depends on the prior):
    SPEC-050 (lint command core + Rule/Finding/Severity framework + exit-7 + 2 rules)
      → SPEC-051 (config) → SPEC-052 (JSON report) → SPEC-053 (shipped-capability rules).
  For EACH spec, run the full cycle: make its Failing Tests pass → all gates green →
  PR → wait for green CI → merge → ship bookkeeping on main → next spec.
  When all four ship, complete STAGE-013 and STOP for a checkpoint (STAGE-014/015 specs
  are backlog-only and need a framing pass before they can be built).

BEFORE BUILDING (read these):
  - The spec's own `## Implementation Context` + `## Failing Tests` (the contract).
  - DEC-050 (the lint command/rule-catalog/severity/exit-7/config contract).
  - docs/research/proj-002-design-lint.md (the AUTHORITATIVE rule catalog + Lighthouse-parity
    map + config/exit design).
  - The PROJ-004 brief + STAGE-013; guidance/constraints.yaml.

MERGE-ON-GREEN FLOW (authorized): build → push `feat/spec-0NN-<slug>` branch → PR →
  `gh pr checks <n> --watch` → `gh pr merge <n> --squash --delete-branch` on green →
  do verify+ship bookkeeping on `main` (reflection, cost, timeline, archive to done/,
  stage backlog + counts) → next spec. Ship each spec FULLY before opening the next PR
  (main moving under an open PR makes it BEHIND — if that happens, `gh pr update-branch`,
  never `--admin`).

GATES (all must be green before merge): `cargo test`, `cargo clippy --all-targets -- -D
  warnings`, `cargo fmt --check`, `cargo build --no-default-features` + `cargo test
  --no-default-features` (the lean job), and `just deny`. CI also runs the 3-OS matrix +
  avif/webp-lossy feature jobs + msrv(1.89) + cost-data.

COST: build/verify run in the main loop (background subagents can't get a shell here), so
  they're not metered — record cost.sessions build/verify `tokens_total` as clearly-labelled
  order-of-magnitude ESTIMATES (NOT null; `just cost-audit` needs positive values). design/ship
  stay null-with-note. See docs/cost-tracking.md / AGENTS §4.

GUARDRAILS:
  - `src/lint/` is READ-ONLY — no Sink/write path; every finding NAMES a fix command, never
    runs it. Decode failure is a FINDING (size/truncated-or-corrupt → exit 7), not an abort.
  - Reuse shipped plumbing: `source::resolve` (glob/dir/file fan-out), `CliError::CheckFailed`
    (exit 7, DEC-025), the hand-rolled `write_json`/`escape_json` pattern (no serde_json).
    No new default dependency.
  - Determinism: sort findings (path, severity, rule); no network/mtime/wall-clock. Make any
    JSON golden test a SYNTHETIC-input exact-string test (cross-platform safe), per SPEC-049.
  - `just status` should show PROJ-004; keep its backlog/counts accurate as specs ship.
  - Verify `git branch --show-current` before every commit; never push a release tag.

DELIVERABLE: STAGE-013 shipped — `crustyimg lint` is a working CI linter (source resolve,
  the rule framework, config, JSON report, and the shipped-capability rule catalog), `main`
  green, all four specs archived. Then stop and report; STAGE-014 (engine-backed rules) needs
  framing next.
```

## Scoping options

- **One cycle only:** to build just SPEC-050 (review the framework before the rest of STAGE-013
  layers on it), replace the TASK block with "Build only SPEC-050 through ship, then stop."
- **PR-review mode:** the merge-on-green authorization is baked in (the session merges its own PRs
  on green CI). Drop that paragraph if you'd rather it open PRs and stop for your review.

## Pointers

- First build target: `projects/PROJ-004-image-lint/specs/SPEC-050-lint-command-core.md`
- Contract: `decisions/DEC-050-lint-command-and-rule-catalog.md`
- Authoritative rule catalog: `docs/research/proj-002-design-lint.md`
- Stage: `projects/PROJ-004-image-lint/stages/STAGE-013-lint-core-and-shipped-rules.md`
