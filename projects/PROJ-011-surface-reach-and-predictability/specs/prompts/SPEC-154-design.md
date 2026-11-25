# SPEC-154 — DESIGN prompt: the CI path classifier runs the gates for every build and test input

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-154-design -- --model opus
```

This session starts from the stub the frame pushed to `main` (`815fc90`, cost in `ed05ccf`, rulings
recorded after). **No new DEC is expected.** This is CI plumbing. If you find one is needed after
all, don't name a number: the other STAGE-063 designs may run alongside yours and claim ids. Put it
in your report as a maintainer question.

**Read the stub's `## Frame` section first.** It's your binding scope. It holds:
- the premise: a PR touching only `recipes/web.toml` shows the three required `check` legs green
  with nothing built or tested, and `pages.yml` doesn't trigger;
- the full path table (20 misclassified paths in 9 groups) and how it was derived;
- the allow-list shape, its replay over the last 600 commits, and its cost (12 flips, 2 of them
  allow-list overhead);
- the self-test rows and the mutation rows driven in a scratch probe;
- pages' two `paths:` lists and the drift check;
- what only a PR can prove, and the post-merge draft PR.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables.**

## Rulings (settled — do not re-open)
The maintainer accepted all three of the frame's recommendations on 2026-11-22 (recorded in the stub
under `### Maintainer rulings`):
1. **The docs-only allow-list.** Anything not provably docs-only runs the gates, plus a short
   exception list for docs-shaped files a test reads.
2. **`README.md` is a code path.** README-only PRs run the full gates.
3. **A short `docs/development.md` paragraph** on which PRs skip the Rust gates and why. In scope
   now, not optional.

Also in scope from the frame: the derived self-test row (every `include_str!`/`include_bytes!`
target classifies as code), `recipes/**` and `assets/**` in both of `pages.yml`'s lists, and the
pages drift check. Out of scope: which jobs are required, the publish workflows, building `fuzz/`,
branch protection.

## Order and neighbours
SPEC-154 **lands first** in STAGE-063: SPEC-155, SPEC-156 and SPEC-157 build after it merges, so
their PRs run under the fixed classifier. Their designs may run alongside yours. They touch `src/`,
`tests/` and docs, not `scripts/ci-code-paths.sh` or the workflows, but check: at Step 0, run
`gh pr list --state open --json number,title,files` and `git branch -r`, and list every file another
open PR or pushed `feat/spec-15[5-8]*` branch touches that this spec also touches. Write that map
into Implementation Context **from their file lists, not their specs' text**. On a rebase conflict
in STAGE-063's file, keep both sides and recount the `**Count:**` line with a fresh grep. On a
conflict in any other file, stop and report. Never edit `projects/PROJ-017-geometry-as-recipe-steps/`.

## Things in the frame to check before you build on them
The frame drove its rows in a scratch probe, not in the real script. Re-drive these against the
real `scripts/ci-code-paths.sh` on a prototype branch (never pushed):
- **Every path the frame calls a test input.** For each exception and each newly-code path, quote
  the line in `src/`, `tests/`, `build.rs` or a workflow that reads it (for example `tests/lint.rs`
  reading `docs/github-actions.md`, `tests/adoption_glue.rs` reading `README.md`). A path with no
  reader stays code under the allow-list anyway, but the exception list must hold only paths that
  are really read.
- **The derived row is really mechanical.** List every `include_str!`/`include_bytes!` in `src/`,
  `tests/` and `build.rs`. Include `concat!(env!("CARGO_MANIFEST_DIR"), …)` forms and paths relative
  to the including file, and show how the script resolves each to a repo path. If any form can't be
  resolved by grep/sed in a shell script, say which, and decide: resolve it by hand in a list, or
  refuse the form. Then do the negative control: add a fake `include_str!("../docs/x.md")` to a
  scratch copy and show the row goes red.
- **Runtime reads.** The frame says reads built with `format!`/`join` can't be derived and stay
  hand-listed. Sweep for them (`fs::read`, `read_to_string`, `CARGO_MANIFEST_DIR`, `Path::new` with a
  literal under a docs-shaped root) and paste the commands. Every hit under a docs-only root becomes
  an exception or is shown not to be one.
- **Where the self-test runs.** Read `ci.yml`: is `--self-test` run on every PR today, from which job,
  and is that job ungated? The new rows and the drift check are only a guard if they run on a
  docs-only PR too. Say where they run after the change.
- **`cargo-deny` runs on every PR.** The frame says so. Quote the job and its `if:`.
- **The replay.** Re-run the 600-commit replay with the final regex, not the probe's, and give the
  flip count. If it moved from 12, say which commits and why.
- **Pages.** Confirm the wasm demo really embeds `recipes/*.toml` and `assets/fonts/Go-Regular.ttf`
  (quote the `include_*!` sites), and that `pages.yml`'s two lists are the only lists that need the
  addition (grep every workflow's `paths:`).

## Design calls that are yours
- **The script's shape.** The final `DOCS` and `EXCEPT` patterns, verbatim, with a comment for each
  alternative saying why it's docs-only. The derived row, the drift check, and how each prints its
  failure. Name each piece with the `file:line` it changes. Choose shell tools (grep/awk, or Ruby as
  `validate-frontmatter.sh` uses) and say why.
- **The drift check's two halves:** pages' `push.paths` and `pull_request.paths` are identical, and
  every pages path prefix classifies as code. Say which job runs it and that it runs on a docs-only
  PR.
- **Every message, verbatim,** that the script prints: the `changes` job's two log lines (code paths
  touched / docs only), each self-test failure line, each drift-check failure line. One AC row and
  one self-test row or check per message.
- **Comments.** The script header and the `changes` job comment in `ci.yml`, verbatim. They state the
  rule ("anything not provably docs-only runs the gates") and the exception rule. Behaviour-first, no
  spec, stage or DEC ids.
- **`docs/development.md`.** The paragraph, verbatim, and where it goes (cite the heading). About
  three sentences: which PRs skip the Rust gates, why, and that a new top-level directory runs them
  until someone classifies it.
- **CHANGELOG.** Does a CI-only change get a line? Read how past CI changes were handled
  (`/usr/bin/grep -n -i 'ci\b\|workflow' CHANGELOG.md`) and follow that. If yes, the line verbatim.
- **Size.** The frame says [S]. Say whether that holds, with the count's derivation next to it.

## Tests: each one must be able to fail
HANDOFF's traps and AGENTS §15 bind here. The tests are the script's `--self-test` rows and checks.
**Drive every mutation row at design on the prototype; don't reason them.** Each row **quotes its
exact mutant edit**, names every self-test row expected red, and is marked *driven*. Read the red set
**off the prototype's full self-test run**, not off the frame's probe. At least:
- each `DOCS` alternative deleted, one at a time;
- each `EXCEPT` alternative deleted, one at a time;
- the leading `^` dropped; `[^/]+\.md$` widened to `.*\.md$`;
- a code directory added to `DOCS` (for example `src/`);
- the derived row disabled (the include scan returns nothing): which row goes red? If none, the
  derived row needs its own "found at least N includes" floor, with N derived;
- the drift check: the two pages lists made unequal; a pages path prefix that classifies as docs.

A negative control for the whole self-test: show a run where every row passes, then one deliberate
wrong verdict that makes it exit non-zero, with the exit code quoted.

**Every count in the spec carries its derivation** (the command, or the sum), next to the number.

## What the build can't prove, and ship's step
The build PR touches `scripts/` and `.github/workflows/`, which are code under both designs, so it
can't prove the classification. Write into Implementation Context, for ship:
- after merge, the maintainer opens a **draft PR touching only `recipes/web.toml`** (a comment line),
  checks once that `changes` prints the code line, the three `check` legs run `cargo test`, and
  `pages` runs, then closes it unmerged;
- the next docs-only push to `main` should still print the docs-only line.
No polling, no wait loops (AGENTS §16 habit 6). The maintainer checks in.

## Rules every design session follows

**You are the design cycle only.** You write Acceptance Criteria, Failing Tests and Implementation
Context. You don't write the code or any DEC. Separate build and verify sessions follow: one builds
the spec from your text alone, and another verifies it, so everything the builder needs must be in
the spec (AGENTS §15).

### Step 0 — get onto current `main`
Run `git fetch origin`, then `git rebase origin/main`. If `git status` shows you are in the
maintainer's main checkout rather than a worktree, stop and say so. Then build the overlap map above.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 and §16 habit 6.
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for".
3. `projects/PROJ-011-surface-reach-and-predictability/stages/STAGE-063-honest-output-and-the-0-8-3-release.md`:
   your entry and SPEC-155 to SPEC-158.
4. `docs/reviews/2026-11-10-external-code-review.md`: the CI finding.
5. `scripts/ci-code-paths.sh`, `.github/workflows/ci.yml`, `.github/workflows/pages.yml`, and every
   other workflow's `paths:`.
6. `/guidance/constraints.yaml`.
7. A shipped PROJ-011 spec in `specs/done/` (SPEC-150 is a recent one) as the shape to match: ACs,
   mutation tables, Build Completion.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`.
- **Drive the current behaviour** on `main`: run today's script on each path in the frame's table and
  quote its verdict. Give every measurement a negative control.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  check per thing.
- No version bump.

### Ids
Don't run `just new-spec`. Fill in Acceptance Criteria, Failing Tests and Implementation Context,
and keep `## Frame`. If you find the frame wrong, say so in a short `### Design corrections to the
frame` subsection at the top of `## Design`, and in your report.

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-154 build`.
2. In STAGE-063's backlog, change SPEC-154's marker from `(framed 2026-11-22)` to
   `(designed <date>)`, keeping `[S]` (or the size you found). Mark design `[x]` in the timeline with
   a 2–4 line summary, and add `prompts/SPEC-154-build.md` to the build line.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Message:
   `docs(SPEC-154): design the docs-only allow-list for CI`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`. On a reject,
   rebase and push again; never force.
5. Read back `git show --stat HEAD` and the STAGE-063 diff.

### Guardrails
- ⛔ No code changes outside the spec, the timeline and the stage file. No DEC. No pushed probe
  branch, no draft PR (that's ship's, after merge). No version bump, no tag. Don't raise or plan a
  release.
- `rtk` can truncate `grep` and `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for
  anything you count. A piped command reports the pipe's exit code, so redirect output and read `$?`.
  zsh does not word-split unquoted parameters; brace `${R}:`. macOS has no `timeout(1)`.
- The classifier runs on Linux in CI. If you use a grep, sed or awk flag, check it behaves the same
  on GNU and BSD, or run the self-test under the tool CI uses and say how.
- Write scratch files in your session's scratchpad, never `/tmp` or the system temp dir. Remove the
  prototype worktree before you finish.
- **Budget ~35 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`/usr/bin/grep -l "SPEC-154 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Lines that share an id repeat the same `input_tokens`,
`cache_creation_input_tokens` and `cache_read_input_tokens`, while `output_tokens` grows. Take those
three once per id and the **MAX** `output_tokens`. Include any `subagents/` transcripts.

Price each component at the model in `.message.model`: Opus $5/$25 per MTok, Sonnet $3/$15.
cache_creation is ×1.25 the input rate, and cache_read is ×0.10. Measure at your last turn, after the
push, and append a `design` entry to the spec's `cost.sessions` in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-154 design: DONE | SPLIT | BLOCKED`, then the commit SHA on
`main`. Then, short:
- any correction to the frame, with what you drove (test-input readers, the include forms, runtime
  reads, where the self-test runs, cargo-deny, the replay count, pages' embeds);
- the final `DOCS` and `EXCEPT` patterns, verbatim;
- the script's pieces, one line each with `file:line`;
- the AC list (one line each);
- the self-test rows and the mutation table, each row with its quoted edit, red rows, and driven;
- every exact message the script prints;
- the comments, the `docs/development.md` paragraph, and the CHANGELOG decision, verbatim;
- ship's post-merge draft-PR check, as written into the spec;
- the overlap map with other open PRs and branches, from their file lists;
- size with its derivation, and anything the build prompt must carry (checkpoints, CI legs);
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
