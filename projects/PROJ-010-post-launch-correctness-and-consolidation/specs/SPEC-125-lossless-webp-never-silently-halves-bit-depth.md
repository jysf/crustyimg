---
task:
  id: SPEC-125
  type: bug
  cycle: design
  blocked: true
  priority: high
  complexity: S

project:
  id: PROJ-010
  stage: STAGE-042
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5
  created_at: 2026-08-18

references:
  decisions:
    - DEC-095
    - DEC-090
    - DEC-019
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-121
    - SPEC-090

value_link: >
  A default-path silent downgrade on a shipped verb, reported with a perfect
  quality score. `convert --format webp` halves a >8-bit source and prints
  `ssim 100.0` — the honest-size line reads as reassurance for the one thing
  that went wrong.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Promoted from the
        STAGE-042 backlog item SPEC-121's punch-list cycle filed and measured.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-125: lossless WebP never silently halves bit depth

## Context

**Measured on SPEC-121's branch binary, 2026-08-18**, 32×32 16-bit RGB PNG:

```
convert --format webp
  png → webp · 4791 → 686 B (86% smaller) · ssim 100.0
```

The output round-trips as **8-bit**. `web` reaches it too — `optimize`'s
smallest-candidate search picks WebP for that fixture.

Two defects, and the second is the one that stings:

1. **The downgrade is silent and on the DEFAULT path.** SPEC-121's Call 3 scoped
   its diagnostic to JPEG + lossy WebP. `image`'s *lossless* WebP encoder — **no
   feature flag, always built** — has no 16-bit mode either, so it takes the same
   *"automatically convert the image to some color type supported by the encoder"*
   path with no diagnostic.
2. ⚡ **`ssim 100.0` is reported while the depth is halved.** The SSIM figure is
   computed on **8-bit renderings**, so it structurally *cannot see* the loss it is
   reporting on. Combined with SPEC-090's honest-size line, the output reads as
   *"86% smaller, pixel-perfect"* at the exact moment half the depth was thrown
   away. **A metric that cannot see a defect is worse than no metric**, because it
   converts silence into positive reassurance.

Full evidence: STAGE-042's backlog item, filed by SPEC-121's punch-list cycle.
**Read it; do not re-derive it.**

## The design calls — settled here

### Call 1 — widen the diagnostic to every 8-bit-only target, derived from code

SPEC-121 deliberately did not reopen this. Its Call 3 warns for JPEG and lossy
WebP. The rule should be **"the target format cannot hold the source's depth"**,
not a hand-maintained list of two.

⚠ **Derive the set from the encoder capabilities, not from this spec.** Candidates
visible at `src/sink/mod.rs:216-226` are `Gif`, `Bmp`, `Ico`, lossless `WebP` —
**and `Tiff` and `Png` are believed to be 16-bit-capable, so they must NOT warn.**
That is a prior to check, not a conclusion
[[a-measurement-specs-cost-lives-in-the-refutation]]. A hard-coded list is exactly
the shape that goes stale [[mechanical-sweeps-need-a-mechanical-check]].

### Call 2 — the SSIM line must not claim a perfect score across a depth change

This is the half that makes the bug dangerous, and it is **not** fixed by the
warning alone — a user reading `ssim 100.0` has been told the opposite of the
truth by the tool's own quality instrument.

Options, and the build picks one **with the reasoning recorded in the DEC**:
suppress the figure when depth was reduced; qualify it (*"ssim 100.0 (8-bit
comparison; source was 16-bit)"*); or compute it at source depth if the scorer
can. **Do not leave a bare `100.0`.**

⚠ **Do not silently change what SSIM means elsewhere.** DEC-019 anchors the
scorer for the byte-budget search; a change there perturbs `optimize`'s candidate
selection. If your fix would touch that path, **stop and report** — this spec is a
reporting fix, not a search change.

### Call 3 — do not reopen SPEC-121's narrowing rule

The colour-type/bit-depth preservation rule is settled and shipped. This spec adds
a diagnostic where the *target format* cannot hold what the pipeline correctly
preserved. No `Operation` body changes.

## Acceptance Criteria

- [ ] **AC-1.** `convert --format webp` on a >8-bit source **warns on stderr**;
      `-o -` stdout stays pure WebP (AGENTS §11).
- [ ] **AC-2.** The warning fires for **every** 8-bit-only target, and **does not**
      fire for targets that genuinely hold the depth. The set is **derived and
      cited**, not hard-coded from this spec's candidate list.
- [ ] **AC-3.** **`web` / `optimize` reach it too** — driven, not reasoned, since
      the smallest-candidate search is how most users hit this.
- [ ] **AC-4.** **No bare `ssim 100.0` across a depth change** (Call 2), pinned by
      a test asserting the rendered line.
- [ ] **AC-5.** **A negative control** — an 8-bit source through the same verbs
      warns **not at all** and its output is byte-identical to `main`.
- [ ] **AC-6.** **DEC-019's search path is unchanged** — `optimize`'s candidate
      selection byte-identical to `main` on the corpus.
- [ ] **AC-7.** Clean full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequential:
      default, `--no-default-features`, `--features webp-lossy`. Clippy and
      `fmt --check` each. Then read the CI legs individually.

## Failing Tests

- **`tests/sink.rs`**
  - `"lossless_webp_reports_the_depth_downgrade"` — AC-1. **RED.**
  - `"eight_bit_only_targets_all_warn_and_others_do_not"` — AC-2. **RED.**
  - `"ssim_line_is_qualified_across_a_depth_change"` — AC-4. **RED.**
  - `"eight_bit_source_warns_nowhere"` — AC-5. Passes today; the control.

## Implementation Context

### Out of scope
- SPEC-121's narrowing rule (Call 3), and any `Operation` change.
- Adding 16-bit WebP support. Upstream has no encoder; that is a dependency
  question, not this fix.
- Changing which candidate `optimize` picks (AC-6 pins it).

## Notes for the Implementer

- ⚠ **Blocked on SPEC-121 merging** — it owns `src/sink/mod.rs`'s diagnostic, and
  this widens it.
- **Budget ~150 exchanges.** Never poll CI; background `gh pr checks --watch`.
- macOS has no `timeout(1)`. `git commit -s`. **Own git worktree.** **Do not merge
  the PR. Do not bump the version.**
- Follow `closing-steps-snippet.md`, including `just advance-cycle SPEC-125 verify`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
- **Deviations from spec:**
- **Follow-up work identified:**

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?**
2. **Was there a constraint or decision that should have been listed but wasn't?**
3. **If you did this task again, what would you do differently?**

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
