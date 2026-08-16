---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-117
  type: task                       # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: medium
  complexity: S                    # S | M | L  (L means split it)

project:
  id: PROJ-010
  stage: STAGE-045
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5     # build on Sonnet: no behaviour change, two
                                   # integration tests against fixtures that
                                   # already exist. Verify stays Opus.
  created_at: 2026-08-15

references:
  decisions:
    - DEC-089
  constraints:
    - clippy-fmt-clean
    - one-spec-per-pr
  related_specs:
    - SPEC-115

value_link: >
  STAGE-045's last item. SPEC-115 fixed the adopted-format defect at one seam that four
  verbs share. Its verify DROVE `build` and `apply --recipe web` green on the real binary
  but pinned neither, so a refactor can re-break them in silence while the SVG/RAW/HEIC
  tests stay green.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Filed from SPEC-115's verify
        readout, which drove both paths green and flagged the missing pin as its own
        punch-list item.
    - cycle: build
      agent: claude-sonnet-5
      interface: claude-code
      tokens_total: 28467342
      duration_minutes: 39
      recorded_at: 2026-08-16
      tokens_breakdown: {input: 364, output: 126826, cache_creation: 393686, cache_read: 27946466}
      estimated_usd: 11.76
      note: >
        MEASURED — own transcript sum (main-loop session executing the build prompt
        directly, not dispatched as a subagent), 182 assistant messages with usage.
        Sonnet $3/$15 per MTok anchors (model actually reported by every message);
        cache_creation x1.25, cache_read x0.10.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-117: pin `build` and `apply --recipe web` against the adopted-format defect

## Context

SPEC-115 stopped `optimize` shipping bytes it cannot name: an SVG, HEIC or RAW container
passed through verbatim and reported as the PNG or JPEG it never produced. The fix lives in
`optimize_decide_one`, and **four verbs share that seam** — `optimize`, `web`,
`apply --recipe web`, and `build` (via `encode_one_optimize_decided`).

SPEC-115 shipped tests for the first two. Its **verify cycle drove the other two by hand** on
the real binary and found them correct:

```
apply --recipe web  <svg fixture>  → a real WebP; summary reads svg → webp · 336 → 444 B
                                      (32% larger), with the new fourth reason in the note
build               <svg source>   → rect_text_40x30.webp, RIFF … WebP image
```

**Both work. Neither is pinned.** Verify filed that as a punch-list item rather than fixing it
in-branch, and this spec is that item.

### Why a passing path still needs a test

`encode_one_optimize_decided` delegates *unconditionally* to the fixed function, so today the
behaviour is correct by construction. That is exactly the situation in which a regression is
invisible: the SVG, RAW and HEIC tests all drive `optimize` directly, so a change to the
delegation — a wrapper that stops calling through, an early return, a second code path added
for `build` — leaves every existing test green while two shipped verbs go back to writing
mislabeled bytes.

This is a **regression pin on working behaviour**, not a defect fix. It should be boring, and
the spec is short because the work is.

## Goal

Two integration tests, one per verb, asserting the written bytes are a real raster of the
declared format rather than the source container.

## Inputs

- **Files to read:**
  - `src/cli/optimize.rs` — `optimize_decide_one`, and `encode_one_optimize_decided`, the
    delegating wrapper `build` uses.
  - `src/cli/build.rs:430-445` — the `OutputFormatPlan::Decide` arm.
  - `tests/input_svg.rs` — SPEC-115's own tests. **Match their assertion style**: sniff the
    written bytes, do not trust the extension or the summary line.
  - `DEC-089` — the `Image` origin model SPEC-115 introduced.
- **Fixtures that already exist:** `tests/fixtures/svg/rect_text_40x30.svg`. **Do not build a
  new one** — SPEC-115's fixture hunt cost most of its overrun, and SVG is the family that
  reproduces with a committed fixture.

## Outputs

- **Files modified:** `tests/build.rs` and `tests/cli.rs` (or `tests/input_svg.rs` — put each
  test where its verb's other tests live; say which you chose and why).
- **New fixtures:** none.
- **Source changes:** **none expected.** If you find yourself editing `src/`, stop — that means
  the behaviour is not actually correct today, which is a finding that changes this spec from a
  pin into a fix. Report it.
- **No new DEC expected.**

## Acceptance Criteria

- [x] **AC-1.** `apply --recipe web <svg fixture>` writes a **real WebP** — asserted with
      `image::guess_format` or a decode, on the written bytes, **not** the extension.
- [x] **AC-2.** `build` with an SVG source on a Decide-plan target writes a **real WebP**, same
      assertion style.
- [x] **AC-3.** **Neither output is byte-identical to the source SVG.** The defect's signature
      was passing the container through verbatim; assert against it directly rather than
      inferring from the format sniff alone.
- [x] **AC-4.** **The reported format matches the bytes.** Assert the summary/`--json` names the
      real container, not the adopted `png` label — SPEC-115 fixed both the bytes *and* the
      report, and a pin that only checks bytes lets half the fix regress.
- [x] **AC-5.** **A negative control**: force the delegation to bypass the fix (revert
      `encode_one_optimize_decided` to a pre-SPEC-115 path, or stub the origin to `Native`),
      confirm **both** new tests go RED, restore. Prove the revert reached the built artifact.
      **Per-verb, not one coarse revert** — if reverting once turns only one of them red, the
      other is not actually pinned, and that is the finding.
      [[a-harness-that-exercises-nothing-reports-green]]
- [x] **AC-6.** Clean **full matrix**, fresh per-leg `CARGO_TARGET_DIR`, sequential, through
      `rtk proxy`: default, `--no-default-features`, `--features webp-lossy`. Clippy and
      `fmt --check` each. Establish your own `main` baseline; the delta should be exactly the
      new tests. **Then read the CI legs individually.**

## Failing Tests

Written during **design**, BEFORE build.

**Read this honestly: neither test fails on `HEAD`, and that is correct.** This spec pins
behaviour that already works, so `test-before-implementation` does not apply in its usual form
— there is no red-to-green transition to demonstrate. **AC-5's negative control is what proves
the tests are real**, and it is therefore the load-bearing criterion of this spec rather than a
formality. A reviewer should treat AC-5 the way they would normally treat a failing test.

- **`tests/cli.rs`** (or `tests/input_svg.rs`)
  - `"apply_recipe_web_on_svg_writes_a_real_raster"` — AC-1/AC-3/AC-4. **Passes today.**
- **`tests/build.rs`**
  - `"build_on_svg_source_writes_a_real_raster"` — AC-2/AC-3/AC-4. **Passes today.**
- **Negative control** (AC-5, run and recorded, not committed)
  - Bypass the fix per verb → each test RED in turn.

## Implementation Context

### Decisions that apply
- **DEC-089** — `Image` records its container origin separately from the decoded pixel format.
  Unchanged here; this spec only pins that two more callers benefit from it.

### Constraints that apply
- `clippy-fmt-clean` (**blocking**).
- `one-spec-per-pr` (**blocking**) — SPEC-116 also touches `build`. Separate branch, separate PR.
- `test-before-implementation` — **does not apply in its usual form**, see Failing Tests. Do not
  manufacture a fake red by breaking the source first; use AC-5's control instead.

### Prior related work
- **SPEC-115** — the fix, the fixture hunt, and the verify that drove these two paths green.
  Its Build Completion records that `tight_preview.nef` and a naive noise preview both failed
  to reproduce; **do not repeat that search**, SVG reproduces with the committed fixture.

### Out of scope
- RAW and HEIC coverage for these two verbs. SVG is sufficient to pin the delegation, and the
  format-specific behaviour is already covered at the `optimize` seam. Adding two more families
  × two more verbs is four more tests for no additional signal.
- Any behaviour change.

## Notes for the Implementer

- **If this spec takes more than an hour, something is wrong** — re-read the scope. It is two
  tests against an existing fixture.
- **The trap is a vacuous pin.** A test that passes because the verb was never exercised looks
  identical to one that passes because the fix works. AC-5 is the only thing separating them,
  and SPEC-113 shipped exactly that mistake two specs ago.
- **A piped command reports the pipe's exit code.** Redirect and read `$?`.
- macOS has no `timeout(1)`. `git commit -s` (DCO). **Own git worktree.** **Do not merge the
  PR. Do not bump the version.**
- Follow `projects/_templates/prompts/closing-steps-snippet.md` when you finish, including
  `just advance-cycle SPEC-117 verify` — and confirm the `cycle:` line actually moved.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `test/spec-117-pin-build-and-apply-adopted-format`
- **PR (if applicable):** #174 (jysf/crustyimg), opened, not merged.
- **All acceptance criteria met?** yes.
  - AC-1/AC-3/AC-4: `apply_recipe_web_on_svg_writes_a_real_raster`, added to `tests/input_svg.rs`
    (chosen over `tests/cli.rs` — that file is 207 KB and this test's siblings, including
    SPEC-115's own `web_svg_auto_decide_writes_a_real_raster`, already live in `input_svg.rs`).
  - AC-2/AC-3/AC-4: `build_on_svg_source_writes_a_real_raster`, added to `tests/build.rs`.
  - AC-5: per-verb negative control run and recorded below (not committed — no source diff
    survives; `git diff src/` is empty).
  - AC-6: full matrix (default, `--no-default-features`, `--features webp-lossy`), fresh
    per-leg `CARGO_TARGET_DIR`, sequential, through `rtk proxy` — build, clippy
    (`--all-targets -- -D warnings`), `fmt --check` (once, feature-independent), and the full
    `cargo test` suite on each leg. All three legs: 36/36 test-binary suites `ok`, 0 `FAILED`,
    both new tests present and passing on every leg. `main` baseline established directly
    (`git show HEAD:tests/*.rs | grep -c '#\[test\]'`): 25→26 in `tests/build.rs`,
    7→8 in `tests/input_svg.rs` — the delta is exactly the two new tests, nothing else moved.
- **New decisions emitted:** none.
- **Deviations from spec:** none. `src/` is untouched (`git diff --stat` shows only
  `tests/build.rs` and `tests/input_svg.rs`) — the behaviour was already correct by
  construction, as the spec predicted.
- **Follow-up work identified:** none beyond what the spec already scoped out (RAW/HEIC
  coverage for these two verbs).

**AC-5 detail — per-verb negative control (run, recorded, not committed):**

Two independent, targeted reverts were used rather than one coarse revert, because the two
verbs reach `optimize_decide_one` through two *different* call sites (`build`'s
`encode_one_optimize_decided` wrapper vs. `apply`'s direct `run_optimize_autodecide` call in
`run_apply`) — a bug in either wrapper individually needs its own control.

1. **Build-only revert:** temporarily replaced `encode_one_optimize_decided`'s body with a raw
   passthrough (`read_raw_bytes` + `metadata_output_ext`, bypassing `optimize_decide_one`
   entirely) — simulating "a wrapper that stops calling through" (the spec's own example).
   Result: `build_on_svg_source_writes_a_real_raster` → **RED** (`guess_format` on the written
   `dist/rect_text_40x30.svg` returned `None`, not `Some(WebP)`).
   `apply_recipe_web_on_svg_writes_a_real_raster` → **GREEN**, unaffected (confirms the apply
   test does not depend on this wrapper — not a vacuous shared pass).
2. **Restored** (`git diff src/cli/optimize.rs` empty — confirmed clean before the next step).
3. **Apply-only revert:** temporarily forced `run_apply`'s `pinned` local to `true`
   unconditionally, short-circuiting the terminal-`optimize` auto-decide branch — simulating
   "an early return" (the spec's other example). Result:
   `apply_recipe_web_on_svg_writes_a_real_raster` → **RED** (stderr never printed
   `svg → webp`; the plain pinned path re-encoded the pipeline output as PNG instead).
   `build_on_svg_source_writes_a_real_raster` → **GREEN**, unaffected (confirms independence
   the other direction).
4. **Restored** (`git diff src/cli/optimize.rs` empty). Rebuilt and reran both tests on the
   restored `HEAD`: both **GREEN** again — each revert step forced a real recompile (`cargo
   test`'s own change detection; each run recompiled `crustyimg` before executing), so the
   revert demonstrably reached the built artifact each time, not a stale binary.

Each test is therefore sensitive to its OWN verb's delegation site independently — neither is
a vacuous pass riding on the other's fix.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** Nothing major. AC-1/AC-2 ask for "a
   real WebP" specifically — stronger than SPEC-115's own tests, which only assert "sniffs as
   *some* raster format." Confirming that WebP wins deterministically (not just for this one
   build) required tracing `format_shortlist` + `fast_fallback_lossy_entry` by hand and then
   empirically running `apply --recipe web` / `build` against the fixture on all three feature
   legs before committing to the specific assertion — worth doing regardless, but the spec
   could have flagged "the reference behaviour holds on every leg, not just the one it quotes"
   explicitly rather than leaving that verification to the implementer.
2. **Was there a constraint or decision that should have been listed but wasn't?** No — DEC-089
   and the two file-level Inputs pointers were sufficient. The exact byte counts in the spec's
   reference block (336 → 444 B) turned out to reproduce byte-for-byte on every feature leg,
   which was a useful (if implicit) determinism check.
3. **If you did this task again, what would you do differently?** Nothing procedurally — running
   the real binary against the fixture on all three feature legs *before* writing the final
   assertions (rather than writing them speculatively against the "reference" block first) is
   what caught that the WebP outcome is feature-leg-invariant, and I'd do that verification
   step first again.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
