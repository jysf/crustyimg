# SPEC-157 — RE-VERIFY prompt: the two claim fixes, and nothing else

Cycle: **verify (narrow re-verify)**. Fresh CLI session, own worktree, **Opus**, **read-only**. The
first verify found two **claims** in `build`'s destination checks, and the punch list fixed both:
- a source overwritten on APFS through `ß`/`SS` or NFC/NFD;
- a glob source that builds once into its own folder, then refuses every later build.

You check those two fixes, the rows and the sentences that now describe them. Everything else verify
already approved, so don't re-open it. You did not design, build, verify or punch-list SPEC-157.

```
just wt verify/spec-157-re -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-157-build-destinations:refs/remotes/origin/feat/spec-157-build-destinations
git checkout --detach origin/feat/spec-157-build-destinations
```

When this was written, draft PR #270's head was `8a7b03c`, two punch-list code commits on top of
`9733cd8`:
- `14b1b0f`: the identity check and the listing check;
- `7e1411c`: the rows and the wording.

`8a7b03c` itself only records the CI snapshot in the spec. The orchestrator read the checks at the
head: 18 passed, 8 skipped (release-only), none failed.

If the maintainer has since updated the branch from `main`, the head will have moved:
- `/usr/bin/git log --first-parent --no-merges --oneline 8a7b03c..HEAD` must list nothing;
- after a merge, show any conflict resolution with `git show --remerge-diff <merge>`, and check that
  none of it changes a punch-list line;
- after a rebase, show with `git range-diff origin/main 8a7b03c HEAD` that the PR's own commits are
  unchanged.

Anything else: stop and report. Record the SHA you verify.

The maintainer pastes your final message back to the orchestrator, so it must stand on its own.
**Write it as plain lists, not box-drawn tables.**

## What the punch list claims
- **Fix 1, identity:**
  - **The check.** `check_destinations` stats each planned output (under every candidate
    extension), and refuses it with `OutputOverlap` and that read's clause when its device and inode
    match a file the build reads. It runs after the folded-string lookup misses. It is `cfg(unix)`
    only; Windows skips it, and the reason is in a comment (reasoned, not driven).
  - **Rows**, each red before (exit 0) and green after:
    - FT-35: `ß`/`SS` over its own source, on macOS;
    - FT-36: an NFD name over an NFC source, on macOS;
    - FT-37: a hard link to a source planted at the output name.

    FT-35 and FT-36 ran on APFS, not skipped.
  - **M15**, the check removed, turns those three red.
- **Fix 2, listing:**
  - **The check.** In the same pass, a planned output is refused when a glob source matches it, or
    when it sits directly in a folder source with an extension that folder would list. The glob's
    literal prefix is normalized the way `out` is.
  - **The extension list.** `READABLE_EXTENSIONS` is a copy of the source filter's list, because
    `src/source/` was out of scope. FT-52 is said to fail if the two drift.
  - **Messages:** ``which target #0's source pattern `src/*.png` would read on the next build`` and
    the same clause with ``source folder `src` ``.
  - **Rows:**
    - FT-38 (glob), FT-39 (folder), FT-41 (spelling matrix) and FT-42 (links), each red before;
    - FT-40, a non-matching glob that builds twice, is a green pin.
  - **Mutations.** M16 (the whole check removed) turns FT-38, 39, 41 and 42 red. M16glob (the glob
    arm only) turns 38, 41 and 42 red.
  - **Shapes the punch list reasoned rather than drove:**
    - `SRC/*.png` on a case-sensitive filesystem;
    - `x/../src/*.png`;
    - a sub-folder of a folder source, not refused.
- **Fix 3, pins:** FT-43 to FT-51. M17 → FT-43, M18 → FT-44 and M19 → FT-45.
  - The re-runs: M7 is now 39 red and TRAP14 40.
- **Fix 4, wording.**
  - **Exit codes in `SECURITY.md`:** exit 6 for a folder that moved. Line 42 now reads "exit 5; build
    aggregates per-input failures, so it exits 6".
  - **FT-47:** a link at the output file makes `build` exit 6, and the link's target is untouched.
  - **The M12 cell** has been rewritten.
  - **The "one component" sentence** is corrected in "the DEC body", the `write_planned` comment and
    the reflection. This spec emits no DEC file, so find what "DEC body" refers to.
  - **The narrowed case wording** is in `docs/cli-reference.md` and `CHANGELOG.md`.
  - **`skills/crustyimg/SKILL.md`** is a new file in scope. It gains "a glob or folder source that
    would list its own output next time".
- **Upgrade re-drive:**
  1. A lock `main` wrote, then `--check`: exit 7 with the two expected drift lines.
  2. One build: the warm cache hits.
  3. `--check` again: exit 0.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes. Report; don't fix. Mutants and builds go in a
scratch worktree with its own `CARGO_TARGET_DIR` under your scratchpad; both are gone when you finish.

## Read first
- On the PR branch, the spec: `### Punch list (verify)` with fix 2's shapes table, the mutation table
  (M12, M15–M19), Failing Tests FT-35 to FT-52, the message list, and the verbatim doc copies.
- The punch list's diff in full: `git diff 9733cd8 8a7b03c -- src/ tests/ docs/ SECURITY.md CHANGELOG.md skills/`.
- AGENTS §15, "During design" (the coverage map) and "During verify". A bound or "can't happen" is a
  claim until its worst input is driven.

## Check, in order
1. **Scope.**
   - `git diff --stat 9733cd8 HEAD` shows only the seven files the report names.
   - `Cargo.*`, `src/sink/`, `src/source/`, `src/build/` and `.github/` are unchanged.
   - Every commit is signed off.
   - The PR still merges cleanly onto current `main`.
   - Say whether the `SKILL.md` edit is correct and needed. It is a public surface.
2. **Fix 1, driven, and its false refusals.** Build `9733cd8` and the head `--release`, each in a
   fresh `CARGO_TARGET_DIR`. On both, give the exit code and whether the source's bytes changed for:
   - **the three rows' cases:** `ß`/`SS`, NFD over NFC, and a hard link at the output name;
   - **an output that is a symlink to a source:** which refusal fires first, and is its message right?
   - **a source reached through a link** (`src` a link to `real/`) with `out = "real"` and a
     `SS`/`ß` name;

   Then check what it must **not** refuse, on the head:
   - an ordinary project built twice, so the outputs from the first build exist (the identity check
     must not see its own outputs as reads);
   - `--watch` started, then one source edited: one rebuild, no refusal;
   - an output that is a hard link to a file the build does **not** read.

   **Cost:** time one `build --check` on 2,000 small inputs with all 16 candidate extensions, on
   `9733cd8` and on the head. Quote both times. Say whether the per-extension `stat` is worth a
   wording note.
3. **Fix 2, driven, and its false refusals.** Use the head's binary, `name = "{stem}_w.{ext}"` unless
   a case says otherwise. Each case must still be **refused** (exit 2, tree unchanged):
   - the verify reproduction on the first build;
   - target B's glob that would list target A's output (chaining);
   - `src/**/*.png` with `out = "src/web"`;
   - a manifest given by a non-default path in a sub-folder, with a relative glob. Are globs resolved
     from the working directory or from the manifest? Say which, and whether the check agrees with the
     resolver.

   Each case must still **build twice**, exit 0 both times:
   - `src/*.png` with `out = "src/web"`;
   - folder source `src` with `out = "src/web"`, the sub-folder case the punch list only reasoned
     about;
   - `src/*.png` with `out = "src"` and a pinned `webp` name, as FT-40;
   - folder source `photos` with `out = "dist"`.

   For any case that is refused but builds on `main` twice without growing files, say whether the
   change is listed in `CHANGELOG.md`.

   Then drive the shapes the punch list only reasoned about:
   - `SRC/*.png` on a case-sensitive volume (`hdiutil create -fs "Case-sensitive APFS"` in your
     scratchpad, detached when you finish);
   - `x/../src/*.png` after `mkdir x`.
4. **The copied extension list.** Diff `READABLE_EXTENSIONS` against the source filter's list.
   Mutate the copy, adding one extension and then removing one, each alone. FT-52 must go red both
   times. If it doesn't, the drift guard is a claim.
5. **Mutants.** Each alone: `cargo clean -p crustyimg`, then a full `cargo test --no-fail-fast` with
   stdout redirected to a file, then read the red set off the run.
   - M15 and M16: compare with the table.
   - **M16folder**, the folder arm only: FT-39 must go red. If nothing does, that's an **assert**
     item.
   - M17–M19: spot-check one.

   Don't re-run M7 or TRAP14 unless a fix-1 or fix-2 result disagrees with the table.
6. **Every claim in the new sentences.** For each changed sentence (docs, CHANGELOG, `SECURITY.md`,
   `SKILL.md`, the `write_planned` comment):
   - quote it as it reads;
   - name the step above, or a new drive, that shows it is true.

   In particular:
   - "never overwrites a file the build reads" now has to hold for every shape in steps 2 and 3;
   - the case sentence must name exactly the residual that is left: two new outputs merged by APFS;
   - `SECURITY.md` line 42's "exit 5 … build … exits 6": drive one `apply` case and one `build` case.

   Every verbatim copy in the spec must equal its source byte for byte. Check that mechanically.
7. **CI.** Run `gh pr checks 270` once at the head you verify. Read every leg, Windows and lean
   included. FT-35 and FT-36 are `cfg(target_os = "macos")`: confirm they ran on the macOS leg rather
   than being skipped there. If the job log isn't available, say so.
8. **Bookkeeping, report only.** Does `cost.sessions` have a `verify` entry ($12.19) and a
   `punchlist` entry? Ship fixes both, so neither is a punch item.

## Classify every finding (AGENTS §15)
- **wording:** a doc, comment or spec line that misstates what the code already does;
- **assert:** a missing or loose test row, where the behaviour is already right;
- **claim:** a fix that would change what a doc or test claims is true, or what the code does.

## Guardrails
- A piped command reports the pipe's exit code, so redirect and read `$?`. zsh does not word-split
  unquoted parameters, so loop with `while IFS= read -r`, and it reads `$O:s…` as a modifier, so
  brace it (`${O}:`). macOS has no `timeout(1)`. Use `/usr/bin/grep` and `/usr/bin/git` for anything
  you count.
- Redirect `cargo test` stdout to a file. An interactive terminal fails `display_sink_refuses_non_tty`.
- One fresh `CARGO_TARGET_DIR` per binary or mutant, never shared with another session or another
  copy of your own script. The punch list lost a whole mutation series that way.
- Kill any `--watch` you start, and confirm none is left running.
- Never poll CI and never write your own wait loop.
- Scratch files only in your session's scratchpad, never `/tmp`. Detach any disk image you mount, and
  leave the tree clean.
- **Budget ~35 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working directory's
  absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `/usr/bin/grep -l "SPEC-157 — RE-VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
- Price at Opus's $5/$25 per MTok, cache_creation ×1.25, cache_read ×0.10. You're read-only, so
  report the numbers; ship writes them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is exactly one of:
- `✅ APPROVED — PR #270 at commit <SHA>; merge only after the orchestrator confirms.`
- `⚠ PUNCH LIST — PR #270 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–8, one or two lines each, with evidence;
- the punch items, numbered, each with its kind, evidence and expected fix;
- out-of-spec findings, each with where it belongs (STAGE-063 or `guidance/ideas.yaml`) and whether
  it is pre-existing;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own line.
