# SPEC-163 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-163-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-25, Opus) — GO at [M], one spec; DEC-112 amendment recommended (rule 8). Driven: `aspect = "3:2"`
  crops a portrait 800×1200 to 800×533 and pads it to 1800×1200; the step sees the stored buffer unless
  `auto-orient` runs first. Rule: larger side along the buffer's larger side, strictly wider = landscape, square
  as written, `1:1` a no-op; 1000×1001 flips a 1000×667 crop to 667×1001. wasm needs no change.
- [x] **rulings** (2026-11-25, maintainer) — all four recommendations accepted: `orientation = "match"`; a square takes the
  aspect as written and `1:1` is a no-op; `--crop-orientation` and `--pad-orientation` in this spec ([M]); amend DEC-112 (rule 8).
- [x] **design** (2026-11-25, Opus) — `prompts/SPEC-163-design.md`. One private helper `follow_orientation` in
  `src/operation/mod.rs` called by both ops before their unchanged fit; `OneOf(["match"])` rows; two `edit` flags built
  like the gravity flags; 18 ACs, 20 FTs (14 in new `tests/aspect_orientation.rs`), 26 parity rows + self-test N3, 3
  fixtures. 17 mutants driven on a prototype, all red as predicted. Frame corrections: 6 existing key-list tests move;
  `tests/orientation.rs` exists; `docs/cli-reference.md` added.
- [ ] **build** — `prompts/SPEC-163-build.md`
- [ ] **verify**
- [ ] **ship**
