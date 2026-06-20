# Handoff — STAGE-005 + STAGE-006 shipped; STAGE-007 started; next = release & distribution

Paste the block below into a fresh **Opus** session in the `crustyimg` repo. This
session shipped **8 specs end-to-end (SPEC-032…039)**: it **completed STAGE-005**
(edit + --save-recipe), **completed STAGE-006** (the MVP hardening exit gate, 5
specs), and **started STAGE-007** (release & distribution: the crate is now
publish-ready + a CHANGELOG/release policy). Process mechanics are unchanged (also
in auto-memory).

---

You are the ORCHESTRATOR / architect for **crustyimg**, a pure-Rust, permissive
(`MIT OR Apache-2.0`) image CLI rebuilt spec-driven. Repo root:
`/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg` (own git repo,
remote `git@github.com:jysf/crustyimg.git`, `gh` authed). **`main` is clean at
`95cdf89`**, all CI green.

## Orient (read first)
- `AGENTS.md` (esp. §4 cost, §5/§6 gates, §13 git/PR, §15 build cycle, §18 pointers).
- `projects/PROJ-001-crustyimg-mvp/stages/STAGE-007-release-and-distribution.md` —
  **status: active, 2 shipped / 0 active / 5 pending.** The pending items are the work.
- `docs/moat.md` (current strategy snapshot — 5 built axes + trust; only distribution
  remains), `RELEASING.md` (the release-cut checklist + the maintainer-authorized rule),
  `SECURITY.md` (the STAGE-006 `## Verification` table).
- Run `just status` / `just specs-by-stage`.

## Where we are (2026-06-19)
**The MVP functional surface AND the hardening exit gate are COMPLETE.** Shipped
this session:
- **STAGE-005** — SPEC-032 `edit` + `--save-recipe` (recipe round-trip byte-pinned).
- **STAGE-006 (MVP exit gate, 5 specs)** — SPEC-033 decode limits (DEC-034), SPEC-034
  path/symlink guards (DEC-035), SPEC-035 recipe limits (DEC-036), SPEC-036 full
  supply-chain cargo-deny CI gate (DEC-037), SPEC-037 capstone: resize output cap +
  save-recipe symlink guard + threat-model verification (DEC-038). An adversarial
  security review over the cumulative diff found **no unresolved finding**.
- **STAGE-007 started** — SPEC-038 made the crate **publish-ready** (Cargo.toml
  metadata + `exclude` keeping `assets/` + dual `LICENSE-MIT`/`LICENSE-APACHE`,
  verified by `cargo publish --dry-run`); SPEC-039 added `CHANGELOG.md` (0.1.0 = the
  MVP) + `RELEASING.md` (SemVer 0.x + `vX.Y.Z` tag convention + release-cut checklist).
- **crates.io name `crustyimg` is FREE.** `Cargo.toml` version is `0.1.0`.

New DECs this session: **DEC-034..038** (decode/path/recipe/supply-chain/resize
hardening). No new runtime dependency added in STAGE-006/007 (cargo-deny is CI tooling).

## Your next task: finish STAGE-007 (release & distribution)
Backlog (`STAGE-007` file): **#1 publish-ready crate** ✓, **#2 changelog/release docs** ✓.
Remaining:
- **#6 README install/usage rewrite + clap shell completions** — **SAFE** (no
  outward-facing action). Good next spec: rewrite README install (brew/cargo/download)
  + a usage example; generate shell completions via `clap_complete` (a small build.rs
  or a `completions` subcommand). Run the normal design→build→verify→ship cycle.
- **#3 release CI pipeline (cargo-dist) + MSRV (`rust-version`)** — sets up a
  tag-triggered workflow → cross-platform binaries + checksums → GitHub Releases.
  Writing the config/workflow is safe; **cutting an actual release/tag is
  outward-facing**.
- **#4 Homebrew tap (`jysf/homebrew-tap`)**, **#5 `cargo publish`**, **#7 dual
  lean/full artifacts** — **OUTWARD-FACING / IRREVERSIBLE.**

⚠ **STAGE-007 is different from the code stages: several items create real releases,
a public tap repo, or an irreversible crates.io publish.** Per the operating rules
(and `RELEASING.md`), the **publish / tag / push-tag / tap-creation steps need
EXPLICIT maintainer authorization at execution** — do NOT run `git tag`, `cargo
publish`, `gh release`, or create the tap repo without the user's go-ahead in the
moment. Design + dry-run freely; pause before the outward-facing action itself
(not just before the PR merge).

Recommended order: **#6 (safe) → #3 config (safe, dry-run) → then pause for the user
to authorize the actual release/tap/publish.**

## Critical process notes (do NOT relearn — also in auto-memory)
- Per spec: design (Opus, you) → build (Sonnet, prescriptive prompt, `model:
  "sonnet"`) → verify (independent **Explore** subagent, Opus) → ship. **Pause for the
  user before every merge.** Push design+DECs to `main` first.
- **Background subagents can't get Bash** — dispatch build/verify as FOREGROUND agents.
- **Run the lean build (`cargo build --no-default-features`) AND the full `cargo deny
  check advisories bans sources licenses`** in build + verify (the deny gate is now the
  full supply-chain check, SPEC-036/DEC-037).
- **⚠ VERIFY THE GIT INDEX BEFORE EVERY SHIP COMMIT** (`git show :<file>` after `git
  mv` + re-add; editor/linter churn re-stages stale content). Confirm cost-capture +
  lean + supply-chain jobs green on main CI after push.
- **Cost capture:** build = real metered `subagent_tokens` (Sonnet $3/$15 per MTok,
  ~80/20). Verify (Explore) = no usage block → order-of-magnitude estimate, labeled.
  design/ship = null-with-note. `session_count: 4`. `just cost-audit` must stay green.
- **Ship bookkeeping by hand on `main`** (helpers mis-glob; `advance-cycle` may point at
  the prompt, not the spec — set `cycle:` by hand). Flip `cycle`→ship, fill cost +
  `## Reflection (Ship)`, `git mv` spec+timeline to `specs/done/`, update stage
  backlog/Count, `brag add … -k shipped`.
- **Merge dance:** branch off current `main` → `gh pr merge <N> --squash
  --delete-branch`. Untracked `reports/daily|weekly/*.md` and a root
  `TESTING-WITH-YOUR-PHOTOS.md` (a user dogfood guide — leave it) may appear; don't
  commit them.
- **Verification/hardening lesson** ([[read-whole-function-before-asserting-a-gap]]):
  read the WHOLE target function + audit existing guards before asserting a gap (SPEC-037
  wrongly assumed resize was unbounded — SPEC-010 had already hardened it).

Start by confirming state (`just status` / `just specs-by-stage`), then design the next
STAGE-007 spec (recommend **#6 README + completions**, the last safe item). **Pause for
the user before merging, and before any outward-facing release/tap/publish action.**
