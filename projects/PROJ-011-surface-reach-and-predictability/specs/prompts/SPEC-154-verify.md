# SPEC-154 — VERIFY prompt: the CI path classifier runs the gates for every build and test input

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not frame,
design or build SPEC-154. Your job is to find what the build's own evidence can't show. A green
self-test and 17 ticked ACs are claims until you drive them, and a classifier's self-test can only
check the rows someone thought to write.

```
just wt verify/spec-154 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-154-ci-allow-list:refs/remotes/origin/feat/spec-154-ci-allow-list
git reset --hard origin/feat/spec-154-ci-allow-list
```

When this was written, draft PR #266's head was `2a70156`, force-pushed after a rebase. CI had been
green on every leg at the previous head `e20126b`. At `2a70156`, the orchestrator saw it part-way:
`detect changed paths`, macOS, lean, msrv, pages, parity, wasm, cargo-deny, DCO and front-matter
green, and the rest in progress. Record the SHA you verify. The maintainer will paste your final
message back to the orchestrator, so it must stand on its own. **Write it as plain lists, not
box-drawn tables.**

## What the build claims
- **Baselines:** the first is rc=1, `88 rows, 12 mismatched, 0 crashed, 0 junk, 9 failed`, with
  exactly the spec's 21 FAIL lines. The second is rc=1 with only the two `pages.yml lists …` lines.
- **Script:** byte-identical to the spec's block, before and after the rebase. The other three files
  were applied from the spec's diff.
- **ACs:** 17 met. AC-18 is ship's post-merge draft PR.
- **Mutations:** M1–M32 each run alone in a scratch copy; every red set matched the table, and no
  cell was wrong.
- **Linux:** the PR's `changes` job on ubuntu-latest printed
  `88 rows, 0 mismatched, 0 crashed, 0 junk, 0 failed`, then `-> code paths touched: running the
  full gates`. The design and the build had no GNU tools locally.
- **Deviation:** the spec's id grep (AC-15, verification step 7) finds two hits, at
  `scripts/ci-code-paths.sh:161-162`. They are the spec's own fixture paths (`…/SPEC-142-x.md`,
  `decisions/DEC-001-x.md`), kept verbatim. The orchestrator's view is that the grep is over-broad
  and the fixtures are fine. Confirm or dispute it.
- **Cost:** $1.37 over 23 messages. That's low for 32 driven mutations, so check that Build
  Completion holds each mutation's FAIL lines, not just a summary.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes, no new PR. Report; don't fix. Any mutation you
drive happens in a scratch copy and is gone before you finish.

## Read first
1. The spec on the PR branch, in full:
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-154-ci-path-classifier-runs-the-gates-for-every-build-and-test-input.md`.
   Read these sections:
   - `## Frame` with the maintainer rulings;
   - `## Design` and its corrections;
   - the ACs, the self-test rows, both baselines and the mutation table;
   - `## Implementation Context`: the script, the three diffs, the messages, ship's step, the
     overlap;
   - Build Completion.
2. `scripts/ci-code-paths.sh`, `.github/workflows/ci.yml` and `pages.yml`, both on the PR and on
   `main`.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for", especially traps 10, 12 and 13.
4. AGENTS §15's rules: a bound or "can't happen" is a claim until its worst input is driven, and
   every finding is classified.

## Maintainer rulings the build had to honour
1. The docs-only allow-list: anything not provably docs-only runs the gates.
2. `README.md` is a code path.
3. `docs/development.md` gains the section, verbatim.
4. No message, comment or doc line names a spec, stage or DEC id.

## Check, in order
1. **Scope.**
   - Compare `git diff --name-only origin/main...HEAD` with `## Outputs`: 4 files plus the spec.
   - Confirm that no Rust file, test, `Cargo.*` or `CHANGELOG.md` changed.
   - In `ci.yml`, confirm the diff is comments only. Diff the non-comment lines of both versions, and
     show that both `echo` lines and every `if:`/`needs:` are byte-identical.
   - Check that every commit is signed off, that the PR merges cleanly onto current `main`, and which
     open PRs touch `ci.yml` (#240, #237).
2. **The script against the spec, independently.** Extract the spec's script block yourself and
   `diff` it with the PR's file. Do the same for the `pages.yml` and `docs/development.md` text.
3. **Every tracked path, both scripts.** Run `git ls-files` through `main`'s script and the PR's,
   then list:
   - every path the PR calls docs-only (`false`);
   - every path whose verdict changed, in either direction.

   A path that `main` called code and the PR calls docs-only is a regression, and the design claims
   there are 0. Give the counts with the command next to each.
4. **Is every docs-only path really unread?** This is the check the self-test can't make. For every
   path or prefix the PR calls docs-only, search for anything a gated job reads:
   - Rust sources and tests: `include_*!`, `fs::read*`, `File::open`, `read_to_string`, and path
     literals naming a docs-only root;
   - scripts, `justfile` recipes and Node/Python test files that a **gated** job runs. Read each
     gated job in `ci.yml` and `pages.yml`, list the commands it runs, and follow each one: for
     example `just` recipes, `scripts/*.sh`, `tests/*.mjs` and the demo build.

   Any docs-only path a gated job reads is a finding. Paste the commands. The ungated jobs
   (front-matter, cost-capture, DCO, cargo-deny) read `projects/` and `decisions/` on every PR
   anyway; say which jobs those are and confirm they don't depend on `code`.
5. **How the file list reaches the script.** Read the `changes` job's step that builds the list:
   - which `git diff` it runs, the base it diffs against for a PR, for a push to `main`, and for a
     branch's first push (an all-zero `before`);
   - whether it uses rename detection.

   Then drive these on a scratch repo or by hand through stdin, and say what each classifies as:
   - **a rename** of a code file into `docs/` (does the list carry the old path, or only the new
     one?);
   - **a deletion** of a code file alone;
   - **a path with a space**;
   - **a push of several commits** where only an earlier one touches code.

   A change that touches code but classifies as docs-only is a finding. Say whether `main`'s script
   had the same hole (pre-existing) or this PR made it.
6. **Linux.** Read the `detect changed paths` job log at the head you verify with
   `gh run view --log` (one read, no polling). Quote the self-test summary line and the verdict line.
   If the head has no completed run, say so and quote the last completed one with its SHA.
7. **The include check bites.** Drive these mutations yourself in a scratch copy, using the spec's
   exact edits, and compare the FAIL lines with the table and with Build Completion:
   - M20, edited on the `DOCS=` line;
   - M22, an empty scan;
   - M23, no `..` collapse;
   - M27, a fake include of `docs/development.md`;
   - M30, `docs/**` added to `pages.yml`;
   - M32, the whole-self-test negative control.

   Then drive one row the table doesn't have: a test that **reads** a docs-only file at runtime
   (`std::fs::read_to_string("docs/development.md")` in a scratch test). The design says runtime
   reads can't be derived, so this should stay green. Confirm it, and say whether the docs paragraph
   and the script header say so honestly.
8. **Every message and the docs paragraph, driven.**
   - Check each self-test message against the spec's list with a script, not by eye.
   - For each sentence of the `docs/development.md` section and the script header that states
     behaviour, drive it through stdin and quote it. That includes "a new top-level directory runs
     the gates", the three exceptions, and "an empty list prints true".
   - Run `/usr/bin/grep -nE '(SPEC|STAGE|DEC|PROJ)-[0-9]'` over the four files and classify each
     hit: user-facing text, or test data.
9. **The floor.** Re-derive the include count on the PR head (`include_targets | /usr/bin/grep -c .`)
   and compare it with `INCLUDE_FLOOR=35`. Say what happens to the next PR that removes one include
   on purpose: the message it gets, and whether that's reasonable.
10. **Ship's step.** Read the post-merge draft-PR check as the spec writes it. Say whether it can
    really tell the fix from no fix: for example, whether a `recipes/web.toml`-only PR runs `pages` and
    all three `cargo test` legs under the new script, and none of them under `main`'s. Drive
    `printf 'recipes/web.toml\n'` through both scripts.
11. **CI.** Run `gh pr checks 266` once at the head, and say whether every leg is green, including
    Windows.

## Classify every finding (AGENTS §15)
For each punch item and optional cleanup, say which kind it is:
- **wording:** a doc, comment or spec line that misstates something the script already does;
- **assert:** a missing or loose self-test row, where the behaviour is already right;
- **claim:** a fix that would change what a doc or test claims is true, or the classifier's
  behaviour.

Only a **claim** needs a re-verify after its punch list.

For every bound, count or "can't happen" in the spec, say whether it was driven, by whom, and over
what range. Examples:
- "0 paths move from code to docs-only";
- the 12-commit replay;
- `INCLUDE_FLOOR=35`;
- "runtime reads can't be derived";
- "the build PR can't prove the classification".

## Guardrails
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters, so loop with `while IFS= read -r`. macOS has no `timeout(1)`.
  `rtk` can truncate `grep` and `git log`, so use `/usr/bin/grep` and `/usr/bin/git` for anything
  you count.
- No `cargo` build is needed unless an item asks for one. If you build, use a fresh
  `CARGO_TARGET_DIR` that no other session shares.
- Never poll CI and never write your own wait loop (AGENTS §16 habit 6).
- Write scratch files only in your session's scratchpad, never `/tmp` or the system temp dir.
  Leave the tree clean.
- **Budget ~45 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `/usr/bin/grep -l "SPEC-154 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10. Measure at your last turn. You're read-only, so report the numbers; a later session writes
  them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only, no box-drawn tables. The first line is exactly one of:
- `✅ APPROVED — merge PR #266 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #266 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–11, one or two lines each, with evidence;
- the punch items, numbered. Give each its kind (wording / assert / claim), its evidence, the fix
  you'd expect, and whether it lands on the PR or on `main`;
- optional cleanups, listed separately, each with its kind;
- the id-grep deviation: your ruling and why;
- out-of-spec findings, each with the stage it belongs to (STAGE-063 or `guidance/ideas.yaml`), and
  whether each one is pre-existing;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
