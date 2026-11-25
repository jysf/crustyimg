# SPEC-155 — VERIFY prompt: a write that fails must not report success or destroy the old file

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not frame,
design or build SPEC-155. Your job is to find what the build's own evidence can't show. 22 ticked
ACs, 39 green tests and a matching mutation table are claims until you drive them. A test suite can
only check the writers and failure paths someone thought to write rows for.

```
just wt verify/spec-155 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-155-atomic-writes:refs/remotes/origin/feat/spec-155-atomic-writes
git checkout --detach origin/feat/spec-155-atomic-writes
```

When this was written, draft PR #268's head was `471a0a7`. CI was green on every leg at `47003ae`,
the last code push; the orchestrator checked that run's check-runs. `471a0a7` adds only the spec's
Build Completion and cost entry, and its CI run was still in progress. Record the SHA you verify.
The maintainer will paste your final message back to the orchestrator, so it must stand on its
own. **Write it as plain lists, not box-drawn tables.**

## What the build claims
- **Baseline:** on `main`'s code, exactly the spec's 15 E rows were red, each for its stated reason,
  and the 8 pins (E10, E12, E13, E14, E17, E18, E20, E21) were green. Its first run was invalid
  (six rows panicked on a missing `in/`); it fixed the fixture helper and re-ran.
- **ACs:** 22 met. AC-20 (Windows) rests on the Windows CI leg: W1–W4 passed, and W2 got exit 5 with
  the `could not write output old.png:` prefix.
- **Mutations:** each alone, clean rebuild, full `cargo test --no-fail-fast`. Every row matched the
  table except:
  - M1, M2 and M3 also turn FT-E23 red, which the table's Red lists omit;
  - M11 is 48 red, not 46: unit rows U9 and U11 publish a 5,000 B payload through `RealFs`;
  - M9's edit used `std::fs::write`, because the parameter `fs` shadows the module;
  - trap 14a gave 3 distinct compile errors, repeated for the lib and the lib test, not 5.
- **The orchestrator's correction:** three doc lines a batch can reach now say "exits 5 (6 for a
  batch, after the other inputs are written)": the CHANGELOG Fixed line, the CHANGELOG read-only
  directory line and the `docs/cli-reference.md` "Replacing a file" note.
- **Clippy:** the unit-test module is a bare `#[cfg(test)]` plus a separate
  `#[cfg(not(target_arch = "wasm32"))]`, because clippy's test exemption only sees a bare
  `cfg(test)`. Clippy and fmt were re-run on three legs after that fix. The three local test legs
  were not; CI's were.
- **DEC:** `DEC-116`, the next free id at build time.
- **Known wording slip, don't re-report it:** Build Completion names `3802b14` as the clippy fix. That
  is the pre-rebase SHA; on the branch it is `3dc1c3c`.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits, no pushes, no new PR. Report; don't fix. Any mutation you
drive happens in a scratch worktree with its own `CARGO_TARGET_DIR` under your scratchpad, and both
are gone before you finish.

## Read first
1. The spec on the PR branch, in full:
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-155-a-write-that-fails-must-not-report-success-or-destroy-the-old-file.md`.
   Read these sections:
   - `## Frame`, especially Q2 (every writer), Q3 (the fix's shape), and the maintainer rulings;
   - `## Design`: its nine corrections and "Things the frame asked design to check";
   - the ACs, Failing Tests, the baseline and the mutation table;
   - `## Implementation Context`: the code shape, the messages, the DEC text, the docs, the CHANGELOG,
     the Windows reasoning, out of scope;
   - Build Completion.
2. `src/sink/publish.rs`, and the diff of `src/sink/mod.rs`, `src/cli/ops.rs`, `src/cli/build.rs` and
   `src/cli/mod.rs` against `main`.
3. `tests/atomic_write.rs`.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this
   wave paid for", especially traps 10, 12, 13 and 14.
5. AGENTS §15's rules: a bound or "can't happen" is a claim until its worst input is driven, and
   every finding is classified.

## Maintainer rulings the build had to honour
1. `-y` into a read-only directory over a writable file is refused, exit 5, naming the directory. No
   in-place fallback.
2. No `fsync` and no directory `fsync`. The guarantee covers errors and killed processes, not power
   loss, and the DEC says so.
3. `--save-recipe` gains the overwrite guard, checked before the run.
4. A full disk fails loudly; staging needs room for both files.
5. `write_lock` moves onto the helper. A replaced file keeps its permission bits, not its owner, ACLs,
   xattrs or hard links, and `-y`'s docs say so.
6. No message, comment or doc line names a spec, stage or DEC id.

## Check, in order
1. **Scope.**
   - Compare `git diff --name-only origin/main...HEAD` with `## Outputs`.
   - Confirm `Cargo.toml`, `Cargo.lock`, `CACHE_SCHEMA_VERSION` and the lockfile format didn't
     change.
   - Check that every commit is signed off and that the PR merges cleanly onto current `main`.
   - SPEC-156's draft PR #269 edits `CHANGELOG.md`, `docs/api-contract.md` and
     `docs/cli-reference.md` too. Run `git merge-tree` on the two heads and say whether they conflict,
     and where.
2. **The code against the spec's code shape.** Walk `publish` against the spec's numbered steps,
   one by one, and say whether each matches:
   - the guard order: symlink, then overwrite, then the write-probe, then staging;
   - the retry bound;
   - the name cut;
   - when the remover is disarmed;
   - what each error maps to.

   Do the same for each of the four sink arms, `run_edit`'s `--save-recipe`, and `write_lock`. Any
   difference from the spec is a finding, even if every test passes.
3. **Every writer, independently.** This is the check the tests can't make.
   - Re-run the frame's Q2 sweep on the PR head: both greps, verbatim, over `src/`.
   - List every production call that writes a file a user named or a build produces, and say whether
     each now goes through `publish_file` or `publish_file_io`.
   - Name any writer the frame didn't list, including any added on `main` since the frame.
   - The build cache (`src/build/cache.rs`) is out of scope by design. Confirm it is untouched.
4. **Drive the behaviour yourself.** Build `main` and the PR head `--release`, each in its own fresh
   `CARGO_TARGET_DIR`. Don't reuse the build's binaries or its test file. Use `ulimit -f 1` with XFSZ
   ignored, unless a row says otherwise. Give exit code, stderr and the old file's bytes, on both
   binaries:
   - a small and a big output for each of `convert -o`, `convert --out-dir`, `meta strip -o`, and one
     `optimize` or `responsive` output. These are surfaces the end-to-end rows don't all cover;
   - a `build` whose output write fails;
   - XFSZ at its default (killed). Say what is left on disk: the old file, and any staged
     `.crustyimg-` file. Then check that the DEC and docs describe that leftover honestly;
   - `-y` over a symlinked destination, on both binaries: still refused (DEC-035);
   - `-y` over a file with a second hard link: what happens to the other link, and whether `-y`'s docs
     say so;
   - `-o -` (stdout): unchanged.
5. **Did the tests exercise what they claim?**
   - From the Linux and macOS `build / test / clippy / fmt` logs at the head you verify (one read
     each), count the `atomic_write` rows that ran, failed or were ignored. Do the same for the
     `publish` unit rows.
   - From the Windows log, quote W1–W4, and say which E rows are `cfg(unix)` and so never ran there.
   - Confirm that the root-sensitive rows (E11, E15, E19, E21) assert their precondition, and that the
     ulimit helper's limit really applies. Prove it with one row driven red on `main`.
6. **Mutations.** Each alone, in a scratch worktree, with a clean rebuild, a full
   `cargo test --no-fail-fast`, and the red set read off the run:
   - **The disputed cells:** M1, M2, M3. Does FT-E23 go red, and for what reason? Also M11's 48.
   - **The rows the claims rest on:** M2 (the split between U and E rows), M9 and M10.
   - **Two rows the table doesn't have:**
     - (a) `staged_name` without the `.tmp` suffix. Design correction 1 calls the suffix load-bearing,
       but FT-E20 plants a literal `.tmp` file, so does any row see the change?
     - (b) The name cut at a byte, not a char boundary. FT-U9 is all ASCII; drive a name with a
       multi-byte character across byte 100, both with the code as built and with the mutant.

   A mutant no row sees is a finding. Classify it **assert**.
7. **Every message, and the DEC and docs, against the spec.**
   - Check each message in the spec's Messages list against the code with a script, not by eye.
   - `diff` the DEC file with the spec's final DEC text, after removing the quote markers. Do the same
     for every docs block and the CHANGELOG lines, allowing only the orchestrator's correction.
   - For each docs sentence that states behaviour, give the step-4 row (or your own drive) that shows
     it true.
   - Run `/usr/bin/grep -nE '(SPEC|STAGE|DEC|PROJ)-[0-9]'` over the changed lines of every non-spec
     file. Classify each hit: user-facing text, or test data. `DEC-116`'s own filename and the
     `decisions/` cross-references are expected.
8. **wasm.** Read the `process_id` cfg, and say whether any wasm path can reach the helper. Read the
   wasm legs on CI; don't re-run them.
9. **CI.** Run `gh pr checks 268` once at the head, and say whether every leg is green, including
   Windows.

## Classify every finding (AGENTS §15)
For each punch item and optional cleanup, say which kind it is:
- **wording:** a doc, comment or spec line that misstates something the code already does;
- **assert:** a missing or loose test row, where the behaviour is already right;
- **claim:** a fix that would change what a doc or test claims is true, or what the code does.

Only a **claim** needs a re-verify after its punch list.

For every bound, count or "can't happen" in the spec, say whether it was driven, by whom, and over
what range. Examples:
- the 8 KiB line;
- "at most 16" retries;
- `NAME_KEEP = 100` under the 255-byte limit;
- "a staged file lives for microseconds";
- "no handle crustyimg holds is open at the rename";
- "`FailingFs` hands `RealFs` at most 1,024 B a call";
- "the helper is unreachable on wasm".

## Guardrails
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters, so loop with `while IFS= read -r`. macOS has no `timeout(1)`.
  `rtk` can truncate `grep` and `git log`, so use `/usr/bin/grep` and `/usr/bin/git` for anything
  you count.
- Redirect `just check` and `cargo test` stdout to a file. An interactive terminal fails
  `display_sink_refuses_non_tty`.
- Every build uses a fresh `CARGO_TARGET_DIR` under your scratchpad, one per binary or mutant, never
  shared with another session. A stale incremental build is a false green: `cargo clean -p crustyimg`
  between mutants.
- Never poll CI and never write your own wait loop (AGENTS §16 habit 6). Don't use `pgrep -f`: it
  matches its own shell.
- Write scratch files only in your session's scratchpad, never `/tmp` or the system temp dir.
  Restore any `chmod` you make before you delete a scratch folder, and leave the tree clean.
- **Budget ~50 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `/usr/bin/grep -l "SPEC-155 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
  `output_tokens`. Include any subagent transcripts under the session's `subagents/` folder.
- Price each component at Opus's $5/$25 per MTok, with cache_creation at ×1.25 and cache_read at
  ×0.10. Measure at your last turn. You're read-only, so report the numbers; a later session writes
  them into `cost.sessions`.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only, no box-drawn tables. The first line is exactly one of:
- `✅ APPROVED — merge PR #268 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #268 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–9, one or two lines each, with evidence;
- the punch items, numbered. Give each its kind (wording / assert / claim), its evidence, the fix
  you'd expect, and whether it lands on the PR or on `main`;
- optional cleanups, listed separately, each with its kind;
- the mutation table's disputed cells: your ruling on each;
- out-of-spec findings, each with where it belongs (STAGE-063 or `guidance/ideas.yaml`), and
  whether each one is pre-existing;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
