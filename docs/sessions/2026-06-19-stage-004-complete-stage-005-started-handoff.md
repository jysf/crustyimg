# Handoff — STAGE-004 complete; STAGE-005 started; next = `edit` + `--save-recipe`

Paste the block below into a fresh **Opus** session in the `crustyimg` repo. This
session shipped 6 specs end-to-end (SPEC-026…031): it **completed STAGE-004**
(compose & metadata) and **started STAGE-005** (batch & recipes) with the parallel
batch `apply`. The one remaining STAGE-005 spec is **`edit` + `--save-recipe`** — the
recipe-*creation* command. Process mechanics are unchanged (also in auto-memory).

---

You are the ORCHESTRATOR / architect for **crustyimg**, a pure-Rust, permissive
(`MIT OR Apache-2.0`) image CLI rebuilt spec-driven. Repo root:
`/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg` (own git repo,
remote `git@github.com:jysf/crustyimg.git`, `gh` authed). **`main` is clean at
`c798d4e`**, all CI green.

## Orient (read first)
- `AGENTS.md` (esp. §4 cost, §5/§6 gates, §13 git/PR, §15 build cycle, §18 pointers).
- `projects/PROJ-001-crustyimg-mvp/stages/STAGE-005-batch-and-recipes.md` —
  **status: active, 1 shipped / 0 active / 1 pending.** The pending item is the next task.
- `decisions/DEC-005` (recipe round-trip via the registry), `DEC-006` (no async; rayon),
  `DEC-031` (watermark recipe round-trip deferred to STAGE-005 — relevant if `edit`
  exposes `--watermark`).
- Run `just status` / `just specs-by-stage`.

## Where we are (2026-06-19)
**STAGE-004 SHIPPED** (PRs #30–#34): metadata lane (`strip`, `clean --gps`, `set`,
`copy-metadata` — all container-lane, no pixel re-encode) + compositing (`watermark
--image` and `watermark --text`). **STAGE-005 STARTED** (PR #35): SPEC-031 made
`apply --recipe` a **rayon-parallel batch** over a source list (`-j` bounded) with an
`indicatif` progress bar, name-template output, and exit-6 partial failure.

New decisions this session: **DEC-029** (img-parts/little_exif), **DEC-030**
(copy-metadata JPEG-only — PNG EXIF crate mismatch), **DEC-031** (multi-image overlay
loaded at IO boundary), **DEC-032** (ab_glyph + bundled BSD-3 Go font, no imageproc),
**DEC-033** (indicatif). Deps added: img-parts, little_exif, ab_glyph, rayon, indicatif
(all pure-Rust, permissive; `just deny` green). Bundled asset: `assets/fonts/Go-Regular.ttf`.

## Model policy (set + validated this session)
- **build = Sonnet 4.6** (dispatch the build Agent with `model: "sonnet"`; the
  prescriptive build prompt is written for it). **verify = Opus** (the read-only Explore
  subagent inherits the session model). design/ship = Opus (you, main loop).
- SPEC-031 was the first Sonnet build — it cleared Opus verify with no concerns at ~40%
  lower build cost. Keep this split.

## Your next task: SPEC-032 — `edit` + `--save-recipe` (last STAGE-005 spec)
The recipe-*creation* counterpart to SPEC-031's replay. Scope (confirm against the stage
file + DEC-005):
- **`edit <input> [op flags…] [-o OUT] [--save-recipe FILE]`** — build an ordered op
  list from CLI flags, run it once on a single image (decode-once → ops → encode), and
  **optionally serialize the exact chain to a TOML recipe** via the registry (DEC-005,
  `Recipe::from_ops` / `to_toml`) so `apply --recipe` replays it identically.
- The op flags map to the **registry ops** (identity/invert/resize/auto-orient today).
  Decide the flag surface (e.g. `--resize-max N`, `--auto-orient`, …) — keep v1 to ops
  that round-trip through `with_builtins`. Watermark in recipes is **deferred** (DEC-031);
  if `edit --watermark` is wanted, it needs the registry wiring first (scope out or a
  separate spec).
- `edit` is currently a clap stub (`Commands::Edit { input, save_recipe }`,
  `NotImplemented`). Reuse `Recipe`, the registry, the single-input load + `build_sink`.
- A **design-time probe** isn't needed (no new external crate — it composes existing
  registry ops + recipe serialization, both shipped in SPEC-006). No new dep expected →
  likely **no new DEC**. Verify `Recipe::from_ops(...).to_toml()` round-trips what `edit`
  builds (a quick check against SPEC-006's API).

Then: scaffold (`just new-spec "..." STAGE-005 PROJ-001`), author the spec (with
`## Failing Tests` + `## Implementation Context`), **push design to `main` before the
build**, build (Sonnet) → verify (Opus Explore) → **pause for the user before merge** →
ship. When it ships, **STAGE-005 is complete** — run the stage-ship reflection and flip
STAGE-005 to shipped (see how STAGE-004 was closed this session).

## Critical process notes (do NOT relearn — also in auto-memory)
- Per spec: design (Opus, you) → build (Sonnet, prescriptive prompt, `model: "sonnet"`)
  → verify (independent **Explore** subagent, Opus; `code-reviewer` type unavailable) →
  ship. **Pause for the user before every merge.** Push design+DECs to `main` first.
- **Background subagents can't get Bash** — dispatch build/verify as FOREGROUND agents.
- **Run `cargo build --no-default-features` (the CI lean build) in build AND verify** —
  it's CI-only otherwise; a dep default-feature (e.g. ab_glyph `std`) or lean-only break
  surfaces post-merge. (A transient crates.io flake once reddened it on SPEC-028 — rerun
  + a local lean build confirms flake vs real.)
- **⚠ VERIFY THE GIT INDEX BEFORE EVERY SHIP COMMIT.** Editor/linter churn re-stages
  stale spec content: after `git mv`, re-`git add` the `done/` paths, `git show :<file>`
  to confirm `cycle: ship` + real build/verify cost are staged, commit, then `git show
  HEAD:<file>`, and after push **confirm the cost-capture audit job is green on main CI.**
- **Cost capture:** build runs as a metered subagent — record real `subagent_tokens` at
  ship (Sonnet $3/$15, Opus $5/$25 per MTok, ~80/20). Verify (Explore) returns no usage
  block → order-of-magnitude estimate, labeled. design/ship = null-with-note.
  `session_count: 4`. `just cost-audit` must stay green.
- **Ship bookkeeping by hand on `main`** (helpers mis-glob): flip `cycle`→ship, fill cost
  + `## Reflection (Ship)`, mark the timeline, `git mv` spec+timeline to `specs/done/`,
  update the stage backlog/Count, `brag add … -t "<title>" -p crustyimg -k shipped`.
- **Merge dance:** branch off current `main` → clean squash via `gh pr merge <N> --squash
  --delete-branch`. An unrelated untracked `reports/daily|weekly/*.md` may appear — don't
  commit it.
- New fan-out commands **reuse the global flags** (`--out-dir`/`--format`/`-q`/`-y`/`-j`/
  `--name-template`) — declaring locals collides (the SPEC-024 lesson).

Start by confirming state (`just status`), then scaffold + design SPEC-032 (`edit` +
`--save-recipe`). **Pause for the user before merging.** Shipping it completes STAGE-005.
