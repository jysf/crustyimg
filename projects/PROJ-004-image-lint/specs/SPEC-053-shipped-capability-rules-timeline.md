# SPEC-053 timeline

Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.
In the claude-only variant the spec's `## Implementation Context` section IS the build handoff.

## Instructions

- [x] **design** — the shipped-capability rules (camera-metadata, orientation, oversized-bytes,
  oversized-dimensions, colorspace + ICC, animated-gif) specified with positive/negative Failing
  Tests. (PROJ-004 framing, 2026-07-06.)
- [ ] **build** — depends on SPEC-050 + SPEC-051 (framework + config). Reads info/EXIF only; extend
  tests/common with CMYK/multi-frame-GIF/Make-Model fixtures.
- [ ] **verify**
- [ ] **ship**
