# SPEC-148 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-148-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-05, Opus 5.5, `prompts/SPEC-148-frame.md`) — GO, size S. Every printed
  `fix:` wrote the image to stdout and left the file unchanged; both `convert --format` fixes exit 3;
  paths are unquoted; `optimize` ignores the budget; the 16-bit PNG fix cannot clear. GPS rule keys on
  any GPS-IFD tag; a version-only block flags (exiftool fixture). Predicate: a coordinate tag.
- [ ] **design** — `prompts/SPEC-148-design.md`
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
