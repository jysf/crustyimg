# SPEC-015 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-015-<cycle>.md`.

## Instructions

- [x] **design** — spec authored (Context, Goal, Failing Tests, Implementation
  Context), `auto-orient` api-contract entry pinned, build prompt written,
  **DEC-017** emitted (ops may READ the captured MetadataBundle to drive a pixel
  transform; auto-orient uses image's native Orientation, no kamadak-exif).
  Authored by the orchestrator (Opus), 2026-06-15.
- [x] **build** — PR #16 opened. `AutoOrient` op + `orientation_from_exif_segment`
  helper + registry entry + `run_auto_orient` CLI + `jpeg_with_orientation` fixture;
  all named Failing Tests pass; 206/206 tests green; 4 gates clean.
  Implemented by claude-sonnet-4-6, 2026-06-15.
- [ ] **verify** — Opus read-only review of the PR against the spec; ✅/⚠/❌.
- [ ] **ship** — squash-merge PR, bookkeeping on `main`, archive, brag. This is
  the last STAGE-003 spec → then run the STAGE-003 STAGE SHIP.
</content>
