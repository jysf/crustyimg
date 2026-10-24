# SPEC-130 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-130-<cycle>.md`.

## Instructions

- [x] **design** — 2026-10-23. 8 ACs, 4 failing tests (all confirmed RED on `main` before the fix).
      Filed from two STAGE-052 backlog items (the gap + its consequence for DEC-093's strict gate).
- [x] **build** — 2026-10-23, same session as design (a side session dispatched with "follow the
      spec cycle"; see Build Completion's deviations). PR opened against `main`.
- [ ] **verify** — fresh session. Re-run the negative control (remove `"webp"`, expect AC-1/3/4/5 RED).
- [ ] **ship** — on `main` after merge: cost totals, reflection, archive.
