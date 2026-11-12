# SPEC-160 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-160-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-11, Opus) — GO at [M], one spec, no DEC. Premise confirmed (`edit --pad`/`--trim`
  exit 2, unknown flag). Driven: `round((N/long)·long) == N` for all 2.5e9 pairs up to 50,000 (worst error
  7.3e-12 px); `+f+f+f` loads on all four routes, and `+f+f+f+f` adds alpha 0f. Recommends one `--pad N|W:H`,
  a two-place alpha check in `run_pixel_op` gated on `introduces_alpha()`, and `parse_color` refusing any non-hex byte.
- [x] **rulings** (2026-11-11, maintainer) — all three recommendations: entries 1–4 moved (ideas.yaml ×3,
  STAGE-060 ×1), nine-gravities entry folded in, so this spec closes STAGE-058; [M] uncut; one `--pad N|W:H`.
- [ ] **design** — `prompts/SPEC-160-design.md`
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
