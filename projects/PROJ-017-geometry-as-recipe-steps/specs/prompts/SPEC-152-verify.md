# SPEC-152 — VERIFY prompt: `trim` as a recipe step

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not frame,
design or build SPEC-152. Your job is to find what the build's own evidence can't show. CI green
and 65 ticked ACs are claims until you drive them.

```
just wt verify/spec-152 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-152-trim:refs/remotes/origin/feat/spec-152-trim
git reset --hard origin/feat/spec-152-trim
```

Draft PR #258's head was `3d02d45` when this was written, with CI green on every leg there
(18 passed, 8 skipped; checked by the orchestrator). Record the SHA you verify. The maintainer will
paste your final message back to the orchestrator, so it must stand on its own. **Write it as plain
lists, not box-drawn tables**: a wide table was cut off in transit before.

## What the build claims
- **ACs:** 65 of 65 met.
- **Red baseline:** `tests/trim.rs` 19 of 19 red (`unknown operation 'trim'`; the non-ASCII row
  panicked at `src/text/mod.rs:60:49`); the lib tests didn't compile (`no field named 'max'`).
- **AC-3:** `float_rule_is_exact_at_every_edge` asserts 2,357,504 cases. The spec named the groups
  but not their sizes; the build **recovered the breakdown by search**: per in-range `(c, t, sign)`,
  14 match, 14 past-by-step and 7 past-by-level values, plus 65,536 level read-backs.
- **Mutations:** every row red where the spec says, with three departures:
  - **M3** was driven with a **different mutant** (`ceil(f64(s) × 255 × 257)`), not the spec's
    `f64 / 255` maths, and turned `every_colour_type_trims_exactly` red where the spec says green;
  - **M2:** AC-13's binary no-op row stayed green (the spec reasoned it red);
  - **M12:** 24 red. The mutation driver died before M12 and the restore check; the build finished
    both by hand.
- **Corpus:** 10 files added, none modified; the three per-target files `cmp`-identical; the two
  recipes' pixel lines identical to each other (design correction 3).
- **wasm:** 55 full / 53 slim, locally and in CI. Self-test 18/18, workflow 16/0. Bundle
  +13,011 B raw, +2,910 B brotli against a `main` CI run.
- **SPEC-150** (PR #257) merged mid-build; the branch was rebased onto it and force-pushed. No
  `UInt` literal lacks `max`; `help_has_no_internal_ids` passes.
- **Matrix:** default, lean and `webp-lossy`, each fresh: fmt, clippy `-D warnings`, full tests, 0.
- **Files:** 28 changed, 29 with the spec. No deviations.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Report; don't fix. A mutation you drive lives in
your worktree only and is restored before you finish.

## Read first
1. The spec on the PR branch, in full:
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-152-trim-as-a-recipe-step.md`. Read
   `## Frame`, `## Design` (its eight corrections, the op in code terms, the `UInt` `max` call, the
   match-rule table and the exact messages), the ACs, Failing Tests, the mutation table and Build
   Completion.
2. DEC-003, DEC-095, DEC-102, DEC-110, DEC-112, DEC-113.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 9, 10, 12, 13, 14 and 17 especially).
4. AGENTS §15's two newest rules: a bound or "can't happen" is a claim until its worst input is
   driven, and every finding is classified.

## Maintainer rulings the build had to honour
1. No `color` means the top-left pixel, in the image's own samples; all four sides are trimmed
   against it.
2. `tolerance` is an integer 0..=255 in 8-bit steps, per channel, inclusive: ×257 at 16 bits; a
   float sample is read as `round(s × 65535)` clamped to 0..=65535; NaN never matches. `8.0` is
   refused.
3. Alpha 0 matches alpha 0 whatever colour is hidden; otherwise all four channels are compared.
   Grey `L` compares as `(L, L, L)`.
4. One colour; each side searched on its own.
5. A whole match fails at apply (exit 1; exit 6 in a batch, nothing written for that input).
   Nothing to trim is a no-op.
6. `color` is optional and may be translucent; no `transparent` keyword.
7. The `parse_color` non-ASCII fix is in this spec.
8. No message or doc line names a spec, stage or DEC id.

## Check, in order
1. **Scope.**
   - `git diff --name-only origin/main...HEAD` against the spec's `## Outputs` plus the spec. Name
     every extra or missing file, and settle the 28/29 count.
   - `git diff origin/main...HEAD -- Cargo.toml Cargo.lock` is empty; `CACHE_SCHEMA_VERSION` and the
     lockfile format are unchanged (AC-64).
   - Every commit is signed off.
   - Say whether the PR still merges cleanly onto current `main`, and whether any commit on `main`
     since the PR's base touches a file this PR changes. Re-read `FULL_COUNT`/`SLIM_COUNT` on
     current `main` (55/53 assumes `main` is still 53/51).
   - `/usr/bin/grep -rn 'UInt {' src tests` on the PR head: every literal has `max`.
2. **The `UInt` change.** Read the `ParamKind` hunk, `expectation()` and `check()`. Confirm the three
   existing rows' messages are unchanged by driving them on the binary (a `resize` `width = 0` and a
   `watermark` `margin = -1`, say), not by reading the tests. Confirm the CHANGELOG's `### Breaking`
   line says what changed and what outside code must do.
3. **The messages, mechanically.** Extract T1–T23 and W1/W2 from the spec's "Exact messages", and
   the same from the code and `tests/trim.rs`. Diff them by script, not by eye; the spec elides a
   prefix on T9–T15, so expand it first. Report every byte that differs. Confirm the test asserts 23
   rows ran. Confirm the `internal:` fallback text differs from the table's (M15). Then
   `/usr/bin/grep` every new user-facing string and doc comment for `SPEC-`, `STAGE-`, `DEC-` and
   `PROJ-` (AC-63).
4. **The match rule, against an oracle you write.** In Python, write the rule from the rulings and
   the design's prose (levels, ×257, the float read with clamp and NaN, the alpha-0 rule, grey as
   three channels), **not** from the code. Then:
   - run all 36 rows of the design's match-rule table through your oracle and say whether each row's
     expected verdict agrees with it;
   - read `float_rule_is_exact_at_every_edge` and say whether it builds **every** group the design's
     "What was driven at design" names (`t` steps off built two ways, ±3 ulp of each, one 8-bit step
     past, one 16-bit level past, each ±3 ulp, and the 65,536 read-backs). Derive 2,357,504 yourself
     from those groups and the in-range rule; if your count differs, or the test's breakdown only
     reaches the number by a different grouping, it's a finding;
   - confirm the test's expected verdicts come from literals or the rule's prose, not from the
     matcher under test.
5. **Pixels, against an independent tool.** Use a release build from this branch, never one on
   `PATH`. Make small, odd, asymmetric PNGs with ImageMagick or Python stdlib, in 8-bit RGB, 16-bit
   RGBA and 8-bit grey, each with a border of **one colour in all four corners**.
   - At `tolerance = 0`, compare the output with ImageMagick `-trim +repage` at AE 0 and the same
     dims. ImageMagick's `-fuzz` is a colour distance, not per channel, so compare **only** at
     tolerance 0 (the frame established this; don't call a fuzz difference a bug).
   - With a tolerance: a 16-bit border 2056 levels off a stated `#000000` at `tolerance = 8` is
     trimmed, and 2057 is kept. Read the output's dims and raw samples yourself.
   - Each side on its own: a border thicker on one side than the others gives the right box.
   - The no-op: an image with nothing to trim comes back byte-identical in pixels, and an `L8` input
     stays `L8` (read the PNG header yourself).
   - Alpha: a 16-bit RGBA border of alpha 0 with varied hidden colours trims to the opaque box.
   - If any float format decodes in this build (try a 32-bit float TIFF from Python or ImageMagick),
     drive the `f32(1/255)`-at-`t = 1` edge through the binary; if none decodes, say so.
   Quote each comparison.
6. **Apply failures and batches, driven.** On the binary: a uniform image exits 1 with W1 byte for
   byte and writes nothing; the same with `color = "#FFFFFF"`, `tolerance = 5` gives W2 with the
   case kept. A two-input `--out-dir` batch with one uniform input and one that trims exits 6,
   prints `error: 1 of 2 inputs failed`, and writes only the good one. A NaN top-left with no
   `color` is a no-op, not W1 (design correction 4); drive it if a float format decodes, else read
   the unit test.
7. **The `parse_color` fix, driven.** On the PR binary: `color = "aéaaa"` in a `pad` step, a
   `watermark` step and a `trim` step each give the hex message and exit 1; `watermark --color
   aéaaa` gives `invalid color 'aéaaa'` and exit 2. Then the same four on `main`'s binary (or the
   0.8.2 release), to confirm the panic is the baseline. Confirm the check is `is_ascii` (or
   equivalent) and runs before any slice.
8. **The metadata rule, driven (DEC-113).** Tag a PNG with `exiftool -Orientation#=6`, run a `trim`
   with no `auto-orient`, and read the output with `exiftool`: no Orientation tag, and the pixels
   are the stored pixels trimmed. Then `trim` then `auto-orient`: the tag still reaches
   `auto-orient`. Read AC-16's test and control B: would it catch a decoder that drops `eXIf`?
9. **The inverse (AC-45).** Read its test. Then drive one case on the binary: an `L8` image padded
   with `#123456` then trimmed with `#123456` gives back the box as `Rgb8`, every pixel `(l, l, l)`;
   and a white top row padded white loses that row.
10. **Mutations, driven by you.** Don't re-run all 21. Drive these, each rebuilt clean (reverting
    source doesn't rebuild the binary), and compare with the table and the build's account:
    - **M3 as the spec wrote it** (`|f64(s) − c/255| <= t/255` for a stated colour on `Rgb32F` and
      `Rgba32F`), not the build's `ceil` variant. Record the red set. The spec says
      `every_colour_type_trims_exactly` stays green and `match_rule_table` is the float guard; say
      whether that holds on the real code, and whether the build's different mutant leaves the
      spec's row unconfirmed;
    - **M1** (`× 257` → `<< 8` for the tolerance): the corpus goes red on `v2_trim_tolerance × rgba16`
      only, by dims;
    - **M2** (`<=` → `<`): confirm AC-13's binary no-op row stays green, and say whether that's a
      table-cell error (wording) or a missing assertion (assert). Reason it from what the row's input
      is;
    - **M12** (trap 14: restore `main`'s `with_builtins` verbatim, not a one-line delete): the build
      finished this by hand after its driver died. Re-drive it and say whether the red set is 24;
    - **M16** (crop shifted one pixel, same size): the corpus goes red **by hash**, same dims
      (trap 13);
    - **M18** (NaN read as 0): red as the table says.
    Then confirm the tree is restored (`git status` clean, the suite green).
11. **The corpus.**
    - `git diff --diff-filter=MD --name-only origin/main...HEAD -- tests/fixtures/recipe_corpus/` is
      empty.
    - `cmp` the three per-target files for each new recipe, and compare the two recipes' pixel lines.
    - Read both `.golden` files: keys in `to_toml`'s order, and `recipe_round_trip` green on them.
    - Check the literals (`RECIPES` 22, `EXPECTED_DIMS` 22, `ROWS` 72, `GOLDEN_LINES` 75, the `66`,
      `CORPUS` 19, `pairs.len()` 22) against a count you make yourself.
    - Recompute the expected dims (`2652x1782` / `1782x2652` / `2652x1782`) by hand from the pad
      border rule and design correction 2, and compare with `EXPECTED_DIMS`.
12. **Docs (AC-55 to AC-62).** Read each surface and say whether it says what the AC says:
    top-left default, 8-bit steps with ×257 and the float read, the alpha rule, the inverse with its
    colour-type condition, the whole-match failure, and why `-fuzz` differs. No surface may promise
    a `transparent` keyword or a softer alpha rule. `tests/docs_ops.rs` must resolve every new
    example.
13. **wasm.** Run `just wasm-test` and `just wasm-test-slim` once each, and quote the result lines.
    Read the two wasm tests: do they assert exact pixels and the native whole-match text (AC-51,
    AC-52)? Run `LC_ALL=en_US.UTF-8 scripts/wasm-ci-check.sh --self-test` and `… workflow`.
14. **Full suite.** Run the default leg once and the lean leg once, each in a fresh target dir, with
    `--no-fail-fast`. Redirect stdout and read `$?`.
15. **CI.** Run `gh pr checks 258` once at the head, and say whether every leg is green.

## Classify every finding (AGENTS §15)
For each punch item and optional cleanup, say which kind it is:
- **wording:** a doc, comment or table cell that misstates something the code already does;
- **assert:** a missing or loose assertion, where the behaviour is already right;
- **claim:** a fix that would change what a doc, DEC or test claims is true, or the code's
  behaviour.

Only a **claim** needs a re-verify after its punch list. For every bound, count or "can't happen"
in the spec (the 2,357,504 cases, "the scan allocates nothing", "the side searches can't come up
empty once `top` is found", "the crop is at most the input"), say whether it was driven, by whom,
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
- Find it by content: `/usr/bin/grep -l "SPEC-152 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
  `output_tokens`.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10. Measure at your last turn. You're read-only, so report the numbers; a later session
  writes them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only, no box-drawn tables. First line, exactly one of:
- `✅ APPROVED — merge PR #258 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #258 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–15, one or two lines each, with evidence;
- the punch items, numbered, each with its kind (wording / assert / claim), its evidence and the
  fix you'd expect;
- optional cleanups, listed separately, each with its kind;
- out-of-spec findings, each with the stage it belongs to;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
