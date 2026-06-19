# SPEC-026 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-026-<cycle>.md`.

## Instructions

- [x] design (2026-06-18) — Opus, main loop. Activated STAGE-004; authored the spec
  (`## Failing Tests` + `## Implementation Context`), emitted **DEC-029** (pins
  `img-parts` `=0.4.0` + `little_exif` `=0.6.23`, pure-Rust + permissive), added both
  deps to `Cargo.toml`, ran a **design-time probe** (throwaway) confirming strip +
  clean --gps on real JPEG + PNG with byte-identical decoded pixels, and fleshed out
  the `strip`/`clean` entries in `docs/api-contract.md`. v1 = JPEG + PNG; container
  lane only (no pixel re-encode). `just deny` green. Design pushed to `main` before
  build.
- [x] build (2026-06-18, PR #30) — foreground metered subagent (Opus, 172k tok).
  Added `src/metadata/mod.rs` (`strip_all`/`clean_gps`/`MetadataError`),
  `Sink::write_bytes` (raw-bytes container write), `run_metadata_lane` fan-out +
  `run_strip`/`run_clean`, `CliError::Metadata` (UnsupportedFormat→4, else→1),
  `pub mod metadata`. 16 new tests (8 unit + 8 integration). Mirrors DEC-029's
  probe exactly; container lane, no pixel re-encode. All gates green; no new DEC.
- [x] verify (2026-06-18) — independent read-only Explore subagent: ✅ APPROVED,
  no punch list. Confirmed the no-pixel-encode invariant (lane never builds an
  `Image`), exact DEC-029 API match, all exit codes (4/1/2/2/6/5/3), JPEG+PNG
  coverage honesty, no `unwrap` off test paths, all 16 named tests present +
  substantive, no DEC-003/DEC-029 drift. Orchestrator independently re-ran the
  gates on the branch: `cargo test` 308 ok, clippy/fmt/deny clean.
- [x] ship (2026-06-18, PR #30 squash-merged → `44afb81`) — reflections + cost
  totals filled (build 172444 real / verify ~50k estimate / design+ship null),
  STAGE-004 backlog flipped, archived to `specs/done/`, `just cost-audit` green +
  cost-capture confirmed on main CI.
