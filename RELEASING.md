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
bare/lightweight tags.

Pushing a version tag is the one decision to publish (DEC-111). After every target has built
and the GitHub Release exists, the same workflow run publishes, in parallel:

- the Homebrew formula to **`jysf/homebrew-tap`** (`publish-homebrew-formula`;
  `brew install jysf/tap/crustyimg`);
- **crates.io** (`custom-publish-crates`, `cargo publish --locked`);
- **npm** `crustyimg-wasm` (`custom-publish-npm`, `just wasm-npm-publish` by trusted publishing).

If any build fails, nothing publishes. A prerelease tag (`v0.9.0-rc.1`) creates a GitHub
prerelease and publishes nowhere else.

## One-time setup (done — kept for reference) — **[MAINTAINER-AUTHORIZED]**

These were the prerequisites for the Homebrew + crates.io channels. They were
completed before `v0.1.0` (2026-07-04); crustyimg has published to crates.io and
pushed a Homebrew formula on every tag since (currently 0.4.0). Kept here so a
fork or a credential rotation has the checklist.

1. **Create the Homebrew tap repo** — a public GitHub repo **`jysf/homebrew-tap`**
   (the cargo-dist job creates the `Formula/` dir + commits `crustyimg.rb` on release).
2. **Add repo secrets** to `jysf/crustyimg` (Settings → Secrets and variables → Actions):
   - **`CARGO_REGISTRY_TOKEN`** — a crates.io API token (crates.io → Account Settings →
     API Tokens), used by the `custom-publish-crates` job.
   - **`HOMEBREW_TAP_TOKEN`** — a GitHub PAT with **write access to `jysf/homebrew-tap`**
     (a fine-grained PAT scoped to that repo is ideal), used by the cargo-dist homebrew job.
3. **npm trusted publisher**: on npmjs.com, open `crustyimg-wasm` → **Settings** → **Trusted
   publisher** → **GitHub Actions**, then enter user **`jysf`**, repository **`crustyimg`** and
   workflow filename **`release.yml`**. Name `release.yml`, the workflow the tag starts, **not**
   `publish-npm.yml`: npm checks the *calling* workflow. Leave the environment empty.
4. **After the first tag publishes npm from CI**, and only then: `crustyimg-wasm` →
   **Settings** → **Publishing access** → "Require two-factor authentication and disallow
   tokens". Until that first green publish, the hand fallback below must keep working.

A crates.io publish is **irreversible** — a given version can never be re-published. The
`cargo publish --dry-run` + full gate suite below are the guard; run them before tagging.

## Release-cut checklist

> **Shortcut:** `just release X.Y.Z` does the mechanical, error-prone parts of steps
> 1, 3 & 4 — it bumps `Cargo.toml`, refreshes `Cargo.lock`, and **guards that the tag
> you're about to cut matches the crate version** (the exact check that would have
> caught the v0.1.1 mis-tag) and that the CHANGELOG has a section for it. It does NOT
> commit, tag, or push. You still do step 2 (the CHANGELOG markdown roll) by hand.

Work through this list in order. Steps marked **[MAINTAINER-AUTHORIZED]** are
outward-facing actions that require explicit human authorization — do not automate
or delegate them.

1. **Update `Cargo.toml`** — bump `version` to the new `MAJOR.MINOR.PATCH`. (Or run
   `just release X.Y.Z`, which also refreshes `Cargo.lock` and runs the guards below.)

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

   > **Before rolling, diff the CHANGELOG against what merged since the previous
   > tag.** List the code-bearing commits (`git log --oneline vPREV..HEAD --no-merges
   > -- src/ Cargo.toml`) and check that every user-visible change among them has an
   > entry in `[Unreleased]`. Changes are not always written up when they merge: 0.7.0
   > shipped with one missing, and 0.8.0's prep found four with no entry.

2a. **Update the agent skill** — [`skills/crustyimg/SKILL.md`](skills/crustyimg/SKILL.md)
   teaches AI coding agents how to use the CLI, and it drifts silently. Walk the
   CHANGELOG section you just rolled: for every added, renamed, or removed command or
   flag, changed default, or changed exit code, update the skill to match, and bump the
   "Written against crustyimg X.Y" line on a minor release. Remove a caveat once it's
   fixed (e.g. the WebP-in-a-glob note, removed for 0.8.0). If the skill is listed on any external
   marketplace, re-upload it there after the tag ships.

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

5. **Commit the release** — stage `Cargo.toml`, `Cargo.lock`, `CHANGELOG.md`, and the skill if it changed;
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
   This single push fires all four channels, in this order: the builds (four targets) → `host`
   creates the **GitHub Release** → **Homebrew**, **crates.io** and **npm** in parallel →
   `announce`. Watch it on the Actions page (workflow **Release**). Confirm the one-time setup
   above first.

8. **Verify the channels.** When the Release run is green: the GitHub Release has its artifacts
   and checksums; `cargo search crustyimg` and `npm view crustyimg-wasm version` both show the
   new version; `brew install jysf/tap/crustyimg` installs it.

   **If one publish job fails:** open the run and click **Re-run failed jobs**. Each publish job
   first asks its registry whether the version is already there. If it is, the job prints
   `SKIP: … already on …` and passes. If the registry could not be asked, the job fails with
   `COULD NOT ASK …` and publishes nothing; re-run it later. A failed *build* publishes nothing at
   all. Re-run it the same way.

   **Hand fallback (recovery only, [MAINTAINER-AUTHORIZED], irreversible).** Use it only when a
   re-run cannot succeed, for example a revoked token or a broken npm trusted-publisher setting.
   From a clean checkout of the tag (`git worktree add ../crustyimg-vX.Y.Z vX.Y.Z`), run
   `scripts/release-publish-guard.sh npm vX.Y.Z`. If it prints `action=publish`, run `npm login`
   and then `just wasm-npm-publish`. For crates.io, run `scripts/release-publish-guard.sh crates
   vX.Y.Z`, then `cargo publish --locked`. Once "disallow tokens" is on, the npm hand path needs a
   token-free login; fix the trusted-publisher setting and re-run the job instead.

### The first tag after the publish jobs moved into the Release workflow (one time)

The pull request that moved npm and crates.io behind `host` proved the four builds and both
dry runs. Only a real tag proves the npm OIDC exchange and the two real publishes. For that tag:

- [ ] Before tagging: one-time setup item 3 is done (the npm trusted publisher names
      **`release.yml`**). Item 4 ("disallow tokens") is **not** done yet.
- [ ] Before tagging: `npm whoami` works on your machine, so the hand fallback is ready.
- [ ] After pushing the tag, in the Release run: four `build-local-artifacts` jobs green
      (Apple Silicon on `macos-15`), then `host`, then `custom-publish-crates / guard` and
      `custom-publish-npm / guard` print `PUBLISH: …`, then both `… / publish` jobs green.
- [ ] `npm view crustyimg-wasm@X.Y.Z dist.attestations` shows a provenance attestation (trusted
      publishing adds it).
- [ ] If `custom-publish-npm / publish` fails on authentication: check the trusted-publisher
      workflow filename first (`release.yml`), then **Re-run failed jobs**. Use the hand fallback
      only if that cannot work.
- [ ] Once both registries show the version from CI: turn on one-time setup item 4.

## After the release

- Verify the GitHub Release page was created by the pipeline (the release body is
  auto-generated from `CHANGELOG.md` by cargo-dist), or create it manually if the
  pipeline did not run.
- Update any install instructions or `brew` formula if the version is referenced
  there.
- Open a follow-up commit to start the next development cycle (e.g. bump to the
  next pre-release version if needed).
