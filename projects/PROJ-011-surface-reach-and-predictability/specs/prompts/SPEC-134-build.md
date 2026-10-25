# SPEC-134 — BUILD prompt: a `--name-template` extension pins the output format

Cycle: **build**. Fresh CLI session, own worktree, **Opus** (a 41-row roster across nine verbs
plus precedence and must-not-change rows; thoroughness across the roster is the job):

```
just wt feat/spec-134-name-template-pins-format -- --model opus
```

You did not design this. The spec carries the context (AGENTS §15). The maintainer pasted this
prompt into a fresh session and will paste your final message back to the orchestrator, so that
message must stand on its own.

**What you are building:** a recognised extension in `--name-template` pins the output format on
every verb that encodes, at one input and at many, and on plain-recipe `build` targets, exactly as
a recognised `-o` extension does (DEC-105, which extends DEC-087). On `web` / `optimize` /
`apply --recipe web` that pin skips the automatic choice. `meta` refuses (exit 4, before writing)
when a template **or `-o`** names a different format than the source. `{ext}` templates and
extensionless or unrecognised ones stay byte-identical to `main`.

## Step 0 — get onto current `main`
`git fetch origin && git rebase origin/main`. The spec must be at `cycle: build`, its design
commit `0d4d643` must be in your history, and its AC list must include **AC-22** (added after
design). If any of those is missing, stop and report.

## Read in order, before any code
1. **The spec** — `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-134-name-template-extension-pins-the-output-format.md`.
   Read `## Frame`, `## Design`, all 22 ACs, `## Failing Tests` (including the NC table), the
   Roster and function list in `## Implementation Context`, and `## Notes for the Implementer` in
   full. **Its design calls are settled.** If one turns out wrong, say so in Build Completion and
   your report; do not quietly re-decide it.
2. **DEC-105** (on `main` already, written at design) and **DEC-087**. Read both; do not trust a
   summary of either. DEC-064, DEC-098, DEC-099, DEC-015, DEC-058/DEC-101.
3. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md` — "Read first" and "Traps this
   wave paid for". Binding. `/guidance/constraints.yaml` (`metadata-not-via-pixel-encode`).

## Maintainer rulings (settled — do not re-open)
1. Honour the template: a recognised extension pins, like `-o`, on every encoding verb, both
   arities, and plain-recipe `build` targets.
2. On `web` / `optimize` / `apply --recipe web` a template pin skips the automatic choice.
3. `meta` refuses with exit 4 before writing when the template names a different format.
4. **(2026-10-25, after design) `meta … -o x.png` refuses the same way — AC-22.** Reuse the AC-12
   pre-flight; do not add a second check. Add its test and its guard rows; NC-6 (delete the
   pre-flight) must flip it too. Update the roster/row counts if you put it in the table, and keep
   the count assertion honest.

## Order
1. **Write every failing test first, run against unmodified `src/`**, and record that each RED
   test is RED for the reason the spec gives and each GUARD is GREEN. The fixture JPEG is built
   with the `image` crate's encoder, never with crustyimg (a fixture made by the code under test
   cannot fail). Confirm the AVIF-gated guard is **run**, not filtered out.
2. Implement: `sink::template_literal_format` → `common::name_pinned_format` → the ~8 call sites →
   `build`'s lenient plain-target plan (strict `target_format_plan` stays for terminal-`optimize`)
   → `meta` pre-flight (template and `-o`).
3. **Push a WIP commit as soon as it compiles**, before the matrix (`git commit -s`, push).
4. Negative controls NC-1…NC-10 one at a time, binary rebuilt each time, flips recorded.
   ⚠ Reverting source does not rebuild the binary.
5. Clean matrix, docs (AC-20: the `grep -n 'does \*\*not\*\* pin' docs/api-contract.md` check
   must return 0 hits), PR.

## The things most likely to go wrong
- **The must-not-change rows.** `{ext}` templates stay byte-identical to `main`, and plain `build`
  with `name = "{stem}"` or `"{stem}.txt"` still exits 0. Reusing the strict
  `target_format_plan` for plain targets would make those exit 4; that is why the spec adds a
  lenient one.
- **The cache key.** A plain target that now pins must hash differently from a default target on
  the same recipe (the design drove both at `65304eca…` on `main`). Pin it with the spec's test.
  No `CACHE_SCHEMA_VERSION` bump: the key already includes the crate version.
- **The `meta` mixed batch** (AC-13): `a.png` listed first must **not** be written. The check runs
  over every input before any write; moving it into the per-input loop (NC-7) is exactly the bug.
- **Stay in your functions.** The spec lists the functions per file; `watch_impl`,
  `target_recipe_hash`, `encode_one` and `build_one` are **not** yours. SPEC-132 is editing
  `watch_impl` / `register_roots` in `src/cli/build.rs` and may merge first: rebase, do not
  reflow. SPEC-131 (`src/operation/**`, `src/recipe/**`) may merge first too. You build **before**
  SPEC-133, which edits the same `run_pixel_op` / `run_apply` you do and will rebase onto you.
  Append your `CHANGELOG.md` / `docs/api-contract.md` text without reflowing others'.
- **Users will notice** (say so in the CHANGELOG, as the spec lists): `--json` with a template pin
  on `web`/`optimize`/`apply --recipe web` becomes exit 2 (as with `-o` today); `optimize
  --profile preserve` now pins on a literal template; a template-pinned `web` prints no summary
  line, same as `-o`.

## Guardrails
- ⛔ No version bump, no release, no cache-schema or lockfile change. PROJ-011 ships as ONE release
  (0.8.0); merging is fine, tagging is the maintainer's.
- **Every commit `git commit -s`.** The "DCO sign-off check" fails any PR with an unsigned commit.
- **Never poll CI.** Run `gh pr checks <PR> --watch` in the background; when it exits, read a direct
  snapshot at the PR's true head SHA and check every leg, including windows-latest.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to
  a file. A piped command reports the **pipe's** exit code: redirect and read `$?`.
- zsh does not word-split unquoted parameters: loop with `while IFS= read -r`. macOS has no
  `timeout(1)`.
- Matrix legs **sequentially**, each with a fresh `CARGO_TARGET_DIR` (default,
  `--no-default-features`, `--features webp-lossy`); clippy `-D warnings` + `fmt --check` each.
  Never run a background build while editing source for a revert.
- Comments and user-facing text are behaviour-first, with no spec/DEC ids.
- **Budget ~150 exchanges.** If the matrix is green and you are past that, stop and report what is
  left.

## When you finish
1. Fill `## Build Completion`: every file from `git diff --name-only origin/main...HEAD` (not from
   memory), the RED baseline, the NC flips, the matrix legs, and an honest reflection naming any
   design call that turned out wrong.
2. **Measure your own cost.** Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where
   `<slug>` is your working directory's absolute path with every `/` and `.` replaced by `-`.
   Identify it **by content, not recency**: `grep -l "SPEC-134 — BUILD prompt" ~/.claude/projects/<slug>/*.jsonl`.
   ⚠ **Dedup by `.message.id`**: Claude Code writes one JSONL line per *content block*; lines sharing
   an id repeat identical `input_tokens` / `cache_creation_input_tokens` / `cache_read_input_tokens`
   while `output_tokens` grows. Take those three once per id and the **MAX** `output_tokens`.
   Summing every line over-counts ~2×; taking the first line's output under-counts ~11×; both
   mistakes are on record. Price each component at the model that actually ran (Opus $5/$25 per
   MTok, Sonnet $3/$15; cache_creation ×1.25 the input rate, cache_read ×0.10), never a flat rate.
   Append a `build` entry to `cost.sessions` with `tokens_total`, a `tokens_breakdown`,
   `estimated_usd` and the rate anchors.
3. `just advance-cycle SPEC-134 verify`. Open the PR: title
   `feat(cli): a --name-template extension pins the output format (SPEC-134)`, body per AGENTS §13
   (Summary, Spec metadata, Decisions referenced: DEC-087 extended by DEC-105, Constraints checked,
   New decisions: DEC-105, written at design), ending with the Claude Code footer.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-134 build: PR #NNN at <head SHA> — CI <green | red: legs>`. Then, short: AC
completion (22 rows); RED baseline confirmed; NC flips; the table's final row count; matrix
result; files changed vs the spec's list; any design call that turned out wrong; your measured
cost.
