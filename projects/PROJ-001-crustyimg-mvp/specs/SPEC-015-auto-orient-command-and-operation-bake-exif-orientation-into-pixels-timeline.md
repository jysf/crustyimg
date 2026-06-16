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
- [ ] **build** — add the `AutoOrient` op + registry entry + `run_auto_orient`
  CLI + the `jpeg_with_orientation` fixture; make the `## Failing Tests` pass;
  4 gates; open PR. Prompt: `prompts/SPEC-015-build.md` (Sonnet 4.6).
- [ ] **verify** — Opus read-only review of the PR against the spec; ✅/⚠/❌.
- [ ] **ship** — squash-merge PR, bookkeeping on `main`, archive, brag. This is
  the last STAGE-003 spec → then run the STAGE-003 STAGE SHIP.
</content>
