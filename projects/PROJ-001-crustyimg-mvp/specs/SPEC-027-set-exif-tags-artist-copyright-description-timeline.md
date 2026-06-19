# SPEC-027 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-027-<cycle>.md`.

## Instructions

- [x] design (2026-06-18) — Opus, main loop. Authored the spec (`## Failing Tests`
  + `## Implementation Context`) on top of SPEC-026's metadata lane; `set` is one
  transform fn (`metadata::set_tags` + `TagSet`) + one handler (`run_set`) reusing
  `run_metadata_lane`/`Sink::write_bytes`. Ran a design-time probe confirming
  `little_exif` set-then-write PRESERVES existing tags (Orientation/GPS) and the
  no-EXIF fresh-create fallback, pixels byte-identical (real JPEG). No new dep / no
  new DEC. Fleshed out the `set` api-contract entry. Design pushed to `main` before
  build.
- [ ] build — see `prompts/SPEC-027-build.md`.
