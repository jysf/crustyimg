# Session prompt: re-derive the PR #113 review's measured numbers

You are running a **measurement-only** session for crustyimg. Working dir:
`/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg`, on `main`
(should be at `e8cdbc5`; verify with `git rev-parse --short HEAD`). Tree is clean.

**You are not fixing anything.** No spec, no design, no patch. Your only job is to
determine which of a code review's measured claims reproduce, and report numbers.
A fix session comes later and will build on what you report — so a number you
report loosely becomes a number someone designs against.

## Background

A max-effort multi-agent review ran against merged commit `54ba05e` (SPEC-105,
"high-entropy images are never graphics" — a shared-classifier change). It produced
15 findings. It posted nothing to the PR; the full write-up is at:

`/private/tmp/claude-501/-Users-jyashinsky-PSeven-experiments-crustimg-redo-plus-crustyimg/21be66a3-93ac-4b5b-bb63-b3c1ee07c1af/scratchpad/pr113-review-handoff.md`

**Read that file first.** It has file:line anchors and the reproduction recipes.

The review's own numbers came from sub-agents driving the release binary. Per this
repo's standing lesson `a-number-from-an-unproven-path-is-not-a-measurement`, they
are reproduction recipes, not established measurements. That is what you are testing.

## Already verified — do not redo

An orchestrator confirmed these **structurally** by reading source. Take them as
given; spend no time re-checking:

- Classification runs after the resize pipeline: `let out_img = pipeline.run(img)?`
  at `src/cli/optimize.rs:989` → `Analysis::compute(&out_img)` at `:1013`.
- Rule 6 (`src/analysis/mod.rs:625`) is unreachable: it needs `entropy >= PHOTO_ENTROPY`
  (5.0) but is only reached when `entropy < 4.0` (rule 3.5 returns at ≥ 4.0).
  `PHOTO_ENTROPY` and `PHOTO_FLAT_MAX` appear nowhere else in `src/` or `tests/`
  (raw grep + positive control) — both are inert.
- `DOC_ENTROPY_MAX` (4.5) > `PHOTO_ENTROPY_STRONG` (4.0) → contradictory band
  `[4.0, 4.5)` resolved only by rule order.
- The Icon rule (rule 1) still precedes rule 3.5.
- Only one `Profile::Docs` arm exists (`src/analysis/decide.rs:145`), keyed on `MixedSafe`.
- `calibration_gap_holds_for_committed_fixtures` (`src/analysis/mod.rs:945`) asserts
  `graphic_max < T && T <= photo_min` over committed fixtures only — structurally a
  tautology across that window.

## What to re-derive (priority order)

**1. THE LOAD-BEARING ONE — resize promotes graphics to lossy.**
Claim: a 3840×2160 code-editor screenshot measures entropy 0.79 native → 3.62 at
`--max 2560` → **4.24 at `--max 2048`**, classifying `photograph`, and ships a
**358,227 B lossy AVIF for a 111,095 B lossless source**.

This is the one that decides whether the launch waits. Get it right.
- Use your own screenshot source (a real code-editor screenshot ≥ 3840 wide). The
  review's exact file is not committed, so an exact byte match is not expected —
  what must reproduce is the **direction and the threshold crossing**.
- Report entropy and class at native, 2560, and 2048, plus input and output bytes.
- Note whether output > input. "Never bigger" is a product contract; a violation on
  the `web` default path is the finding.

**2. Same mechanism, second source:** a 3000×2250 1-bit halftone: 0.56 native →
**4.79 at `--max 2048`** → lossy.

**3. The committed adversarial fixture defects at small sizes:**
`tests/fixtures/classify/dithered_graphic.png` measures 3.03 native (its test asserts
this) but a claimed **7.08 at `--max 256`** → `photograph` → lossy. This one uses a
**committed** file, so it should reproduce exactly. If it does not, say so loudly —
it would mean the review's harness and the repo disagree.

**4. Mixed content loses its bucket.** Composite the repo's `color_photo_fuji.png`
into flat UI chrome plus a text bar at 1600×1000. Claim: at 25% / 33% photo area →
`document` → lossless WebP (entropy 3.35 / 3.92); at **50% photo area entropy 4.93 →
`photograph` → lossy AVIF q85**.

**5. The tautology mutation.** Set `PHOTO_ENTROPY_STRONG = 5.5` in
`src/analysis/mod.rs:97` and run the classify test suite. Claim: **it stays green** —
even though 5.5 reinstates the original bug for every real photo between 4.58 and 5.5.
**Revert the mutation and confirm `git status` is clean before you finish.**

**6. Lower priority — the Icon escape hatch (finding 6).** A 128×128 EXIF-stripped
B&W photo thumbnail still classifies `Icon` → `LosslessFlat`.

**7. Lowest — dirty alpha (finding 8), which the review itself rated PLAUSIBLE, not
CONFIRMED.** A logo with non-zero RGB under fully transparent pixels: 6.25 native vs
1.04 at `--max 500`. Do this only if the others are done; report it as mechanism-only
if you cannot source a realistic asset.

## Method

- Build a **clean** release binary. Do not measure on stale incremental artifacts —
  that exact mistake cost this repo about a day on SPEC-105.
- Read class, entropy, and disposition from `--explain=json` rather than inferring
  from output format. `guess_format` cannot distinguish lossy from lossless WebP.
- For each claim report: **the claim, what you measured, and a verdict** —
  CONFIRMED / DIRECTIONALLY CONFIRMED (crossing reproduces, numbers differ) /
  NOT REPRODUCED / COULD NOT TEST (say why).
- Where you could not source the review's exact input, say so explicitly. A
  substituted input that reproduces the *mechanism* is a real result; state the
  substitution.

## Guardrails

- **`rtk` silently corrupts output.** It has returned "0 matches" for greps and finds
  against files that plainly match, twice in recent sessions. Cross-check every count
  with raw `grep`/`find` **plus a positive control that must return nonzero** — if the
  control returns 0, the tooling is lying, not the repo. Use `rtk proxy <cmd>` when
  you need real stdout.
- Read-only except the one threshold mutation, which you revert. **Do not commit, do
  not push, do not open a PR.** Leave the tree clean and confirm with `git status`.
- Work in a scratch worktree if you prefer, but the primary checkout is free — nothing
  is in flight.
- Put generated test images in your scratchpad, not the repo.

## Deliverable

A single markdown file in your scratchpad: a verdict table (claim → measured → verdict),
the commands that produced each number so a third party can re-run them, and a short
closing paragraph answering the one question the orchestrator actually needs:

> **Does `crustyimg web <screenshot>` — the default path, the flagship verb — produce a
> file that is both larger than and visually worse than its input? Yes or no, and at
> what input sizes does it start?**

Then report that file's path back. Do not summarize away the numbers; the orchestrator
needs them verbatim.
