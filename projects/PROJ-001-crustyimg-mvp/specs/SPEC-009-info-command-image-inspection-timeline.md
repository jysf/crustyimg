# SPEC-009 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-009-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-06-15. Filled in the spec (Context, Goal,
  Inputs/Outputs incl. the `InfoReport`/`ExifTag` DTOs + locked JSON schema,
  Acceptance Criteria, Failing Tests, Implementation Context, Notes); emitted
  **DEC-013** (kamadak-exif always-on, read-only); clarified the `info` entry in
  `docs/api-contract.md`. Build prompt at `prompts/SPEC-009-build.md`.
- [ ] **build** — make the failing tests pass on branch
  `feat/spec-009-info-command-image-inspection`. Add `kamadak-exif = "=0.6.1"`
  (always-on) + `serde_json` dev-dep; replace the `info` stub with `run_info` +
  the DTOs + `format_label`/`color_type_label`/`read_exif_tags`/`write_json`/
  `print_human`; write the integration + unit tests. Run the four standard gates
  (NO `--features display`). See `prompts/SPEC-009-build.md`.
