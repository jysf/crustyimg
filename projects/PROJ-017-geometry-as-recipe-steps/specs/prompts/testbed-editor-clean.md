# Test bed — FIX prompt: install the editor into an empty edit folder, as the README says

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
`scripts/editor.sh` installs the edit page by running crustyimg's own
`just ddphotos-edit-install "$CONFIG"`, which copies every page file into
`$CONFIG/static/edit/` (and the engine into `edit/vendor/`). **It never clears that folder
first.** That's deliberate on crustyimg's side: the installer also accepts any folder as its
target, so it won't delete files there. crustyimg's `integrations/ddphotos/README.md` tells
users instead: *"To update the page, replace the whole `edit` folder, then Build (and Export)
again."*

The test bed doesn't do that. The last session (test bed `6e8ca04`) found the cost:
- Installing `main` and then an older ref left `main`'s `recipe.js` next to the older `edit.js`.
- The new `check.py` then raised "more than one installed editor file defines GO_ADVANCE /
  GO_WIDEST (edit.js and recipe.js)".
- The session had to clear the folder by hand to get a clean "before" run.

Any file the page drops in a later version would linger the same way, and the HTTP checks would
keep passing on a file the page no longer ships.

## The fix (settled, do not re-decide)
1. **In `scripts/editor.sh`, remove `$CONFIG/static/edit` just before the install step** (before
   the `say "just ddphotos-edit-install $CONFIG"` line, about `:45`).
   - Put it after the `albums.yaml` check, so a missing config still dies first with its own
     message.
   - Guard the path. Refuse to remove anything unless `$CONFIG` is non-empty and the target ends in
     `/static/edit`. Use `rm -rf -- "$CONFIG/static/edit"`; never an unquoted or computed glob.
   - Print one `say` line naming the folder it cleared, so the run log shows it.
   - Add a short comment citing the README's "replace the whole `edit` folder" rule. Keep it
     behaviour-first, with no spec or PR numbers.
2. **The static build must not keep a stale copy either.** Read `scripts/gallery.sh build`, which
   `editor.sh` runs at the end, and whatever ddphotos does when it copies `config/static/` into the
   built site.
   - If the built site's `edit/` is rebuilt from scratch, say so with the evidence, and change
     nothing there.
   - If it can keep a stale file, clear the built site's `edit/` the same way, in `gallery.sh`,
     and say why.
3. Change nothing else. `git diff --stat` must show only `scripts/editor.sh`, plus `gallery.sh`
   only if step 2 needs it. Fix any `README.md` or `CHECKLIST.md` line that says the editor install
   overwrites in place.

## Prove it
The crustyimg checkout is `~/PSeven/experiments/crustimg_redo_plus/crustyimg`. Don't change
anything in it. `editor.sh` takes a ref directly, so you need no worktree. Run each step once, in
order, and record its result.
1. **The stale-file case, from a dirty start.** Run `just editor main`. Then drop a marker file
   `work/config/static/edit/zz-stale.js` by hand, and run `just editor 53954d4` (the crustyimg
   commit before `recipe.js` existed).
   - After it, `work/config/static/edit/` holds neither `recipe.js` nor `zz-stale.js`.
   - List the folder. It must match crustyimg's `integrations/ddphotos/edit/` at `53954d4`
     (`git -C <crustyimg> ls-tree --name-only 53954d4 integrations/ddphotos/edit/`) plus
     `vendor/`.
   - `just stop`, `just serve-all`, `just check`: green, with the table read from `edit.js` and
     no "more than one" error.
   - If step 2 found the built site can go stale, also show the built `edit/` has no
     `recipe.js` or `zz-stale.js`.
2. **Back to `main`.** `just editor main`, then `just stop`, `just serve-all`, `just check`:
   green, with the table read from `recipe.js`, and `/edit/recipe.js` served as JavaScript on every
   server that answers.
3. **The guard.** With `CONFIG` pointed at an empty string or a path that doesn't end in
   `/static/edit` (a scratch copy of the guard is enough; don't edit `lib.sh`), the guard must
   refuse and remove nothing. Show it.
4. **One full `just retest main`.** Record the browser counts per server (the last run saw 40/0 on
   dev and static) and whether the automated stage completes. If Docker isn't running, apache is
   skipped; say so.
5. Run `just stop`, and confirm `pgrep -f serve.sh` is empty.

## Guardrails
- ⛔ **Don't commit the results.** `just test`/`retest` write `results/<date>-<sha>/` and a
  `.draft.md`; the maintainer reviews and commits those. Leave them, and the other untracked files
  already present, untracked and untouched.
- **Commit only the scripts** (plus any doc line from fix 3), with `git commit -s` and the message
  `editor.sh: install into an empty edit folder, as the page's README says`, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. There is no remote, so don't push.
- Never edit `vendor/ddphotos/`. Check that `git -C vendor/ddphotos status --short` is empty at
  the end.
- Never open the user's browser. Leave no servers running.
- macOS has no `timeout(1)`. A piped command reports the pipe's exit code, so redirect and read
  `$?`. zsh does not word-split unquoted parameters.
- **Budget ~20 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `grep -l "install the editor into an empty edit folder" ~/.claude/projects/<slug>/*.jsonl`.
  If that matches an earlier session's file too, use the one whose first user message names this
  prompt.
- Dedup by `.message.id`: take the input and both cache counts once per id, and the **MAX**
  `output_tokens`.
- Price each component at Sonnet's $3/$15 per MTok, with cache_creation at ×1.25 and cache_read
  at ×0.10.

## Report back (your final message — it is pasted to crustyimg's orchestrator)
First line: `Test bed editor clean: DONE at <commit SHA> | BLOCKED: <why>`.

Then, short:
- the diff stat;
- what step 2 found about the built site's `edit/`, with the evidence;
- step 1: the folder listing after the `53954d4` install, and `just check`'s result;
- step 2's result and the `recipe.js` HTTP row per server;
- step 3's refusal;
- step 4's browser counts per server, and whether the automated stage completed;
- the results directories the runs created, left uncommitted;
- your measured cost.
