# SPEC-131 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-131-<cycle>.md`.

## Instructions

- [x] **frame** — 2026-10-24 (`prompts/SPEC-131-frame.md`). **GO, M.** Premise corrected: a bad
      recipe already fails before any input on `apply`/`build`; the defect is silent acceptance —
      unknown keys ignored on every op (DEC-061 #1), and watermark's optional keys silently
      defaulted when wrong-typed (exit 0, output byte-identical to the default). Keep the map as
      the serialized form → `to_toml()` byte-stable. Sequential with SPEC-133 (`Watermark`).
- [ ] **design** — `prompts/SPEC-131-design.md`
- [ ] **build**
- [ ] **verify**
- [ ] **ship**
