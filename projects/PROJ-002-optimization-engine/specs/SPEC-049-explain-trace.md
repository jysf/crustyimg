---
# Maps to ContextCore task.* semantic conventions.

task:
  id: SPEC-049
  type: story                      # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # M | S | L

project:
  id: PROJ-002
  stage: STAGE-012
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-opus-4-8     # usually same Claude, different session
  created_at: 2026-07-05

references:
  decisions: [DEC-016, DEC-024, DEC-047, DEC-048, DEC-049]
  constraints: [untrusted-input-hardening, no-new-top-level-deps-without-decision, ergonomic-defaults, test-before-implementation]
  related_specs: [SPEC-048, SPEC-047]

value_link: >
  Makes the auto-decision auditable, not a black box — the answer to the "is it just
  another f_auto?" risk. Prints why crustyimg picked the format, which turns the moat
  from "smallest file" into "smallest file, with the reasons shown."

cost:
  sessions: []
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-049: `--explain` — the auditable decision trace

## Context

SPEC-048 already prints a **one-line summary** (chosen format + savings %) by default — the format
choice is never silent. But a one-liner doesn't show *why*, and an auto-decider that can't justify
itself reads as a black box — exactly the differentiator risk flagged in the PROJ-002 thesis ("may
read as a black box vs. Cloudinary `f_auto` if the trace isn't clear"). This spec adds the
**auditable full trace**: `optimize --explain` supersedes the default summary with the features
detected, the class, every candidate the engine tried (format / quality / score / bytes /
met-target), the winner + a one-line reason, and the source→output savings.

The trace is designed as a **first-class typed record** — `ExplainTrace` — not ad-hoc prints. It
has two renderers: a concise human view to **stderr** (keep stdout clean) and a machine-readable
`--explain=json`. The JSON is **hand-rolled** (matching the existing `write_json`/`write_diff_json`
pattern) so it adds **no `serde_json` runtime dependency**. The same `ExplainTrace` is reused
downstream: it is the golden-test fixture for the decision engine, and it becomes the per-image
`optimization` field of the PROJ-005 web-asset manifest. Its schema is recorded in **DEC-049**.

Design source: `docs/research/proj-002-design-format-engine.md` §"Explain trace" and
`-design-planner.md` §"Explain trace" (the fuller schema the planner will extend). This spec is
**render-only** — SPEC-048 already produces the `Candidate` array and winner; SPEC-049 populates
the trace and formats it.

## Goal

Add `--explain` (human, to stderr) and `--explain=json` (hand-rolled, stdout or stderr per the
existing convention) to `optimize`, driven by a typed `ExplainTrace` populated from SPEC-048's
decision record — deterministic and golden-testable, adding no dependency.

## Inputs

- **Files to read:**
  - `docs/research/proj-002-design-format-engine.md` §"Explain trace" — the per-run fields (input
    facts, features, profile/mode/target, each evaluated candidate, winner + reason, savings %).
  - `docs/research/proj-002-design-planner.md` §"Explain trace" — the richer
    `ExplainTrace { goal, objective, classification, shortlist, candidates[…], winner_format,
    win_reason, met_goal, warnings[], savings }` the planner will extend; shape v1 as a subset so
    PROJ-003/PROJ-005 grow it without a breaking rename.
  - `src/quality/decide.rs` (SPEC-048) — the `Candidate` array + winner this renders.
  - `src/cli/mod.rs` — the existing `write_json` / `write_diff_json` hand-rolled JSON writers to
    match style; the `optimize` arg struct + `run_optimize` to add the flag and the stderr channel.
- **Related code paths:** `src/analysis/mod.rs` — `class`/features surfaced in the trace (read
  only).

## Outputs

- **Files created:** none required (the `ExplainTrace` type may live in `src/quality/decide.rs`
  next to `Candidate`, or a small `src/quality/explain.rs`); a `tests/optimize_explain.rs`
  integration test + a golden JSON fixture under `tests/`.
- **Files modified:** `src/cli/mod.rs` — add `--explain[=json]` to `Optimize`; after the engine
  runs, build the `ExplainTrace` and render it (human→stderr; json→hand-rolled).
- **New exports:**
  - `pub struct ExplainTrace { input, features, class, profile, mode, target, candidates:
    Vec<CandidateTrace>, winner_format, win_reason, savings }` (a subset of the planner schema).
  - `pub fn ExplainTrace::render_human(&self, w: &mut impl Write)` and
    `pub fn ExplainTrace::write_json(&self, w: &mut impl Write)` — hand-rolled, no `serde_json`.
- **Database changes:** none.

## Acceptance Criteria

- [ ] `optimize --explain <file>` prints to **stderr** (stdout stays pipe-clean): input facts,
  detected features, class, profile/mode/target, each evaluated candidate
  (format/quality/score/bytes/met), the winner + a one-line reason, and source→output bytes +
  savings %.
- [ ] `optimize --explain=json <file>` emits the full `ExplainTrace` as **hand-rolled JSON**
  (feature vector + candidate array), suitable as a regression fixture — with **no `serde_json`
  runtime dependency** (`just deny` unchanged).
- [ ] Without `--explain`, behaviour and output are **byte-identical to SPEC-048** (explain is
  purely additive; the decision itself is unchanged).
- [ ] The trace is **deterministic and golden-testable**: no timestamps, no absolute paths in the
  metric portion; the same input+flags yields byte-identical JSON. A checked-in golden fixture is
  asserted.
- [ ] The "already-optimal, unchanged" case explains itself: the trace shows the candidates tried
  and "kept source (no candidate beat it)" as the winner reason.
- [ ] The `ExplainTrace` field set is a **forward-compatible subset** of the planner's schema
  (DEC-049) — PROJ-003 (planner objective/warnings) and PROJ-005 (manifest `optimization` field)
  extend it additively, no breaking rename.
- [ ] No panic on any input; `--explain` on an unsatisfiable-target run still renders a coherent
  best-effort trace (`met=false` surfaced), exit 0.

## Failing Tests

Written during **design**, BEFORE build. The renderers are pure over a synthetic `ExplainTrace`
(no encoding needed for the format tests); one integration test drives the real CLI + golden JSON.

- **`src/quality/explain.rs` (or `decide.rs`) unit tests — pure rendering**
  - `"human render lists every candidate + winner + savings"` — a synthetic trace with 3
    candidates: asserts each format/score/bytes line, the winner marker, and the savings % appear.
  - `"json render is hand-rolled, stable, and parseable-shaped"` — asserts the JSON string
    contains the schema key, a `candidates` array of the right length, and the winner field; and
    that it is byte-identical across two renders (determinism).
  - `"passthrough (no winner beat source) renders 'kept source' reason"`.
  - `"unsatisfiable target renders met=false without panic"`.
- **`tests/optimize_explain.rs` (integration — real CLI)**
  - `"optimize --explain writes the trace to stderr, leaves stdout clean"` — asserts stdout is the
    image bytes (or empty on passthrough) and the human trace is on stderr.
  - `"optimize --explain=json matches the checked-in golden fixture"` — byte-compare against
    `tests/fixtures/optimize_explain_golden.json` for a fixed synthetic input.
  - `"optimize without --explain is byte-identical to the SPEC-048 baseline"`.

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply
- `DEC-049` (emitted with this spec) — the `ExplainTrace` schema: the field set, the hand-rolled
  JSON (no `serde_json` in v1), the stderr-human / json channels, and the forward-compat subset
  contract (planner + manifest extend it).
- `DEC-048` — the decision record (`Candidate` array + winner + reason) this renders; SPEC-049 must
  not re-derive the decision, only format it.
- `DEC-024` — `optimize` shape; `--explain` is additive to it.
- `DEC-016` — the shipped bytes == measured candidate bytes, so the savings % in the trace is
  truthful.
- `DEC-047` — the `class` label surfaced (cosmetic one-word label, the only user-facing leak of
  classification).

### Constraints that apply
- `untrusted-input-hardening` — rendering never panics; a degenerate/unsatisfiable run still
  produces a coherent trace.
- `no-new-top-level-deps-without-decision` — JSON is hand-rolled like `write_json`/`write_diff_json`;
  **no `serde_json`**. (If serde is later promoted for the manifest, switching to `Serialize` is a
  future, non-blocking change — noted in DEC-049.)
- `ergonomic-defaults` — the full trace is **off by default** (SPEC-048's one-line summary is the
  default; `--explain` opts into detail); all of it goes to stderr so `-o -` pipes stay clean
  (AGENTS §11 logging rule).
- `test-before-implementation` — the golden fixture + pure-render tests are the contract.

### Prior related work
- `SPEC-048` (this stage) — produces the decision record; this spec renders it.
- `DEC-025` / `write_diff_json` (shipped) — the hand-rolled-JSON precedent to match; the `diff`
  command already emits machine JSON with no `serde_json`.

### Out of scope (for this spec specifically)
- The planner's richer trace fields (`goal`, `objective`, `warnings[]`, `met_goal`, per-candidate
  `scale_percent`/`excluded_reason`) — PROJ-003 adds them; v1 ships the subset.
- The manifest emission itself — PROJ-005 consumes `ExplainTrace`; this spec only makes it exist
  and be serializable.
- Applying `--explain` to commands other than `optimize`.

## Notes for the Implementer

- Populate `ExplainTrace` from SPEC-048's `Candidate` array — do **not** re-run the engine or
  recompute anything. Explain is a pure projection of the decision.
- Match the existing hand-rolled JSON writers exactly (escaping, key order, number formatting) so
  the golden fixture is stable across platforms; the metric portion must carry no wall-clock or
  absolute-path data (determinism is the whole point of the golden test).
- Shape the field names as the **subset** of the planner schema in `-design-planner.md`, so
  PROJ-003 and PROJ-005 extend rather than rename. Note this contract in DEC-049.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-049-explain-trace`
- **PR (if applicable):** see STAGE-012 ship log (opened + merged in the autonomous run).
- **All acceptance criteria met?** yes — `ExplainTrace` (a forward-compatible subset of the planner
  schema) + `CandidateTrace` in `src/analysis/decide.rs`, with `render_human` (stderr) and
  hand-rolled `write_json` (**no `serde_json`**, matching `write_json`/`write_diff_json`). `optimize`
  gains `--explain` (human→stderr) and `--explain=json` (→stdout); threaded out of
  `optimize_decide_one` (no re-run). 4 unit tests (incl. an exact-JSON golden + determinism) + 2
  integration tests. Green on default (475)/webp-lossy (482)/lean (475)/avif; fmt/clippy/deny green;
  no new dependency. Verified live: a photographic PNG auto-decides to JPEG (−43%) with a clear
  trace.
- **New decisions emitted:**
  - None beyond DEC-049 (the `ExplainTrace` schema / hand-rolled-JSON / subset contract). The pinned
    schema id is `crustyimg.optimize.explain/v1`.
- **Deviations from spec:**
  - **The "checked-in golden fixture" is an inline exact-JSON unit test over a *synthetic*
    `ExplainTrace`** (fixed field values → exact string), not a golden of real-encoded CLI output.
    This is fully deterministic and **cross-platform safe** — it involves no encoder bytes, no
    `log2`, and no paths — so it can't flake on the mac/Windows CI legs. A second integration test
    asserts `--explain=json` is byte-identical across two runs. Together they give the golden's
    regression guarantee without the 3-OS fragility.
  - **Channels:** `--explain` (human) → stderr; `--explain=json` → stdout. `--explain=json` combined
    with `-o -` would mingle JSON with the image on stdout — use `--out-dir`/a file (documented).
  - **Determinism guards:** the trace is path-free and renders floats at 2 decimals, so tiny
    cross-platform `log2` ULP differences can't change the JSON bytes.
- **Follow-up work identified:**
  - None. STAGE-012 is complete; PROJ-002 is ready to cut **0.3.0** (version bump + CHANGELOG — the
    release *tag/publish* is left to the maintainer as the outward-facing step).

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
   — Nothing major. The one judgement was the golden-fixture form: a real-output golden is fragile
   across 3 OSes, so I made the golden a synthetic-trace exact-string test + a determinism check —
   same guarantee, no flakiness.
2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. SPEC-048 had already computed the full per-candidate record internally, so SPEC-049 was a
   clean projection: thread quality into `SolvedCandidate`, build the trace, render. No re-run.
3. **If you did this task again, what would you do differently?**
   — Nothing substantive — building `ExplainTrace` + its renderers in `decide.rs` (pure, with the
   golden unit test) before the CLI wiring kept it low-risk, same as SPEC-048.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*

1. **What would I do differently next time?**
   — <answer>
2. **Does any template, constraint, or decision need updating?**
   — <answer>
3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
