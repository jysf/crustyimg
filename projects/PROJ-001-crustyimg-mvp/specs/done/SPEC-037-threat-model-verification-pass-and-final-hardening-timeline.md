# SPEC-037 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-037-<cycle>.md`.

## Timeline

- [x] **design** (2026-06-19, Opus) — authored the spec (`## Hardening policy (PINNED)`,
  `## Failing Tests`, `## Implementation Context`) + **DEC-038** (resize output cap). Read
  `Resize::apply` (one `(tw,th)` choke point for all six modes) + `run_edit` (raw
  save-recipe write). Scope: resize output ≤ 512 MiB at apply (== decode cap, DEC-034) +
  `edit --save-recipe` reusing the Sink's `reject_symlink_destination` (DEC-035) + a
  `SECURITY.md` threat-model verification table (authored now). The verify cycle doubles
  as an adversarial security review over the cumulative STAGE-006 diff. std-only, no new
  dep. Build prompt at `prompts/SPEC-037-build.md`.
- [x] **build** (2026-06-19, Sonnet 4.6) — PR #41; save-recipe symlink guard (reuse
  `sink::reject_symlink_destination`, `pub(crate)`) + resize cap. The build DISCOVERED the
  pre-existing SPEC-010 `MAX_EDGE`/`MAX_AREA` guards and flagged the overlap. 411 tests
  green. subagent tokens=82390 (~$0.45).
- [x] **correction** (2026-06-19, Opus/architect) — dropped the redundant new const the
  build added; tightened the existing `MAX_AREA` 256→128 Mpx (512 MiB RGBA) for decode
  symmetry; rewrote DEC-038 + spec to the honest framing (resize was already bounded). 411
  tests still green (`744f6ed`).
- [x] **verify + security review** (2026-06-19, Opus Explore) — APPROVED; verified both
  fixes AND ran an adversarial sweep over the cumulative STAGE-006 surface (decode / paths /
  recipes / supply chain / error hygiene) — all output writes guarded, no untrusted-path
  panics, every SECURITY.md mitigation confirmed, no unresolved finding. Gates green (411).
- [x] **ship** (2026-06-19) — squash-merged PR #41 (4579999); cost totals + ship reflection
  + archived to `specs/done/`. **STAGE-006 — the MVP exit gate — complete.**
