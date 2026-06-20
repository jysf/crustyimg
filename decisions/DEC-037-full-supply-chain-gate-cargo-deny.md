---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-037
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
  - .github/workflows/ci.yml
  - deny.toml
  - justfile

tags:
  - security
  - ci
  - supply-chain
  - dependencies
  - advisories
---

# DEC-037: `cargo deny check` (full) is the single supply-chain CI gate

## Decision

Extend the existing license-only `cargo-deny` CI job (DEC-018) to the **full
supply-chain check** — `cargo deny check advisories bans sources licenses` — and
make `just deny` run the same, so the dependency tree is gated on FOUR axes:

- **advisories** — RUSTSEC security advisories (vulnerabilities, unmaintained,
  yanked crates) → fail CI.
- **bans** — banned/duplicate crates policy (warn on multiple versions; deny list
  empty for now).
- **sources** — every dependency must come from crates.io (no unvetted git/registry
  sources).
- **licenses** — the existing permissive-license policy (DEC-018), unchanged.

**`cargo audit` is intentionally NOT added.** `cargo deny check advisories`
reads the **same RUSTSEC advisory database** `cargo-audit` does, so a separate
`cargo audit` job would be redundant maintenance (two advisory configs, two
ignore lists). One tool (`cargo-deny`) covers the dependency-advisory + license +
ban + source gate the stage calls for. The stage backlog item names "cargo audit
/ cargo deny"; this satisfies the *intent* (a supply-chain gate in CI) with the
tool already wired.

A surfaced advisory/ban is resolved by **bumping the dependency** or, if no fix
exists, a **narrowly-scoped, commented `ignore`/exception in `deny.toml`** with a
revisit note — never a blanket disable (the same discipline as the license
`exceptions`).

## Context

STAGE-006 backlog item #4 is "cargo audit / cargo deny wired into CI." CI already
runs `cargo-deny` but only `check licenses` (DEC-018) — so license drift is
gated, but a **known-vulnerable, unmaintained, or yanked** dependency, a
**duplicate/banned** crate, or a **non-crates.io source** would pass unnoticed.
`cargo-deny` already evaluates all four with one config (`deny.toml`) and one
action (`EmbarkStudios/cargo-deny-action@v2`); only the `command` and a few
`deny.toml` sections need adding. Because `cargo-deny`'s `advisories` check is
backed by RUSTSEC, it subsumes `cargo-audit`.

## Alternatives Considered

- **Add a separate `cargo audit` job (literal reading of the backlog)** —
  rejected as redundant: same RUSTSEC DB as `cargo deny check advisories`, double
  the config/ignore-list maintenance, more CI minutes for no extra coverage.
- **Keep license-only + add only advisories** — rejected: bans (duplicate/yanked)
  and sources (crates.io-only) are cheap, valuable supply-chain guards worth
  enabling at the same time, since the tool/config already exist.
- **`cargo deny check` (all, implicit)** — equivalent; we list the four checks
  explicitly in the CI `command` for readability/auditability.

## Consequences

- **Positive:** A vulnerable/unmaintained/yanked dependency, a banned or
  duplicate crate, or a non-crates.io source now fails CI — the supply-chain
  blind spot is closed with one tool. `just deny` mirrors CI exactly, so it is
  reproducible locally.
- **Negative:** A *new* RUSTSEC advisory against a transitive dep can turn CI red
  with no code change on our side (the nature of an advisory gate); resolved by a
  dep bump or a justified, commented `ignore`. Advisory checks fetch the RUSTSEC
  DB (network) in CI.
- **Neutral:** No new top-level runtime dependency; `cargo-deny` is CI tooling
  (the action), not a crate in the binary. DEC-018's license policy is unchanged.

## Validation

Right if: the CI `cargo-deny` job runs `check advisories bans sources licenses`
and is green, `just deny` runs the same locally, `deny.toml` carries
schema-correct `[advisories]`/`[bans]`/`[sources]` sections, and any finding is
handled by a dep bump or a narrowly-scoped commented exception. Revisit if: a
real need for `cargo-audit`-specific behavior appears, or the advisory noise
warrants a scheduled (non-blocking) advisory job in addition to the blocking gate.

## References

- Related specs: SPEC-036 (this change); SPEC-001 (CI matrix, DEC-009)
- Related decisions: DEC-018 (permissive license policy via cargo-deny — the gate
  this extends), DEC-009 (CI), DEC-034/035/036 (the other STAGE-006 hardening)
- Constraints: `untrusted-input-hardening` (supply chain is part of the threat
  surface), `no-agpl-default-deps` (licenses check), `pure-rust-codecs-default`
- External docs: https://embarkstudios.github.io/cargo-deny/ , https://rustsec.org/
