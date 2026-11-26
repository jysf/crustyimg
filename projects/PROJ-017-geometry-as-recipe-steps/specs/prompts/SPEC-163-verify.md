# SPEC-163 — VERIFY prompt: an aspect that follows each image's orientation

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not frame,
design or build SPEC-163. Your job is to find what the build's own evidence can't show. CI green and
18 ticked ACs are claims until you drive them.

```
just wt verify/spec-163 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-163-aspect-follows-orientation:refs/remotes/origin/feat/spec-163-aspect-follows-orientation
git reset --hard origin/feat/spec-163-aspect-follows-orientation
```

Draft PR #272's head was `fccb368` when this was written. CI was green there on every leg, as the
orchestrator checked: 18 passed, 8 release jobs skipped. Record the SHA you verify. The maintainer
pastes your final message back to the orchestrator, so it must stand on its own. **Write it as plain
lists, not box-drawn tables.**

⛔ **Don't merge PR #272 until this verify has reported.**

## What the build claims
- **ACs:** 18 of 18 met.
- **Red baseline on `main`:**
  - in the new suite, FT-4 passed (it checks the test's own table) and 9 failed, each with
    `unknown key 'orientation'`;
  - `tests/crop.rs` had 1 red and `tests/pad.rs` had 2;
  - FT-1 and the registry tests didn't compile.
- **Behaviour sentences:** all eleven were driven on one input and a folder batch. The list is in
  Build Completion.
- **Parity:** `just ddphotos-parity`, both modes. The summary line reads `16 rows, 0 row failures,
  0 table failures`: 16 is the old TABLE, and the 26 orientation rows are counted separately by
  `ORIENTATION_ROW_COUNT`. `PASS N3` is exactly the eight rows. No reasoned size was wrong.
- **Mutations:** M0 to M17 were each driven with a clean rebuild and the full suite, and every red set
  equals the table. M16's lib unit tests compile. Only M5 was run against the parity rows, and that
  run was native-only against an unmutated wasm package. **The other parity cells are still
  reasoned.**
- **Deviations:** FT-1 has one extra row, `(1,1,48,48)`. Clippy's `type_complexity` made the build let
  the compiler infer the row tables' types. Build Completion also says the build prompt isn't on
  `main`; that's wrong (it is, at `eaf76c3`), and it doesn't matter.
- **Matrix:** default, lean and `webp-lossy` all pass, as do clippy and fmt. `just wasm-check` and
  `just wasm-test` pass (56 passed).
- **Files:** 22, exactly the spec's `## Outputs`.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes, no CI reruns. Report; don't fix. Run any mutation
in a scratch `git worktree` with its own `CARGO_TARGET_DIR` under your scratchpad, and restore it
before you finish.

## Read first
1. The spec on the PR branch, in full:
   `projects/PROJ-017-geometry-as-recipe-steps/specs/SPEC-163-an-aspect-that-follows-each-image-s-orientation.md`.
   Read these sections:
   - `## Frame`, with the maintainer rulings;
   - `## Design`: the corrections, the code shape, "Every message, with its exit code", the replay
     rows, "What doesn't change", the fixtures, the parity rows, the docs, the behaviour sentences,
     DEC-112's rule 8 and the page port;
   - the ACs, the Failing Tests, the mutation table and Build Completion.
2. DEC-112, DEC-113, DEC-114 and DEC-106 on `main`.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for", especially traps 5, 10, 12, 13 and 14.
4. AGENTS §15's rules: a bound or "can't happen" is a claim until its worst input is driven, and
   every finding is classified.

## Maintainer rulings the build had to honour
1. The key is `orientation = "match"`; absent means off.
2. A square takes the aspect as written; `1:1` with the key changes nothing. Landscape means strictly
   wider than tall.
3. `--crop-orientation` and `--pad-orientation` are in this spec.
4. DEC-112's rule 8 lands at ship, not on this PR.
5. No message, help line or comment names a spec, stage or DEC id.

## Check, in order
1. **Scope.**
   - Compare `git diff --name-only origin/main...HEAD` with `## Outputs`. Give the real count, and
     name every file outside the list.
   - Confirm these are unchanged: `Cargo.toml`, `Cargo.lock`, `CACHE_SCHEMA_VERSION`, `src/wasm.rs`,
     `scripts/wasm-ci-check.sh`, `tests/wasm_roundtrip.rs`, `tests/orientation.rs` and DEC-112.
   - Check every commit is signed off. Check the PR merges cleanly onto current `main`.
   - Run `git diff --name-status origin/main...HEAD -- tests/fixtures/`. It should show only `A`
     lines plus `M tests/fixtures/ddphotos/README.md`.
   - Run `gh pr list --state open --json number,files`. The build named only #237 (`CHANGELOG.md`).
2. **The rule, against an independent model.**
   - Read `follow_orientation` and both call sites. Say what size each call site passes in: the
     buffer the step receives, as `u32`.
   - Write your own model in Python from the rulings (not from the code). Drive the binary over a
     grid big enough to reach the edges:
     - every w, h from 1 to 12;
     - near-squares up to 2000 (w = h ± 1);
     - extreme shapes (1×1000, 1000×1);
     - aspects 1:1, 3:2, 2:3, 16:9, 9:16, 1:1000 and 7:5;
     - crop and pad, key on and off.
   - Make the inputs with Python or ImageMagick, never with crustyimg. Compare the binary's output
     sizes with your model and with `main`'s binary when the key is off. Say how many rows you drove
     and give every mismatch.
   - Say what happens when the aspect rounds a side to 0 or a 1-pixel image meets a far aspect, and
     whether that is the same with the key on as off.
3. **Refusals and their order.**
   - Extract every line from the spec's "Every message, with its exit code". Extract the same lines
     from the code and the tests, and diff them with a script, not by eye.
   - Drive each line on a release build of this branch (a fresh `CARGO_TARGET_DIR`, never a binary on
     `PATH`). Compare stderr with `==`, and check the exit code and that nothing was written.
   - Drive the overlaps:
     - a box crop with `gravity` and `orientation`, and a border pad with both: gravity's message
       wins;
     - a box crop with `orientation = true`: the type message;
     - a crop with only the key;
     - `edit` with `--crop-gravity` and `--crop-orientation` on a pixel crop: the gravity flag's
       message wins.
   - Add rows the tests don't have: `orientation = ""`, `orientation = "match "`, `orientation =
     ["match"]` and the key written twice.
   - Run `/usr/bin/grep` over every new user-facing string and comment for `SPEC-`, `STAGE-`, `DEC-`
     and `PROJ-`.
4. **AC-5: the key reads the buffer the step receives.**
   - Find the R input the parity rows use. Read its Orientation tag with ImageMagick or `exiftool`.
   - Drive `crop` and `pad` with the key, with and without an `auto-orient` step before them, on the
     binary. Read the output sizes with ImageMagick.
   - Do the same through `edit`, which auto-orients by default. Say whether `edit` and the matching
     saved recipe agree.
5. **`edit`, `--save-recipe` and replay.**
   - Drive both flags with and without `--save-recipe`. Compare the saved text with the spec's
     literals byte for byte. The keys must be sorted.
   - Replay each saved recipe with `apply` on landscape, portrait and square inputs, and compare
     pixels with ImageMagick `compare -metric AE`. Expect 0.
   - Drive a folder `edit --save-recipe` with the aspect crop and the key: one recipe, each image
     turned by its own shape.
   - Drive `--crop-orientation` without `--crop`, and `--pad-orientation` without `--pad`: clap,
     exit 2.
   - Check `edit --help` for both lines.
6. **The fixtures.**
   - Re-run the spec's three ImageMagick commands in your scratchpad. Compare the sizes with the
     committed files, and the bytes and SHA-256 with the README's table. Say which ImageMagick
     version you used: a different version may give different bytes, which is fine if the README
     says so.
   - Confirm that no fixture came from crustyimg.
7. **The parity rows, driven past what the build did.**
   - Run `just ddphotos-parity` locally once. Confirm the 26 orientation rows ran: show the `O-count`
     check, or count the rows in the log.
   - Read CI's ddphotos-parity job log at the head once, and say whether it ran the orientation rows
     and N3.
   - **Drive one wasm-side mutant** the build didn't: M1 (`w > h` → `w >= h` in
     `follow_orientation`), built into **both** the binary and the wasm package. The table reasons
     `O-crop-S-on-9:16` and `O-pad-S-on-9:16` red, but parity compares wasm with native, so when both
     sides carry the mutant only the literal sizes can catch it. Say which rows go red and why.
   - Then drive M1 into the wasm package only, with native unmutated, and say which rows go red.
   - Restore both sides, and show the parity run green.
8. **Mutations, driven by you.** Don't re-run all 18. Drive these, each with a clean rebuild, the
   **exact edit the spec quotes** and the full suite with `--no-fail-fast`. Compare each red set with
   the table and with the build's account.
   - **M1** (square as landscape): only the 9:16 square rows red? Say whether FT-13 really can't
     catch it, and whether a square row written `9:16` in FT-13 would.
   - **M5** (negative control): the rule's rows red and the refusal rows green.
   - **M14** (one-pixel tolerance): the near-square rows red.
   - **M15** (square as portrait).
   - **M9** (the key always saved).
   - **M16 (trap 14):** restore `main`'s `src/operation/` and `src/cli/`. Do the lib unit tests
     compile? Is FT-4 the only new test that stays green?
   - Then confirm the tree is restored: `git status` clean and the suite green.
9. **Docs (AC-16) and CHANGELOG, each sentence driven.**
   - Read every row of the spec's docs list on the PR: `docs/recipes.md`, `recipes/README.md`,
     `docs/data-model.md`, `docs/api-contract.md`, `docs/cli-reference.md`, `skills/crustyimg/SKILL.md`,
     `edit --help` and `CHANGELOG.md`.
   - For **each sentence that states behaviour**, quote it and drive it on the binary, or say which
     earlier item drove it. Pay attention to:
     - every exit-code row (1 and 2) in `api-contract.md` and `cli-reference.md`;
     - the square and near-square sentences;
     - any sentence about `auto-orient` or EXIF.
   - Undriven doc sentences have been the likeliest defect in this project's recent specs, so treat
     them as the likeliest place for one here.
   - CHANGELOG: a user should understand each of the two lines without the spec. Check they are
     verbatim.
10. **DEC-112's rule 8 and the page port.**
    - Read the rule-8 text in the spec. For each sentence, point to the code that does it, and say
      whether ship can apply it as written. Run `just decisions-audit` and quote the result.
    - Drive the page port's four rows on the binary: a 3:2 crop with the key on 1600×1071, 1071×1600,
      1071×1071 and 1071×1072. Compare with the spec's 1600×1067, 1067×1600, 1071×714 and 715×1072.
      SPEC-161 will copy these.
11. **Cache.** Confirm that a recipe without the key gives the same canonical text and cache key as on
    `main`. Drive one `build` or cache-key comparison, and say whether `recipe_round_trip` and the 22
    corpus goldens are unchanged.
12. **Full suite.** Run the default leg once and the lean leg (`--no-default-features`) once, each in
    a fresh target dir with `--no-fail-fast`. Redirect stdout and read `$?`. Show the
    `tests/aspect_orientation.rs` counts in both legs.
13. **CI.** Run `gh pr checks 272` once at the head. Say whether every leg is green.

## Classify every finding (AGENTS §15)
For each punch item and optional cleanup, say which kind it is:
- **wording:** a doc, comment or table cell that misstates something the code already does;
- **assert:** a missing or loose assertion, where the behaviour is already right;
- **claim:** a fix that would change what a doc, DEC or test claims is true, or the code's behaviour.

Only a **claim** needs a re-verify after its punch list.

For every bound, count or "can't happen" in the spec, say whether it was driven, by whom and over what
range. Examples:
- "26 sizes match an independent model";
- "aspect_size / aspect_canvas stay byte for byte";
- "a recipe with the key could never have been cached";
- "FT-13 cannot catch M1";
- "the parity cells are as reasoned".

## Guardrails
- In an interactive terminal `cargo test` fails `display_sink_refuses_non_tty`, so redirect stdout to
  a file.
- A piped command reports the pipe's exit code, so redirect output and read `$?`.
- Reverting source doesn't rebuild the binary, so rebuild clean after each mutation.
- zsh doesn't word-split unquoted parameters, and it reads `$R:r…` as a modifier, so brace it
  (`${R}:`).
- macOS has no `timeout(1)`.
- `rtk` can truncate `grep` and `git log`, so use `/usr/bin/grep` and `/usr/bin/git` for anything you
  count.
- Never share a `CARGO_TARGET_DIR` with another session.
- **Never poll CI and never write your own wait loop** (AGENTS §16 habit 6).
- Never drive the maintainer's browser.
- Write scratch files only in your session's scratchpad, never in `/tmp` or the system temp dir.
  Delete the scratch target dirs, worktrees and images, and say how much space that freed (measure
  before you delete). Leave the tree clean.
- **Budget ~80 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. The `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `/usr/bin/grep -l "SPEC-163 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
- Price each component at Opus: $5 per MTok input and $25 output, with cache_creation at ×1.25 and
  cache_read at ×0.10. Measure at your last turn. You're read-only, so report the numbers; ship
  writes them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only, no box-drawn tables. The first line is exactly one of:
- `✅ APPROVED — merge PR #272 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #272 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–13, one or two lines each, with the evidence;
- the punch items, numbered. Give each its kind (wording / assert / claim), its evidence, the fix
  you'd expect, and whether it lands on the PR or on `main`;
- optional cleanups, listed separately, each with its kind;
- the mutation-table cells ship should correct, one line each;
- DEC-112's rule 8: apply as written, or the change it needs;
- out-of-spec findings, each with where it belongs (a stage, or `guidance/ideas.yaml`);
- the space freed;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
