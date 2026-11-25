# SPEC-148 — FIX prompt: verify punch list for PR #253

Cycle: **build (fix round)**. Fresh CLI session, **Sonnet**, on the existing PR branch. Verify (Opus)
returned a PUNCH LIST at `8f61167`: every check, 15 mutations, M-12 live on Windows and the merged tree
passed. One required defect, R-1.

```
just wt fix/spec-148 -- --model sonnet
```

Then, in the worktree: `git fetch origin && git checkout -B fix/spec-148 origin/<PR #253's head branch>`
(`gh pr view 253 --json headRefName`). You **push to that branch**: `git push origin HEAD:<branch>`.
The PR stays a **draft**; no `gh pr ready`, no merge, no tags. Use your own `CARGO_TARGET_DIR`.

⚠ Another orchestrator thread owns PROJ-017 (SPEC-147: `src/operation/`, `src/recipe/`, `src/cli/mod.rs`,
`tests/common/mod.rs`, `docs/recipes.md`, `README.md`, `CHANGELOG.md`, `scripts/wasm-ci-check.sh`). Stay out.
Files you may touch: `src/lint/{mod,rules,report}.rs`, `tests/lint.rs`, `tests/adoption_glue.rs`,
`tests/fixtures/lint/`, `docs/github-actions.md`, and the SPEC-148 spec (Build Completion addendum only).

## R-1 — never print a fix the metadata lane cannot run (required)
`lint` flags GPS on WebP, TIFF and AVIF (exit 7) and prints `crustyimg meta clean --gps w.webp -o w.webp -y`.
Run verbatim it gives `error: metadata lane does not support WebP yet`, rc 4, file unchanged. The same
happens for `privacy/camera-metadata`'s `meta strip` on WebP and TIFF. The spec's goal is "every `fix:` line
run verbatim rewrites the file" (AC-2), and DEC-050 says a fix a user can run verbatim. The metadata lane
handles JPEG and PNG only (`src/metadata/mod.rs`, `Lane`).

Fix: when a finding's fix fragment is a `meta` verb and the file's container is not JPEG or PNG, the fragment
is `None`, so the output is `fix: null` in JSON, no suffix in SARIF, and no command in text. The message must
say what to do instead, in the same register as the 16-bit PNG finding (look at how that one words it): name
the real route, for example `convert --format jpeg` is NOT acceptable unless you drive it and it clears the
finding without losing the user's pixels; if there is no honest one-line route, say the metadata lane cannot
rewrite this format yet. Decide from what you can drive, and report which wording you chose and why.

- Base the check on the sniffed container (the same `guess_format` the lane uses), not on the file extension.
- Audit every rule that emits a `meta` fragment (`grep -n '"meta ' src/lint/rules.rs`), not only the two named.
- Do not change the GPS predicate, the byte-budget fix, or the CMYK fix.

### Order (trap 13: RED first, for its reason)
1. Add an FT-1 row for **WebP with GPS** and one for **WebP with camera EXIF** (and TIFF if you can build one
   independently of the code under test; ImageMagick and exiftool were used by verify). Run `tests/lint.rs`
   FT-1 on the unmodified PR head and quote the new rows RED with their failure line (`shell_failed=1`, or the
   printed command), not a harness error.
2. Make the change. The rows now assert the finding still fires and the printed fix is absent (`fix: null`),
   and FT-1's "every printed fix works" loop covers the rest. Quote GREEN.
3. Drive it by hand with the built binary, outside the test harness: WebP, TIFF and AVIF GPS fixtures
   through `lint` in text, JSON and SARIF. Quote the three outputs for WebP, and confirm JPEG and PNG still
   print their commands unchanged.
4. Mutation (trap 14): restore the old fragment for non-JPEG/PNG formats exactly, show the new rows red, then
   revert and show `git diff` empty.
5. Update the docs table in `docs/github-actions.md` §5 if it shows a WebP/TIFF row, and add one sentence
   saying a finding on a format the metadata lane cannot write prints no command.

## Do not do (rule 7: the optional items are not this round's work)
O-1 to O-5 from verify are filed on STAGE-063 by the orchestrator. Do not touch `optimize`'s unreachable
budget, `max_bytes = 0` validation, the `$`/backtick FT-1 row, `common::jpeg_with_gps`, or the fixture README.
If you believe one has become required because of your change, stop and say so.

## Before pushing
1. `git fetch origin`; `git merge origin/main` into your tree to prove it still merges cleanly.
2. `just check` (all gates, default and lean), `cargo fmt --check`, clippy `-D warnings`. Redirect and read `$?`;
   no pipes.
3. `git commit -s`, ending with `Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`.
4. Push, background `gh pr checks 253 --watch` (never poll), then read one snapshot at the head SHA, every leg
   including windows-latest (the new FT-1 rows run there through PowerShell).
5. Append a short "Fix round" paragraph to the spec's Build Completion and a `build-fix` entry in
   `cost.sessions` (dedup JSONL by `.message.id`, MAX output, per-component pricing; find the transcript by
   searching `FIX prompt: verify punch list for PR #253`).

## Report (final message, pasted to the orchestrator)
First line: `SPEC-148 fix: PR #253 at <head SHA> — CI <green | red: legs>`.
Then: the audit list of `meta` fragments and which formats each now prints for; the new rows RED with lines,
then GREEN; the by-hand WebP/TIFF/AVIF outputs; the wording chosen; the mutation; files changed
(`git diff --name-only 8f61167..HEAD`); anything out of spec; measured cost.
