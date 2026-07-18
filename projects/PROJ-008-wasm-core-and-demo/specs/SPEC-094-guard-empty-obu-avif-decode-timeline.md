# SPEC-094 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-094-<cycle>.md`.

## Instructions

- [x] design — framed build-ready 2026-07-18. SPEC-091 follow-up #2 (the correctness/fuzz one). An empty
  OBU stream reaches re_rav1d's debug-only `debug_abort()`, uncatchable by `catch_unwind` OR the
  scoped-thread `join` (abort ≠ unwind) → crashes debug builds (where the SPEC-069 fuzz gate + tests run),
  violating DEC-062. The alpha path (`avif.rs:168-169`) is unguarded; reachability plausible-but-unconfirmed.
  Fix = an `is_empty()` guard at the shared `decode_obus` chokepoint (covers primary + alpha). Confirm
  reachability FIRST. No DEC expected.
- [x] build — single session, primary checkout, branch `spec-094-empty-obu-guard`. Confirmed reachability
  FIRST (hand-built a conforming AVIF whose alpha item is genuinely empty via an `iloc` `ToEnd`-extent
  duplicate-offset trick; drove it through the pre-fix `decode_avif` in an isolated subprocess and observed
  `SIGABRT` — the bug is real, not hypothetical). Guard added at the `decode_obus` chokepoint (1 `send_data`
  site, 1 `Decoder::with_settings` site, 2 callers — all covered). 3 failing tests written and made to pass;
  fuzz target survives the crafted input in debug; valid AVIFs (with/without alpha) proven pixel-identical
  against the pre-fix binary. All gates clean. No DEC. PR #97 opened against main.
- [x] verify — ✅ APPROVED 2026-07-18 (claude-opus-4-8, main-loop interactive, ~$5.85 est). Reachability
  proven independently: regenerated the crafted AVIF, removed the guard, drove the pre-fix `SIGABRT` from
  `re_rav1d::rav1d_send_data` (container-driven empty alpha, not a direct empty-slice call — genuine
  reachability). Fuzz gate proven in BOTH directions: clean post-fix (2 ms, no ASAN), `deadly signal` with
  the guard removed. Grep re-run (1 send_data / 1 Decoder / 2 callers). Valid-alpha decode is a definitional
  no-op (guard = early-return on empty); fixture carries a real `auxC` alpha item. Gates green (759 default /
  771 avif / clippy x2 / fmt / lean / validate); PR #97 CI 27/27 incl. avif on all three OSes. No punch list.
  Build (Sonnet) indistinguishable from Opus on the hard parts.
- [ ] ship — orchestrator.
