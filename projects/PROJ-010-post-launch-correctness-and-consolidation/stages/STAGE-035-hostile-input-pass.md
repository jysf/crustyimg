---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-035
  status: proposed
  priority: critical
  target_complete: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-07-26
shipped_at: null

value_contribution:
  advances: >
    Closes the launch-readiness gap on hostile/edge input behaviour. The checklist has
    always said "hold natively; confirm in the browser" and nobody has ever recorded the
    confirmation. A launch will bring unexpected inputs; this stage ensures the tool
    handles them predictably — no hang, clear message, documented exit code — on both
    the native CLI and the wasm build.
  delivers:
    - "A committed hostile-input corpus with driven results for every file on native CLI and headless wasm"
    - "No hang, clear message, documented exit code on every input in the set"
    - "The launch-readiness hostile-input blocker moved off 'hold natively; confirm in the browser' to a stated outcome"
    - "Fixes for any defects the pass discovers, landed in the same spec"
  explicitly_does_not:
    - "Redesign error handling, the pipeline, or any codec — this confirms and fixes, it does not rearchitect"
    - "Cover the browser-hosting half (demo UI surfacing, mobile behaviour) — that folds into the maintainer's mobile device test"
---

# STAGE-035: Hostile/edge input confirmation pass

## What This Stage Is

The second launch-gating stage. SPEC-107 (framed in PROJ-008's STAGE-033) drives a committed hostile/edge input corpus against the native CLI **and** the headless wasm build — truncated AVIF/JPEG, `.txt`→`.jpg`, zero-byte, decompression-bomb PNG, RAW near the 60 MP gate, empty-OBU AVIF — and records what holds. No hang, clear message, documented exit code on every input. Fixes anything it finds. Then updates `docs/launch-readiness.md` to reflect the driven result.

## Why Now

Sequenced immediately after the classifier fix (STAGE-034), so the launch has both a correct default path **and** documented behaviour on bad inputs. The hostile-input pass touches no engine source by default — it is a verification harness — but if it finds a defect, that defect is pre-launch and pre-fix, not post-launch and post-mortem.

## Success Criteria

- Every input in the committed hostile set has a recorded, driven result — native CLI and headless wasm, each file actually run, no hang, a message a user can act on, and the documented exit code.
- The empty-OBU AVIF still hits the SPEC-094 `debug_abort()` guard, shown by driving it — not by reading the guard — and the `debug_assertions` build is the leg that proves it.
- Any defects found are fixed in the same spec (touching engine-adjacent code if needed).
- The launch-readiness board item moves from "hold natively; confirm in the browser" to a stated outcome, with the genuinely browser-specific remainder named and left on the launch board (not silently dropped).
- All gates green on the full clean matrix.

## Scope

### In scope
- Build the committed hostile corpus.
- Build and run the harness against both native CLI and headless wasm.
- Fixes to anything the pass discovers, even if they land in engine-adjacent code.
- Update `docs/launch-readiness.md`.

### Explicitly out of scope
- **The browser half of the hostile-input pass**: whether the demo UI *surfaces* these errors legibly,
  and how a phone behaves on the large ones. That needs a real device and a human looking at a screen,
  so it folds into the maintainer's mobile test on the launch board
  ([[never-drive-the-maintainers-live-browser]]). This stage covers the wasm build driven headlessly,
  which is where the engine behaviour actually lives.
- **Platform-aware RAW gating.** SPEC-107 drives inputs either side of the current global 60 MP gate
  and records what happens; whether that gate should ever become device-dependent is decided by the
  mobile test, not here. (Carried forward from STAGE-033, where this fence was written — it was absent
  from the PROJ-010 draft of both this stage and STAGE-038.)
- Any engine redesign, classifier work, or CLI surface changes.
- New codecs, formats, or capabilities.

## Spec Backlog

- [ ] SPEC-107 (frame) — **hostile / edge input confirmation pass.** An open blocker in
  `docs/launch-readiness.md` ("hold natively; confirm in the browser"), and browser confirmation has
  never been recorded either way. Framed as a spec, not a chore, because it can find real defects and
  needs stated acceptance criteria. Drive the native CLI **and** the wasm build against a hostile set
  and record what holds: a truncated AVIF and JPEG; a `.txt` renamed `.jpg`; a zero-byte file; a
  decompression-bomb PNG (small file, huge canvas); a RAW just under and just over the 60 MP demo
  gate; an AVIF with an empty OBU (confirm the SPEC-094 `debug_abort()` guard still fires — per
  [[a-thread-boundary-does-not-catch-abort]] a thread boundary will not save us). Acceptance: no hang,
  no cryptic failure, a clear message on every input, and the documented exit codes. **Split by
  design:** everything above is testable without a browser and is repo work; what remains genuinely
  browser-specific is whether the demo *surfaces* those errors clearly and how a phone behaves on the
  big ones — that part folds into the maintainer's mobile device test and stays on the launch board.
  Touches no engine source (verification only, unless it finds something). Complexity **S–M**.

**Count:** 0 shipped / 0 active / 1 pending

> **Provenance.** SPEC-107 was framed in PROJ-008's STAGE-033 and its full detail is inlined above
> rather than cross-referenced, because that stage moved here as STAGE-038 *without* SPEC-107 — a
> pointer to it would now dangle.

## Design Notes

- **Split by design: engine-facing vs browser-facing.** The engine-facing half (native CLI + headless wasm) is repo work and lives here. The browser-facing half (demo UI surfacing, mobile behaviour) is a human-with-a-phone test and folds into the maintainer's mobile device test on the launch board. This split is deliberate and documented — do not let the browser half drift from the launch board back into this spec.
- **The empty-OBU AVIF is a specific verify target.** SPEC-094's `debug_abort()` guard was the fix; driving the input that triggered it confirms it still fires. Per [[a-thread-boundary-does-not-catch-abort]], a thread boundary will not save us — drive the `debug_assertions` build specifically.
- **Harness design: deterministic exit-code check, not "did it print something?"** The harness should assert the exit code matches the expected code for each input class, and that stderr contains a non-empty, non-cryptic message.
- **A recorded "it held" counts only if the run happened.** An untested assumption does not close the launch-readiness item — that is the whole reason this spec exists rather than a checklist tick. [[a-plausible-test-result-is-not-a-checked-one]]
- **`docs/launch-readiness.md:34` is stale and should be corrected while this stage is in the file.** It still reads "Mobile — ⚠ STILL OPEN, the remaining cross-browser blocker", but SPEC-101's record shows that gate was closed (iOS Safari + DuckDuckGo PASS on real devices; Android Chrome untested, accepted on maintainer judgment). The launch board is maintainer-owned, so this is a correction to a stale line, not a re-grading — but left alone it will make a future session re-open a closed gate.

## Dependencies

### Depends on
- STAGE-034 (classifier fix) — sequenced before this stage so the engine is on a known-correct baseline before we test hostile inputs.
- The shipped CLI and wasm build from PROJ-008.

### Enables
- The launch board hostile-input blocker moving from "assumed" to "measured."

## Stage-Level Reflection

*Filled in when status moves to shipped.*
