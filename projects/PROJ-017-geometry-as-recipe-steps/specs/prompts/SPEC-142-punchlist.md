# SPEC-142 — PUNCH LIST prompt: a classifier crash must fail CI, and the preview path gets a golden

Cycle: **build (punch list)**. Fresh CLI session, own worktree, **Sonnet**. Small and mechanical:
one `ci.yml` line, one golden literal, a few printed lines, and notes. You did not design, build
or verify SPEC-142.

```
just wt fix/spec-142-punch -- --model sonnet
```

Put the worktree on the PR's code. The fresh branch has nothing on it to lose:
```
git fetch origin test/spec-142-ddphotos-parity:refs/remotes/origin/test/spec-142-ddphotos-parity
git reset --hard origin/test/spec-142-ddphotos-parity
```
Push to the PR branch with `git push origin HEAD:test/spec-142-ddphotos-parity`. Draft PR
#234's head was `1b1c010` when this was written. If it has moved, stop and report.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so it must stand on its own.

## Why
Verify passed the comparator, the harness, the fixtures, the page and the parity job. Every
mutation row matched, K = 1 reproduced, and a second x86 CPU (an AMD EPYC 7763) gave identical
numbers. Two items fail.

1. **A crashing classifier skips CI silently.** The `changes` step in `.github/workflows/ci.yml`
   compares `"$(echo "$files" | scripts/ci-code-paths.sh)"` with `"true"` inside `[ … ]`.
   - Under `bash -e`, a command substitution that fails inside a test doesn't stop the step.
   - Verify made the script fail only in its classify path, after the self-test passed. The step
     printed `docs/specs only: heavy jobs will skip`, set `code=false` and exited 0, so every
     heavy job would skip and show green.
   - This PR introduced that path.
2. **The preview recipe has no golden text.**
   - Verify made `previewRecipe` return its input unchanged, dropping `quality = 88`. Both modes
     stayed green, because native and wasm move together on rows 9–12.
   - P5 exists for exactly this, but pins only rows 4 and 8.

## The fix (settled, do not re-decide)
1. **`ci.yml`, the `changes` step only.**
   - Assign first, then compare:
     `code=$(echo "$files" | scripts/ci-code-paths.sh)`.
   - Then require the value to be exactly `true` or `false`. Anything else prints the value and
     exits 1.
   - Then branch on `"$code"`.
   - Keep `scripts/ci-code-paths.sh --self-test` before it. Change nothing else in the workflow.
2. **P5 gains a third literal:** row 12's recipe text, `previewRecipe(pixelRecipe(both))`. That's
   the row-4 text with `quality = 88`, wherever `withQuality` puts it.
   - Paste it from a real run of `recipe.js`; don't type it.
   - Add it to the spec's "Golden recipe texts (FT-8)" section, and to FT-8 / AC-5's P5 wording
     ("rows 4, 8 and 12").
3. **The harness also prints the native SHA-256 on each row line,** next to the wasm one. Then the
   cross-target check can be read straight from the CI log. Keep the `PASS|FAIL <id> …` prefix as
   it is, and update the spec's one-line-format note.
4. **The harness prints one summary line:** `largest differing-pixel count on a resampling row:
   <n> (K = <K>)`. That way a K raised without new evidence can be seen in review. It's
   informational, not an assertion.
5. **Text only, in the spec and DEC-110:**
   - **The mutation table:** the B-edge row also turns **B2** red when K = 1, because B2 equals
     B4 then. The B-dims row turns only **B7** red; N2's rows are caught by the per-row size
     literals, not by B-dims. Add a note for the next spec that changes K or adds rows.
   - **DEC-110's Validation section** gets three lines, and nothing outside Validation changes:
     - The second x86 CPU, the EPYC 7763 from run `41147581056`, agreed with the EPYC 9V45.
     - **K is guarded by review only.** Raising K up to the cap turns nothing red, so a slide from
       1 pixel to 78 would pass. The summary line in fix 4 makes that visible.
     - **The measured remedy.** Verify swapped `libm::powf` into native's `srgb_to_linear` and
       `linear_to_srgb` on aarch64, and all 16 rows came out `px=0 max=0` and byte-equal. Both
       powf sites contribute: the decode table alone fixed row 15 only. That's a revisit trigger:
       if native adopts a deterministic powf, re-measure and set K = 0.
   - **Build Completion:** the head run's CPU was the EPYC 7763, not the 9V45 the first run
     recorded. Note it.
6. **AC-1's owned list in the harness** gains `planEdit`, `GO_ADVANCE` and `GO_WIDEST`, if the
   harness keeps such a list.

Stay in these files: `.github/workflows/ci.yml` (the `changes` step only),
`tests/ddphotos_parity.mjs`, the SPEC-142 spec and DEC-110. `git diff --stat` must show nothing
else, and `git diff 1b1c010 -- src/ Cargo.toml Cargo.lock` must be empty.

## Prove it can fail
Run each of these one at a time and record the result.
- **The classifier crash, driven.**
  - Save the new step's `run:` body to a scratch script, with `$files` and `GITHUB_OUTPUT` set
    by hand.
  - Make `scripts/ci-code-paths.sh` fail only in its classify path, after the self-test passes
    (for example, a bad `grep` flag there). Run the scratch script under `bash -e`. It must exit
    non-zero, and write no `code=` line.
  - Make the classifier print `maybe`. It must exit 1, naming the value.
  - Restore the script. With `files=src/lib.rs` you get `code=true`; with `files=docs/x.md` you
    get `code=false`.
  - Run the **old** step body the same way with the crashing script, and show it exits 0 with
    `code=false`. That proves the control can tell the two apart.
- **P5's new row.** Make `previewRecipe` return its input, run the harness, and the plain run
  must go red on P5 (row 12). Restore it.
- **The summary line.** Paste it from a plain run. Set K = 2 temporarily: the line must read
  `… 1 (K = 2)`. Restore it.
- Then run `just ddphotos-parity` from clean (`rm -rf target/ddphotos-parity`), and record that
  it's green.

## Guardrails
- Every commit uses `git commit -s` and ends with
  `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
- ⛔ The PR stays a **draft**. No `gh pr ready`, no merge, no version bump, no tag. No `src/`
  change. Don't edit the test bed.
- **Never poll CI.** Run `gh pr checks 234 --watch` in the background. When it exits, read one
  direct snapshot at the PR's true head SHA.
- Then read the `changes` job's log once, and quote its classify line. Read the
  `ddphotos-parity` job's log once, and quote the summary line and one row line with both
  hashes.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal, so redirect
  stdout to a file. A piped command reports the pipe's exit code, so redirect and read `$?`.
- zsh does not word-split unquoted parameters. macOS has no `timeout(1)`.
- Delete scratch scripts and target dirs when you're done.
- **Budget ~30 exchanges.**

## When you finish
- Add a `### Punch list (verify)` subsection to the spec's `## Build Completion`. It lists:
  - the six fixes;
  - each control above, with its result;
  - the CI run id and the quoted lines.
- **Measure your own cost.**
  - Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
    directory's absolute path with every `/`, `.` and `_` replaced by `-`.
  - Find it **by content, not recency**:
    `grep -l "SPEC-142 — PUNCH LIST prompt" ~/.claude/projects/<slug>/*.jsonl`.
  - ⚠ **Dedup by `.message.id`.** Take `input_tokens`, `cache_creation_input_tokens` and
    `cache_read_input_tokens` once per id, and the **MAX** `output_tokens`.
  - Price each component at the model that actually ran (`.message.model`): Sonnet $3/$15 per
    MTok, Opus $5/$25. cache_creation is ×1.25 the input rate, and cache_read ×0.10.
  - Append a `punchlist` entry to the spec's `cost.sessions`.
  - Also append the **verify** session's entry, which the verify measured but couldn't write
    (it was read-only): 7,142,148 tokens (input 98, output 31,059, cache_creation 151,338,
    cache_read 6,959,653), Opus, **$5.20**.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-142 punch list: PR #234 at <head SHA> — CI <green | red: jobs>`.

Then, short:
- `git diff --stat 1b1c010`;
- the classifier controls: new step crashing, new step on `maybe`, old step crashing, plus the
  two restored cases;
- P5's red under the preview mutation;
- the quoted CI lines: classify, summary, and a row with both hashes;
- the DEC-110 lines added;
- anything that did not go as written;
- your measured cost.
