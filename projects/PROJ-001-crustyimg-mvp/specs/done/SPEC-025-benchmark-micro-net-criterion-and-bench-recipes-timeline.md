# SPEC-025 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-025-<cycle>.md`.

## Instructions

- [x] design (2026-06-18) — Opus, main loop. Authored the spec + DEC-028 (criterion
  micro-benches + the equal-quality principle). Infrastructure spec: no `## Failing
  Tests` (benches aren't behavioral unit tests); verification is `cargo bench
  --no-run` + `just bench`. Dev-dependency only; no shipped-binary impact.
- [x] build (2026-06-18, PR #29) — Opus, main loop. Added `criterion` (=0.8.2
  dev-dep) + `benches/pipeline.rs` (decode/resize/encode_jpeg/score/pipeline) +
  `[[bench]]` + `just bench`/`bench-cli`. No shipped-code change. First numbers:
  score ~9.8ms dominates; codec paths sub-ms. All gates + CI green.
- [x] verify (2026-06-18) — independent read-only Explore subagent: ✅ APPROVED.
  All 5 groups call the real library paths, in-memory fixture, criterion dev-only
  (no `src/**` change), DEC-028 aligned; `cargo bench --no-run`/clippy/fmt/deny green.
- [x] ship (2026-06-18, PR #29 squash-merged) — reflections + cost totals filled
  (index-verified), STAGE-009 backlog flipped, archived to `specs/done/`,
  `just cost-audit` green. Last core STAGE-009 spec → stage shipped.
