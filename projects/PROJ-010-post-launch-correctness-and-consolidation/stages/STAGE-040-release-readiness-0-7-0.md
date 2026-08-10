---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-040
  status: shipped                    # proposed | active | shipped | cancelled | on_hold
  priority: critical
  target_complete: 2026-08-10

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-09
shipped_at: 2026-08-10

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

## 0.7.0 is live — channels verified 2026-08-10

The maintainer pushed the annotated tag; it points at `0c1114d`, exactly `origin/main`. All three
tag workflows succeeded, and each channel was then checked **at its own source rather than by the
job's exit status**:

- **crates.io** — `crustyimg 0.7.0` present in the sparse index (`index.crates.io/cr/us/crustyimg`),
  `yanked=false`.
- **GitHub Release** — published, not draft, not prerelease; **15 assets**: four platform archives
  (aarch64/x86_64 darwin, x86_64 linux-gnu, x86_64 windows-msvc), a `.sha256` beside each, both
  installers, `dist-manifest.json` and `source.tar.gz`.
- **Homebrew** — `jysf/homebrew-tap`'s `Formula/crustyimg.rb` now reads `version "0.7.0"` with
  checksums pointing at the v0.7.0 asset URLs.

**Then the shipped artifact was driven, not just inspected** — the discipline this whole project
runs on. Downloaded `crustyimg-aarch64-apple-darwin.tar.xz`; its SHA-256 matches both the published
`.sha256` **and** the hash the Homebrew formula pins (three-way agreement,
`8f282eef…3097`). The extracted binary reports `crustyimg 0.7.0`, and:

| driven on the released binary | result |
|---|---|
| `apply --recipe web` on a 203,671 B photo | 4,085 B of real AVIF (`ftypavif`), exit 0 |
| `web --max 256` on `dithered_graphic.png` (**the 18.5× defect**) | 34,346 → **31,988 B, 7% smaller**, lossless WebP, ssim 100.0 |

That second row is the launch gate: on 0.6.0 this input class came back **18.5× larger and visibly
degraded** (SSIMULACRA2 69.2) through the default `web` path with no flags. It is fixed in the
binary a user actually installs — which was the entire point of this stage.

## Remaining — maintainer-only, no repo work

1. **npm republish** at 0.7.0, per the decision above. Not done; irreversible and
   maintainer-gated (SPEC-076).
2. Then the genuinely maintainer-only launch items, unchanged: the device pass (the ~60 MP RAW
   preview decode has never run on hardware, and whether the demo surfaces errors legibly), the
   install one-liner re-verification **at 0.7.0**, the post draft's CLI-vs-demo RAW split fix,
   and the go/no-go.
3. **The stage brag** — written at this close, per the brag-at-STAGE-close discipline. Cost:
   SPEC-112 **109,071,623 tokens / $76.16** across 2 metered cycles (build Sonnet $39.46, verify
   Opus $36.70); the release chore itself was un-metered main-loop work (AGENTS §4).

## Stage-Level Reflection

**The stage delivered what it existed for.** PROJ-010 fixed four defects on the flagship paths and
none of them had reached a user, because the released binary predated every one. That is closed:
0.7.0 is on crates.io, Homebrew and the Releases page, and the fix was confirmed **by driving the
downloaded artifact**, not by trusting three green workflows.

**Every error this stage made was a claim about something, not the something itself.** Four, all
the same shape — reading the nearest plausible source instead of the authoritative one:

1. **The stage's own npm note** asserted version coupling from `pkg/package.json`, a *gitignored
   `wasm-pack` artifact*. The registry said `crustyimg-wasm` had only ever published 0.5.0 — wrong
   by a whole minor, and it inverted the conclusion: npm was not "about to go stale", it had been
   stale since before the fix that makes the README true.
2. **The build's cost entry** priced itself off the *parent orchestrator's* transcript, resolved as
   "newest `.jsonl` in the directory". It reported Opus/$6.75 and raised a confident finding about
   a model mismatch **that did not exist**; its own transcript was 320 Sonnet messages / $39.46.
3. **Three doc comments** in the shipped code, two found by verify — including `OPTIMIZE_STEP_OP`
   claiming "every caller strips it", which `cli::common::encode_one` falsifies.
4. **The CHANGELOG**, written in advance, had no entry for the spec that landed after it. The
   release would have shipped its headline fix silently.

None was an engineering error. The code was right every round. What is worth institutionalising is
the cheap cure that caught all four: **go to the authoritative source** — the registry, the
session's own transcript, the code the comment describes, the list of specs merged since the last
tag. And the corollary the artifact check proved: a green workflow is a claim too. The tag's three
jobs all said success; the sparse index, the formula file and a checksum-verified binary running
`web --max 256` are what made it a fact.

**What to change.** `projects/_templates/prompts/cost-snippet.md` should carry verify's technique
(identify your transcript by a probe symbol only your session emitted) rather than relying on an
agent's judgement, and the orchestrator should re-read both transcripts at ship — after completion,
since a session cannot count its own tail (verify self-reported 156 messages; the finished file had
165). `RELEASING.md` should gain two steps it lacks: diff the CHANGELOG against the specs shipped
since the previous tag, and run `just wasm-test`, which **no CI leg runs** — filed on STAGE-038.

**One design lesson, landing on the architect rather than the builder.** SPEC-112's design offered
two "reasonable" options for reaching `split_terminal_optimize`, and one of them was *impossible*:
`cli` and `wasm` are mutually exclusive `#[cfg(target_arch)]` trees, so no visibility on a
`cli`-hosted item reaches `wasm::transform`. Build and verify each caught it independently. A
design that offers a false choice is a design that has not been driven — the same lesson this wave
kept teaching the builders, arriving one level up.
