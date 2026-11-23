# SPEC-157 — BUILD prompt: `build` destinations have one identity

Cycle: **build**. Run it in a fresh CLI session, in its own worktree, on **Sonnet**. The design gives
the change in code terms (all of it in `src/cli/build.rs` unless noted):
- the resolver: `build_root`, `fold_case`, `resolve_out_dir`, `root_relative`, `join_rel`,
  `first_link`, and `OUTPUT_EXTENSIONS`;
- `check_destinations`, which runs out-of-tree, then collision, then overlap, against a read set
  from `reads_of`;
- `write_planned`, which creates the folder, resolves it again, and writes through `Sink::File`'s
  arm of `write_bytes`;
- the lockfile path from the build root;
- the `--watch` exclusion and its read carve-out;
- three exit-2 error variants in `src/cli/mod.rs`.

It also gives every message verbatim, 24 ACs, 34 failing tests and a 14-row mutation table,
**every row driven at design on a prototype** except the Windows ones. The code is about 360 lines
in `build.rs` and 40 in `mod.rs`; the tests and mutations are most of the work. Be thorough across
every row: Sonnet's known weak spot is sweep thoroughness.

```
just wt feat/spec-157-build-destinations -- --model sonnet
```

You did not design this; the spec carries the context (AGENTS §15). The maintainer pastes this
prompt into a fresh session and pastes your final message back to the orchestrator, so that message
must stand on its own. Keep it plain text with short lists; a wide box-drawn table can garble when
copied out of the terminal.

## Before you start: SPEC-155 must have merged
SPEC-155 (all-or-nothing writes) also changes `src/cli/build.rs` and `src/sink/mod.rs`. Your
`write_planned` calls `Sink::File`'s arm of `write_bytes`, which after SPEC-155 calls
`publish_file(path, bytes)` and stages in `path.parent()`, the folder you resolved. At Step 0, check
that SPEC-155's PR is merged into `main` (`gh pr list --state merged --search "SPEC-155"`). If it
isn't, stop and report; don't start.

## What you are building
- **Planning:** every target's `out` is resolved once, before the lockfile load, the cache open or
  any write:
  - drop `.`;
  - canonicalize the longest existing prefix;
  - add the rest with `..` applied lexically;
  - measure from the canonical working directory.

  `PreparedTarget` gains `out_dir` and `rel_dir`. `prepared.out_dir` is set only by
  `check_destinations`, and `build_one` never falls back to `target.out`.
- **Refusals, all exit 2, nothing written:**
  - an `out` resolving outside the root through a link;
  - two outputs on one case-folded path (the existing collision message, word for word, with the
    resolved path);
  - an output that is a file the build reads: any target's source, a recipe, a watermark image or
    font, the manifest, the lockfile;
  - an output under `.crustyimg/`;
  - chaining.

  `{ext}` names are checked under every extension in `OUTPUT_EXTENSIONS`, or only the pinned one when
  the format is pinned.
- **Case:** `str::to_lowercase()` on every OS, with no new dependency. It refuses `a.png`/`A.png` on
  Linux too. That is the ruling, and the Linux case rows assert it.
- **The sink re-check:** `write_planned` resolves the folder again after `create_dir_all`, and refuses
  with exit 5 (existing text) if it moved. `write_encoded` stays for `apply`.
- **The lockfile** records the root-relative path. An old lockfile drifts once, on paths only.
- **`--watch`:** the resolved `out` is excluded, and an event on a file the build reads always
  rebuilds. `watch_reads` and `is_read` are `#[cfg(feature = "watch")]`. `src/build/watch.rs` doesn't
  change; keep `watch_roots` lexical.
- **Docs, `--help` and CHANGELOG,** verbatim from the spec.

**The new DEC and DEC-061's amendment are ship's, not yours.** Don't create the DEC file or edit
DEC-061. The spec's `## Outputs` is the complete file list for this build. Anything outside it is a
deviation; say why.

## Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. Then check:
- SPEC-155 has merged (above), and `Sink::File`'s arm of `write_bytes` in `src/sink/mod.rs` calls
  `publish_file`. Quote the line;
- the spec is at `cycle: build`, and design commits `47650bf` and `05d8241` are in your history;
- the functions the spec cites still exist: `check_output_injective`, `output_collision_key`,
  `lock_output_path`, `lock_record`, `watch_set_for`, and the two `write_encoded` calls in `build.rs`.
  SPEC-155 changed `write_lock` in the same file, so line numbers have moved. Find each by content.

If any check fails, stop and report.

⚠ **Other open work touches nearby lines.** The spec's "Overlap with other open work" lists it from
the PRs' file lists:
- #226: `src/cli/mod.rs`, the module doc only;
- #237: `CHANGELOG.md`; keep both entries.

SPEC-156's build may be open: it touches `src/metadata/`, `tests/metadata.rs`, two docs files and
`CHANGELOG.md`. After every rebase, run `gh pr list --state open --json number,title,files` and
compare; name any new overlap in Build Completion. If a rebase conflicts in a file outside the spec's
list, stop and report.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-157-build-destinations-have-one-identity.md`.
   Read it in full:
   - `## Frame` with the maintainer rulings;
   - `## Design`, especially "Design corrections to the frame" and "What was driven";
   - every AC, the Failing Tests, the baseline and the mutation table;
   - `## Implementation Context`: the design calls, the messages, the DEC and amendment text (ship's),
     the docs, the CHANGELOG, the overlap, "What the build prompt must carry";
   - `## Notes for the Implementer`.

   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in your
   report; don't quietly re-decide it.
2. **DECs:** DEC-010, DEC-035, DEC-057, DEC-059, DEC-061 and DEC-106, and the DEC SPEC-155's build
   created ("a file write is all or nothing"). Read them yourself.
3. SPEC-155's spec in `specs/` or `specs/done/`, for `publish_file`'s contract.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first" and "Traps this wave
   paid for" (traps 10 to 14). Also `/guidance/constraints.yaml`.

## Maintainer rulings (settled — do not re-open)
1. A symlinked `out` resolving outside the build root is refused. In-tree links work.
2. Chaining is refused.
3. Case-folded comparison on every OS.
4. Exit 2 for every new refusal.
5. A CHANGELOG note covers the one-time lock path drift.
6. No message, help line or comment names a spec, stage or DEC id.

## Order, with a pushed checkpoint after each stage
A long build with no pushed commit has been lost before. Commit with `git commit -s` and push after
each numbered stage. Stages 1–4 are the spec's four checkpoints.
1. **The RED baseline.** Write `tests/build_destinations.rs` and the two `tests/build_watch.rs` rows,
   and run them against `main`'s code (src untouched), stdout redirected and `$?` read. Expected, per
   the spec's baseline:
   - red **for their reason**: FT-1 to FT-4, FT-6 to FT-18, FT-21 to FT-26, FT-29 and FT-30. FT-22
     fails at its `assert_ne`;
   - green on `main`: FT-5, FT-19, FT-20 and FT-27;
   - FT-28 and FT-31 to FT-34 don't compile yet, and that is their red.

   A row red for another reason (a fixture that won't decode, a watermark that doesn't fit) is a
   broken row: fix the row, not the code. Quote each line. Open the PR **as a draft** with this
   commit.
2. **The resolver, `check_destinations` and the three variants.** FT-1 to FT-20 and the unit rows
   green.
3. **`write_planned`, the lock path and `--watch`.** FT-21 to FT-30 green. Read `refused()`'s
   snapshot code: it must record a link as a link and never follow `link` in FT-25.
4. **Docs, `--help` and CHANGELOG.**
   - Write each doc row at the `file:line` the spec gives, found by content.
   - **Drive every behaviour sentence on every exit path it names**, using the spec's per-sentence
     list (exit 2, exit 5, exit 0, exit 7). Quote each drive in Build Completion.
   - Also drive the upgrade on the binary: build with `main`'s binary through an in-tree link, then
     run this branch's `--check` (exit 7 once), one plain build, then `--check` again (exit 0).
5. **Mutations, one at a time** (HANDOFF traps 12 and 14), in a scratch `git worktree` of your
   pushed head with its own `CARGO_TARGET_DIR`, never in the build worktree. The spec quotes each
   mutant's exact edit: **apply that edit, not your own version of it.** If the quoted line doesn't
   exist in your code, make the closest edit, quote what you applied, and say why. For each row:
   - apply it, and check `git diff` is non-empty;
   - rebuild **clean** (`cargo clean -p crustyimg`), because a reverted source doesn't rebuild the
     binary;
   - run the full suite with `--no-fail-fast`, stdout redirected and `$?` read, and read the red set
     **off that run**. Compare it with the table and call out every test the table doesn't list;
   - **M11** (root not canonicalized) is an equivalent mutant on macOS and Linux, by design. Drive it
     and say whether it's still all green;
   - **Trap 14:** restore `main`'s `src/cli/build.rs` verbatim, with SPEC-155's change as merged, and
     list what goes red against the table;
   - restore, rebuild, and show it green.

   Paste each mutation's red output, trimmed to the lines that name tests, into Build Completion.
   Where a cell is wrong, correct it in Build Completion and say so; don't edit the design's table.
6. **Clean matrix:**
   - default, `--no-default-features` (which compiles out `--watch`), and `--features webp-lossy`,
     one after another, each in a fresh `CARGO_TARGET_DIR`, with clippy `-D warnings` and
     `fmt --check` on each;
   - then `just wasm-check` once.

## The things most likely to go wrong
- **Reading the lock or opening the cache before the refusals.** M7 moves them late, and 24 rows go
  red at "leave every file as it was". A refused build, `--check` included, writes nothing.
- **Following the file name.** The resolver resolves only the `out` folder. The output file name is
  joined as one component and never followed, so a link planted at the file name is still the sink's
  DEC-035 refusal (exit 5).
- **`{ext}`.** Overlap compares every extension the sink can write unless the format is pinned.
  Check `OUTPUT_EXTENSIONS` against what the sink really writes (AC-21).
- **The collision message.** Keep it word for word; existing tests pin it. Only the path inside it
  changes.
- **Symlinks in tests.** Every link and every out-of-tree target stays inside the test's temp dir.
  Never point a link at `~` or a system path.
- **`--watch` rows.** Each watcher in a test has a fixed lifetime and is killed at the end. Count
  builds in a window; don't wait on an event that might not come.
- **Windows.** FT-34 and the case rows on NTFS are reasoned, not driven. **The Windows CI leg is
  their only evidence.** Quote its result for each.
- **Retyped messages.** Copy every message from the spec byte for byte.
- **A green leg on one OS isn't the matrix.** Read every CI leg, including lean, msrv, Windows and
  wasm.

## Guardrails
- No version bump, no release, no tag. No change to `CACHE_SCHEMA_VERSION`, the lockfile's version,
  `Cargo.toml` or `Cargo.lock`. Paste `git diff --stat origin/main...HEAD` for those paths.
- No DEC file, no DEC edit, and no change to the spec's timeline. Ship and the orchestrator keep them.
- **Every commit uses `git commit -s`**, ending with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`. The DCO check fails a PR with an
  unsigned commit.
- **Never poll CI and never write your own wait loop** (AGENTS §16 habit 6). Run
  `gh pr checks <PR> --watch` once, in the background, after your last push, and take one snapshot
  when it exits. If it exits with legs still pending, say so; don't call that green.
- **Rebase onto `main` before each push** and redo Step 0's overlap check after each rebase. Don't
  chase `main` beyond that.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout to
  a file. A piped command reports the **pipe's** exit code, so redirect and read `$?`. Gate each commit
  on its test run; don't chain `git commit` after `cargo test` with `;`. zsh does not word-split
  unquoted parameters, so loop with `while IFS= read -r`, and it reads `$R:r…` as a modifier, so brace
  it (`${R}:`). macOS has no `timeout(1)`. `rtk` can truncate `grep` and `git log` output, so use
  `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Delete each scratch target dir and the
  mutation worktree when you're done, and leave no watcher running. Write scratch files in your
  session's scratchpad, never `/tmp` or the system temp dir, and don't leave PR-body files behind.
- **Budget ~120 exchanges.** If you're past budget with the matrix green, stop and report what's
  left.

## When you finish
1. **Fill in `## Build Completion`:**
   - every file, from `git diff --name-only origin/main...HEAD`, not from memory, each with a phrase on
     what changed; give the count and say whether it includes the spec file;
   - the RED baseline, quoted;
   - every AC, one line each: met, or not and why;
   - each behaviour sentence's drive on each exit path, and the upgrade drive, quoted;
   - every mutation row, with the edit you applied, the tests that went red and those that stayed
     green, against the spec's table, with every mismatch called out;
   - the matrix legs, the wasm check, and the CI legs, with FT-34 and the case rows quoted from the
     Windows leg;
   - the overlap check after your last rebase;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost** at your **last** turn, after the last push.
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `/usr/bin/grep -l "SPEC-157 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`. Include any `subagents/`
     transcripts.
   - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per MTok,
     Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read is ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
3. **Advance, and leave the PR as a draft.**
   - Run `just advance-cycle SPEC-157 verify`, then commit and push.
   - ⛔ The PR stays a **draft**. A separate verify session approves it, and the orchestrator marks it
     ready. Never mark it ready yourself.
   - Title: `fix(build): one identity for every destination, resolved before anything is written (SPEC-157)`.
   - Body per AGENTS §13:
     - Summary: `build` could overwrite its own sources, recipe or watermark, write two outputs to one
       file and record two lock entries, write outside the project through a link, and rebuild in a loop
       under `--watch`, all with exit 0; each `out` is now resolved before anything is written, and
       those builds are refused with exit 2;
     - Spec metadata;
     - Decisions referenced: DEC-010, DEC-035, DEC-057, DEC-059, DEC-061, DEC-106, and the
       all-or-nothing write DEC;
     - New decisions: one, written at ship (a build destination has one identity), plus DEC-061's
       amendment;
     - Constraints checked;
     - Breaking: out-of-tree `out` links refused; chaining refused; case-folded collisions on every OS;
       lockfiles with odd `out` spellings drift once;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-157 build: draft PR #NNN at <head SHA> — CI <green | red: legs | pending>`.
Then, briefly, as plain lists:
- the RED baseline on `main`, against the spec's expected sets;
- AC completion (as a count, plus any row not met);
- each behaviour sentence's drive, one line per exit path, and the upgrade drive;
- each mutation's flips against the table, the edit you applied where it differs from the quoted one,
  and every cell that turned out wrong;
- the matrix result, `just wasm-check`, and the CI legs, including lean, msrv, Windows (FT-34 and the
  case rows quoted) and wasm;
- the overlap check after your last rebase;
- files changed against the spec's `## Outputs`;
- any design call that turned out wrong, and any out-of-spec finding;
- your measured cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its
  own line.
