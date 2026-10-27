# SPEC-134 — FIX 2 prompt: a glob row for the N-input pin, on the same draft PR #202

Cycle: **build** (second punch-list follow-up, small). Fresh CLI session, **Sonnet**, on the PR's
own branch:

```
just wt feat/spec-134-name-template-pins-format -- --model sonnet
```

(That reuses the existing worktree; close any old session in it first.) You did not build, fix or
verify SPEC-134. Its re-verify at `a3547cc` found the behaviour, docs and CI correct, and one test
gap. Fix exactly the items below on branch `feat/spec-134-name-template-pins-format`, pushing to
the **same draft PR #202**. The maintainer pasted this prompt into a fresh session and will paste
your final message back to the orchestrator, so it must stand on its own.

## Step 0
`git fetch origin && git status` (must be clean), then `git pull --ff-only`. `git log -1` should
show `a3547cc` or later. Then `git merge origin/main` (not rebase: the PR is shared). `main` has
only docs changes since. Read `src/cli/common.rs` (`name_pinned_format`, `resolved_input_count`)
and `tests/name_template_pins_format.rs` (the `ROWS` table and its N-input rows).

## 1 — Required: an N-input row whose inputs come from a glob
Every N-input row passes two literal paths, so the args count already equals the resolved count.
Re-verify's mutation M2 (`resolved_input_count` returns `inputs.len()`) left the full suite green,
yet the binary then writes AVIF into `in.png` for
`web 'in*.jpg' -o x.bin --out-dir d --name-template '{stem}.png'`, and JPEG for
`apply --recipe web 'in*.jpg' -o x.jpg …`.
- Add rows where **one glob argument** (`in*.jpg`, passed to the binary unexpanded; the test's
  `Command` has no shell) resolves to two inputs: at least `web … -o x.bin` and
  `apply --recipe web … -o x.jpg`, asserting PNG sniffed in both outputs and no `x.*` written. A
  directory argument row is welcome if the fixture allows it cheaply.
- **Prove it:** apply M2, rebuild in a **fresh** `CARGO_TARGET_DIR` (reverting source does not
  rebuild the binary), confirm exactly the new rows go RED; restore, rebuild, GREEN. Update the
  row-count assertion (46 → N) and the roster note in the spec's `## Build Completion`.

## 2 — Cleanup (orchestrator's call): one source of the N-input rule
`refuse_a_name_asking_for_another_format` (`src/cli/ops.rs`) has its own N-input branch. It
repeats `name_pinned_format`'s template branch. Make it call `name_pinned_format(global,
all.len())` for both branches. Keep the `via` label logic: `-o` only when `all.len() == 1 &&
global.output.is_some()`, else "the name template". No behaviour change. The existing `meta`
refusal tests (`meta_dash_o_*` and the meta rows) must stay green unchanged. Say in your report
which tests cover the N-input `meta` path. If none does, add one row.

Not in scope: resolving inputs once instead of twice (the orchestrator filed it as a backlog
item).

## Boundaries
Only `tests/name_template_pins_format.rs`, `src/cli/ops.rs` (that one function), and the spec's
`## Build Completion` / `cost.sessions`. No docs or CHANGELOG change (no user-visible change). Do
not edit the spec's timeline (it lives on `main`).

## Finish
- Clean matrix: default, `--no-default-features`, `--features webp-lossy`. Give each leg a
  **fresh** `CARGO_TARGET_DIR` and run them **one after another**. Pass feature flags as
  separate literal arguments; zsh does not word-split. Each leg runs the tests and `clippy
  --all-targets -D warnings`. Then run `fmt --check`. **Delete each target dir when its leg is
  done:** the last fix filled the disk.
- Every commit `git commit -s` (DCO). Push the branch. ⛔ The PR stays a **draft**; never mark it
  ready.
- **Never poll CI.** Background `gh pr checks 202 --watch`; when it exits, read a direct snapshot
  at the true head SHA and check every leg.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to
  a file. A piped command reports the pipe's exit code. macOS has no `timeout(1)`.
- **Budget ~40 exchanges.**

## Measure your own cost
Transcript: a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working directory's
absolute path with every `/`, `.` and `_` replaced by `-`; find it by content:
`grep -l "SPEC-134 — FIX 2 prompt" ~/.claude/projects/<slug>/*.jsonl`. Dedup by `.message.id`
(input / cache_creation / cache_read once per id, MAX output_tokens), price per component (Sonnet
$3/$15 per MTok, cache_creation ×1.25, cache_read ×0.10), never flat. Append a `build` entry to
the spec's `cost.sessions` on this branch, and report tokens_total, the breakdown and
estimated_usd.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-134 fix 2: draft PR #202 at <SHA> — CI <green|red>`. Then:
- the new rows, and their RED under M2 / GREEN after, with the fresh-target-dir evidence;
- the `ops.rs` change and which tests cover the N-input `meta` path;
- the matrix;
- the files changed;
- your cost.
