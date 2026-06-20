---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-037
  type: story                      # epic | story | task | bug | chore
  cycle: verify                    # frame | design | build | verify | ship
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
    - cycle: build
      agent: claude-sonnet-4-6
      interface: claude-code
      tokens_total: null
      estimated_usd: null
      duration_minutes: null
      recorded_at: 2026-06-19
      notes: "STAGE-006 capstone: resize output 512MiB cap in Resize::apply (all modes, DEC-038) + edit --save-recipe reuses sink::reject_symlink_destination (pub(crate), DEC-035); reuse OperationError::Apply (exit 1) / SinkError::Traversal (exit 5); std-only, no new dep"
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 1
---

# SPEC-037: threat-model verification pass and final hardening

## Context

**The STAGE-006 capstone — the MVP exit gate.** SPEC-033/034/035/036 hardened
decode, paths, recipes, and the supply chain. This spec closes the **last two
concrete runtime gaps** the prior specs' verify cycles surfaced and explicitly
deferred to here, then records a **threat-model verification pass** confirming
every `SECURITY.md` mitigation actually holds as built:

1. **Resize upscale-bomb (deferred from SPEC-035 / DEC-036).** `Resize` has no
   upper output bound, so a recipe or CLI `resize exact 100000x100000` (≈40 GB)
   or `resize percent 1000000` would drive an enormous allocation. The decode
   limit (DEC-034) bounds *inputs*, not resize *outputs*.
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
  - `src/operation/mod.rs` — add `const MAX_RESIZE_OUTPUT_BYTES: u64 = 512 * 1024
    * 1024;`; in `Resize::apply`, after `(tw, th)` is computed and BEFORE the
    output buffer is allocated, reject when `tw as u64 * th as u64 * 4 >
    MAX_RESIZE_OUTPUT_BYTES` with `OperationError::Apply { op: "resize", reason }`.
    Unit-tested.
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

- **Resize output cap (DEC-038):** in `Resize::apply`, immediately after the
  `(tw, th)` match that computes target dims (covers all six modes), BEFORE
  building the output image: `if (tw as u64) * (th as u64) * 4 >
  MAX_RESIZE_OUTPUT_BYTES { return Err(OperationError::Apply { op: "resize",
  reason: format!("resize output {tw}x{th} exceeds the {} byte limit",
  MAX_RESIZE_OUTPUT_BYTES) }); }`. `MAX_RESIZE_OUTPUT_BYTES = 512 MiB` (== the
  decode alloc cap, DEC-034). Reject, never clamp. Maps to CLI exit **1**
  (`CliError::Operation(_) => 1`). `max`/`fit` never upscale → unaffected.
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
  - The existing `MAX_EDGE` / `MAX_AREA` checks in `Resize::apply` (already present
    from a prior hardening pass) are retained unchanged. The new
    `MAX_RESIZE_OUTPUT_BYTES` guard is inserted immediately after the `(tw, th)`
    match and fires first, so its semantics are correct. No behavior regression.
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
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
