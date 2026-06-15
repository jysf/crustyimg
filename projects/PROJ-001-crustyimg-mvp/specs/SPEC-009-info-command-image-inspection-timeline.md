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
- [x] **build** — PR #9 opened 2026-06-14. Made all failing tests pass; 111 tests
  green; all four gates clean (cargo build, cargo test, clippy -D warnings, fmt
  --check). Added `kamadak-exif = "=0.6.1"` (DEC-013) + `serde_json` dev-dep;
  replaced info stub with `run_info` + `InfoReport`/`ExifTag` DTOs +
  `format_label`/`color_type_label`/`read_exif_tags`/`write_json`/`print_human`;
  wrote integration tests in `tests/cli.rs` + new `tests/info_exif.rs`; wrote
  unit tests in `src/cli/mod.rs`.
