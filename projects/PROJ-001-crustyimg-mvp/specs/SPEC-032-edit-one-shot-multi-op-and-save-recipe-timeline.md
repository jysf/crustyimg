# SPEC-032 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-032-<cycle>.md`.

## Timeline

- [x] **design** (2026-06-19, Opus) — authored the spec (`## Command surface (PINNED)`,
  `## Round-trip guarantee (PINNED)`, `## Failing Tests`, `## Implementation Context`).
  Flag surface: `--auto-orient` / `--resize-max N` / `--invert`, canonical order
  auto-orient→resize→invert; `--save-recipe` serializes via `Recipe::from_ops`/`to_toml`;
  reuses `run_pixel_op` + the registry. No new dep, no new DEC. Updated
  `docs/api-contract.md` + the stage backlog. Build prompt at `prompts/SPEC-032-build.md`.
- [ ] **build** (Sonnet) — make the failing tests pass; see `prompts/SPEC-032-build.md`.
- [ ] **verify** (Opus, Explore) — independent read-only review + gate re-run (incl. lean).
- [ ] **ship** — pause for the user before merge; on ship STAGE-005 is complete.
