---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-082
  type: chore
  cycle: build
  blocked: false
  priority: high
  complexity: M

project:
  id: PROJ-008
  stage: STAGE-028
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-5
  created_at: 2026-07-14

references:
  decisions: [DEC-070, DEC-064, DEC-079]
  constraints: [ergonomic-defaults]
  related_specs: [SPEC-080, SPEC-085, SPEC-075, SPEC-099]

value_link: >
  The front door for the r/rust-first launch — and the page crates.io renders. Turns a good CLI
  reference into a launch README: the browser demo, the why-vs-alternatives, honest install.

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

# SPEC-082: README front door

## Context

The README is the launch front-door — the first thing an r/rust or HN reader hits, **and the page
crates.io renders** on the crate page (`Cargo.toml` `readme = "README.md"`). It ships in the 0.5.0 release
(this spec lands right before that cut). The current README (365 lines) is a *good CLI reference* — the
Usage section already reflects the frozen post-STAGE-030 surface (`web`/`optimize`/`convert`/`meta`/etc.).
But as a launch front-door it has concrete gaps, including a stale claim of the exact kind SPEC-099 just
swept:

1. **Stale "not yet published" claim.** The Install section splits "Works today" vs **"Once v0.1.0 is
   published"** (README ~line 26) — but crustyimg has been on crates.io since v0.1.0 and Homebrew since
   then too (currently 0.4.0; 0.5.0 is imminent). `cargo install crustyimg` and `brew install
   jysf/tap/crustyimg` **work**. This is the same false premise SPEC-099 corrected elsewhere; the README
   was missed.
2. **No browser-demo link.** The live client-side demo (**https://jysf.github.io/crustyimg/**) — drop an
   image, convert to AVIF in-browser, see the SSIMULACRA2 score, zero install — is the single best "watch
   it just work" hook and it is **not in the README at all.**
3. **No wasm / library story.** That the same pure-Rust engine compiles to wasm and runs the demo
   client-side (SPEC-072/079) is a real differentiator and is absent.
4. **No positioning.** There's no "why crustyimg vs sharp / squoosh / ImageMagick" — the wedge (we
   *measure* quality with SSIMULACRA2 instead of guessing; one pure-Rust binary with zero system deps vs
   sharp's native/libvips addon; `web` = one command to modern-format + never-bigger + a score; and it
   runs in the browser) is the launch story.

## Goal

Turn the README into a launch front-door: lead with the concrete win + the browser demo, add the
why-vs-alternatives positioning and the wasm/library story, and **de-stale every "not yet published"
claim** — while keeping the already-current Usage reference. Every command shown must actually run on the
0.5.0 binary. No overclaiming.

## Inputs

- **Files to read:** `README.md` (the current 365-line version — keep the good Usage section); the live
  demo at `https://jysf.github.io/crustyimg/` (what it does, to describe it honestly); `RELEASING.md` +
  `decisions/DEC-079-*.md` (the true publish state — published since v0.1.0, auto-per-tag); the STAGE-030
  benchmark numbers (from `stages/STAGE-030-*` / the committed `scripts/bench.py` — for an honest headline
  figure); `crustyimg --help` and each subcommand's `--help` (so every command shown is real);
  `projects/PROJ-008-*/specs/done/SPEC-075-*` (the `crustyimg-wasm` npm package — **note it is NOT yet
  published**, SPEC-076 gated).

## Outputs

- **Files modified:** `README.md` — restructured as a front-door. Suggested shape (implementer's judgment
  on exact layout):
  - **Hook** (top): one or two lines that land the win + a link to the browser demo. Keep the "Set the
    look, not the number" idea but lead with something concrete a reader acts on.
  - **▶ Try it in your browser** — link `https://jysf.github.io/crustyimg/`: drop an image → AVIF
    client-side → SSIMULACRA2 score, 100% in-browser (the same Rust engine compiled to wasm), zero
    install/upload. This is the hook — put it high.
  - **Why crustyimg** — a short, honest positioning block/table vs the field: *measured* quality
    (SSIMULACRA2, not a guessed quality number); one pure-Rust static binary, zero system deps (vs sharp's
    native/libvips addon); `web` = one command → smallest modern format that beats the source, never
    silently bigger, with a score; runs in the browser via wasm. **An honest headline number** from the
    real corpus (e.g. `web` ≈ median ~98% smaller in a few seconds, size-insensitive) — cite it as a
    measurement, not a boast; do not fabricate.
  - **Install** — **de-staled**: drop the "Works today / Once v0.1.0 is published" split; all channels are
    live (`cargo install crustyimg`, `brew install jysf/tap/crustyimg`, prebuilt Releases binaries,
    `cargo install --git` for bleeding edge). (`cargo add crustyimg` as a library is now caret-friendly
    per DEC-079 — mention only if it fits; the primary story is the CLI.)
  - **Usage** — keep the existing, current section (light edits only; it already matches the frozen CLI).
  - **A short wasm/library note** — the engine compiles to wasm and powers the demo; the `crustyimg-wasm`
    npm package exists but is **not yet published** — describe the capability honestly, link the demo +
    source; **do NOT write `npm install crustyimg-wasm` as if it works.**

## Acceptance Criteria

- [ ] **No stale publish claims** — grep the README for `v0.1.0`, "once published", "not yet", "works
      today"; none may imply the published channels aren't live. Install lists the real, working channels.
- [ ] **The browser demo is linked prominently** near the top, described honestly (client-side, in-browser
      AVIF + SSIMULACRA2, zero install).
- [ ] **Positioning present** — a short honest "why crustyimg" (the measured-quality wedge + pure-Rust
      zero-deps + `web` + browser), with any headline number attributed to the real benchmark, not
      fabricated.
- [ ] **Every command shown actually runs** on the freshly built 0.5.0 binary (parses + does what the
      prose says) — no invented flags, no removed verbs (`shrink` is gone), no renamed ones
      (`meta strip`, not `strip`).
- [ ] The wasm/library mention is honest — no `npm install crustyimg-wasm` claim while it's unpublished.
- [ ] All links resolve (the demo URL, Releases, RELEASING.md, tap).
- [ ] `just validate` stays green; no `src/`/behavior change (docs only).

## Failing Tests

README is prose, so verification is a **commands-and-claims sweep**, not a unit test:

- A script or checked procedure that **extracts every fenced `crustyimg …` command from README.md and runs
  it** against the freshly built binary (on a fixture), asserting it parses and exits as the prose implies
  — the "a citation looks like prose, not a claim" discipline applied to README commands. Removed/renamed
  verbs or invented flags must fail this.
- `grep -niE "v0\.1\.0|once .*published|not yet|works today"` over README.md → no stale-publish hit
  survives (allow legitimate historical mentions only if clearly past-tense).
- A link check (the demo URL returns 200; internal links resolve).

## Implementation Context

### Decisions that apply
- `DEC-070` — the `web` flagship + bundled recipes (the headline capability to feature).
- `DEC-064` — the pure-Rust engine compiles to wasm (the demo/library story).
- `DEC-079` — crustyimg IS published (caret deps); the Install section must reflect that, not the old
  "not yet published" framing.

### Constraints that apply
- `ergonomic-defaults` — the front-door must read for a newcomer; plain, concrete, honest
  ([[comments-plain-no-spec-refs]]) — no internal jargon, no SPEC/DEC refs in the README prose.

### Prior related work
- `SPEC-080` (shipped) — the live demo this links; `SPEC-085` — the `web` verb; `SPEC-075` — the npm
  package (unpublished); `SPEC-099` — the publish-state correction the Install section must match.

### Out of scope
- The formal `BENCHMARKS.md` (SPEC-083) — a separate next-wave doc; here, cite a headline number inline.
- Any `src/` / behavior / CLI change; new features.
- Publishing the npm package or cutting the release (0.5.0 is a separate release op after this).
- A logo/graphic (the demo-polish v2 batch owns that).

## Notes for the Implementer
- **Every command is a claim — run it.** Build the binary and actually execute each `crustyimg …` snippet;
  the frozen surface removed `shrink` and renamed metadata verbs into `meta`, so a stale command would ship
  a broken README on the crate page.
- **Honesty over polish** (the whole project's discipline): link the demo because it's live; do NOT write
  an `npm install` that doesn't work; attribute the benchmark number to the real corpus; if unsure a claim
  is true, cut it.
- **Keep the good parts** — the Usage section is current; this is a front-door + de-stale, not a rewrite.
- **Plain voice, no SPEC/DEC refs** in the README ([[comments-plain-no-spec-refs]]).
- The maintainer will want to **eyeball the final README voice/positioning** before it ships (it's
  marketing-adjacent) — flag it for review at handback.

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
