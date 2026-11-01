# SPEC-144 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-144-<cycle>.md`.

## Instructions

- [x] **frame** (`prompts/SPEC-144-frame.md`), 2026-10-31, Opus, GO, low [M]. Premise confirmed from
  run 41251779509 (macos-14 brownout ×2) and the 0.8.2 timestamps (crates.io/npm out 2 h before the
  Release). Fix: 3-line `[dist.github-custom-runners]` (release.yml unchanged); npm + crates.io as
  cargo-dist custom publish jobs after `host`, npm by OIDC (trusted publisher names `release.yml`).
- [ ] **design** (`prompts/SPEC-144-design.md`): ACs, failing checks, DEC-111.
- [ ] **build**: draft PR; one-time `pr-run-mode = "upload"` proof of all four builds.
- [ ] **verify**
- [ ] **ship**
