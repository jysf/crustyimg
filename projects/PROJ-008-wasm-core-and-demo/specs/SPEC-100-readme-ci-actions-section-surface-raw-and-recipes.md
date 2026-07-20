---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-100
  type: chore
  cycle: build
  blocked: false
  priority: high
  complexity: S

project:
  id: PROJ-008
  stage: STAGE-028
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-5
  created_at: 2026-07-20

references:
  decisions: [DEC-051]
  constraints: [ergonomic-defaults]
  related_specs: [SPEC-082]

value_link: >
  Surfaces two under-marketed differentiators in the launch README — the CI GitHub Actions and the
  RAW/recipes story — before the 0.5.0 cut. Small, ships with the README.

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-100: README CI/Actions section + surface RAW and recipes

## Context

SPEC-082 shipped the launch README front-door, but two real differentiators are still under-surfaced —
the maintainer flagged both (2026-07-20):

1. **The CI GitHub Actions are invisible.** Two published composite actions wrap the binary — the
   "image budget in CI, one line" adoption vector — and the README doesn't mention them:
   - **`jysf/setup-crustyimg@v1`** — installs the CLI on any OS runner (cargo-dist installer, cached).
   - **`jysf/crustyimg-action@v1`** — a lint/optimize wrapper with inline PR annotations (inputs:
     `mode` `lint`|`optimize`, `paths`, `args`, `fail-level`, `version`).
   (Both verified live 2026-07-20; DEC-051.)
2. **RAW input and recipes are buried.** Two things similar tools don't do well: reading a **RAW** file
   (crustyimg extracts the embedded preview from `.DNG`/`.CR2`/etc. — sharp/squoosh can't), and
   **declarative recipes** (`apply --recipe`, bundled `web`/`gallery`/`product`, tune-once-replay-across-
   a-batch — and the *same* recipe TOML runs in the browser via the wasm `transform()` binding). These
   are positioning wins that deserve a line in "Why crustyimg" / a short recipes mention, not just a
   Usage entry.

Small, docs-only, ships with the README in the 0.5.0 cut.

## Goal

Add a **CI / GitHub Actions** section to the README with the two real actions + working snippets, and
surface **RAW** and **recipes** as differentiators in the positioning. Every command/snippet must be
real; keep the human, non-AI voice SPEC-082 established.

## Inputs

- **Files to read:** `README.md` (as shipped by SPEC-082); the two action repos —
  `https://github.com/jysf/setup-crustyimg` and `https://github.com/jysf/crustyimg-action` (for the
  correct `uses:` refs, inputs, and a real usage snippet); `decisions/DEC-051-*.md` (the actions'
  design); `docs/recipes.md` (recipe examples) and `crustyimg apply --help` / the bundled recipe names
  (for an accurate recipes line); a RAW fixture / `crustyimg info` on a `.dng` if available (to describe
  RAW honestly — it's the *embedded preview*, not a full RAW develop).

## Outputs

- **Files modified:** `README.md`:
  - **A new "Use it in CI" section** — the two actions with copy-paste snippets:
    ```yaml
    - uses: jysf/setup-crustyimg@v1
    - run: crustyimg lint assets/
    ```
    and
    ```yaml
    - uses: jysf/crustyimg-action@v1
      with: { paths: assets content }
    ```
    with one line on what `crustyimg lint` catches (EXIF/GPS leaks, wrong formats, oversized/unrotated/
    corrupt) and that it exits non-zero so CI fails on a finding.
  - **RAW** — a line in "Why crustyimg" (or near the input-formats mention): reads RAW files by
    extracting the embedded preview (`.DNG`/`.CR2`/…), which sharp/ImageMagick-without-plugins/squoosh
    don't. Be honest: embedded-preview extraction, not a RAW developer.
  - **Recipes** — a short mention that pipelines are declarative and reusable: `apply --recipe`, the
    bundled `web`/`gallery`/`product`, tune-once and replay across a batch, and the same recipe TOML
    runs in the browser (wasm `transform`). Keep it tight (the Usage section already has the mechanics).

## Acceptance Criteria

- [ ] A "Use it in CI" README section exists with **both** actions (`jysf/setup-crustyimg@v1`,
      `jysf/crustyimg-action@v1`), correct `uses:` refs and inputs, and a working snippet for each.
- [ ] RAW is surfaced as a differentiator, described **honestly** (embedded-preview extraction, not a
      RAW develop).
- [ ] Recipes are surfaced as declarative/reusable (CLI + browser), tightly (no duplication of the Usage
      mechanics).
- [ ] Every `crustyimg …` command in the new/changed prose runs on the binary (extend the SPEC-082
      command sweep to the additions); the action snippets are valid YAML with real `uses:` refs.
- [ ] Voice stays human/non-AI (SPEC-082's bar — no AI-tell vocabulary, terse, concrete); links resolve
      (both action repos → 200).
- [ ] `just validate` green; no `src/`/behavior change (docs only).

## Failing Tests

- Extend the SPEC-082 command sweep: any new fenced `crustyimg …` snippet runs against the binary.
- Link check: both action repo URLs return 200.
- `grep` the new prose for AI-tell vocabulary → none (SPEC-082 list).

## Implementation Context

### Decisions that apply
- `DEC-051` — the two composite GitHub Actions (`setup-crustyimg` + `crustyimg-action`) live in their own
  repos and wrap the shipped binary; the README should point at them, not re-document them.

### Constraints that apply
- `ergonomic-defaults` — the CI section must be copy-paste-able by a newcomer; plain voice, no jargon.

### Prior related work
- `SPEC-082` (shipped) — the README front-door this extends; reuse its command-sweep + AI-voice discipline.

### Out of scope
- The npm/wasm library section (that's SPEC-076 — publish crustyimg-wasm — which will update the README's
  wasm line when it goes live; don't touch it here).
- BENCHMARKS.md (SPEC-083); any `src/`/behavior change; re-documenting the actions' internals.

## Notes for the Implementer
- **Read the two action repos** for the exact `uses:` version tag (`@v1`) and input names — don't guess;
  the snippets ship on the crate page.
- **RAW honesty:** it's the *embedded preview*, not a full RAW pipeline — say so.
- **Keep SPEC-082's voice** — human, terse, command-first, no AI-tell vocabulary ([[comments-plain-no-spec-refs]]).
- Ships with the README in the 0.5.0 cut; keep it small.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
