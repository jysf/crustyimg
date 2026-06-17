# SPEC-018 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-018-<cycle>.md`.

## Instructions

- [x] **design** — spec + `## Failing Tests` + Implementation Context authored by the ORCHESTRATOR (Opus) directly. **Verified the dep empirically** (the SPEC-016 discipline): `image/avif`→`ravif` builds pure-Rust with no nasm; the tree is permissive in the shipped binary (the lone NCSA crate `libfuzzer-sys` is fuzz-only via rav1e, not shipped → scoped deny exception). Pinned `AvifEncoder::new_with_speed_quality`. Emitted **DEC-020**. Build prompt at `prompts/SPEC-018-build.md`. Completed 2026-06-16.
- [x] **build** — orchestrator-direct (Opus, main loop — background subagents can't get Bash here), per `prompts/SPEC-018-build.md`: `avif = ["image/avif"]` feature + scoped `libfuzzer-sys` deny exception + sink AVIF encode (`CodecNotBuilt`/`ensure_codec_built`→exit 4 without the feature) + quality AVIF arm + CI `--features avif` job. **Build-time correction:** perceptual AVIF (`--target`/`--ssim`) needs a decoder (deferred) → split the `LossyFormat` seam; only `--max-size` drives AVIF (DEC-020 amended). Both builds + `just deny` green. **PR #21 opened.** Branch `feat/spec-018-avif-output-behind-a-feature-gated-ravif-codec`. Completed 2026-06-16.
- [x] **verify** — independent medium `/code-review` (Opus, 7 finder angles / 5 Explore subagents) + targeted empirical checks. Default build unchanged + exit 4 for avif; feature build produces valid AVIF (`guess_format`); `--max-size` drives AVIF; perceptual falls back gracefully (decode deferred); `just deny` green with the scoped exception; all gates on BOTH builds + 3-OS CI. One low-severity finding (multi-input shrink/resize→AVIF exits 6 not 4) tracked in the STAGE-008 backlog. Completed 2026-06-17.
- [x] **ship** — squash-merged **PR #21** to `main` (`98e333c`); branch deleted; CI green (incl. the avif job, cargo-deny, cost-audit). Bookkeeping by hand on `main`: cost sessions (build/verify labeled estimates — main-loop), Reflection (Ship), STAGE-008 backlog + Count (3 shipped / 0 active / 2 pending) + 3 follow-ups, spec archived to `specs/done/`. Completed 2026-06-17.
