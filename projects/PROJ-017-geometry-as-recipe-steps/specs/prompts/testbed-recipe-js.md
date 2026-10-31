# Test bed — FIX prompt: read the page's font table from wherever it lives, and check every editor file

A fresh Claude Code session, on **Sonnet**, in the **test bed** repository, not in crustyimg:

```
cd ~/claudia/web/jysites/fogcityphotosandbeyond/crustyimg-ddphotos-testbed && claude --model sonnet
```

The test bed is a single local checkout with no git remote. Work on it directly, with no
worktree. The maintainer pasted this prompt into that session and will paste your final message
back to crustyimg's orchestrator, so the message must stand on its own.

Read the test bed's own `CLAUDE.md` first. Its hard rules bind you:
- never edit `vendor/ddphotos/`;
- browser checks run only in Playwright's headless Chromium, never the user's browser;
- Python stays stdlib-only and runs under `python3 -I`.

## Why
crustyimg PR #234 (SPEC-142, still a draft) moves the edit page's recipe code out of `edit.js` into
a new `edit/recipe.js`, so the page has one copy of it. The page behaves the same: the build's
`just retest` passed the browser checks 40/0 on all three servers. Two things in this test bed
assumed the old layout.

1. **`go_advances()` in `scripts/check.py`** (about `:241-250`) reads `GO_ADVANCE` and
   `GO_WIDEST` out of the installed `edit/edit.js`. With #234 installed, both names live in
   `recipe.js`, so the automated stage crashes:
   `RuntimeError: edit.js has no GO_ADVANCE / GO_WIDEST`.
2. **`http_checks()`** (about `:93-99`) lists the editor's files by name, so it would never
   request `recipe.js`. A server that failed to serve it, or served it with the wrong MIME type,
   would go unreported. The browser checks would still fail, but less clearly.

The fix must pass **both before and after** #234 merges. crustyimg `main` has no `recipe.js`
yet, and the PR branch does.

## The fix (settled, do not re-decide)
1. **`go_advances()` reads the table from whichever installed editor file defines it.**
   - Search every `*.js` directly under the installed `edit/` dir, not `vendor/`.
   - Exactly one file must define `const GO_ADVANCE = [` and `const GO_WIDEST =`.
   - **None** raises `RuntimeError`, naming every file it searched.
   - **More than one** raises `RuntimeError` naming both. Two copies would let the test bed's
     mirror follow the wrong one.
   - Keep the docstring's point, that the mirror can't drift from the page, and name the file it
     actually read. Update `watermark_size()`'s docstring, which says "in edit.js", to say "in
     the page's recipe code".
2. **`http_checks()` requests every installed editor file.**
   - Derive the list from the installed `edit/` directory: every file directly in it, plus the
     `vendor/` files it already checks. Don't hard-code it, so the next file the page adds is
     covered too.
   - Keep the gallery and album paths as they are.
   - Each `.js` must also be served with a JavaScript content type, the same check the engine JS
     already gets. A module script served as `text/plain` fails in the browser.
3. Change nothing else. `git diff --stat` must show only `scripts/check.py`, plus any `README.md`
   or `CHECKLIST.md` line that names the editor's files one by one; fix those in the same spirit.

## Prove it
Read `scripts/editor.sh` first, so you know how it takes a crustyimg ref or path. The crustyimg
checkout is `~/PSeven/experiments/crustimg_redo_plus/crustyimg`. Don't change anything in it. If
you need the PR's code, make a **detached worktree** of
`origin/test/spec-142-ddphotos-parity` under `/tmp`, and remove it afterwards.

Run each step below once, in order, and record its result.
1. **Before #234 (crustyimg `main`):** run `just editor main`, then `just stop`, `just serve-all`
   and `just check`.
   - The run must be green, with no `recipe.js` row.
   - `go_advances()` reports reading `edit.js`.
2. **After #234 (the PR's code):** run `just editor <the detached worktree path>`, then
   `just stop`, `just serve-all` and `just check`.
   - The run must be green.
   - `GET /edit/recipe.js` is 200 with a JavaScript type, on every server that answers.
   - `go_advances()` reports reading `recipe.js`.
3. **Negative controls**, on the installed copy under `work/`, never in crustyimg:
   - Rename `GO_ADVANCE` in the installed `recipe.js`. `just check` must raise the "none" error,
     naming the files it searched.
   - Then copy the two `const` lines into the installed `edit.js` too, so both files define the
     names. `just check` must raise the "more than one" error.
   - Then re-run `just editor <path>` to restore.
4. **One full `just retest <the detached worktree path>`.**
   - Record the browser-check pass/fail counts per server. The build saw 40/0 on dev, static and
     apache, with no editor check skipped.
   - Record whether the automated stage now completes.
   - If Docker isn't running, apache is skipped. Say so.
5. Run `just stop`. Confirm `pgrep -f serve.sh` is empty, and remove the `/tmp` worktree
   (`git -C <crustyimg> worktree remove …`).

## Guardrails
- ⛔ **Don't commit the results.** `just test`/`retest` write `results/<date>-<sha>/` and a
  `.draft.md`. The maintainer reviews and commits those. Leave them, and the other untracked
  files already present (`CLAUDE.md` and earlier results dirs), untracked and untouched.
- **Commit only `scripts/check.py`** (plus any doc line from fix 3), with `git commit -s`. Use the
  message `check.py: read the font table from whichever editor file defines it; check every
  editor file`, ending with `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. There is
  no remote, so don't push.
- Never edit `vendor/ddphotos/`. Check that `git -C vendor/ddphotos status --short` is empty at
  the end.
- Never open the user's browser. Leave no servers running.
- macOS has no `timeout(1)`. A piped command reports the pipe's exit code, so redirect and read
  `$?`.
- **Budget ~25 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `grep -l "Test bed — FIX prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
  `output_tokens`.
- Price each component at Sonnet's $3/$15 per MTok, with cache_creation at ×1.25 and cache_read
  at ×0.10.

## Report back (your final message — it is pasted to crustyimg's orchestrator)
First line: `Test bed fix: DONE at <commit SHA> | BLOCKED: <why>`.

Then, short:
- the diff stat;
- steps 1–2: green or red, which file `go_advances()` read, and the `recipe.js` HTTP row per
  server;
- step 3's two error messages;
- step 4's browser counts per server, and whether the automated stage completed;
- the results directories the runs created, left uncommitted for the maintainer;
- anything else in the test bed that assumes the old layout;
- your measured cost.
