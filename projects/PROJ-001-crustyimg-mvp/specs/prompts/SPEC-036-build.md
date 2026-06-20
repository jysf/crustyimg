# SPEC-036 build prompt — full supply-chain gate (cargo-deny)

Start a **fresh session**. You are the IMPLEMENTER for SPEC-036 in the `crustyimg`
repo (cwd is the repo root). The architect (Opus) wrote the spec and **DEC-037**.
This is a **CI/config chore** — extend the cargo-deny gate; there are NO Rust unit
tests to add. The acceptance check is `cargo deny check advisories bans sources
licenses` exiting 0. Make the change, confirm the gate is clean, open a PR, STOP.
Follow this prompt exactly.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-036-full-supply-chain-gate-cargo-deny-advisories-bans-sources-in-ci.md`
   — especially `## Policy (PINNED)`, `## Failing Tests`, `## Notes for the Implementer`.
2. `decisions/DEC-037` (the gate + cargo-audit consolidation), `DEC-018` (license policy
   — do NOT change `[licenses]`), `DEC-009` (CI).
3. `.github/workflows/ci.yml` (the `licenses` job), `deny.toml` (`[graph]` + `[licenses]`
   only today), `justfile` (the `deny` recipe).

## What to build
- **`deny.toml`** — ADD three sections (keep `[graph]` and `[licenses]` byte-for-byte):
  - `[advisories]` — deny RUSTSEC vulnerabilities + unmaintained; `yanked = "deny"`;
    `ignore = []`.
  - `[bans]` — `multiple-versions = "warn"`; `deny = []`; `skip = []`.
  - `[sources]` — `unknown-registry = "deny"`; `unknown-git = "deny"`;
    `allow-registry = ["https://github.com/rust-lang/crates.io-index"]`.
  **Validate against the real tool** — `cargo-deny`'s schema changes across versions
  (e.g. `[advisories]` `vulnerability`/`unmaintained` keys are now defaults; a section
  may want `version = 2`). Run the check, read any deprecation warning/error, and adjust
  the keys until the tool is happy. The TOOL is the source of truth, not these example keys.
- **`.github/workflows/ci.yml`** — change the cargo-deny step's
  `command: check licenses` → `command: check advisories bans sources licenses`; rename
  the job/step to reflect the broader scope (e.g. `name: supply-chain policy
  (cargo-deny)` / step `cargo-deny (supply-chain: advisories + bans + sources +
  licenses, DEC-037/DEC-018)`). Do NOT add a toolchain step (the action is self-contained).
- **`justfile`** — `deny` recipe → `cargo deny check advisories bans sources licenses`.
- **Do NOT** add a separate `cargo audit` job (redundant — DEC-037).

## Hard rules
- The gate must be **GREEN**: run `cargo deny check advisories bans sources licenses`
  locally and confirm exit 0. If an advisory/ban surfaces:
  - a transitive dep with a fix → `cargo update -p <crate>` to resolve it;
  - NO fix available → add a **commented, dated** `ignore = ["RUSTSEC-XXXX-YYYY"]`
    (advisory) or `[bans] skip` (duplicate) entry with a one-line reason + revisit note.
  - **NEVER** switch a whole check to `"allow"` / delete a section to make CI pass.
- **Do NOT change `[licenses]`** (`allow`/`exceptions`/`confidence-threshold` — DEC-018).
- **No new top-level runtime dependency** (cargo-deny is CI tooling). If `cargo update`
  changes `Cargo.lock`, that is fine and expected; do not edit `Cargo.toml` deps.
- The full Rust gate suite must still pass (the cargo-deny change shouldn't affect it, but
  confirm):
```
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --no-default-features
cargo deny check advisories bans sources licenses
```
- Paste the exact `cargo deny check …` clean output into the spec's `## Build Completion`.

## Git / PR
- Branch `feat/spec-036-supply-chain-gate` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked `reports/*.md`
  and `TESTING-WITH-YOUR-PHOTOS.md` (do NOT stage them).
- PR title: `ci(deny): full supply-chain gate — advisories + bans + sources (SPEC-036)`.
- PR body per AGENTS.md §13 (Decisions referenced — DEC-037, DEC-018, DEC-009 /
  Constraints — `untrusted-input-hardening`, `no-agpl-default-deps` / New decisions —
  "DEC-037 at design").
- Fill the spec's `## Build Completion` + 3 reflection answers; append the build cost
  session (numerics null; agent `claude-sonnet-4-6`).

## Cost
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-19
  notes: "supply-chain gate: ci.yml cargo-deny command -> check advisories bans sources licenses + deny.toml [advisories]/[bans]/[sources] sections + just deny; cargo-audit consolidated into cargo-deny (DEC-037); no new runtime dep; full check green"
```

## When done
`just advance-cycle SPEC-036 verify` (if it mis-globs or doesn't update the spec's
`cycle:` field, set `cycle: verify` in the spec frontmatter by hand), open the PR with
`gh`, confirm the CI supply-chain job is green on the PR, and **stop** — the orchestrator
pauses for the user before any merge.
