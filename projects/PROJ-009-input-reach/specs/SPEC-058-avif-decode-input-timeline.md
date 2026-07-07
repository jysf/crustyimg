# SPEC-058 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

In the claude-only variant the spec's `## Implementation Context` section IS the build handoff —
there is no separate prompt file unless a cycle needs one.

## Instructions

- [x] **design** — AVIF decode as a default pure-Rust input; Failing Tests (default-build decode,
  decode-cap, corrupt→typed error, dir-source discovery, optimize→webp, feature-gated round-trip)
  + full Implementation Context. Load-bearing item: the decoder-dependency probe → **DEC-053**
  (candidates: `re_rav1d` via `image` vs `rav1d` BSD-2 + ISOBMFF glue; `zenavif` AGPL excluded).
  Framing, 2026-07-07.
- [ ] **build** — first do the DEC-053 decoder probe on a real AVIF; then wire + make Failing Tests
  pass; verify lean build + `just deny` in-cycle. Split to a container-parse spec if the probe shows
  substantial ISOBMFF glue is needed.
- [ ] **verify**
- [ ] **ship**
