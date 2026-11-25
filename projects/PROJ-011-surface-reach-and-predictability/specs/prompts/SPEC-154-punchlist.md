# SPEC-154 — PUNCH LIST prompt: ship's proof edit, and what the include check covers

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. One wording item and one
assert item. Neither changes a classifier verdict, so no re-verify follows. You did not design,
build or verify SPEC-154.

```
just wt fix/spec-154-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin feat/spec-154-ci-allow-list:refs/remotes/origin/feat/spec-154-ci-allow-list
git checkout --detach origin/feat/spec-154-ci-allow-list
```
Push to the PR branch with `git push origin HEAD:feat/spec-154-ci-allow-list`. Draft PR #266's head
was `2a70156` when this was written. If it has moved, stop and report.

The maintainer pastes this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own. **Write it as plain lists, not box-drawn tables.**

## Why
Verify checked every tracked path through both scripts: 0 paths moved from code to docs-only, and
39 moved to code, exactly the design's list. It traced every command each gated job runs and found
no docs-only file any of them reads, apart from the three exceptions. It re-drove six mutations,
and the Linux log shows 88 rows, 0 failed. It found two things to fix:

1. **Ship's proof edit turns pages red.** The spec's "What the build cannot prove, and ship's step"
   tells the maintainer to add a comment line to `recipes/web.toml` and check that `pages` ran.
   But `tests/demo_smoke.mjs:853-857` asserts the demo's inlined recipe is byte-identical to
   `recipes/web.toml`, so that edit fails the pages smoke. The Rust tests survive it
   (`recipe_round_trip.rs` re-emits without comments).
2. **The include check is narrower than two sentences claim.** AC-9 says "An include the scan
   cannot resolve is refused", and `docs/development.md` says "the self-test fails if a build or a
   test includes a file the list calls docs-only". Verify drove four forms the scan doesn't see,
   all green: a raw string (`include_str!(r"../docs/development.md")`), a space before the
   parenthesis, a multi-line `include_str!(`, and `include!("../docs/x.rs")`. Today's tree uses
   only the plain form, so every verdict is right today.

## The fixes (settled, do not re-decide)
1. **Ship's proof edit.** Pick a proof that stays green if one exists:
   - grep `tests/`, `scripts/`, `demo/` and `integrations/` for anything that compares
     `recipes/gallery.toml` or `recipes/product.toml` byte for byte, or reads its comments;
   - if one of them is compared by nothing, change ship's step to add the comment line to **that**
     file. `pages` still triggers (`recipes/**`), the three `cargo test` legs still run, and nothing
     should go red;
   - if both are compared, keep `recipes/web.toml` and say plainly that the pages smoke fails on
     the verbatim-recipe check (`tests/demo_smoke.mjs:855`), and that this failure is itself the
     proof `pages` ran.

   Quote the grep and say which you chose. Edit only that section of the spec.
2. **Narrow the claims; don't widen the script.** The orchestrator chose narrowing over widening
   `unresolvable_includes`, so the script stays byte-identical to the spec's block.
   - In `docs/development.md`'s new section, narrow the include sentence to the form the scan
     reads: a plain string literal in `include_str!` or `include_bytes!`. Add one sentence saying
     a file a test reads at run time, or includes any other way, must be added to the exception
     list by hand.
   - Then update the spec's verbatim copy of that paragraph to match, byte for byte.
   - In the spec, narrow AC-9's wording to what the check does. It refuses an include whose path is
     built with `concat!`/`env!`. Say that other forms aren't scanned.
   - Read the script header (`scripts/ci-code-paths.sh`, about lines 2-32). Verify says it already
     states the runtime-read limit honestly (about :30-31). If its include sentence claims more than
     the plain form, **don't edit the script**. Put the exact sentence in your report, and the
     orchestrator will decide.
3. No spec, stage or DEC id in any changed line.

Stay in these files: `docs/development.md` (the new section only) and the SPEC-154 spec (ship's step,
AC-9, the verbatim paragraph, Build Completion and `cost.sessions`). `git diff --stat 2a70156` must
show nothing else. **`scripts/ci-code-paths.sh` must not change**; show `git diff 2a70156 --
scripts/` is empty.

**Not yours:**
- verify's out-of-spec findings, which the orchestrator has filed on `main`: the rename hole in
  STAGE-063, and the quoted non-ASCII paths and the include floor's drift in `guidance/ideas.yaml`;
- the optional fixture-path rename at `:161-162`;
- the CHANGELOG.

Don't touch them.

## Check it
- Run `bash scripts/ci-code-paths.sh --self-test > out.txt 2>&1; echo rc=$?`. It must give rc=0 and
  88 rows (unchanged, because the script didn't change).
- For each sentence you changed in `docs/development.md`, drive what it says through stdin or a
  scratch include, and quote the result. Include one driven example of the raw-string form staying
  green, so the narrowed sentence is shown true.
- Run `/usr/bin/grep -nE '(SPEC|STAGE|DEC|PROJ)-[0-9]'` on your changed lines in
  `docs/development.md`. It must find nothing.
- No Rust changes, so no cargo run and no mutations. CI runs on push.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag.
- The PR is behind `main` by docs-only commits. Rebase onto `main` before you push, so it can merge
  under strict protection. Never force-push over someone else's commit: if the PR's head isn't
  `2a70156` when you push, stop.
- **Never poll CI.** Run `gh pr checks 266 --watch` once in the background after your push, and
  don't write your own wait loop. When it exits, take one snapshot at the PR's head SHA, every leg.
- A piped command reports the pipe's exit code, so redirect and read `$?`. zsh does not word-split
  unquoted parameters. `rtk` can truncate `grep`, so use `/usr/bin/grep` for anything you count.
- Write scratch files in your session's scratchpad, never `/tmp` or the system temp dir.
- **Budget ~20 exchanges.**

## When you finish
- Add a short `### Punch list (verify)` subsection to the spec's `## Build Completion`. It holds:
  - ship's step as it now reads, and the grep that chose its file;
  - the `docs/development.md` sentences and AC-9 as they now read, with each drive;
  - the self-test line;
  - the CI snapshot.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `/usr/bin/grep -l "SPEC-154 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - Dedup by `.message.id`. Take the input and both cache counts once per id, and the **MAX**
    `output_tokens`.
  - Price at the model in `.message.model`: Sonnet $3/$15 per MTok, Opus $5/$25. cache_creation is
    ×1.25 the input rate and cache_read ×0.10.
  - Append a `punchlist` entry to the spec's `cost.sessions` with `tokens_total`, a
    `tokens_breakdown`, `estimated_usd` and the rate anchors. Also record verify's cost there as a
    `verify` entry, from these numbers: input 82, cache_creation 132,899, cache_read 5,026,203,
    output 31,258, total 5,190,442, $4.13, Opus.

## Report back (your final message — it is pasted to the orchestrator)
Plain lists only. The first line is `SPEC-154 punch list: PR #266 at <head SHA> — CI <green | red: legs>`.

Then, briefly:
- `git diff --stat 2a70156`, and that `scripts/` is unchanged;
- ship's step as it now reads, and which recipe file it uses, with the grep;
- each changed sentence and AC-9, quoted as they now read, with their drives;
- the script header's include sentence, quoted, if it claims more than the plain form;
- anything that did not go as written;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
