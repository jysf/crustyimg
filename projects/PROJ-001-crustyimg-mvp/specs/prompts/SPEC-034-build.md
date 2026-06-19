# SPEC-034 build prompt — path/symlink traversal hardening (Source + Sink)

Start a **fresh session**. You are the IMPLEMENTER for SPEC-034 in the `crustyimg`
repo (cwd is the repo root). The architect (Opus) wrote the spec, its failing tests,
and **DEC-035** (the policy). **No new dependency** — `std::fs::{symlink_metadata,
canonicalize}` only. Make the spec's `## Failing Tests` pass with the smallest correct
change, then open a PR and STOP. Follow this prompt exactly.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-034-path-and-symlink-traversal-hardening-across-source-and-sink.md`
   — especially `## Hardening policy (PINNED)`, `## Failing Tests`, `## Notes for the
   Implementer` (read the "Honest note on the glob fix").
2. `decisions/DEC-035` (the policy), `DEC-010` (escape-check origin), `DEC-007`.
3. `src/sink/mod.rs` — `safe_join`, `guard_overwrite`, `Sink::write` (File/Dir arms),
   `Sink::write_bytes` (File/Dir arms), `SinkError::Traversal`. `src/source/mod.rs` —
   `resolve_glob` (the `root_opt = None` bypass), `resolve_directory` (robust reference).

## What to build
- **`src/sink/mod.rs`:**
  - Add `fn reject_symlink_destination(path: &Path) -> Result<(), SinkError>`:
    `match std::fs::symlink_metadata(path) { Ok(m) if m.file_type().is_symlink() =>
    Err(SinkError::Traversal(path.display().to_string())), _ => Ok(()) }`. (A missing
    path → `Err` from symlink_metadata → falls to `_ => Ok(())`, which is correct.)
  - Call it in ALL FOUR file-producing arms, **before opening the file** and
    **regardless of `overwrite`** (do NOT gate behind `Overwrite::Allow`):
    - `Sink::write` `File { path, .. }` — on `path`.
    - `Sink::write` `Dir { .. }` — on the `full_path` from `safe_join` (after safe_join,
      around `guard_overwrite`).
    - `Sink::write_bytes` `File { path, .. }` — on `path`.
    - `Sink::write_bytes` `Dir { .. }` — on the `full_path` from `safe_join`.
- **`src/source/mod.rs`:** in `resolve_glob`, change
  `let root_opt = std::fs::canonicalize(&base).ok();` to
  `let root_opt = std::fs::canonicalize(&base).or_else(|_| std::fs::canonicalize(".")).ok();`
  Keep the existing `if let Some(ref root) = root_opt { … }` per-entry escape check
  unchanged. Do NOT touch `resolve_directory` (already robust) — only add its test.

## Hard rules
- **Smallest correct change.** Reuse `SinkError::Traversal` (exit 5, already mapped in
  `CliError::code` via `CliError::Sink(_) => 5`) — no new error variant or exit code.
  **No new dependency.** No `unwrap`/`expect`/`panic!` on the new non-test paths (DEC-007).
- The symlink-destination rejection is enforced **even with `--yes`** (it's a traversal
  guard, not an overwrite choice) — a test pins exactly this.
- Symlink tests are **Unix-gated**: `#[cfg(unix)]` + `std::os::unix::fs::symlink`. The
  production code stays cross-platform (`symlink_metadata`/`canonicalize` exist on
  Windows); only the test fixtures use Unix symlinks. Non-symlink tests are unconditional.
- The glob/dir escape-skip tests pin the security PROPERTY (an entry resolving outside
  the root is skipped); per the spec's honest note, they may already pass against a
  canonicalizable base — that is expected, do NOT contrive an unreachable case.
- Every named test in `## Failing Tests` must EXIST and PASS:
  - `src/sink/mod.rs`: `reject_symlink_destination_ok_for_regular_and_missing`,
    `#[cfg(unix)] reject_symlink_destination_rejects_symlink`,
    `#[cfg(unix)] write_file_through_symlink_is_rejected_even_with_yes`,
    `#[cfg(unix)] write_bytes_dir_through_symlink_is_rejected`,
    `write_dir_normal_destination_still_succeeds`.
  - `src/source/mod.rs`: `#[cfg(unix)] glob_skips_symlink_escaping_root`,
    `#[cfg(unix)] directory_skips_symlink_escaping_root`, `glob_resolves_all_in_tree_images`.
- Run clippy right after writing doc comments (the SPEC-031 `doc_lazy_continuation` lesson).

## Gates (all must pass — INCLUDING the lean build)
```
cargo fmt && git add -u
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --no-default-features
cargo deny check licenses
```

## Git / PR
- Branch `feat/spec-034-traversal-hardening` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked `reports/*.md`.
- PR title: `feat(io): path/symlink traversal hardening (SPEC-034)`.
- PR body per AGENTS.md §13 (Decisions referenced — DEC-035, DEC-010, DEC-007 /
  Constraints — `untrusted-input-hardening` / New decisions — "DEC-035 at design").
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
  notes: "traversal hardening: reject_symlink_destination on all 4 Sink file arms (reject even with --yes) + glob root cwd-anchor fallback (close root_opt=None bypass); reuse SinkError::Traversal exit 5; std-only, no new dep"
```

## When done
`just advance-cycle SPEC-034 verify` (if it mis-globs or doesn't update the spec's
`cycle:` field, set `cycle: verify` in the spec frontmatter by hand), open the PR with
`gh`, and **stop** — the orchestrator pauses for the user before any merge.
