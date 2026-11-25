# SPEC-162 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-162-<cycle>.md`.

## Instructions

- [x] **frame + design** (2026-11-24, orchestrator, Opus) — folded into the spec: the measurement of run
  45501714010 is the frame; `## Outputs`, the ACs and the mutation table are the design. No separate sessions.
- [x] **build** (2026-11-25, Sonnet, $2.17 measured before its last edits) — draft PR #271 at `e54e5b5`; code last
  changed at `ecf0dc7` (run 45539781614 green). 102.6 → 62.3 runner-minutes; cache kept (Build + Test −43/−23/−47%).
  M3 did not match AC-5: a shared `CARGO_TARGET_DIR` serves stale outer artifacts across worktrees (the AC's mistake).
- [x] **verify** (2026-11-25, Opus, $3.37 measured before its final message) — PUNCH LIST at `e54e5b5`. Two claims:
  macOS's Build + Test saving holds on one of two warm runs (−22.5%, then −15.5% in run 45541262385); the cache
  step has no `if:`, so a docs-only run restores ~1.75 GB for nothing. One wording item: the probe hash protects
  nothing (a shared target folder serves the other checkout's outer test binary — a 1 s false green), the `ci.yml`
  comment is loose, and AC-5, M3/M4 and Build Completion misstate. Numbers re-checked against the API by the
  orchestrator. Three out-of-spec findings filed in `guidance/ideas.yaml` (`ci-classifier-pr-diffs-the-whole-branch`,
  `native-lean-plus-avif-build`, `rust-cache-on-feature-legs`); the optional skip-name guard was declined.
  Rulings: the cache stays, with AC-7 restated to the Build step (−70–90% on every warm leg); the hash goes.
- [~] **punch list** — prompts/SPEC-162-punchlist.md (Sonnet). Don't merge #271 before the re-verify reports.
- [ ] **re-verify** — after the punch list reports.
- [ ] **ship** — after the maintainer merges the PR.
