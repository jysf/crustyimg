# SPEC-149 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-149-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-06, Opus 5.5, `prompts/SPEC-149-frame.md`) — GO, size M, rulings settled 2026-11-06 (see design prompt):
  refuse `--keep-gps` (hidden; real feature filed on PROJ-013 STAGE-052), implement `-v`, refuse the rest, keep exit 3 for the font. All five items reproduce on `main`; a 20-verb × 13-flag driven matrix adds `responsive
  --format` and `--check`/`--strict`/`--no-cache` off `build`, plus `watermark --image`'s unnamed path.
  Recommended: refuse `--keep-gps` (hidden), minimal `-v` logger, refuse responsive's template and format,
  refuse `--strict` alone. Three out-of-spec findings filed on STAGE-063.
- [x] **design** (2026-11-06, Opus 5.5, `prompts/SPEC-149-design.md`) — 11 ACs, 12 failing tests, 13
  mutation rows. One guard table (`src/cli/global_flags.rs`, 8 rows, scope by exclusion so a new verb is
  covered) replaces the `--watch` `if`; a `-v`/`-vv` stderr logger in `src/cli/logger.rs` (driven: usvg
  warning + debug lines, avif_parse debug on the lean leg); `watermark --font`/`--image` name the file, exit 3.
- [x] **build** (2026-11-06, Sonnet 5.5, `prompts/SPEC-149-build.md`) — draft PR #256 at e53e9f6, CI green (20 pass,
  12 skipped, 0 fail, windows + lean). 6 commits, no mutation in the diff, M1-M12 + M6b red for their reasons. Deferred to ship:
  `docs/api-contract.md` (36, 37, 39, 192, 591-593), `docs/data-model.md:122`. ~$2.67.
- [x] **verify** (2026-11-06, Opus 5.5, `prompts/SPEC-149-verify.md`) — PUNCH LIST at e53e9f6: one required (R6 + 4 doc lines said
  `meta copy` works on "JPEG and PNG"; JPEG only). Fix round `prompts/SPEC-149-fix.md` (10d62b2), merged as 302afba (PR #256). ~$7.65.
- [ ] **ship** (`prompts/SPEC-149-ship.md`, Sonnet)
