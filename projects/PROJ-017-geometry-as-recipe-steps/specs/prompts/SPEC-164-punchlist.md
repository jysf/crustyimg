# SPEC-164 — PUNCH LIST prompt: two comments in the Chrome lib, one history note, and the preset row

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. You did not design,
build or verify SPEC-164. Verify found the code correct everywhere it drove it: its own model agreed
with all 153 literals and with 187,096 fuzzed steps. What's left is three comments and one row that
pins a rule the code already follows. No item changes what the code does, so no re-verify follows.

```
just wt fix/spec-164-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-164-crop-state-model:refs/remotes/origin/feat/spec-164-crop-state-model
git reset --hard origin/feat/spec-164-crop-state-model
```
Push to the PR branch with `git push origin HEAD:feat/spec-164-crop-state-model`. Draft PR #274's
head was `b7fee27` when this was written. If it has moved, stop and report. If a push is refused,
stop and report the command; don't work around it. Check `git ls-remote` before saying anything
landed.

The maintainer will paste your final message back to the orchestrator, so it must stand on its
own. **Write it as plain lists, not box-drawn tables.**

## What to fix (settled; don't re-decide)
Each item says its kind: **wording** (text only) or **assert** (a new check; behaviour unchanged).

1. **wording — a spec id in `scripts/lib/chrome.mjs:72`.** The comment says the engine's Web Worker
   is an auto-attached target "(SPEC-078)". Drop the parenthetical and keep the sentence.
2. **wording — the start-up budget comment, `scripts/lib/chrome.mjs:~106–113`.** It says Chrome's
   start-up "gets the same budget as every other wait in the checks that use it (90 s)". That's
   wrong now: `tests/ddphotos_page.mjs` waits 60 s, or 120 s for a Make. Rewrite the comment to say
   what the code does and why: start-up gets 90 s because a cold Chrome on a loaded CI runner can
   take well over 10 s. Drop the 2026-08-21 history and the "same as" claim. Don't change
   `CHROME_STARTUP_MS`.
3. **wording — a history note in `tests/ddphotos_page.mjs:~132`.** `waitFor`'s comment ends with
   "(A wait that only timed out took 60 s and could not say why.)". That's the prototype's history.
   Drop that sentence and keep the rest.
4. **assert — pressing the active preset again.** `cropReducer` always returns a new state for
   `preset`, even when the same preset is already active with no drag. The design's preset text
   says so, but no row pins it.
   - Add one reducer row, **X31**, on each of the four shapes (W, P, R and the square): from
     `reset`, press `3:2`, then press `3:2` again.
   - Its expected crop, preset, drag and crop-step text are the same as the existing row that
     presses `3:2` once from `reset` on that shape. Copy them from that row's literals; never compute
     them with `recipe.js`. Its same-object field is `false`. If no existing row presses `3:2` once
     from `reset`, stop and report rather than derive the values yourself.
   - The crop-mode count goes from 153 to 157. Update the `mode-count` check and anything else in
     the harness that holds 153 as a literal. Don't touch the spec's prose counts: ship does those.
   - **Negative control.** In a scratch copy, add a same-object shortcut to `preset` (return `s`
     when the preset is already the active one and there's no drag), and show X31 go red on all four
     shapes. Quote the first failure line, and say whether any other row goes red. Then restore it
     and show green.

## Not yours
Ship handles these on `main` after the merge. Don't edit them:
- the spec's design sentence "An action that changes nothing returns the same object" and AC-4's
  wording (verify's punch 3);
- the M4 cell in the mutation table;
- the spec's row counts (153 → 157 in its prose, ACs and Failing Tests);
- `agents.implementer` in the spec's front matter;
- `guidance/ideas.yaml` and STAGE-060.

Stay in these files: `scripts/lib/chrome.mjs`, `tests/ddphotos_page.mjs`, `tests/ddphotos_parity.mjs`,
and the SPEC-164 spec (Build Completion and `cost.sessions` only). `git diff --stat b7fee27` must
show nothing else. `recipe.js`, `edit.js`, `index.html`, `worker.js`, `demo_smoke.mjs`, `ci.yml` and
the `justfile` stay unchanged.

## Temp folders (set up before the first harness run)
The parity harness keeps its `mkdtempSync(tmpdir())` folders whenever a run fails, and `--self-test`
always keeps a `ddphotos-orientation-*` folder because N3 is red by design. The page check makes
`ddphotos-page-site-*` and `ddphotos-page-chrome-*` folders.
- Before the first run, count the system temp folder's `ddphotos-*` folders and their size. Don't
  delete any of them: some belong to other sessions.
- Run every harness and page-check command with `TMPDIR` set to a folder in your scratchpad. After
  the first run, confirm the folders landed there; stop if they didn't.
- Empty that folder after you've read each red run, and at the end.
- At the end, count the system temp folder's `ddphotos-*` again and report both counts.

## Check it
- `just ddphotos-parity`, both modes, with your own `CARGO_TARGET_DIR` and stdout redirected, reading
  `$?`. Both must exit 0. Quote the normal-mode summary line: it reads `157 crop-mode rows`, and the
  other five counts are unchanged.
- `just ddphotos-page` and `just demo-smoke`: both exit 0. Quote their last lines. (Items 1–3 change
  only comments, but these files are what the checks import.)
- Show the negative control from item 4: red, then restored and green.
- `/usr/bin/grep -n 'SPEC-\|STAGE-\|DEC-\|PROJ-'` over `scripts/lib/chrome.mjs`,
  `tests/ddphotos_page.mjs` and your added lines in `tests/ddphotos_parity.mjs` must find nothing.
- No Rust changed, so no cargo matrix. CI runs on push.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag.
- Rebase onto `main` only if the push is rejected for being behind. Never force-push over someone
  else's commit.
- **Never poll CI.** After your last push, run `gh pr checks 274 --watch` once in the background,
  and don't write your own wait loop. When it exits, take one snapshot at the PR's head SHA, every
  leg. Read the `ddphotos-parity` log once and quote its summary line and the page check's last line.
- A piped command reports the pipe's exit code, so redirect and read `$?`. zsh doesn't word-split
  unquoted parameters, and `"$H:tests/…"` needs braces (`${H}:`) or zsh reads `:t` as a modifier.
  `git pull` overwrites `FETCH_HEAD`.
- `rtk` can truncate `grep` and `git log`, so use `/usr/bin/grep` and `/usr/bin/git` for anything
  you count.
- Never share a `CARGO_TARGET_DIR` with another session, and delete yours when you're done. Write
  scratch files only in your session's scratchpad.
- Never drive the maintainer's browser.
- **Budget ~40 exchanges.**

## When you finish
- Add a short `### Punch list (verify)` subsection to the spec's `## Build Completion`. It holds:
  - the three comments as they now read;
  - X31's four rows, and the row you copied each one's literals from;
  - the negative control, with its quoted failure line;
  - the parity summary line, and the page check's and demo smoke's last lines;
  - the CI snapshot.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `/usr/bin/grep -l "SPEC-164 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
    `output_tokens`.
  - Price at the model in `.message.model`: Sonnet $3/$15 per MTok, Opus $5/$25. cache_creation
    is ×1.25 the input rate and cache_read ×0.10.
  - Append a `punchlist` entry to the spec's `cost.sessions` with `tokens_total`, a
    `tokens_breakdown`, `estimated_usd` and the rate anchors.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-164 punch list: PR #274 at <head SHA> — CI <green | red: legs | pending>`.

Then, briefly:
- `git diff --stat b7fee27`;
- items 1–4, each with what changed, quoted where it's text;
- the negative control, with its quoted failure line, and any other row that went red;
- the parity summary line, and the page check's and demo smoke's last lines;
- the system temp folder's `ddphotos-*` counts before and after, and the space freed;
- anything that didn't go as written;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
