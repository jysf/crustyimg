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
The future release pipeline (backlog #3, cargo-dist) triggers on these tags to
build and publish release artifacts; do not create bare/lightweight tags.

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
   This push is what the release pipeline (backlog #3) will trigger on to build
   binaries, publish to crates.io, and create the GitHub Release.

8. **Publish to crates.io** — **[MAINTAINER-AUTHORIZED]** (once the pipeline
   exists, this step is automated by it; until then, run manually after the tag is
   pushed and CI is green on the tag commit):
   ```
   cargo publish
   ```

## After the release

- Verify the GitHub Release page was created (once the pipeline exists) or create
  it manually, pasting the relevant `CHANGELOG.md` section as the release body.
- Update any install instructions or `brew` formula if the version is referenced
  there.
- Open a follow-up commit to start the next development cycle (e.g. bump to the
  next pre-release version if needed).
