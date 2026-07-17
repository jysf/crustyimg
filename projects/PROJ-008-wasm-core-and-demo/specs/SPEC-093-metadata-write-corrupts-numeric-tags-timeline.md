# SPEC-093 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-093-<cycle>.md`.

## Instructions

- [x] design — framed build-ready 2026-07-17. Origin: SPEC-089's verify flagged (out of scope) that `set`
  mangles EXIF Orientation (6 → 1536) and loses GPS precision; the orchestrator **independently
  reproduced it and found it is WIDER** — `meta clean --gps` (the PRIVACY verb, documented to preserve
  orientation) corrupts it too, so the bug is in the shared container lane (`src/metadata/tiff.rs`), not
  in `set`. Pre-existing (SPEC-026/027 era); the pre-move binary is identical. **Survived because ASCII
  tags are immune and every test checks only ASCII tags — plus byte-identity proofs compared against an
  equally-broken oracle ("identical to the old bytes ≠ correct bytes", SPEC-089's verify).** A clean
  orchestrator hypothesis (normalized-LE serialize vs big-endian input) was **REFUTED at framing**: both
  `MM` and `II` inputs corrupt identically → the bug is unconditional and undiagnosed. DEC at build.
- [ ] build — worktree session.
- [ ] verify — independent worktree session.
- [ ] ship — orchestrator.
