# SPEC-037 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-037-<cycle>.md`.

## Timeline

- [x] **design** (2026-06-19, Opus) — authored the spec (`## Hardening policy (PINNED)`,
  `## Failing Tests`, `## Implementation Context`) + **DEC-038** (resize output cap). Read
  `Resize::apply` (one `(tw,th)` choke point for all six modes) + `run_edit` (raw
  save-recipe write). Scope: resize output ≤ 512 MiB at apply (== decode cap, DEC-034) +
  `edit --save-recipe` reusing the Sink's `reject_symlink_destination` (DEC-035) + a
  `SECURITY.md` threat-model verification table (authored now). The verify cycle doubles
  as an adversarial security review over the cumulative STAGE-006 diff. std-only, no new
  dep. Build prompt at `prompts/SPEC-037-build.md`.
- [ ] **build** (Sonnet) — make the failing tests pass; see `prompts/SPEC-037-build.md`.
- [ ] **verify** (Opus, Explore) — verify the two fixes AND run an adversarial security
  review over the cumulative STAGE-006 surface; gate re-run (incl. lean).
- [ ] **ship** — pause for the user before merge; on ship STAGE-006 (the MVP exit gate)
  is complete.
