# SPEC-164 — PUNCH LIST 2 prompt: the page check reads the page before it has loaded

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. You did not design,
build or verify SPEC-164, and you did not run the first punch list. The first punch list landed
(`3852b45`, `10ae6e5`), and CI went red on one leg at `10ae6e5`. That red is a race in the page check,
not a flake, and it is the only thing left before ship.

```
just wt fix/spec-164-punch-2 -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-164-crop-state-model:refs/remotes/origin/feat/spec-164-crop-state-model
git reset --hard origin/feat/spec-164-crop-state-model
```
Push to the PR branch with `git push origin HEAD:feat/spec-164-crop-state-model`. Draft PR #274's
head was `10ae6e5` when this was written. If it has moved, stop and report. If a push is refused,
stop and report the command; don't work around it. Check `git ls-remote` before saying anything
landed.

The maintainer will paste your final message back to the orchestrator, so it must stand on its
own. **Write it as plain lists, not box-drawn tables.**

## What failed (read, don't re-investigate)
CI's "Edit page in headless Chrome" step at `10ae6e5` (run 46032026678, job 140618042044):
```
FAIL Error: TypeError: Cannot read properties of null (reading 'src')
    at <anonymous>:1:52
    at CDP.eval (scripts/lib/chrome.mjs:98:13)
    at async waitFor (tests/ddphotos_page.mjs:143:19)
    at async waitForFirstPreview (tests/ddphotos_page.mjs:157:3)
    at async run (tests/ddphotos_page.mjs:218:5)
ddphotos-page: 0 rows, 1 failed
```
The same code passed at `3852b45` and locally.

**The cause.** `open()` sends `Page.navigate` and returns. `Page.navigate` resolves once the
navigation starts, not once the new document has parsed. `waitFor`'s first poll can therefore run in
the document that was there before: `about:blank` for W, where `#preview` doesn't exist, so the
expression throws and `CDP.eval` turns that into a failure.

The same race has a quieter second form at the R photo (`tests/ddphotos_page.mjs:~253`). When R is
opened, the W document already has a blob preview, `#busy` hidden and Make enabled. So
`waitForFirstPreview("R")` can return true on the **old W page**, and H6/H7 then read whichever
document happens to be loaded.

## What to fix (settled; don't re-decide)
1. **`open()` waits for the new document.** Change only `tests/ddphotos_page.mjs`. Before
   navigating, mark the current window (for example `window.__leaving = true`). After
   `Page.navigate`, poll until the marker is gone, `location.href` is the URL you navigated to, and
   `document.readyState` isn't `"loading"`. Each poll must be safe in any document: wrap it in a
   `try`, and return `false` on a throw. Give the poll a timeout and a message naming the URL.
   - Don't make `waitFor` swallow throws. Once `open()` guarantees the right document, a throw from
     the predicate is a real bug and must still fail.
   - Don't change `scripts/lib/chrome.mjs`. `demo_smoke.mjs` shares it.
   - A short comment on `open()` says why it waits: the navigation command returns before the new
     page exists. Write it plainly, with no spec ids and no history.
2. **Prove both forms of the race, then prove the fix.** Use a scratch copy, not the worktree's files.
   Show each run's exit code and last lines.
   - **(a) The CI failure.** Delay the static server's response for `/edit/index.html` by about 2 s;
     a wrapper or a scratch copy of the server both work. Without the fix, the check must fail with
     the `reading 'src'` error from W. With the fix, it must pass with 8 rows and 0 failed.
   - **(b) The stale page.** Keep the same delay. Without the fix, log `location.search` at the moment
     `waitForFirstPreview("R")` returns, and show that it is still `?album=test&photo=1`. If the delay
     doesn't make this happen, say so: don't claim it. With the fix, show `?album=test&photo=2`.
   - Remove the logging and the delay before you commit. `git diff --stat 10ae6e5` must show only
     `tests/ddphotos_page.mjs` and the spec.

## Not yours
- Everything listed for ship in the first punch-list prompt: AC-4's wording, the M4 cell, 153 → 157
  in the prose, `agents.implementer`, `guidance/ideas.yaml` and STAGE-060.
- `recipe.js`, `edit.js`, `index.html`, `worker.js`, `chrome.mjs`, `demo_smoke.mjs`,
  `ddphotos_parity.mjs`, `ci.yml` and the `justfile` all stay unchanged.

## Temp folders
- The page check makes `ddphotos-page-site-*` and `ddphotos-page-chrome-*` folders. Run every
  harness and page-check command with `TMPDIR` set to a folder in your scratchpad, and confirm after
  the first run that they landed there.
- Count the system temp folder's `ddphotos-*` folders and their size before your first run and again
  at the end. Delete none of them: some belong to other sessions.
- Empty your scratchpad's temp folder when you're done.

## Check it
- `just ddphotos-page` exits 0 **five times in a row** with the fix, and no delay. Quote the last line
  once and give the five exit codes.
- `just ddphotos-parity` (normal mode) and `just demo-smoke` each exit 0. Use your own
  `CARGO_TARGET_DIR`, redirect stdout and read `$?`. Quote the parity summary line: it still reads
  `157 crop-mode rows`.
- `/usr/bin/grep -n 'SPEC-\|STAGE-\|DEC-\|PROJ-' tests/ddphotos_page.mjs` must find nothing.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- Commit the code and the spec's record **in one commit**, so CI's snapshot describes the final head.
  Measure your cost just before that commit.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag.
- The PR is BEHIND `main`, but that's only docs. Don't update the branch or rebase unless the push is
  rejected.
- **Never poll CI.** After the push, run `gh pr checks 274 --watch` once in the background. If it
  returns before any check has registered, run it once more and no more. When it exits, take one
  snapshot of every leg at the PR's head SHA. Read the `ddphotos-parity` log once, and quote its
  summary line and the page check's last line.
- **If the page check is red in CI after the fix, don't re-run it and don't re-fix it.** Report the
  failure lines and stop.
- A piped command reports the pipe's exit code, so redirect and read `$?`. zsh doesn't word-split
  unquoted parameters, and `"$H:tests/…"` needs braces (`${H}:`).
- Use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session, and delete yours when you're done.
- Never drive the maintainer's browser.
- **Budget ~35 exchanges.**

## When you finish
- Add a `### Punch list 2 (CI race)` subsection to the spec's `## Build Completion`. It holds:
  - the cause, in one sentence;
  - the new `open()` and its comment;
  - (a) and (b), each red and then green, with the quoted lines;
  - the five exit codes;
  - the parity, page-check and demo-smoke last lines.

  The CI snapshot goes in your report, not the spec.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it by content: `/usr/bin/grep -l "SPEC-164 — PUNCH LIST 2 prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
    `output_tokens`.
  - Sonnet costs $3/$15 per MTok; cache_creation is ×1.25 the input rate and cache_read ×0.10.
  - Append a `punchlist-2` entry to `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-164 punch list 2: PR #274 at <head SHA> — CI <green | red: legs | pending>`.

Then, briefly:
- `git diff --stat 10ae6e5`;
- the new `open()`, quoted;
- (a) and (b): what each looked like before and after the fix, with the quoted lines;
- the five page-check exit codes, and the parity, page-check and demo-smoke last lines;
- the CI snapshot, every leg;
- the system temp folder's `ddphotos-*` counts before and after;
- anything that didn't go as written;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
