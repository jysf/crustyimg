---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-043
  type: decision
  confidence: 0.8
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

created_at: 2026-07-04
supersedes: null
superseded_by: null

affected_scope: []

tags:
  - process
  - methodology
  - patch-lane
  - framework
---

# DEC-043: introduce a lightweight "patch" lane for fixes to shipped behavior

## Decision

Add a **patch lane** to the spec-driven workflow: a lightweight track for bounded fixes
to *already-shipped* behavior (bugs, UX papercuts, small hardening) that introduce **no
new feature/command** and don't warrant a full spec + stage.

A patch runs a **3-step collapsed cycle** instead of the spec's 5:

| Spec cycle | Patch cycle |
|---|---|
| frame → design → build → verify → ship | **patch → verify → ship** |

- **patch** — design + build collapsed into ONE pass: write the failing test(s) *and*
  the fix together (still test-first within the pass).
- **verify** — **kept, and kept INDEPENDENT** (a different session). This is the single
  discipline the framework retrospective proved catches real defects; it is
  non-negotiable for a patch.
- **ship** — CHANGELOG `[Unreleased] → Fixed` + archive. **No stage bookkeeping** —
  patches attach to the project, not a stage.

**What stays (non-negotiable):** the full gate suite (fmt/clippy/test + lean
`--no-default-features` + `cargo deny`), a **DEC only when there's a real decision**, and
index-verify-before-ship. **What's shed:** the separate frame + design cycles, the stage
backlog/`Count:` bookkeeping, and the spec's heavier frontmatter.

**Mechanics:** patches live in `projects/PROJ-NNN/patches/PATCH-NNN-<slug>.md` with their
own `PATCH-NNN` number line (separate from `SPEC-NNN`); prompts in
`patches/prompts/PATCH-NNN-patch.md`. Patches roll into the next release (a `0.1.x` patch
or a `0.x.0` minor) via the CHANGELOG.

## Context

This directly implements the top recommendation of the framework retrospective
(`docs/framework-feedback/process-feedback.md` + `signals-harvest.md`): the full
four-cycle was **disproportionate for trivial changes** (SPEC-043, a 3-line `deny.toml`
edit, ran design→build→verify→ship), and the two elements that actually *bought* quality
were the **DEC log** and the **independent verify** — not the ceremony of the four named
cycles. The patch lane keeps exactly those two and drops the rest.

It also matches real need: crustyimg is now shipped (v0.1.0), so post-release maintenance
(e.g. the `--out-dir` auto-create fix, PATCH-001) shouldn't require standing up a stage.

## Alternatives Considered

- **Self-verify + gates-only patches (no independent verify)** — lighter still, but
  loses the one lever the retrospective validated. Rejected: independent verify is the
  quality floor.
- **Just use a normal spec for small fixes** — the status quo; its overhead is the exact
  problem being fixed. Rejected.
- **A generic "fast lane" for any change** — too broad; a patch is specifically a fix to
  *shipped* behavior with no new surface, which is what makes the collapsed cycle safe.

## Consequences

- **Positive:** post-release fixes cost ~2 metered cycles instead of ~4, with no stage
  overhead, while keeping independent verify + DECs + gates. Encourages fixing papercuts
  instead of deferring them.
- **Negative:** a second artifact type + numbering sequence to track; risk of scope
  creep (a "patch" that's really a feature). Guard: if a change adds a command/flag or
  needs its own design exploration, it's a spec, not a patch.
- **Neutral:** no `just new-patch` helper yet — patch files are hand-created from the
  convention above (a helper is optional future tooling). Documented in
  `docs/development.md`.

## First-use learnings (PATCH-001)

The `--out-dir` auto-create fix was the first patch. It confirmed the lane fits a bounded
fix well (the collapsed patch pass + independent verify caught a real code detail without
a full spec). Two refinements for future patch prompts:

- **Hedge "remove/dedupe X" instructions.** PATCH-001's prompt asserted deduping
  `run_responsive`'s `create_dir_all`; the code didn't match that assumption (it uses
  `Sink::File`, not `Sink::Dir`), and removal broke tests. The parenthetical escape hatch
  saved it, but a patch instruction that touches *other* code should be phrased
  conditionally by default: **"remove X only if the affected command's tests still pass;
  otherwise leave it with a comment explaining why."**
- **Independent verify stays independent even when interrupted.** If the verify subagent
  is stopped mid-run, the orchestrator completes the substantive checks (here: the
  security-boundary reasoning + guard-tests-unchanged diff + end-to-end) in the main loop
  before ship — verify is a gate, not a formality.
