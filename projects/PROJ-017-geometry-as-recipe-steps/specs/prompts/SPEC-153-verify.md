# SPEC-153 — VERIFY prompt: `edit` flags for crop, rotate and flip, in typed order

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not frame,
design or build SPEC-153. Your job is to find what the build's own evidence can't show. CI green
and 27 ticked ACs are claims until you drive them.

```
just wt verify/spec-153 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-153-edit-geometry-flags:refs/remotes/origin/feat/spec-153-edit-geometry-flags
git reset --hard origin/feat/spec-153-edit-geometry-flags
```

Draft PR #261's head was `d917d60` when this was written, with CI green on every leg there
(18 passed, 8 skipped, all eight release jobs; checked by the orchestrator). Record the SHA you
verify. The maintainer will paste your final message back to the orchestrator, so it must stand on
its own. **Write it as plain lists, not box-drawn tables.**

## What the build claims
- **ACs:** 27 of 27 met. `just decisions-audit` exits 0.
- **Byte pin:** FT-1 passed on `main`'s code before any source edit, and after. FT-2 failed on
  `main` with `0x62d0525855cff6a7` and passes after.
- **Red baseline:** 22 of 23 `edit_geometry` tests failed on `main` at clap's
  `unexpected argument '--crop' found`; `auto_orient_alone_is_accepted` passed there.
- **Mutations:** all 15 rows and trap 14 went red. Matches the table: M3–M11, M13, M14, M7. Every
  reasoned cell confirmed. Cells that differ (more red than the table):
  - **M1** also reds FT-9 and FT-10;
  - **M2** also reds FT-10, 11, 12, 30 and `tests/cli.rs`' `stub_command_returns_not_implemented`;
    FT-2 stayed green;
  - **M15** also reds the unit test `edit_steps_require_an_op_flag` and `tests/orientation.rs`'
    `already_correct_verbs_do_not_double_rotate`.
  - Trap 14: FT-2 red on its hash, 22 of 23 `edit_geometry` red; FT-1, FT-25 and the 9 other
    `tests/edit.rs` tests green. Unit tests don't compile on `main`, so that run used the
    integration targets only.
- **Files:** 10 with the spec, exactly the spec's Outputs.
- **Deviations:** stages 2–4 went in as one checkpoint push (`07b766f`); FT-14 uses `tests/common`'s
  little-endian EXIF helper, not the big-endian one the spec describes.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Report; don't fix. A mutation you drive lives in
your worktree only and is restored before you finish.

## Read first
1. The spec on the PR branch, in full:
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-153-edit-flags-for-the-five-geometry-ops-and-the-ordering-rule.md`.
   Read `## Frame` (with the maintainer rulings), `## Design` (corrections, drives, the design in
   code terms, every message verbatim), the ACs, Failing Tests, the mutation table and Build
   Completion.
2. DEC-114 (`decisions/DEC-114-edit-runs-its-op-flags-in-the-order-typed-and-saves-what-ran.md`),
   and DEC-005, DEC-102, DEC-106, DEC-112, DEC-113.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 10, 12, 13, 14 and 17 especially).
4. AGENTS §15's rules: a bound or "can't happen" is a claim until its worst input is driven, and
   every finding is classified.

## Maintainer rulings the build had to honour
1. Op flags (`--resize-max`, `--invert`, `--crop`, `--rotate`, `--flip`) run in the order typed;
   `auto-orient` always first and positionless; each op flag at most once (exit 2).
2. `edit --invert --resize-max N` now inverts first, with a CHANGELOG "Changed" line.
3. `--crop` takes `WxH+X+Y` (pixels) and `W:H` (aspect, optional `--crop-gravity`) only.
4. A pixel crop past the edge of the image the step receives is exit 1, naming the flag and that
   image's size.
5. From design (DEC-114 part 6): with `--save-recipe`, a pixel `--crop` needs one input image; a
   batch is refused (P16, exit 2) before any image is read. A failed run writes no recipe.
6. No message, help line or comment names a spec, stage or DEC id.

## Check, in order
1. **Scope.**
   - `git diff --name-only origin/main...HEAD` against the spec's `## Outputs` plus the spec.
   - `git diff origin/main...HEAD -- Cargo.toml Cargo.lock` is empty; `CACHE_SCHEMA_VERSION` and the
     lockfile format unchanged; no `edit` code in `src/wasm.rs` (AC-25).
   - Every commit is signed off.
   - Whether the PR still merges cleanly onto current `main`, and whether any commit on `main` since
     the PR's base touches a file this PR changes. Run `gh pr list --state open --json number,files`
     and name every open PR that touches `src/cli/mod.rs`, `src/cli/ops.rs`, `tests/edit.rs` or
     `CHANGELOG.md` (the build's own overlap line was cut off in its report).
2. **DEC-114 against the code.** Read each of its seven parts and point to the code that does it.
   Any part the code doesn't follow, or does differently, is a finding.
3. **The messages, mechanically.** Extract P1–P16 and A1–A3 from the spec's "Every message,
   verbatim", and the same from the code and the tests. Diff them by script, not by eye, and report
   every byte that differs (including `×` U+00D7 in A1/A2). Then drive each on the PR binary and
   diff stderr against the spec, with the exit code. Check `crustyimg edit --help` against the
   spec's help sentences. Then `/usr/bin/grep` every new user-facing string and comment for `SPEC-`,
   `STAGE-`, `DEC-` and `PROJ-`.
4. **Typed order, driven.** Use a release build from this branch, never one on `PATH`.
   - The frame's case: a 60×40 PNG, `--crop 30x40+0+0 --rotate 90` and `--rotate 90 --crop 30x40+0+0`
     (or the spec's equivalents): dims and pixels each match ImageMagick doing the same two steps in
     the same order (`magick in.png -crop 30x40+0+0 +repage -rotate 90 …`), at AE 0.
   - Spelling and position: `--crop=…` against `--crop …`, and flags before against after the input.
   - `edit in.png` with no op flag exits 2 (P15); `edit in.png --auto-orient` alone runs.
   - A repeated op flag, and `--crop-gravity` with no `--crop`, each exit 2.
   - Each FT's expected values: say whether they come from literals or an independent source, not
     from `edit_op_order`, `crop_fractions` or the run under test.
5. **The byte pin, independently.** In Python stdlib, regenerate the 61×41 fixture from the spec's
   seed rule (xorshift64 from `s = 7`, `[0, 30, 255][s % 3]`), write it as a PNG, run the PR binary
   on it for each pinned invocation, and hash the decoded output with your own FNV-1a 64 over a
   stdlib PNG decoder. Compare with the spec's five hashes, including the "Changed" row
   (`--invert --resize-max 30` → `0x5d248c136b8b3ad7`). Run the same on `main`'s binary (or the 0.8.2
   release) for the old value `0x62d0525855cff6a7`. Include a control that shows your decoder can
   tell two images apart. Then confirm FT-1 builds its fixture the same way you did, not through
   `crustyimg` or `image`.
6. **Pixel crops, driven.**
   - On the binary: a pixel crop on its own, after `--resize-max`, and after `--rotate 90` on a
     non-square input. Compare each with ImageMagick running the same steps, at AE 0.
   - Read each saved TOML's fractions and check in Python that each equals `X / W` (in `f64`) of the
     **size the step received**, exactly, with nothing rounded.
   - Save `--crop 631x1+9+0` on a 640-px-wide image and `apply` the recipe; it must load and give
     the same pixels. Confirm FT-12 carries (640, 9, 631) as a literal row, plus an interior row
     where `X/n + W/n ≠ (X+W)/n` bitwise (check one yourself in Python).
7. **Replay.** For a handful of FT-6/7/10 rows of your choosing (including 16-bit and grey):
   `edit … --save-recipe r.toml`, then `apply r.toml`; the decoded pixels, colour type and dims
   match, and the saved step list is in typed order. Then an Orientation-5 tagged input with
   `--rotate 90 --crop …`: the saved recipe starts with `auto-orient`, and replay matches. Say
   whether FT-14 would still catch M6 with the little-endian EXIF helper the build used (that is,
   does the Orientation-5 row really carry a tag the decoder reads? prove it with a control).
8. **Failures and batches, driven.**
   - A1 and A2 (past the edge after a resize, and after a rotate): exit 1, the message names the
     received size, no output file and no recipe written.
   - P16: a 3-image directory with a pixel `--crop` and `--save-recipe`, exit 2, nothing written.
     The same batch without `--save-recipe`, with one image too small: exit 6, the others written.
     An aspect `--crop` batch with `--save-recipe`: exit 0 and one recipe.
   - `--rotate -90` and `--rotate=-90` run; a bare `-g.png` input is refused the same way on `main`
     and on the PR; `-- -g.png` runs on both.
9. **Mutations, driven by you.** Don't re-run all 15. Drive these, each rebuilt clean (reverting
   source doesn't rebuild the binary), with the **exact edit the spec quotes**, and compare with the
   table and the build's account:
   - **M2** (drop the `value_source` filter): record the red set, and **explain why
     `tests/cli.rs`' `stub_command_returns_not_implemented` goes red.** That test isn't about `edit`.
     If the order collection runs for subcommands other than `edit`, or a defaulted flag leaks into
     another verb, it's a finding; say which kind.
   - **M1** (Alternative A's fixed order): confirm the extra FT-9/FT-10 reds, and whether the spec's
     "FT-9 green under M1" cell was a table error (wording) or the build wrote a stronger row.
   - **M4** (`{:.6}`): only FT-11 and FT-12 red.
   - **M6** (`--auto-orient` takes its typed position): only FT-14's Orientation-5 row reds.
   - **M10** (past-the-edge check removed): only the message assertion reds; the exit stays 1.
   - **Trap 14** (restore `main`'s `src/cli/mod.rs` and `src/cli/ops.rs` verbatim, tests kept):
     the red and green sets match the build's account.
   Then confirm the tree is restored (`git status` clean, the suite green).
10. **The checkpoint deviation.** Stages 2–4 landed in one commit. Read `07b766f`; say whether
    anything the spec placed in a later stage is missing or only partly done, or whether this is
    process only.
11. **Docs (AC-24) and CHANGELOG.** Read each surface the spec lists (`--help`,
    `docs/cli-reference.md`, `docs/api-contract.md`, `docs/recipes.md`, `recipes/README.md`) and say
    whether it states typed order with `auto-orient` first, the new flags, that a pixel `--crop` is
    saved as fractions of the image at that step, and the batch rule. `cli-reference.md`'s stale
    orientation lines (since SPEC-111) are rewritten. CHANGELOG: three "Added" lines and one
    "Changed" line that a user would understand without the spec. `tests/docs_ops.rs` green.
12. **Full suite.** Run the default leg once and the lean leg (`--no-default-features`) once, each in
    a fresh target dir, with `--no-fail-fast`. Redirect stdout and read `$?`. The lean leg must run
    `tests/edit_geometry.rs`, not skip it: show its count.
13. **CI.** Run `gh pr checks 261` once at the head, and say whether every leg is green.

## Classify every finding (AGENTS §15)
For each punch item and optional cleanup, say which kind it is:
- **wording:** a doc, comment or table cell that misstates something the code already does;
- **assert:** a missing or loose assertion, where the behaviour is already right;
- **claim:** a fix that would change what a doc, DEC or test claims is true, or the code's
  behaviour.

Only a **claim** needs a re-verify after its punch list. For every bound, count or "can't happen"
in the spec (the round trip's 22.5M and 13M rectangles, "pad's rounds-to-0 is unreachable", "the
conversion can't trip the op's fraction bound", the 86 rows), say whether it was driven, by whom,
and over what range.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- Reverting source doesn't rebuild the binary, so rebuild clean after each mutation.
- zsh does not word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate
  `grep` and `git log`, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Never poll CI and never write your own
  wait loop (AGENTS §16 habit 6). Never drive the maintainer's browser.
- Write scratch files only in your session's scratchpad, never `/tmp`. Delete scratch target dirs
  and scratch images, and leave the tree clean.
- **Budget ~70 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `/usr/bin/grep -l "SPEC-153 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
  `output_tokens`.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10. Measure at your last turn. You're read-only, so report the numbers; a later session
  writes them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only, no box-drawn tables. First line, exactly one of:
- `✅ APPROVED — merge PR #261 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #261 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–13, one or two lines each, with evidence;
- the punch items, numbered, each with its kind (wording / assert / claim), its evidence and the
  fix you'd expect;
- optional cleanups, listed separately, each with its kind;
- out-of-spec findings, each with the stage it belongs to;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
