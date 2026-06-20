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
- [ ] **build** (Sonnet) — add metadata + license files; `cargo package --list` clean; see prompt.
- [ ] **verify** (Opus, Explore) — confirm the packaged file set (assets in, scaffolding out),
  license files, valid categories; gate re-run.
- [ ] **ship** — pause for the user before merge.
