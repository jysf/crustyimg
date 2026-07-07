# SPEC-053 timeline

Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.
In the claude-only variant the spec's `## Implementation Context` section IS the build handoff.

## Instructions

- [x] **design** — the shipped-capability rules (camera-metadata, orientation, oversized-bytes,
  oversized-dimensions, colorspace + ICC, animated-gif) specified with positive/negative Failing
  Tests. (PROJ-004 framing, 2026-07-06.)
- [x] **build** — `src/lint/rules.rs` (7 rules + SOF/GIF sniffs) + single-parse `ExifFacts` +
  `Rule::default_enabled` opt-in + png_16bit/animated_gif fixtures. 11 unit + 4 integration tests;
  no new dep. PR #62. (2026-07-06.)
- [x] **verify** — all CI green on #62 (3-OS matrix incl. Windows, avif/webp-lossy, lean, msrv 1.89,
  cargo-deny). (2026-07-06.)
- [x] **ship** — squash-merged #62 → `main` (ec374d6); reflection + cost recorded; archived to
  `done/`. STAGE-013 complete (4/4). (2026-07-06.)
