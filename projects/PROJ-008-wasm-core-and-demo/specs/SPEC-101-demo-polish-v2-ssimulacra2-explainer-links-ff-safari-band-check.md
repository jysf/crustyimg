---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-101
  type: chore
  cycle: design
  blocked: false
  priority: medium
  complexity: S

project:
  id: PROJ-008
  stage: STAGE-029
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-5
  created_at: 2026-07-20

references:
  decisions: []
  constraints: [ergonomic-defaults]
  related_specs: [SPEC-081]

value_link: >
  Small pre-launch demo polish: make the SSIMULACRA2 score self-explaining (link the metric) and confirm
  the score band renders correctly on Firefox/Safari before real users hit it.

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

# SPEC-101: demo polish v2 — SSIMULACRA2 explainer links + FF/Safari band check

## Context

Two small demo items surfaced during launch prep (queued 2026-07-19), batched here so they ship as one
build:

1. **The SSIMULACRA2 score is unexplained.** SPEC-081 shows a real SSIMULACRA2 number on a band, but a
   visitor who's never heard of the metric has no way to learn what it is. Link it.
2. **The score band is Chrome-verified only.** SPEC-081's band/meter uses `color-mix()` (12 uses) and was
   only driven in headless Chrome; it degrades gracefully but has never been confirmed on real
   Firefox/Safari — a launch carry.

Small, demo-files-only, no engine change. (A third queued item — the 🦀→logo swap — is **deferred until a
logo exists**; not in this spec.)

## Goal

Make the SSIMULACRA2 score self-explaining (link the metric explainer + the Rust impl) and confirm the
score band renders correctly on real Firefox and Safari before launch. Demo files only.

## Inputs

- **Files to read:** `demo/demo.js` (`renderScore` — the score panel from SPEC-081), `demo/index.html`
  (the score/meter markup), `demo/demo.css` (the band/meter + the `color-mix()` uses); the two link
  targets (below).

## Outputs

- **Files modified:**
  - `demo/demo.js` / `demo/index.html` — make the score panel's **"SSIMULACRA2"** label a link with a
    small "what's this?" affordance, pointing at:
    - the metric explainer → **https://github.com/cloudinary/ssimulacra2** (Jon Sneyers/Cloudinary — note
      the "2"; `cloudinary/ssimulacra` without it is the older v1, wrong), and
    - the Rust implementation we actually run → **https://github.com/rust-av/ssimulacra2**.
    Keep it unobtrusive (a linked label + a secondary "Rust impl" link), honest, and consistent with the
    existing panel voice.
  - `demo/demo.css` — any styling for the link affordance; theme-aware, matches the panel.
- **Verification artifact (not a code change):** the FF/Safari band render is confirmed (see below).

## Acceptance Criteria

- [ ] The score panel's "SSIMULACRA2" is a **link to https://github.com/cloudinary/ssimulacra2** (the
      metric explainer, with the "2"), plus a secondary link to **https://github.com/rust-av/ssimulacra2**
      (the impl). Both resolve (200). Unobtrusive, honest, on-voice.
- [ ] The **score band renders correctly on real Firefox and Safari** — the `color-mix()` band colors show
      (or degrade to the documented `var(--muted)`/`var(--good)` fallback without looking broken); confirmed
      by driving the demo on Firefox + Safari (the SPEC-078/080 multi-browser harness), not just Chrome.
      Record what each browser showed.
- [ ] Zero network requests during a conversion still holds (the links are static `href`s, not fetches).
- [ ] Browser smoke stays green; no `src/`/engine change; no wasm rebuild needed.

## Failing Tests

- Extend the demo smoke: the score panel contains an `href` to `github.com/cloudinary/ssimulacra2`; the
  conversion path still makes **0 network requests**.
- **Cross-browser (the point):** drive the demo on real Firefox + Safari and assert the band element gets a
  non-default computed background (or the documented graceful fallback), and the panel is not visually
  broken. Report per browser.

## Implementation Context

### Constraints that apply
- `ergonomic-defaults` — the link must help a non-expert without cluttering the panel; plain, honest.

### Prior related work
- `SPEC-081` (shipped) — the score panel + band this extends; the `color-mix()` FF/Safari carry it noted.
- `SPEC-078`/`SPEC-080` — the multi-browser (Chrome/FF/Safari) demo harness to reuse for the band check.

### Out of scope
- The 🦀→logo swap (deferred until a logo exists — its own tiny follow-up).
- Any engine/wasm change or new score behavior; a side-by-side pixel diff (SPEC-081 out-of-scope, still).

## Notes for the Implementer
- **Get the URL right:** `cloudinary/ssimulacra2` (with the "2") is the metric; `rust-av/ssimulacra2` is the
  impl we run. Both verified live 2026-07-19.
- **The FF/Safari check is the load-bearing half** — it closes the SPEC-081 launch carry. Actually drive
  those browsers; don't assume `color-mix()` support.
- Keep the links `href`-only (no fetch) so the zero-network invariant holds.
- Plain voice, no SPEC/DEC refs in the page ([[comments-plain-no-spec-refs]]).

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
