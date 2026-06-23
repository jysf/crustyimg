---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-040
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-19
supersedes: null
superseded_by: null

affected_scope:
  - dist-workspace.toml
  - .github/workflows/release.yml
  - Cargo.toml

tags:
  - release
  - ci
  - distribution
  - cargo-dist
  - msrv
---

# DEC-040: `cargo-dist` (`dist`) for the tag-triggered release pipeline + a declared MSRV

## Decision

Adopt **`cargo-dist` (the `dist` binary), pinned to `0.32.0`**, as the release
engineering tool for crustyimg. It is configured by a **`dist-workspace.toml`** at the
repo root and generates a **`.github/workflows/release.yml`** that, on a pushed
`v*` git tag, cross-compiles the binary, bundles checksummed archives + shell/powershell
installers, and creates a **GitHub Release**. Config (probe-verified with `dist plan`):

```toml
# dist-workspace.toml
[workspace]
members = ["cargo:."]

[dist]
cargo-dist-version = "0.32.0"
ci = "github"
installers = ["shell", "powershell"]
targets = [
  "aarch64-apple-darwin",
  "x86_64-apple-darwin",
  "x86_64-unknown-linux-gnu",
  "x86_64-pc-windows-msvc",
]
install-path = "CARGO_HOME"
```

plus a `[profile.dist]` (`inherits = "release"`, `lto = "thin"`) in `Cargo.toml`. This
is the **four-platform set the project brief names** (macOS arm64 + x86_64, Linux
x86_64, Windows); `dist plan` confirmed it emits per-target `.tar.xz`/`.zip` archives,
each with a `.sha256`, each bundling the binary + `LICENSE-MIT` + `LICENSE-APACHE` +
`README.md` + `CHANGELOG.md`, plus a combined `sha256.sum` and `crustyimg-installer.sh`
/ `.ps1`.

We also **declare an MSRV** via `rust-version` in `Cargo.toml` (a `0.x` support floor,
enforced by a dedicated `msrv` CI job that builds on exactly that pinned toolchain) —
the release contract should state the minimum Rust it builds on.

## Context

STAGE-007's success criteria call for "a tagged release (`v0.1.0`) [that] produces
downloadable binaries for macOS (arm64 + x86_64), Linux (x86_64), and Windows, with
checksums, on GitHub Releases" and "CI release pipeline … reproducible from a tag (no
manual artifact building)". The stage Design Notes already nominate `cargo-dist` (the
Rust analog to the Go `goreleaser` used by the earlier bragfile project). This is the
**plumbing for backlog #4 (Homebrew tap), #5 (`cargo publish`), and #7 (dual
artifacts)** — they extend this same pipeline (a `homebrew` installer, a crates.io
`publish-jobs` entry, a second `--no-default-features` artifact).

A **design-time probe** (installed `dist 0.32.0`, ran `dist init` / `dist generate` /
`dist plan` against the real crate, then reverted) verified the config above and the
**safety model** of the generated workflow:

- `release.yml` triggers on `pull_request` (a **plan only** — `publishing: false`, no
  artifacts uploaded, no release created) and on `push:` **filtered to `tags:`** that
  match a version. It does **not** run on an ordinary push to `main`.
- The `gh release create` step is gated behind a real tag push; with `installers` set
  and **no `publish-jobs`**, the default pipeline creates a **GitHub Release only — it
  does NOT publish to crates.io** (that is backlog #5, a separate, maintainer-authorized
  `publish-jobs = ["..."]` addition).

So **merging the spec that adds these files cuts no release** — it only arms the
pipeline. The outward-facing act is the maintainer pushing a `vX.Y.Z` tag (RELEASING.md
step 7, **[MAINTAINER-AUTHORIZED]**).

## Alternatives Considered

- **Hand-rolled GitHub Actions release workflow** (build matrix + `softprops/
  action-gh-release` + manual checksums) — rejected: re-implements what cargo-dist does
  declaratively (cross-compile, archive, checksum, installer scripts, release notes from
  CHANGELOG), and the brief/stage Design Notes already prefer cargo-dist. More
  maintenance, more ways to get checksums/permissions wrong.
- **`cargo-binstall`-only / no installer scripts** — rejected: the brief wants a
  one-line install story; the shell/powershell installers cargo-dist emits give that for
  free and feed the README's install section (SPEC-040).
- **Include the Homebrew installer + crates.io publish now** — rejected for this spec:
  the Homebrew installer needs the not-yet-created tap repo (#4) and crates.io publish
  is the irreversible #5. Both are **maintainer-authorized** and out of scope here; the
  config has explicit slots to add them later.
- **A 5th target `aarch64-unknown-linux-gnu`** (dist's default) — dropped to match the
  brief's named set and avoid Linux-ARM cross-compile flakiness in the first pipeline; a
  trivial future addition.
- **No MSRV declaration / compute the true minimum via `cargo-msrv`** — we declare a
  conservative floor enforced by CI rather than block on an empirical bisect (rustup /
  cargo-msrv are not in the local toolchain); the true minimum can be refined later with
  `cargo msrv find`.

## Consequences

- **Positive:** One tag push → reproducible cross-platform binaries + checksums +
  installers + a GitHub Release, no manual artifact building. The release archives
  automatically include the dual licenses (SPEC-038) and the user README (SPEC-040). The
  config is the extension point for #4/#5/#7. A declared, CI-enforced MSRV gives users a
  support contract.
- **Negative:** `cargo-dist` is external release tooling pinned at `0.32.0` (an upgrade
  is a deliberate `cargo-dist-version` bump + `dist generate`). The generated
  `release.yml` is large and machine-owned — edit config + regenerate, don't hand-edit.
  A `pull_request` plan job adds one (fast, non-publishing) check to PR CI. The MSRV
  value is only verified once the `msrv` CI job runs on the PR.
- **Neutral:** `cargo-dist` is CI/release tooling, not a runtime dependency — it adds
  nothing to the shipped binary's dependency tree and does not affect `cargo deny` (cf.
  DEC-037, the cargo-deny supply-chain gate, which is the precedent for a tooling DEC).
  `[profile.dist]` is inert for normal `cargo build`/`test`.
