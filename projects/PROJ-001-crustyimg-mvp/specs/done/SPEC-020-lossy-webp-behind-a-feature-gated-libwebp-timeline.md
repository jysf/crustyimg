# SPEC-020 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-020-<cycle>.md`.

## Instructions

- [x] **design** — spec + `## Failing Tests` + Implementation Context authored by the ORCHESTRATOR (Opus) directly. **Verified the dep empirically** (the discipline): `webp` 0.3.1 → `libwebp-sys` 0.9.6 builds via `cc` (VENDORED libwebp, no system lib install); licenses webp MIT/Apache + libwebp-sys MIT + vendored libwebp BSD-3 → `just deny` GREEN with NO new exception (tested with the feature enabled). Pinned `webp::Encoder::from_rgba(&[u8], w, h).encode(q as f32)`; use from_rgba on to_rgba8() bytes (not the webp `image` feature). Emitted **DEC-022** (lossy WebP via libwebp behind off-by-default `webp-lossy`; first C dep, opt-in; BOTH searches drive WebP — the decoder exists). Build prompt at `prompts/SPEC-020-build.md`. Completed 2026-06-17.
- [x] **build** — orchestrator-direct (Opus, main loop), per `prompts/SPEC-020-build.md`: `webp` optional dep + `webp-lossy = ["dep:webp"]`; sink WebP lossy arm (lossy iff quality set, else lossless fall-through); quality WebP arm + BOTH predicates (cfg-gated matches); `--features webp-lossy` CI job; docs. Two deviations: dropped the would-be-dead `WEBP_DEFAULT_QUALITY` const; rewrote predicates as match arms. Default/avif/webp-lossy builds + `just deny` all green (NO new exception). **PR #23 opened.** Branch `feat/spec-020-lossy-webp-behind-a-feature-gated-libwebp`. Completed 2026-06-17.
- [x] **verify** — independent review (Opus, 2 focused Explore subagents): sink↔search cross-sync byte-identity + lossy-iff-quality selection; predicate-rewrite regression + default-build-unchanged + single-image-library (one image crate, from_rgba not from_image). Both clean — no findings. Default build unchanged (lossless WebP, `-q` ignored); feature build produces lossy WebP smaller than lossless; BOTH searches drive WebP (perceptual works — the AVIF contrast); `just deny` green with NO new exception; all gates on default/avif/webp-lossy builds + 3-OS CI + the webp-lossy job. Completed 2026-06-17.
- [x] **ship** — squash-merged **PR #23** to `main` (`eccf621`); branch deleted; CI green. Bookkeeping by hand on `main`: cost sessions (build/verify labeled estimates — main-loop), Reflection (Ship), STAGE-008 backlog + Count (5 shipped), spec archived to `specs/done/`. Completed 2026-06-17.
