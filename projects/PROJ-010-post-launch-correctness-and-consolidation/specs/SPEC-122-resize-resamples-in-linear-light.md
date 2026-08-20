---
task:
  id: SPEC-122
  type: bug
  cycle: verify
  blocked: false
  priority: high
  complexity: M

project:
  id: PROJ-010
  stage: STAGE-046
repo:
  id: crustyimg

agents:
  architect: claude-opus-5
  implementer: claude-sonnet-5
  created_at: 2026-08-16

references:
  decisions:
    - DEC-092
    - DEC-019
    - DEC-058
  constraints:
    - clippy-fmt-clean
    - test-before-implementation
    - one-spec-per-pr
  related_specs:
    - SPEC-120
    - SPEC-121

value_link: >
  STAGE-046's last defect, and the only one whose premise was measured before it
  was specced. `Resize::apply` resamples non-linear sRGB as if it were linear;
  against an independent reference the shipped downscale scores 70.45 and 84.45
  where a linear-light prototype scores ~100.

cost:
  sessions:
    - cycle: design
      interface: claude-code
      tokens_total: null
      duration_minutes: null
      estimated_usd: null
      note: >
        Un-metered main-loop design cycle (AGENTS §4). Scoped down from the
        backlog entry on two SPEC-120 findings: the premultiplied-alpha half is
        false, and the migration the entry worried about already exists.
    - cycle: build
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 182177233
      duration_minutes: 66
      recorded_at: 2026-08-18
      tokens_breakdown:
        input: 1216
        output: 354963
        cache_creation: 662926
        cache_read: 181158128
      estimated_usd: 103.60
      note: >
        MEASURED — transcript sum over 608 assistant messages, priced per
        component at OPUS anchors ($5/$25 per MTok; cache_creation x1.25 input,
        cache_read x0.10 input). The spec's front matter and the build prompt
        both name claude-sonnet-5, but `.message.model` reports claude-opus-5 on
        all 608 messages, so the model that actually ran sets the anchors
        (AGENTS §4). WARNING — roughly $60 of this is CI polling, not work: the
        same transcript measured $32.01 at 231 messages, with the substantive
        build already complete and only CI left to watch. Cache reads are 99.4%
        of volume, so every poll re-read the whole accumulated context.
        SPEC-123 measured this hazard at $5.80; here it was an order of
        magnitude worse and is the single largest line item in the spec.
    - cycle: build
      agent: claude-opus-5
      interface: claude-code
      tokens_total: 28526492
      duration_minutes: 27
      recorded_at: 2026-08-20
      tokens_breakdown:
        input: 390
        output: 161232
        cache_creation: 343905
        cache_read: 28020965
      estimated_usd: 20.19
      note: >
        MEASURED — punch-list return cycle (verify returned ⚠ PUNCH LIST on
        PR #182; five items). Transcript sum over 195 assistant messages
        (~/.claude/projects/.../d4f32638-b94d-44c7-b912-c13e0d3208e2.jsonl),
        priced at OPUS anchors ($5/$25 per MTok; cache_creation ×1.25 input,
        cache_read ×0.10 input) — `.message.model` is `claude-opus-5` on all
        195. Ran in the main loop, not as a dispatched subagent, so there is no
        `subagent_tokens` to cross-check against. The build's session above is
        untouched. Cache reads are 98.2% of volume.
        ⚠ Read at the cost-append step, BEFORE the push and before CI settles,
        so it under-states this cycle — SPEC-121's punch list measured that same
        gap at 9.5%. Corrected below if the post-CI reading moves it.
        The CI polling that cost the build ~$60 of its $103.60 did not recur:
        the watch was backgrounded and the reading taken once. This whole cycle
        costs a third of that one line item. 195 messages against the prompt's
        ~150 budget, with a four-leg matrix run twice (once for the counts, once
        on the committed code) and four release builds for the memory arms.
        ⚠ No `verify` entry exists in this list: the verify cycle that produced
        the punch list did not append one — the same gap SPEC-121 flagged.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-122: `resize` resamples in linear light

## Context

`Resize::apply` converts to RGBA8 and hands `PixelType::U8x4` to
`fast_image_resize` with Lanczos3 (`src/operation/mod.rs:395-527`). No gamma
handling anywhere. So crustyimg resamples **non-linear sRGB values as if they
were linear** — high-contrast edges darken on downscale, worst on thin bright
features against dark backgrounds.

**This spec exists because SPEC-120 measured it first, and the gate fired.**
DEC-092:

| case | source → target | mean signed luma err | today's SS2 | linear prototype | Δ |
|---|---|---|---|---|---|
| synthetic worst case (positive control) | 2048²→256² | −0.104350 (−88.07%) | −63.85 | 100.00 | **+163.85** |
| `graphic_large.png` | 512²→128² | −0.001386 (−0.44%) | **70.45** | 100.00 | +29.55 |
| `photo_forest_cc0.jpg` | 800×532→200×133 | −0.004920 (−2.63%) | **84.45** | 99.41 | +14.96 |

**Read the load-bearing numbers correctly.** The prototype's ~100 and the Δ
column are partly self-fulfilling — a linear-light Lanczos prototype scored
against a linear-light Lanczos reference should agree. **The claim that survives
is the first column: today's shipped path scores 70.45 and 84.45 against an
independent reference** (ImageMagick 7.1.2-29 Q16-HDRI, `-colorspace RGB`).

The positive control is what makes those readable rather than an uninterpretable
null: an −88% physical error registered as a 163.85-point swing, so **the
instrument can see this defect.**

### ⚡ Scoped down by SPEC-120 — this is ONE premise, not two

The backlog entry paired linear light with premultiplied alpha. **The alpha half
is FALSE.** `fast_image_resize` 6.0.0's `ResizeOptions::default()` sets
`mul_div_alpha: true` (`src/resizer.rs:52-60`) and `ResizeOptions::new()` *is*
`Default::default()` (`:63-64`) — and `Resize::apply` overrides only the
algorithm. **It has always premultiplied.** Do not add alpha handling.

## The design calls — settled here

### Call 1 — convert inside `Resize::apply`; the pipeline stays as it is

Linearize → resample → re-encode to the output transfer function, **inside the
op**. No pipeline-wide colour management, no new `Image` field, no 16-bit-
throughout project. `fast_image_resize` 6.0.0 ships `F32x4` and `U16x4`
(`src/pixels.rs:26,21`), so the backend is already in place.

### Call 2 — the reference implementation already exists in this repo

**SPEC-120 committed a working linear-light prototype and its harness:**
`examples/spec120_linear_probe.rs` and `scripts/spec120_linear_light.py`.

**Start from the prototype.** It is the code that produced the numbers above, so
starting anywhere else means re-earning them. It is an `examples/` throwaway —
productionizing it is this spec's work.

### Call 3 — the measurement is the acceptance test, and it reuses SPEC-120's harness

Do not invent a new oracle. Re-run `scripts/spec120_linear_light.py` against the
**branch** binary and show the shipped path's score moving toward the reference on
the same three cases. **The independent reference stays independent** — regenerate
it with the same external tool, do not substitute crustyimg's own output.

### Call 4 — sRGB is assumed; ICC-aware conversion is NOT this spec

crustyimg keeps ICC profiles but does not interpret them. Assume the sRGB transfer
function, and **say so in the DEC**. An image with a non-sRGB profile is resampled
under an assumption that is wrong for it — better than today's, still an
assumption. Full colour management is its own project.

### Call 5 — the migration already exists (shared with SPEC-121)

`cache_key_for` includes `crate::version()` (`src/cli/build.rs:294`), and the
lockfile explicitly never promised output-hash stability across versions
(`src/build/lock.rs:32-36`). A release changes every key; old and new renders
cannot collide. **Drive it, do not design it** — and share **one DEC** with
SPEC-121 so the wave is a single decision.

## Inputs

- `src/operation/mod.rs:390-530` — `Resize::apply`.
- `examples/spec120_linear_probe.rs`, `scripts/spec120_linear_light.py` — the
  prototype and the harness.
- **DEC-092** — the verdict, the numbers, and the alpha refutation.
- `docs/backlog.md`'s linear-light entry, **including SPEC-120's appended result**.

## Outputs

- **Files modified:** `src/operation/mod.rs`, `tests/`.
- **New DEC:** **shared with SPEC-121** — the byte change, the sRGB assumption
  (Call 4), and the migration posture. `affected_scope`: `src/operation/**`.
- The `examples/` prototype may be deleted once production carries it — say which
  you chose.

## Acceptance Criteria

- [ ] **AC-1.** **The shipped path's SSIMULACRA2 against the independent reference
      improves materially on all three SPEC-120 cases**, re-run through that
      harness. State the before/after per case.
- [ ] **AC-2.** **The physical quantity improves too** — mean signed luminance
      error against the reference moves toward zero. Both metrics reported, and
      **if they disagree, that disagreement is the finding**.
- [ ] **AC-3.** **The positive control still fires** — the synthetic worst case
      shows the large swing. A fix that fixes only realistic cases is suspicious.
- [ ] **AC-4.** **The reference is regenerated independently** — same external
      tool, not crustyimg's own output. Committing a crustyimg-derived reference
      would make this untestable forever
      [[fixtures-from-the-code-under-test-cannot-fail]].
- [ ] **AC-5.** **Alpha behaviour is unchanged** — translucent-edge error stays at
      SPEC-120's measured 27/255 band. Call 2's refutation says nothing should move
      here; prove it didn't.
- [ ] **AC-6.** **Upscale and no-op resize are unaffected**, byte-identical to
      `main` where no resampling occurs.
- [ ] **AC-7.** **A negative control** — revert the linearization; the three cases
      return to 70.45 / 84.45 / −63.85. **The behavioural flip is the evidence.**
- [ ] **AC-8.** **The migration driven** (Call 5), shared with SPEC-121: key
      changes, `--frozen` fails, regeneration succeeds, no stale cache hit.
- [ ] **AC-9.** **Performance is measured and reported.** f32 resampling is more
      work than u8. Not a gate — but `resize` is the most-used op and nobody asked
      at design, so report it with controls.
- [ ] **AC-10.** Clean full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequential,
      through `rtk proxy`: default, `--no-default-features`, `--features
      webp-lossy`. Clippy and `fmt --check` each. Then read the CI legs.

## Failing Tests

- **`tests/linear_light_resize.rs`** (new)
  - `"downscale_scores_better_against_an_independent_reference"` — AC-1. **RED.**
  - `"mean_luminance_error_moves_toward_zero"` — AC-2. **RED.**
  - `"translucent_edge_error_is_unchanged"` — AC-5. Passes today; the control.
  - `"upscale_and_noop_resize_are_byte_identical_to_main"` — AC-6.
- **Harness re-run** (AC-1/AC-3, recorded not committed as a test): the three
  SPEC-120 cases before and after.

## Implementation Context

### Decisions that apply
- **DEC-092** — the premise verdict and the alpha refutation. Binding.
- **DEC-019** — SSIMULACRA2 as the oracle; SPEC-120 proved it can see this defect.
- **DEC-058** — the cache-key composition Call 5 rests on.

### Out of scope
- Premultiplied alpha (**refuted**), ICC-aware conversion (Call 4), a 16-bit
  pipeline, and the colour-type fix (**SPEC-121** — same function, sequence together).

## Notes for the Implementer

- **Do not re-litigate the premise.** It was measured; DEC-092 is the record.
- **Start from `examples/spec120_linear_probe.rs`.**
- **Keep the reference independent** — AC-4 is what stops this becoming a test
  that cannot fail.
- **Budget in exchanges (~250), not minutes.**
- macOS has no `timeout(1)`. `git commit -s`. **Own git worktree.** **Do not merge
  the PR. Do not bump the version.**
- Follow `closing-steps-snippet.md`, including `just advance-cycle SPEC-122 verify`.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:** `fix/spec-122-resize-resamples-in-linear-light`
- **PR (if applicable):** #182
- **All acceptance criteria met?** **9 of 10 in full; AC-6 in half, deliberately
  and with measurement.** AC-5 was met by a *better* result than it predicted,
  which is also worth reading as a deviation rather than a pass.
- **New decisions emitted:** **none — DEC-095 amended**, as instructed. The
  amendment adds the linear-light change, Call 4's sRGB assumption, SPEC-122's
  consequences and alternatives, and a Validation section covering the three
  negative controls. **`affected_scope` is widened, not confirmed** — the build
  reported it as confirmed on a false premise (*"the diff touches
  `src/operation/mod.rs` and `tests/` only"*). It does not: it also touches
  **`scripts/lib/wasm-artifact.mjs`** and **`scripts/demo-assemble.mjs`**, where
  a safety constant was moved. Both are now in `affected_scope`, and
  `justfile` + `docs/backlog.md` + `scripts/spec120_linear_light.py` are also in
  the diff (the last of those is already covered by DEC-092's scope). Corrected
  in the punch-list cycle; see Deviation 4.
- **Deviations from spec:** four — three from the build, all measured, plus one
  raised by verify; see below.
- **Follow-up work identified:** four, all filed rather than fixed — two from
  the build, plus `MAX_AREA`'s output-vs-peak gap and the memory cost, both of
  which ride the same `F32x4` decision as the 3.83× slowdown.

### The result (AC-1, AC-2, AC-3, AC-4)

`scripts/spec120_linear_light.py` re-run against the **branch** binary, with the
reference **regenerated** by the same external tool (ImageMagick 7.1.2-29
Q16-HDRI aarch64 — the same build DEC-092 used), never substituted:

| case | SSIMULACRA2 before → after | mean signed luma err before → after |
|---|---|---|
| synthetic worst case (**AC-3** positive control) | **−63.85 → 100.00** | −0.104350 (−88.07%) → 0.000000 |
| `graphic_large.png` | **70.45 → 100.00** | −0.001386 (−0.44%) → 0.000000 |
| `photo_forest_cc0.jpg` | **84.45 → 99.41** | −0.004920 (−2.63%) → 0.000000 |

**AC-2 — the two metrics do not disagree.** Both go to ~zero error on all three
cases, so there is no disagreement to report as a finding. DEC-092's warning
that mean luminance *understates* the defect on graphics is unchanged and still
worth carrying: on `graphic_large.png` a −0.44% mean error carried a 29.5-point
perceptual penalty.

**Read the "after" column the way DEC-092 asked.** crustyimg and ImageMagick now
agree **pixel-exactly** on two of the three cases — ImageMagick's own
`compare -metric AE` returns **0**, and 0.00098 on the photo. That is a genuine
result against an outside tool (ImageMagick is not the code under test), but the
agreement is tighter than two independent implementations usually manage, and
there is a mechanism: the sRGB OETF's slope is below 1 above ~0.03 linear, so it
compresses small differences in the linear intermediate and more of them round to
the same 8-bit code. In sRGB space the two implementations do *not* agree
exactly (AE 111 / 0.36 / 8.77 on the same three cases). The photo — the least
regular case — is the one that still does not land exactly, which is the
disconfirming evidence against "the harness is comparing a file to itself".

**AC-4 held structurally**: the harness generates the reference on every run and
refuses to run without ImageMagick on `PATH`; nothing crustyimg-derived was
committed as a reference. The AFTER run was repeated from a clean working
directory and the two `--json` reports are **byte-identical** (`diff` exit 0),
the same reproducibility bar DEC-092 set for the BEFORE run.

### The negative controls (AC-7) — one revert per condition

| revert | what goes red | what stays green |
|---|---|---|
| **A** — the linearization removed | the 3 linear-light tests | the alpha and no-op tests **run and pass** |
| **B** — alpha put through the transfer function | `resize_does_not_apply_the_transfer_function_to_alpha` only | the other 5 **run and pass** |
| **C** — the same-size short-circuit removed | **nothing** | all 6 pass |

Revert A on the harness returns **exactly −63.85 / 70.45 / 84.45** and an alpha
edge error of **27** — AC-7's numbers to the digit. Two harness controls flip and
are themselves evidence: **C1** (the probe's sRGB arm reproduces the shipped
binary pixel-exactly) goes PASS → FAIL, and **C3** (the binary agrees with
ImageMagick's *sRGB-space* resize) goes from mean |luma err| 0.0003 to 0.1045.

**Revert C is reported as a control that cannot fail, not hidden.** The
transfer-function round-trip is exact for all 256 8-bit and all 65,536 16-bit
values (unit-tested exhaustively), so the short-circuit has no behaviour for a
test to observe. It is kept as a cost measure and as insurance on targets whose
float behaviour differs, and DEC-095 says so.

### AC-8 — the migration, driven

Built a `crustyimg.build.toml` target with `main`'s binary, committed the
lockfile, then ran the branch binary. **At the unbumped 0.7.0** the key was
unchanged, `build --check` reported *"lockfile is up to date"* **exit 0**, and a
plain `build` served the **stale pre-fix bytes** (`1c4ebb57…`) with no warning.
**At a bumped 0.7.1** (verified applied — the binary reported `crustyimg 0.7.1`
before the arm ran; `git checkout Cargo.toml` alone does **not** rebuild the
version string) the key moved `e4abb010…` → `d8abf134…`, `--check` failed **exit
7** with an explicit drift message and left the lockfile untouched, and a plain
`build` regenerated `87642f1a…`, byte-identical to a direct branch `resize`.

All four checks hold, conditionally on the version being bumped — **the same
conditional SPEC-121 measured, now independently reproduced on a second,
unrelated byte change.** The STAGE-042 item SPEC-121 filed is confirmed rather
than assumed. The version was **not** bumped in this branch.

### AC-9 — performance, measured, and worse than hoped

`benches/pipeline.rs` `resize` (256²→128² RGB, criterion, 100 measurements per
arm, Apple M4 Pro, release, same machine, same input):

| arm | time | vs `main` |
|---|---|---|
| `main` — `U8x4`, sRGB space | **169.27 µs** | — |
| `F32x4`, **no transfer function at all** (diagnostic) | 515.88 µs | +204.6% |
| `F32x4` + threshold-table encode (diagnostic) | 641.13 µs | +279.1% |
| **shipped** — `F32x4` + `powf` encode | **648.52 µs** | **+283.5%** (p < 0.05) |

Whole-verb, best of 9 runs each, timer proven able to see a 100 ms difference
before any figure was believed:

| case | `main` | branch | ratio |
|---|---|---|---|
| 800×532 `--max 400` | 8.1 ms | 12.2 ms | 1.51× |
| 4000×2660 `--max 1600` | 113.9 ms | 203.5 ms | 1.79× |
| 4000×2660 `--max 400` | 99.0 ms | 174.6 ms | 1.76× |
| 2048² `--exact 256x256` | 20.0 ms | 49.0 ms | 2.45× |
| 800×532 `--exact 1600x1064` | 21.2 ms | 42.5 ms | 2.01× |

**The finding is that the obvious optimisation is not available.** 72% of the
added time is the `F32x4` working type itself, not the maths — the same pipeline
with the transfer function removed entirely still costs 515.88 µs. Swapping the
`powf` encode for a threshold-table search recovers **7 µs of 479** and is not
even bit-exact (2 pixels of 160,000 move by 1). Recovering the time means
changing the working type — `U16x4` has SIMD kernels `F32x4` does not — which
trades measured quality for measured speed and is a design call, not a build
one. Both diagnostic builds were discarded; the shipped code is the spec's.

### AC-9's other half — memory, measured (added by the punch-list cycle)

**AC-9 asked for performance and the build measured only time.** `F32x4` multiplies the resize
working set as well, and neither the spec nor DEC-095 mentioned memory anywhere. Peak RSS
(`/usr/bin/time -l`, "maximum resident set size"; 3 runs per cell, median reported, spread
≤ 0.5 MiB across every cell; Apple M4 Pro, release, same machine as AC-9):

| case | `main` | branch as built | branch, dst copy removed | vs `main` |
|---|---:|---:|---:|---:|
| 4000×2660 `--max 400` (downscale) | 165.9 MiB | 464.8 MiB | **462.8 MiB** | 2.79× |
| 512² → 6000×6000 (upscale) | 265.7 MiB | 1406.3 MiB | **857.0 MiB** | 3.23× |

The `main`/as-built columns reproduce verify's numbers to the megabyte, which is the control that
the two measurements are of the same thing.

**The free half was free on one case and not the other.** `resample_linear_f32x4` returned
`dst.pixels().to_vec()` — `TypedImage` has no `into_vec`, so that was a second full copy of the
float destination, 16 B per output pixel. It now returns the `TypedImage` and the caller reads the
samples in place. That is **−549.3 MiB (−39.1%)** on the upscale, where the destination is
36 Mpx — and **−2.0 MiB (−0.4%)** on the downscale, where the destination is 106 kpx and the copy
was never the cost. Output is **byte-identical** across all three arms on both cases (sha256), so
this is an allocation change and nothing else.

**What is left on the downscale is the working type, not a second avoidable copy.** The peak there
is the *source*-side linear buffer — 4000×2660 × 16 B = 170 MiB — alive at the same time as the
widened `Rgba8` source it is built from. A `drop(src)` probe (fourth diagnostic build, discarded)
moves nothing measurable: 462.8 → 462.8 MiB and 857.0 → 856.8 MiB, because the peak happens
*during* the `collect()`, while both buffers necessarily coexist. Measured and rejected rather
than assumed, and **not taken** — like the 3.83× slowdown, the remainder is the `F32x4` decision
itself, which AC-9 forbids the build from making. Both findings feed the same maintainer call.

**⚠ `MAX_AREA`'s comment was made false by this change and is corrected here.**
`src/operation/mod.rs` caps a resize at 128 Mpx and describes it as a *"512 MiB RGBA output (== the
decode allocation cap)"* bound — untrusted-input hardening (SPEC-010/037, DEC-034/038). It bounds
the **output**, and it always did; what changed is that the float intermediates are now 4× the
number it quotes, with the source's own float copy alongside, so a request at the cap can peak near
5× what the comment reads as. The **comment** is fixed to say so. The **bound** is not moved: that
is a decision about how much an untrusted input may be allowed to allocate, it is filed for the
maintainer, and per the punch list's own instruction it is said rather than done.

### AC-10 — the matrix

**Re-run in full by the punch-list cycle (2026-08-20)** — the build's
`webp-lossy` row was a false green and had to be replaced, and items 2 and 5
changed code, so nothing here is inherited:

| leg | `cargo test --release` | `clippy --all-targets -D warnings` | `fmt --check` |
|---|---|---|---|
| default | ✅ 927 passed / 0 failed (39 suites) | ✅ | ✅ |
| `--no-default-features` | ✅ 907 passed / 0 failed (39 suites) | ✅ | ✅ |
| `--features webp-lossy` | ✅ **933** passed / 0 failed (**39** suites) | ✅ | ✅ |
| `--no-default-features --features webp-lossy` | ✅ 911 passed / 0 failed (39 suites) | ✅ | ✅ |

**Twelve checks, twelve exit-0s**, read from recorded exit codes rather than
from a summary line. `tests/colour_type_preservation.rs` — SPEC-121's suite, now
a regression guard on this change — is **21/21 green**, and
`tests/linear_light_resize.rs` is **6/6**.

**⚠ The build reported `--features webp-lossy` as "835 passed / 28 suites", and
that row matched nothing.** **39 suites is invariant** across every feature
combination in this repo: 36 files in `tests/` + lib + bin + doc, and
`Cargo.toml` declares **zero** `required-features` on any `[[test]]` target, so
no suite can be feature-skipped. A 28-suite result is not a lean run or a
different selection — it is an incomplete one. The other two rows reproduce to
the test (927 and 907, unchanged), which is the control that says the counting
method is sound and the third row was the outlier
[[a-harness-that-exercises-nothing-reports-green]]. The two `webp-lossy` legs
are now both run and both reported.

Method, unchanged from the build: clean, **sequential**, fresh per-leg
`CARGO_TARGET_DIR` removed before and after each leg, nothing piped so no exit
code is swallowed.

**CI, read leg by leg rather than from the summary** (final commit; 16 pass, 0
fail, 6 skipped — the skips are the release/publish jobs that only run on a tag):

| leg | | leg | |
|---|---|---|---|
| build / test / clippy / fmt (ubuntu) | ✅ 10m51s | avif feature | ✅ 13m00s |
| build / test / clippy / fmt (macos) | ✅ 13m01s | webp-lossy feature | ✅ 13m16s |
| build / test / clippy / fmt (windows) | ✅ 14m14s | heic feature (ubuntu) | ✅ 14m57s |
| lean build (`--no-default-features`) | ✅ 4m56s | heic feature (macos) | ✅ 11m15s |
| msrv (rust 1.90.0) | ✅ 1m29s | **build + browser smoke** | ✅ 3m24s |
| supply-chain policy (cargo-deny) | ✅ 37s | cost-capture audit | ✅ 4s |
| front-matter validation | ✅ 4s | DCO sign-off | ✅ 5s |
| detect changed paths | ✅ 7s | plan | ✅ 14s |

The **windows** and **heic** legs are worth naming: they are the two the local
matrix cannot cover, and the change touches float behaviour, which is exactly
where a per-platform difference would show. Both green. `build + browser smoke`
is green on the corrected wasm baseline; it was the one leg that went red, and
its failure is written up above.

### A fourth thing the spec did not predict — the wasm bundle got 16.9% smaller

CI's `build + browser smoke` leg went **red**, and it was right to: the demo's
`.wasm` came out **1,144,921 B brotli** against a 1,394,631 B baseline, outside
the ±5% band on the **low** side. The guard's message diagnoses that direction as
"the AVIF encoder is probably missing (DEC-065)" — which is **not** what happened;
the CI log's own `wasm-pack` line carries `--features avif`.

The cause is this change. `main` calls `fast_image_resize`'s **dynamic**
`Resizer::resize`, which matches on `PixelType` and therefore monomorphizes the
convolution and alpha-multiply kernels for **all thirteen** of them. The branch
calls the typed entry point `resize_typed::<F32x4>` directly, so twelve
instantiations become unreachable and the linker drops them. Measured on one
machine, same toolchain, AVIF on both sides:

| build | raw | brotli |
|---|---|---|
| `main` | 5,819,379 B | 1,377,233 B |
| **branch** | 5,261,547 B | **1,144,864 B** |
| delta | **−557,832 (−9.6%)** | **−232,369 (−16.9%)** |
| branch, `--no-default-features` (control) | 3,266,389 B | 865,980 B |

**The baseline is moved to 1,144,921 B** (the CI measurement — CI is where the
gate runs; it and the local number agree to 57 B), with the reason recorded in
`scripts/lib/wasm-artifact.mjs` next to the constant. **The floor keeps its
discriminating power**, which is the check that mattered before touching a safety
constant: the new floor is 1,087,675 B and the lean no-AVIF control measures
865,980 B — **20.4% below it**, so a build that quietly drops the AVIF encoder
still trips the guard.

The guard's "Under:" message is also corrected. It asserted one cause and this was
the other, which is exactly the failure mode `wasm-artifact.mjs`'s own header
warns about ("misreport the cause"); it now names both and tells the reader to
check the build line first. **The punch-list cycle then had to make the log
agree with it** — the lean arm's own size banner printed the *default* feature
set, so the nearest feature line contradicted the true one. See punch-list
item 5.

### Deviations from spec

1. **⚠ AC-6's upscale half is not met, deliberately.** AC-6 asks for upscale to
   be "byte-identical to `main` where no resampling occurs" — but an upscale *is*
   a resample: Lanczos3 interpolates, and interpolating non-linear samples is
   wrong in the same way averaging them is. Gating the linearization on direction
   would put a discontinuity at exactly 100% and would have no answer at all for
   `fill`/`cover`, where one axis can shrink while the other grows. So the fix
   applies to every real resample, and **the upscale direction turns out to have
   been defective in the same way**: measured against the same independent
   reference, `graphic_large.png` 512²→1024² **65.93 → 100.00** and
   `photo_forest_cc0.jpg` 800×532→1600×1064 **89.16 → 98.44**. The **no-op** half
   of AC-6 *is* met and is asserted at four colour types (`Rgb8`, `Rgba8`,
   `Luma8`, `Rgb16`), byte-identical to `main`. The architect's ruling is invited;
   the behaviour is pinned by a test either way.

2. **The `Failing Tests` list is renamed by one entry.**
   `upscale_and_noop_resize_are_byte_identical_to_main` became
   **`noop_resize_is_byte_identical_to_its_source`** plus
   **`upscale_is_resampled_in_linear_light_too`** — a test whose name asserts
   something the measurement says is false would be worse than a rename. The other
   three names are as the spec wrote them.

3. **AC-5 was met by an improvement, not by a null.** It predicted the
   translucent-edge error would "stay at SPEC-120's measured 27/255 band". It did
   not: **max premultiplied-RGB edge error 27 → 0, mean 0.364 → 0.0**, confirmed
   independently (`compare -metric AE` = 0 against the premultiplied reference;
   the `use_alpha(false)` control arm still reads 68/18.34, so the oracle can
   still fire). **DEC-092's explanation of that residual was wrong — and so was
   this build's first correction of it.** DEC-092 read the 27 as "Lanczos ringing
   at hard corners"; the build replaced that with "8-bit quantization inside
   `fast_image_resize`'s premultiply/divide round-trip", which is right in kind
   and wrong in the specific. **It is 8-bit quantization in the integer
   resampling path generally — the alpha channel's own convolution included.**
   The evidence was already in this build's own harness output, in control
   **C4**:

   | arm | max premul RGB err | max **alpha** err | mean alpha err |
   |---|---:|---:|---:|
   | `main` — `U8x4`, premultiplication **ON** | 27 | **27** | 0.4203 |
   | **C4** — `U8x4`, premultiplication **OFF** | 68 | **27** | 0.4203 |
   | branch — `F32x4` | 0 | **0** | 0.0000 |

   Toggling premultiplication moves the premultiplied-RGB error and leaves the
   alpha statistics bit-identical — a residual that does not move when the
   variable moves is not caused by it. And `fast_image_resize`'s
   `alpha::u8x4::multiply_alpha_pixel` copies `pixel.0[3]` through untouched, so
   **alpha is never premultiplied or divided** and cannot pick up error from that
   round-trip at all. Widening the convolution to `F32x4` is the only variable
   that moved. Premultiplication itself did not move — C5
   still shows the binary differing from the non-premultiplied arm in 10,512
   pixels. **Nothing regressed**; the AC expected a null where an improvement was
   available. DEC-092 now carries this correction under *Amended 2026-08-20*,
   and `scripts/spec120_linear_light.py`'s printed footnote — the source of the
   original wording — is fixed too, so the harness stops re-emitting it.

4. **`affected_scope` was reported "confirmed, not widened" on a false premise,
   and is widened.** The build's own words were *"the diff touches
   `src/operation/mod.rs` and `tests/` only"*. `git diff --name-only` against the
   merge base says otherwise: it also touches **`scripts/lib/wasm-artifact.mjs`**
   and **`scripts/demo-assemble.mjs`** — where `WASM_BROTLI_BASELINE` was moved
   and the size guard's message rewritten — plus `docs/backlog.md` and (in this
   cycle) `justfile` and `scripts/spec120_linear_light.py`. DEC-095 records *why*
   the constant moved, but with those files out of scope nothing surfaced that
   record to the next person who edits it: **the exact failure DEC-095's own
   scope comment was written about**, repeated one spec later on the same
   record. Both `scripts/` files are now in `affected_scope`
   (`scripts/spec120_linear_light.py` is already covered by DEC-092's). The fix
   for next time is mechanical, not a resolution to be more careful: diff the
   claimed file list against `git diff --name-only`
   [[mechanical-sweeps-need-a-mechanical-check]].

**Not a deviation but worth flagging: this build ran on `claude-opus-5`, not the
`claude-sonnet-5` the spec's front matter and the build prompt both name.** The
cost below is priced at Opus anchors accordingly, per AGENTS §4 ("the model that
actually ran, not the one a prompt names").

### Punch-list cycle (2026-08-20) — 5 items

Verify returned ⚠ PUNCH LIST on PR #182, having confirmed the fix independently
(all three cases reproduced, AC-7's numbers to the digit, the wasm guard
relaxation ruled legitimate after being forced red). The fix was not
re-litigated. `cycle:` stays at `verify`.

1. **The DEC-092 correction never landed, and the correction was itself wrong.**
   The build's `decisions/` diff touched only DEC-095, so `DEC-092:127-128` still
   read *"i.e. Lanczos ringing at hard corners"* — refuted, unamended, with no
   forward pointer, while `decisions-audit` names DEC-092 as governing
   `src/operation/mod.rs`. DEC-092 now carries an **`### Amended 2026-08-20`**
   section in the convention DEC-095 uses, the refuted sentence is flagged in
   place rather than silently rewritten, and the References gained a forward
   pointer. The replacement mechanism is corrected in all four places it had been
   written (DEC-092, DEC-095, this spec, `docs/backlog.md`) — see Deviation 3 for
   the evidence, which is control **C4**, and which was in the build's own
   harness output the whole time. The harness's printed footnote
   (`scripts/spec120_linear_light.py`), where the wording originated, is fixed at
   the source so it stops re-emitting the refuted claim beside `max alpha err 0`.

2. **AC-9 measured time and never measured memory.** Added above, with the one
   avoidable copy removed and re-measured, the `drop(src)` probe measured and
   rejected, and `MAX_AREA`'s now-false comment corrected. The bound itself is
   **not** moved — said, not done, as instructed.

3. **The AC-10 `webp-lossy` row was a false green.** Re-run; the real numbers and
   why 39 suites is invariant are above. The matrix is now four legs.

4. **`affected_scope` was confirmed on a false premise.** Deviation 4 above.

5. **The lean wasm log contradicted the guard message it tells you to trust.**
   `wasm-build` ended with `@just wasm-size` — a *fresh* `just` invocation, which
   does not inherit `--set _wasm_features`, so a lean build printed the DEFAULT
   feature set in its own size banner three lines above the guard failure. The
   guard's rewritten message tells the reader to check the feature line, so a
   pre-existing cosmetic bug became load-bearing. `wasm-size` is now a **`&&`
   (subsequent) dependency** of `wasm-build`, which runs in the same invocation
   and does inherit the override; the guard message now names the
   `wasm-pack build ... -- <features>` line specifically, since that is the
   invocation and the only feature line that is always true.

   Driven both ways rather than reasoned about. New form, `just --dry-run`:
   with `--set _wasm_features " --no-default-features"` the banner interpolates
   `features:  --no-default-features` and matches the `wasm-pack` line; with no
   override both read `--no-default-features --features avif`. Old form, same
   override: the recipe's last line is a bare `just wasm-size`, and running that
   bare invocation prints `features: --no-default-features --features avif` —
   the contradiction, reproduced. `demo-build`'s ordering is unchanged — a
   dry-run still reads wasm-pack → size report → `demo-assemble.mjs` — which is
   what CI's `build + browser smoke` leg runs. The trap was flagged latent by
   **SPEC-102's** verify and never fixed; SPEC-122 is what made it matter.

### On the prototype (Call 2)

**Kept, not deleted.** `examples/spec120_linear_probe.rs` and
`scripts/spec120_linear_light.py` are the acceptance test (Call 3), they are named
in DEC-092's `affected_scope`, and their C1/C3 controls are now the cheapest
available negative control for this change — C1 flips PASS → FAIL the moment the
linearization is present. Deleting the probe would delete the way to re-derive
both the before and the after. The production code carries the prototype's
arithmetic exactly: on the synthetic case the branch binary's output is
**pixel-identical** to the probe's linear arm, which is the evidence that Call 2's
"start from the prototype" was actually followed rather than merely claimed.

### Follow-up work identified

- **The `resize` performance cost, for the architect.** A `U16x4` linear
  intermediate is the plausible recovery and is a design call (it quantizes the
  linear intermediate, where 16 bits is close to the floor for 8-bit sRGB
  shadows). Recorded in DEC-095's Consequences and Alternatives with the numbers.
- **The same-version cache hazard**, already filed on STAGE-042 by SPEC-121 and
  now confirmed on a second byte change. Not re-filed; the existing item is
  strengthened in DEC-095 instead.
- **⚠ `MAX_AREA` bounds the output, not the peak — for the architect.** The
  128 Mpx / "512 MiB RGBA" cap is untrusted-input hardening, and the linear-light
  change made its float intermediates 4× that number. The comment is corrected;
  whether the *bound* should move is a decision about how much an untrusted
  input may allocate, and is deliberately not taken here. It rides the same
  `F32x4` call as the two findings below.
- **The `resize` memory cost, for the architect — same decision as the time
  cost.** 2.79×/3.23× peak RSS after the one free copy was removed. The
  remainder is the working type; `U16x4` would halve the intermediate as well as
  restoring the SIMD kernels, at the quality cost already recorded. Numbers in
  DEC-095's Consequences.

### Build-phase reflection (3 questions, short answers)

1. **What was unclear in the spec that slowed you down?** **AC-6.** "Upscale and
   no-op resize are unaffected, byte-identical to `main` where no resampling
   occurs" reads as one claim but is two, and the qualifier contradicts the first
   half — an upscale resamples. Resolving it needed a measurement the spec did not
   ask for (upscale against the independent reference) before I could tell whether
   I was looking at a defect in my change or a defect in the AC. Cheap to fix at
   design: write the criterion against the operation's actual behaviour
   ("a same-size resize returns its input untouched") rather than against a
   direction.

2. **Was there a constraint or decision that should have been listed but
   wasn't?** The **no-op path is a `copy_image` short-circuit inside
   `fast_image_resize`**, not a resample — that is what makes AC-6's no-op half
   achievable at all, and it is a fact about the dependency that no grep of `src/`
   would surface [[a-grep-of-src-cannot-see-a-dependencys-default]]. It belonged
   in Implementation Context next to the `F32x4`/`U16x4` note. Second: **AC-9
   asked for performance to be measured but named no threshold and no baseline
   input set**, so "is 3.83× bad?" is a question the build cannot answer and the
   architect now has to.

3. **If you did this task again, what would you do differently?** **Check the
   harness's own arithmetic before trusting its first output.** The BEFORE run
   reported the prototype at *exactly* 0.000000 luma error against the reference,
   which I nearly accepted because it matched DEC-092's recorded 100.00 — two
   independent implementations agreeing to the last bit should have been
   interrogated on sight, not on the second look. It took an outside check
   (`magick compare -metric AE`) to establish it was real. I would run that
   cross-check *first*, as a standing control on the harness, rather than reaching
   for it only when a number looked too good.

---

## Reflection (Ship)

*Appended during the **ship** cycle.*
