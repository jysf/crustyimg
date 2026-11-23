# SPEC-155 — PUNCH LIST prompt: where the mode comes from and when it's applied, and two staged-name rows

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. Two **claim** items, two
**assert** items and three wording items. Because two items are claims, a narrow re-verify follows
your report. You did not design, build or verify SPEC-155.

```
just wt fix/spec-155-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-155-atomic-writes:refs/remotes/origin/feat/spec-155-atomic-writes
git checkout --detach origin/feat/spec-155-atomic-writes
```
Push to the PR branch with `git push origin HEAD:feat/spec-155-atomic-writes`. Draft PR #268's head
was `471a0a7` when this was written. If it has moved, stop and report.

The maintainer pastes this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own. **Write it as plain lists, not box-drawn tables.**

## Why
Verify drove every failure path on `--release` builds of `main` and the PR. It re-ran the writer
sweep (no writer missed), re-drove M1, M2, M3, M9, M10 and M11, and read every CI leg; all green. It
found two regressions in the spec's own `publish` steps, which the build implemented exactly as
written, and two mutants no test sees:

1. **The copied mode comes from a symlink, not the file.** `publish` takes `old_perm` from
   `std::fs::symlink_metadata(path)` (`src/sink/publish.rs:141`). The four sink arms and
   `--save-recipe` refuse a symlinked destination first. `write_lock` has no such guard. Verify
   drove two cases, with `crustyimg.build.lock` as a symlink:
   - **To a `0600` file:** the PR writes a `0755` regular file, where `main` wrote `0644`. On Linux the
     link's mode is `0777`, so it would be `0777`; that part is reasoned.
   - **Dangling:** the PR exits 5 (`could not write build lockfile …: No such file or directory`), where
     `main` exits 0. The write-probe opens through the link without `create`.
2. **A private file's new bytes are world-readable while they are written.** The staged file is
   created at `0666 & ~umask`, and the old mode is applied only after the write (`:189-192`).
   Driven: `-y` over a `0600` file, killed by XFSZ, leaves `.priv.png.crustyimg-*.tmp` at `0644`
   holding the new bytes. `main` truncated in place and kept `0600`.
3. **The char-boundary cut has no row.** A mutant that cuts the name at byte 100 instead of a char
   boundary stays green. On a real name (99 `a`s, then `é.png`) it panics, exit 101:
   `end byte index 100 is not a char boundary`. The built code writes that name, exit 0.
4. **The `.tmp` suffix has no row.** Dropping it from `staged_name` leaves 0 tests red. FT-E20 plants
   a hand-written `.tmp` name, so it never sees the names the code makes.

## The fixes (settled, do not re-decide)
1. **Mode from the file, and never through a symlink.** In `publish`'s step 2:
   - When `symlink_metadata` says the destination is a **symlink**, skip both the write-probe and the
     mode copy. The rename then replaces the link with a new regular file at the umask mode. That is
     what `main`'s `write_lock` did, so the dangling case exits 0 again.
   - Otherwise keep the probe, and take the mode from the same metadata, which is now known to be the
     file's own.

   Add one end-to-end row per case, `cfg(unix)`, in `tests/atomic_write.rs`. Each builds the project
   the way FT-E21 does:
   - a lockfile symlinked to a `0600` file: `build` exits 0, the lockfile is a regular file, and its
     mode is the umask mode (assert `0o644` with umask `022` set in the child), not `0755`/`0777`;
   - a dangling lockfile symlink: `build` exits 0 and writes a regular lockfile.

   Check whether any sentence in the DEC or the docs claims the symlink guard covers the lockfile.
   If one does, narrow it, and update the spec's verbatim copy to match.
2. **Apply the old mode before the first byte is written.** Move the `set_permissions` call to
   straight after `create_new` succeeds, before the write loop. Keep it on the `StageFs` seam and
   keep its error as `PublishError::Write`.
   - Add a row that fails if the mode arrives late. Preferred: a unit row with a `StageFs` wrapper
     that, at its first `write`, reads the staged path's mode and records it. Assert it equals the old
     file's `0600`.
   - Also drive the end-to-end case by hand and quote it: `-y` over a `0600` file under `ulimit -f 1`
     with XFSZ at its default. The leftover staged file must be `0600`.
3. **A row for the char-boundary cut.** Add `FT-U9b`: a name whose multi-byte character straddles
   byte 100 (99 `a`s, then `é`, then `.png`). `publish` must write it. Assert that `staged_name` of
   that name keeps a prefix of at most 100 bytes that is valid UTF-8.
4. **A row for the suffix.** Add a unit row in `src/sink/publish.rs` that ties the two halves
   together:
   - `staged_name(…)` starts with `.` and ends with `.tmp`;
   - the source resolver doesn't treat that name as an image. Use whatever `crate::source` exposes
     at `pub(crate)` or wider, for example `has_image_extension`. If nothing is reachable, assert only
     the suffix, and say so.

   FT-E20 stays as it is. It is an integration test, so it can't call `staged_name`, which is
   `pub(crate)`.
5. **Wording.**
   - `PublishError::CreateTemp`'s doc says "sixteen staged names in a row were taken". The code tries
     17 (16 retries). Make the doc say what the code does.
   - In `SECURITY.md`, indent the three lines inserted under item 3 so they sit inside that list item.
   - In `docs/cli-reference.md`'s "Replacing a file" note, add one sentence: a process that is killed
     mid-write can leave a `.<name>.crustyimg-<…>.tmp` file beside the destination, and it is safe to
     delete. Don't touch the CHANGELOG for this.
   - In every case, update the spec's verbatim copy to match, byte for byte.
6. **The spec's mutation table.** Edit it in place, marking each change `(corrected at punch list)`:
   - add FT-E23 to M1's, M2's and M3's Red lists;
   - make M11's count 48 and name U9 and U11;
   - trap 14a: 3 distinct errors, each reported for the lib and for the lib test;
   - M9's edit: `std::fs::write`;
   - add the new rows, each driven alone:
     - **M12:** `old_perm` taken through a symlink again (the case check removed). It must turn red
       item 1's two new rows.
     - **M13:** `set_permissions` moved back after the write. It must turn red item 2's row.
     - **M14:** the byte cut. It must turn red FT-U9b.
     - **M15:** the `.tmp` suffix dropped. It must turn red item 4's unit row. FT-E20 stays green;
       say so in the row.

   Add the new tests to Failing Tests and the ACs they serve (AC-8 for the mode, AC-14 for the
   lockfile, AC-17 and AC-18 for the names).
7. No spec, stage or DEC id in any changed line outside the spec.

Stay in these files:
- `src/sink/publish.rs`;
- `tests/atomic_write.rs`;
- `docs/cli-reference.md`, the "Replacing a file" note only;
- `SECURITY.md`, the indentation only;
- the DEC file, only if fix 1's check finds a sentence to narrow;
- the SPEC-155 spec.

`git diff --stat 471a0a7` must show nothing else. **`src/sink/mod.rs`, `src/cli/` and `CHANGELOG.md`
must not change.**

**Not yours:**
- verify's out-of-spec finding (`optimize`'s "kept the source unchanged" note), which the orchestrator
  has filed on `main`;
- the timeline;
- the PR's draft state.

## Check it
- **Before each fix**, drive its new row against the current head, and quote that it is red:
  - item 1: the `0755` mode, and the dangling-link exit 5;
  - item 2: `0644` at the first write;
  - items 3 and 4 are green today by design: they are pins, and their mutants make them red.
- Run M12–M15, each alone, in a scratch worktree with its own `CARGO_TARGET_DIR`:
  `cargo clean -p crustyimg`, then a full `cargo test --no-fail-fast` with stdout redirected, and
  read the red set off the run. Also re-run M4 (mode not copied) and M8 (disarm before rename),
  because item 2 moves code next to both. Each must still be red where the table says.
- Run the matrix, each leg in a fresh `CARGO_TARGET_DIR`: `fmt --check`,
  `clippy --all-targets -D warnings` and the full test, on default, `--no-default-features` and
  `--features webp-lossy`. Then `just wasm-check`.
- Drive each changed doc sentence and quote the result.
- CI runs on push. Windows has no symlink or mode rows (`cfg(unix)`), but check its leg stays green,
  because the call order changed on every OS.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag.
- If the PR is behind `main`, rebase onto `main` before you push. Never force-push over someone
  else's commit: if the PR's head isn't `471a0a7` when you start, stop.
- Push after each fix lands green locally (checkpoints), not only at the end.
- **Never poll CI.** Run `gh pr checks 268 --watch` once in the background after your last push, and
  don't write your own wait loop. Don't use `pgrep -f`. When it exits, take one snapshot at the PR's
  head SHA, every leg.
- A piped command reports the pipe's exit code, so redirect and read `$?`. zsh does not word-split
  unquoted parameters. `rtk` can truncate `grep`, so use `/usr/bin/grep` for anything you count.
- Redirect `cargo test` stdout to a file. An interactive terminal fails
  `display_sink_refuses_non_tty`.
- Write scratch files in your session's scratchpad, never `/tmp` or the system temp dir. Restore any
  `chmod` before you delete a scratch folder.
- **Budget ~45 exchanges.**

## When you finish
- Add a `### Punch list (verify)` subsection to the spec's `## Build Completion`. It holds:
  - each fix with its red-before and green-after evidence;
  - M12–M15, M4 and M8, each with its red set;
  - the matrix;
  - the CI snapshot.
- Set the spec's `cycle` back to `verify`.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `/usr/bin/grep -l "SPEC-155 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
    `output_tokens`.
  - Price at the model in `.message.model`: Sonnet $3/$15 per MTok, Opus $5/$25. cache_creation is
    ×1.25 the input rate and cache_read ×0.10.
  - Append a `punchlist` entry to the spec's `cost.sessions` with `tokens_total`, a
    `tokens_breakdown`, `estimated_usd` and the rate anchors. Also record verify's cost there as a
    `verify` entry, from these numbers: input 82, cache_creation 148,361, cache_read 5,536,277,
    output 30,974, total 5,715,694, $4.47, Opus.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-155 punch list: PR #268 at <head SHA> — CI <green | red: legs>`.

Then, briefly:
- `git diff --stat 471a0a7`, and that `src/sink/mod.rs`, `src/cli/` and `CHANGELOG.md` are unchanged;
- each fix: the code change in one line, the new rows' names, red before and green after;
- M12–M15, M4 and M8, each with its red set;
- each changed doc or DEC sentence, quoted as it now reads, with its drive;
- the matrix and the CI snapshot;
- anything that did not go as written;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
