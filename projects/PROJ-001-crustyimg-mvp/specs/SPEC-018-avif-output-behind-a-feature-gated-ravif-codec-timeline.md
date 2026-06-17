# SPEC-018 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-018-<cycle>.md`.

## Instructions

- [x] **design** — spec + `## Failing Tests` + Implementation Context authored by the ORCHESTRATOR (Opus) directly. **Verified the dep empirically** (the SPEC-016 discipline): `image/avif`→`ravif` builds pure-Rust with no nasm; the tree is permissive in the shipped binary (the lone NCSA crate `libfuzzer-sys` is fuzz-only via rav1e, not shipped → scoped deny exception). Pinned `AvifEncoder::new_with_speed_quality`. Emitted **DEC-020**. Build prompt at `prompts/SPEC-018-build.md`. Completed 2026-06-16.
- [x] **build** — orchestrator-direct (Opus, main loop — background subagents can't get Bash here), per `prompts/SPEC-018-build.md`: `avif = ["image/avif"]` feature + scoped `libfuzzer-sys` deny exception + sink AVIF encode (`CodecNotBuilt`/`ensure_codec_built`→exit 4 without the feature) + quality AVIF arm + CI `--features avif` job. **Build-time correction:** perceptual AVIF (`--target`/`--ssim`) needs a decoder (deferred) → split the `LossyFormat` seam; only `--max-size` drives AVIF (DEC-020 amended). Both builds + `just deny` green. **PR #21 opened.** Branch `feat/spec-018-avif-output-behind-a-feature-gated-ravif-codec`. Completed 2026-06-16.
- [ ] **verify** — confirm default build unchanged + exit 4 for avif; feature build produces valid AVIF (`guess_format`); auto-quality/`--max-size` drive AVIF; `just deny` green with the scoped exception; all gates on BOTH builds.
- [ ] **ship** — orchestrator bookkeeping on `main` after merge (real cost numbers; PAUSE before merge/ship).
