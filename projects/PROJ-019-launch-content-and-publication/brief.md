---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-019                      # stable, zero-padded, never reused
  status: proposed                  # proposed | active | shipped | cancelled
  priority: medium                  # critical | high | medium | low
  target_ship: null                 # optional: YYYY-MM-DD

repo:
  id: crustyimg                     # must match .repo-context.yaml

created_at: 2026-11-11
shipped_at: null

# Business value. Testable claim, not marketing copy.
value:
  thesis: >
    crustyimg is correct and published on every channel, and almost nobody knows it exists.
    A launch built on honest numbers and honest limits, stated up front, earns more lasting
    attention than one that lets readers find the losses themselves.
  beneficiaries:
    - "Prospective users who have never heard of crustyimg"
    - "The maintainer, who writes and publishes the post in their own voice"
  success_signals:
    - "A written publication plan: channels, order, schedule, and the asset each one needs"
    - "Factual scaffolding where every number is re-derived from a command at the then-current release"
    - "A demo asset made from the live deployed demo, not a local build"
    - "Install paths and the README's commands driven at the release being announced, right before publishing"
  risks_to_thesis:
    - "The repo has understated this work since 2026-08-16: three of four items were reported done outside the repo, so the real remaining work may be one item or none"
    - "Numbers go stale fast; anything prepared now must be re-derived at publication time"
---

# PROJ-019: Launch content and publication

## What This Project Is

The work that turns a shipped, correct product into a launched one: where to publish, in what
order, with what asset, and the facts a hostile reader will check. It stops at the maintainer's
own words. Writing the post, publishing it and the go/no-go call all stay with the maintainer.

## Why Now

It doesn't have to be now. This project exists to give the work a **home**, not a slot.
STAGE-041 was created in PROJ-010 (2026-08-10), carried to PROJ-011 on 2026-08-23 for continuity,
and parked there on 2026-11-05. In both projects it was flagged as not sharing the thesis and not
to be counted at reflection. Giving it its own project (2026-11-11, maintainer request) stops it
riding along in whatever project is active and distorting that project's counts.

## Success Criteria

- The maintainer has a publication plan, a demo asset and checked facts ready to drop into their own post.
- Every figure in the scaffolding comes from a command run at the release being announced.
- The install paths and the live demo were driven, not read, right before publication.

## Scope

### In scope
- STAGE-041's four items: the publication plan, the factual scaffolding, the demo asset, and the pre-publication verification pass.

### Explicitly out of scope
- Writing the post. The narrative is the maintainer's voice.
- Publishing anything. Every outward-facing post is maintainer-authorized.
- The go/no-go decision. That stays on `docs/launch-readiness.md`.
- Product work of any kind. A defect found while preparing the facts goes to the active project.

## Stage Plan

- [ ] STAGE-041 (on_hold, parked 2026-11-05) — Launch content and publication plan. Moved from
  PROJ-011 on 2026-11-11 with its id kept. ⚠ **Get the real status from the maintainer before
  re-planning it**: three of its four items were reported substantially done outside the repo.

**Count:** 0 shipped / 0 active / 1 pending (on hold)

## Dependencies

### Depends on
- The maintainer deciding they're ready to publish. That is what un-parks STAGE-041.
- A current release to announce. Re-derive numbers and re-run the install checks at whatever
  release is current then, not at 0.7.0 as the stage's text still says.

### Enables
- Adoption, which `docs/roadmap.md` Track B is built around.

## Project-Level Reflection

*Filled in when status moves to shipped.*
