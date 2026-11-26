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
- [x] **build** (2026-11-26, Sonnet) — `prompts/SPEC-163-build.md`. Draft PR #272 at `fccb368`, 5 signed commits,
  22 files (exactly `## Outputs`); CI green on every leg incl. lean, Windows, wasm and ddphotos parity. 18/18 ACs; RED
  baseline matched; M0–M17 driven, every red set as tabled; parity local 16 TABLE rows + 26 orientation rows green, N3
  exactly the eight. Parity cells for mutants other than M5 still reasoned (no wasm-side mutant run). Cost $5.50,
  measured before its last commit; ship re-measures. Space freed not recorded.
- [x] **verify** (2026-11-26, Opus) — `prompts/SPEC-163-verify.md`. APPROVED at `fccb368`, no punch items. Drove an
  independent model over 9324 rows (0 mismatches; 4662 key-off rows equal `main`), every message by `==`, AC-5 on the
  Orientation-6 fixture, 12 save/replay rows (AE 0), fixtures byte-identical with ImageMagick 7.1.2-34, M1 into wasm
  (both sides, and wasm only: exactly the two S-on-9:16 rows), M1/M5/M9/M14/M15/M16 as tabled. DEC-112 rule 8: apply as
  written. Five optional cleanups: ship takes 1 (data-model wording) and 5 (tick ACs); 2–4 filed as
  `aspect-orientation-loose-ends` in `guidance/ideas.yaml`. Cost $6.83. Freed ~18 GB.
- [x] **merge** (2026-11-26, maintainer) — PR #272 as 3de9f64, head 4c8591c (approved head fccb368; the later commits touch only docs, `guidance/ideas.yaml` and this timeline).
- [x] **ship** (2026-11-26, Sonnet) — `prompts/SPEC-163-ship.md`. Merged 3de9f64, PR #272. DEC-112 rule 8 applied; one `data-model.md` sentence; ACs ticked; M1/M5 parity cells corrected; reflected; archived. Total cost $31.37 across 5 sessions (design, build re-measured; verify measured).
