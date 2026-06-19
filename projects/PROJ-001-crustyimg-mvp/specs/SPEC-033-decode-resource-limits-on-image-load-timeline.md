# SPEC-033 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-033-<cycle>.md`.

## Timeline

- [x] **design** (2026-06-19, Opus) — authored the spec (`## Limits policy (PINNED)`,
  `## Failing Tests`, `## Implementation Context`) + **DEC-034** (decode-limits policy).
  Probed `image::Limits` / `ImageReader::limits` / PNG codec enforcement against the
  vendored crate source: confirmed `check_dimensions` at decoder construction +
  `limits.reserve(total_bytes)` at decode enforce dimension + alloc caps and surface
  `ImageError::Limits`. Caps: per-dimension ≤ 65 535, alloc ≤ 512 MiB; reject→typed
  `LimitsExceeded`→exit 1; one choke point (`decode_with_format`). No new dep. Updated
  `SECURITY.md` + `docs/api-contract.md`. Build prompt at `prompts/SPEC-033-build.md`.
- [ ] **build** (Sonnet) — make the failing tests pass; see `prompts/SPEC-033-build.md`.
- [ ] **verify** (Opus, Explore) — independent read-only review + gate re-run (incl. lean).
- [ ] **ship** — pause for the user before merge.
