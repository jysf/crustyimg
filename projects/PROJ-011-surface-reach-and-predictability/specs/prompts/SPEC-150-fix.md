# SPEC-150 — FIX prompt: two messages name commands that do not work as printed

Cycle: **build (fix round)**. Fresh CLI session, **Sonnet**, on the existing PR branch. Verify (Opus) returned a PUNCH
LIST at `5996c6b`: gates, CI, the merged tree, the 22-invocation main-vs-PR comparison, the help scan and 16 of 17
mutations all passed. Two required defects, both in message text; the code paths are right.

```
just wt fix/spec-150 -- --model sonnet
```

Then, in the worktree: `git fetch origin && git checkout -B fix/spec-150 origin/feat/spec-150-two-messages-tell-the-truth`,
then `git merge origin/main` (takes `4c94ba6`, which turns the `cost-capture audit` leg green; say so if the merge is not
clean). You **push to that branch**: `git push origin HEAD:feat/spec-150-two-messages-tell-the-truth`. PR #257 stays a
**draft**; no `gh pr ready`, no merge, no tags. Use your own `CARGO_TARGET_DIR`. Every commit `git commit -s`, ending
`Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.

⚠ PROJ-017 owns `src/operation/`, `src/recipe/`, `src/text/mod.rs`, `docs/recipes.md`, `recipes/README.md`,
`skills/crustyimg/SKILL.md`, `CHANGELOG.md`, `tests/common/mod.rs`, `tests/wasm_roundtrip.rs`. Stay out of them.

## R1 — the `-q` message names a command that fails as written (required)
`optimize -q 10 a.png -o o.png` (and `web -q 10 …`) print `…; to encode at a fixed quality, use \`crustyimg convert -q N\``.
Verify ran it: `crustyimg convert -q 10 a.png -o fixed.jpg` exits 2 with
`error: the following required arguments were not provided: --format <FORMAT>`. The wording came from the spec's own
AC-7, so the build followed the spec; the spec was wrong.

Ruling (orchestrator): the message names **`crustyimg convert --format <FORMAT> -q N`**. Do not change `convert`'s
flags and do not touch the guard logic.
1. Find the message in `reject_quality_with_auto` (`src/cli/`) and change the command it names. The placeholder
   `<FORMAT>` stays literal; it is what the user fills in.
2. Update every test row that pins the old sentence (FT-M8 in `tests/messages_tell_the_truth.rs`, six conflict rows).
3. Update the spec: AC-7 and the FT-M8 description in
   `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-150-two-messages-tell-the-truth-and-help-carries-no-spec-ids.md`
   (and any docs line that copies the sentence: `grep -rn "fixed quality, use" src tests docs README.md`).
4. **Make the test run the command it names** (the SPEC-149 lesson): add a test that takes the printed command with
   `<FORMAT>` replaced by `jpeg` and `N` by `10`, runs it on a small fixture, and asserts exit 0 and a non-empty
   output. A test that compares the message with a literal copy of itself cannot catch a false claim.

## R2 — the subfolder hint cannot be pasted when the folder name contains `'` (required)
For a folder named `it's` the message reads `no images in folder 'it's': … use a glob such as 'it's/**/*'`. Pasted into
`sh -c` that gives `unexpected EOF while looking for matching '` and exit 2, on all 16 input verbs. Every other name
shape (space, `"`, unicode, trailing slash, symlink, absolute path) works.

Fix: shell-quote the **glob suggestion** in the hint. `lint::shell_arg` (`src/lint/mod.rs:113`) already does this for
POSIX and PowerShell; reuse it rather than writing a second quoter (move it to a shared place only if the visibility
forces it, and say so). The folder name in the first half of the message (`no images in folder '…'`) stays the plain
quoted display form: it is a label, not a command.
1. The hint for an ordinary name must stay byte-identical (`'sub/**/*'`): the existing FT-M rows must not change.
2. Add an `it's` row to FT-M3 that **runs the printed glob in `sh -c`** and asserts the nested image is found
   (Unix only, `#[cfg(unix)]`; it must fail where it is required, trap 11, so it runs on the ubuntu and macOS legs).
3. Prove it red: with the old quoting the new row fails with `unexpected EOF`; quote that line in your report.

## Optional, cheap (do it, no re-verify)
Verify found that switching `optimize --out-dir` from `OnEmpty::Fail` to `Skip` turns nothing red (the mutant writes
`a.webp` and exits 0 for `optimize --out-dir od nope a.png`; the PR exits 3). Add mixed-argument rows to FT-M for
`optimize --out-dir` and `web` (`nope` + a real image must exit 3 and write nothing), and show the `Skip` mutant red
on them. Revert the mutant to an empty diff.

Everything else on verify's punch list is not yours: do not touch `docs/cli-reference.md:305`, `README.md:125`,
`--keep-gps` in completions, `MANIFEST_DROPPED_WARNING`, `#[non_exhaustive]`, `[` in paths, Windows `\` rows.

## Gates
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` default and `--no-default-features`, `just check`,
lean build and tests, `just wasm-check`, `just wasm-test`. Update `## Build Completion` in the spec with a short
"Fix round" paragraph (what changed, red lines quoted, commit SHA). Do not edit the spec's timeline file.
Push, then background `gh pr checks 257 --watch` (never poll) and read one direct snapshot at the true head SHA.

## Measure your own cost
Transcript JSONL under `~/.claude/projects/<slug>/`; identify **by content**: search `SPEC-150 — FIX prompt`.
Python or `/usr/bin/grep`. Dedup by `.message.id` (input, cache_creation, cache_read once per id, MAX output_tokens);
Sonnet $3/$15 per MTok, cache_creation ×1.25, cache_read ×0.10. **Report the token breakdown and tokens_total.**
Add a `fix` entry to `cost.sessions` in the spec.

## Report back (your final message)
First line: `SPEC-150 fix: PR #257 at <head SHA> — CI <green | red: legs>`. Then: R1 (the new sentence, the test that
runs it and its output), R2 (the new hint for `it's`, the red line with the old quoting, the green line), the optional
rows and the `Skip` mutant red, gates, the merge result, files changed, measured cost with breakdown.
