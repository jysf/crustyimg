# SPEC-146 — FRAME prompt: CI runs the wasm round-trip tests

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-146-frame -- --model opus
```

**Reserved:** **SPEC-146**. No DEC is reserved; the design session reserves one if it needs it.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own.

⚠ **Another orchestrator thread owns PROJ-017 (geometry as recipe steps) and is running at the same
time.** Its `crop` build will edit `src/operation/`, `src/recipe/`, `docs/recipes.md`,
`tests/recipe_corpus_output.rs` and `tests/fixtures/recipe_corpus/`. This spec must not touch any of
them, and you must never edit anything under `projects/PROJ-017-geometry-as-recipe-steps/`. If a
rebase conflicts in a PROJ-017 file, stop and say so; do not resolve it.

## Why this, and why now
`tests/wasm_roundtrip.rs` has 47 tests, including SPEC-143's FT-12 (a watermark that draws nothing
is refused in the browser build). They run only under `just wasm-test` and `just wasm-test-slim`,
which need `wasm-bindgen-test-runner`. **No workflow in `.github/workflows/` runs either.** So a
wasm-only regression merges on green CI. This is the same thesis as STAGE-053: a guard that does not
run.

Two backlog entries describe the same gap and are both in scope for this frame:
- STAGE-054: **"No CI job runs the wasm round-trip tests."** Filed by SPEC-143's verify
  (2026-10-30).
- STAGE-053: **"A wasm32 CI leg running `just wasm-check` + `just wasm-test`"** (chore, S–M, moved
  there from STAGE-038 item #8).

Fold them into one spec unless your frame finds a reason not to. Say which, with the reason.

## What is known (reproduce it, don't trust it)
- `.github/workflows/ci.yml` already has a job, `ddphotos-parity`, that installs the wasm32 target
  (`dtolnay/rust-toolchain@stable` with `targets: wasm32-unknown-unknown`), `just`, and `wasm-pack`,
  and caches with `Swatinem/rust-cache@v2`. A comment at the end of that job sketches the extra
  step: `cargo install wasm-bindgen-cli --version 0.2.126 --locked && just wasm-check && just
  wasm-test`, or `taiki-e/install-action` with `tool: wasm-bindgen@0.2.126`. Treat that as a
  starting hypothesis, not a design.
- `ci.yml` has a `changes` job and `scripts/ci-code-paths.sh` (SPEC-142) that gate heavy jobs on
  `needs.changes.outputs.code == 'true'`. A new job joins that gate.
- `justfile`: `wasm-check`, `wasm-test`, `wasm-test-slim` (feature set via `_wasm_features`;
  `wasm-test-slim` runs `just --set _wasm_features " --no-default-features --features avif"
  wasm-test`). The recipes resolve the rustup toolchain through `_wasm_bin`
  (`rustup which --toolchain stable rustc`), because Homebrew's `rustc` has no wasm std.
- `wasm-bindgen` is pinned `0.2.126` in `Cargo.toml`. `wasm-bindgen-test-runner` must match that
  exact version or it refuses to run the tests.
- `pages.yml` already builds the wasm artifact in CI, and the `ddphotos-parity` job runs wasm in
  Node. Neither runs `wasm_roundtrip.rs`.
- Required checks: find out which `ci.yml` jobs are required in branch protection
  (`gh api repos/jysf/crustyimg/branches/main/protection` if you can read it; otherwise say you
  could not) and whether `ddphotos-parity` is advisory ("Advisory: not a required check" in its
  comment).

## Questions this frame must answer
1. **Does it run today?** On a clean checkout with the toolchain CI would have, run `just wasm-check`
   and `just wasm-test` and `just wasm-test-slim`. Do all 47 tests pass on both feature sets? Is any
   test skipped or environment-dependent (Node version, fonts, timing)? A job that is red on day one
   is a different spec. Report the pass counts verbatim.
2. **The runner.** How does CI get `wasm-bindgen-test-runner` at exactly the version in
   `Cargo.lock`? Compare `cargo install … --locked` (slow, cacheable) against
   `taiki-e/install-action` (fast, prebuilt) and `wasm-pack test`. Does the action offer that exact
   version? Probe the install on a runner-equivalent environment as far as you can locally; a
   version mismatch must fail loudly, not skip.
3. **Which Node.** The runner executes the `.wasm` in Node. What Node does `ubuntu-latest` ship, and
   does the harness need a minimum? Pin it if so.
4. **New job or extra steps?** Add a job `wasm-roundtrip` next to `ddphotos-parity`, or extend
   `ddphotos-parity`? Weigh: a failure names the cause, cache sharing, wall time, and whether the
   check should be required while the other stays advisory.
5. **Runtime and required-ness.** Measure the job's wall time (cold and with a warm cache, as far as
   you can estimate it) before recommending that it be a required check. HANDOFF trap 5 says
   `just check` is weaker than CI; say whether the local gate list in CLAUDE.md or the justfile
   should name the new leg.
6. **Does it actually guard?** The job is only worth adding if a wasm-only regression turns it red.
   Name one real mutation, in `src/wasm.rs` or in wasm-gated code, that leaves every native test
   green but turns a `wasm_roundtrip` test red. Run it locally on both feature sets, restore the code
   exactly, and report which test went red and its failure message (HANDOFF traps 12 to 14). Also
   name how the CI job would show the mutation as red, not skipped.
7. **The slim leg.** Is `wasm-test-slim` worth a second CI invocation (it proves the slim build
   refuses SVG with the hint)? Cost versus value.
8. **Docs.** Which docs name the wasm test commands or list CI jobs (`CLAUDE.md`, `RELEASING.md`,
   `docs/`, `justfile` comments, the `ddphotos-parity` comment block)? List each sentence that
   changes.
9. **Size and scope.** Expect [S] or low [M]. State what the verification costs (a CI job's design
   needs a way to prove it runs and fails: see HANDOFF traps 11 and 15). Give the scope boundary.

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly sized, correctly scoped
spec stub. You do **not** write acceptance criteria, failing tests or Implementation Context.
Separate design, build and verify sessions follow. The orchestrator only sequences.

### Step 0 — get onto current `main`
You should be in your own worktree (made with `just wt <branch>`). Run `git fetch origin`, then
`git rebase origin/main`. If `git status` shows you are in the maintainer's main checkout rather
than a worktree, stop and say so. Use your own `CARGO_TARGET_DIR`; never share one with another
session.

### Read first
1. `/AGENTS.md` (§2 sizing, §8 cycles, §13 git, §15 cycle rules).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Read first", the rules, and
   "Traps this wave paid for" (1 to 16). Binding. Traps 11, 12, 15 and 16 are about CI steps.
3. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-054-recipe-reach-follow-through.md`
   (the wasm round-trip entry) and `STAGE-053-guards-that-cover-this-project.md` (the wasm32 CI leg
   entry).
4. `.github/workflows/ci.yml`, `.github/workflows/pages.yml`, `scripts/ci-code-paths.sh`, the
   `justfile` wasm recipes, `tests/wasm_roundtrip.rs`, `src/wasm.rs`.
5. DEC-064, DEC-065, DEC-066 (wasm target, feature set, size profile) and any DEC naming
   `.github/workflows/` (`./scripts/decisions-audit.sh`).
6. `/guidance/constraints.yaml`.

### What a good frame does here
- **Run it before you describe it.** Question 1 and question 6 are measurements; do them.
- **Read the tools' docs for what they do:** `taiki-e/install-action`'s supported tools and
  versions, and wasm-bindgen's test-runner version rule.
- ⛔ **Run nothing that publishes or tags.** No `git tag`, no `npm publish`, no `cargo publish`
  without `--dry-run`, no `gh workflow run`.
- **Size the verification, not just the change** (AGENTS §2): how will verify prove the job runs,
  runs the right tests, and goes red on a wasm-only regression?
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths). It must not include `src/operation/`, `src/recipe/`,
  `docs/recipes.md`, or the recipe-corpus test and fixtures.
- **Questions only the maintainer can answer:** one line each, with your recommendation.

### The id is reserved — use exactly SPEC-146
Before allocating, confirm SPEC-146 is unused everywhere:
```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-146' "$b" -- . ; done
```
This prompt and the PROJ-011 brief name SPEC-146, so expect those; nothing else should. Then run
`just new-spec "<title>" STAGE-054 PROJ-011`. `just new-spec` scans only the working tree, so it may
allocate a different number. If it allocates anything other than SPEC-146, `git mv` the spec and
timeline files to SPEC-146 and fix the id inside them. Then `just advance-cycle SPEC-146 frame`.
Write no DEC at frame.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`
  (decisions and related specs), `cycle: frame`.
- `## Context` and `## Goal`: what the maintainer sees today (47 wasm tests that no CI run executes)
  and what they see when it ships (a wasm-only regression turns a named CI job red).
- A `## Frame` section directly after `## Goal`: verdict (go / no-go / split), premise check with
  evidence, the answers to questions 1 to 9, sizing and why, scope in/out, rough build file list,
  open maintainer questions.
- `### Out of scope (for this spec specifically)`, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2 to 4 line summary; add `[ ]` lines for design
  (`prompts/SPEC-146-design.md`), build, verify, ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), no PR.
1. On **go**: `just advance-cycle SPEC-146 design`. On no-go or split: leave `cycle: frame`.
2. In STAGE-054's backlog, replace the "No CI job runs the wasm round-trip tests" entry with
   `- [ ] **SPEC-146** (framed <date>) — [size] **<title>**` and a 2 to 3 line summary. In
   STAGE-053, mark the wasm32 CI leg entry as folded into SPEC-146 (or leave it if you did not
   fold it, saying why). File any out-of-spec finding as a new `- [ ] (not yet written)` entry,
   and update each `**Count:**` line with a grep you have just run. Edit no other stage.
3. `git commit -s`, ending the message with
   `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`. **The DCO sign-off is mandatory.**
   Message: `docs(SPEC-146): frame — CI runs the wasm round-trip tests`.
4. `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. Other sessions
   push to `main` too; if the push is rejected, rebase and push again.
5. Read back what you pushed (`git show --stat HEAD`, and the stage diffs), and run `just backlog`
   to confirm the tooling sees SPEC-146.

### Guardrails
- ⛔ No code, no workflow edits, no DEC, no version bump, no tag, no publish. (A scratch copy of a
  workflow step, to test an install command locally, is fine. Discard it.)
- Restore any mutation exactly; show `git diff` empty afterwards.
- A piped command reports the pipe's exit code: redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`.
- Keep scratch files in a scratch directory, never in `~`.
- **Budget ~45 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:** search it for `FRAME prompt: CI runs the wasm round-trip tests`. `grep -c` can come
back empty through the `rtk` wrapper; count with Python or `/usr/bin/grep`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
under-counts about 11×.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and cache_read
×0.10. Append a `frame` entry to the spec's `cost.sessions` with `tokens_total`, a
`tokens_breakdown`, `estimated_usd` and the rate anchors, before your final commit.

### Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-146 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`. Then, short:
- the premise check: do the 47 tests pass today on both feature sets (counts verbatim);
- the runner install route and the version-match evidence;
- new job vs extra steps, with the runtime you measured;
- the mutation that only the wasm tests catch, and the red test's failure message;
- required vs advisory, with your recommendation;
- size and why; scope in/out, including the fold-in of the STAGE-053 wasm32 leg;
- the rough build file list (and confirmation it avoids PROJ-017's files);
- maintainer questions, each with your recommendation;
- any out-of-spec findings and where you filed them;
- your measured cost.
