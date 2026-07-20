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
