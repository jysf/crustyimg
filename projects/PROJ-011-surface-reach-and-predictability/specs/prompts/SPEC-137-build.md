# SPEC-137 — BUILD prompt: SVG text in any font family renders in the bundled font

Cycle: **build**. Fresh CLI session, own worktree, **Sonnet**. The design gives the exact code
and every test row, so the work is mechanical. The job is to be thorough across the rows, get
the commit order right, and **stop rather than loosen a check** if CI disagrees with the design.

```
just wt feat/spec-137-svg-font-family -- --model sonnet
```

You did not design this. The spec carries the context (AGENTS §15). The maintainer pasted this
prompt into a fresh session and will paste your final message back to the orchestrator, so that
message must stand on its own. This is one of the two specs that open **0.8.1**; SPEC-138 builds
after you, in its own session.

## What you are building
- `hardened_options()` in `src/image/svg.rs` points fontdb's five generic families at the
  bundled Go font. SVG text in **any** family (generic, installed, unknown, a list, set in
  `style=` or a `<style>` element) then draws in Go instead of drawing nothing.
- Bold and italic draw upright in Go Regular. No faces are added, and no system font is read.
- Every SVG that draws today keeps its exact pixels.
- No warning, no new flag, no new DEC. DEC-054 is amended.

## Step 0 — get onto current `main`
`git fetch origin && git rebase origin/main`. Then check all three:
- the spec is at `cycle: build`;
- its design commit `d7f028e` is in your history;
- its `### Maintainer rulings` has rulings 1 (silent fallback) and 2 (accept Go Regular).

If any of those fails, stop and report.

## Read in order, before any code
1. **The spec:**
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-137-svg-text-in-any-font-family-renders-in-the-bundled-font.md`.
   Read all of these in full:
   - `## Frame` and its rulings;
   - `## Design`: the RED baseline and Calls 1–8;
   - all 10 ACs;
   - `## Failing Tests` (7 tests) and the 6 negative controls;
   - `## Implementation Context`, including the DEC-054 amendment text;
   - `## Notes for the Implementer`.

   **The design calls are settled.** If one turns out wrong, say so in Build Completion and in
   your report. Do not quietly re-decide it.
2. **DECs:** DEC-054, DEC-045, DEC-034 and DEC-066. Read them; do not trust a summary.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`, "Read first" and "Traps this
   wave paid for". They are binding. Also `/guidance/constraints.yaml`.

## Order
1. **Commit 1: the tests, against unmodified behaviour.** Write `tests/svg_font_family.rs`, the
   two unit tests in `src/image/svg.rs`'s `#[cfg(test)]` module, and the wasm row in
   `tests/wasm_roundtrip.rs`. Run them on the unmodified code and record, test by test:
   - 4 RED, each **for the reason the spec gives** (ink 0, hash `bb918147…`, the five `None`
     queries, the wasm ink 0);
   - 3 GREEN, including `working_family_forms_keep_their_pinned_bytes`. That one being green
     proves your hash recipe matches `GO_HELLO_PIXELS_SHA256`. If it is red, **stop**: the test
     or the constant is wrong, not the code.

   ⛔ Commit 1 changes nothing in `src/` outside the `#[cfg(test)]` module. Verify checks it with
   `git show <commit 1> -- src/`.
2. **Implement Call 1** exactly as written: the five `set_*_family` lines, `opt.font_family`
   kept, and the doc comment and module doc wording. Push a WIP commit as soon as it compiles
   (`git commit -s`, then push). A long build with zero commits has cost this project 3 hours.
3. **Run the 6 negative controls, one at a time.** For each one: apply the mutation, check
   `git diff` is non-empty, rebuild **clean** (fresh `CARGO_TARGET_DIR` or
   `cargo clean -p crustyimg`), run the named tests, and record exactly which go RED. Then
   restore, rebuild, and show them green.
   - ⚠ Reverting source does not rebuild the binary.
   - **Control 2 is expected to turn only the unit row red.** Every CLI row stays green. Say so;
     do not "fix" it.
   - **Control 4 is macOS-only and temporary.** Never commit it.
4. **Docs and record,** worded in the spec (Call 8 and the amendment):
   - `docs/cli-reference.md` under `## Conventions` (there is no SVG section), and `README.md`;
   - the `CHANGELOG.md` `### Fixed` line under `## [Unreleased]`, naming 0.5.0 through 0.8.0.
     Append it; do not reflow other entries;
   - the DEC-054 amendment. Run `./scripts/decisions-audit.sh` afterwards.
   - No spec or DEC ids in help text, messages, comments or the CHANGELOG.
5. **Clean matrix,** then `just wasm-test`, then the PR.

## The things most likely to go wrong
- **The pinned hash on Windows.** The design proved the pin on arm64 macOS and emulated x86-64
  Linux, not on Windows. If CI fails **only the pinned-hash assertion on one OS**, and the
  in-run equality to the `Go` render passes there:
  - ⛔ do not loosen the pin, add a tolerance, or pin a second hash;
  - leave the PR as a draft, and report the leg, the hash that OS produced, and the passing
    equality check. The orchestrator takes it to the maintainer.
- **The Linux system-font row may skip.** The design could not confirm a font on
  `ubuntu-latest`. A printed SKIP there is expected; quote it from the CI log. A **silent** pass
  there is a defect.
- **The tspan row.** The spec's RED baseline says the mixed `Arial` + `Go` `<tspan>` row draws
  253 px on `main` (the frame said 359). Record what you measure; if it is neither, say so.
- **Ink is counted from `image::open(…).to_rgba8()`**, alpha > 0. Never through crustyimg's own
  decode path.
- **Don't add `fontdb` as a dependency or enable any of its features.** Reach it as
  `usvg::fontdb`. `Cargo.toml` and `Cargo.lock` must not change.
- **Stay in your files:** `src/image/svg.rs`, `tests/svg_font_family.rs`,
  `tests/wasm_roundtrip.rs`, `docs/cli-reference.md`, `README.md`, `CHANGELOG.md` and
  `decisions/DEC-054-…md`. Anything else is a deviation; say why.

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
   - the RED/GREEN baseline, test by test, with the measured values;
   - each negative control's flips;
   - each table's final row count (18, 6, 3);
   - the matrix legs, `just wasm-test`, and the `--nocapture` line of the system-font row;
   - a build-phase reflection that names any design call that turned out wrong.
2. **Measure your own cost.**
   - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
     directory's absolute path with every `/`, `.` and `_` replaced by `-`.
   - Identify it **by content, not recency**:
     `grep -l "SPEC-137 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
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
   - Run `just advance-cycle SPEC-137 verify`.
   - Open the PR **as a draft**: `gh pr create --draft …`. ⛔ A draft cannot be merged. It stays
     a draft until a separate verify session approves it and the orchestrator marks it ready.
     Never mark it ready yourself.
   - Do **not** edit the spec's timeline file. It lives on `main`, and the orchestrator keeps it.
   - Title: `fix(svg): draw text in any font family in the bundled font (SPEC-137)`.
   - Body per AGENTS §13:
     - Summary;
     - Spec metadata;
     - Decisions referenced: DEC-054 (amended), DEC-045, DEC-034, DEC-066;
     - Constraints checked;
     - New decisions: none;
     - end with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-137 build: draft PR #NNN at <head SHA> — CI <green | red: legs>`.

Then, short:
- commit 1's SHA, and that it touches no `src/` outside the test module;
- AC completion (10 rows);
- the RED/GREEN baseline confirmed, with the tspan row's measured ink;
- each negative control's flips;
- the matrix, `just wasm-test`, and what the system-font row printed on each CI OS;
- the pinned hash on every CI leg;
- files changed vs the spec's list;
- any design call that turned out wrong;
- any out-of-spec finding;
- your measured cost.
