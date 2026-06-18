---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-011
  type: decision
  confidence: 0.8                    # honest: viuer is clearly the right crate
                                     # for terminal display, and feature-gating
                                     # neutralizes its transitive-dep cost. The
                                     # residual risk is viuer's heavier
                                     # dependency tree (sixel/kitty/iterm
                                     # backends) and version drift (0.9 was
                                     # pre-named; 0.11 is current) — gating, not
                                     # the choice of crate, is the soft spot.
  audience:
    - developer
    - agent
    - operator

agent:
  id: claude-opus-4-8
  session_id: null

# Decisions are repo-level, but it's useful to track which project
# caused them to be emitted.
project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-14
supersedes: null
superseded_by: DEC-027   # display flipped to ON by default (2026-06-18)

# Path globs this decision governs.
affected_scope:
  - src/sink/**
  - Cargo.toml

tags:
  - dependencies
  - sink
  - terminal-display
  - feature-gating
---

# DEC-011: `viuer` for the terminal-display sink, behind an off-by-default `display` feature

## Decision

The terminal-display sink (SPEC-005) uses **`viuer`** (pure-Rust terminal
image rendering; MIT) as its rendering backend, added as an **optional**
top-level dependency behind an **off-by-default cargo feature `display`**
(`viuer = { version = "=X.Y.Z", optional = true }`, `[features] display =
["dep:viuer"]`). The `viuer` *render call* is `#[cfg(feature = "display")]`;
the `Sink::Display` variant and its **non-tty refusal** (`SinkError::NotATty`,
detected with `std::io::IsTerminal`) are compiled **unconditionally** so the
default build still type-checks the display path and the refusal test always
runs. Pin the **exact latest** `viuer` version at build (architecture.md
pre-named `0.9`; the current release is `0.11.x` — pin the real latest and
record it in AGENTS.md §5 if it differs).

## Context

`docs/architecture.md` (§ Components -> Sink, § Crate Choices) and AGENTS.md §5
pre-name `viuer` as the terminal-display crate, but pre-naming is not a decision
record. The constraint `no-new-top-level-deps-without-decision` requires a DEC
before any new top-level crate lands in `Cargo.toml`; SPEC-005 is the spec that
actually adds `viuer`, so the DEC belongs here.

Two real concerns drove the feature-gating call:

1. **Transitive dependency weight.** `viuer` pulls a non-trivial tree to
   support multiple terminal graphics protocols (sixel, kitty, iterm2) and
   terminal sizing. It depends on `image` (the SAME pixel crate we already use,
   DEC-002 — so it does **not** introduce a second image-processing library;
   the single-pixel-library invariant is preserved). However, the broader tree
   is heavier than the rest of the default set, and some terminal-graphics
   backends can drag in platform-specific code. We do not want that cost on the
   default `cargo build` / `cargo test` that the multi-OS CI runs on every push
   (DEC-004's portability spirit: keep the default build trivial and
   system-dep-free).
2. **Display is genuinely optional to the core value.** The thesis ("tune once,
   replay across many") is about the file/dir/stdout output paths and the
   pipeline. Terminal preview is a convenience (the `view` command, STAGE-002).
   Gating it keeps headless/CI/server builds lean and reserves the heavier tree
   for users who actually want in-terminal rendering.

Confirmed: `viuer` reuses `image`, so no second pixel library is added (the
`single-image-library` constraint holds). The transitive tree is acceptable
**because** it is feature-gated — if it were on by default it would not be.

## Alternatives Considered

- **Option A: `viuer` as a normal (always-on) dependency**
  - What it is: add `viuer` to `[dependencies]` unconditionally, as
    architecture.md's table literally lists.
  - Why rejected: puts viuer's heavier transitive tree (multiple
    terminal-graphics backends) on every default build and the three-OS CI for
    a feature most batch/headless invocations never use. Contradicts DEC-004's
    "default build stays trivial" spirit.

- **Option B: hand-roll terminal rendering (ANSI/sixel) ourselves**
  - What it is: emit our own escape sequences for block/sixel output.
  - Why rejected: correct multi-protocol terminal graphics (kitty/iterm/sixel
    detection + encoding) is exactly the kind of fiddly, terminal-specific code
    a maintained crate should own. Reinventing it is high effort, low value.

- **Option C: a different display crate (e.g. `termimage`, `catimg`-style)**
  - Why rejected: `viuer` is the de-facto Rust choice, is already pre-named in
    the architecture, reuses our `image` crate, and supports the modern
    high-fidelity protocols (kitty/iterm/sixel) with graceful block fallback.

- **Option D (chosen): `viuer`, optional, behind an off-by-default `display` feature**
  - What it is: `viuer` as `optional = true`; `[features] display =
    ["dep:viuer"]`; render call `#[cfg(feature = "display")]`; the
    `Sink::Display` variant + `NotATty` refusal always compiled.
  - Why selected: the right crate for the hard part, zero cost on the default
    build/CI, single pixel library preserved, and the public `Sink` API +
    refusal semantics stay stable whether or not the feature is on. CI can add
    one extra job that builds `--features display` to keep it from bit-rotting.

## Consequences

- **Positive:** Default `cargo build`/`cargo test` and the three-OS CI stay
  free of viuer's transitive tree. Single pixel library invariant preserved
  (viuer reuses `image`). The display sink is reachable via
  `--features display`. The non-tty refusal is testable without a feature flag.
- **Negative:** Two build configurations to keep green (default + `--features
  display`); CI should add a `--features display` build job (a STAGE-001/006
  follow-up, not blocking SPEC-005). Users who want terminal preview must build
  with the feature (documented). A `view` command (STAGE-002) will need the
  feature on to actually render — STAGE-002 must account for that.
- **Neutral:** Architecture.md pre-named `viuer 0.9`; the current release is
  `0.11.x`. Pinning the real latest is fine; update AGENTS.md §5 / the crate
  table if the pinned version differs from the pre-named `0.9`.

## Validation

Right if: default `cargo build`/`cargo test`/`clippy`/`fmt` stay green on all
three OSes with NO viuer in the dependency graph, `cargo build --features
display` compiles viuer in, and `Sink::Display.write(...)` returns
`SinkError::NotATty` under `cargo test` regardless of feature. Revisit if:
viuer's transitive tree proves to break a default-feature build (it should not,
being optional), if a CI `--features display` job reveals platform-specific
breakage, or if a future maintainer wants display on by default (then promote
it and drop the gate, recording a superseding DEC).

## References

- Related specs: SPEC-005 (Sink — adds viuer here), STAGE-002 (`view` command —
  consumes the display sink; needs `--features display`)
- Related decisions: DEC-002 (single pixel library — viuer reuses `image`, so
  no second image lib), DEC-004 (pure-Rust/trivial-default-build spirit; the
  feature gate keeps the default lean), DEC-007 (typed `SinkError::NotATty`/
  `Display(_)`), DEC-010 (precedent: a one-crate add with a written DEC)
- Constraints: `no-new-top-level-deps-without-decision`, `single-image-library`,
  `untrusted-input-hardening` (sink hardening lives alongside in SPEC-005)
- External docs: https://docs.rs/viuer , https://crates.io/crates/viuer
- Architecture: `docs/architecture.md` § "Crate Choices" (pre-names viuer 0.9),
  § Components -> Sink
