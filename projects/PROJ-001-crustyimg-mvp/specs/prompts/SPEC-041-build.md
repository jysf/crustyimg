# SPEC-041 build prompt — cargo-dist release pipeline + MSRV (design + dry-run only)

Start a **fresh session**. You are the IMPLEMENTER for SPEC-041 in the `crustyimg`
repo (cwd is the repo root). You are adding the **tag-triggered `cargo-dist` release
pipeline** + a declared **MSRV**. This is **DESIGN + DRY-RUN ONLY — it cuts no
release.** ❌ Do NOT run `git tag`, `git push --tags`, `gh release create`, `cargo
publish`, or any `dist build` upload, and do NOT create a Homebrew tap. Open a PR and
STOP. Follow this prompt exactly.

## Read first (in order)
1. `projects/PROJ-001-crustyimg-mvp/specs/SPEC-041-release-pipeline-cargo-dist-and-msrv.md`
   — the whole spec, especially `## Acceptance Criteria`, `## Failing Tests`, `## Notes`.
2. `decisions/DEC-040-cargo-dist-release-pipeline.md` — **authoritative**: the exact
   `dist-workspace.toml` contents, the 4 targets + installers, GitHub-Releases-only
   scope, and the **safety model** you must verify in the generated workflow.
3. `STAGE-007` stage file (Success Criteria + Design Notes), `RELEASING.md` (the
   checklist that references "the pipeline (backlog #3)"), `.github/workflows/ci.yml`
   (where the `msrv` job goes), `Cargo.toml`.

## What to build

### A. cargo-dist pipeline
1. **Install `dist` 0.32.0** — prebuilt installer
   (`curl --proto '=https' --tlsv1.2 -LsSf https://github.com/axodotdev/cargo-dist/releases/latest/download/cargo-dist-installer.sh | sh`
   then ensure `~/.cargo/bin` is on PATH; confirm `dist --version` == `cargo-dist 0.32.0`)
   or `cargo install cargo-dist --version 0.32.0 --locked`.
2. **Write `dist-workspace.toml`** at the repo root with EXACTLY the DEC-040 config:
   ```toml
   [workspace]
   members = ["cargo:."]

   [dist]
   cargo-dist-version = "0.32.0"
   ci = "github"
   installers = ["shell", "powershell"]
   targets = ["aarch64-apple-darwin", "x86_64-apple-darwin", "x86_64-unknown-linux-gnu", "x86_64-pc-windows-msvc"]
   install-path = "CARGO_HOME"
   ```
   (You may instead run `dist init --yes` and then edit the generated config to match
   this exactly — drop the default `aarch64-unknown-linux-gnu` target and set the two
   installers.)
3. **`dist generate`** — writes `.github/workflows/release.yml` and adds `[profile.dist]`
   to `Cargo.toml`. Then **`dist generate --check`** — must report the workflow is in
   sync with the config (no diff). Do NOT hand-edit `release.yml`.
4. **`dist plan`** (the DRY-RUN) — must exit 0 and list, for `v0.1.0`: the 4 per-target
   archives (unix `.tar.xz`, windows `.zip`), each with a `.sha256`, plus a combined
   `sha256.sum` and `crustyimg-installer.sh` / `.ps1`. **Paste this output into Build
   Completion.** (`dist plan` only prints — it builds/uploads nothing.)
5. **Verify the safety model yourself** in the generated `release.yml`:
   - `publishing: ${{ !github.event.pull_request }}` is present (PR = non-publishing plan).
   - the `push:` trigger is filtered to `tags:` (NOT branch pushes).
   - there is **NO** `cargo publish` / crates.io step and **NO** homebrew installer.
   If `dist` ever emitted a crates.io publish step, remove any `publish-jobs` from the
   config and regenerate — this spec is GitHub-Releases-only.

### B. MSRV
6. Add `rust-version = "1.85.0"` to `[package]` in `Cargo.toml` (a conservative `0.x`
   support floor for the modern pinned deps).
7. Add an **`msrv` job** to `.github/workflows/ci.yml` (same style as the other jobs):
   `dtolnay/rust-toolchain@1.85.0`, then `cargo build` and `cargo build
   --no-default-features`. The job's pinned toolchain MUST equal `rust-version`.
   - **The PR's `msrv` job is the verification** (rustup isn't available locally). If it
     goes red because a dep needs newer Rust, raise BOTH `rust-version` and the job pin
     to the next version that builds, and push again until green. (Optional: if you can
     run `cargo msrv find`, pin the true minimum instead.)

### C. Docs
8. **`RELEASING.md`** — light wording pass so it reflects that the pipeline now exists
   (e.g. "the future release pipeline (backlog #3) will trigger" → "the release pipeline
   triggers on this tag to build artifacts and create the GitHub Release"). **Keep every
   `[MAINTAINER-AUTHORIZED]` marker.** Do not rewrite the checklist.

## Hard rules
- **No outward-facing action.** No `git tag`, no tag push, no `gh release`, no `cargo
  publish`, no `dist build` upload, no tap repo. `dist plan` is the only `dist` command
  that "runs".
- **No secrets.** The release workflow uses the default `GITHUB_TOKEN` only — do NOT add
  a crates.io token or any secret (that's #5).
- **No `src/` change**, no new runtime dependency, no Homebrew installer, no crates.io
  `publish-jobs`, no 5th Linux-ARM target. DEC-040 is already authored — do NOT create a
  new DEC.
- `release.yml` is machine-generated — change config + `dist generate`, never hand-edit.

## Gates (all must pass)
```
dist --version                               # cargo-dist 0.32.0
dist generate --check                        # release.yml in sync with dist-workspace.toml
dist plan                                    # dry-run: 4 targets + checksums + installers for v0.1.0
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --no-default-features            # lean still builds
cargo deny check advisories bans sources licenses
git tag                                      # MUST show NO new tag
grep -c "cargo publish" .github/workflows/release.yml   # MUST be 0
```
(If `cargo fmt`/`build` complain after `dist generate` added `[profile.dist]`, that's a
formatting nit in Cargo.toml — fix formatting only; the profile is correct.)

## Git / PR
- Branch `feat/spec-041-release-pipeline` off current `main`. One spec, one PR.
- Verify `git branch --show-current` before EVERY commit; ignore untracked `reports/*.md`
  and `TESTING-WITH-YOUR-PHOTOS.md` (do NOT stage them).
- If a later `cargo fmt` reformats committed files, re-add ALL touched files
  (`git add -u`) before committing.
- PR title: `ci(SPEC-041): cargo-dist release pipeline + MSRV`.
- PR body per AGENTS.md §13: Decisions referenced (DEC-040, DEC-009, DEC-037); Constraints
  (no-new-top-level-deps-without-decision [tooling, DEC-040]; no-secrets-in-code;
  one-spec-per-pr); New decisions (none — DEC-040 pre-authored). **State plainly in the
  body that this arms the pipeline but cuts NO release (PR=plan only; tag push is the
  maintainer-authorized trigger) and adds no crates.io publish / tap.**
- Fill the spec's `## Build Completion` + the 3 build-reflection answers (paste the
  `dist plan` output there); append the build cost session entry below (agent
  `claude-sonnet-4-6`, numerics null — the orchestrator fills real `subagent_tokens` at
  ship).

## Cost
```
- cycle: build
  agent: claude-sonnet-4-6
  interface: claude-code
  tokens_total: null
  estimated_usd: null
  duration_minutes: null
  recorded_at: 2026-06-23
  notes: "ci/release tooling: dist 0.32.0 → dist-workspace.toml (4 targets, shell+powershell installers, GH-Releases-only) + generated release.yml (PR=plan, tag=publish; no cargo publish / tap) + [profile.dist]; rust-version=1.85.0 + msrv CI job (default+lean); RELEASING.md wording. dist plan dry-run green; fmt/clippy/test/lean/deny green; NO tag/release/publish."
```

## When done
`just advance-cycle SPEC-041 verify` (if it mis-globs or doesn't update the spec's
`cycle:` field, set `cycle: verify` in the spec frontmatter by hand), open the PR with
`gh`, and **stop** — the orchestrator pauses for the user before any merge, and the
actual release cut is a separate maintainer-authorized action.
