---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-039
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
    The same thesis as STAGE-034, one layer out. STAGE-034 fixes a verb that returns a
    worse file; this stage fixes verbs that return the wrong file or refuse to run at
    all. Both were found the same way — by someone driving the shipped binary rather
    than by a gate — and both are live on paths the README documents.
  delivers:
    - "`convert` no longer hands back a sideways image: EXIF Orientation is either baked or preserved, decided deliberately, and every other re-encoding verb is swept mechanically for the same defect"
    - "`build` can actually run the bundled recipes it ships with, instead of failing UnknownOperation on every one of them"
    - "docs/data-model.md stops advertising three operations that do not exist"
  explicitly_does_not:
    - "Add any operation, codec, or engine capability — `unsharp` and `clean-gps` stay unimplemented and `watermark` stays deliberately unregistered; this stage corrects the documentation, it does not satisfy it"
    - "Touch the classifier or the pixel-lane decision engine — that is STAGE-034"
    - "Redesign the recipe schema. D-2 is fixed by wiring up the strip helper that already exists, not by adding an [output] table"
---

# STAGE-039: shipped-verb correctness

## What This Stage Is

Three defects on verbs that have already shipped, surfaced by the read-only exploration in
`docs/research/photo-preset-import-and-photographic-ops.md` and **re-verified against this repo's
source** during the PROJ-010 framing session. Full evidence in `docs/backlog.md`, section
"⚠ Live defects on shipped verbs".

They are grouped as one stage because they share a shape, not a module: **each is a shipped surface
that behaves differently from its own documentation**, and none was catchable by a gate. Keeping them
out of STAGE-034 keeps that stage a single subject — the classifier pixel lane — and keeps its
"pure pixel-lane engine work" fence standing.

## Why Now

- **D-1 and D-2 are launch-gating.** The Show HN audience runs the documented commands. `convert` on
  a phone photo returns it sideways, and `build` on any bundled recipe does not run at all. Both are
  worse to discover in a comment thread than to fix now.
- **D-2 in particular is a total failure, not a degradation.** There is no partial success to soften
  it: a manifest target bound to `web`, `product` or `gallery` exits with `UnknownOperation`.
- **The fix for D-2 is wiring, not design.** The strip helper already exists and is already used by
  the `optimize` path; `build` simply never calls it.

## Success Criteria

- **D-1:** a JPEG with `Orientation=6` run through `convert` produces an image whose **dimensions**
  prove the rotation was handled — 800×1200 from a 1200×800 source, matching what `web` / `optimize` /
  `auto-orient` already return. The fixture asserts dimensions, **not** tag absence; a test that only
  checks the tag is gone passes today, on the broken behaviour.
- **D-1 sweep:** `thumbnail`, `resize`, `responsive`, and `edit` without `--auto-orient` are each
  checked **mechanically**, with the grep cited and its scope stated as a claim
  ([[mechanical-sweeps-need-a-mechanical-check]]). "We looked and it seemed fine" does not close this.
- **D-2:** `crustyimg build` against a manifest target bound to each of `web`, `product` and `gallery`
  completes and writes output. A negative control proves the test can fail — e.g. a recipe whose
  terminal step is a genuinely unknown op must still error.
- **D-3:** every op named in `docs/data-model.md` resolves against `OperationRegistry::with_builtins`,
  asserted by a test that reads the doc rather than by a human re-reading it. If that is too much for
  one spec, the doc is corrected and the test is filed — but say which.
- All gates green on the clean full matrix (default / `--no-default-features` / `--features
  webp-lossy`, clippy `-D warnings` each, plus `fmt --check`).

## Scope

### In scope
- The `convert` orientation decision and fix, plus the mechanical sweep of the other re-encoding verbs
  and fixes for whatever it finds.
- Wiring the terminal-`optimize` strip into the `build` path.
- Correcting `docs/data-model.md`'s worked example.

### Explicitly out of scope
- Implementing `unsharp`, `clean-gps`, or registering `watermark`. `watermark` is unregistered
  **deliberately** (`src/operation/mod.rs:784`, `src/cli/ops.rs:945`); D-3 corrects the doc to match
  the code, not the reverse.
- Recipe schema v2 / a top-level `[output]` table. It would subsume D-2, but it is a much larger
  change and is not launch work.
- Classifier, codec, wasm, or demo work.
- The four **unverified** reports filed alongside these in `docs/backlog.md` (RAW EXIF loss, the
  metadata lane's reach, AVIF determinism, gain-map detection). Each needs its own confirmation
  first — one of them explicitly reports its key case as cannot-determine.

## Spec Backlog

- [ ] SPEC-110 (frame) — **`convert` orientation: decide, fix, sweep.** `run_convert`
  (`src/cli/optimize.rs:507`) builds `Pipeline::new()` at `:538` — *"Pure re-encode: an empty pipeline
  returns the pixels unchanged"* — and the pixel-lane re-encode drops the metadata bundle, so the
  Orientation tag is discarded while the rotation it described is never applied. `optimize`/`web` pin
  `auto-orient` first (`:790`, DEC-017). **There is a real design call here:** `convert`'s contract is
  a lossless-intent format change, so baking pixels is not automatically right — preserving the tag
  may be the better answer. Decide it, then sweep. Complexity **S–M**.
- [ ] SPEC-111 (frame) — **`build` runs bundled recipes.** `prepare_target` (`src/cli/build.rs:80`)
  calls `recipe.build_pipeline(registry)` at `:85` without stripping the terminal `optimize` marker —
  `build.rs` contains **0** references to `optimize`, `OPTIMIZE_STEP` or `strip_terminal`. Every
  bundled recipe ends with that marker (`src/recipe/bundled.rs:20`, asserted at `:91`), and the
  registry holds four ops (`src/operation/registry.rs:80-83`), none named `optimize`. The helper to
  reuse is `OPTIMIZE_STEP_OP` and its consumer at `src/cli/optimize.rs:32-41`. Self-documented in
  **DEC-070 point 4**. Complexity **S–M**.
- [ ] (chore, may not need a spec) — **`docs/data-model.md` worked example.** `:142-182` advertises
  `op = "unsharp"` (`:161`), `op = "watermark"` (`:166`), `op = "clean-gps"` (`:174`) and the CLI flags
  `--unsharp` / `--watermark` (`:181-182`). Rewrite against the four real ops. Complexity **S**.

**Count:** 0 shipped / 0 active / 2 specs + 1 chore pending

## Design Notes

- **D-1's regression fixture is the interesting part.** The obvious test — "assert the Orientation tag
  is gone" — **passes on the broken behaviour**, because stripping the tag is exactly what `convert`
  does today. Only output dimensions distinguish the two outcomes. The exploration got this right by
  running a positive control: `web` / `optimize` / `auto-orient` all return 800×1200 from the same
  source, which is what proves the harness can show the other result
  ([[a-plausible-test-result-is-not-a-checked-one]]).
- **D-1's blast radius is ordinary.** Orientation 6 is the standard phone-photo case, not an edge
  input. This is why it ranks alongside the classifier work rather than below it.
- **D-2 may be masked by its own test suite.** Before fixing, check whether any existing `build` test
  uses a bundled recipe — if none does, that absence is itself the finding, and the fix should close
  it ([[a-criterion-nobody-claims-is-a-criterion-nobody-checks]]).
- **Provenance.** Every claim above was re-verified in this repo during framing, not relayed from the
  exploration document. The exploration's own unverified items were kept out of this stage on purpose;
  they are in `docs/backlog.md` labelled as such.

## Dependencies

### Depends on
- Nothing. All three are independent of STAGE-034's classifier work and share no files with it — the
  two can run concurrently.

### Enables
- A launch where the documented commands do what they are documented to do.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
