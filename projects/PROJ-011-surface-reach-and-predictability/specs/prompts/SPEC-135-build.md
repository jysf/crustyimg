# SPEC-135 — BUILD prompt: `watermark` refuses a size or scale that is not a sensible number

Cycle: **build**. Fresh CLI session, own worktree, **Sonnet**. The design spelled out every
message, row and mutation, so the work is mechanical. The job is to be thorough across the rows
and to get the commit order right.

```
just wt feat/spec-135-watermark-flag-checks -- --model sonnet
```

You did not design this. The spec carries the context (AGENTS §15). The maintainer pasted this
prompt into a fresh session and will paste your final message back to the orchestrator, so that
message must stand on its own. **This spec gates 0.8.0**: the release is this spec plus release
prep, nothing else.

## What you are building
- `watermark` checks `--size`, `--scale` and `--opacity` against the same key table a recipe step
  is checked against (`WATERMARK_PARAMS`).
- A bad value is a usage error that names the flag (exit 2). It is reported before any file is
  read: before the inputs, `--font` and `--image`. `--gravity` and `--color` keep their own
  parsers, but those parsers move ahead of all file reads too.
- `size <= 1024` becomes a schema row. The CLI and recipes share it, and so does wasm, through
  the same `build_pipeline` call.
- A scaled mark too large for a `u32` is refused as
  `the scaled mark does not fit inside the W×H px image`. It never prints `4294967295`.
- Every mark that is accepted today stays byte-identical to `main`.

## Step 0 — get onto current `main`
`git fetch origin && git rebase origin/main`. Then check all three:
- the spec is at `cycle: build`;
- its design commit `3704216` is in your history;
- its `### Maintainer rulings` list goes up to **ruling 7**. Rulings 4–7 were added after design.

If any of those fails, stop and report.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-135-watermark-refuses-a-size-or-scale-that-is-not-a-sensible-number.md`.
   Read all of these in full:
   - `## Frame` and its rulings 1–7;
   - `## Design`: the RED-baseline table and Calls 1–6;
   - all 14 ACs;
   - `## Failing Tests`, including the six negative controls;
   - `## Implementation Context`;
   - `## Notes for the Implementer`.

   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in
   your report. Do not quietly re-decide it.
2. **DECs:** DEC-102, DEC-104, DEC-031, DEC-015, DEC-036 and DEC-063. Read them; do not trust a
   summary. The highest DEC on `main` is **DEC-105**. You write **DEC-106**.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`, "Read first" and "Traps this
   wave paid for" 1–10. They are binding. Trap 9 (test at the type the op reads) and trap 10 (a
   test needs an input where the old and new behaviour differ) are this spec's whole test design.
   Also read `/guidance/constraints.yaml`: `untrusted-input-hardening` and
   `no-unwrap-on-recoverable-paths`.
4. As shape references:
   - `tests/watermark_damage.rs`: `pixel_hash` at `:139-150`, bless at `:174-187`;
   - `tests/recipe_strict_params.rs`: `error_lines` at `:566`.

## Maintainer rulings (settled — do not re-open)
1. The cap is `size <= 1024`, inclusive, as a range row in `WATERMARK_PARAMS`.
2. Marks that draw nothing on **both** paths stay out: `opacity = 0`, blank text, `size = 0.01`.
   Do not refuse them.
3. The CLI keeps parsing these flags as `f32`. `--size 1024.00001` is accepted on the CLI and
   refused in a recipe. That residual is pinned by a test and documented, not fixed.
4. The ~50-row matrix ships whole, with no cut.
5. Two CLI messages change wording because they now come from the table. That is accepted:
   - `--opacity must be a number in 0.0..=1.0, got …`;
   - `--size must be a number > 0 and <= 1024, got …`.
6. `text::render_text` still panics for a direct library caller. It is filed for 0.8.1.
   **Do not fix it here.**

## Order
1. **Commit 1: goldens from unmodified `src/`.** Add `tests/watermark_cli_params.rs` with only the
   byte-identity test and the `#[ignore]` bless test. Bless
   `tests/fixtures/watermark/cli_params.golden` with
   `CRUSTYIMG_BLESS=1 cargo test --test watermark_cli_params -- --ignored bless`, and commit.
   - ⛔ **This commit touches no `src/` file.** Verify checks it with
     `git show --stat <commit 1>`.
   - Before committing, confirm the golden has exactly 3 lines.
   - The `logo` overlay is a 60×30 **gradient** RGBA PNG, built in-test with the `image` crate,
     never with crustyimg. A flat colour would hide the Lanczos3 → Triangle mutation (negative
     control 6).
2. **Every other failing test, run against unmodified `src/`.** Write each test the spec's
   `## Failing Tests` lists. Record, test by test, that:
   - each RED test is red **for the reason the spec gives**, with `main`'s exit code (0, 3, 101
     or other words);
   - each test the spec marks green-on-`main` is green.

   The new-API unit tests (AC-3 and AC-9) are red because they don't compile. Write them, note
   that, and keep them out of commit 1. Every table-driven test asserts its own row count: 10,
   10, 5 and 21.
3. **Implement, in this order:**
   1. the `size` cap row;
   2. the three `pub` items in `registry.rs`: `ParamKind::accepts`, `ParamKind::expectation` and
      `param_spec`. `check_params` keeps its wording byte for byte;
   3. `mark_size` → `Option`, then `Refusal::ScaledMarkTooLarge`, its `Display`, and its use in
      `refusal` / `apply`;
   4. in `src/cli/ops.rs`, `check_watermark_flag` and `flag_value_text`. Call them, plus the
      `--gravity` / `--color` parsers, **before** `watermark_overlay` and every other file read.
      Then delete the three hand checks at about `:1382`, `:1419-1423` and `:1425-1428`;
   5. the `--size` help line in `src/cli/mod.rs`.
4. **Push a WIP commit as soon as it compiles**, before the matrix (`git commit -s`, then push).
   A long build with zero commits has cost this project 3 hours before.
5. **Run negative controls 1–6, and 3b, one at a time.**
   - For each one: apply the mutation, check `git diff` is non-empty, rebuild, run the named
     tests, and record which go RED. Then restore the source, rebuild, and show them green.
   - ⚠ Reverting source does not rebuild the binary. ⚠ An incremental build shared across a
     mutation can serve a stale binary: use `cargo clean -p crustyimg` or a fresh
     `CARGO_TARGET_DIR` for each one.
   - **3b is the trap-10 pair.** Dropping only the height guard must turn AC-9 (c) RED, and
     nothing else.
   - **Control 4 is expected to leave AC-6 green.** Inputs are read after both positions. Say so;
     do not "fix" it.
6. **Docs and record,** worded in the spec:
   - `docs/data-model.md` and `docs/api-contract.md` (Call 3);
   - the `CHANGELOG.md` lines under `## [Unreleased]`: one Breaking and four Fixed (Call 5).
     Append them; do not reflow other entries;
   - **DEC-106** (Call 4), with a block-list `affected_scope`. An inline array is silently
     dropped by the audit:
     ```yaml
     affected_scope:
       - src/cli/ops.rs
     ```
     Run `./scripts/decisions-audit.sh` and confirm DEC-106 lists every path;
   - **DEC-102's Consequences line**, exactly as Call 4 words it.
   - No spec or DEC ids in help text, messages, comments or the CHANGELOG.
7. **Clean matrix,** then the PR.

## The things most likely to go wrong
- **A hollow row (trap 10).** The design picked each input so `main` and the fix disagree. Do not
  "simplify" them:
  - the 1024 and 1025 rows run on **1200×1000**. On 400×300, a 1025 mark is fit-refused on `main`
    too;
  - the 100000 rows use `© crustyimg`. With `hi`, `main` refuses instead of panicking;
  - AC-9 (c) is a **1×100** overlay at `scale 1562500`, so the width fits a `u32` and the height
    does not.
- **Input arity is not argument count (AC-6).** Cover each shape for both flags:
  - one file;
  - a batch directory of two PNGs;
  - a glob that resolves to one file, passed as one argument with no shell;
  - stdin `-` fed a **valid** PNG through `Stdio::piped()`, then closed;
  - a missing input.

  Each must exit 2 with the exact line and write nothing.
- **The batch row in AC-11** needs two inputs with **different names**. The same name twice
  collides in `--out-dir`.
- **The message prints the value the user typed, as parsed.** That gives `got 1.5`, not
  `got 1.5000000`, and `got inf` for `1e39`. Never build a `toml::Value` from the f32 and print
  that; it gives `got 1.100000023841858`.
- **No `unwrap` or `expect` in `src/**`.** A missing table row returns the "internal error" usage
  error. A unit test pins that all three keys exist.
- **No changes to `Cargo.lock`, the `Cargo.toml` version or `CACHE_SCHEMA_VERSION`** (AC-13).
- **Stay in your files.** The spec's Build Completion file list is what verify diffs against:
  - `src/operation/registry.rs`;
  - `src/operation/mod.rs`: the size row, `Refusal`, `mark_size`, `refusal` and `apply`;
  - `src/cli/ops.rs`: `run_watermark`, `watermark_overlay`, and the two new helpers;
  - `src/cli/mod.rs`;
  - the two test files and the golden;
  - the two docs, `CHANGELOG.md`, DEC-106 and DEC-102.

  Anything else is a deviation; say why. `recipes/README.md` is **not** touched; it states no
  size range.

## Guardrails
- ⛔ No version bump, no release and no tag. 0.8.0 is cut by release prep, and the tag is the
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
  `fmt --check` on each. Never run a background build while editing source for a revert. Delete
  each scratch target dir when you're done with it.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids.
- **Budget ~100 exchanges.** If the matrix is green and you are past that, stop and report what
  is left.

## When you finish
1. **Fill `## Build Completion`:**
   - every file from `git diff --name-only origin/main...HEAD`, not from memory;
   - commit 1's SHA, with its `--stat` showing no `src/`;
   - the RED baseline, test by test;
   - each negative control's flips;
   - each table's final row count;
   - the matrix legs;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `grep -l "SPEC-135 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
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
   - Run `just advance-cycle SPEC-135 verify`.
   - Open the PR **as a draft**: `gh pr create --draft …`. ⛔ A draft cannot be merged. It stays
     a draft until a separate verify session approves it and the orchestrator marks it ready.
     Never mark it ready yourself.
   - Do **not** edit the spec's timeline file. It lives on `main`, and the orchestrator keeps it.
   - Title: `fix(watermark): refuse a size or scale that is not a sensible number (SPEC-135)`.
   - Body per AGENTS §13:
     - Summary;
     - Spec metadata;
     - Decisions referenced: DEC-102 (amended), DEC-104, DEC-031, DEC-015;
     - Constraints checked;
     - New decisions: DEC-106;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-135 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`.

Then, short:
- commit 1's SHA, and that it touches no `src/`;
- AC completion (14 rows);
- the RED baseline confirmed;
- each negative control's flips, 3b included;
- the tables' final row counts;
- the matrix result;
- files changed vs the spec's list;
- any design call that turned out wrong;
- any out-of-spec finding;
- your measured cost.
