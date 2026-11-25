# SPEC-157 — VERIFY prompt: `build` destinations have one identity

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not frame,
design or build SPEC-157. Your job is to find what the build's own evidence can't show. 24 ticked ACs
and a mutation table that matches are claims until you drive them. The rows test the shapes design
thought of, and the resolver touches every path a user can spell.

```
just wt verify/spec-157 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-157-build-destinations:refs/remotes/origin/feat/spec-157-build-destinations
git checkout --detach origin/feat/spec-157-build-destinations
```

When this was written, draft PR #270's head was `9733cd8`. Its CI was still running: 13 passed,
8 skipped (release-only), 5 in progress (the three-OS test legs among them), none failed. Record the
SHA you verify. The maintainer pastes your final message back to the orchestrator, so it must stand
on its own. **Write it as plain lists, not box-drawn tables.**

⛔ **PR #270 is not to be merged until this verify, and any punch list and re-verify it leads to,
has reported.**

## What the build claims
- **Baseline on `main`'s code:** FT-5, FT-19, FT-20 and FT-27 green; the other 23 destination rows and
  both watch rows red for their reason. FT-29 saw 15 builds in 3 s; FT-30 never saw a second build.
- **ACs:** all 24 met. AC-22 (Windows, FT-34) rests on the Windows CI leg alone.
- **Drives on the binary:** every refusal exit 2 with nothing written; in-tree link exit 0, locking
  `build/dist/a.png`; `./dist/` and `a/../dist` lock as `dist/…`; `--check` exit 7 on drift, exit 2
  with nothing written when refused. The sink re-check (exit 5) is asserted by FT-28 only, not driven.
- **Upgrade:** a lock written by `main` through an in-tree link drifts once under `--check` (exit 7),
  one plain build rewrites it, `--check` then exits 0.
- **Mutations:** M1–M14 and M3b match the table, except **M12**, which also turns FT-28 red (the
  mutant's inserted check fires on FT-28's temp dir). M11 is all green (equivalent on Unix). Trap 14
  gives the table's 25 red.
- **Matrix:** default, `--no-default-features` and `--features webp-lossy` clean; `just wasm-check`
  passes with the existing `MANIFEST_DROPPED_WARNING` warning.
- **Deviations:** stages 2 and 3 in one commit; `check_output_injective` doesn't take `root`; two
  `--force-with-lease` pushes of its own branch after rebasing.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes, no new PR. Report; don't fix. Every build and
mutant goes in a scratch worktree with its own `CARGO_TARGET_DIR` under your scratchpad; all are gone
when you finish.

## Read first
1. The spec on the PR branch, in full:
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-157-build-destinations-have-one-identity.md`.
   - `## Frame` with the five maintainer rulings;
   - `## Design`: the seven corrections and "What was driven". Note which claims are marked
     *reasoned* rather than driven;
   - every AC, Failing Tests, the baseline and the mutation table;
   - `## Implementation Context`: the design calls, the new DEC and DEC-061's amendment (both ship's),
     the docs, the CHANGELOG, the overlap, out of scope, the follow-up filed;
   - Build Completion and the build's reflection.
2. The PR's diff of `src/cli/build.rs` in full, then `src/cli/mod.rs`, `tests/build_destinations.rs`
   and `tests/build_watch.rs`.
3. `/AGENTS.md` §15 "During design": the coverage-map rule (input shapes × every step that reads them;
   what an observer sees between ordered steps). It landed after this spec was designed. You apply it
   here, at check 4.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and traps 12–14.

## Check, in order
1. **Scope.**
   - `git diff --name-only origin/main...HEAD` against `## Outputs`: 9 files plus the spec.
   - `Cargo.*`, `decisions/`, `src/sink/` and `src/cli/common.rs` didn't change. `apply` still calls
     `write_encoded`.
   - Every commit is signed off. The PR merges cleanly onto current `main`. Run `git merge-tree`
     against #226 and #237 and name any conflicting hunk.
2. **The code against the design calls.** For each call in "Design calls (settled)", find it in the
   PR and say whether it matches. In particular:
   - `prepared.out_dir` is set only by `check_destinations`, and no path in `build_one` or `--watch`
     falls back to `target.out`. Grep every use of `target.out` and classify each;
   - the refusals run before `load_lock`, `Cache::open` and any `create_dir_all`;
   - `write_planned` re-resolves after `create_dir_all` and writes through `Sink::File`'s arm of
     `write_bytes`, so SPEC-155's `publish` and DEC-035's per-file link refusal both still apply;
   - the build's reflection says the read-set lookup joins one component onto `out_dir` for Windows.
     Read that code and say whether a name with a separator could reach it.
3. **Drive every refusal and every success the docs name.** Build `main` and the PR head `--release`,
   each in its own fresh `CARGO_TARGET_DIR`. For each AC that names a message, run it on both binaries
   in a fresh tree and quote exit code and stderr. Every message must equal the spec's text byte for
   byte. Snapshot the tree (sha256 of every file, links recorded as links) before and after each
   refusal, and say whether anything changed, `.crustyimg/` and the lockfile included.
4. **The coverage map, applied after the fact.** This is the main check. Build both grids, drive every
   cell on the PR's release binary (and on `main` where the behaviour should be unchanged), and say
   which test row covers it, if any:
   - **`out` shapes × the steps that read `out`** (planning, `create_dir_all`, the re-resolve, the
     write, the lock record, the `--watch` exclusion):
     - missing; an existing directory; a regular file; a link to a file; a link to an in-tree
       directory; a link to an out-of-tree directory; a dangling link at `out`; a dangling link
       mid-`out` (`dl/x`); `..` in the non-existent tail after a link (`lnk/new/../d`, marked
       *reasoned, not driven on the prototype* in the design); a read-only directory; an `out` whose
       case differs from the folder on disk (`DIST` over `dist/`);
     - a link planted **at the output file name** inside a real `out` (DEC-035: exit 5 even with
       `--yes`).
   - **Read-set shapes × the overlap check:** a single-file source, a glob source, a folder source, a
     recipe given as a file, a recipe given as a bundled name (not a file), a watermark image, a
     watermark font file, the embedded font, the manifest, the lockfile, `.crustyimg/`. Include an
     `{ext}` name against a source with each output extension, and with the format pinned (literal
     extension, recipe `format`), where only the pinned extension should be refused.
   - **Case and Unicode:** `ß`/`SS`, `é` NFC vs NFD (filed as out of scope; confirm the behaviour is
     what the DEC text and CHANGELOG say, not worse), and a Turkish dotless `ı`/`I`. For each, give
     what `fold_case` returns and what the build does.
   - **Between the steps of one write:** plan → `create_dir_all` → re-resolve → stage (SPEC-155) →
     rename. Say what an observer or a killed process sees after each step, and which row pins each.
     The race after the re-check is accepted in the design; confirm nothing else is open.

   A cell with no row and the right behaviour is an **assert** item. A cell with the wrong behaviour
   is a **claim**.
5. **Order of errors.** Drive a manifest that has three faults at once (a `..` escape, a collision
   and an overlap) in each order of targets, and confirm the first error reported follows the
   documented order.
6. **`--check` and the lockfile.**
   - The build says `--check` after deleting an output and `.crustyimg/` wrote both back. Drive it on
     `main` too: is writing under `--check` pre-existing, and do the docs describe it?
   - Re-drive the upgrade (AC-16 and the in-tree link case) on both binaries.
   - Confirm `CACHE_SCHEMA_VERSION` and the lock version are unchanged, and that a warm cache from
     `main` still hits after the upgrade (`(n cached, 0 rebuilt)`).
7. **`--watch`.**
   - Re-drive FT-29's and FT-30's scenarios by hand on both binaries, and quote the build counts.
   - FT-29 counts builds in a 3 s window and FT-30 polls for a second summary. Run each test 10 times
     in a loop on the PR head, and report any flake. A timing row that flakes on CI is a finding.
   - Confirm no watcher process is left running when you finish (`ps -o pid,command` filtered by your
     scratch path, not `pgrep -f`).
8. **Mutations.** Each alone, `cargo clean -p crustyimg`, full `cargo test --no-fail-fast`, stdout to a
   file, red set read off the run:
   - M1, M3b, M6, M7 and M14: compare with the table;
   - **M12:** confirm FT-28 is red too, and explain why from the code. Classify the table cell
     (**wording**);
   - Trap 14;
   - one mutant per uncovered cell from check 4 that you think matters most (at most three), to show
     no row turns red.
9. **Windows.** Read the PR's Windows leg once at the head you verify:
   - did it run FT-34, and did it pass? Quote the line from the job log (one read);
   - did FT-6 to FT-8 pass there (NTFS case-insensitivity)?
   - M11 (root not canonicalized) is *reasoned* to break every build row on Windows. You can't drive
     it locally. Say what evidence would settle it, and whether its absence matters.
10. **Docs, CHANGELOG, SECURITY.md and SKILL.md.**
    - `diff` each against the spec's exact text.
    - Drive each sentence that states behaviour, quoting check 3 or 4 rows as evidence.
    - The CHANGELOG's upgrade note: does it match check 6's drift?
    - `/usr/bin/grep -nE '(SPEC|STAGE|DEC|PROJ)-[0-9]'` over the changed lines of every non-spec
      file, and classify each hit.
11. **CI.** `gh pr checks 270` once at the head. Every leg, Windows and lean included. If legs are
    still running, say which; don't wait.

## Classify every finding (AGENTS §15)
For each punch item and optional cleanup, say which kind it is:
- **wording:** a doc, comment or spec line that misstates something the code already does;
- **assert:** a missing or loose test row, where the behaviour is already right;
- **claim:** a fix that would change what a doc or test claims is true, or what the code does.

Only a **claim** needs a re-verify after its punch list.

For every bound, count or "can't happen" in the spec, say whether it was driven, by whom, and over what
range. Examples:
- "a directory that does not exist cannot be a link", so lexical `..` is exact in the tail;
- "the build's own writes never match a read", which the `--watch` carve-out relies on;
- "every input is known before any work";
- "no output path in the cache key";
- "`./dist` and `dist/` already lock as `dist/…`, so they do not drift".

## Guardrails
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters, so loop with `while IFS= read -r`. zsh also reads `$B:r…` as a
  modifier, so brace it (`${B}:`). macOS has no `timeout(1)`. `rtk` can truncate `grep` and `git log`,
  so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Redirect `cargo test` stdout to a file. An interactive terminal fails `display_sink_refuses_non_tty`.
- One fresh `CARGO_TARGET_DIR` per binary or mutant, never shared with another session. Run
  `cargo clean -p crustyimg` between mutants.
- For wasm32, use the rustup `cargo`; a bare `cargo` finds Homebrew's `rustc`.
- Kill every `--watch` process you start, by PID.
- Never poll CI and never write your own wait loop.
- Scratch files only in your session's scratchpad, never `/tmp` or the system temp dir. Restore any
  `chmod` before you delete a scratch folder. Leave the tree clean.
- **Budget ~50 exchanges.** Check 4 is the most important. If you run short, cut from checks 7–8's
  extras first, and say what you cut.

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working directory's
  absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `/usr/bin/grep -l "SPEC-157 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
  If more than one transcript matches, count every one that did work, and say so.
- Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
- Price at Opus's $5/$25 per MTok, cache_creation ×1.25, cache_read ×0.10. Measure at your last turn.
  You're read-only, so report the numbers; a later session writes them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only, no box-drawn tables. The first line is exactly one of:
- `✅ APPROVED — merge PR #270 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #270 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–11, one or two lines each, with evidence. For item 4, list every cell: its behaviour, its
  row (or "none"), and its kind if it is a finding;
- the punch items, numbered. Give each its kind, its evidence, the fix you'd expect, and whether it
  lands on the PR or on `main`;
- optional cleanups, listed separately, each with its kind;
- out-of-spec findings, each with where it belongs (STAGE-063 or `guidance/ideas.yaml`) and whether it
  is pre-existing;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own line.
