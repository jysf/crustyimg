---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-040
  status: active                     # proposed | active | shipped | cancelled | on_hold
  priority: critical
  target_complete: null
  # Both backlog items are done as far as this repo can take them. The stage
  # stays `active`, not `shipped`, because the last step is outward-facing and
  # maintainer-authorized: the `v0.7.0` tag push (RELEASING.md steps 6-7) and
  # the channel verification (step 8). It ships when the tag fires and
  # crates.io / Homebrew / Releases carry 0.7.0.

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

- [x] SPEC-112 — **`wasm::transform` runs the bundled recipes.** SHIPPED 2026-08-10, PR #144
  (`3bd26b5`), 27/27 required CI legs green. All 10 ACs met; verify returned ⚠ PUNCH LIST and
  closed both items on the branch. `transform` now strips the terminal marker via
  `split_terminal_optimize`, **moved** (not copied) to `src/recipe/mod.rs` — the spec's
  alternative of widening it in `cli::optimize` was impossible, since `cli` and `wasm` are
  mutually exclusive `#[cfg(target_arch)]` trees. Driven on both sides: all three bundled
  recipes returned `unknown operation 'optimize'` on `main` and succeed on the branch, and the
  markerless demo shape is byte-identical across the change. DEC-087 amended. Cost
  109,071,623 tokens / $76.16.
- [x] (chore) — **cut 0.7.0.** DONE as far as this repo can take it, 2026-08-10. Release commit
  prepared; **not tagged** — that is maintainer-authorized and is the only thing left.
  `just release 0.7.0` bumped `Cargo.toml` + `Cargo.lock` and both guards passed (tag-matches-
  crate-version, CHANGELOG-has-a-section). `[Unreleased]` rolled into `## [0.7.0] - 2026-08-10`.
  Two repairs made in the same pass: the CHANGELOG had **no entry for SPEC-112** (it was written
  before that spec landed), and the `[Unreleased]` link reference still pointed at
  `v0.5.0...HEAD` after the 0.6.0 cut. npm republish decided — see below.

**Count:** 1 shipped / 1 chore done-pending-maintainer / 0 pending

## Design Notes

- **A minor bump, not a patch.** Orientation baking changes output dimensions on any
  orientation-bearing input, and `edit --save-recipe` changes the shape of what it writes.
  Both are behaviour changes on shipped verbs, which is a minor under the semver discipline
  this repo has followed since 0.5.0.
- **The release is larger than PROJ-010.** SPEC-103, 104 and 105 also missed the `v0.6.0`
  cut, so 0.7.0 additionally carries the demo's RAW support and the first classifier fix.
  The CHANGELOG already reflects this; do not narrow it back to PROJ-010.
- **The npm package is version-coupled — and it is already TWO minors behind, not one.**
  *Corrected 2026-08-10 against the registry; the original note here was wrong.* `pkg/` is
  **gitignored** (`.gitignore:38`) — a generated `wasm-pack` artifact whose `version` is copied
  from `Cargo.toml` and never hand-edited (`npm/package.overrides.json` deliberately does not
  override `version`, DEC-067). So the local `pkg/package.json` reading 0.6.0 is a stale build
  output, not a maintained file, and the bump propagates to it automatically on the next
  `just wasm-npm-pkg`.
  The registry tells the real story: the published package is **`crustyimg-wasm`** (renamed
  from the crate by `wasm-npm-finalize.mjs`), and it has **exactly one version, 0.5.0,
  published 2026-07-21**. There is no 0.6.0 on npm — the v0.6.0 cut (2026-07-24) never
  republished it, and nobody noticed. `crustyimg` itself is not a published npm name.
  This sharpens the decision rather than softening it: **npm's only release predates
  SPEC-112**, so a JS consumer following `README.md:34-36` today installs the very build whose
  `transform` cannot run a bundled recipe. Republishing at 0.7.0 is the only thing that makes
  that README claim true for the audience it is written for. The publish itself stays
  irreversible and maintainer-gated (SPEC-076; `just wasm-npm-smoke` does not publish).
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

## The npm republish — decided 2026-08-10

**Decision: republish `crustyimg-wasm` at 0.7.0.** Recommended, not performed — the publish is
irreversible and maintainer-gated (SPEC-076), so it sits in the handoff below alongside the tag.

The reasoning changed once the registry was checked rather than the working tree. The stage
originally recorded "`pkg/package.json` reads 0.6.0", which is a **generated, gitignored**
`wasm-pack` artifact, not a maintained file. On npm, `crustyimg-wasm` has **exactly one published
version — 0.5.0, 2026-07-21**. The v0.6.0 cut (2026-07-24) never republished it.

So npm is not "about to go stale"; it has been stale for a minor already, and its only release
predates SPEC-112. A JS consumer who follows `README.md:34-36` today installs the build whose
`transform()` answers `unknown operation 'optimize'` for every bundled recipe. **Republishing is
what makes the README true for the audience the README is written for** — the crate and Homebrew
channels do not reach that reader at all.

Mechanics when authorized: `just wasm-npm-pkg` regenerates `pkg/` with the version copied from
`Cargo.toml` (DEC-067 — no hand-editing, no override), `just wasm-npm-smoke` installs the packed
tarball into a fresh project and drives it, and only then `npm publish`. Publishing 0.5.0 → 0.7.0
skips 0.6.0 on npm, which npm permits and which is honest: there was no 0.6.0 wasm release.

## Handoff — the maintainer-authorized remainder

Everything below is outward-facing and deliberately **not** done here.

1. **Tag and push** (RELEASING.md 6–7). The release commit is `chore(release): v0.7.0` on
   `chore/release-0-7-0`; merge it, then from `main`:
   `git tag -a v0.7.0 -m "crustyimg v0.7.0" && git push origin v0.7.0`. That single push fires
   cargo-dist (binaries + checksums + GitHub Release), the Homebrew formula job, and
   `publish-crates.yml`.
2. **Verify the channels** (step 8): Release page with artifacts, `cargo search crustyimg`,
   `brew install jysf/tap/crustyimg`. **The stage ships when this passes**, not before.
3. **npm republish** at 0.7.0, per the decision above.
4. Then the genuinely maintainer-only launch items, unchanged: the device pass (the ~60 MP RAW
   preview decode has never run on hardware, and whether the demo surfaces errors legibly), the
   install one-liner re-verification **at 0.7.0**, the post draft's CLI-vs-demo RAW split fix,
   and the go/no-go.
5. **The STAGE-040 brag is owed at stage close, not now** — the discipline is brag-at-STAGE-close,
   and this stage does not close until step 2 passes. Deliberately held rather than forgotten:
   two weeks and $460 of this wave went unrecorded because nobody wrote one. Figures ready for it:
   SPEC-112 cost **109,071,623 tokens / $76.16** across 2 metered cycles (build Sonnet $39.46,
   verify Opus $36.70); the release chore itself was un-metered main-loop work (AGENTS §4). The
   most quotable finding is not the fix but its guard: **no CI leg runs `just wasm-test`**, so the
   seven tests pinning SPEC-112 — and the thirty before them — run only on a maintainer's machine.

## Stage-Level Reflection

*Filled in when status moves to shipped — i.e. after the tag fires and the channels verify.*

Provisional note, recorded while it is fresh: the stage's own framing contained the error it
existed to prevent. It asserted a version-coupling fact about npm from a **generated file in the
working tree** instead of from the registry, and was wrong by a whole minor release. That is the
same failure mode as the build's cost entry (read the newest transcript, not its own) and the
three doc comments verify corrected — *the engineering was sound; the claims about it drifted*.
The cure that keeps working is the cheap one: go to the authoritative source, not the nearest
plausible one.
