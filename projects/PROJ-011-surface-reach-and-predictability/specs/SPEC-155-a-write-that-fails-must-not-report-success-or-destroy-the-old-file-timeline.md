# SPEC-155 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-155-<cycle>.md`.

## Instructions

- [x] **frame** (2026-11-22, Opus): GO at [M]. Premise driven on a `main` release build under
  `ulimit -f 1` with XFSZ ignored: outputs under 8 KiB exit **0** with a 512 B PNG (`convert -o`,
  `--out-dir`, `meta strip`). Larger ones exit 5 with the old file destroyed. `--save-recipe`
  overwrites without `-y`. A new DEC ("a file write is all or nothing") is recommended, and four
  rulings are open (read-only directory, fsync, `--save-recipe` guard, full disk).
- [x] **rulings** (maintainer, 2026-11-22): all four recommendations accepted; see the spec's Frame.
- [x] **design** (2026-11-22, Opus): `prompts/SPEC-155-design.md`. One hand-rolled helper
  (`src/sink/publish.rs`) under the four sink arms, `--save-recipe` (guarded before the run) and
  `write_lock`; 22 ACs, 39 tests, 15 mutation rows driven on a prototype. Corrections: a rename
  replaces a `0444` file (kept refused by a write-probe), std's Windows rename tries `MoveFileExW`
  first, the `.tmp` suffix (not the dot) keeps staged files out of inputs.
- [x] **build** (2026-11-22, Sonnet, $7.61): draft PR #268, head `471a0a7`; CI green on every leg
  at `47003ae` (the last code push). Baseline: exactly the 15 E rows red on `main`, 8 pins green.
  22 ACs met; W1–W4 passed on the Windows leg. Mutations matched except M1/M2/M3 also redden E23 and
  M11 is 48, not 46. `DEC-116`. The orchestrator's "exits 5 (6 for a batch …)" correction applied.
- [x] **verify** (2026-11-23, Opus, $4.47): ⚠ PUNCH LIST at `471a0a7`. Every failure path driven on
  release builds; writer sweep clean; M1/M2/M3 → E23 and M11 = 48 confirmed. Two **claims** in the
  spec's own steps: the mode is copied from a symlink (`write_lock` has no symlink guard; a dangling
  lockfile link now exits 5), and it is applied after the bytes are written (a killed `-y` over
  `0600` leaves a `0644` staged file). Two **asserts**: the char-boundary cut and the `.tmp` suffix
  have no row. Out of spec: `optimize`'s note → `guidance/ideas.yaml`.
- [ ] **punch list**: `prompts/SPEC-155-punchlist.md` (Sonnet), then a narrow re-verify of the two claims
- [ ] **ship**
