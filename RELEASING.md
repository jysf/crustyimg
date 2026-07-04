# Releasing crustyimg

## Versioning policy

crustyimg follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

While the version is `0.x`:
- **Patch bumps** (`0.x.Y → 0.x.Y+1`) are bug fixes only — no new commands,
  no changed flags, no changed exit codes.
- **Minor bumps** (`0.x.Y → 0.x+1.0`) may carry **breaking CLI changes** (renamed
  flags, removed subcommands, changed exit-code semantics). Treat each `0.x` minor
  release as potentially breaking.
- **`1.0.0`** is the first stability commitment: from that point, minor bumps add
  features without breaking changes and only major bumps break the CLI surface.

## Git tags

Releases are marked with **annotated** git tags named `vMAJOR.MINOR.PATCH`
(e.g. `v0.1.0`). The tag version **must equal** the `version` field in `Cargo.toml`.
The release pipeline (cargo-dist, `dist-workspace.toml`) triggers on these tags
to build cross-platform binaries, checksummed archives, and shell/powershell
installers, and creates a GitHub Release automatically; do not create
bare/lightweight tags. Pushing a version tag ALSO (SPEC-042):

- pushes an updated Homebrew formula to the **`jysf/homebrew-tap`** tap
  (`brew install jysf/tap/crustyimg`), via the cargo-dist `publish-homebrew-formula`
  job; and
- **publishes to crates.io** (`cargo publish --locked`) via the separate
  `.github/workflows/publish-crates.yml` workflow.

## One-time setup (before the first tagged release) — **[MAINTAINER-AUTHORIZED]**

These are prerequisites for the Homebrew + crates.io channels. Do them once, before
cutting `v0.1.0`; the tag push will fail those jobs if they are missing.

1. **Create the Homebrew tap repo** — a public GitHub repo **`jysf/homebrew-tap`**
   (the cargo-dist job creates the `Formula/` dir + commits `crustyimg.rb` on release).
2. **Add repo secrets** to `jysf/crustyimg` (Settings → Secrets and variables → Actions):
   - **`CARGO_REGISTRY_TOKEN`** — a crates.io API token (crates.io → Account Settings →
     API Tokens), used by `publish-crates.yml`.
   - **`HOMEBREW_TAP_TOKEN`** — a GitHub PAT with **write access to `jysf/homebrew-tap`**
     (a fine-grained PAT scoped to that repo is ideal), used by the cargo-dist homebrew job.

A crates.io publish is **irreversible** — a given version can never be re-published. The
`cargo publish --dry-run` + full gate suite below are the guard; run them before tagging.

## Release-cut checklist

Work through this list in order. Steps marked **[MAINTAINER-AUTHORIZED]** are
outward-facing actions that require explicit human authorization — do not automate
or delegate them.

1. **Update `Cargo.toml`** — bump `version` to the new `MAJOR.MINOR.PATCH`.

2. **Update `CHANGELOG.md`** — move the contents of `## [Unreleased]` into a new
   dated version section:
   ```
   ## [X.Y.Z] - YYYY-MM-DD
   ```
   Leave an empty `## [Unreleased]` section above it. Update the link-reference
   definitions at the bottom:
   ```
   [Unreleased]: https://github.com/jysf/crustyimg/compare/vX.Y.Z...HEAD
   [X.Y.Z]: https://github.com/jysf/crustyimg/releases/tag/vX.Y.Z
   ```

3. **Dry-run publish check** — verify the crate is publish-ready:
   ```
   cargo publish --dry-run
   ```
   Fix any manifest or metadata warnings before proceeding.

4. **Run the full gate suite** — all must be green:
   ```
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   cargo build --no-default-features   # lean build
   cargo deny check                    # supply-chain gate
   ```

5. **Commit the release** — stage `Cargo.toml`, `Cargo.lock`, and `CHANGELOG.md`;
   commit with a message like:
   ```
   chore(release): v0.2.0
   ```

6. **Create the annotated tag** — **[MAINTAINER-AUTHORIZED]**
   ```
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   ```
   The tag message may include a brief summary of the highlights.

7. **Push the tag** — **[MAINTAINER-AUTHORIZED]**
   ```
   git push origin vX.Y.Z
   ```
   This single push fires all three channels (SPEC-042): the cargo-dist pipeline
   builds cross-platform binaries + checksums and creates the **GitHub Release**;
   the `publish-homebrew-formula` job pushes the formula to **`jysf/homebrew-tap`**;
   and `publish-crates.yml` **publishes to crates.io**. Confirm the one-time setup
   (tap repo + both secrets, above) is done first.

8. **Verify the channels** — after the tag's workflows finish, confirm: the GitHub
   Release exists with artifacts + checksums; `cargo search crustyimg` shows the new
   version on crates.io; and `brew install jysf/tap/crustyimg` installs it. If the
   crates.io job ever needs to be run by hand (e.g. a token issue), the manual
   fallback is `cargo publish --locked` — **[MAINTAINER-AUTHORIZED]** (irreversible).

## After the release

- Verify the GitHub Release page was created by the pipeline (the release body is
  auto-generated from `CHANGELOG.md` by cargo-dist), or create it manually if the
  pipeline did not run.
- Update any install instructions or `brew` formula if the version is referenced
  there.
- Open a follow-up commit to start the next development cycle (e.g. bump to the
  next pre-release version if needed).
