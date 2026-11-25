# SPEC-146 — DESIGN prompt: CI runs the wasm round-trip tests

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-146-design -- --model opus
```

Starts from the stub the frame session pushed to `main` (`2d18134`). **Reserved: SPEC-146.** No DEC
is reserved; if you need one, see "Ids" below. The change is small ([S], about 30 lines of YAML);
the verification is the expensive part.

⚠ **Another orchestrator thread owns PROJ-017 (`crop` as a recipe step) and is running at the same
time.** Never edit anything under `projects/PROJ-017-geometry-as-recipe-steps/`. Your spec's build
file list must not include `src/operation/`, `src/recipe/`, `docs/recipes.md`,
`tests/recipe_corpus_output.rs` or `tests/fixtures/recipe_corpus/`. If a rebase conflicts in a
PROJ-017 file, stop and say so; do not resolve it.

**Read the stub's `## Frame` section first.** It holds the premise evidence (47 and 45 tests pass
cold), the runner route, the Node pin, the new-job-vs-steps call, the mutation that only the wasm
tests catch, the scope, the file list and the maintainer questions. It is your binding scope.

## Rulings (settled — do not re-open)
1. **A new job `wasm-roundtrip`**, with its own cache key, inside the `changes` gate, calling the
   `just` recipes (`wasm-check`, `wasm-test`, `wasm-test-slim`). `ddphotos-parity` is not touched.
2. **The runner is installed with `taiki-e/install-action`** at the version in `Cargo.lock`, and the
   job asserts `wasm-bindgen-test-runner --version` after install. `wasm-pack test` is ruled out
   (DEC-064). `cargo install --locked` is the fallback only if the action cannot do it.
3. **Node is pinned to 22** with `actions/setup-node`.
4. **`.cargo/` is added to `scripts/ci-code-paths.sh`.** The frame found that a change to
   `.cargo/config.toml`, which holds the wasm test runner, classifies as docs-only and skips every
   heavy job.
5. **Required-check status is the maintainer's, after the first green run on `main`.** The spec does
   not change branch protection; it says what the maintainer does and when.
6. **Out of scope:** wasm32 clippy or `-D warnings` (filed on STAGE-053 as its own entry), wasm
   tests in `just check`, the bundle-size gate, `npm_smoke` on PRs, any change to `src/wasm.rs` or
   `tests/wasm_roundtrip.rs`.

## Design calls that are yours
- **The job, step by step.** Write the YAML in the spec. For each step: the action and its exact
  version (look at what `pages.yml` and `ddphotos-parity` pin; do not guess a tag), what it needs,
  and what a failure of that step looks like in the log. Say how the job reads the wasm-bindgen
  version from `Cargo.lock`, and what it prints when the runner's version differs.
- **The test-count floor.** The frame found a hole: if a cfg change compiled the tests out, the
  job would print `0 passed` and stay green. Specify the floor. Candidates: assert the log line
  `47 passed` / `45 passed` exactly, or a minimum count read from a number kept in one place. Weigh
  the cost of editing the number each time a wasm test is added against the cost of a silent drop.
  Say where the number lives and who updates it. Remember `cargo test` output goes to stdout and a
  pipe reports the pipe's exit code (trap in the guardrails): write the step so a failing
  `cargo test` fails the step.
- **Both feature sets.** `wasm-test` and `wasm-test-slim` each run the suite; the slim run is the
  only one that proves the slim build refuses SVG with the hint. One job running both, in order, or
  two matrix legs? Decide with the wall times the frame measured (about 20 s each locally).
- **The classifier.** Extend `scripts/ci-code-paths.sh` and its test table (SPEC-142 built a table
  harness; find it). Add rows for `.cargo/config.toml` (true) and one nearby path that must stay
  false. Per HANDOFF trap 15, the design lists the classifier crashing and printing junk as
  mutation rows, and the harness summary counts every failure kind. Check whether
  `.github/workflows/ci.yml` itself counts as a code path: a PR that adds the job must run the job.
- **Local gate.** The frame says CLAUDE.md has no "just check is weaker than CI" list, so STAGE-053's
  citation of one is stale. Decide where the local-gate truth lives (`AGENTS.md` §6 is the frame's
  pick: one line) and word it. Do not edit the stage's other entries.
- **Docs.** Word each sentence in the frame's list: the `ci.yml:100-102` comment block (replace
  it), `justfile` comments and the install hint, `.cargo/config.toml` comment, `AGENTS.md` §6,
  `docs/roadmap.md`, `docs/research/proj-008-wasm-build.md`.

## Acceptance and tests: each one must be able to fail
This spec changes a workflow, a script and docs, not `src/`. Most checks are structural or live-run.
Each AC names how it is checked and what would turn that check red.
- **The job exists, is gated, and runs.** Check by script (kept in `scripts/`, run at verify, ideally
  in CI) against `ci.yml`: the job name, `needs: changes`, the `if:` on the code output, the
  runner-version assertion, the Node pin.
- **The live run.** On the build's draft PR: the job log shows `47 passed` and `45 passed`, quoted.
- **A wasm-only mutation turns it red.** The frame's mutation: `src/wasm.rs:152`,
  `self.has_alpha` → `!self.has_alpha`. Verify puts it on a throwaway commit of the draft PR and
  records the job red, with the failing test's message (`wasm::info_reports_png_dimensions`,
  "the RGBA fixture carries alpha"). The spec must say this edit is **never merged** and that the
  verify session reverts it exactly (`git diff` empty). Per trap 14, the control for the fix is the
  mutation going red on the new job and the same commit being green on every native leg.
- **The version assertion fails on a mismatch.** The frame could not drive this (the auto-mode
  classifier blocked downloading upstream binaries). Specify a way verify can drive it: for example
  pinning a deliberately wrong version in a scratch copy of the step, or running the assertion
  script against a stub that prints a wrong version. The check must be red for its reason: quote
  the message it prints.
- **The floor trips.** Specify a mutation that makes the suite run zero (or fewer) tests while
  `cargo test` exits 0, such as a cfg that compiles `tests/wasm_roundtrip.rs` out, and the check
  that goes red on it. Name the failure message (trap 13): it must be the floor's, not a compile
  error.
- **The classifier row.** `.cargo/config.toml` alone classifies as code. Revert the one line in
  `scripts/ci-code-paths.sh` and the new table row goes red, with the message that names that path.
- **A mutation table (trap 12)** with the exact location of each row and every check expected red,
  on both the classifier path and the job path. At least: remove `needs: changes`; remove the
  version assertion; remove the floor; revert the `.cargo/` line; swap `wasm-test-slim` for
  `wasm-test` (does anything notice that the slim set is no longer run?); the `src/wasm.rs`
  mutation above.
- **Trap 11:** nothing here may skip silently. A job that cannot find the runner must fail, not pass.
- **Trap 16:** if the job sets a step-level or job-level env var, name every command in that step.
- **Count the nouns** in each AC against the checks (trap 2). An AC naming "both feature sets" needs
  a check per set.

**Size.** The frame says [S]. If verification pushes it past S, report the cut before writing it.

## Files
The frame's list: `.github/workflows/ci.yml`, `scripts/ci-code-paths.sh` (and its test table),
`justfile` (comments and install hint only), `.cargo/config.toml` (comment only), `AGENTS.md` §6
(one line), `docs/roadmap.md`, `docs/research/proj-008-wasm-build.md`, any `scripts/` check you add.
Confirm none is a PROJ-017 path.

## DECs to read
DEC-064, DEC-065, DEC-066 (wasm target, feature set, size profile), and any DEC naming
`.github/workflows/` or `scripts/ci-code-paths.sh` (`./scripts/decisions-audit.sh`). HANDOFF
traps 11, 12, 13, 14, 15 and 16.

---

## Rules every design session follows

**You are the design cycle only.** Write the spec; do not build it. The maintainer pastes this
prompt into a fresh CLI session and pastes your final message back to the orchestrator, so that
message must stand on its own.

### Step 0 — get onto current `main`
Run `git fetch origin && git rebase origin/main`. The stub must be at
`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-146-ci-runs-the-wasm-round-trip-tests.md`
with `cycle: design`. If not, stop and report; do not create it. A separate session builds it from
your spec alone, and another verifies it, so everything the builder needs must be in the spec
(AGENTS §15). Use your own `CARGO_TARGET_DIR`; never share one with another session.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules and
   "Traps this wave paid for" (1 to 16). Binding.
3. The stub's `## Frame`, and the STAGE-054 and STAGE-053 entries it touched.
4. `.github/workflows/ci.yml`, `pages.yml`, `scripts/ci-code-paths.sh` and its tests, the `justfile`
   wasm recipes, `.cargo/config.toml`, `tests/wasm_roundtrip.rs`.
5. `/guidance/constraints.yaml` and the DECs above.
6. The tools' own docs for the versions in use: `taiki-e/install-action`'s tool list, and
   `actions/setup-node`. Quote at most one short phrase per fact, and cite the page.

### How to design well here
- **Probe, don't assume.** Run the classifier and its table harness locally. Run `actionlint` on a
  scratch copy of your YAML if it is installed. You may run the wasm recipes to confirm the commands
  you put in the spec; use your own target dir.
- ⛔ **Nothing that publishes or tags.** No `git tag`, no `npm publish`, no `cargo publish`, no
  `gh workflow run`. Do not edit branch protection.
- **Ask what the maintainer sees:** the PR check list before and after, a red job's name and log,
  and the day a wasm test is added and the floor must change.

### Ids
Do not run `just new-spec`. Fill Acceptance Criteria, Failing Tests (or "Checks", if that reads
truer for workflows; say so) and Implementation Context; keep `## Frame`. If you find the frame
wrong, say so in your report rather than silently widening or narrowing it. If you need a DEC, run
`ls decisions/` after your last rebase: other projects add DECs on `main` in parallel. Use the
**block-list** `affected_scope` form (an inline array is silently dropped by
`scripts/decisions-audit.sh`). Prefer no DEC unless a real decision is made; say which.

### Landing the design
Design commits go to `main` directly (AGENTS §13), no PR.
1. `just advance-cycle SPEC-146 build`.
2. In STAGE-054's backlog, change **only** SPEC-146's marker from `(framed 2026-11-05)` to
   `(designed <date>)`. Mark design `[x]` in the timeline with a 2 to 4 line summary, and add the
   build prompt path `prompts/SPEC-146-build.md` to the build line.
3. `git commit -s`, ending the message with
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. **The DCO sign-off is mandatory.**
   Message: `docs(SPEC-146): design the wasm round-trip CI job`.
4. `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. If the push is
   rejected, rebase and push again.
5. Read back `git show --stat HEAD` and run `just backlog`.

### Guardrails
- ⛔ No workflow or code changes outside the spec, timeline, stage file (and a DEC if one is used).
  No version bump, no tag, no publish.
- A piped command reports the pipe's exit code: redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~50 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:** search it for `DESIGN prompt: CI runs the wasm round-trip tests`. `grep -c` can come
back empty through the `rtk` wrapper; count with Python or `/usr/bin/grep`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
under-counts about 11×.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and cache_read
×0.10. Append a `design` entry to the spec's `cost.sessions` with `tokens_total`, a
`tokens_breakdown` and `estimated_usd`, before your final commit.

### Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-146 design: <commit SHA on main> — <N> ACs, <M> checks`. Then, short:
- each design call and how you settled it;
- the job in outline: runner install and version assertion, Node, the two feature sets, the floor;
- the test-count floor: where the number lives and who updates it;
- the classifier change and its table rows; whether `ci.yml` itself counts as a code path;
- how verify drives the version-mismatch check, the floor and the `src/wasm.rs` mutation;
- the mutation table: each mutation's location and every check expected red;
- the file list the build will touch, and confirmation it avoids PROJ-017's files;
- whether a DEC is used;
- anything that needs a **maintainer** ruling, each with your recommendation;
- whether to build on Sonnet or Opus, and why;
- your measured cost.
