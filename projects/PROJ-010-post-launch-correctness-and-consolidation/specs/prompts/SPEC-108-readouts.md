# SPEC-108 — VERIFY readout

Branch `spec-108-classify-the-source-image` @ `68eeac0`. Verified in an isolated worktree
(`.claude/worktrees/spec-108-build`), read-only against the repo except for a scratch test file
(`tests/zzz_scratch_*.rs`) used to synthesize fixtures for manual CLI driving — deleted before
finishing; never committed. No commits made. `main` used for comparison at `14eab28` (a detached
worktree, no branch conflict).

**Note on branch base.** The branch is not literally "main + 1 commit": it diverges from `main` at
the merge-base `3bb2102`. `main` carries two commits not in this branch — `68e6804` (a fix to the
*build prompt* text only, no source/test changes) and `14eab28` (SPEC-109 ship bookkeeping —
reflection/archive/timeline docs only). Both confirmed via `git show --stat` to touch no `src/` or
`tests/` files. No functional discrepancy results.

## Per-AC verdicts

All independently driven through my own release builds (branch + main, each in its own
`CARGO_TARGET_DIR`), not inferred from `cargo test` passing alone.

- **AC-1 — verified.** `dithered_graphic.png` through branch `web --json` at `--max`
  4096/512/256/128 all report `"class":"graphic-logo"` and no `"disposition":"lossy"` candidate.
  Re-measured myself; matches the design table exactly, including at 256 and 128 (the two traps).
- **AC-2 — verified, more strongly than the test itself checks.** The test only compares
  `--max 256` vs `4096`; I drove all four values and confirmed `features` (entropy 3.03,
  edge_ratio 0.28, flat_ratio 0.49, unique_colors 9) is byte-identical across all of them.
- **AC-3 — verified.** `dither_32color.png` through default `web` (no `--max`): `graphic-logo`,
  no lossy candidate, `winner: null` (source itself wins), `savings_percent: 0`, no
  `larger_than_source`.
- **AC-4 — verified.** `color_photo_fuji.png` and `grayscale_photo_leica.png` at `--max` 4096 and
  256: both classify `photograph` with a lossy candidate at every value.
- **AC-5 — verified, with a judgement call.** The cascade code (3.5a unconditional at
  `entropy >= DOC_ENTROPY_MAX`, 3.5b contested-zone defer at
  `[PHOTO_ENTROPY_STRONG, DOC_ENTROPY_MAX)`) matches DEC-084 Option C exactly. The new unit test
  (`halftone_scan_in_the_contradiction_band_is_not_a_photograph`) constructs `ClassifyInput`
  directly rather than synthesizing pixels — **this is a genuinely novel pattern in this file**
  (grepped `ClassifyInput {` across `src/analysis/mod.rs`: the only other construction site is
  production code inside `Analysis::compute`, not a test — no precedent for this style). I judge
  it legitimate anyway: no committed fixture can land in `[4.0, 4.5)` (photo floor 4.5176, dither
  ceiling 3.8396, both re-confirmed in RECIPES.md), engineering a real image to hit an exact
  entropy/bimodality pair is what the spec calls impractical, and `classify()` — a pure function
  of these exact features — is the actual unit AC-5's cascade-structure requirement is about.
- **AC-6 — verified, mechanically.** `grep -rn "PHOTO_ENTROPY\b\|PHOTO_FLAT_MAX" --include="*.rs" .`
  (excluding `target/`) returns three hits, all three inside the explanatory comment at
  `src/analysis/mod.rs:639-645` documenting the deletion — zero live code references anywhere.
  `rule_six_is_reachable_or_absent` does not exist under that name anywhere in the tree (grepped),
  consistent with the spec's own instruction that a deleted rule deletes its test too.
- **AC-7 — verified, with a real (not dramatic) improvement measured.** On my own lean-leg
  (`--no-default-features`) release build, `detailed_rgba_png(1600,1200)` through `web --max 400`:
  branch offers both `lossless(webp)` (112612 B, winner) and `lossless(png)` (180313 B) —
  68% savings. The identical fixture through main's lean-leg build offers **only** `lossless(png)`
  (180313 B, 49% savings) — confirms the WebP fallback is new and does help, though for this
  fixture it's an improvement (49%→68% savings), not literally rescuing a "blow-up" case.
- **AC-8 — verified.** `color_photo_fuji.png` through `optimize`: default profile → `photograph` +
  lossy (AVIF); `--profile docs` → same class, but only lossless webp/png candidates. Confirmed
  independently on my own build.
- **AC-9 — verified.** See Full Matrix below.

**Both `pipeline.run` sites, confirmed mechanically (not just cited from the build's own
account).** `grep -n "format_shortlist\|Analysis::compute" src/cli/optimize.rs` returns exactly
two lines, both inside `optimize_decide_one` (the fixed site). The other two `pipeline.run` calls —
`run_apply`'s plain-pixel-recipe path (old-main `:160`) and `run_responsive` (old-main `:1600`,
found by my own read, not just the spec's pointer) — reach neither `Analysis::compute` nor
`format_shortlist`; the first writes directly via `sink.write` with no auto-decision at all (and
explicitly rejects `--json`/`--timing` via `reject_audit_without_autodecide` for that reason), the
second writes a fixed per-width/per-format fan-out. Neither needs the fix. This matches what the
build's own timeline entry already claimed — independently re-derived, not copied.

## EXIF fallout — empirically measured, with a real finding

**The zero-entry-IFD `jpeg_with_exif` fixture (used throughout the existing suite) does NOT
exercise this risk at all.** `AutoOrient::apply` (`src/operation/mod.rs:597-617`) only drops the
metadata bundle when it finds an actual non-identity orientation tag; `orientation_from_exif_segment`
returns `None` for a zero-entry IFD, so `AutoOrient` no-ops and metadata survives the pipeline
regardless of where `Analysis::compute` runs. I confirmed this by constructing four EXIF variants
(no EXIF / zero-entry EXIF / real orientation-6 / explicit orientation-1) over three content types
(gradient, dithered-graphic-as-JPEG, a synthesized bimodal "scan") — 12 fixtures — and driving all
of them through both binaries:

| fixture | branch class | main class | flip? |
|---|---|---|---|
| scan, no EXIF | document | document | no |
| scan, zero-entry EXIF | photograph | photograph | no (AutoOrient no-ops both ways) |
| **scan, real orientation-6** | **photograph** | **document** | **YES** |
| scan, explicit orientation-1 | photograph | photograph | no (orientation=1 is also a no-op) |
| gradient / dithered-as-jpeg (all 4 EXIF variants) | photograph | photograph | no — JPEG re-encode already pushes entropy ≥ `DOC_ENTROPY_MAX`, so 3.5a claims it unconditionally regardless of EXIF |

**The one genuine flip, quantified:** `scan_real_orientation6.jpg` (17980 B) — branch ships a
lossy AVIF (2956 B, SSIM **92.2**); main ships a lossless WebP (2918 B, SSIM **100.0**). Near-equal
size, but the branch version has real, avoidable quality loss on a text-like document that used to
be pixel-perfect — for zero size benefit. This is the SPEC-108/109 real-world case: a phone
scanner app or camera-scan workflow that embeds genuine EXIF (with an orientation tag, which is the
common case, not the degenerate zero-entry case the test suite happens to use everywhere).

**DEC-084 does name this risk** (`## Consequences` → Negative: "classifying pre-pipeline means
EXIF is now present... changes behaviour for EXIF-bearing sources"; `## Validation` → "Revisit
when: ... EXIF-bearing input now reaching rule 2 pre-pipeline surfaces a real-world
misclassification worth its own fixture") — so the verify prompt's framing ("predicted at design,
absent from the build's account") slightly overstates it: the *risk* is in the account, in prose.
What's genuinely absent is any **measurement** or **fixture** of it — mine is the first. Given
DEC-084 already treats this as a known, accepted, revisit-later tradeoff (not a regression to fix
in this spec — EXIF being the decisive prior is DEC-047's rule, not new here), I'm not marking this
a blocking finding, but it should become SPEC-108's/109's promised follow-up fixture rather than
staying an open "revisit when."

**The verify prompt's claim about `tests/cli.rs:5023` is incorrect.** That line (in `main`'s
version) sits inside `web_larger_than_original_noted_on_default_channel`, which uses
`detailed_jpeg_with_icc` — no EXIF at all, not `jpeg_with_exif`. The actual no-EXIF-path claim the
prompt is worried about is already addressed by a **pre-existing, unmodified** test,
`web_classifies_a_no_exif_source` (`tests/cli.rs`, ~5185-5230 on both `main` and this branch,
attributed in its own comment to SPEC-109 AC-7): it drives `grayscale_photo_leica.png` and
`dithered_graphic.png` through `web`, explicitly asserts `has_exif:false` via `info --json` first,
then asserts the classification. This test is unchanged by this branch's diff (no hunk touches
that region) and passed in my full-matrix run. So `web`'s no-EXIF path has real, existing coverage
— the actual gap is the *EXIF-with-real-orientation* case documented above, which is genuinely new
and genuinely uncovered.

## The three modified tests — all judged as corrected truth, all independently confirmed

1. **`tests/audit_bench.rs::non_json_output_unchanged`** — fixture bumped from
   `jpeg_with_exif(256, 256)` to `jpeg_with_exif(3000, 2000)`. Verified directly: at 256×256 on the
   branch's lean-leg binary, `web` now emits **two** stderr lines (the summary plus an unrelated
   `note: ... larger than the source` line — the has_alpha fix now correctly routes this tiny
   EXIF-JPEG through a lossy re-encode that happens to land larger), which would break this test's
   `err.lines().count() == 1` assertion for a reason that has nothing to do with what the test
   checks. At 3000×2000 on the same binary: exactly one clean line, 38% smaller. The fixture bump
   is mechanically necessary, not a weakening.
2. **`tests/cli.rs::web_output_larger_than_original_is_surfaced`** and
   **`web_larger_than_original_noted_on_default_channel`** — fixture changed from
   `detailed_jpeg_with_icc(2200, 1467)` + `--max 512` to `jpeg_with_icc(gradient_jpeg(200, 150))`
   with no `--max`. Verified both directions: the **old** fixture, run through the branch's fixed
   lean-leg binary at `--max 512`, now **shrinks 68%** (60436 B from 187320 B) — it no longer
   reproduces a larger-than-source case at all, because `has_alpha` correctly reads `false` and the
   shortlist now offers a real lossy JPEG re-encode instead of an inflated lossless PNG. The **new**
   fixture genuinely reproduces the phenomenon by a different, equally legitimate mechanism: ICC
   forces a strip-and-reencode (DEC-017), and the fixed-quality (no search) JPEG re-encode of an
   already-near-optimal 2272 B source lands at 2307 B — 2% larger, `larger_than_source: true`,
   confirmed on my own lean-leg build.

All three: the underlying invariant each test guards is unchanged; only the *mechanism* used to
reproduce the triggering condition changed, because the mechanism that used to work (the has_alpha
bug) no longer exists. None of the three loosened an assertion to match new output — I checked the
assertion bodies too, none were touched.

## New finding: a real, measured performance regression against `web`'s own documented claim

Not covered by any AC, not mentioned in DEC-084's Consequences. `src/cli/mod.rs:309` documents
`web` as "Size-insensitive: a 24 MP photo finishes as fast as a small one because it downscales
first." `Analysis::compute` (`src/analysis/mod.rs:211`) is an unconditional O(pixels) full-buffer
scan (histogram, unique-color set, edge/flat pass) — its own doc comment says so. Before this
spec, that scan ran on the *post-resize* (bounded) buffer; now it runs on the *source*.

Measured on a synthetic 24 MP (6000×4000) input, three runs each, `--timing`:

- **Photograph → AVIF-lossy path** (`--json --timing`, default `web`): branch 4282–4468 ms total,
  main 4234–4342 ms total. Difference (~50–230 ms) is mostly noise — the ~3.8 s AVIF encode
  dominates and swamps the classify-placement delta almost entirely.
- **Few-colour graphic → lossless path** (same size, a coarse checker pattern so the class is
  `graphic-logo` and encode is cheap): branch 484–507 ms total, main 344–352 ms total — a
  **consistent, repeatable ~140–150 ms (≈40%) slowdown**, isolatable because `decode_ms`
  (8–14 ms) and `encode_ms` (~6 ms) are near-identical between branch and main, so the gap sits
  entirely in the unreported classify+resize segment.

So the "size-insensitive" claim is now **false specifically for lossless/graphic-class large
inputs** (where encode is cheap enough that classify's cost stops being negligible), while staying
approximately true for the lossy/photograph path (where AVIF encode still dominates). This is a
real behavior change from this spec that isn't mentioned anywhere in the spec, ACs, or DEC-084, and
the CLI help text is now a stale claim for one of the two major code paths.

## Mutation guard — confirmed live, with a must-fail control

All three steps run through `cargo test --release --lib analysis` in one isolated
`CARGO_TARGET_DIR`, sequentially, each preceded by a source edit so recompilation is forced:

- **Baseline** (`PHOTO_ENTROPY_STRONG = 4.0`, unmodified): 55 passed, 0 failed.
- **Mutation** (`= 5.5`): 54 passed, 1 failed — `calibration_gap_matches_the_documented_gap`,
  panic message: `"threshold 5.5 must fall in the calibration window (3.6414278, 4.5176096]..."`.
  **RED, as required.**
- **Must-fail control** (`= 7.0`): 52 passed, 3 failed — the calibration test plus two real-photo
  tests now failing outright (`real_exif_stripped_colour_photo_is_photograph`,
  `real_grayscale_photo_is_photograph_not_graphic`), panic messages echoing `"threshold 7"` and the
  actual measured entropies (6.373698, 6.074273). The panic messages directly quoting the mutated
  value is itself proof the binary picked up each edit — stronger evidence than grepping for a
  `Compiling` log line, which I also confirmed separately (0 extra "Compiling crustyimg" lines on a
  repeat run with no source change, 1 on each run that followed an edit).
- **Reverted** to `4.0`: 55 passed, 0 failed; `git diff --stat src/analysis/mod.rs` empty —
  confirmed no leftover mutation.

## Full matrix — clean, isolated, sequential

Each leg in its own fresh `CARGO_TARGET_DIR`, run sequentially (not concurrently), branch and
`main` (`14eab28`) both built for comparison:

| leg | branch pass/fail | main pass/fail (measured, not just cited) | delta | `Compiling crustyimg` seen |
|---|---|---|---|---|
| `--no-default-features` | 783 / 0 | 776 / 0 | +7 | yes, both |
| default | 802 / 0 | (not re-run; spec cites 796) | +6 (consistent) | yes |
| `--features webp-lossy` | 809 / 0 | (not re-run; spec cites 803) | +6 (consistent) | yes |

I independently re-ran main's lean leg myself (776/0, exact match to the spec's cited reference,
not merely trusted). I did not independently re-run main's default/webp-lossy legs — trusted the
spec's cited 796/803 given the lean leg matched exactly and the delta reconciles cleanly against
the actual new tests added (6 tests present on every leg: `dithered_graphic_stays_graphic_at_every_max`,
`classification_is_independent_of_max`, `boundary_specimen_stays_lossless_or_smaller_through_default_web`,
`real_photo_stays_photograph_at_every_max`, `halftone_scan_in_the_contradiction_band_is_not_a_photograph`,
`docs_profile_downgrades_a_promoted_image`, plus 1 more — `promoted_alpha_photo_gets_a_webp_fallback_on_the_lean_leg`
— gated `cfg(not(any(avif, webp-lossy)))`, so it only compiles on the true lean leg: 776+7=783,
796+6=802, 803+6=809, all matching exactly).

`cargo clippy --all-targets -D warnings` clean (exit 0) on all three legs. `cargo fmt --check`
clean (exit 0). Working tree clean throughout (`git status --porcelain` empty at every checkpoint);
no commits made.

## What I did NOT check

- Did not run an actual WASM build/browser round-trip (`just wasm-*` or a real browser) —
  `wasm_roundtrip.rs` ran natively as part of `cargo test`, which is not the same as exercising the
  compiled `.wasm` artifact.
- Did not run `just deny` / license-compliance tooling.
- Did not test on any platform other than this machine (macOS/Darwin) — no Linux/Windows leg.
- Did not fuzz or corpus-test the EXIF-with-real-orientation finding beyond the 12 constructed
  fixtures; a real-world corpus could surface content types I didn't try (e.g. a genuinely
  high-contrast line-art scan, or an image whose EXIF orientation is present but happens to be a
  90°/180° rotation of something that would otherwise fail Document's bimodality gate differently).
- Did not re-derive every one of the 783/802/809 individual test assertions by hand — trusted the
  green run for tests I didn't have a specific reason to distrust; spot-checked deeply only the
  AC-mapped tests, the three modified tests, and the mutation guard.
- Did not benchmark the photograph/AVIF-lossy path's classify overhead in isolation — inferred it's
  swamped by encode time from near-equal totals, but didn't instrument a classify-only timer to
  confirm the mechanism precisely (only inferred from the algorithm's shape and the graphic-path
  measurement).
- Did not check binary size or compile-time impact of this change.
- Did not exercise non-UTF8 paths, network/concurrency edge cases, or anything outside this spec's
  stated blast radius (`src/analysis/`, `src/cli/optimize.rs`, `src/analysis/decide.rs`).

## Timeline

Updated the `verify` line in
`projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-108-classification-placement-and-scale-aware-entropy-timeline.md`.

## Cost readout
cycle:            verify
spec:             SPEC-108
agent:            claude-sonnet-5
tokens_total:     45599785
breakdown:        in 550 / out 198117 / cache-write 1043030 / cache-read 44358088
duration_minutes: 555
estimated_usd:    33.65
source:           transcript sum over 275 assistant messages (session 80643cc3-8ba0-4b5b-bb21-9fb891909ccf)

Priced at the Opus anchors the build prompt specifies ($5/MTok in, $25/MTok out, cache-write
×1.25 input rate, cache-read ×0.10 input rate) regardless of the model actually used
(claude-sonnet-5), per the stated methodology. `duration_minutes` is wall-clock first-to-last
timestamp and includes idle time between messages (there was a real multi-turn gap mid-session
over an accidentally-rejected tool call), not continuous active compute — reported as instructed,
flagged as likely overstating actual working time.
