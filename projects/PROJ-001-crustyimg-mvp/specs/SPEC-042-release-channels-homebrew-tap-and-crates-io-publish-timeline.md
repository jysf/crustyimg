# SPEC-042 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-042-<cycle>.md`.

## Instructions

- [x] **design** (2026-07-03, claude-opus-4-8) — Spec + DEC-041 (Homebrew tap via
  cargo-dist + a separate tag-only crates.io `cargo publish` workflow; design-time
  `dist` probe confirmed Homebrew is native but crates.io is NOT a dist job, then
  reverted) + Sonnet build prompt. STAGE-007 #4+#5. **Config only — arms Homebrew +
  crates.io on the next `v*` tag; #7 deferred; tap-repo + 2 secrets + tag push are
  maintainer-authorized, not in this spec.**
- [ ] **build** — `prompts/SPEC-042-build.md` (run on Sonnet, fresh session).
- [ ] **verify** — independent Explore subagent (Opus); dist plan + safety inspection
  (tag-only triggers, no hard-coded secret) + gate suite.
- [ ] **ship** — pause for maintainer before merge. Then the irreversible phase:
  create `jysf/homebrew-tap`, add secrets, cut `v0.1.0`.
