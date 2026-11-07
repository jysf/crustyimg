# SPEC-150 — VERIFY prompt: folders, help and a few messages tell the truth

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**. You did not design or build SPEC-150. Read-only toward
the PR under review.

```
just wt verify/spec-150 -- --model opus
```

Put the worktree on the PR's code:
```
git fetch origin feat/spec-150-two-messages-tell-the-truth:refs/remotes/origin/feat/spec-150-two-messages-tell-the-truth
git reset --hard origin/feat/spec-150-two-messages-tell-the-truth
```
Draft PR #257's head was `5996c6b` when this prompt was written (7 commits; every build and test leg green incl.
windows-latest; only `cost-capture audit` was red, because shipped SPEC-149 lacked a verify token total on `main`;
that is fixed on `main` at `4c94ba6`, so the PR needs a merge or `update-branch` to turn green: the maintainer does
that, not you). Record the SHA you verify. The maintainer pastes this prompt into a fresh session and pastes your
final message back to the orchestrator, so it must stand on its own.

⚠ **Another orchestrator thread owns PROJ-017.** Do not touch `src/operation/`, `src/recipe/`, `src/text/mod.rs`,
`docs/recipes.md`, `recipes/README.md`, `skills/crustyimg/SKILL.md`, `CHANGELOG.md`, `tests/common/mod.rs`,
`tests/wasm_roundtrip.rs`, `tests/recipe_*`, `scripts/wasm-ci-check.sh`. Use your own `CARGO_TARGET_DIR`; never
share one.

## What the build claims
- Spec: `projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-150-two-messages-tell-the-truth-and-help-carries-no-spec-ids.md`.
  Read it all (`## Frame`, `## Design` calls, 17 ACs, the Failing Tests FT-M*, FT-G*, FT-U*, FT-W1, the 25-row
  mutation table, `## Build Completion`). HANDOFF traps 1 to 17 and rules 1 to 7 bind you. Rule 7: optional items
  go in ONE punch list and are not re-verified. Trap 17: a bound worked out on paper is a claim until its
  maximising input is driven. Trap 13: a red must be red for its reason.
- Core: `source::resolve_all(args, reader, OnEmpty::{Fail,Skip})` with `SourceError::{NoImagesInFolder,
  NoImagesMatched}` and `CliError::InputIsAFolder` (exit 3), 8 callers; new messages for the quality/auto refusal,
  the `meta` lane, the browser refusal sentence and the demo; spec ids removed from 8 help doc comments in
  `src/cli/mod.rs`; new guards `tests/messages_tell_the_truth.rs` and `tests/help_has_no_internal_ids.rs`.
- Evidence the build gives: baseline RED on `main` (207 id lines over 44 pages; completions zsh 136, fish 92,
  powershell 136, elvish 136, bash 0; `src/wasm.rs:237`), GREEN after; 25 mutations run, all red for their reason,
  tree restored to an empty diff; every gate green; `just wasm-test` 53/53.
- Deviations to judge: (a) `tests/source.rs` was edited, because an old test asserted `NotFound` for an empty glob,
  the exact behaviour call 2 changes; (b) FT-U5 was dropped, with AC-14 carried by FT-W1 plus FT-G4; (c) mutation 23's
  expected-red row cannot go red (that fixture holds no `//`), only row 1 does; (d) the build says
  `docs/cli-reference.md` had no line restating the old `lint`/`--strict` help.
- The build's own out-of-spec finding: `just wasm-check` warns that `MANIFEST_DROPPED_WARNING`
  (`src/metadata/mod.rs:154`) is unused on wasm32; it exists on `main`. Confirm it is on `main`, and list it in the
  punch list only (PR #240 adds wasm clippy and may trip on it).

## Checks (do all; report each)
1. **Scope and commit order.** `git diff origin/main...HEAD --name-only` is the 19 files in the spec's list plus
   `tests/source.rs`; nothing under the PROJ-017 paths above; every commit signed off (`git log --format=%B | grep -c
   Signed-off-by` equals the commit count); no mutation in the diff; the first commit (RED WIP) is red for the right
   reason.
2. **Merge against current `main`.** In a scratch tree `git merge --no-commit --no-ff origin/main`. It must be clean,
   and `just check` plus `./scripts/cost-audit.sh` green on the merged tree. If a PROJ-017 PR has landed, say which.
3. **Read the code.** `resolve_all`, `refuse_folder`, `quoted()`, the 8 callers, every exit code. Look for: a caller
   that still builds its own "not found" text; a caller where `Skip` should be `Fail` or the reverse (`optimize
   --out-dir`, `build`, `report`, `lint`); a path that reaches a message unquoted; a symlink to a folder; a folder
   that is unreadable (`chmod 000`) reported as "no images"; non-UTF-8 names; and any caller outside the 8 that
   reaches the old text.
4. **Drive the folder claim yourself, not through the build's tests.** Fresh fixtures (say what you used). Shapes:
   folder with images at the top level; empty folder; folder with only text files; folder with images only in a
   subfolder; trailing-slash folder; symlink to a folder; unreadable folder (Unix); glob that matches nothing; glob
   that matches a folder; missing path; names containing a space, a `'`, a `"` and a unicode letter. For **every verb
   that takes input** (list them from `crustyimg --help`, do not trust the build's 8), run each shape and quote the
   message and exit code. A readable folder with no top-level image must say so and exit 3; a missing path must keep
   `input not found or unreadable: <path>`. Report any verb or shape that still prints the old text.
5. **Run the commands the new messages name (the SPEC-149 lesson: a message that names a command is a claim about
   it).** Take the subfolder hint, `... use a glob such as 'sub/**/*'`, copy it exactly into `sh -c` for each verb and
   each name shape above (space, `'`, unicode). Does the glob find the nested images and the command succeed? Does it
   still work from a different cwd, with an absolute path, and for a folder name that is itself `**`-hostile? Then
   the same for the other new messages: the quality/auto text ("they auto-tune quality") for every verb that can
   print it, the `meta` lane message ("does not support Gif yet", what does it tell you to do, and does that work),
   and the browser refusal sentence (run the demo path if you can, else read it against `tests/wasm_roundtrip.rs:270-283`).
   Anything the message promises that does not work is a required defect.
6. **Help carries no ids, driven with the real binary.** For every command path (walk `--help` recursively from the
   binary, not from the test's list), plus `help <verb>`, `--version`, `-h`, all five `completions` shells, and any man
   or markdown generator the CLI has, grep for `SPEC-`, `DEC-`, `PROJ-`, `STAGE-`, `AC-`, `FT-`, and any other
   internal token you find in `AGENTS.md`'s id scheme. Report counts per surface. Then look for what the guard could
   not see: `after_help`, `long_about`, `value_enum` variant docs, `hide = true` subcommands and flags (hidden items
   do not render, but `--help` for them may), error strings in `src/**` that mention an id, and `demo/`, `README.md`
   and `docs/cli-reference.md`.
7. **Trap 17: the literal scanner.** FT-G4 lexes string literals in `src/`. Drive its maximising inputs on a scratch
   copy: a raw string `r#"..."#` carrying `SPEC-1`; a byte string; a string containing `//` before the id; a
   `\`-continued multi-line string; an id split across `concat!`; an id in a `///` doc comment of a non-CLI item (must
   NOT trip); an id in a `#[cfg(test)]` module (FT-G5). Which are caught, which are gaps? A gap that real code could
   plausibly hit is a required defect; a contrived one goes in the punch list.
8. **Behaviour that must NOT have changed.** A folder with images through each verb still works exactly as on `main`;
   a glob with matches works; mixed arguments where some match and some do not behave as on `main` (and as the spec
   says for `Skip` callers); stdout bytes, output files and exit codes for every previously-passing invocation are
   identical (diff `main`'s binary against the PR's on a 15-case matrix; build both in separate target dirs).
9. **Public API and docs.** The new enum variants are public: are `SourceError` and `CliError` `#[non_exhaustive]`?
   Is any `match` in tests/examples/wasm now non-exhaustive? Check `docs/api-contract.md` against the real variants and
   exit codes sentence by sentence, `docs/cli-reference.md` (including the old `:312` line and the exit-3 row),
   `docs/architecture.md:148-151`, `AGENTS.md:501-503`, and that DEC-003's amendment says what the code does.
10. **Re-run the gates.** `just check`, `cargo clippy --all-targets -- -D warnings` default and `--no-default-features`,
    `cargo fmt --check`, lean build and tests, `just wasm-check`, `just wasm-test`.
11. **Mutations.** Re-apply in a scratch tree, each with a non-empty `git diff`, reverted to empty, each red for its
    reason with the failing line quoted: build mutations 1, 3, 7, 9, 11, 15, 18, 22 and 24 of the spec's table.
    Control for the fix is the old code restored exactly (trap 14). **Then at least four the spec did not list**, for
    example: the subfolder hint removed from one verb; `InputIsAFolder` mapped to exit 1; `quoted()` replaced by
    `{}` (then a name with a space and a `'`); `lint` taking the last failing cause instead of the first; an id
    reintroduced in a hidden subcommand's help; an id reintroduced only in a flag's `value_name`; `OnEmpty::Skip`
    swapped for `Fail` in `optimize --out-dir`. Which are caught, which are gaps?
12. **The Windows leg.** Open the windows-latest job log and quote the FT-M/FT-G lines proving they ran and were not
    skipped; list the Unix-only rows and say what, if anything, covers Windows for them (a `\` path in the folder
    message, a `C:\` glob).
13. **CI legs.** Every leg at the head: Linux, macOS, windows-latest, lean, msrv, DCO, wasm, `cost-capture audit`
    (expected red until the PR takes `4c94ba6`; say whether the merged tree is green).

## Guardrails
- ⛔ Push nothing to #257. A throwaway PR only for a live CI row, from `verify-scratch/*`, closed with `gh pr close
  --delete-branch`; `git ls-remote --heads origin 'verify-scratch/*'` must be empty at the end.
- ⛔ Nothing publishes or tags. A piped command reports the pipe's exit code: redirect and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`; BSD `sed -i` needs a suffix argument (use perl). Scratch
  files in a scratch directory, never in `~`.
- **Budget ~60 exchanges.** Required defects trigger a fix round and a narrow re-verify; the optional punch list does
  not.

## Measure your own cost
Transcript JSONL under `~/.claude/projects/<slug>/` (absolute working directory with every `/`, `.`, `_` replaced by
`-`); identify **by content**: search `SPEC-150 — VERIFY prompt`. Python or `/usr/bin/grep`. ⚠ Dedup by
`.message.id`: input, cache_creation, cache_read once per id, **MAX** `output_tokens`. Price per component at the
model that ran (Opus $5/$25, Sonnet $3/$15; cache_creation ×1.25, cache_read ×0.10). **Report the token breakdown
and the total tokens, not only the dollars**: the cost audit needs `tokens_total`.

## Report back (your final message)
First line, one of:
- `✅ APPROVED — merge PR #257 at commit <SHA>.`
- `⚠ PUNCH LIST — PR #257 at <SHA> needs: <required fixes>.` (required items separate from optional ones)
- `❌ REJECTED — <reason>.`
Then checks 1 to 13 with evidence (quoted lines), the per-verb folder matrix, the result of running each named
command, the literal-scanner gap table, the merged-tree result, cleanup confirmation (no `verify-scratch/*`
branches), anything only the maintainer can do, and your measured cost with the token breakdown.
