---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-042                     # stable, zero-padded within the project
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: high
  target_complete: null

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-10
shipped_at: null

value_contribution:
  advances: >
    PROJ-010 fixed five defects. Four of them escaped the same way — they sat in a cell of a
    matrix nobody had enumerated, so every existing test was green and every one of them was
    found by a human driving the binary. This stage builds the instruments that would have
    caught them mechanically, and the signal that would have stopped the fixes sitting
    unreleased for two weeks afterwards.
  delivers:
    - "A conformance matrix derived from the code's own lists, so coverage extends when the product does"
    - "A release-lag signal, so fixed-but-unreleased cannot go unnoticed again"
    - "Guards that actually run: a wasm CI leg, and the two RELEASING.md steps this cut earned"
  explicitly_does_not:
    - "Chase the defects themselves — all five are fixed and shipped in 0.7.0"
    - "Gate STAGE-041. This protects the NEXT release, not the launch."
    - "Adopt STAGE-036's unsourced candidate list. Different stage, different standard of evidence."
---

# STAGE-042: release-safety instruments

## What This Stage Is

The stage that answers *"how do we make sure we don't release bugs like that again?"* with
instruments rather than intentions.

PROJ-010's five defects were not five unrelated mistakes. **Four of them escaped by the same
route**: a shipped asset crossed with a shipped entry point, in a cell nobody had enumerated.

| defect | the unenumerated cell |
|---|---|
| SPEC-111 — `build` cannot run a bundled recipe | bundled recipe × `build` target |
| SPEC-112 — `wasm::transform` cannot either | bundled recipe × the wasm entry point |
| SPEC-110 — seven verbs returned sideways images | orientation-bearing input × pixel-lane verb |
| SPEC-107 — a truncated JPEG succeeded silently | hostile input × decode path |

The fifth (SPEC-108's 18.5× blow-up) was a *parameter interaction* — verb × `--max` × content
class — which is a different instrument and one the repo now has, in SPEC-109's boundary
specimens. SPEC-107 likewise closed its own class with a committed hostile corpus. **The two
classes still uninstrumented are the first two rows**, and they are the two that shipped total
failures on documented paths.

## Why Now

- **The gap is measurable, and it is worse than it looks.** `bundled::names()` — the canonical
  list of what crustyimg ships — is called in **exactly one place in the entire repo**
  (`src/cli/common.rs:171`, building an error message). **No test iterates it.** All six test
  files that touch bundled recipes hardcode `"web"`, `"gallery"`, `"product"` as string literals.
  A fourth bundled recipe would land with zero coverage, silently, on every surface.
- **The same shape holds for verbs.** The verb loops that exist
  (`tests/hostile_inputs.rs:390,423`, `tests/cli.rs:223`) are hand-written subsets of 3–4 verbs.
  SPEC-110 fixed seven broken verbs; nothing stops verb fifteen from being added without
  orientation baking, which is precisely how those seven got there.
- **A guard that does not run is not a guard.** SPEC-112's verify found that **no CI leg runs
  `just wasm-test`** — `ci.yml` has no wasm32 step, and `pages.yml`'s browser smoke drives only
  the demo's markerless path. All 37 wasm tests execute on a maintainer's machine only.
- **And the multiplier was separate from all of it.** Every PROJ-010 fix sat on `main` for two
  weeks while `brew install` served the broken build. That is not a testing failure — it is a
  missing signal, and it is what turned four bugs into two weeks of shipping them.

## Success Criteria

- **A new bundled recipe cannot be added without being exercised on every entry point that
  accepts one** — because the test iterates `bundled::names()` rather than a literal list.
- **A new pixel-lane verb cannot be added without orientation coverage** — and the verb list the
  test iterates is asserted **exhaustive** against the source of truth, or the guard is just
  another subset that rots. [[a-guards-advertised-reach-is-a-claim]]
- Both are proven by **negative control**: adding a fake fourth recipe, and a fake verb, each
  turns the matrix RED without editing the test.
- `just wasm-test` runs in CI on every PR.
- `just status` surfaces release lag in a line, with a threshold that is a recorded decision
  rather than a guess.
- `RELEASING.md` gains the two steps this cut earned.

## Scope

### In scope
- The shipped-surface conformance matrix (recipes × entry points; verbs × orientation).
- The release-lag signal.
- A wasm32 CI leg.
- The two `RELEASING.md` additions.

### Explicitly out of scope
- **Re-fixing any of the five defects.** All shipped in 0.7.0.
- **STAGE-036's candidate list.** Those need provenance first; these have it.
- Any change to what the product does. Every item here is a guard, not a behaviour.
- A general coverage push. The matrix is targeted at a demonstrated escape route, not at a number.

## Spec Backlog

- [ ] **SPEC-118** (framed 2026-08-15) — **The shipped-surface conformance matrix.** Iterate
  `recipe::bundled::names()` across every entry point that accepts a recipe — `apply --recipe`,
  a `build` manifest target, and `wasm::transform` — and assert each runs and produces valid
  output for the requested format. Three recipes × three entry points is nine assertions today,
  and it extends by itself. **This single test would have caught SPEC-111 and SPEC-112 before
  either shipped.**
  The verb half needs one extra piece to be real: a `PIXEL_LANE_VERBS` list that a test asserts
  is **exhaustive** against `Commands` (`src/cli/mod.rs:229`), so adding a verb without
  classifying it fails the build rather than silently skipping coverage.
  Note the wasm leg only runs under `just wasm-test`, so this spec is coupled to the CI chore
  below — a matrix nobody runs is worth nothing. Complexity **M**.

- [ ] (not yet framed) — **The release-lag signal.** `just status` (and/or a CI job) reports when
  `main` has drifted from the last tag in a way users can feel: commits touching `src/**` since
  the tag, and how long a non-empty `[Unreleased]` has been sitting. Wants a **recorded
  threshold** — "N src commits or M days" — as a small DEC rather than a magic number, and it
  should be advisory, not a blocking gate (a red CI leg for "you haven't released lately" is the
  kind of alarm people learn to ignore). Complexity **S**.

- [ ] (chore) — **A wasm32 CI leg** running `just wasm-check` + `just wasm-test`. Currently filed
  as STAGE-038 item #8 and **moved here**, because it is the same thesis as this stage rather
  than housekeeping: a guard that does not run. The runner needs the `wasm32-unknown-unknown`
  target and `wasm-bindgen-test-runner` — [[probe-load-bearing-crates-at-design]] applies to the
  test *runner* for a new target. Complexity **S–M**.

- [x] (chore, done 2026-08-15) — **`npm publish` is the one link the build chain does not cover, so the unguarded
  path is shorter than the guarded one.** Raised by the maintainer 2026-08-10, immediately before
  publishing 0.7.0.

  The repo already makes unsafe paths *unreachable by chaining*, and says so explicitly:
  `wasm-npm-pkg: wasm-build` is documented as depending on the profiled build **"ON PURPOSE — the
  packaging step must never be reachable without going through the size profile (DEC-066), or the
  package silently ships a stock-profile .wasm, +109 KB on the wire"**, and `demo-build` likewise
  **"REFUSES a .wasm that did not come through the profiled build."**

  The chain is `wasm-build → wasm-npm-pkg → wasm-npm-smoke` — **and then it stops.** There is no
  publish recipe (`grep publish justfile` finds only comments). So the actual publish is
  `cd pkg && npm publish`, which runs **no** build, **no** size profile and **no** smoke test, and
  is *easier to type* than the safe route. `pkg/` is gitignored, so nothing ties the artifact to the
  current checkout: switching branches leaves it untouched, and the version guard in
  `wasm-npm-finalize.mjs` (which dies if `pkg.version != Cargo.toml version`) only runs at **build**
  time, never at publish time. A stale or wrong-branch artifact publishes silently, and npm
  publishes are effectively irreversible.

  **Fix: `wasm-npm-publish: wasm-npm-smoke`** — one more link, exactly the argument DEC-066 already
  makes one step earlier. This does **not** weaken SPEC-076's maintainer gate; the gate is *"a human
  decides to publish"*, not *"it must be typed as raw npm"*, and chaining strictly increases what
  runs before bytes leave the machine. npm's OTP prompt works fine inside a recipe. Worth also
  printing the resolved name@version and the git commit before the final step, so the maintainer
  confirms against something rather than nothing. Queued item #11. Complexity **S**.

- [ ] (chore) — **`just validate` silently skips untracked files.**
  `scripts/validate-frontmatter.sh:31` enumerates via `git ls-files`, so a **newly created** spec
  or stage — exactly the file most likely to have malformed front-matter — is invisible to the
  validator until it is staged. It reports success with an unchanged block count, which reads as
  a pass. Found while writing this very stage: `just validate` said "250 blocks ✓" with two new
  stage files on disk, and only said 252 after `git add`. Fix: warn when an untracked
  `*.md`/`*.yaml` sits under `projects/` or `decisions/`, or enumerate the working tree and note
  which files are untracked. **The block count should be part of the output people read**, since
  the count not moving is the only tell. [[a-harness-that-exercises-nothing-reports-green]]
  Complexity **S**.

- [ ] (chore) — **Two `RELEASING.md` steps, both earned by the 0.7.0 cut.** (a) Diff the CHANGELOG
  against the specs merged since the previous tag — 0.7.0's `[Unreleased]` section was written in
  advance and had **no entry for SPEC-112**, so the release would have shipped its headline fix
  silently had the roll not caught it. (b) Run `just wasm-test`, which no CI leg does. Complexity
  **S**.

- [ ] (not yet framed, **added 2026-08-15**) — **`next_id` mints duplicate spec IDs.** It scans
  only the WORKING TREE, so any spec living on an unmerged branch is invisible to it. Driven
  live: with SPEC-116 and SPEC-117 sitting in PR #166, `just new-spec` on a branch off `main`
  minted **SPEC-116 again**. `next_id`'s own comment warns about a different scoping failure
  (passing a single project dir restarts at 001), so this one was unanticipated rather than
  accepted. Same family as the counter bugs fixed 2026-08-15: it fails by producing a plausible
  wrong answer, silently. Fix is to consult git refs, or at minimum warn when a higher ID exists
  on another ref. Complexity **S**.

**Count:** 1 chore done (npm publish guard) / 1 framed (SPEC-118) / 1 spec + 4 chores pending

## Design Notes

- **Derive the matrix from the code, never from a literal list.** This is the whole idea. A
  hand-written list of three recipes is exactly what the six existing test files already have,
  and it is why the gap existed. The test must fail when the *product* grows, not when someone
  remembers to update the test.
- **An exhaustiveness assertion is what separates this from another subset.** For recipes,
  `bundled::names()` gives it for free. For verbs it must be built: match on `Commands`
  non-exhaustively-forbidden, or assert list length against the enum. Without it the verb matrix
  is `tests/cli.rs:223` again — three verbs, hand-picked, green forever.
- **The repo already does this for documentation, and it works.** `tests/docs_ops.rs`,
  `tests/adoption_glue.rs` and `tests/demo_copy.rs` police doc claims mechanically, and
  `docs_ops.rs`'s own header states the principle better than this note can: prose that reads
  like a caveat while standing in for a check nobody runs. **This stage points that same
  instinct at surfaces instead of sentences.**
- **What this stage would NOT have caught, stated up front so it is not oversold:** SPEC-108's
  classifier blow-up (a parameter interaction — covered by SPEC-109's boundary specimens) and
  SPEC-107's truncated JPEG (hostile input — covered by its committed corpus). Two of five. The
  honest claim is that the repo would then have an instrument for four of the five escape routes
  it has actually experienced, not that it becomes defect-proof.
  [[a-guards-advertised-reach-is-a-claim]]
- **Sequenced after STAGE-041 by maintainer decision.** The launch is the time-sensitive item;
  this protects the next release, and nothing in it gates the current one.

## Dependencies

### Depends on
- STAGE-040 (shipped) — the defects and their escape routes are the evidence this stage is built
  from, and they are only fully documented after that close-out.

### Enables
- Every subsequent release. And, specifically, the confidence to cut one without a human
  re-driving the flagship paths by hand — which is what 0.7.0 required.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
