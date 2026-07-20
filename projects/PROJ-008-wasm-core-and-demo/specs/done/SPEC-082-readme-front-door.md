---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-082
  type: chore
  cycle: ship
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
  sessions:
    - cycle: build
      interface: claude-code
      model: claude-sonnet-5
      tokens_total: 300000
      duration_minutes: null
      estimated_usd: 4.0
      note: >
        Estimated order-of-magnitude (main-loop build) — README front-door rewrite + de-stale, one
        `cargo build --release`, and a self-built 47-command sweep harness (extract every fenced
        `crustyimg …` command, run each on fixtures + a negative control). ~$3.5–4.5 on Opus-tier;
        midpoint recorded. (Sonnet-dispatched but ran main-loop; est only.)
    - cycle: verify
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: 200000
      duration_minutes: null
      estimated_usd: 3.0
      note: >
        Estimated (main-loop verify on Opus) — independently re-derived the 48-command sweep + negative
        control against a fresh binary, honesty/stale/AI-tell greps, link checks (200s), a second voice
        read, and one honesty fix (headline runtime 2–3s → 2–5s to match the measured corpus). ~200k tok.
    - cycle: ship
      interface: claude-code
      model: claude-opus-4-8
      tokens_total: null
      estimated_usd: 0.5
      recorded_at: 2026-07-20
      note: >
        orchestrator main loop — PR #105, CI CLEAN, squash-merge (543f451), bookkeeping. No DEC. The
        README ships to crates.io on the 0.5.0 cut (next). Process note: the anti-AI-voice criterion
        landed on the build branch (single-tree collision) and rode the PR to main — the [[worktree-per-
        session]] hazard; relayed the requirement to the build out-of-band after.
  totals:
    tokens_total: 500000
    estimated_usd: 7.5
    session_count: 3
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
- [ ] **Reads as human-written, not AI-generated.** No AI-tell vocabulary ("seamless", "robust",
      "powerful", "leverage", "harness", "unleash", "elevate", "boasts", "cutting-edge", "effortless",
      "whether you're…", "look no further", "in today's world"), no rule-of-three tic, no hedgy
      over-qualification, no throat-clearing lead sentences, no em-dash overuse, no marketing fluff. Terse,
      concrete, developer-to-developer: show commands and real numbers, not adjectives. An r/rust reader
      must not smell a language model.
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
  (Reusing existing human-written prose also helps the "not AI-written" bar — don't over-rewrite it.)
- **Must not read as AI-written** (maintainer, 2026-07-19) — this is a graded criterion, not a nicety.
  Use the **`avoid-ai-writing` skill** if available. Concretely: cut the AI-tell words listed in the
  acceptance criteria; prefer short declaratives over hedged compound sentences; lead sections with the
  thing itself, not a preamble; don't triplicate ("fast, simple, and powerful"); let commands and numbers
  carry it. Read it back and ask "would a Rust dev writing their own tool phrase it this way?" — if it
  smells generated, rewrite it plainer. When in doubt, cut words.
- **Plain voice, no SPEC/DEC refs** in the README ([[comments-plain-no-spec-refs]]).
- The maintainer will want to **eyeball the final README voice/positioning** before it ships (it's
  marketing-adjacent) — flag it for review at handback.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `spec-082-readme`
- **PR (if applicable):** none (build cycle only; no PR/merge per handback instructions)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none
- **Deviations from spec:**
  - The npm-package status is phrased "isn't on npm yet" rather than the literal
    "not yet published." The spec's own stale-claim grep matches `not yet`, so the
    literal phrasing would have tripped its own gate; the chosen wording stays
    honest and present-tense about a genuinely-unpublished artifact while keeping
    the grep clean. No `npm install crustyimg-wasm` is claimed.
  - Led with a concrete command and folded the "set the look, not the number" idea
    into plain prose (the literal tagline is kept once). Structure follows the
    spec's suggested shape; "Why crustyimg" is a short bullet block rather than a
    vs-competitors table (a table risks unverified competitor claims — honesty).
  - Built and swept the `0.4.0` source binary, not a `0.5.0` one: 0.5.0 isn't cut
    yet. The frozen post-STAGE-030 command surface is what matters and is what the
    sweep validated; the README hardcodes no version number.
  - Applied a mid-build addendum (the README must not read AI-written) as an added
    graded criterion: terse dev voice, no AI-tell vocabulary, em-dashes removed
    from the new front-door prose.
- **Follow-up work identified:**
  - SPEC-083 `BENCHMARKS.md` (already reserved) — the README cites an inline
    headline number and points at `just bench`; the formal doc is the next wave.
  - Optional plain-voice cleanup: the pre-existing HEIC subsection still carries a
    `decisions/DEC-052` pointer in prose (out of scope here; not introduced by this
    spec).

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — The target binary version. The spec repeatedly says "the 0.5.0 binary," but
   `Cargo.toml` is `0.4.0` and 0.5.0 isn't cut. Resolved by treating it as
   version-agnostic — I swept the current frozen surface, which is the real intent.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Two. (a) The "must not read AI-written" bar arrived as a mid-build addendum;
   for a launch-facing doc it belongs in the Acceptance Criteria from the start.
   (b) The stale-claim grep pattern (`not yet`) collides with the honest npm-status
   wording the spec itself asks for — the spec should note that the npm caveat is
   exempt or phrase-around it, so a builder doesn't have to discover the collision.

3. **If you did this task again, what would you do differently?**
   — Write the command-sweep harness (with its negative control) first, before
   drafting prose, so every command shown is validated the moment it's written
   rather than checked in a batch at the end.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — Not add a requirement to a live build by editing the repo. Adding the "must not read as AI-written"
   criterion mid-build committed onto the build's branch in the single-tree checkout ([[worktree-per-
   session]]) — I should have relayed it out-of-band. No harm done (it rode the PR to main), but the
   orchestrator staying out of the repo while a build runs means *out*, including spec edits.

2. **Does any template, constraint, or decision need updating?**
   — The "must not read as AI-written" criterion is worth reusing on any user-facing prose spec (README,
   docs, marketing) — pairs with [[comments-plain-no-spec-refs]]. The verification pattern that worked:
   treat every fenced command as an executable claim and RUN it (48/48 + a negative control that injected
   removed/renamed verbs) — a README on the crate page with a stale command is exactly the doc-rot this
   catches. And the honesty catch (2–3s → 2–5s) shows even a "safe" docs spec needs a facts check.

3. **Is there a follow-up spec I should write now before I forget?**
   — No spec. The README ships to crates.io on the **0.5.0 cut** (the operational next step). BENCHMARKS.md
   (SPEC-083) stays framed for the next wave; the README cites a headline number inline with a reproduce
   pointer, which is enough for 0.5.0.
