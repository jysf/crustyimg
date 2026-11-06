# SPEC-148 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-148-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-05, Opus 5.5, `prompts/SPEC-148-frame.md`) — GO, size S. Every printed
  `fix:` wrote the image to stdout and left the file unchanged; both `convert --format` fixes exit 3;
  paths are unquoted; `optimize` ignores the budget; the 16-bit PNG fix cannot clear. GPS rule keys on
  any GPS-IFD tag; a version-only block flags (exiftool fixture). Predicate: a coordinate tag.
- [x] **design** (2026-11-05, Opus 5.5, `prompts/SPEC-148-design.md`) — 11 ACs, 12 failing tests, 16
  mutation rows. One formatter: `crustyimg <frag> <file> -o <file> -y`, path quoted POSIX (PowerShell on
  Windows), `./` before a leading `-`; stdin and 16-bit print no command (`fix: null`). Budget fix is
  `optimize --max-size <bytes>`; CMYK `convert --format jpeg`. GPS keys on 4 coordinate tags + area info.
- [x] **build** (2026-11-05, `prompts/SPEC-148-build.md`, Sonnet): draft PR #253 at `8f61167`, CI green incl. windows-latest. 13 mutations red locally; M-12 not driven. Cost about $3.88.
- [~] **verify** (2026-11-05, Opus 5.5, `prompts/SPEC-148-verify.md`) — PUNCH LIST at `8f61167`, about $6.71. 12 checks, 15 mutations, M-12 live on Windows (scratch PR #254, closed) and the merged tree all passed. One required defect R-1: `meta clean --gps` and `meta strip` fixes are printed for WebP, TIFF and AVIF, where the metadata lane exits 4 and leaves the file unchanged. Ruled: fix round (`prompts/SPEC-148-fix.md`, Sonnet); O-1 to O-5 filed on STAGE-063.
- [x] **fix round** (2026-11-06, `prompts/SPEC-148-fix.md`, Sonnet) — R-1 closed at `3c61ef1`, about $0.92. One helper `LintTarget::meta_fix` (sniffed container, JPEG/PNG only) for gps, camera and icc; FT-1 rows gps-webp, camera-webp, gps-tiff red (3 unrunnable) then green; mutation red. Checked by the orchestrator, not re-verified (rule 7): 15 legs green incl. windows-latest, DCO green. SPEC-147 merged meanwhile (#252), so the PR is BEHIND main and needs update-branch before merge.
- [ ] **ship**
