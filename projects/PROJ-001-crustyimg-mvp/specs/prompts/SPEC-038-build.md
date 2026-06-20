# SPEC-038 build prompt — cargo publish metadata + dual license files

Start a **fresh session**. You are the IMPLEMENTER for SPEC-038 in the `crustyimg`
repo (cwd is the repo root). This is a **packaging/metadata chore** — make the crate
publish-ready and VERIFY with `cargo package`/`cargo publish --dry-run`. **Do NOT
publish anything.** No Rust unit tests are added. Open a PR and STOP. Follow this
prompt exactly.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-038-cargo-publish-metadata-and-dual-license-files.md`
   — especially `## Publish hygiene (PINNED)`, `## Acceptance Criteria`, `## Notes`.
2. `decisions/DEC-018` (the `MIT OR Apache-2.0` license policy).
3. `Cargo.toml` (`[package]`), `LICENSE` (Apache text today), `README.md`.

## What to build
- **`Cargo.toml` `[package]`** — ADD (keep existing name/version/edition/description/
  `license = "MIT OR Apache-2.0"`):
  ```
  repository = "https://github.com/jysf/crustyimg"
  homepage = "https://github.com/jysf/crustyimg"
  readme = "README.md"
  keywords = ["image", "cli", "webp", "resize", "optimize"]
  categories = ["command-line-utilities", "multimedia::images"]
  exclude = ["/decisions", "/docs", "/projects", "/reports", "/guidance", "/feedback", "/scripts", "/.github", "/.claude"]
  ```
  Do NOT add `authors` (avoid publishing a personal email). Do NOT add `rust-version`
  (MSRV is a separate later item). Do NOT change `license`.
- **License files** — `git mv LICENSE LICENSE-APACHE`, then create `LICENSE-MIT` with
  the canonical MIT License text and `Copyright (c) 2026 jysf`. End state: exactly
  `LICENSE-MIT` + `LICENSE-APACHE` (no bare ambiguous `LICENSE`).

## Hard rules
- **No publish.** Use ONLY `cargo package --list`, `cargo package`, and/or `cargo
  publish --dry-run` — none of these upload. NEVER run a bare `cargo publish`.
- **Verify the packaged file set** with `cargo package --list` and confirm:
  - INCLUDED: `assets/fonts/Go-Regular.ttf` (the bundled font is `include_bytes!`'d —
    if it's dropped the crate won't build), `src/...`, `Cargo.toml`, `README.md`,
    `LICENSE-MIT`, `LICENSE-APACHE`, `deny.toml`.
  - EXCLUDED: anything under `decisions/`, `docs/`, `projects/`, `reports/`,
    `guidance/`, `feedback/`, `scripts/`, `.github/`, `.claude/`.
  Paste the `cargo package --list` output into `## Build Completion`.
- `categories` MUST be valid crates.io slugs (`command-line-utilities`,
  `multimedia::images`) — an invalid slug fails the dry-run.
- The crate must still build + all gates pass (the metadata must not change
  compilation or the dep tree). Do NOT add/remove dependencies.
- If `cargo publish --dry-run` needs crates.io network and the runner is offline,
  fall back to `cargo package` (still builds the `.crate`) and note the limitation in
  Build Completion.

## Gates (all must pass)
```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --no-default-features
cargo deny check advisories bans sources licenses
cargo package --list          # inspect the file set (see Hard rules)
cargo publish --dry-run       # build the package, NO upload (or note offline)
```

## Git / PR
- Branch `feat/spec-038-publish-metadata` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked `reports/*.md`
  and `TESTING-WITH-YOUR-PHOTOS.md` (do NOT stage them).
- PR title: `chore(release): cargo publish metadata + dual license files (SPEC-038)`.
- PR body per AGENTS.md §13 (Decisions referenced — DEC-018 / Constraints —
  `no-agpl-default-deps` / New decisions — none).
- Fill the spec's `## Build Completion` (incl. the `cargo package --list` output) + 3
  reflection answers; append the build cost session (numerics null; agent `claude-sonnet-4-6`).

## Cost
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-19
  notes: "publish metadata: Cargo.toml repository/homepage/readme/keywords/categories/exclude + LICENSE-MIT/LICENSE-APACHE (git mv LICENSE); verified lean package via cargo package --list (assets font in, scaffolding out) + cargo publish --dry-run; NO publish; no dep/DEC"
```

## When done
`just advance-cycle SPEC-038 verify` (if it mis-globs or doesn't update the spec's
`cycle:` field, set `cycle: verify` in the spec frontmatter by hand), open the PR with
`gh`, and **stop** — the orchestrator pauses for the user before any merge.
