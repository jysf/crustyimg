---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-018
  type: decision
  confidence: 0.95
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

created_at: 2026-06-16
supersedes: null
superseded_by: null

affected_scope:
  - Cargo.toml
  - deny.toml
  - .github/workflows/ci.yml

tags:
  - license
  - dependencies
  - ci
  - policy
  - permissive
---

# DEC-018: Permissive-only dependency policy, enforced in CI by cargo-deny

## Decision

crustyimg is distributed under **`MIT OR Apache-2.0`** (unchanged), and **every
dependency must carry a permissive license** — MIT, Apache-2.0 (incl. the
LLVM-exception form), BSD-2/3-Clause, Zlib, 0BSD, Unlicense, or Unicode-3.0. **No
GPL/AGPL dependencies** (strong/network copyleft would relicense the whole
statically-linked binary). **LGPL is allowed only via a documented per-crate
`cargo-deny` exception.** The policy is enforced mechanically by
`cargo deny check licenses` (config in `deny.toml`, run as a CI `licenses` job
and via `just deny`), evaluating the **full feature graph** so a copyleft dep
cannot slip in behind a feature flag.

Today there is exactly **one** exception: **`ansi_colours` (LGPL-3.0-or-later)**,
pulled in transitively by **`viuer`** behind the optional `display` feature
(DEC-011) for RGB→nearest-ANSI-color in the half-block fallback. LGPL is *weak*
copyleft — it does not relicense crustyimg — and it only enters the `display`
build, so it is accepted via a scoped exception. The **planned removal path** is
a thin in-house, fully-permissive `Display` sink (Kitty + iTerm2 emitted
directly + `icy_sixel` MIT/Apache for Sixel + truecolor half-blocks needing no
ANSI-256 quantization), which drops both `viuer` and `ansi_colours` and makes the
"100% permissive" claim literally true.

## Context

A research pass (2026-06-15/16) surfaced several of the *best* pure-Rust image
crates as **AGPL-3.0** (`gifski` — top-quality GIF; `jpegli-rs`/`zenjpeg` —
perceptual JPEG; likely `zenwebp`) or GPL (`imagequant`). Because Cargo
**statically links** dependencies into the binary, taking an AGPL/GPL dep would
force the *entire* crustyimg binary under that copyleft license — destroying the
"widely usable / embeddable in closed-source" property that the permissive
license exists to provide (a stated project goal: "more usable, even widely
usable"). The constraint was being *remembered* rather than *enforced*; a
contributor could wire in `gifski` and silently relicense the project. A license
gate makes the rule mechanical and keeps the published crates.io metadata honest.

An audit of the current tree (`cargo metadata --all-features`) found it permissive
**except** `ansi_colours` (LGPL, via viuer/display) — see the Decision.

## Alternatives Considered

- **Option A: No tooling — rely on review/discipline.**
  - Why rejected: the AGPL crates are attractive and the trap is silent (static
    linking, no compile error). One missed review relicenses the binary. The
    whole point is that this must not depend on remembering.

- **Option B: `cargo-audit` only (security advisories).**
  - Why rejected: `cargo-audit` checks vulnerabilities, not licenses. It's
    complementary (and is the STAGE-006 security item) but does not enforce this
    policy. cargo-deny does licenses + advisories + bans + sources; we wire the
    `licenses` check now and can enable the others at STAGE-006.

- **Option C (chosen): `cargo-deny check licenses` with an SPDX allowlist +
  scoped per-crate exceptions, in CI + a `just` recipe.**
  - Why selected: declarative allowlist, fails the build on any non-permissive
    (or unlisted) license, supports a documented per-crate exception for the one
    known LGPL transitive dep, evaluates the full feature graph, and is the
    de-facto Rust tool for exactly this. Verified locally: the gate passes today
    and *fails* correctly if the `ansi_colours` exception is removed.

## Consequences

- **Positive:** the permissive promise is enforced, not hoped for; crates.io
  metadata stays truthful; a new GPL/AGPL/LGPL dep fails CI with a clear message;
  the policy is documented and auditable. Down-payment on the STAGE-006 hardening
  (cargo-deny also does advisories/bans/sources).
- **Negative:** the *best* GIF/JPEG/PNG-quantization crates (gifski, jpegli-rs,
  imagequant) are off the table as defaults — must use permissive pure-Rust
  alternatives (`image` + `color_quant`, `ravif`, `icy_sixel`, …) or feature-gate
  an AGPL crate behind an explicit opt-in build the user accepts (never default).
  A genuinely permissive license that's simply not yet in the allowlist (e.g.
  ISC) will fail until added — a trivial one-line fix, and the gate working as
  intended.
- **Neutral:** the LGPL `ansi_colours` exception is a knowing, scoped acceptance,
  not a precedent for GPL/AGPL; it has a planned removal path (the in-house
  display sink). The check is OS-independent (license graph), so it runs as a
  single ubuntu job, not the 3-OS matrix.

## Validation

Right if: `cargo deny check licenses` passes on `main`, fails on any added
GPL/AGPL dep (and on LGPL not covered by an exception), and the only exception is
`ansi_colours`. Revisit if: the in-house display sink lands (then delete the
`ansi_colours` exception and viuer drops out), or a real need arises for an AGPL
crate (then design an explicit opt-in feature build + update this DEC, never the
default), or a new permissive license legitimately appears (add it to the
allowlist).

## References

- Related decisions: DEC-004 (pure-Rust codec policy — the codec crates this
  bumps against), DEC-011 (viuer terminal display — the source of the LGPL dep),
  DEC-009 (CI matrix — this adds a license job).
- Related constraints: `no-agpl-default-deps` (the rule this enforces),
  `no-new-top-level-deps-without-decision`.
- Config: `deny.toml`; CI `licenses` job in `.github/workflows/ci.yml`;
  `just deny`.
- Watchlist: capabilities declined *because of* this policy (with permissive
  alternatives to find or build, and revisit triggers) are tracked in
  `/guidance/license-watchlist.yaml` (`just watchlist`) — the "way back" ledger
  that pairs with this gate.
- External docs: https://embarkstudios.github.io/cargo-deny/ ,
  https://spdx.org/licenses/ , https://www.gnu.org/licenses/agpl-3.0.html
- Follow-up: a permissive in-house `Display` sink (Kitty + iTerm2 + `icy_sixel` +
  truecolor half-blocks) to drop viuer + ansi_colours — tracked in `docs/backlog.md`.
</content>
