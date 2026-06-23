# SPEC-041 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-041-<cycle>.md`.

## Instructions

- [x] **design** (2026-06-23, claude-opus-4-8) — Spec + DEC-040 (cargo-dist 0.32.0
  release pipeline; design-time probe installed `dist`, ran init/generate/plan,
  verified config + the PR=plan / tag=publish safety model, then reverted) + Sonnet
  build prompt. STAGE-007 #3: tag-triggered cross-platform binaries + checksums +
  installers → GitHub Releases, plus a CI-enforced MSRV. **Design + dry-run only —
  arms the pipeline, cuts no release; Homebrew installer / crates.io publish deferred
  to #4/#5.**
- [ ] **build** — `prompts/SPEC-041-build.md` (run on Sonnet, fresh session).
- [ ] **verify** — independent Explore subagent (Opus); re-run dist plan + safety
  inspection + gate suite.
- [ ] **ship** — pause for maintainer before merge. The actual `v0.1.0` tag/release
  remains a separate maintainer-authorized action.
