---
project:
  id: PROJ-013
  status: proposed
  priority: medium
  target_ship: null

repo:
  id: crustyimg

created_at: 2026-08-23
shipped_at: null

value:
  thesis: >
    ⚠ **This is a continuation lane, not a bounded wave, and that is a deliberate
    choice rather than an oversight.** PROJ-010 shipped its code and met all five
    of its success signals, but left ~53 real items behind — measured defects,
    release-safety instruments, and housekeeping. They needed a home that
    `just backlog` can see, because the alternative is a document nothing reads,
    and this repo has now lost work that way four times in one week.
    The honest claim is therefore modest: **crustyimg should keep the defects it
    has already found from reaching users, and keep the instruments that find
    them working.** That is testable per item — every entry here names a defect
    or a broken guard — even though the set as a whole does not describe one
    outcome.
  beneficiaries:
    - "Users hitting defects crustyimg has already measured and not yet fixed — silent multi-image data loss chief among them"
    - "The next orchestrator, who gets one place to look rather than seven stages spread across a shipped project"
    - "The guards themselves — cost-audit went vacuous the moment a new project activated, and nothing announced it"
  success_signals:
    - "Every item here either ships, or is cancelled with a reason — none is carried into a fourth project untouched"
    - "The measured silent-data-loss defects (multi-page TIFF, multi-size ICO) are fixed or explicitly declined with a ruling"
    - "No guard in this repo can report success having checked nothing — every gate reports what it covered"
  risks_to_thesis:
    - "⚠ A continuation lane with no end state is exactly the shape that accumulates. PROJ-010 became one without anyone deciding it; this one starts as one ON PURPOSE, which only helps if it is triaged on a schedule rather than when someone asks"
    - "~10 of the items here are things nobody will do. Carrying them makes the real ones harder to see, and the 2026-08-23 triage says which are which"
    - "STAGE-047 is template feedback, not crustyimg work. It has no momentum because it is in the wrong repo, and moving it here does not fix that"
---

# PROJ-013: Carried Correctness and Instruments

## What This Project Is

**The home for what PROJ-010 left behind.** PROJ-010 shipped — two releases, 18 specs, all five
success signals met — and still had ~53 open items: measured defects nobody had got to,
release-safety instruments earned by shipped work, and housekeeping.

This project exists so those items live somewhere `just backlog` reads. ⚠ **It is a lane, not a
wave**, and the brief says so rather than manufacturing a thesis to disguise it.

## Why Now

**Because the alternative is proven to fail.** In one week this repo lost: 358 lines of measured
research in `docs/backlog.md`; four CLI-surface findings that had been found and lost at least
twice; three items living only in `feedback/`; and a **decided `.cube` LUT feature invisible for 13
days** in a triage document no command reads. AGENTS §10 now says a decision that outlives its
session gets a `- [ ]` where `just backlog` reads it, or it is not decided. **These items needed
that place to exist.**

## Success Criteria

- **Every item ships or is cancelled with a reason.** None is carried into a fourth project
  untouched. ⚠ That is the criterion that stops this becoming a graveyard.
- The measured **silent-data-loss** defects (multi-page TIFF, multi-size ICO — both reported as a
  size win today) are fixed, or explicitly declined with a ruling.
- **No guard can report success having checked nothing.** Every gate reports what it covered.

## Scope

### In scope
- Everything carried from PROJ-010: five stages moved whole, and continuation stages for the two
  that could not move.

### Explicitly out of scope
- **Anything with a home elsewhere.** PROJ-011 owns invocation consistency and recipe reach;
  PROJ-012 owns animated output and ICC transforms. ⚠ **Do not pull them back here.**
- **New capability.** If an item is a feature rather than a defect or an instrument, it does not
  belong in this project's thesis and should be said so plainly.

## Stage Plan

**Five stages moved whole from PROJ-010** (none carried a shipped spec, so all could be re-homed):

- [ ] **STAGE-036** — engineering quality and code health (3)
- [ ] **STAGE-037** — post-launch CLI surface (2, on hold)
- [ ] **STAGE-038** — post-launch polish and repo housekeeping (11)
- [ ] **STAGE-047** — framework and harness tooling (10). ⚠ **Template feedback in the wrong repo.
  Route it upstream or cancel it** — moving it here did not give it momentum.
- [ ] **STAGE-048** — multi-image input completeness (4). ⚡ **Contains measured silent data loss.**

**Two continuation stages**, because ⚠ **a stage with shipped specs closes in place and cannot be
re-homed** — STAGE-042 and STAGE-046 stay in PROJ-010 as shipped:

- [ ] **STAGE-051** — release-safety instruments, continued (from STAGE-042).
- [ ] **STAGE-052** — output fidelity on shipped verbs, continued (from STAGE-046).

**Count:** 0 shipped / 0 active / **7 pending** — re-derive with a grep you just ran.

## Dependencies

### Depends on
- **PROJ-010 shipped** — this is its continuation.
- ⛔ **Two maintainer rulings** carried in with the work: the **ICO round-trip** (warn / fix /
  accept — a real fix changes bytes) and the **`-o`-extension pin**.

### Enables
- PROJ-011 and PROJ-012 staying narrow, because there is somewhere else for good work to go.

## Project-Level Reflection

*Filled in when status moves to shipped.*
