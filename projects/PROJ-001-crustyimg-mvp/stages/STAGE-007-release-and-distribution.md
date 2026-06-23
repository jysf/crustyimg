---
# Maps to ContextCore epic-level conventions.
# A Stage is a coherent chunk of work within a Project.
# It has a spec backlog and ships as a unit when the backlog is done.

stage:
  id: STAGE-007                     # stable, zero-padded within the project
  status: active                    # proposed | active | shipped | cancelled | on_hold
  priority: medium                  # critical | high | medium | low
  target_complete: null             # optional: YYYY-MM-DD

project:
  id: PROJ-001                      # parent project
repo:
  id: crustyimg

created_at: 2026-06-14
shipped_at: null

# What part of the project's value thesis this stage advances.
value_contribution:
  advances: >
    Makes crustyimg actually obtainable — delivers the brief's
    "installable from a release artifact" success criterion. Pure release
    engineering; no new image features.
  delivers:
    - Versioned, cross-platform binaries published to GitHub Releases
    - `brew install` via a Homebrew tap
    - (optional) `cargo install crustyimg` from crates.io
    - A README/usage that tells a new user how to install and run it
  explicitly_does_not:
    - Add any new image operation or command (those are STAGE-002..005)
    - Do the security hardening (that is STAGE-006, which must precede release)
---

# STAGE-007: release and distribution

## What This Stage Is

The final PROJ-001 stage: turn the working, hardened binary into something
a stranger can install in one line. It produces versioned cross-platform
release artifacts, a Homebrew tap, optional crates.io publication, and the
install/usage docs to match. No new image functionality — this is the
"ship it to humans" stage.

## Why Now

Last, by necessity. Releasing only makes sense once the MVP commands are
real (STAGE-002..005) and the untrusted-input surfaces are hardened
(STAGE-006). Packaging an incomplete or unsafe binary would be premature.

## Success Criteria

- A tagged release (`v0.1.0`) produces downloadable binaries for macOS
  (arm64 + x86_64), Linux (x86_64), and Windows, with checksums, on GitHub Releases.
- `brew install jysf/tap/crustyimg` (or similar) installs a working binary.
- `crustyimg --version`/`--help` and a real command work from the installed binary.
- README documents install (brew / cargo / download) + a usage example.
- CI release pipeline is reproducible from a tag (no manual artifact building).

## Scope

### In scope
- `Cargo.toml` publish metadata + license; CHANGELOG + semver + git tags.
- A tag-triggered release workflow (recommend `cargo-dist`) → GitHub Releases.
- Homebrew tap + formula (mirror the bragfile000 approach).
- Optional `cargo publish` to crates.io (after confirming the name is free).
- README install/usage polish; shell completions (+ optional man page).
- **Dual release artifacts (decide here):** the default full binary ships with
  `view` (display on by default, DEC-027) for desktop users; ALSO publish a **lean
  / headless artifact** built `--no-default-features` (no viuer tree, smaller, for
  CI/servers — the "CI tool that doesn't need view"). The `--no-default-features`
  build path + its CI `lean` job already exist (DEC-027), so this is a packaging
  choice, not new code. Decide artifact names (e.g. `crustyimg` vs
  `crustyimg-headless`) + which channels get which.

### Explicitly out of scope
- New image operations/commands; security hardening (STAGE-006).
- Marketing/publicity (dev.to, Product Hunt) — a post-release activity.

## Spec Backlog

Format: `- [status] SPEC-ID (cycle) — one-line summary`

- [x] SPEC-038 (shipped 2026-06-19, PR #42) — Cargo.toml publish metadata (repository/homepage/readme/keywords/categories + `exclude` to drop scaffolding, keep `assets/`) + dual `LICENSE-MIT`/`LICENSE-APACHE` files; verified by `cargo package --list` / `cargo publish --dry-run` (NO publish — crate is now publish-ready). crates.io name `crustyimg` confirmed free. No new dep/DEC.
- [x] SPEC-039 (shipped 2026-06-19, PR #43) — `CHANGELOG.md` (Keep a Changelog; `0.1.0` = the MVP) + `RELEASING.md` (SemVer `0.x` policy, `vX.Y.Z` annotated-tag convention, release-cut checklist with publish/tag/push steps **maintainer-authorized**) + a README pointer. Docs only — no tag/publish/code/dep/DEC.
- [~] SPEC-041 (design 2026-06-23) — release CI pipeline (cargo-dist `dist` 0.32.0): `dist-workspace.toml` (4 targets: macOS arm64/x86_64, Linux x86_64, Windows; shell+powershell installers) + generated tag-triggered `release.yml` (PR=non-publishing plan; `v*` tag=publish; GitHub-Releases-only, no crates.io/tap) + `[profile.dist]` + declared MSRV (`rust-version` + `msrv` CI job). DEC-040. **Design + dry-run only (arms the pipeline; cuts no release).**
- [ ] (not yet written) — Homebrew tap + formula (jysf/homebrew-tap), install-from-tap verified
- [ ] (not yet written) — crates.io publish (`cargo publish`), optional, after name check
- [x] SPEC-040 (shipped 2026-06-19, PR #44) — README install/usage rewrite (tool-first: cargo/release/brew install honestly labeled + works-today `cargo install --git` path, usage quickstart, License corrected to `MIT OR Apache-2.0`, dev-process relocated below) + clap-generated shell completions via a `completions <shell>` subcommand for bash/zsh/fish/powershell/elvish (`clap_complete =4.6.5`, DEC-039; stdout only; works in lean build). Man page (`clap_mangen`) deferred. Safe — no tag/publish/tap.
- [ ] (not yet written) — dual artifacts: publish a lean `--no-default-features` (headless, no-`view`) build alongside the default full binary; pick artifact names + channels (DEC-027 made this a packaging choice, not new code). Future option: actually fork `view` into a separate `crustyimg-view` crate/bin (a later PROJ — only if a real headless/desktop split emerges).

**Count:** 3 shipped / 1 active / 3 pending  (#1 publish-ready crate, #2 changelog/release docs, #6 README/completions [SPEC-040] shipped. #3 release pipeline + MSRV [SPEC-041, design done] active — design + dry-run is SAFE; the actual tag/release cut is the maintainer-authorized trigger and is NOT done here. Remaining pending — all OUTWARD-FACING, need maintainer authorization: #4 Homebrew tap, #5 cargo publish, #7 dual artifacts.)

## Design Notes

- `cargo-dist` is the Rust analog to goreleaser (used for the Go bragfile);
  it generates the release workflow, GitHub Release artifacts, a shell
  installer, and the Homebrew formula from one config — strongly preferred
  over hand-rolling each.
- License is Apache-2.0 (see README/LICENSE); pin it in Cargo.toml.
- This stage interacts with branch protection / CI — the tag-release workflow
  is separate from the PR-CI matrix (DEC-009).

## Dependencies

### Depends on
- STAGE-002..005 (a feature-complete MVP) and STAGE-006 (hardening) — release
  is the last gate.

### Enables
- Public availability; the foundation for PROJ-002+ feature waves to ship as
  point releases.

## Stage-Level Reflection

*Filled in when status moves to shipped. Run Prompt 1c (Stage Ship) in
FIRST_SESSION_PROMPTS.md to draft this.*

- **Did we deliver the outcome in "What This Stage Is"?** <yes/no + notes>
- **How many specs did it actually take?** <number vs. plan>
- **What changed between starting and shipping?** <one sentence>
- **Lessons that should update AGENTS.md, templates, or constraints?**
  - <one-line updates>
- **Should any spec-level reflections be promoted to stage-level lessons?**
  - <one-line items>
