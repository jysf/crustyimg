# SPEC-042 build prompt — Homebrew tap + crates.io publish channels (config only, no release)

Start a **fresh session**. You are the IMPLEMENTER for SPEC-042 in the `crustyimg` repo
(cwd is the repo root). You are adding two release channels to the SPEC-041 pipeline:
**Homebrew** (via cargo-dist) and **crates.io** (a separate tag-triggered `cargo
publish` workflow). This is **CONFIG ONLY — it arms the channels but fires nothing.**
❌ Do NOT run `git tag`, `git push --tags`, `gh release`, `cargo publish`, any `dist
build` upload; ❌ do NOT create the `jysf/homebrew-tap` repo; ❌ do NOT add any secret.
Open a PR and STOP. Follow this prompt exactly.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-042-release-channels-homebrew-tap-and-crates-io-publish.md`
   — the whole spec: `## Acceptance Criteria`, `## Failing Tests`, `## Notes`.
2. `decisions/DEC-041-release-channels-homebrew-and-crates-io.md` — authoritative config +
   the probe-verified mechanism split (Homebrew = native dist job; crates.io = separate
   workflow) + the safety model.
3. `dist-workspace.toml` + `.github/workflows/release.yml` (SPEC-041) — what you extend;
   `RELEASING.md`; `Cargo.toml` (no change).

## What to build

### A. Homebrew (cargo-dist native)
1. Confirm `dist --version` == `cargo-dist 0.32.0` (installed at `~/.cargo/bin/dist`;
   `export PATH="$HOME/.cargo/bin:$PATH"`).
2. Edit `dist-workspace.toml`: set
   `installers = ["shell", "powershell", "homebrew"]`, and add
   `tap = "jysf/homebrew-tap"` and `publish-jobs = ["homebrew"]`. Leave targets,
   `cargo-dist-version`, `install-path` unchanged.
3. `dist generate` (regenerates `release.yml`), then `dist generate --check` (must be
   in sync — no diff). Do NOT hand-edit `release.yml`.
4. `dist plan` — must exit 0 and now list the Homebrew formula alongside the
   shell/powershell installers + the 4 target archives. Paste into Build Completion.

### B. crates.io (separate tag-only workflow)
5. Create `.github/workflows/publish-crates.yml` — minimal, **tag-only**:
   ```yaml
   name: Publish to crates.io
   on:
     push:
       tags: ['**[0-9]+.[0-9]+.[0-9]+*']   # SAME glob release.yml uses — verify it
   jobs:
     publish:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4
         - uses: dtolnay/rust-toolchain@stable
         - run: cargo publish --locked
           env:
             CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
   ```
   **Check `release.yml`'s actual tag glob and match it exactly.** NO `pull_request:`,
   NO `branches:` — it must ONLY ever run on a version tag. Token comes from
   `secrets.CARGO_REGISTRY_TOKEN` — never hard-code it (`no-secrets-in-code`).

### C. Docs
6. `RELEASING.md` — add the one-time prerequisites (create the `jysf/homebrew-tap`
   repo; add repo secrets `HOMEBREW_TAP_TOKEN` and `CARGO_REGISTRY_TOKEN`) and note the
   tag push now also publishes to Homebrew + crates.io. **Keep every
   `[MAINTAINER-AUTHORIZED]` marker.** Small wording addition, not a rewrite.

## Safety — verify it yourself after `dist generate`
- `release.yml` still has `publishing: ${{ !github.event.pull_request }}` (PR = plan).
- `release.yml` `push:` is still tag-filtered; the new `publish-homebrew-formula` job
  references `repository: "jysf/homebrew-tap"` and `secrets.HOMEBREW_TAP_TOKEN`.
- `publish-crates.yml` has NO `pull_request`/`branches` trigger — tag only.

## Hard rules
- **No outward-facing action.** No `git tag`/push, no `gh release`, no `cargo publish`,
  no `dist build` upload, no tap-repo creation, no secret creation.
- **No secrets in code** — only `secrets.*` references.
- **No `src/`/dependency change**, no `#7` lean artifact (deferred), no `release-plz`.
- DEC-041 is already authored — do NOT create a new DEC. `release.yml` is
  machine-generated — change config + `dist generate`, never hand-edit.

## Gates (all must pass)
```
dist --version                               # cargo-dist 0.32.0
dist generate --check                        # release.yml in sync
dist plan                                    # lists Homebrew formula + installers + 4 archives
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --no-default-features
cargo deny check advisories bans sources licenses
git tag                                      # MUST show NO new tag
grep -n "publish-homebrew-formula" .github/workflows/release.yml   # present
grep -nE "pull_request|branches:" .github/workflows/publish-crates.yml   # MUST be empty (tag-only)
grep -c "CARGO_REGISTRY_TOKEN" .github/workflows/publish-crates.yml      # via secrets, not hard-coded
```

## Git / PR
- Branch `feat/spec-042-release-channels` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked `reports/*.md`
  and `TESTING-WITH-YOUR-PHOTOS.md`.
- If `cargo fmt` reformats committed files later, re-add all touched files (`git add -u`).
- PR title: `ci(SPEC-042): Homebrew tap + crates.io publish channels`.
- PR body per AGENTS.md §13: Decisions (DEC-041, DEC-040, DEC-037); Constraints
  (no-secrets-in-code; one-spec-per-pr; clippy-fmt-clean); New decisions (none —
  DEC-041 pre-authored). **State plainly: config only — arms Homebrew + crates.io on the
  NEXT `v*` tag; creates no tag/release/publish/tap/secret; #7 deferred.**
- Fill the spec's `## Build Completion` + 3 reflection answers (paste `dist plan`);
  append the build cost session entry (agent `claude-sonnet-4-6`, numerics null).

## Cost
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-07-03
  notes: "ci/release: dist-workspace.toml += homebrew installer + tap jysf/homebrew-tap + publish-jobs=[homebrew] (regenerated release.yml w/ publish-homebrew-formula + HOMEBREW_TAP_TOKEN); new publish-crates.yml (tag-only, cargo publish --locked, secrets.CARGO_REGISTRY_TOKEN); RELEASING.md prereqs. dist plan/generate --check green; fmt/clippy/test/lean/deny green; NO tag/release/publish/tap/secret. Arms #4+#5; #7 deferred."
```

## When done
`just advance-cycle SPEC-042 verify` (if it mis-globs, set `cycle: verify` in the spec
frontmatter by hand), open the PR with `gh`, and **stop** — the orchestrator pauses for
the maintainer before merge, and the tap-repo / secrets / tag cut are separate
maintainer-authorized actions.
