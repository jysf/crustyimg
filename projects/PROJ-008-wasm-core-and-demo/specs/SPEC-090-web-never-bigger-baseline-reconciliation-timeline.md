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
- [x] build — 2026-07-17, branch `spec-090-web-never-bigger`. Option **(A)**: dimension contract wins,
  docs corrected, larger-than-original surfaced (stderr `note:` + additive/gated `larger_than_source`
  `--json` field). Pre-spec oracle reproduced 36% larger; `web==apply` + `optimize` byte-identity both
  verified against the parent-commit binary. **Framing mechanism was imprecise** (`source_bytes` is the
  original, not the downscaled intermediate — the `pipeline_altered` override is the real path). DEC-075
  emitted. All gates green. PR (pending).
- [ ] verify — independent worktree session.
- [ ] ship — orchestrator.
