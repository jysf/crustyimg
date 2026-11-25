# SPEC-148 — SHIP prompt: `lint` fixes what it names

Cycle: **ship**. Bookkeeping only. Code is on `main` as `bb78fa1` (PR #253, merged 2026-11-06).
Verify (Opus) returned a punch list at `8f61167` (R-1); the fix round (`3c61ef1`) closed it; the
orchestrator checked the fix round (rule 7: no re-verify).

Fresh CLI session, own worktree, Sonnet is enough: `just wt docs/spec-148-ship -- --model sonnet`.
The maintainer pastes your final message back to the orchestrator, so it must stand on its own.

## Step 0
`git fetch origin && git rebase origin/main`. Proceed only if `gh pr view 253 --json state` is MERGED.
Otherwise stop and report.

## Read first
AGENTS §4, §13, §15 "During ship"; the spec in full (Frame, Design, 11 ACs, Build Completion incl. the
Fix round); the timeline; HANDOFF "Traps this wave paid for"; STAGE-063's backlog.

## Facts to reflect on
- **Verify found what CI green could not:** FT-1 ran every printed fix through a real shell and passed on
  JPEG and PNG, so it could not see that `meta clean --gps` and `meta strip` are printed for WebP, TIFF and
  AVIF, where the metadata lane exits 4 and leaves the file unchanged. The design's fixture list named the
  formats the *rules* fire on, not the formats the *fix verbs* accept. Name the lesson: an end-to-end
  "every printed fix works" test must be driven over the whole input domain of the rule, not the formats
  the author tested by hand.
- **The one formatter held:** `Finding::fix_command` is the single place a fix is built, so R-1 was a
  3-rule change behind one helper (`LintTarget::meta_fix`, sniffed container), not a sweep.
- **M-12 (Windows quoting) was driven live,** on a throwaway PR (#254, closed, branch deleted): only
  windows-latest went red, so the PowerShell leg really runs FT-1. Record that the build left it undriven
  by design and verify drove it.
- **Verify's unlisted mutations** (6 of 8 caught by an existing test; two were caught only by FT-4/goldens,
  one needed an FT-1 row): say which row was missing (`$`/backtick, filed as O-3).
- **Cross-thread:** SPEC-147 (rotate/flip, #252) merged while this PR was in verify; `update-branch`
  merged cleanly and CI stayed green. The build never touched `tests/common/mod.rs`.
- Costs: frame and design from the spec's cost block; build about $3.88 (Sonnet); verify about $6.71 (Opus,
  measured before the cleanup step, so slightly low); fix about $0.92. Compute the total.

## Do
1. Fill the spec's reflection and cost totals; set `status: shipped`; `just archive-spec SPEC-148`.
2. STAGE-063: tick SPEC-148 in the Spec Backlog; leave the O-1..O-5 entry (the orchestrator filed it) as is;
   re-derive the stage's Count line with grep (cross-check the count; `rtk` can corrupt grep counts).
3. **Propose, do not write, the CHANGELOG text.** `CHANGELOG.md` is PROJ-017-adjacent. Put a proposed
   `Changed`/`Fixed` entry for 0.8.3 in the spec's Ship section, in the maintainer's voice, behavior-first
   and without spec or DEC ids: the printed `fix:` now rewrites the file in place (`-o <file> -y`), quotes
   paths, names `optimize --max-size` for budget findings and `convert --format jpeg` for CMYK; stdin,
   16-bit PNG and formats the metadata lane cannot write (WebP, TIFF, AVIF) print no command
   (`fix: null` in JSON); the GPS rule fires on a coordinate tag, not on any GPS block (a version-only block
   no longer flags). Say plainly that the `fix: null` and the changed GPS rule are user-visible behaviour
   changes in 0.8.3.
4. Leave `.github/workflows/`, `src/` and the timeline file alone (the timeline is the orchestrator's).
5. Commit `git commit -s` (Co-Authored-By: Claude Sonnet 5.5 <noreply@anthropic.com>); docs-only: push to
   `main` directly per AGENTS §13. Measure your own cost (dedup by `.message.id`, MAX output, per-component
   pricing; find the transcript by searching `SHIP prompt: lint fixes what it names`).

## Report
First line: `SPEC-148 ship: archived, <commit>`. Then: what changed, cost total, the proposed CHANGELOG
entry in full, the STAGE-063 Count line, anything out of spec, your cost.
