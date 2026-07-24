# Contributing to crustyimg

Thanks for your interest in improving crustyimg. This guide covers how to build
and test the project, the commit and PR conventions we follow, and the one legal
requirement for contributions: a DCO sign-off.

## Ways to contribute

- **Report a bug** or request a feature via GitHub Issues — include the command
  you ran, the input, and what you expected vs. what happened.
- **Improve the docs** — `README.md`, `docs/cli-reference.md`, and `docs/development.md`.
- **Send a code change** via a pull request (read the conventions below first).

If you're planning a larger change, open an issue to discuss it before you start —
crustyimg is built with a spec-driven workflow (see [AGENTS.md](AGENTS.md)), and a
quick chat saves you from building against the wrong grain.

## Building and testing

crustyimg is a pure-Rust CLI with no system dependencies. You need a recent stable
Rust toolchain (see `rust-version` in `Cargo.toml` for the minimum supported version).

```sh
# Build
cargo build                 # debug build
cargo build --release       # optimized binary at target/release/crustyimg

# Run the CLI locally
cargo run -- --help
cargo run -- info path/to/image.jpg

# Test
cargo test                  # all tests (unit + integration)
cargo test <name>           # a single test or module

# Lint and format — both must pass before you open a PR
cargo clippy --all-targets -- -D warnings   # warnings are errors
cargo fmt --check                            # run `cargo fmt` to fix
```

Please make sure `cargo test`, `cargo clippy --all-targets -- -D warnings`, and
`cargo fmt --check` are all green locally before pushing — these are the same gates
CI enforces.

`webp-lossy` (lossy WebP encode) is an opt-in compile-time feature; `avif` (AVIF
encode) and the terminal `view` command are both on by default and can be turned
off with `--no-default-features` for a lean/headless build. If your change
touches those paths, build and test the relevant feature combination too. See the
[README](README.md#feature-notes) for the feature matrix.

## Commit and PR conventions

crustyimg follows a few conventions from [AGENTS.md §13](AGENTS.md#13-git-and-pr-conventions);
the essentials for outside contributors:

- **Conventional Commits.** Format your commit subjects as
  `<type>(<scope>): <summary>` — types are `feat`, `fix`, `docs`, `refactor`,
  `test`, `chore`, `perf`, `ci`. Use imperative mood, a lowercase summary, and no
  trailing period (e.g. `fix(resize): clamp long-edge to source dimensions`).
- **One logical change per PR.** Keep pull requests focused and reviewable; split
  unrelated changes into separate PRs.
- **Branch off `main`** with a descriptive name (`fix/<slug>`, `docs/<slug>`,
  `feat/<slug>`, `ci/<slug>`).
- **Describe your change** in the PR body: what it does, why, and how you tested it.
- **Keep the tree green** — new behavior needs tests, and all the gates above must pass.

## Developer Certificate of Origin (DCO) — sign-off required

crustyimg is licensed **MIT OR Apache-2.0**, and contributions are **inbound=outbound**:
by contributing, you license your contribution to the project under those same terms.
We do not ask you to sign a CLA or assign copyright — you keep the copyright to your work.

To make that explicit and auditable, every commit must carry a **`Signed-off-by:`**
line certifying the [Developer Certificate of Origin](https://developercertificate.org)
(DCO) — a short statement that you wrote the code (or otherwise have the right to
submit it) and that you're contributing it under the project's license.

Add the sign-off automatically with the `-s` flag:

```sh
git commit -s -m "fix(resize): clamp long-edge to source dimensions"
```

This appends a trailer like:

```
Signed-off-by: Your Name <you@example.com>
```

Use your real name and an email you can be reached at. The DCO check on each pull
request enforces that every commit is signed off; if you forget, you can add the
sign-off to existing commits and force-push:

```sh
git commit --amend -s          # for the most recent commit
git rebase --signoff main      # to sign off a range of commits
```

By submitting a pull request with signed-off commits, you certify the DCO for each
of those commits and agree that your contribution is licensed under **MIT OR
Apache-2.0**.

## Code of conduct

Be respectful and constructive. Assume good faith, keep discussion focused on the
work, and help make this a project people want to contribute to.

## Questions

Open an issue or start a discussion on GitHub. Thanks for contributing to crustyimg.
