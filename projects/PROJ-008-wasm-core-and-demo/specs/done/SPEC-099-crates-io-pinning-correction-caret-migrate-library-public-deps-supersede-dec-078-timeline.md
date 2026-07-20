# SPEC-099 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-099-<cycle>.md`.

## Instructions
- [x] design — framed 2026-07-19. Corrects DEC-078's FALSE premise (it assumed crustyimg wasn't on
  crates.io; it's been published since v0.1.0, latest 0.4.0, `has_lib`, auto-per-tag — verified via
  crates.io API + `gh run list`). So the 30 exact `=` pins are live on a published lib. Spec = (1)
  caret-migrate the ~23 runtime dep reqs (strip `=`; `[dependencies]` + both `[target.*.dependencies]`),
  keep the 4 dev-deps pinned; **`Cargo.lock` MUST stay byte-unchanged** (caret ⊇ exact); (2) DEC-079
  supersedes DEC-078 (draft in spec); (3) de-stale RELEASING.md/STAGE-007/DEC-041/audit-D4; AGENTS §5
  repoints. Reproducibility stays via the committed lock (PROJ-007 intact). MSRV is the one real risk (CI
  has no `--locked`). **Build gated on maintainer go** (supersedes a shipped DEC + edits the published
  manifest). Complexity S.
- [x] build — Sonnet, primary checkout. 26 runtime dep rows caret-migrated (11 `[dependencies]` + 12
  native-target + 3 wasm-target); 4 dev-deps kept pinned. **`Cargo.lock` byte-unchanged** through every
  check. DEC-079 written (supersedes DEC-078; DEC-078 `superseded_by` set, body preserved). De-staled
  RELEASING.md/STAGE-007/DEC-041/audit-D4/AGENTS §5 (+ a bonus stale hosting line). MSRV verified on the
  specific newly-permitted patches. Negative control real.
- [x] verify — ✅ CLEAN (Opus, adversarial). Independently re-derived the lock-unchanged gate (sha held
  across ~7 invocations), ran a **fresh `cargo update` + MSRV 1.90 recompile** (gold standard — every
  permitted patch compiles), reproduced the negative control. **Found + fixed a real defect:** the build's
  single-banner de-stale left 3 residual "not on crates.io" claims (summary table, Disposition record,
  prior-got-wrong conclusion) — corrected as verify hardening (5c33927). Full matrix green.
- [x] ship — squash-merged PR #104 (**106d5bf**) 2026-07-19, CI CLEAN. DEC-078's false premise corrected;
  the published manifest is now consumer-friendly (caret) with reproducibility via the committed lock. No
  new DEC beyond DEC-079. Bookkeeping: cycle→ship, 3 cost sessions (build Sonnet $3.5 / verify Opus $3 /
  ship $0.5 ≈ **$7.0**), timeline, archive, STAGE-031 backlog. **Operational follow-up: cut 0.5.0 to
  actually publish the fix.**
