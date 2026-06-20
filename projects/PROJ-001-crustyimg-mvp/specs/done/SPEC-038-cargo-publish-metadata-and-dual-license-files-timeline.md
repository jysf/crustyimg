# SPEC-038 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-038-<cycle>.md`.

## Timeline

- [x] **design** (2026-06-19, Opus) — authored the spec (`## Publish hygiene (PINNED)`,
  acceptance via `cargo package`/`--dry-run`, Implementation Context). Confirmed the
  crates.io name `crustyimg` is free; identified the gaps (missing repository/keywords/
  categories/readme/exclude; single Apache-only LICENSE → needs dual MIT/Apache files;
  `assets/` font must stay in the package). No publish — dry-run/`--list` only. No new
  dep/DEC. Build prompt at `prompts/SPEC-038-build.md`.
- [x] **build** (2026-06-19, Sonnet 4.6) — PR #42; Cargo.toml metadata + `exclude` +
  `git mv LICENSE LICENSE-APACHE` + new `LICENSE-MIT`. `cargo package --list`: assets font
  IN, scaffolding OUT; `cargo publish --dry-run` clean (no upload). 411 tests; clippy/fmt/
  lean/deny clean. subagent tokens=55206 (~$0.30).
- [x] **verify** (2026-06-19, Opus Explore) — APPROVED; re-ran `cargo package --list` +
  `--dry-run` — bundled font packaged, scaffolding excluded, category slugs valid, dual
  license files match, no dep change, nothing published. ~50k est.
- [x] **ship** (2026-06-19) — squash-merged PR #42 (a44887d); cost totals + ship reflection
  + archived to `specs/done/`. STAGE-007 backlog #1 complete (crate is publish-ready).
