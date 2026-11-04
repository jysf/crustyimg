# SPEC-144 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-144-<cycle>.md`.

## Instructions

- [x] **frame** (`prompts/SPEC-144-frame.md`), 2026-10-31, Opus, GO, low [M]. Premise confirmed from
  run 41251779509 (macos-14 brownout ×2) and the 0.8.2 timestamps (crates.io/npm out 2 h before the
  Release). Fix: 3-line `[dist.github-custom-runners]` (release.yml unchanged); npm + crates.io as
  cargo-dist custom publish jobs after `host`, npm by OIDC (trusted publisher names `release.yml`).
- [x] **design** (`prompts/SPEC-144-design.md`), 2026-11-02, Opus, [M] upper end, not cut. 18 ACs.
  Checks: C1–C7, G1–G14, R1–R6, L1–L4, D1–D2, A1, V1–V3, 16 mutations. Probes found that custom
  jobs get no `contents` (fixed by `github-custom-job-permissions`), that `npm view` cannot tell absent from
  unreachable (the guard reads the package document and the sparse index), and that `dist plan` is already a drift check. DEC-111 written.
- [x] **build** (2026-11-04, `prompts/SPEC-144-build.md`, Opus, $11.97): draft PR #243 at `17c8b8a`,
  CI green. Upload-mode run 41786007198 built all four targets (aarch64 on `macos-15`, arm64
  binary); (b) `400092f` removed it. Two design calls changed: an npm wrapper for the PR dry run,
  and AC-15 relaxed for dist's generated-file shellcheck findings. M1a–M19 red for their reason.
- [ ] **verify**: `prompts/SPEC-144-verify.md`
- [ ] **ship**
