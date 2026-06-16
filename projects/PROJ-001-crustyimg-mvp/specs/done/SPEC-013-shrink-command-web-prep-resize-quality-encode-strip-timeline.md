# SPEC-013 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-013-<cycle>.md`.

## Instructions

- [x] **design** — completed 2026-06-15. Authored by the ORCHESTRATOR (Opus) directly after two consecutive design-subagent sessions dropped on API socket errors. Emitted **DEC-016** (encode quality policy: `-q` → JPEG quality, ignored for lossless formats, `shrink` default 80). Spec: `shrink` = resize to default max 1600 + quality encode (default 80) + inherent metadata drop, reusing the shipped `Resize` op + the shared `run_pixel_op`; the new work is a quality-aware encode path in `src/sink` (`encode_to_bytes`/`Sink::write` gain a `quality` param) threaded through `run_pixel_op`. Complexity M. Build prompt: `prompts/SPEC-013-build.md`. api-contract shrink entry pinned (defaults + metadata scope).
- [x] **build** — PR #14 opened 2026-06-15. All 181 tests pass; all gates green (cargo build/test/clippy --all-targets/fmt --check). encode_to_bytes made pub; stub_command_returns_not_implemented repointed to convert. (Clean first-pass build; incremental-commit + test-existence-checklist lessons applied.)
- [x] **verify** — ✅ APPROVED (read-only, Opus) at commit `c0c94be`. Re-ran 4 gates cold (181 tests) + CI 6/6 green. Proved the quality knob hands-on (JPEG -q 20 → 1507 bytes vs -q 90 → 3611 bytes; PNG ignores quality; default bounds 1600 + smaller file); resize/thumbnail unchanged (21 tests); quality threaded through every sink.write caller; DEC-016/015 conformance; no new dep/DEC/CliError variant. Ruled the encode_to_bytes-pub deviation a non-issue (module's 4 sibling helpers are already pub). No punch list. 2026-06-15.
- [x] **ship** — Merged PR #14 (squash) → main on 2026-06-15 (merge `48bc5fc`; clean merge). Cost: 4 sessions, $null (design/verify/ship Opus, build Sonnet 4.6 — subagent numerics null). Archived to done/. **STAGE-003: 4 of 6 shipped — resize op + resize CLI + thumbnail + shrink. Wired the -q quality knob (DEC-016) that convert reuses.**
