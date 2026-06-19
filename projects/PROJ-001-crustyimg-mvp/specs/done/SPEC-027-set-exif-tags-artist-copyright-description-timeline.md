# SPEC-027 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-027-<cycle>.md`.

## Instructions

- [x] design (2026-06-18) — Opus, main loop. Authored the spec (`## Failing Tests`
  + `## Implementation Context`) on top of SPEC-026's metadata lane; `set` is one
  transform fn (`metadata::set_tags` + `TagSet`) + one handler (`run_set`) reusing
  `run_metadata_lane`/`Sink::write_bytes`. Ran a design-time probe confirming
  `little_exif` set-then-write PRESERVES existing tags (Orientation/GPS) and the
  no-EXIF fresh-create fallback, pixels byte-identical (real JPEG). No new dep / no
  new DEC. Fleshed out the `set` api-contract entry. Design pushed to `main` before
  build.
- [x] build (2026-06-18, PR #31) — foreground metered subagent (Opus, 113k tok,
  ~13 min). Added `metadata::TagSet` + `set_tags` (sniff → load-then-set preserve →
  write_to_vec) and `run_set` (at-least-one-flag → exit 2, else `run_metadata_lane`
  closure). 12 new tests (7 unit + 5 integration). No new dep, no new DEC. All gates
  green.
- [x] verify (2026-06-18) — independent read-only Explore subagent: ✅ APPROVED,
  no punch list. Confirmed the critical load-then-set PRESERVE pattern
  (`new_from_vec(..).unwrap_or_else(|_| Metadata::new())`, not unconditional
  `Metadata::new()`), no-pixel-encode invariant, exit codes (2/4/6/5), reuse of
  `run_metadata_lane`, all 12 tests present + substantive. Orchestrator re-ran the
  gates: `cargo test` 320 ok (0 failed), clippy/fmt/deny clean.
- [x] ship (2026-06-18, PR #31 squash-merged → `9db5c2f`) — reflections + cost
  totals filled (build 112775 real / verify ~45k est; totals 157775 / $1.41 / 4),
  STAGE-004 backlog flipped, archived to `specs/done/`, `just cost-audit` green +
  cost-capture confirmed on main CI.
