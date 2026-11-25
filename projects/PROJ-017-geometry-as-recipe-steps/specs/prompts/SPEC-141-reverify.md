# SPEC-141 — RE-VERIFY prompt: the punch list on the recipe-output guard

Cycle: **verify (re-verify)**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not
design, build or fix SPEC-141. Scope is narrow: did the punch list close its three items without
breaking what verify already passed?

```
just wt verify/spec-141-re -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin test/spec-141-recipe-output-guard:refs/remotes/origin/test/spec-141-recipe-output-guard
git reset --hard origin/test/spec-141-recipe-output-guard
```

Draft PR #233's head was `6b90d71` when this prompt was written. `f6d297b` is the last code
commit, and CI run `41152445637` there is green on all 15 non-skipped legs. Record the SHA you
verify. The maintainer pasted this prompt into a fresh session and will paste your final message
back to the orchestrator, so it must stand on its own.

## What verify found (at `dfaa28b`) and what the punch list claims
Verify's punch items:
1. A missing golden file printed no CI-log dump, so a new recipe's Linux and Windows goldens
   couldn't be blessed from the log.
2. The README didn't name the literals to bump.
3. FT-3 didn't check AC-4's three control lines.

The punch list (`prompts/SPEC-141-punchlist.md` on `main`) claims:
- **Missing golden.** It's one mismatch line naming the file, the run carries on, and all 14 dumps
  print. Driven: exit 101, 14 `BEGIN` blocks, and the moved-aside file equals its dump with the
  markers stripped. FT-9 is new.
  - **Deviation:** the missing recipe's rows also print one "no golden line" line each. FT-9
    loads empty goldens for the other recipes, so its counts don't depend on pixels.
- **FT-3** checks the three control-line formatters, and each goes red on a one-character change.
- **`<N> of 48 rows changed`** counts rows: M3 now reads `42 of 48`, and M1 `36 of 48`.
- **FT-4** reports both rows before failing. Under M1, `png` is red and `webp` green.
- **The README** names `ROWS` (48), `GOLDEN_LINES` (51), `EXPECTED_DIMS` and its `[…; 14]`, the
  `42`, `CORPUS` and its `14`, plus the byte-row and bundled literals. It also gives the CI-log
  bless procedure and the libm-drift rule.
- No `.output` changed, and there's no `src/` change.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Don't fix what you find; report it.

## Check, in order
1. **Scope.**
   - `git diff --stat dfaa28b HEAD` lists only `tests/recipe_corpus_output.rs`, the README and
     the spec.
   - `git diff dfaa28b HEAD -- 'tests/fixtures/recipe_corpus/*.output' src/ Cargo.toml Cargo.lock`
     is empty.
   - Every commit is signed off.
   - `git diff --name-only f6d297b HEAD` lists only the spec.
2. **Punch item 1, driven by you,** from a clean build.
   - Move `bundled_web.linux-x86_64.output` aside. That isn't the build's file, and it's a
     target you aren't on, so it should change nothing on your Mac. Run the test. Report whether
     it stays green.
   - **Then simulate a new recipe:** add a copy of an existing corpus `.toml` under a new stem to
     `RECIPES` and `CORPUS`, with no `.output` files, and bump nothing else. Run the test.
     - It must fail.
     - It must print the new recipe's `BEGIN … END` block for your target.
     - Every literal the README names must show up as its own red, or be named by the failure.
     - **Follow the README's procedure literally:** paste the block into a new `.output`, bump the
       literals the README lists, and run again. It must go green. If you needed a literal the
       README doesn't name, that's the punch item still open.
     - Restore it all, and show `git status --short` empty.
   - **Judge the deviation.** For a missing file, one line names the file plus one line per row.
     Is the failure readable? Does the `N of 48 rows changed` count the missing recipe's rows
     correctly?
   - Read FT-9 whole, and confirm it couldn't pass with the dump suppressed. Re-apply the old
     panic yourself: FT-9 must go red.
3. **Punch items 2 and 3.**
   - Read the README against the test. Grep the test for every numeric literal and array length
     yourself, with `/usr/bin/grep -nE`.
   - Then change one character in each of FT-3's three new formatters, rebuilding each time:
     FT-3 must go red each time. Restore.
4. **The wording and FT-4 changes.**
   - Re-drive **M1** (rebuilt clean): 36 lines, `36 of 48 rows changed`, FT-4 `png` red and `webp`
     green.
   - Re-drive **M3** (re-blessed under the mutation): `42 of 48`. Restore with
     `git checkout -- src/ tests/fixtures/`.
5. **CI.** Read run `41152445637` once on one Linux leg, the Windows leg and the lean leg
   (`gh run view <run> --log --job <job>`; never poll). `recipe_corpus_output` ran 9 tests, or 8
   on lean and webp-lossy, and only the bless is `ignored`. Then run `gh pr checks 233` once at
   the head.
6. **Full suite.** Run the default leg once, in a fresh target dir, with `--no-fail-fast`.
   Redirect stdout and read `$?`.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- Reverting source doesn't rebuild the binary. Rebuild clean after each mutation (a fresh
  `CARGO_TARGET_DIR` or `cargo clean -p crustyimg`).
- zsh does not word-split unquoted parameters. `rtk` can truncate `grep`, so use `/usr/bin/grep`
  for anything you count.
- Delete each scratch target dir when you're done, and leave the tree clean.
- **Budget ~30 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `grep -l "RE-VERIFY prompt: the punch list on the recipe-output guard" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
  `output_tokens`.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10.
- Report the total and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #233 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #233 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–6, one or two lines each, with evidence;
- **the simulated new recipe, on its own lines:** what failed, which literals went red, and
  whether the README's procedure alone took it green;
- anything the punch list broke;
- optional cleanups, listed separately;
- your cost.
