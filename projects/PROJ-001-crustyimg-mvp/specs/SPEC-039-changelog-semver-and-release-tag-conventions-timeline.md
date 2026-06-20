# SPEC-039 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-039-<cycle>.md`.

## Timeline

- [x] **design** (2026-06-19, Opus) — authored the spec (`## Policy (PINNED)`, acceptance
  by inspection, Implementation Context). Pure-docs chore: `CHANGELOG.md` (Keep a Changelog;
  `0.1.0` = the MVP, narrated from `docs/moat.md` + the command surface) + `RELEASING.md`
  (SemVer `0.x` policy, `vX.Y.Z` annotated-tag convention, release-cut checklist with
  publish/tag steps marked maintainer-authorized) + a README pointer. No tag/publish/code/
  dep/DEC. Build prompt at `prompts/SPEC-039-build.md`.
- [ ] **build** (Sonnet) — write the two docs + the README pointer; see prompt.
- [ ] **verify** (Opus, Explore) — confirm the sections, version match, links, and that NO
  tag/publish happened.
- [ ] **ship** — pause for the user before merge.
