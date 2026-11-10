# SPEC-154 — FRAME prompt: the CI path filter must not call a real build input docs-only

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-154-frame -- --model opus
```

**Reserved: SPEC-154** (STAGE-063, reserved 2026-11-10 with SPEC-155 to SPEC-158; PROJ-017 holds
SPEC-153). No DEC reserved. The maintainer pastes this prompt into a fresh session and pastes your
final message back to the orchestrator, so it must stand on its own. Before you allocate, confirm
SPEC-154 is unused everywhere:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-154' "$b" -- . ; done
```

Expected hits: this prompt, STAGE-063 and `docs/reviews/2026-11-10-external-code-review.md`. If a spec
file or another reservation uses SPEC-154, stop and report. Don't pick another number.

⚠ **Another orchestrator thread owns PROJ-017** (geometry as recipe steps) and may be building
SPEC-153 at the same time. It edits `src/operation/`, `src/recipe/`, `docs/recipes.md`,
`skills/crustyimg/SKILL.md`, `CHANGELOG.md`'s Unreleased section and `tests/common/mod.rs`. This
spec's files (`scripts/ci-code-paths.sh`, `.github/workflows/`) should not overlap; **list any that
do.** Never edit `projects/PROJ-017-geometry-as-recipe-steps/`.

## Why this, and why now
An external review (`docs/reviews/2026-11-10-external-code-review.md`) found that the `changes` job
in `.github/workflows/ci.yml` classifies a diff as docs-only when it isn't. The classifier is one
regex in `scripts/ci-code-paths.sh` (`CODE_PATHS`), with a self-test (`--self-test`). The reviewer
ran it against four paths and each printed `false`:

- `recipes/web.toml`: embedded by `src/recipe/bundled.rs` with `include_str!`.
- `assets/fonts/Go-Regular.ttf`: embedded by `src/text/mod.rs` with `include_bytes!`.
- `clippy.toml` and `deny.toml`: they decide the lint and the dependency-audit result.

A PR that edits only one of those skips the native build and test legs and can merge green. This
spec is the **first of five** from the review and **lands first and alone**: SPEC-155 to SPEC-158
change `src/sink/`, `src/metadata/` and `src/cli/build.rs`, and they must run through the full
gates. The same gap exists in `.github/workflows/pages.yml` (its `paths:` lists have `src/**`,
`tests/**` and `Cargo.*`, but not `recipes/` or `assets/`), and its header says the two lists are
duplicated on purpose and must be kept in sync.

## Your questions
1. **Premise.** On `main`, run the classifier against each of the four paths and quote the output.
   Then show the harm end to end: which `ci.yml` jobs are gated on `changes`, and what a
   `recipes/web.toml`-only PR skips. Does a skipped required job count as passing for the merge?
   (Read the workflow and the repo's branch-protection notes in `AGENTS.md`/`docs/development.md`;
   don't change the gating semantics.)
2. **The full set, not the four.** The review named four; there may be more. Drive it, don't guess:
   - every path an `include_str!`, `include_bytes!`, `env!("CARGO_MANIFEST_DIR")` or `concat!` pulls
     into the build (`src/`, `tests/`, `benches/`, `build.rs`);
   - every path a test reads at runtime (a test that reads `docs/…`, `skills/…`, `README.md`,
     `CHANGELOG.md` or `recipes/…` makes that path a test input; `tests/` includes scripts the
     tests shell out to);
   - every top-level file and directory in `git ls-files`, each with a verdict (code / docs-only),
     so the answer is a table you can check, not a recollection;
   - `Cargo.toml`-adjacent files: `rust-toolchain*`, `.cargo/`, `dist-workspace.toml`, `deny.toml`,
     `clippy.toml`, any `rustfmt.toml` (none today), `integrations/` outside `ddphotos/`, `demo/`,
     `skills/`, `examples/`.
   Say which of them the classifier gets wrong today, and which it gets right by accident.
3. **Allow-list or deny-list.** The regex lists what is code; anything unlisted is docs-only, so a
   new top-level directory is silently skipped. Evaluate the opposite design: a **docs-only
   allow-list** (`docs/`, `projects/`, `decisions/`, `guidance/`, `*.md` …) where anything else
   runs the full gates. Recommend one with evidence, including how it classifies every row of your
   table from question 2, and what it costs (CI minutes on a docs-adjacent PR that the allow-list
   can't prove is docs-only).
4. **Self-test.** Rows to add for every class (each of the four, the new ones from question 2, and a
   negative control that stays `false`, such as `docs/recipes/web.toml`). The script's own anchor
   `^` matters: say what a nested `docs/.cargo/config.toml` does after the change. **Mutation
   rows** (AGENTS §15, and SPEC-152's ship reflection): for each alternative you add, name the exact
   edit that removes it and show the self-test goes red.
5. **Pages and the other workflows.** Compare `pages.yml`, `publish-crates.yml`, `publish-npm.yml`
   and any other `paths:` filter against the classifier. Which shared inputs does each need?
   GitHub's workflow parser has no YAML anchors, so the lists stay duplicated: recommend a way to
   catch drift (a self-test row, or a script that compares the lists), or say why not.
6. **What can't be tested locally.** You can run the script and `--self-test`; you can't run
   Actions. Say what you can only verify by opening a PR, and recommend the smallest PR that proves
   it (for example one that touches only `recipes/web.toml`). Do not poll CI (AGENTS §16 habit 6).
7. **Fold-ins.** Read every open STAGE-063 entry and recommend fold or leave. Likely none; say so.
8. **Docs.** Where the classifier is described (`AGENTS.md`, `docs/development.md`, comments in
   `ci.yml`), and what each says after.
9. **Size.** The backlog says [S]. Confirm or resize, and say why.

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly sized, correctly scoped
spec stub. You do **not** write acceptance criteria, failing tests or Implementation Context, and
you do **not** write any DEC. Separate design, build and verify sessions follow. Write your final
message as plain lists, not box-drawn tables.

### Step 0 — get onto current `main`
You should be in your own worktree. Run `git fetch origin`, then `git rebase origin/main`. If
`git status` shows you are in the maintainer's main checkout rather than a worktree, stop and say so.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 backlog, §13 git, §15 (its two newest verify rules bind
   every cycle: "A bound or 'can't happen' is a claim until its worst input is driven" and
   "Classify every finding") and §16 habit 6 (no CI wait loops).
2. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-063-honest-output-and-the-0-8-3-release.md`:
   your entry, SPEC-154 to SPEC-158, and the success criteria.
3. `docs/reviews/2026-11-10-external-code-review.md`: the finding and its evidence.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for".
   If the file isn't in your checkout, say so.
5. `scripts/ci-code-paths.sh`, `.github/workflows/ci.yml` (the `changes` job and what `needs` it),
   `.github/workflows/pages.yml`, `.github/workflows/publish-*.yml`.
6. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code**, as in question 1, with the output quoted.
- **Make the path table** (question 2) from commands, and paste the commands. Mechanical sweeps need
  a mechanical check, and the scope of the grep is itself a claim.
- **Size the verification, not just the code** (AGENTS §2): paths × classifier designs × workflows ×
  the self-test and its mutations.
- **Rough build file list** (paths).
- **Questions only the maintainer can answer:** one line each, with your recommendation. The
  allow-list versus deny-list choice is likely one.

### Allocate the id
Run `just new-spec "<title>" STAGE-063 PROJ-011`. If it gives anything other than SPEC-154,
`git mv` the spec and its timeline to SPEC-154 and fix the id inside both. Then run
`just advance-cycle SPEC-154 frame`.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`, and
  `cycle: frame`.
- `## Context` and `## Goal`: what goes wrong today, and what is true when this ships.
- A `## Frame` section directly after `## Goal`: the verdict (go / no-go / split); the premise check
  with evidence; the path table and the design you recommend; the self-test and mutation rows; the
  other workflows; what only a PR can prove; sizing and why; scope in and out; the rough build file
  list; open maintainer questions.
- `### Out of scope (for this spec specifically)`.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary, and add `[ ]` lines for design, build,
  verify and ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), with no PR.
1. On **go**, run `just advance-cycle SPEC-154 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-063's backlog, change your item's marker from
   `- [ ] **SPEC-154** (reserved 2026-11-10)` to `- [ ] **SPEC-154** (framed <date>)`, and fold or
   leave other entries as you recommend. File an out-of-spec finding as a new
   `- [ ] (not yet written)` entry in the stage it belongs to, and update that stage's `**Count:**`
   line with a grep you've just run. Something with no stage goes in `guidance/ideas.yaml`.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Message:
   `docs(SPEC-154): frame the CI path classifier`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back what you pushed: `git show --stat HEAD` and the STAGE-063 diff.
   `just backlog` reads only the active project, so use `just status` and a grep as well.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag. A probe you drive at frame lives in a
  scratch directory and is never committed. Do not edit the classifier or any workflow.
- Other sessions may be working at the same time. Never share a `CARGO_TARGET_DIR`. If
  `git rebase origin/main` conflicts in a file outside this spec's, stop and report.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate `grep` and
  `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Keep scratch files in your session's scratchpad, never in `~` or `/tmp`.
- **Budget ~30 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. Identify it by content, not
recency: `/usr/bin/grep -l "SPEC-154 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

Dedup by `.message.id`: lines that share an id repeat the same `input_tokens`,
`cache_creation_input_tokens` and `cache_read_input_tokens`, while `output_tokens` grows. Take the
three once per id and the **MAX** `output_tokens`. Price each component at the model that ran
(`.message.model`): Opus $5/$25 per MTok, Sonnet $3/$15; cache_creation ×1.25 the input rate,
cache_read ×0.10. Measure at your last turn, after the push, and append a `frame` entry to the
spec's `cost.sessions` with `tokens_total`, a `tokens_breakdown`, `estimated_usd` and the rate
anchors, in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-154 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`.
Then, short:
- the premise check, with the classifier's output quoted for the four paths;
- the path table's verdict: how many paths the classifier gets wrong, and which;
- the design you recommend (allow-list or deny-list) and why;
- the self-test rows and the mutation rows;
- what `pages.yml` and the publish workflows need;
- what only a PR can prove, and the smallest PR that proves it;
- fold-ins: folded or not, and why;
- size and why, scope in/out, and any split;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
