# SPEC-090 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-090-<cycle>.md`.

## Instructions

- [x] design — framed build-ready 2026-07-16. Origin: SPEC-088's verify measured `web` shipping a file
  14% LARGER than a 3000px source; the pre-spec oracle reproduces it → pre-existing SPEC-085 behavior,
  honestly reported, but the documented promise is measured against a different baseline than the code
  enforces (`pick_winner` compares against the DOWNSCALED intermediate, not the original file). Spec
  decides claim-vs-behavior with evidence; recommendation (A) correct-the-claim + surface it, to be
  proven or refuted at build. DEC-075 at build.
- [ ] build — worktree session.
- [ ] verify — independent worktree session.
- [ ] ship — orchestrator.
