# SPEC-149 — SHIP prompt: no flag is accepted and ignored; `-v` shows warnings

Cycle: **ship**. Bookkeeping only. Code is on `main` as `302afba` (PR #256, merged 2026-11-06). Verify (Opus) returned
a punch list at `e53e9f6` (R-1: the `--keep-gps` message and four doc lines promised `meta copy` on "JPEG and PNG";
it is JPEG only); the fix round (`10d62b2`) closed it; the orchestrator read the diff (rule 7: no re-verify).

Fresh CLI session, own worktree, Sonnet is enough: `just wt docs/spec-149-ship -- --model sonnet`.
The maintainer pastes your final message back to the orchestrator, so it must stand on its own.

## Step 0
`git fetch origin && git rebase origin/main`. Proceed only if `gh pr view 256 --json state` is MERGED. Otherwise stop
and report. Then `gh pr view 255 --json state`: record whether SPEC-151 (`pad`) has merged. It decides step 3.

## Read first
AGENTS §4, §13, §15 "During ship"; the spec in full (Frame, Design, 11 ACs, Build Completion incl. the fix round); the
timeline; HANDOFF "Traps this wave paid for"; STAGE-063's backlog.

## Facts to reflect on
- **Verify found what CI green could not.** The R6 message was copied word for word from the spec's docs-sweep
  table, and FT-1/FT-3 compared the message to a literal copy of the same sentence, so a wrong claim in the sentence
  was green everywhere. The frame's "JPEG or PNG" had never been tried on a PNG. Name the lesson: a message that tells
  the user a command to run is a claim about that command; the test must run the command it names, and the design must
  drive it on every input format it promises. The fix round added `tests/keep_gps_workaround.rs`, which parses the
  `meta copy` command out of the refusal text and runs it (JPEG restores GPS; PNG exits 4 and so goes red the day the
  lane learns PNG).
- **The guard table held.** One table in `src/cli/global_flags.rs`, scope by exclusion, no catch-all arm: 135 matrix rows
  and 20 mutations went red for their reasons; verify's unlisted mutations were all caught.
- **Cross-thread.** SPEC-151 (`pad`) is a recipe step with no CLI change, so there are still 20 commands and no
  `minimal_args` entry was needed. The build deferred the PROJ-017-owned doc hunks. A stray `…spec.md-E` file (a macOS
  `sed -i -E` backup the orchestrator committed) came into the PR branch via an update-branch and was removed on `main`
  in `def92d8`; check `git ls-files | grep -E '\.md-E$'` is empty on `main`.
- **A draft PR cannot merge.** Draft status does not hold up CI, but the maintainer must click "ready for review"
  (`gh pr ready`) before the merge is allowed. Builds open drafts by rule, so that click belongs between green CI and the
  merge. Not a finding about the code; note it in the reflection as process.
- Costs: frame about $4.92, design about $7.09, build about $2.67 (Sonnet), verify about $7.65 (Opus, measured before
  its final report); the fix round was not measured, so record it as unmeasured and say the total is a floor. Compute it.

## Do
1. Fill the spec's reflection and cost totals; set `status: shipped`; `just archive-spec SPEC-149`.
2. STAGE-063: tick SPEC-149 in the Spec Backlog (PR #256, `302afba`); leave the existing findings as is; ADD these
   as one entry, short, each with a one-line pointer:
   - a `"` in a path prints unescaped in `could not read font file "a"b.ttf": …`;
   - FT-1's `meta copy` row copies PNG to PNG so it exits 4 even without the flag (the guard-runs-first proof only;
     covered for real by `tests/keep_gps_workaround.rs`);
   - the existing message `metadata lane does not support meta copy supports JPEG only in v1 yet` is garbled (SPEC-150);
   - `--keep-gps=false` gets clap's `unexpected value 'false'` (exit 2) instead of the R6 line; still refused;
   - rendered `--help` still carries spec/DEC ids for: `--no-cache` (SPEC-064), `--check` (SPEC-066), the
     `--frozen`/`--locked` note and `--strict` (DEC-059), `--watch` (SPEC-067), and the `build` (SPEC-063, DEC-057),
     `lint` (SPEC-050, DEC-050) and `meta` (SPEC-087, SPEC-089) descriptions (SPEC-150 owns this sweep).
   Re-derive the stage's Count line with grep and cross-check (`rtk` can corrupt grep counts).
3. **The deferred doc hunks.** `docs/api-contract.md` lines 36, 37, 39, 192, 591-593 and `docs/data-model.md:122` (the
   exact text is in the spec's Docs sweep table; the data-model row says `JPEG only`).
   - If SPEC-151 (#255) has **merged**: apply them, keeping each hunk to its line, in this ship commit.
   - If it has **not**: do not touch either file. Write the exact lines into the spec's Ship section under a heading
     "Deferred doc hunks, apply after #255 merges", and say so in your report.
4. **Propose, do not write,** the PROJ-017-adjacent text, in the spec's Ship section: the CHANGELOG entry (it is in the
   spec, already `JPEG only`; re-read it, keep it in the maintainer's voice, behavior-first, no spec or DEC ids),
   `docs/recipes.md:123`, and `skills/crustyimg/SKILL.md:67-68`. Add one more proposed line: `skills/crustyimg/SKILL.md:145`
   says "JPEG and PNG" for the whole `meta` group and is stale for `meta copy` (listed at :151); `meta copy` is JPEG only.
   Say plainly that refusing `--keep-gps` (exit 0 to 2) and refusing the build-only flags off `build` are user-visible
   behaviour changes in 0.8.3.
5. Leave `.github/workflows/`, `src/`, `README.md`, `CHANGELOG.md` and the timeline file alone (the timeline is the
   orchestrator's).
6. Commit `git commit -s` (`Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>`); docs-only: push to `main`
   directly per AGENTS §13. Measure your own cost (dedup by `.message.id`, MAX output, per-component pricing; find the
   transcript by searching `SHIP prompt: no flag is accepted and ignored`).

## Report
First line: `SPEC-149 ship: archived, <commit>`. Then: what changed, whether #255 had merged and which branch of step 3
you took, the cost total, the proposed CHANGELOG entry and the three other proposed lines in full, the STAGE-063 Count
line, anything out of spec, your cost.
