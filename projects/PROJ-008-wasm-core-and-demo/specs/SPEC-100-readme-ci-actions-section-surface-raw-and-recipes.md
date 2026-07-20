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
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-sonnet-5
      tokens_total: 120000
      duration_minutes: null
      estimated_usd: 1.2
      note: >
        Estimated order-of-magnitude (main-loop build, Sonnet). Read README/DEC-051/docs/recipes.md,
        fetched both action repos' live action.yml via `gh api` for exact input ground truth (found the
        existing CI section already satisfied most acceptance criteria — the spec's "invisible" framing
        was stale), read src/image/raw.rs + src/recipe/bundled.rs + src/wasm.rs for accurate RAW/recipe
        claims, wrote 2 new "Why crustyimg" bullets + a CI-inputs addition, ran a 6-command sweep against
        the release binary, grepped for AI-tell vocabulary, link-checked both repos (200/200).
  totals:
    tokens_total: 120000
    estimated_usd: 1.2
    session_count: 1
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

- **Branch:** `spec-100-readme-ci` (commit `ceec7a2`)
- **PR (if applicable):** none — hand back for verify, no PR/merge per orchestrator instruction
- **All acceptance criteria met?** yes
  - "Use it in CI" section with both actions, correct `uses:` refs/inputs, working snippets —
    **already existed** (DEC-051's original adoption glue, `## Continuous integration` →
    `### GitHub Actions`, shipped 2026-07-06). Verified against the live `action.yml` in both
    repos (fetched via `gh api`) and extended it with the four inputs it wasn't showing yet
    (`mode`, `args`, `fail-level`, `version`) — the setup-crustyimg snippet + "what lint catches
    + exits non-zero" line were already accurate and unchanged.
  - RAW surfaced honestly in "Why crustyimg" as embedded-preview extraction (verified against
    `src/image/raw.rs`'s doc comment and `RAW_EXTENSIONS`; confirmed live with `crustyimg info`
    on a `.nef` fixture — reports `format: jpeg`, the extracted preview).
  - Recipes surfaced in "Why crustyimg" as declarative/reusable, CLI + browser (verified
    `apply --recipe` resolves bundled `web`/`gallery`/`product` by name via
    `src/recipe/bundled.rs`, and `src/wasm.rs::transform()` docs itself as "the browser twin of
    `crustyimg apply --recipe`" reading the same TOML).
  - Command sweep: `crustyimg info`/`web` on the RAW fixture, `edit --save-recipe`,
    `apply --recipe web|gallery|product` all ran clean against `./target/release/crustyimg`.
    Both action repo URLs return 200 (`curl -o /dev/null -w %{http_code}`). Grepped the new
    prose for SPEC-082's AI-tell list — none found. `just validate` green (front-matter only;
    no other README-content check exists in the repo).
- **New decisions emitted:** none
- **Deviations from spec:**
  - The Outputs section describes the CI section as new ("A new 'Use it in CI' section"), but
    it already existed in full — both actions, exact refs, the lint-catches + exit-code line —
    from DEC-051's glue work (2026-07-06), predating this spec. The spec's Context claim ("the
    README doesn't mention them") didn't match the repo at build time. Treated the acceptance
    criteria as the source of truth over the literal Outputs framing: verified the existing
    section against the two live `action.yml` files rather than rewriting it, and only added
    the genuinely missing pieces (the four under-documented Action inputs, and the RAW/recipes
    positioning lines, which really were absent). No section rename — `## Continuous
    integration` already reads as "use it in CI" and a rename would just churn anchors/links
    for no acceptance-criteria gain.
- **Follow-up work identified:** none

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — The spec's Context asserted the CI section didn't exist; the actual README already had a
   thorough one. Cost one research pass to confirm before writing anything — worth it, since
   writing a duplicate section would have been the wrong deliverable. [[a-plausible-test-result-is-not-a-checked-one]]-adjacent: a spec claim about repo state is exactly the kind of thing that needs re-checking against the live file, not trusted from the framing session's memory of it.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No — DEC-051 was listed and was the right decision to read; it explained *why* the section
   already existed (STAGE-015 glue work), which resolved the confusion once found.

3. **If you did this task again, what would you do differently?**
   — Same approach: read the current README in full before touching it, fetch both action.yml
   files directly (not just repo READMEs) for exact input ground truth, and run every
   command/flag referenced in new prose against the real binary before committing.

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
