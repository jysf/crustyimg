# SPEC-149 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-149-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-06, Opus 5.5, `prompts/SPEC-149-frame.md`) — GO, size M, rulings settled 2026-11-06 (see design prompt):
  refuse `--keep-gps` (hidden; real feature filed on PROJ-013 STAGE-052), implement `-v`, refuse the rest, keep exit 3 for the font. All five items reproduce on `main`; a 20-verb × 13-flag driven matrix adds `responsive
  --format` and `--check`/`--strict`/`--no-cache` off `build`, plus `watermark --image`'s unnamed path.
  Recommended: refuse `--keep-gps` (hidden), minimal `-v` logger, refuse responsive's template and format,
  refuse `--strict` alone. Three out-of-spec findings filed on STAGE-063.
- [~] **design** (`prompts/SPEC-149-design.md`, Opus)
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
