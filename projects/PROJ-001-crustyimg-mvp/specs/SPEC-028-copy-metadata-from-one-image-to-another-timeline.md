# SPEC-028 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-028-<cycle>.md`.

## Instructions

- [x] design (2026-06-18) — Opus, main loop. Authored the spec (`## Failing Tests`
  + `## Implementation Context`); the last metadata-lane command. A design-time probe
  verified JPEG EXIF+ICC transfer via `img-parts`' `ImageEXIF`/`ImageICC` traits with
  byte-identical DST pixels, AND surfaced that PNG copy is non-viable (little_exif
  zTXt vs img-parts eXIf chunk) → emitted **DEC-030** (copy-metadata JPEG-only v1).
  `copy_metadata(from,to)` + `run_copy_metadata` (two inputs, single fixed output, NOT
  a fan-out; default writes back to DST in place behind `-y`). No new dep. Fleshed out
  the api-contract entry. Design + DEC-030 pushed to `main` before build.
- [ ] build — see `prompts/SPEC-028-build.md`.
