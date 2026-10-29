# SPEC-138 — BUILD prompt: `build` refuses the output flags its manifest owns

Cycle: **build**. Fresh CLI session, own worktree, **Sonnet**. The design gives the exact
messages, every test and a mutation table, so the work is mechanical. The job is to be thorough
across the rows and to prove the build path's bytes did not move.

```
just wt feat/spec-138-build-refuses-output-flags -- --model sonnet
```

You did not design this. The spec carries the context (AGENTS §15). The maintainer pasted this
prompt into a fresh session and will paste your final message back to the orchestrator, so that
message must stand on its own. This is one of the two specs that open **0.8.1**; SPEC-137 builds
before you.

## What you are building
- `build` and `build --watch` refuse `-o`/`--output`, `--out-dir`, `--name-template` and
  `--format` with exit 2, before the manifest, the lockfile or any input is read. Nothing is
  written.
- One guard in `src/cli/mod.rs`'s dispatch, after the existing `--watch` guard and before the
  verb `match`. Every given flag gets its own stderr line, in a fixed order, with one exit 2.
- Every other `build` flag keeps working, and `apply` still honours all four.
- `build`'s outputs, cache keys and lockfile are unchanged.

## Step 0 — get onto current `main`
`git fetch origin && git rebase origin/main`. Then check all three:
- the spec is at `cycle: build`;
- its design commit `4db475c` is in your history;
- its `### Maintainer rulings` list goes up to **ruling 6**. Rulings 3–6 were added after design.

If any of those fails, stop and report.

If SPEC-137's PR (`fix(svg): …`) has merged by the time you open yours, rebase over it. Both
specs add a `CHANGELOG.md` entry under `## [Unreleased]`, and both touch `README.md` and
`docs/cli-reference.md`; on a conflict, keep both entries.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-138-build-refuses-the-output-flags-its-manifest-owns.md`.
   Read all of these in full:
   - `## Frame` and its rulings 1–6;
   - `## Design`: the RED baseline and Calls 1–8, including `compare-builds.sh` in Call 6;
   - all 17 ACs;
   - `## Failing Tests` (FT-1 to FT-15) and the mutation table;
   - `## Implementation Context`;
   - `## Notes for the Implementer`.

   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in
   your report. Do not quietly re-decide it.
2. **DECs:** DEC-057, DEC-058, DEC-105, DEC-087, DEC-099, DEC-101 and DEC-015. Read them; do not
   trust a summary.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`, "Read first" and "Traps this
   wave paid for". They are binding. Also `/guidance/constraints.yaml`.

## Maintainer rulings (settled — do not re-open)
1. **Refuse**, don't honour, `build --format`.
2. **All four flags**, through one guard, on `build` and `build --watch`.
3. **The `-o` message names `-o/--output`**, whichever spelling was typed, following the existing
   `-q/--quality` message. No argv re-parse.
4. **No committed golden lockfile.** The two-binary `compare-builds.sh` and the empty
   `git diff --stat` replace it.
5. **DEC-057's stale "output format follows the source" bullet** is fixed in the same amendment.
6. **All 15 tests ship**, FT-6 included.

## Order
1. **Every failing test first, against unmodified `src/`.** Write
   `tests/build_refuses_output_flags.rs` (FT-1 to FT-14) and FT-15 in `src/cli/mod.rs`'s
   `#[cfg(test)]` module. Record, test by test:
   - the 12 RED rows, each **for the reason the spec gives**, with `main`'s exit code and what
     `main` wrote (exit 0 and an output file, exit 3, exit 7, or a parse message);
   - FT-13 and FT-14 GREEN (the controls);
   - FT-15's state as the spec describes it.

   ⛔ This commit changes nothing in `src/` outside the `#[cfg(test)]` module. Verify checks it
   with `git show <commit 1> -- src/`.
2. **Implement** the guard and its private helper exactly as the spec words them. Push a WIP
   commit as soon as it compiles (`git commit -s`, then push).
3. **Run the mutation table, one row at a time.** For each one: apply the mutation, check
   `git diff` is non-empty, rebuild **clean** (fresh `CARGO_TARGET_DIR` or
   `cargo clean -p crustyimg`), run the named tests, and record exactly which go RED. Then
   restore, rebuild, and show them green. ⚠ Reverting source does not rebuild the binary.
4. **Byte identity (Call 6).** Paste both into Build Completion:
   - `git diff --stat origin/main...HEAD -- src/build/ src/cli/build.rs`, which must be empty;
   - the output of `compare-builds.sh` exactly as the spec gives it, comparing the merge-base
     binary with yours. It must print `IDENTICAL`.
5. **Docs and record,** worded in the spec:
   - `docs/cli-reference.md` (the build section and the global-flag table's `--format` row) and
     `docs/api-contract.md:626` (one clause). `README.md` needs no change: the design checked it;
   - the `CHANGELOG.md` entry under `## [Unreleased]` → `### Breaking` (Call 7). Leave the
     released 0.8.0 lines as they are;
   - the DEC-057 amendments (both bullets) and DEC-105 rule 2, exactly as Call 8 words them. Run
     `./scripts/decisions-audit.sh` afterwards. No new DEC.
   - No spec or DEC ids in messages, comments or the CHANGELOG.
6. **Clean matrix,** then the PR.

## The things most likely to go wrong
- **"Nothing written" is a file snapshot,** not an exit code: no output file, no `.crustyimg/`
  entry, the lockfile's hash unchanged. FT-13 proves the snapshot can see a write. If FT-13 is
  not green, every "nothing written" row is hollow.
- **FT-11 and FT-12 must not use `.output()`.** It waits forever when the child watches. Spawn,
  poll `try_wait` with the spec's 20 s deadline, and kill the child on timeout.
- **Detection is `is_some()`.** All four are `Option<String>` with no clap default. Don't add a
  `default_value` to any of them; FT-15 goes red if you do.
- **The order of lines is fixed** (`-o`, `--out-dir`, `--name-template`, `--format`), whatever
  order they were typed in. FT-7 tests it.
- **Error order:** the guard sits before the `match`, so a refused flag wins over a missing
  manifest (exit 3 today), a stale `--check` (exit 7 today), and the `--watch --check` conflict.
  It goes **after** the existing `--watch` guard. Read the spec on why.
- **No changes to `Cargo.toml`, `Cargo.lock`, the version or `CACHE_SCHEMA_VERSION`.**
- **Stay in your files:** `src/cli/mod.rs`, `tests/build_refuses_output_flags.rs`,
  `docs/cli-reference.md`, `docs/api-contract.md`, `CHANGELOG.md`, `decisions/DEC-057-…md` and
  `decisions/DEC-105-…md`. Anything else is a deviation; say why.

## Guardrails
- ⛔ No version bump, no release and no tag. 0.8.1 is cut by release prep; the tag is the
  maintainer's.
- **Every commit uses `git commit -s`.** The DCO check fails any PR with an unsigned commit.
- **Never poll CI.** Run `gh pr checks <PR> --watch` in the background. When it exits, read a
  direct snapshot at the PR's true head SHA and check every leg, including windows-latest.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file. A piped command reports the **pipe's** exit code, so redirect and read `$?`.
- zsh does not word-split unquoted parameters; loop with `while IFS= read -r`. macOS has no
  `timeout(1)`.
- **The matrix legs run sequentially, each in a fresh `CARGO_TARGET_DIR`:** default,
  `--no-default-features` and `--features webp-lossy`. Run clippy `-D warnings` and
  `fmt --check` on each. Delete each scratch target dir when you're done with it.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids.
- **Budget ~80 exchanges.** If the matrix is green and you are past that, stop and report what
  is left.

## When you finish
1. **Fill `## Build Completion`:**
   - every file from `git diff --name-only origin/main...HEAD`, not from memory;
   - commit 1's SHA, with `git show <commit 1> -- src/` showing only the test module;
   - the RED/GREEN baseline, test by test, with `main`'s exit codes;
   - each mutation's flips;
   - the empty `git diff --stat` and the `compare-builds.sh` output;
   - the matrix legs;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `grep -l "SPEC-138 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   - ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines
     that share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
     `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
     **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's
     output under-counts about 11×.
   - Price each component at the model that actually ran (read `.message.model`), never at a flat
     rate: Sonnet $3/$15 per MTok, Opus $5/$25. cache_creation is ×1.25 the input rate and
     cache_read ×0.10.
   - Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
     `estimated_usd` and the rate anchors.
3. **Advance and open the PR.**
   - Run `just advance-cycle SPEC-138 verify`.
   - Open the PR **as a draft**: `gh pr create --draft …`. ⛔ A draft cannot be merged. It stays
     a draft until a separate verify session approves it and the orchestrator marks it ready.
     Never mark it ready yourself.
   - Do **not** edit the spec's timeline file. It lives on `main`, and the orchestrator keeps it.
   - Title: `fix(build): refuse the output flags the manifest owns (SPEC-138)`.
   - Body per AGENTS §13:
     - Summary;
     - Spec metadata;
     - Decisions referenced: DEC-057 (amended), DEC-105 (amended), DEC-058, DEC-015;
     - Constraints checked;
     - New decisions: none;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-138 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`.

Then, short:
- commit 1's SHA, and that it touches no `src/` outside the test module;
- AC completion (17 rows);
- the RED/GREEN baseline confirmed;
- each mutation's flips;
- the byte-identity evidence (`git diff --stat` and `compare-builds.sh`);
- the matrix result;
- files changed vs the spec's list;
- any design call that turned out wrong;
- any out-of-spec finding;
- your measured cost.
