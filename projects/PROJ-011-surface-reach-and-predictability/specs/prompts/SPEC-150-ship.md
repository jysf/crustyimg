# SPEC-150 — SHIP prompt: folders, help and a few messages tell the truth

Cycle: **ship**. Bookkeeping only. Code reaches `main` as PR #257 (draft until the maintainer clicks "ready for review",
then merged). Verify (Opus) returned a punch list at `5996c6b` (R1: the `-q` message named `crustyimg convert -q N`,
which exits 2 without `--format`; R2: the subfolder glob hint could not be pasted for a folder named `it's`); the fix
round (`e0e1e3a`, head `0deb220`) closed both; the orchestrator read the diff (rule 7: no re-verify) and CI was green
at `0deb220`.

Fresh CLI session, own worktree, Sonnet is enough: `just wt docs/spec-150-ship -- --model sonnet`.
The maintainer pastes your final message back to the orchestrator, so it must stand on its own.

## Step 0
`git fetch origin && git rebase origin/main`. Proceed only if `gh pr view 257 --json state,mergeCommit` says MERGED;
otherwise stop and report. Record the merge commit. Then `gh pr list --state open` and note whether any PROJ-017 PR is
open: it decides whether the text in step 4 goes to that thread or is only proposed.

## Read first
AGENTS §4, §13, §15 "During ship"; the spec in full (Frame, Design, 17 ACs, Build Completion incl. the fix round); the
timeline; HANDOFF "Traps this wave paid for"; STAGE-063's backlog.

## Facts to reflect on
- **Verify found what CI green could not, for the second time in a row.** The `-q` sentence was written in the spec's
  own AC-7 and copied word for word into code and into FT-M8's six rows, which compared the message to a literal copy of
  itself. Nobody ran `crustyimg convert -q N`; it needs `--format`. SPEC-149's lesson (a message that tells the user a
  command to run is a claim about that command) was in the build prompt and still the spec carried the false command:
  the lesson has to be applied at DESIGN, where the sentence is written, by running it. The fix round's FT-M8 now takes
  the printed command from stderr and runs it; the `it's` row runs the printed glob in `sh -c`.
- **The folder messages held.** 8 callers through one `resolve_all`; verify drove 16 verbs against the folder shapes and
  22 previously-passing invocations produced 0 differences against `main`'s binary (exit code, stdout SHA, stderr, output
  hashes). Help carries 0 id lines over 73 surfaces (main: 822).
- **A mutation the table did not list exposed a gap:** switching `optimize --out-dir` from `Fail` to `Skip` turned nothing
  red. The fix round added mixed-argument rows (`nope` plus a real image) for `optimize --out-dir` and `web`.
- **Two design/frame facts to record:** the build's list missed `tests/source.rs` (the design swept message text, not the
  variant name `NotFound`); mutation 23's second row cannot go red because that fixture holds no `//`.
- **A draft PR cannot merge** until "ready for review" is clicked; a process note, not a code finding.
- Costs: frame $4.42, design $8.57, build $14.54 (Opus), verify $6.79 (Opus, 8,979,718 tokens), fix about $0.96 (Sonnet,
  1,983,916 tokens, measured before its final bookkeeping turns, so a slight under-count). Compute the total from the
  spec's `cost.sessions`; do not trust these numbers over the spec.

## Do
1. Fill the spec's reflection and cost totals; set `status: shipped`; `just archive-spec SPEC-150`.
2. STAGE-063: tick SPEC-150 in the Spec Backlog (PR #257, merge commit from step 0); leave existing findings as is; ADD
   one entry, short, each with a one-line pointer (verify's optional list):
   - user docs still carry ids the SPEC/DEC rule did not cover: `docs/cli-reference.md:305` (STAGE-030) and
     `README.md:125` (`decisions/DEC-052`);
   - the hidden `--keep-gps` flag appears in every completion script (zsh 22 lines, fish 23, bash 22), same on `main`;
   - `MANIFEST_DROPPED_WARNING` (`src/metadata/mod.rs`) warns as unused on wasm32, on `main` too; PR #240 (wasm clippy) may
     trip on it if it uses `-D warnings`;
   - a path containing `[` (folder `a[1]`) is treated as a glob and cannot be escaped (older behaviour);
   - no test sends a `\` path through the folder message or uses a `C:\` glob (only `folder_name(r"d\")` is unit-tested),
     and a single-quoted hint does not paste into `cmd.exe`;
   - `quoted()` is ambiguous when a `\` comes right before a `"` (contrived; `"` is illegal in Windows filenames);
   - `SourceError` and `CliError` are not `#[non_exhaustive]`, so the new variants break downstream exhaustive matches
     (the CHANGELOG says so; whether to add `#[non_exhaustive]` before 1.0 is a maintainer call).
   Re-derive the stage's Count line with grep and cross-check (`rtk` can corrupt grep counts).
3. **Check that nothing was left from the PR:** `git ls-files | grep -E '\.md-E$'` empty; `./scripts/cost-audit.sh` exit 0.
4. **Propose, do not write,** the PROJ-017-adjacent text, in the spec's Ship section: (a) the CHANGELOG `Fixed`/`Changed`
   entry for SPEC-150, in the maintainer's voice, behavior-first, no spec or DEC ids, covering: a folder with no
   top-level image now says so and how to include subfolders (exit 3), `lint` names the first failing cause, the
   `-q`-with-auto message and the `meta` lane message now tell you something you can run, `--help` and shell completions
   no longer carry internal ids, and the public-enum note (new variants on `SourceError` and `CliError`; neither is
   `#[non_exhaustive]`); (b) `docs/recipes.md:148`; (c) any `skills/crustyimg/SKILL.md` line that restates the old
   folder or `-q` text (grep it). Gather the SPEC-148 and SPEC-149 CHANGELOG text from their archived specs and list all
   three together as the draft 0.8.3 block, so the release prompt can use it. Say plainly which are user-visible
   behaviour changes.
5. Leave `.github/workflows/`, `src/`, `README.md`, `CHANGELOG.md` and the timeline file alone (the timeline is the
   orchestrator's).
6. Commit `git commit -s` (`Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`); docs-only: push to `main`
   directly per AGENTS §13. Measure your own cost (dedup by `.message.id`, MAX output, per-component pricing; find the
   transcript by searching `SHIP prompt: folders, help and a few messages tell the truth`) and record a `ship` entry
   with `tokens_total`.

## Report
First line: `SPEC-150 ship: archived, <commit>`. Then: what changed, the merge commit, whether a PROJ-017 PR is open, the
cost total, the proposed CHANGELOG block (SPEC-148 + 149 + 150) and the other proposed lines in full, the STAGE-063
Count line, anything out of spec, your cost with the token breakdown.
