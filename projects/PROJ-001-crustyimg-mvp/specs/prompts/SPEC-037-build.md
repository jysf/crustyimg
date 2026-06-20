# SPEC-037 build prompt — threat-model capstone (resize cap + save-recipe symlink guard)

Start a **fresh session**. You are the IMPLEMENTER for SPEC-037 in the `crustyimg`
repo (cwd is the repo root). The architect (Opus) wrote the spec, its failing tests,
and **DEC-038** (the resize cap). **No new dependency** — `std` only. Make the spec's
`## Failing Tests` pass with the smallest correct change, then open a PR and STOP.
This is the STAGE-006 capstone. Follow this prompt exactly.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-037-threat-model-verification-pass-and-final-hardening.md`
   — especially `## Hardening policy (PINNED)`, `## Failing Tests`, `## Notes for the Implementer`.
2. `decisions/DEC-038` (resize output cap), `DEC-035` (symlink-destination policy),
   `DEC-034` (the symmetric decode cap), `DEC-007`.
3. `src/operation/mod.rs` — `Resize::apply` (the `let (tw, th) = match self.mode {…}`
   point), `OperationError::Apply`. `src/sink/mod.rs` — `reject_symlink_destination`
   (make `pub(crate)`). `src/cli/mod.rs` — `run_edit` (the `std::fs::write(path, toml)`
   recipe-save).

## What to build
- **`src/operation/mod.rs`** — add `const MAX_RESIZE_OUTPUT_BYTES: u64 = 512 * 1024 *
  1024;` (comment: == the decode alloc cap, DEC-034). In `Resize::apply`, on the line
  IMMEDIATELY AFTER `let (tw, th) = match self.mode { … };` (before the image is
  resized/allocated), add:
  ```
  if (tw as u64) * (th as u64) * 4 > MAX_RESIZE_OUTPUT_BYTES {
      return Err(OperationError::Apply {
          op: "resize",
          reason: format!("resize output {tw}x{th} exceeds the {MAX_RESIZE_OUTPUT_BYTES} byte limit"),
      });
  }
  ```
  One check, all six modes. Do NOT change `from_params` or the per-mode dim math.
- **`src/sink/mod.rs`** — change `fn reject_symlink_destination` → `pub(crate) fn
  reject_symlink_destination` (visibility only; no behavior change).
- **`src/cli/mod.rs`** (`run_edit`) — before `std::fs::write(path, toml)` (the
  `--save-recipe` write), add `crate::sink::reject_symlink_destination(std::path::Path::new(path))?;`
  (it returns `SinkError::Traversal`; the `?` flows through `run_edit`'s existing
  `CliError::Sink` mapping → exit 5). Do NOT write a second symlink check — REUSE the
  sink helper.
- **`SECURITY.md`** and **`docs/api-contract.md`** were authored at design; leave them
  unless a test/finding contradicts a claim (then fix the CODE, not the doc).

## Hard rules
- **Smallest correct change.** Reuse `OperationError::Apply` and
  `sink::reject_symlink_destination` — no new error variant, no new exit code, no new
  dependency. No `unwrap`/`expect`/`panic!` on the new non-test paths (DEC-007).
- Cap is exactly **512 MiB**; **reject, never clamp**. The guard MUST precede the resize
  allocation (a test pins that a 2×2 → `exact 40000x40000` is rejected without OOM).
- **Over-cap tests use a TINY input + huge target** so the guard fires before any large
  allocation — never construct a multi-GB buffer in a test.
- Symlink tests are `#[cfg(unix)]` (`std::os::unix::fs::symlink`).
- Every named test in `## Failing Tests` must EXIST and PASS:
  - `src/operation/mod.rs`: `resize_apply_exact_rejects_oversized_output`,
    `resize_apply_percent_rejects_oversized_output`, `resize_apply_normal_outputs_succeed`.
  - `src/sink/mod.rs`: `reject_symlink_destination_is_crate_visible` (only if the existing
    SPEC-034 tests don't already cover the regular + symlink cases — avoid duplication).
  - `tests/edit.rs`: `#[cfg(unix)] edit_save_recipe_through_symlink_is_rejected`,
    `edit_save_recipe_normal_path_still_works`.
  - `tests/apply_batch.rs`: `apply_recipe_with_oversized_resize_exits_1`.
- Run clippy right after writing doc comments (the SPEC-031 `doc_lazy_continuation` lesson).

## Gates (all must pass — INCLUDING the lean build)
```
cargo fmt && git add -u
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --no-default-features
cargo deny check advisories bans sources licenses
```

## Git / PR
- Branch `feat/spec-037-final-hardening` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked `reports/*.md`
  and `TESTING-WITH-YOUR-PHOTOS.md` (do NOT stage them).
- PR title: `feat(hardening): resize output cap + save-recipe symlink guard (SPEC-037)`.
- PR body per AGENTS.md §13 (Decisions referenced — DEC-038, DEC-035, DEC-034, DEC-007 /
  Constraints — `untrusted-input-hardening` / New decisions — "DEC-038 at design").
- Fill the spec's `## Build Completion` + 3 reflection answers; append the build cost
  session (numerics null; agent `claude-sonnet-4-6`).

## Cost
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-19
  notes: "STAGE-006 capstone: resize output 512MiB cap in Resize::apply (all modes, DEC-038) + edit --save-recipe reuses sink::reject_symlink_destination (pub(crate), DEC-035); reuse OperationError::Apply (exit 1) / SinkError::Traversal (exit 5); std-only, no new dep"
```

## When done
`just advance-cycle SPEC-037 verify` (if it mis-globs or doesn't update the spec's
`cycle:` field, set `cycle: verify` in the spec frontmatter by hand), open the PR with
`gh`, and **stop** — the orchestrator pauses for the user before any merge.
