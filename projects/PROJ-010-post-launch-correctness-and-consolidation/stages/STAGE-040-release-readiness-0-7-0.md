---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-040
  status: proposed
  priority: critical
  target_complete: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-09
shipped_at: null

value_contribution:
  advances: >
    PROJ-010 fixed four defects on the flagship paths and none of them reached a
    user, because the released binary predates every one. This stage closes the
    gap between `main` and what `brew install` hands out — and fixes the one
    README promise the code cannot keep before that README renders on the
    crates.io crate page.
  delivers:
    - "`wasm::transform` runs the bundled recipes the README says it runs"
    - "0.7.0 cut and live on crates.io / Homebrew / Releases, carrying every PROJ-010 fix"
    - "the npm package back in step with the crate version"
  explicitly_does_not:
    - "Add capability. This ships what is already on main and fixes one false claim."
    - "Fire the tag push — that is maintainer-authorized (RELEASING.md), as is the npm publish"
    - "Take on the post-launch stages (036/037/038) or the three filed follow-ups"
---

# STAGE-040: release readiness for 0.7.0

## What This Stage Is

The stage that makes PROJ-010's work reach a user. STAGE-034, 035 and 039 fixed the
classifier blow-up, hostile-input behaviour and three shipped-verb defects — **and every
one of those fixes is invisible to anyone who installs crustyimg the normal way**, because
`v0.6.0` was tagged 2026-07-24 and all of it landed after.

Two items: one code fix that must precede the cut, and the cut itself.

## Why Now

- **The released CLI is the broken one.** 63 commits and 13 `src/` files separate `v0.6.0`
  from `main`. `brew install crustyimg` today gives you the 18.5× classifier blow-up, the
  silently-succeeding truncated JPEG, seven verbs returning sideways images, and a `build`
  that cannot run any bundled recipe. The demo is fine — `pages.yml` redeploys from `main`
  — so the gap is invisible from the page the launch points at, which is exactly why it went
  unnoticed until the PROJ-010 close-out.
- **A launch post that mentions the CLI would point at that binary.** This is the last thing
  standing between the finished correctness work and a defensible Show HN.
- **The README advertises a wasm path that errors**, and the README renders on the crates.io
  crate page — so cutting the release without fixing it publishes the false claim wider.

## Success Criteria

- `wasm::transform` runs all three bundled recipes, and the README's claim about them is true
  — proven by driving, not by reading the diff.
- 0.7.0 is live on crates.io, Homebrew and the Releases page, and a fresh install of it
  reproduces the fixed behaviour on the four defects PROJ-010 closed.
- `CHANGELOG.md`'s `[Unreleased]` content moves into a dated `0.7.0` section (it was written
  in advance — see `de097da`).
- The npm package is either republished in step with the crate, or its staleness is a
  recorded, deliberate decision rather than an oversight.
- The full pre-tag gate is green (`RELEASING.md` step 4).

## Scope

### In scope
- The `wasm::transform` terminal-`optimize` fix and the DEC-087 amendment it requires.
- The 0.7.0 version bump, CHANGELOG promotion, gate run, and channel verification.
- Deciding the npm republish.

### Explicitly out of scope
- **Firing the tag push and the npm publish.** Both are irreversible and maintainer-authorized
  (`RELEASING.md`; the npm gate is SPEC-076's). Prepare through the release commit and stop.
- The three filed follow-ups: `build` not threading the truncated-JPEG warning, orphaned
  artifacts when a content change flips the decided extension, and the `escape_json` tail.
  None returns a wrong file; all are post-launch.
- STAGE-036/037/038.
- Any new capability.

## Spec Backlog

- [ ] SPEC-112 (design written 2026-08-09) — **`wasm::transform` runs the bundled recipes.**
  The one call site that still hands a terminal `optimize` step to `build_pipeline`. Driven:
  all three bundled recipes fail with `unknown operation 'optimize'`. README:34–36 tells
  readers this path works. Complexity **S**.
- [ ] (chore) — **cut 0.7.0.** Follow `RELEASING.md`: bump `Cargo.toml`, promote the
  `[Unreleased]` CHANGELOG section (already written), run the full gate, prepare the release
  commit, then **stop** and hand the tag push to the maintainer. Decide the npm republish and
  record it either way. Complexity **S–M**.

**Count:** 0 shipped / 0 active / 1 spec + 1 chore pending

## Design Notes

- **A minor bump, not a patch.** Orientation baking changes output dimensions on any
  orientation-bearing input, and `edit --save-recipe` changes the shape of what it writes.
  Both are behaviour changes on shipped verbs, which is a minor under the semver discipline
  this repo has followed since 0.5.0.
- **The release is larger than PROJ-010.** SPEC-103, 104 and 105 also missed the `v0.6.0`
  cut, so 0.7.0 additionally carries the demo's RAW support and the first classifier fix.
  The CHANGELOG already reflects this; do not narrow it back to PROJ-010.
- **The npm package is version-coupled.** `pkg/package.json` reads 0.6.0 and tracks the crate
  version, so the cut leaves npm stale unless republished — and that publish is permanent and
  maintainer-gated.
- **Why this is a new stage rather than a reopened STAGE-039.** That stage has shipped specs
  and is closed; reopening it would relocate finished work
  ([[a-stage-with-shipped-specs-cannot-be-re-homed]]). This is its continuation.

## Dependencies

### Depends on
- STAGE-034, STAGE-035, STAGE-039 — the fixes this stage delivers to users.
- `de097da` — the CHANGELOG content and the de-staled launch board.

### Enables
- The Show HN go/no-go, which after this is maintainer-only: the device pass, the install
  one-liner re-verification at 0.7.0, the post draft, and the decision itself.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
