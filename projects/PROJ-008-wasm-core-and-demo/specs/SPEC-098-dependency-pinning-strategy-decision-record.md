---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-098
  type: chore
  cycle: build
  blocked: false
  priority: medium
  complexity: S

project:
  id: PROJ-008
  stage: STAGE-031
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8
  created_at: 2026-07-19

references:
  decisions: [DEC-011, DEC-013, DEC-040, DEC-041]
  constraints: []
  related_specs: []

value_link: >
  Closes the audit's D4 thread with a documented pinning policy, removing an open question and
  pre-recording the exact prerequisite the future crates.io publish must satisfy.

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

# SPEC-098: dependency-pinning strategy decision record

## Context

The pre-launch Rust audit (`docs/research/proj-008-rust-directives-audit.md`, its "D4 — semver-in-toml"
section + impact table) left one directive as **DECISION-NEEDED**: crustyimg's `Cargo.toml` uses **30
exact `=` pins**, a deliberate convention (AGENTS.md §5; the DEC-011/DEC-013 pattern) serving the
PROJ-007 reproducible-build thesis. A generic "use caret, not exact pins" rule would fight that
convention — but it has a real point for a *published library*. This spec closes the thread by recording
a decision, not by touching any dependency.

Evidence base (from the audit — treat as given, do **not** re-derive the pin map):
- **No downstream Cargo consumer exists today.** crustyimg is not on crates.io (DEC-040/041 — a crates.io
  publish is authorized backlog item **#5**, future work), and the npm package ships a **compiled**
  `crustyimg_bg.wasm`, not a resolvable Cargo tree. So no `=` pin currently reaches an external resolver.
- Exact pins on a library **published to crates.io** do cause downstream version-unification pain
  (`Cargo.lock` is ignored by library consumers). That only bites **if/when #5 ships.**

This spec's deliverable is a decision record; **frame only, held for maintainer review** before it lands.

## Goal

Record a decision (`decisions/DEC-078-*.md`) that keeps exact `=` pins as today's policy for the CLI/
binary, and makes relaxing the *library-public* deps to caret a **mandatory, deferred prerequisite** of
the crates.io-publish backlog item — reconciling with AGENTS.md §5 / DEC-011 / DEC-013 as a refinement,
not a reversal. **No dependency is changed by this spec.**

## Inputs

- **Files to read:** `docs/research/proj-008-rust-directives-audit.md` (D4 section + impact table — the
  evidence base and the exact list of library-public vs bin-only/dev rows); `AGENTS.md` §5 (the pinning
  convention text); `decisions/DEC-011-*.md`, `decisions/DEC-013-*.md` (the pin pattern);
  `decisions/DEC-040-*.md`, `decisions/DEC-041-*.md` (crates.io/publish posture + backlog #5).
- **Related code paths:** none — this spec reads `Cargo.toml` only as reference, edits nothing.

## Outputs

- **Files created:** `decisions/DEC-078-dependency-pinning-strategy.md` — the decision (draft prose below,
  to be dropped in verbatim/lightly-tuned on execution).
- **Files modified:** the crates.io-publish backlog entry (backlog item #5, wherever it lives —
  `docs/…` / the roadmap) gains a cross-reference: "blocked on the DEC-078 caret migration of the
  library-public deps." No other edits.
- **Explicitly not touched:** `Cargo.toml`, `Cargo.lock`, `constraints.yaml`, any `src/` file.

### The decision to record (DEC-078 — draft, plain/behavior-first)

> **DEC-078 — Dependency pinning: exact for the binary now, caret for the library at publish**
>
> **Status:** accepted. **Context:** the pre-launch Rust audit flagged the 30 exact `=` version pins in
> `Cargo.toml`. This decision sets the policy and reconciles it with AGENTS.md §5 and DEC-011/DEC-013.
>
> **Decision:**
> 1. **Exact `=` pins stay the policy for the CLI/binary today.** crustyimg ships as a binary; its
>    reproducibility comes from the committed `Cargo.lock`. On top of that, an exact pin documents the
>    single version that was built and tested, at **zero downstream cost** — because there is no Cargo
>    consumer to constrain (not on crates.io; the npm package ships a compiled `.wasm`).
> 2. **Relaxing the library-public dependencies to caret (`^x.y`) is a mandatory prerequisite of the
>    crates.io publish (backlog #5).** The moment crustyimg is a library on crates.io, exact pins on its
>    public `[dependencies]` (and the wasm-target rows) force version-unification conflicts on consumers,
>    who ignore our `Cargo.lock`. Those specific rows — enumerated in the audit's D4 impact table — must
>    move from `=` to caret **as part of shipping #5.** `[[bin]]`-only deps and `[dev-dependencies]` never
>    need relaxing: they do not constrain a library consumer's resolution.
> 3. **Do not migrate any pin now.** Converting pins today is churn against a deliberate convention with
>    **zero downstream benefit** (no consumer exists) and a small loss to the reproducible-build
>    documentation value. The migration is **deferred to and gated by #5**, not done here.
> 4. **Relationship to AGENTS.md §5 / DEC-011 / DEC-013:** this **refines**, not contradicts, them. Exact
>    pins remain correct for the binary; caret becomes required for the library-public surface at publish
>    time. AGENTS.md §5 should gain a one-line pointer to DEC-078 so the two read as one policy.
>
> **Consequences:** no code change now; the publish spec inherits a concrete, pre-agreed checklist item
> (caret-migrate the library-public rows + re-verify `cargo update`/lockfile) instead of re-litigating
> pinning under deadline.

## Acceptance Criteria

- [ ] `decisions/DEC-078-dependency-pinning-strategy.md` exists and states all four points above
      (binary-keeps-exact; library-caret-required-at-publish; no-migration-now; refines-not-contradicts
      AGENTS.md §5 / DEC-011 / DEC-013).
- [ ] The DEC references the audit doc's D4 impact table as the authoritative library-public-vs-bin/dev
      split — it does **not** re-list or re-derive the pin map.
- [ ] The crates.io-publish backlog item (#5) carries a cross-reference to DEC-078 as a blocking
      prerequisite (the caret migration).
- [ ] AGENTS.md §5 gains a one-line pointer to DEC-078 so the pinning convention and this refinement read
      as one policy (a minimal doc edit, no convention change).
- [ ] **Zero change** to `Cargo.toml`, `Cargo.lock`, `constraints.yaml`, or any `src/` file — verifiable
      by diff.

## Failing Tests

This spec produces documentation (a DEC + cross-references), so there is no code test. The verification
is structural:

- `just validate` passes (front-matter/state intact) with the new DEC present.
- A diff confirms **no** `Cargo.toml` / `Cargo.lock` / `constraints.yaml` / `src/` bytes changed — only
  `decisions/DEC-078-*.md`, the backlog entry, and the AGENTS.md §5 pointer line.
- The DEC's factual claims match the audit (no consumer today; npm ships compiled wasm; #5 is the gate).

## Implementation Context

### Decisions that apply
- `DEC-011` / `DEC-013` — the exact-pin convention this refines (kept for the binary).
- `DEC-040` / `DEC-041` — crates.io / publish posture; establish that #5 is the future publish and the
  point at which the library-public caret migration becomes mandatory.

### Prior related work
- The pre-launch Rust audit (`docs/research/proj-008-rust-directives-audit.md`) — D4 is DECISION-NEEDED;
  this DEC is that decision. **The audit doc must be on `main` for these references to resolve** (it is
  currently on the `rust-directives-audit` branch — land it first).

### Out of scope (for this spec specifically)
- **The actual caret migration of any dependency** — that is future work owned by the crates.io-publish
  spec (#5), gated by this DEC. Doing it now is explicitly rejected (point 3).
- Any edit to `Cargo.toml` / `Cargo.lock` / `constraints.yaml` / `src/`.
- Re-running the audit or re-deriving the pin map (use the audit's table).

## Notes for the Implementer
- **Keep the DEC prose plain and behavior-first** per the maintainer's standing convention
  ([[comments-plain-no-spec-refs]]); internal DEC cross-references (DEC-011/013/040/041, AGENTS.md §5) are
  fine and expected in a decision doc.
- **Confirm the exact backlog-#5 location** before adding the cross-reference (grep the roadmap/backlog
  docs for the crates.io-publish item; DEC-040/041 will name it).
- Take the **next DEC number in sequence** — DEC-078 at framing time; re-confirm nothing else claimed it
  before writing.
- This is a docs-only change; the "build" is small. It still gets a real verify (diff-confirm nothing
  outside the DEC/backlog/AGENTS-pointer moved).

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
