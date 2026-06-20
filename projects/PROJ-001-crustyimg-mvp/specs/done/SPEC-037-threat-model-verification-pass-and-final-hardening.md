---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-037
  type: story                      # epic | story | task | bug | chore
  cycle: ship                      # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-001
  stage: STAGE-006
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-8
  implementer: claude-sonnet-4-6   # build runs on Sonnet (prescriptive prompt)
  created_at: 2026-06-19

references:
  decisions: [DEC-038, DEC-034, DEC-035, DEC-007]
  constraints:
    - untrusted-input-hardening
    - no-unwrap-on-recoverable-paths
    - clippy-fmt-clean
    - every-public-fn-tested
  related_specs: [SPEC-033, SPEC-034, SPEC-035, SPEC-036, SPEC-032, SPEC-010]

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-006's <capability>". Optional; null is acceptable.
value_link: >
  The STAGE-006 capstone (MVP exit gate): closes the last two concrete hardening
  gaps (resize output cap + save-recipe symlink guard) and records a threat-model
  verification pass confirming every SECURITY.md mitigation holds as built.

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
    - cycle: design
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: >
        Main-loop orchestrator work (incl. a post-build correction). Authored the
        spec + DEC-038 + the Sonnet build prompt + the SECURITY.md threat-model
        verification table. IMPORTANT lesson: the design premise "resize has no
        upper bound" was WRONG — SPEC-010 already shipped MAX_EDGE/MAX_AREA. The
        threat-model pass (its whole point) caught it; I corrected the spec/DEC and
        the code directly — dropped the redundant new const the build had added and
        tightened the existing MAX_AREA 256→128 Mpx (512 MiB RGBA) for decode
        symmetry (DEC-034). Read only part of Resize::apply at design → missed the
        guards below; read the WHOLE function next time.
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: 82390
      estimated_usd: 0.45
      duration_minutes: 8
      recorded_at: 2026-06-19
      notes: >
        Real metered subagent on Sonnet 4.6. subagent_tokens=82390,
        duration_ms=461563. estimated_usd at Sonnet list ($3/$15 per MTok,
        ~80/20). resize cap + edit --save-recipe symlink guard (reuse
        sink::reject_symlink_destination, pub(crate)). The build correctly
        DISCOVERED the pre-existing MAX_EDGE/MAX_AREA guards and flagged the
        overlap (its first attempt added a redundant const, removed in the
        architect correction). 411 tests green; clippy/fmt/lean/deny clean.
    - cycle: verify
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: 60000
      estimated_usd: 0.55
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: >
        ORDER-OF-MAGNITUDE ESTIMATE (~60k — larger because it doubled as the
        STAGE-006 security review) — read-only Explore subagent on Opus + gate
        re-runs (cargo test 411 ok / clippy / fmt / deny / lean). Verdict:
        APPROVED. Part A verified the corrected resize cap (no stray const; MAX_AREA
        = 134_217_728) + the save-recipe guard; Part B ran an adversarial sweep
        over the cumulative STAGE-006 surface (decode / paths / recipes / supply
        chain / error hygiene) — ALL output-write paths guarded, no panics on
        untrusted paths, every SECURITY.md mitigation confirmed, no unresolved
        high-severity finding.
    - cycle: ship
      agent: claude-opus-4-8
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: "Main-loop ship bookkeeping (merge dance + cost totals + reflection + archive + STAGE-006 close); not separately metered."
  totals:
    tokens_total: 142390
    estimated_usd: 1.00
    session_count: 4
---

# SPEC-037: threat-model verification pass and final hardening

## Context

**The STAGE-006 capstone — the MVP exit gate.** SPEC-033/034/035/036 hardened
decode, paths, recipes, and the supply chain. This spec closes the **last two
concrete runtime gaps** the prior specs' verify cycles surfaced and explicitly
deferred to here, then records a **threat-model verification pass** confirming
every `SECURITY.md` mitigation actually holds as built:

1. **Resize upscale-bomb (deferred from SPEC-035 / DEC-036).** *Verification
   finding:* `Resize::apply` was **already bounded** — SPEC-010 shipped an oversize
   cap (`MAX_EDGE = 50 000` per dimension + `MAX_AREA` on total pixels) with the op,
   so `resize exact 100000x100000` / `resize percent 1000000` was already rejected
   before allocation. Not an open hole. The only refinement (DEC-038): **tighten
   `MAX_AREA` from 256 Mpx → 128 Mpx (= 512 MiB RGBA)** so the resize output cap
   matches the decode allocation cap (DEC-034) — one cap, symmetric, no new const.
   (An earlier build draft added a redundant separate byte cap; it was removed.)
2. **`edit --save-recipe` symlink parity (deferred from SPEC-034).** `run_edit`
   writes the recipe via raw `std::fs::write` — unlike `Sink`, it does NOT reject
   a symlinked destination, so a planted symlink at the recipe path could redirect
   the write (a write-through-symlink, the gap SPEC-034 closed for image output).

Per **DEC-038** (the resize output cap) and **DEC-035** (the symlink-destination
policy, extended here), both are closed. The spec also adds a **`## Verification
(STAGE-006 exit gate)` section to `SECURITY.md`** walking the six threats against
the as-built code, and the **verify cycle doubles as an adversarial security
review** over the cumulative STAGE-006 surface. When this ships, STAGE-006 — and
the MVP's hardening exit gate — is complete. Parent: `STAGE-006` (backlog item
#5). Governing: **DEC-038**, **DEC-034**, **DEC-035**, **DEC-007**. No new dep.

## Goal

Close the resize output-size and `edit --save-recipe` symlink gaps with typed
errors, and record a threat-model verification pass in `SECURITY.md` confirming
every mitigation holds — completing the STAGE-006 exit gate.

## Inputs

- **Files to read:**
  - `src/operation/mod.rs` — `Resize::apply` (the single `(tw, th)` compute point
    to guard), `OperationError::Apply`.
  - `src/cli/mod.rs` — `run_edit` (the `std::fs::write(path, toml)` recipe-save at
    ~line 2251 to guard).
  - `src/sink/mod.rs` — `reject_symlink_destination` (SPEC-034; make it
    `pub(crate)` and REUSE it in `run_edit`), `SinkError::Traversal`.
  - `SECURITY.md` — the six-threat model to verify.
  - `decisions/DEC-038` (resize cap), `DEC-034` (decode limit — symmetric),
    `DEC-035` (symlink-destination policy), `DEC-007`.
- **External APIs:** none new (`std::fs::symlink_metadata`).
- **Related code paths:** `src/operation/`, `src/cli/mod.rs`, `src/sink/mod.rs`,
  `SECURITY.md`, `tests/`.

## Outputs

- **Files modified:**
  - `src/operation/mod.rs` — **tighten** the existing SPEC-010 `MAX_AREA` in
    `Resize::apply` from `268_435_456` (256 Mpx) → `134_217_728` (128 Mpx = 512 MiB
    RGBA, == the decode alloc cap, DEC-034); keep `MAX_EDGE` and the
    reject-before-allocate check (which already returns `OperationError::Apply { op:
    "resize", … }`). No new constant. Unit-tested.
  - `src/sink/mod.rs` — change `reject_symlink_destination` from private to
    `pub(crate)` (no behavior change) so the CLI can reuse it.
  - `src/cli/mod.rs` — in `run_edit`, before `std::fs::write(path, toml)`, call
    `crate::sink::reject_symlink_destination(Path::new(path))?` (maps the symlinked
    recipe path to `SinkError::Traversal` → exit 5). Unit/integration-tested.
  - `SECURITY.md` — add `## Verification (STAGE-006 exit gate)` mapping each of the
    six threats → its mitigation → the spec/DEC that delivers it → "verified".
    (Authored at design; the build leaves it as-is unless a finding requires an
    edit.)
  - `docs/api-contract.md` — note the resize output cap (exit 1). (Done at design.)
- **New exports:** `reject_symlink_destination` becomes `pub(crate)` (crate-internal).
- **Database changes:** none.

## Hardening policy (PINNED)

- **Resize output cap (DEC-038):** the existing SPEC-010 oversize check in
  `Resize::apply` (after the `(tw, th)` match, before allocation; covers all six
  modes) is **kept**, with `MAX_AREA` **tightened from 256 Mpx → 128 Mpx**
  (`134_217_728` px = 512 MiB RGBA, == the decode alloc cap, DEC-034). `MAX_EDGE`
  (50 000 px per-dimension) unchanged. Over-cap → `OperationError::Apply { op:
  "resize", … }` (CLI exit **1**). Reject, never clamp. `max`/`fit` never upscale →
  unaffected. **No new constant** (an earlier build draft's `MAX_RESIZE_OUTPUT_BYTES`
  was removed as redundant).
- **`edit --save-recipe` symlink guard (DEC-035):** `run_edit` calls the shared
  `sink::reject_symlink_destination` on the recipe path before writing — a
  symlinked recipe destination is refused with `SinkError::Traversal` (exit 5),
  matching the image-output guard. Enforced regardless of any overwrite flag.
- **Threat-model verification:** the `SECURITY.md` `## Verification` section is a
  factual record (each threat → mitigation → spec/DEC → verified), not code.
- **No behavior change for legitimate use:** normal resizes (< 512 MiB output) and
  non-symlink recipe paths are unaffected.

## Acceptance Criteria

- [ ] `Resize::apply` on an `exact` resize whose output buffer would exceed
  512 MiB (e.g. a 2×2 input → `exact 40000x40000`) returns
  `Err(OperationError::Apply { op: "resize", .. })` — and does so BEFORE
  allocating the buffer (the test stays cheap / cannot OOM).
- [ ] Same rejection for a `percent` resize that computes an over-cap output (e.g.
  a 100×100 input at `percent 2000000`) — proves the apply-time check covers the
  input-dependent modes.
- [ ] A normal resize (`exact 64x64`, `max 32`, `percent 50`) still succeeds (no
  regression); `max`/`fit` are never falsely rejected.
- [ ] An over-cap resize delivered **via a recipe** (`apply --recipe`) exits **1**
  (the same `apply` guard fires through the recipe path).
- [ ] `edit in.png --resize-max 8 --save-recipe LINK` where `LINK` is a symlink →
  refused with a traversal error (exit **5**); the symlink's target is untouched.
- [ ] A normal `edit --save-recipe r.toml` still writes the recipe (no regression).
- [ ] `SECURITY.md` has a `## Verification (STAGE-006 exit gate)` section covering
  all six threats with mitigation + spec/DEC references.
- [ ] `cargo deny check advisories bans sources licenses` green; **lean build**
  compiles; no new dependency; no `unwrap`/`expect`/`panic!` on the new non-test
  paths.

## Failing Tests

Written during **design**, BEFORE build. Native fixtures; the over-cap tests use
a TINY input + huge target so the guard fires before any large allocation (they
must never actually allocate gigabytes).

- **`src/operation/mod.rs` (unit, `#[cfg(test)] mod tests`)**
  - `"resize_apply_exact_rejects_oversized_output"` — a 2×2 image,
    `Resize::from_params(exact 40000x40000)` then `.apply(img)` →
    `Err(OperationError::Apply { op: "resize", .. })` (40000²·4 ≈ 6.4 GB > 512 MiB;
    guard fires before allocation).
  - `"resize_apply_percent_rejects_oversized_output"` — a 100×100 image, `percent
    2000000` → `Err(..)` (output ≈ 2 000 000×2 000 000).
  - `"resize_apply_normal_outputs_succeed"` — `exact 64x64`, `max 32`, `percent 50`
    on a small image all return `Ok` (no false rejection; `max` not upscaled).
- **`src/sink/mod.rs` (unit)**
  - `"reject_symlink_destination_is_crate_visible"` — a trivial call from a sink
    test confirming the now-`pub(crate)` fn still returns `Ok` for a regular path
    and `Traversal` for a symlink (`#[cfg(unix)]` for the symlink arm). (May reuse
    the existing SPEC-034 tests; add only if coverage moved.)
- **`tests/edit.rs` (integration, `#[cfg(unix)]` for symlink)**
  - `"edit_save_recipe_through_symlink_is_rejected"` — create a symlink
    `link.toml` → an outside target; `edit in.png --resize-max 8 --save-recipe
    link.toml -o out.png` → exit **5**; the outside target's bytes are unchanged.
  - `"edit_save_recipe_normal_path_still_works"` — `--save-recipe r.toml` to a
    plain path writes the recipe; exit 0 (regression guard).
- **`tests/apply_batch.rs` (integration)**
  - `"apply_recipe_with_oversized_resize_exits_1"` — a recipe with a `resize exact
    50000x50000` step + a small input image → `apply --recipe` exits **1** (the
    resize guard fires through the recipe path).

## Implementation Context

*Read this section (and the files it points to) before starting the build cycle.*

### Decisions that apply

- `DEC-038` — the resize output cap (512 MiB, apply-time, all modes, reject). The
  cap equals the decode alloc cap (DEC-034) by design. **Implement exactly.**
- `DEC-035` — the symlink-destination policy; `edit --save-recipe` reuses the same
  `reject_symlink_destination` the Sink uses (now `pub(crate)`).
- `DEC-034` — the symmetric decode allocation limit (context for the 512 MiB cap).
- `DEC-007` — typed errors; no `unwrap`/`expect`/`panic!` on the new non-test code.

### Constraints that apply

- `untrusted-input-hardening` (blocking) — the final consolidation/verification of
  the constraint.
- `no-unwrap-on-recoverable-paths`, `clippy-fmt-clean`, `every-public-fn-tested`.

### Prior related work

- `SPEC-033/034/035/036` (shipped) — the four prior STAGE-006 hardening items; their
  verify cycles surfaced the two gaps this spec closes.
- `SPEC-010` (shipped) — the `Resize` op (`from_params` + `apply`) being bounded.
- `SPEC-032` (shipped) — `run_edit` + `--save-recipe` (the raw write being guarded).

### Out of scope (for this spec specifically)

- A configurable `--max-pixels`/env override for decode AND resize — DEC-034's
  planned follow-up; a deliberate additive future, not this gate.
- TOCTOU hardening (`O_NOFOLLOW`) — `symlink_metadata`-then-write is sufficient
  (DEC-035 scope).
- `#[serde(deny_unknown_fields)]` strict recipe parsing (DEC-036 deferral) — not
  reopened here.
- Any NEW user-facing feature — STAGE-006 is hardening/assessment only.

## Notes for the Implementer

- **Resize guard is one block at one place.** Find the `let (tw, th) = match
  self.mode { … };` in `Resize::apply` and insert the cap check on the very next
  line, before the image is resized/allocated. One check covers all six modes,
  including `percent`/`cover`/`fill` (the input-dependent ones). Use the
  `OperationError::Apply { op: "resize", reason }` variant already used in that fn.
- **The over-cap tests must stay cheap.** Use a TINY input image (2×2 / 100×100)
  with a huge target so the guard rejects BEFORE the resize backend allocates the
  output — never construct a multi-GB buffer in a test.
- **Reuse, do not duplicate, the symlink check.** Make
  `sink::reject_symlink_destination` `pub(crate)` and call it from `run_edit`;
  do not write a second copy. It returns `SinkError::Traversal` → wrap as
  `CliError::Sink(...)` (exit 5), consistent with the rest of `run_edit`'s
  recipe-write error mapping.
- **`SECURITY.md` `## Verification` is authored at design** (below); the build does
  not need to change it unless a test/finding contradicts a claim — in which case
  fix the code, not the doc.
- Run clippy right after doc comments (the SPEC-031 `doc_lazy_continuation`
  lesson) and **run the lean build** (`cargo build --no-default-features`).

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `feat/spec-037-final-hardening`
- **PR (if applicable):** opened (see PR URL)
- **All acceptance criteria met?** yes
- **New decisions emitted:**
  - none (DEC-038 authored at design)
- **Deviations from spec:**
  - **Design correction (by the architect, post-build):** the build initially added
    a separate `MAX_RESIZE_OUTPUT_BYTES` const + check, having discovered the
    pre-existing `MAX_EDGE`/`MAX_AREA` guards (from SPEC-010). The architect's
    spec premise ("resize has no upper bound") was wrong — resize was already
    bounded. The redundant new const was **removed**; instead the existing
    `MAX_AREA` was **tightened from 256 Mpx → 128 Mpx** (512 MiB RGBA) for symmetry
    with the decode alloc cap (DEC-034). One area cap, no redundancy. DEC-038 +
    the spec context were corrected to reflect this. All 411 tests still pass
    (the resize tests reject the same oversized cases via `MAX_EDGE`/`MAX_AREA`).
- **Follow-up work identified:**
  - none for this stage

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — The spec said "immediately after `let (tw, th) = match self.mode {…};`" but
   there were already `MAX_EDGE` / `MAX_AREA` guards present from earlier hardening;
   I had to decide whether to replace or prepend. The instruction to "smallest correct
   change" resolved it: prepend the byte cap, leave existing guards in place.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — Nothing missing; DEC-038, DEC-035, DEC-034, and DEC-007 were all the right
   references. The note about `doc_lazy_continuation` was a useful reminder.

3. **If you did this task again, what would you do differently?**
   — Nothing substantial; the spec was prescriptive enough that the build was
   mechanical. Running `cargo test -- --list` immediately after writing tests
   confirmed name matches before the full test run.

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — **Read the whole function before claiming a gap.** My design premise — "resize
   has no upper output bound" — was wrong: SPEC-010 shipped `MAX_EDGE`/`MAX_AREA`
   with the op, and I'd only read `Resize::apply` through the Fill branch, missing
   the guards below it. The build *discovered* this and flagged the overlap; I
   corrected (dropped the redundant new const, tightened the existing `MAX_AREA` for
   decode-symmetry) and rewrote DEC-038 + the spec honestly. The system worked as
   designed — a verification pass is exactly the place a false premise should die —
   but I'd have saved a round-trip by reading the full `apply` at design time. For a
   *verification* spec especially, audit what's already there before asserting a hole.

2. **Does any template, constraint, or decision need updating?**
   — No template change. DEC-038 now records the honest outcome (tighten, don't add).
   The reusable lesson is process: **the verify cycle doubling as the security review
   was high-value** — Part B's adversarial sweep over the whole cumulative surface
   (every output-write path, panic-on-untrusted-input grep, each SECURITY.md row) is
   what makes the exit-gate record trustworthy, and it's worth doing as an explicit
   cumulative pass at the end of any hardening stage, not just per-spec.

3. **Is there a follow-up spec I should write now before I forget?**
   — None for STAGE-006 — **this completes it** (the MVP hardening exit gate). The
   security review surfaced no unresolved finding. Remaining open items are all
   deliberate, documented future work, not gaps: a configurable `--max-pixels`/env
   override (decode + resize, DEC-034/038), `O_NOFOLLOW`-grade TOCTOU hardening, and
   the standing `paste` advisory `ignore` to clear when upstream drops it. The MVP's
   functional + hardening surface is now complete; the natural next stage is
   **STAGE-007 (release & distribution)** — binaries, brew, crates.io.
