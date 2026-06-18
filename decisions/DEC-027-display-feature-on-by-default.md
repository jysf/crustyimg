---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-027
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent
    - operator

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-18
supersedes: DEC-011
superseded_by: null

affected_scope:
  - Cargo.toml
  - .github/workflows/ci.yml
  - src/sink/**

tags:
  - dependencies
  - terminal-display
  - feature-gating
  - ergonomics
  - release
---

# DEC-027: the `display` feature is ON BY DEFAULT (so `view` works out of the box)

## Decision

The `display` feature (terminal preview via `viuer`) is **a default Cargo feature**
(`[features] default = ["display"]`). The released binary, `cargo install`, and a
plain `cargo build` therefore include `view`'s rendering with no extra flag.
Headless / server / CI builds that don't want viuer's transitive tree opt out with
`--no-default-features`; a dedicated CI **lean** job keeps that path green. This
**supersedes DEC-011**, which made `display` off-by-default.

## Context

DEC-011 gated `display` off-by-default to keep the default `cargo build`/CI lean,
treating terminal preview as optional to the "tune once, replay many" thesis. In
practice that made `view` — the project's **headline first feature** (AGENTS.md
purpose: "View images in the terminal …") — fail out of the box on any default
build (`error: terminal display failed: built without the display feature`), which
is a poor first impression for a released tool a user installs to, among other
things, look at images.

The cost DEC-011 was avoiding was measured: enabling `display` adds **~18 pure-Rust
crates** (101 → 129 dependency edges: viuer + crossterm, rustix, libc, parking_lot,
base64, console, …). The maintainer's call (2026-06-18): that cost is **not** worth
gating a headline feature behind for the shipped product — `view` must be present by
default in the final release. Verified before flipping: viuer 0.11 is MIT and
**reuses our `image` crate** (no second pixel library — `single-image-library`
holds), adds **no system/C dependencies** (all ~18 crates build cleanly on the
three-OS matrix), and `cargo deny check licenses` stays green with `display` in the
default scan.

## Alternatives Considered

- **Option A: keep DEC-011 (off by default); document the `--features display` build.**
  - Why rejected: the released/installed binary's `view` keeps failing out of the
    box; pushes the project's headline command behind a flag most users won't know.

- **Option B: keep the default lean, but make only the RELEASE artifact +
  `cargo install` default to `--features display`.**
  - What it is: decouple dev build from shipped build (cargo-dist + install recipe).
  - Why not chosen: more moving parts (release-config-only feature enabling), and a
    plain `cargo build` from source still surprises with a broken `view`. The
    maintainer wanted it simply present by default everywhere; the ~18-crate dev-build
    cost is acceptable.

- **Option C (chosen): make `display` a default feature; add a `--no-default-features`
  CI lean job.**
  - Why selected: `view` works out of the box for built, installed, and released
    binaries with zero extra knowledge; the lean/headless build is still a first-class,
    CI-guarded path for users who want it; single pixel library + license cleanliness
    preserved; simplest mental model ("it's just there").

## Consequences

- **Positive:** `view` works on a plain `cargo build` / `cargo install` / the release
  artifact — no flag, no surprise. The headline command matches the project's stated
  purpose. The error path now tells `--no-default-features` users exactly how to
  re-enable it.
- **Negative:** the default `cargo build`/`cargo test` and the three-OS matrix now
  compile viuer's ~18-crate tree (slightly slower default CI; larger default binary).
  Accepted deliberately. A new **lean** CI job must stay green to prevent the
  headless build from bit-rotting (the mirror of the guarantee DEC-011 gave the
  default build).
- **Neutral:** `avif` and `webp-lossy` remain off-by-default (DEC-020/022) — this
  decision is only about `display`. The `Sink::Display` variant + `NotATty` refusal
  were already always-compiled (DEC-011), so the public API is unchanged.

## Validation

- Right if: a freshly built/installed binary renders `view <img>` in a graphics
  terminal with no extra flags, the three-OS default CI stays green with viuer
  compiled in, and the `--no-default-features` lean job stays green. Revisit if: the
  added tree breaks a default build on some OS (then narrow what `display` pulls in,
  or revert to release-only enabling per Option B), or if binary size/build time
  becomes a real constraint for a target environment.

## References

- Supersedes: DEC-011 (viuer behind an off-by-default `display` feature)
- Related specs: SPEC-005 (Sink — added viuer), STAGE-002 (`view` command),
  STAGE-007 (release & distribution — the released artifact now carries display)
- Related decisions: DEC-002 (single pixel library — viuer reuses `image`), DEC-004
  (pure-Rust default; `avif`/`webp-lossy` stay gated), DEC-018 (license gate — green
  with viuer in the default scan)
- Measurement (2026-06-18): `display` adds ~18 crates (101 → 129 dep edges); viuer
  0.11 MIT; `cargo deny check licenses` green.
