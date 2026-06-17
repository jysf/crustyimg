# SPEC-019 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-019-<cycle>.md`.

## Instructions

- [x] **design** — spec + `## Failing Tests` + Implementation Context authored by the ORCHESTRATOR (Opus) directly. **Verified the dep empirically** (the discipline, extended to the decode path): `image` 0.25.10 `webp` feature → `image-webp` 0.2.4 (MIT/Apache) builds pure-Rust with no nasm; it DECODES lossy+lossless but ENCODES lossless only; `just deny` GREEN with NO new exception; `write_to(_, WebP)` routes to `new_lossless` (no special sink arm). Emitted **DEC-021** (WebP lossless+decode as a pure-Rust DEFAULT; lossy deferred to SPEC-020). Build prompt at `prompts/SPEC-019-build.md`. Completed 2026-06-17.
- [ ] **build** — Sonnet 4.6 (or orchestrator-direct if subagent Bash blocked), per `prompts/SPEC-019-build.md`: add `"webp"` to the image default features + `format_from_extension` webp arm; tests for lossless WebP output, `.webp` input decode, shrink→webp, `-q` ignored; drop the webp branch from `convert_unbuilt_codec_exits_4`; docs. NO new dep, NO sink encode arm, NO quality change. Branch `feat/spec-019-webp-lossless-output-and-decode`.
- [ ] **verify** — confirm `.webp` input decodes; `convert --format webp` / `-o x.webp` produce valid lossless WebP (`guess_format` + round-trip equality); `-q` ignored like PNG; `just deny` green with NO new exception; `convert --format webp` no longer exits 4; all gates + 3-OS CI.
- [ ] **ship** — orchestrator bookkeeping on `main` after merge (real cost numbers; PAUSE before merge/ship).
