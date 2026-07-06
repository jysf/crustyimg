# SPEC-047 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — classification specified with a labelled-corpus Failing-Tests set (framing, 2026-07-05).
- [x] **build** — ImageClass + OptBucket + confidence + no-ML cascade on src/analysis; 9 corpus
  tests, suite 449; fmt/clippy/lean/deny green. has_exif early; Document before Graphic (deviations
  logged). PR #54, 2026-07-06.
- [x] **verify** — CI green on #54 (all jobs); decision drift clean; post-merge suite 449. 2026-07-06.
- [x] **ship** — squash-merged #54 → main (c712afd); reflection + cost recorded; archived to done/.
  Completed STAGE-011. 2026-07-06.
