# SPEC-115 — BUILD prompt

Cycle: **build**. You are NOT the architect. The design is settled; implement it.

**One-line summary:** `cat logo.svg | crustyimg optimize - --out-dir out/` writes **`out/stdin.jpg`
containing XML** and reports it as a PNG — because the auto-decide passthrough trusts an *adopted*
`source_format` label as if it named the bytes on disk. Same defect for HEIC and RAW.

## Read in order

1. **The spec** — `projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-115-optimize-never-passes-through-bytes-it-cannot-name.md`,
   in full. **13 acceptance criteria, 4 settled design calls, 9 pre-written tests, a negative
   control per family.**
2. **The code** — `src/cli/optimize.rs:930-1165` (`optimize_decide_one`, **both** passthrough
   exits), `:1183-1201` (`encode_one_optimize_decided`), `:1270-1292` (the note);
   `src/analysis/decide.rs:222-265` (`pick_winner`); `src/image/mod.rs:410-520` (the decode
   dispatch that adopts the labels); **`src/image/sniff.rs`** (Call 1's evidence).
3. **SPEC-113's branch** `feat/spec-113-optimize-pinned-never-bigger` — it adds
   `pipeline_altered_source`, which you extend. **Rebase on it; do not fork the helper.**
4. **`/AGENTS.md`** — §4 cost, §6 commands, §12 testing, §13 git/PR.

## The driven facts (this repo, `08b367d`, committed fixtures)

```
cat tests/fixtures/svg/rect_text_40x30.svg | crustyimg optimize - --out-dir out/
  → out/stdin.jpg          file: SVG Scalable Vector Graphics image     "kept png … already optimal"

crustyimg optimize tests/fixtures/svg/rect_text_40x30.svg --out-dir out/ --explain
  → png → png (336 → 336 B, 0% smaller)      the written file is SVG XML

crustyimg optimize tests/fixtures/raw/oversize_preview.dng --out-dir out/ --explain
  → jpeg → jpeg (643 → 643 B, 0% smaller)    the written file is the TIFF/DNG container
```

Reproduce all three yourself before changing anything.

## The design calls are SETTLED — do not reopen them

1. **The predicate is RECORDED AT LOAD, not sniffed.** `Image` gains an origin alongside
   `source_format`; the three adopting decoders set it. `source_format()`'s value does not change.
2. **An unshippable source takes the `pipeline_altered` branch** — smallest correct candidate,
   even if larger, with the honest note. No new policy, no size escape hatch.
3. **The larger-than-source note gains its fourth reason** (its current three are all false here).
4. **The report names the real container** on both channels; schema stays `explain/v1` (DEC-074).

## The trap that will make you ship a green build over a live defect

**Do NOT reach for `::image::guess_format(&raw).ok() == Some(fmt)`.** It looks right and it is
wrong on this path:

- `image` 0.25.10 matches AVIF only when the **major brand** is literally `avif`
  (`io/free_functions.rs:120`).
- This crate's own `sniff::is_avif` also accepts a `mif1` major brand with `avif` in the
  compatible brands — **and unit-tests exactly that** (`src/image/sniff.rs:67-74`).
- So a `mif1`-major AVIF would fail the sniff and a **correct passthrough becomes a worse,
  lossy-over-lossy re-encode**. On SPEC-113's pinned path the same expression is safe (a false
  negative there just ships the re-encode, which is today's behaviour). Same code, different
  blast radius. **This is why Call 1 is what it is.**
- All three committed AVIF fixtures are `ftypavif`-major, so a test built on them **cannot catch
  this**. AC-6 requires a fixture built for it.

## The second trap: a fixture that cannot reproduce

- SVG reproduces with the committed fixture. **RAW does not** — `synthetic_preview.nef` correctly
  re-encodes to 160 B and never enters the branch, and `oversize_preview.dng` does reproduce but
  its candidates are 59 MB / 156 MB. You need a **new RAW fixture with a NOISE preview**, so the
  lossless candidates exceed the container. Hand-build it like `synthetic_preview.nef`
  (SPEC-061 documents the shape); generate the preview JPEG with an **independent tool**.
- Same question for HEIC under `--features heic`. If you cannot build one, **say so in Build
  Completion and state what is therefore unproven.** Do not ship a test that cannot fail.

**Prove every new test RED on `main` before writing the fix.** Per family.

## Do not miss the second exit

`optimize_decide_one` passes through in **two** places: `pick_winner → None` (`:1086-1098`) and the
degenerate-analysis early return (`:1006-1015`), three lines above the analysis. The spec settles
the second one: keep the passthrough when the container is shippable, fail with a typed error when
it is not.

## Verify before handing back

Full matrix, fresh per-leg `CARGO_TARGET_DIR`, **sequentially**, **through `rtk proxy` from the
first leg** — default, `--no-default-features`, `--features webp-lossy`, **and `--features heic`**
(CI runs that job; `ci.yml:104-145`). Confirm each log says `Compiling crustyimg`; treat a missing
one as a tooling failure first, and use `/bin/cat` for binary.

**A piped command reports the pipe's exit code** — `cargo test | tail` turns a red leg green.
Redirect to a file and read `$?`.

Run AC-11's negative controls and record them: revert the guard, confirm the SVG / RAW / HEIC tests
go RED, restore. **Prove the revert reached the built artifact** — a changed binary hash shows a
rebuild, driving shows the change took effect.

**Then read the CI legs on your PR individually before claiming green**, the `heic` job included.

## Repo guardrails

`git commit -s` (DCO enforced). Never `git reset --hard`. **Own git worktree — other sessions are
live in this repo**, so do not work in the primary checkout and do not assume `target/` is yours.
macOS has no `timeout(1)`. Cross-check anything load-bearing with `/usr/bin/git` or `python3` plus
a positive control. **Do not merge the PR. Do not bump the version.**

## When you finish

Fill in `## Build Completion` and the three reflection questions. Write the DEC the spec expects
(the `Image` model now separates "format of the decoded pixels" from "container on disk"), with
`affected_scope` covering `src/image/**` and `src/cli/optimize.rs`.

### Cost

Follow `projects/_templates/prompts/cost-snippet.md`. **Identify your transcript by something only
your session emitted — never by "the newest `.jsonl` in the directory."** Price **per component**
at the anchors of the model `.message.model` actually reports. Close with the `## Cost readout`
block, verbatim, as the last thing you emit.

**Report what you could not do as clearly as what you did.**
