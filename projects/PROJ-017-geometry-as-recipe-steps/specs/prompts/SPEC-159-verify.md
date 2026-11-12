# SPEC-159 — VERIFY prompt: pad with a transparent fill, and the no-alpha output rule

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not frame,
design or build SPEC-159. Your job is to find what the build's own evidence can't show. CI green
and 22 ticked ACs are claims until you drive them.

```
just wt verify/spec-159 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-159-pad-transparent:refs/remotes/origin/feat/spec-159-pad-transparent
git reset --hard origin/feat/spec-159-pad-transparent
```

Draft PR #262's head was `920b5ef` when this was written, with CI green on every leg there
(18 passed, 8 release jobs skipped; checked by the orchestrator). Record the SHA you verify. The
maintainer will paste your final message back to the orchestrator, so it must stand on its own.
**Write it as plain lists, not box-drawn tables.**

## What the build claims
- **ACs:** 22 of 22 met, with one gap: DEC-115's `affected_scope` lacks `src/cli/mod.rs` and
  `src/operation/trim.rs`, which AC-1 requires. Build didn't edit it and didn't run
  `just decisions-audit`. DEC-115 lives on `main` (written at design).
- **Byte pin:** FT-6 passed on `main`'s code before any source edit and after the last change, in
  the default, lean and `webp-lossy` legs.
- **Red baseline on `main`:** the translucent rows failed at load with "not supported yet"; FT-16
  exited 1 where 4 is due; FT-17 and FT-6 were green.
- **Mutations:** 17 driven, each with a clean rebuild; M18 by search. "No spec cell was wrong", but
  the build's own account differs from the table in two places:
  - **M1, M2, M3** also red FT-2 ("`Display` reads the capacity"). The table lists FT-2 as reasoned
    and leaves it out of those red sets.
  - **M13:** the table puts `pad_then_trim_is_the_identity` under `tests/pad.rs`; it is in
    `tests/trim.rs`.
  - Confirmed reasoned cells: M3's FT-11, M9's single-input row, M11 (all Rust green; the parity
    harness against a wasm build of the mutant printed `FAIL alpha-B: transform to jpeg did not throw`
    and exited 1).
- **Files:** Build Completion says 22 files; `gh pr view 262 --json files` lists 25 including the
  spec. Two are outside the spec's file list, both caught by CI: `tests/wasm_roundtrip.rs` (row P30,
  which asserted the old load refusal, removed; `transform_refuses_a_fill_alpha_the_format_cannot_store`
  added) and `scripts/wasm-ci-check.sh` (`FULL_COUNT` 55 → 56, `SLIM_COUNT` 53 → 54, self-test
  fixtures renamed).
- **wasm size:** brotli 1,308,562 B, +3.3% over `WASM_BROTLI_BASELINE`, inside ±5%; baseline unmoved.
- **Docs:** every behaviour sentence driven on a release build, "all as written" (list in Build
  Completion).

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Report; don't fix. A mutation you drive lives in
your worktree only and is restored before you finish.

## Read first
1. The spec on the PR branch, in full:
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-159-pad-with-a-transparent-fill-and-the-no-alpha-output-rule.md`.
   Read `## Frame` (with the maintainer rulings), `## Design` (corrections 1–8, the design in code
   terms, every message verbatim, the byte pin, wasm), the ACs, Failing Tests, the docs list, the
   mutation table and Build Completion.
2. DEC-115 (`decisions/DEC-115-a-step-whose-alpha-the-output-format-cannot-store-is-refused-before-the-pipeline-runs.md`),
   and DEC-015, DEC-095, DEC-102, DEC-110.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for" (traps 5, 10, 12, 13 and 14 especially).
4. AGENTS §15's rules: a bound or "can't happen" is a claim until its worst input is driven, and
   every finding is classified.

## Maintainer rulings the build had to honour
1. **GIF:** fill alpha `00` and `ff` (and `transparent`) are allowed; `01`–`fe` are refused (exit 4).
2. **`trim`** accepts `transparent` as `#00000000`, through a helper shared with `pad`.
   `parse_color` itself is unchanged.
3. **Source alpha:** only alpha a `pad` brings is refused. An RGBA source written to JPEG still
   drops alpha silently, and the docs say so.
4. The check is static (recipe × format), runs once the format is known and before the pipeline:
   exit 4 for one input, or a batch or `build` target with one format (nothing written, no
   out-dir); exit 6 per input when each keeps its own format; wasm throws the same line.
5. The auto-decided format keeps alpha after a translucent pad (design correction 1).
6. No message, help line or comment names a spec, stage or DEC id.

## Check, in order
1. **Scope.**
   - `git diff --name-only origin/main...HEAD` against the spec's build file list and docs list,
     plus the spec. Give the real count and name every file outside the lists.
   - `git diff origin/main...HEAD -- Cargo.toml Cargo.lock` is empty; `CACHE_SCHEMA_VERSION` and the
     lockfile format unchanged; `scripts/lib/wasm-artifact.mjs` unchanged.
   - Every commit is signed off.
   - The PR merges cleanly onto current `main`, and no commit on `main` since the merge base touches
     a file this PR changes. `gh pr list --state open --json number,files`: name every open PR that
     touches `src/cli/ops.rs`, `src/cli/mod.rs`, `src/cli/optimize.rs`, `src/operation/pad.rs`,
     `tests/pad.rs` or `CHANGELOG.md`.
2. **DEC-115 against the code.** For each of its parts, point to the code that does it. Any part
   the code doesn't follow, or does differently, is a finding. Then:
   - list every `src/` file the diff touches for this rule, and compare with `affected_scope`
     (the build says `src/cli/mod.rs` and `src/operation/trim.rs` are missing; confirm or correct);
   - run `just decisions-audit` and quote its result.
   DEC-115 is on `main`, so its fix is a `main` edit; say whether it can wait for ship.
3. **Every path that runs a recipe and encodes.** Enumerate them from the code yourself, not from
   the spec's list of seven sites: `apply` (one input, batch, terminal `optimize` pinned and
   decided), `build` (pinned, preserve, decide, and `--watch` if it exists), wasm `transform` (with
   and without `out_format`), and anything else that loads a recipe and writes pixels. For each, say
   whether a check covers it or why a `pad` can't reach it. Then say whether any op other than
   `pad` can add alpha to an image without it (any `widen`, `to_rgba*`, or a colour-type change in
   `src/operation/`). If one can, that is an out-of-spec finding: DEC-115 says only steps that
   report `introduced_alpha` are checked.
4. **The messages, mechanically.** Extract the three lines (JPEG `transparent`, JPEG `#ff000080`,
   GIF `#ff000080`) and the wasm form from the spec's "Every message, verbatim", and the same from
   the code and the tests. Diff them by script, not by eye. Then drive each on a release build of
   this branch (fresh `CARGO_TARGET_DIR`, never a binary on `PATH`) and compare stderr with `==`,
   with the exit code. Add rows the tests don't have:
   - an upper-case alpha (`#FFFFFFFE`): `{aa}` prints `fe`, and the written value is quoted as typed;
   - `color = "ff000080"` (no `#`);
   - a recipe with two translucent pads where step 1 is the first unstorable one.
   Then `/usr/bin/grep` every new user-facing string and comment for `SPEC-`, `STAGE-`, `DEC-` and
   `PROJ-`.
5. **Fill values and widening, independently.** On the binary, not the tests: pad an RGB8, an L8, an
   LA16 and an RGBA16 PNG with `transparent`, `#ff000080` and `#80808080`. Decode each output
   with a decoder you didn't write and that isn't `image` (ImageMagick `-depth N … rgba:`, or a
   stdlib Python PNG decoder), and check the colour type, the fill pixel (16-bit = `c × 257`, alpha
   included) and that every source pixel is copied unchanged. Say where FT-4's expected values come
   from: literals or an independent model, not `pad.rs` or the run under test.
6. **The byte pin, independently.** In Python stdlib, regenerate the four 5×3 fixtures from the
   spec's rule (xorshift64 `x ^= x << 13; x ^= x >> 7; x ^= x << 17`, seeds `0x1590000`–`0x1590003`,
   pixel-then-channel order, masked to the sample width), write each as a PNG with your own encoder,
   run the PR binary's `apply` with the spec's recipe, decode the output with your own decoder, and
   hash it with your own FNV-1a 64 (RGB16 little-endian). Compare with the four literals. Run the
   same on `main`'s binary. Include a control that shows your pipeline can tell two images apart.
   Confirm FT-6 builds its fixtures the same way and not through crustyimg code.
7. **The format rule, driven.** One input: every rung of AC-8, exit 4, nothing written. GIF: `00`,
   `01`, `80`, `fe`, `ff` and `transparent`; read the `#ff000000` border back from the GIF with
   ImageMagick, not `image`. Batch with one format (`--format`, `--name-template '{stem}.jpg'`,
   `recipe.format`): exit 4, one unlabelled line, no out-dir. Kept formats over `src.jpg` +
   `rgb.png`: exit 6, the labelled line, then the summary, `rgb.png` written. `build` pinned and
   preserve, as AC-14. AC-16's at-aspect image: `-o a.jpg` exit 4, `-o a.png` exit 0 with the input's
   colour type. AC-17's `--format bogus` precedence row.
8. **Auto-decide (design correction 1).** A JPEG source through `[pad transparent, optimize]` with no
   format, via `apply --out-dir` and via `build` in a **fresh** project dir (a cached JPEG from an
   earlier run masks it): the output is not JPEG, decodes with alpha, and its corner pixel has alpha
   0 (read with ImageMagick). Then the size fallback: find the code path that falls back to a lossy
   entry (`fast_fallback_lossy_entry`) and say whether it can still pick JPEG after a translucent
   pad, with the line that stops it. The at-aspect "kept jpeg" row: drive it.
9. **Unchanged rows.** An RGBA PNG with half its pixels at alpha 0, recipe `flip`, `-o f.jpg`: exit 0,
   stderr empty. An opaque `pad` to JPEG: exit 0, stderr empty. `trim` with `transparent` and with
   `#00000000` give the same output bytes on an RGBA input; `"Transparent"` and `"TRANSPARENT"` are
   still refused at load with the hex message, for both `pad` and `trim`.
10. **wasm.**
    - Read the `tests/wasm_roundtrip.rs` diff: say what row P30 asserted, what replaces it, and
      whether anything that row covered is now uncovered.
    - Read the `scripts/wasm-ci-check.sh` diff: say whether the self-test still exercises the same
      cases with the new numbers (the fixtures were renamed as well as renumbered).
    - Run `just ddphotos-parity` once and quote its summary. The build drove M11; you need not.
11. **Mutations, driven by you.** Don't re-run all 18. Drive these, each rebuilt clean (reverting
    source doesn't rebuild the binary), with the **exact edit the spec quotes**, full suite
    `--no-fail-fast`, and compare the red set with the table and the build's account:
    - **M1** (GIF as full): confirm whether FT-2 reds, and if it does, whether the table's cell is a
      wording error or the build wrote a stronger FT-2 than the spec described;
    - **M6** (no single-input check): only FT-8, FT-9, FT-15 red;
    - **M8** (no batch pre-check): only FT-10 red, and FT-10 fails on the fallback's exit 6, not on
      something else;
    - **M15** (`trim` on `parse_color`): FT-18 (both), FT-19, FT-9;
    - **M17** (decided path reads only the source's alpha): only FT-13;
    - **Trap 14** (M16: `git show origin/main:src/operation/pad.rs > src/operation/pad.rs`): FT-6
      stays green and the build's red set holds.
    Then confirm the tree is restored (`git status` clean, the suite green).
12. **Docs (AC-22) and CHANGELOG, each sentence driven.** Read every row of the spec's docs list on
    the PR (`src/operation/pad.rs` doc comments, `docs/recipes.md`, `docs/api-contract.md`,
    `docs/cli-reference.md`, `docs/data-model.md`, `recipes/README.md`, `skills/crustyimg/SKILL.md`,
    `CHANGELOG.md`, `docs/backlog.md`). For **each sentence that states behaviour**, quote it and
    drive it on the binary, or say which earlier item drove it. Pay attention to:
    - "approximately" for AVIF: the build read alpha 0.4549 for `#ff000080` on a 4×4 image (128 would
      be 0.502). Does the doc's wording cover an error that size on small images?
    - the TIFF sentence (crustyimg reads alpha back; ImageMagick shows it opaque);
    - the source-alpha-into-JPEG sentence (ruling 3);
    - the exit-4 rows in `api-contract.md`, `cli-reference.md` and `SKILL.md`.
    SPEC-153's only defects were doc sentences nobody drove; treat these as the likeliest place for
    one. CHANGELOG: lines a user would understand without the spec. `tests/docs_ops.rs` green.
13. **Full suite.** Run the default leg once and the lean leg (`--no-default-features`) once, each in
    a fresh target dir, with `--no-fail-fast`. Redirect stdout and read `$?`. Show the `tests/pad.rs`
    and `tests/trim.rs` counts in the lean leg.
14. **CI.** Run `gh pr checks 262` once at the head, and say whether every leg is green.

## Classify every finding (AGENTS §15)
For each punch item and optional cleanup, say which kind it is:
- **wording:** a doc, comment, DEC front-matter field or table cell that misstates something the
  code already does;
- **assert:** a missing or loose assertion, where the behaviour is already right;
- **claim:** a fix that would change what a doc, DEC or test claims is true, or the code's
  behaviour.

Only a **claim** needs a re-verify after its punch list. For every bound, count or "can't happen"
in the spec ("no cache holds such an output", "the shortlist never offers GIF", "the passthrough
can't fire after a pad that writes a fill pixel", "only `optimize.rs:95` can carry a pad into
`run_pixel_op`", the 50 + 10 widening cells, the 40 capacity cells), say whether it was driven, by
whom, and over what range.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- Reverting source doesn't rebuild the binary, so rebuild clean after each mutation.
- zsh does not word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate
  `grep` and `git log`, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Never poll CI and never write your own
  wait loop (AGENTS §16 habit 6). Never drive the maintainer's browser.
- Write scratch files only in your session's scratchpad, never `/tmp` or the system temp dir.
  Delete scratch target dirs and scratch images, and leave the tree clean.
- **Budget ~75 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `/usr/bin/grep -l "SPEC-159 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10. Measure at your last turn. You're read-only, so report the numbers; a later session
  writes them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only, no box-drawn tables. First line, exactly one of:
- `✅ APPROVED — merge PR #262 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #262 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–14, one or two lines each, with evidence;
- the punch items, numbered, each with its kind (wording / assert / claim), its evidence, the fix
  you'd expect, and whether it lands on the PR or on `main` (DEC-115);
- optional cleanups, listed separately, each with its kind;
- the mutation-table cells ship should correct, one line each;
- out-of-spec findings, each with the stage it belongs to;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
