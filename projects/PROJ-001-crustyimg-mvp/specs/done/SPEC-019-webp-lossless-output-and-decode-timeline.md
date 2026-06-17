# SPEC-019 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-019-<cycle>.md`.

## Instructions

- [x] **design** — spec + `## Failing Tests` + Implementation Context authored by the ORCHESTRATOR (Opus) directly. **Verified the dep empirically** (the discipline, extended to the decode path): `image` 0.25.10 `webp` feature → `image-webp` 0.2.4 (MIT/Apache) builds pure-Rust with no nasm; it DECODES lossy+lossless but ENCODES lossless only; `just deny` GREEN with NO new exception; `write_to(_, WebP)` routes to `new_lossless` (no special sink arm). Emitted **DEC-021** (WebP lossless+decode as a pure-Rust DEFAULT; lossy deferred to SPEC-020). Build prompt at `prompts/SPEC-019-build.md`. Completed 2026-06-17.
- [x] **build** — orchestrator-direct (Opus, main loop), per `prompts/SPEC-019-build.md`: added `"webp"` to the image default features + the `format_from_extension` webp arm; lossless encode (write_to) + `.webp` decode came for free. Tests: unit + 4 integration (lossless round-trip, `.webp` input, shrink→webp, `-q` ignored); dropped the webp branch from `convert_unbuilt_codec_exits_4`; docs (api-contract). NO new dep, NO sink encode arm, NO quality change. Default + avif builds green; `just deny` green with NO new exception. **PR #22 opened.** Branch `feat/spec-019-webp-lossless-output-and-decode`. Completed 2026-06-17.
- [x] **verify** — independent focused review (Opus, 1 Explore subagent — proportionate to the tiny additive diff) + CI. `.webp` input decodes; `convert --format webp`/`-o x.webp` produce valid lossless WebP (`guess_format` + bit-exact round-trip); `-q` ignored like PNG; `just deny` green with NO new exception; `convert --format webp` no longer exits 4; all gates + 3-OS CI + the avif job green. Found + fixed 2 stale-comment nits; no correctness issues. Completed 2026-06-17.
- [x] **ship** — squash-merged **PR #22** to `main` (`da405d1`); branch deleted; CI green. Bookkeeping by hand on `main`: cost sessions (build/verify labeled estimates — main-loop), Reflection (Ship), STAGE-008 backlog + Count (4 shipped), spec archived to `specs/done/`. Completed 2026-06-17.
