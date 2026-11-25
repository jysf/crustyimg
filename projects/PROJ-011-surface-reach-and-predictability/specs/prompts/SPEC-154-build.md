# SPEC-154 — BUILD prompt: the CI path classifier runs the gates for every build and test input

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**. The design gives
the whole change verbatim: the 227-line script, the diffs to `pages.yml`, `ci.yml` and
`docs/development.md`, 88 self-test rows, the RED baseline's 21 exact FAIL lines and a 32-row
mutation table, **all driven at design on a prototype**. There is no Rust. The work is copying
exactly and driving every row. Be thorough across every row: Sonnet's known weak spot is sweep
thoroughness.

```
just wt feat/spec-154-ci-allow-list -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this
prompt into a fresh session and pastes your final message back to the orchestrator, so that message
must stand on its own. Keep it plain text with short lists; a wide box-drawn table can garble when
copied out of the terminal.

**This spec lands first in STAGE-063.** SPEC-155, SPEC-156 and SPEC-157 build after it merges, so
their PRs run under the fixed classifier.

## What you are building
- **`scripts/ci-code-paths.sh`:** the deny-list replaced by a docs-only allow-list (`DOCS`) plus
  exceptions (`EXCEPT`). A path is code if it matches `EXCEPT` or doesn't match `DOCS`. Blank lines
  are ignored; an empty list prints `true`. New: `include_targets` (every `include_str!` and
  `include_bytes!` target, resolved against the including file's folder), `unresolvable_includes`
  (refuses `concat!`/`env!` paths), `pages_paths`, and a `--self-test` of 88 rows with an include
  floor of 35.
- **`.github/workflows/pages.yml`:** `"recipes/**"` and `"assets/**"` in both `paths:` lists, plus a
  two-line comment.
- **`.github/workflows/ci.yml`:** two comments rewritten. No control-flow change; both echo lines stay
  byte for byte.
- **`docs/development.md`:** a new `## Which PRs skip the Rust gates` section between `## Reports`
  and `## Key discipline in this variant`.

The spec's `## Outputs` is the complete file list: **exactly these 4 files, plus the spec's own Build
Completion and cost entry.** No Rust, no `tests/`, no CHANGELOG (the design followed precedent: #165
and #245 added none), no version bump. Anything else is a deviation; say why.

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- the spec is at `cycle: build`;
- design commit `cbdf66b` is in your history;
- `scripts/ci-code-paths.sh` on `main` still holds the `CODE_PATHS` one-liner the spec replaces;
- the include count the floor depends on: on `main`, after you paste in `include_targets`, run
  `include_targets | /usr/bin/grep -c .` and compare with 35. If another merge moved it, stop and
  report the new count rather than edit the floor on your own.

If any check fails, stop and report.

⚠ **Other open work touches nearby lines.** The spec's "Overlap with other open work" lists it from
the PRs' file lists: #240 and #237 both touch `ci.yml` in other hunks. After every rebase, run
`gh pr list --state open --json number,title,files` and compare; name any new overlap in Build
Completion. If #240 or #237 merged, re-run the self-test after the rebase. If a rebase conflicts in a
file outside the spec's list, stop and report.

## Read in order, before any change
1. **The spec:**
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-154-ci-path-classifier-runs-the-gates-for-every-build-and-test-input.md`.
   Read it in full: `## Frame` with the maintainer rulings, `## Design` (especially "Design
   corrections to the frame"), every AC, the Failing Tests, the baseline, the mutation table, and
   `## Implementation Context` (the script verbatim, the other three files verbatim, every message,
   the comments, the verification list, ship's step, the overlap) and `## Notes for the Implementer`.
   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in your
   report; don't quietly re-decide it.
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this wave
   paid for" (traps 10, 12, 13 and 14). Also `/guidance/constraints.yaml`.
3. `scripts/ci-code-paths.sh`, `.github/workflows/ci.yml` and `pages.yml` as they are on `main`.

## Maintainer rulings (settled — do not re-open)
1. The docs-only allow-list, not a fixed deny-list.
2. `README.md` is a code path.
3. `docs/development.md` gets the section, verbatim from the spec.
4. No message, comment or doc line names a spec, stage or DEC id.

## Order, with a pushed checkpoint after each stage
A long build with no pushed commit has been lost before. Commit with `git commit -s` and push after
each numbered stage.
1. **The RED baseline.** Add the new self-test rows, the derived include checks and the pages checks,
   with `classify` still today's `CODE_PATHS` one-liner and `pages.yml` unchanged except for the two
   new entries. Run `bash scripts/ci-code-paths.sh --self-test > out.txt 2>&1; echo rc=$?` (redirect,
   don't pipe). Expected: rc=1, last line
   `ci-code-paths self-test: 88 rows, 12 mismatched, 0 crashed, 0 junk, 9 failed`, and **exactly** the
   spec's 21 FAIL lines. Diff your FAIL lines against the spec's list and quote both the match and any
   difference. Then the second baseline the spec names (new `classify`, `pages.yml` without the two
   entries): rc=1 with exactly the two `pages.yml lists …` lines. Commit, push, and open the PR **as a
   draft** with this commit.
2. **The final script and the other three files,** copied verbatim from the spec. Diff your script
   against the spec's text byte for byte (extract the spec's block to a scratch file and `diff` it);
   quote "no differences" or each difference with its reason. Self-test: rc=0, 88 rows. Then the
   spot verdicts through stdin (the spec's verification step 4), the YAML load (step 6), the id grep
   (step 7) and `git diff --name-only origin/main` (step 8, exactly 4 files). Commit and push.
3. **Mutations M1 to M32, one at a time** (HANDOFF traps 12 and 14), in a scratch copy of the script
   in your session's scratchpad, never in the build worktree. The spec quotes each mutant's exact
   edit: **apply that edit, not your own version of it.** For each row:
   - apply it, and check the diff against the green script is non-empty;
   - **M20 edits the `DOCS=` line, not the comment above it** that holds the same text (the design's
     first attempt edited the comment and stayed green);
   - M26 to M28 add a fake include to a Rust file: do it in a scratch copy of the tree, or revert
     `src/lib.rs` after each and show `git status` clean;
   - run the full self-test, redirected, read `$?`, and read the red set **off that run**. Compare it
     with the table, line by line, and call out every FAIL line the table doesn't list and every
     listed line that didn't appear;
   - restore, and show the green run.
   Paste each mutation's FAIL lines into Build Completion. Where a cell is wrong, correct it in Build
   Completion and say so; don't edit the design's table. Commit Build Completion and push.

## The things most likely to go wrong
- **BSD and GNU tools.** The design ran only under macOS tools (BSD grep and awk, bash 3.2); Docker
  was down. CI's `changes` job runs on Linux (GNU grep, mawk or gawk, bash 5). **The PR's own `changes`
  job log is the Linux run of record.** Quote its self-test line from the job log. If it isn't
  `88 rows, 0 mismatched, 0 crashed, 0 junk, 0 failed` with rc 0, stop and report the difference.
  Don't patch around a GNU/BSD difference silently. If you can run the self-test under GNU tools
  locally (for example, `brew`'s `grep` and `gawk` put first on `PATH` in a subshell), do it and say
  how. A Linux container also works if Docker is up.
- **Retyped text.** Copy every pattern, message and comment from the spec byte for byte. The
  canonical-path test keeps its `?` (`(^|/)\.\.?(/|$)`), and `[ -n "$t" ] || continue` stays.
- **The floor.** `INCLUDE_FLOOR=35` is a count of today's tree. Re-derive it at Step 0, not from
  memory.
- **A green self-test that tests nothing** (trap 10). M22 (empty scan) and M23 (no `..` collapse) are
  the rows that prove the derived checks bite; drive both.
- **The build PR can't prove the classification.** It touches `scripts/` and `.github/`, which are
  code under both designs. That proof is ship's draft PR after merge, not yours.

## Guardrails
- No version bump, no release, no tag. No change to `Cargo.toml`, `Cargo.lock`, any Rust file, any
  test, or the CHANGELOG.
- Don't edit the spec's timeline: it lives on `main`, and the orchestrator keeps it.
- **Every commit uses `git commit -s`**, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails a PR with an
  unsigned commit.
- **Never poll CI and never write your own wait loop** (AGENTS §16 habit 6). Run
  `gh pr checks <PR> --watch` once, in the background, after your last push, and take one snapshot
  when it exits. If it exits with legs still pending, say so; don't call that green.
- **Rebase onto `main` before each push** and redo Step 0's overlap check after each rebase. Don't
  chase `main` beyond that.
- A piped command reports the **pipe's** exit code, so redirect and read `$?`. zsh does not
  word-split unquoted parameters, so loop with `while IFS= read -r`, and it reads `$R:r…` as a
  modifier, so brace it (`${R}:`). macOS has no `timeout(1)`. `rtk` can truncate `grep` and `git log`
  output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Write scratch files in your session's scratchpad, never `/tmp` or the system temp dir, and don't
  leave PR-body files behind.
- **Budget ~45 exchanges.** If you're past budget with the self-test green, stop and report what's
  left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory, each with a phrase on
     what changed; give the count and say whether it includes the spec file;
   - both RED baselines, quoted, against the spec's lists;
   - the byte diff of the script against the spec's text;
   - every AC, one line each: met, or not and why (AC 18 is ship's; say so);
   - every mutation row, with the edit you applied, the FAIL lines, against the spec's table, with
     every mismatch called out;
   - the spot verdicts, the YAML load, the id grep;
   - the Linux self-test line from the PR's `changes` job, quoted, and the CI legs;
   - the overlap check after your last rebase;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost** at your **last** turn, after the last push.
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `/usr/bin/grep -l "SPEC-154 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`. Include any `subagents/`
     transcripts.
   - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per MTok,
     Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read is ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-154 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks it
     ready. Never mark it ready yourself.
   - Title: `ci: run the gates for every path not provably docs-only (SPEC-154)`.
   - Body per AGENTS §13:
     - Summary: today a PR touching only `recipes/web.toml` (or a font, `clippy.toml`, `README.md`, a
       docs file a test reads) shows the required checks green with nothing built or tested; after
       this, anything not provably docs-only runs the gates, and the self-test fails if a build or
       test includes a file the list calls docs-only;
     - Spec metadata;
     - Decisions referenced: none; New decisions: none;
     - Constraints checked;
     - Breaking: none (README-only PRs now run the full gates);
     - a note that this PR can't prove the classification on Actions, and the post-merge draft-PR
       check ship runs;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-154 build: draft PR #NNN at <head SHA> — CI <green | red: legs | pending>`.
Then, briefly, as plain lists:
- both RED baselines against the spec's lists;
- the script's byte diff against the spec;
- AC completion (as a count, plus any row not met);
- each mutation's FAIL lines against the table, the edit you applied where it differs from the
  quoted one, and every cell that turned out wrong;
- the Linux self-test line from the PR's `changes` job, and the CI legs;
- the overlap check after your last rebase;
- files changed against the spec's `## Outputs`;
- any design call that turned out wrong, and any out-of-spec finding;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its
  own line.
