# SPEC-161 — PUNCH LIST prompt: DEC-118's wording, one comment, and three new checks

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. You did not design,
build or verify SPEC-161. Verify found the code correct everywhere it drove it. What's left is
wording in DEC-118, the spec and one comment, plus three checks the harness lacks. No item changes
what the code does, so no re-verify follows.

```
just wt fix/spec-161-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-161-page-crop-model:refs/remotes/origin/feat/spec-161-page-crop-model
git reset --hard origin/feat/spec-161-page-crop-model
```
Push to the PR branch with `git push origin HEAD:feat/spec-161-page-crop-model`. Draft PR #273's
head was `f9c81ae` when this was written. If it has moved, stop and report. If a push is refused,
stop and report the command; don't work around it.

The maintainer will paste your final message back to the orchestrator, so it must stand on its
own. **Write it as plain lists, not box-drawn tables.**

## What to fix (settled; don't re-decide)
Each item says its kind: **wording** (text only) or **assert** (a new check; behaviour unchanged).

1. **wording — DEC-118 rule 4's shift sentence.** It says "A one-copy-pixel framing shift reads at
   most 51.71 dB on an aspect row and 52.77 dB on a box row, below both floors." That holds only for
   a shift to the left. Driven right, R1 reads 54.66 dB and R9 54.05 dB, both above the 53.5 floor,
   because a right shift cancels an aspect's half-pixel centring offset. Say "a shift to the left",
   add one sentence on the right shift and those two numbers, and say that aspect placement is
   pinned by the `placement` check instead.
2. **wording — DEC-118 rule 1.**
   - "with the original made from the copy by `resize` mode `max` to 1600 px" is backwards. It should
     say the copy is made from the original that way.
   - "1 px at any ratio" goes past what was driven. Say "at the ratios driven (1:1, 4:5, 3:2 and 16:9,
     in both orders)", and that portrait originals are assumed to mirror landscape, which no one
     drove.
3. **wording — a DEC id in a new comment.** `tests/ddphotos_parity.mjs:~207` says "A row that
   resizes is within DEC-110." Make it "A row that resizes is within the resize tolerance." Leave
   `main`'s existing mentions at `:37` and `:43` alone.
4. **assert — `planEdit`'s `Math.max(1, …)` has no guard.** If you drop both clamps (`recipe.js:135–136`)
   the run stays green. Add a pure `model-min` check with literals:
   - `planEdit` on W (1600×1071) with the box (93, 131, 1460, 1) at size 400 predicts 400×1;
   - one more case of your choice where the short side would round to 0, on a different input.

   Work out each literal by hand or with your own short Python model of `resize` mode `max`
   (`src/operation/mod.rs:999–1009`), never by running `recipe.js`. Then drop both clamps in a
   scratch copy and show `model-min` go red, and only `model-min`.
5. **assert — an R row where the stored-size bug shows as a wrong size or mark, not a throw.**
   Under mutant M4 (`loadPhoto` reads `wasm.info(source)`, the stored 1600×1071), the R rows that
   catch it with a watermark (C27, C30, R12, R15) fail only because the engine refuses an
   out-of-range fraction. That's a failure for the wrong reason (HANDOFF trap 13).
   - Add one crop row, **C36**: R, a placed box that stays inside 1071×1071 from the top-left, so
     its fractions stay ≤ 1 under both the upright and the stored size, with the watermark on and a
     size that resizes.
   - Take its size and mark literals from your own model, never from `recipe.js`.
   - Under M4 (the spec's exact edit), C36 must go red on its size or its mark, **not** as "threw".
     Quote its failure message. If no such box exists, stop and report why rather than add a
     weaker row.
   - Adding C36 moves `crop-count` to 36. If C36 resizes, it isn't in `CROP_NO_RESAMPLE_ROWS`.
     Re-run N1, N2 and N4. If C36 joins a control's red list, add it at the end of that list, and
     say why.
6. **assert — the summary line counts what ran.** Today it prints `ORIENTATION_ROW_COUNT` and the
   alpha count as constants. Make both numbers count the rows that actually ran, and add an
   `A-count` check for the alpha rows, the way `O-count` backs the orientation rows. Don't change
   what the orientation or alpha rows check, or N3. Keep the diff in that code to recording a count.
7. **wording — the spec.**
   - Tick AC-1 to AC-13 (`- [x]`). Each was met at build and confirmed by verify.
   - The three build-phase reflection answers are still `— <answer>`. You didn't build, so don't
     write them as if you had. Write each as `— (filled at punch list from Build Completion and
     verify)` followed by a short, factual answer taken from those records. For example, Q1: the
     design's M4 and N4 cells, which turned out wrong in their reasons.

## Not yours
Ship handles these on `main` after the merge. Don't edit them:
- the mutation-table and control cells verify listed (M4, N4, M5, and the two new mutants);
- AC-12's wasm-size and TABLE `.map` wording;
- the spec's other row counts (35 → 36 in its prose);
- `guidance/ideas.yaml` and the STAGE-060 file.

Stay in these files: `decisions/DEC-118-…md`, `tests/ddphotos_parity.mjs`, and the SPEC-161 spec
(the AC ticks, the reflection, Build Completion and `cost.sessions` only). `git diff --stat f9c81ae`
must show nothing else. `recipe.js`, `worker.js` and `tests/ddphotos_upscale.py` stay unchanged.

## Temp folders (set up before the first harness run)
The harness keeps its `mkdtempSync(tmpdir())` folders whenever a run fails, so every red mutant run
leaves one.
- Before the first run, count the system temp folder's `ddphotos-*` folders and their size. Don't
  delete any of them: some belong to other sessions.
- Run every harness command with `TMPDIR` set to a folder in your scratchpad. After the first run,
  confirm the folders landed there; stop if they didn't.
- Empty that folder after you've read each red run.
- At the end, count the system temp folder's `ddphotos-*` again and report both counts.

## Check it
- `just ddphotos-parity`, both modes, with your own `CARGO_TARGET_DIR` and stdout redirected, reading
  `$?`. Both must exit 0. Quote both summary lines: the crop count reads 36, and the orientation
  and alpha numbers come from rows that ran.
- Show the three red runs: dropped clamps (item 4), M4 on C36 (item 5), and an alpha row removed
  (item 6's `A-count` goes red). Restore after each and show green.
- `just decisions-audit`: 0 structural errors.
- `/usr/bin/grep -n 'SPEC-\|STAGE-\|DEC-\|PROJ-'` on your added lines in `tests/ddphotos_parity.mjs`
  must find nothing.
- No Rust changed, so no cargo matrix. CI runs on push.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag.
- Rebase onto `main` only if the push is rejected for being behind. Never force-push over someone
  else's commit.
- **Never poll CI.** After your last push, run `gh pr checks 273 --watch` once in the background,
  and don't write your own wait loop. When it exits, take one snapshot at the PR's head SHA, every
  leg.
- A piped command reports the pipe's exit code, so redirect and read `$?`. zsh doesn't word-split
  unquoted parameters, and `"$H:tests/…"` needs braces (`${H}:`) or zsh reads `:t` as a modifier.
  `git pull` overwrites `FETCH_HEAD`.
- `rtk` can truncate `grep` and `git log`, so use `/usr/bin/grep` and `/usr/bin/git` for anything
  you count.
- Never share a `CARGO_TARGET_DIR` with another session, and delete yours when you're done. Write
  scratch files only in your session's scratchpad.
- **Budget ~40 exchanges.**

## When you finish
- Add a short `### Punch list (verify)` subsection to the spec's `## Build Completion`. It holds:
  - DEC-118's two rules as they now read;
  - the comment as it now reads;
  - C36's row and how you derived its literals;
  - the `model-min` cases and how you derived them;
  - the three red runs with their quoted failure lines;
  - both summary lines;
  - the CI snapshot.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `/usr/bin/grep -l "SPEC-161 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
    `output_tokens`.
  - Price at the model in `.message.model`: Sonnet $3/$15 per MTok, Opus $5/$25. cache_creation
    is ×1.25 the input rate and cache_read ×0.10.
  - Append a `punchlist` entry to the spec's `cost.sessions` with `tokens_total`, a
    `tokens_breakdown`, `estimated_usd` and the rate anchors.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-161 punch list: PR #273 at <head SHA> — CI <green | red: legs>`.

Then, briefly:
- `git diff --stat f9c81ae`;
- items 1–7, each with what changed, quoted where it's text;
- the three red runs, with quoted failure lines;
- both summary lines;
- whether C36 joined any control's red list;
- the system temp folder's `ddphotos-*` counts before and after, and the space freed;
- anything that didn't go as written;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
