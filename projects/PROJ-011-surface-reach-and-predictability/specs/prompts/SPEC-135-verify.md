# SPEC-135 — VERIFY prompt: `watermark` refuses a size or scale that is not a sensible number

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not design or
build SPEC-135.

```
just wt verify/spec-135 -- --model opus
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin && git reset --hard origin/feat/spec-135-watermark-flag-checks
```

Draft PR #207's head was `8922f31` when this prompt was written. Record the SHA you verify. The
maintainer pasted this prompt into a fresh session and will paste your final message back to the
orchestrator, so it must stand on its own. **This spec gates 0.8.0.**

## What the build claims
- The spec is
  `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-135-watermark-refuses-a-size-or-scale-that-is-not-a-sensible-number.md`.
  Read it all: `## Frame` with rulings 1–7, `## Design` with Calls 1–6, the 14 ACs,
  `## Failing Tests` with the six negative controls, and the build's `## Build Completion`.
- 14 of 14 ACs met.
- Commit 1 is `d1484cf`. It holds the goldens and touches no `src/`.
- The table row counts are:
  - AC-4: 10;
  - AC-6: 10;
  - AC-7: 5;
  - the recipe range table: 21;
  - AC-3: 42;
  - AC-12: 3 goldens over 6 runs.
- Every negative control flipped, with two differences from the spec's predictions:
  - under "cap row removed", **AC-3 stayed green**. The build says AC-3 compares the table with
    itself, so it cannot see the row's value;
  - under "checks moved after the IO", the AC-4 table also went RED, because its
    `© crustyimg --size 100000` row panics before the check runs.
- Deviations:
  - an existing SPEC-131 test (`a_number_past_f32_range_is_refused`) now expects its own phrase
    for `size`;
  - a `fits_u32` helper (`side.is_finite() && side < u32::MAX as f32`) replaces the design's
    `!(w < u32::MAX as f32)`, because clippy rejects the design's form;
  - the help test reads the `--size` entry;
  - the byte-identity test also checks recipe hash = CLI hash.
- The matrix is green on the default, `--no-default-features` and `--features webp-lossy` legs.
  CI is green.

## ⛔ Read-only, and the PR stays a draft
No merge, no `gh pr ready`, no commits and no pushes. Do not fix what you find; report it. Do not
edit the spec or its timeline.

## Check, in order
1. **Scope and commit order.**
   - Run `git diff --stat origin/main...HEAD` and compare it with the spec's file list and the
     build's Build Completion.
   - `git show --stat d1484cf` must show no `src/` path.
   - Every commit is signed off.
   - The diff contains no `Cargo.lock`, no `Cargo.toml` version and no `CACHE_SCHEMA_VERSION`
     change.
2. **The goldens describe `main`, not the build.** Use two independent checks:
   - check out `d1484cf` in a scratch worktree and run its byte-identity test against that
     commit's own unmodified `src/`. It must be green;
   - build `main` and the PR head as release binaries, each in a fresh target dir. Run the three
     AC-12 cases through each binary, CLI and `apply --recipe`, and compare the outputs with a tool
     the build did not write: ImageMagick `compare -metric AE` must report 0, or `cmp` on the
     files. A hash computed by the test's own `pixel_hash` alone is not enough.
3. **Drive every AC row on the PR's release binary.** Do not read the tests and trust them.
   - Check the exact stderr line and exit code for AC-4's ten rows.
   - Check the 1024 / 1025 / 1024.00001 rows on the 1200×1000 input.
   - Check AC-6's input shapes for both flags. Feed stdin a **valid** PNG, and pass the glob as one
     argument.
   - Check AC-7's five font/overlay rows.
   - Check AC-11's four single-input rows plus the batch row; none may contain `4294967295`.
   - For each row, also run `main`'s binary and confirm `main` gives the "RED on `main`" answer
     the spec quotes. A row whose answer is the same on both binaries is hollow (trap 10).
4. **Re-apply the negative controls yourself**, including 3b. Do them one at a time, and for each:
   - check `git diff` is non-empty;
   - rebuild in a **fresh** `CARGO_TARGET_DIR` or after `cargo clean -p crustyimg`. Reverting
     source does not rebuild the binary;
   - show which named tests go RED;
   - restore, and show them green.

   Judge the build's two prediction differences:
   - **AC-3 under "cap removed".** Is it right that AC-3 is a consistency test and cannot see the
     row's value? If so, does some other **library-level** test fail when the cap row is removed?
     The recipe range table and the inclusive-cap test are the candidates. If none does at that
     level, that is a punch item.
   - **The AC-4 panic under "checks after the IO".** Confirm it is the `zeno` panic and comes from
     the reordering, not from a new bug.
5. **Mutations the spec did not list.** Apply each, rebuild, and name the test that catches it,
   or report a gap:
   - **M-a:** the cap becomes exclusive (`< 1024`). The inclusive-cap test and AC-5's 1024 row
     must go RED;
   - **M-b:** the CLI message names the recipe key instead of the flag (`'size'` for `--size`).
     AC-4 must go RED;
   - **M-c:** `flag_value_text` prints the value widened to `f64`, which gives
     `got 1.100000023841858`-style text. A row with a non-integer value must go RED; say which
     one. If no row uses a value that `f32` cannot represent exactly, that is a gap;
   - **M-d:** `fits_u32` drops `is_finite()`. Read the call sites and say whether anything still
     refuses NaN and inf there. If the change is unobservable, say why rather than calling it a
     gap;
   - **M-e:** only `--gravity`, not `--color`, moves ahead of the IO. AC-7's `--color` row must go
     RED.
6. **Code quality.** Read the whole diff:
   - no `unwrap` or `expect` in `src/**`;
   - `check_params`' wording is byte-identical to `main`'s (diff the strings);
   - `ParamKind::accepts`, `expectation` and `param_spec` are minimal and documented;
   - the three hand checks are gone;
   - `fits_u32` is exactly equivalent to the design's form for every `f32`, NaN and ±inf included.
     Also check the rounding near 2³²: can a value below `u32::MAX as f32` still round or cast to a
     saturated `u32`?
   - comments and messages read behaviour-first, with no spec or DEC ids.
7. **Docs and record**, driven on the binary:
   - the `--size` help line;
   - the `docs/data-model.md` row and its f32 paragraph;
   - the `docs/api-contract.md` ordering and the new refusal form;
   - the five `CHANGELOG.md` lines match Call 5, with no spec or DEC ids, and no other spec's
     entries are touched.

   For each sentence, run the command it describes.
   - **0.8.0 wording check:** no line may promise more than "a non-finite or oversized size or
     scale is refused", for example "never an invisible mark".
   - **DEC-106:** check its block-list `affected_scope` with `./scripts/decisions-audit.sh`. Also
     check DEC-102's amended Consequences line keeps its "did you mean" clause.
8. **Wasm (Call 6).** Read `src/wasm.rs:225-245` and confirm `transform` still builds through
   `build_pipeline`, so the schema cap reaches it.
9. **Full suite and CI.**
   - Run the default leg **and** the `--no-default-features` leg once each, sequentially, each in
     a fresh target dir. Use `--no-fail-fast`, with stdout redirected to a file, and read `$?`.
   - Run `gh pr checks 207` once at the head you verified and check every leg. Do not poll.

## Guardrails
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`.
- zsh does not word-split unquoted parameters. macOS has no `timeout(1)`.
- Never share a target dir between differently-featured builds running at once. Delete each
  scratch target dir and scratch worktree when you are done with it.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~80 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Find it by content: `grep -l "SPEC-135 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran (read `.message.model`), never at a flat
  rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and
  cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — merge PR #207 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #207 at <SHA> needs: <one-line summary>.`
- `❌ REJECTED — <reason>.`

Then:
- items 1–9, one or two lines each with evidence;
- the negative-control table: control, tests that went RED, and whether that matches the spec;
- M-a to M-e, each caught (by which test) or a gap;
- optional cleanups, listed separately from punch items;
- your cost.
