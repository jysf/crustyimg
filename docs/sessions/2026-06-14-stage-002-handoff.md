# Handoff — start STAGE-002 in a fresh Opus session

Paste the block below into a new Opus session (in the `crustyimg` repo) to
continue the orchestration identically.

---

You are the ORCHESTRATOR / architect for **crustyimg**, a Rust image CLI being
rebuilt spec-driven. Repo root: `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg`
(its own git repo, remote `git@github.com:jysf/crustyimg.git`, `gh` authed).

**First, read to orient:** `AGENTS.md`, `projects/PROJ-001-crustyimg-mvp/brief.md`,
`projects/PROJ-001-crustyimg-mvp/stages/STAGE-002-view-and-info.md`,
`docs/architecture.md`, `docs/api-contract.md`, `docs/data-model.md`,
`guidance/constraints.yaml`, and the decisions in `decisions/`. Run `just status`
and `just roadmap`. (Your auto-memory also has a `crustimg-rebuild` entry.)

**Where we are:** STAGE-001 (foundation) is SHIPPED — 7 specs, PRs #1–#7. crustyimg is a
runnable clap CLI over `Source → Image::load → Pipeline(+Recipe) → Sink`; `apply --recipe`
runs end-to-end; the other 13 subcommands are stubs returning a typed NotImplemented error.
~97 tests, green 3-OS CI. DEC-002..012 are locked.

**Your job: run STAGE-002 — view & info.** Its backlog (in the stage file) is two specs:
make the `view` command real (terminal display via the Sink's viuer display, behind the
`display` cargo feature) and the `info` command real (dimensions/format/bytes/color-type/
bit-depth/alpha + ICC/EXIF presence; `--exif` reads tags via kamadak-exif; `--json` structured
output to stdout). These replace the two stubs in `src/cli`.

**The orchestration model (follow exactly):**
- This thread orchestrates; each cycle runs as a SEPARATE fresh `Agent` (general-purpose) you
  give a self-contained prompt to. ONE design agent per spec.
- **Model routing:** DESIGN and VERIFY agents → Opus (no model override). BUILD agents →
  Sonnet 4.6 (`model: "sonnet"`); make Sonnet build prompts highly prescriptive (exact
  signatures, exact test names, exact module layout, the gates).
- **Per spec:** scaffold with `just new-spec "<title>" STAGE-002` (assigns next id, SPEC-008+);
  Design agent fills the spec + failing tests + Implementation Context + writes
  `prompts/SPEC-NNN-build.md`; commit the spec to `main`; Build agent (Sonnet) implements on a
  `feat/spec-NNN-<slug>` branch, runs the 4 gates, opens a GitHub PR; Verify agent (Opus,
  READ-ONLY — no file edits) reviews the PR cold and returns ✅/⚠/❌.
- **Cadence:** run design→build→verify autonomously, then PAUSE for the user's OK before each
  merge/ship.
- **Git/PR conventions (AGENTS.md §13):** design+specs commit directly to `main`; build work
  goes through a squash-merged PR; **verify is read-only and ALL build/verify/ship bookkeeping
  (timeline marks, reflections, cost totals, archive to `specs/done/`, stage backlog) is applied
  by YOU on `main` after merge** — this avoids branch/main divergence. PR title carries the spec
  id; PR body has the §13 structured sections + Claude Code footer.
- **Bookkeeping gotchas:** `just advance-cycle`/`archive-spec` MIS-GLOB when a spec has multiple
  `SPEC-NNN*` files — do ship bookkeeping by hand (hand-edit `task.cycle`, `git mv` spec+timeline
  to `done/`, update the stage backlog + Count). Build agents must use ACCURATE timeline-marker
  wording ("PR #N opened", never "merged" at build time — a past build agent got this wrong).
- **Cost:** each cycle appends a `cost.sessions` entry to the spec (subagent → null numerics +
  a note); totals computed at ship (session_count=4 typical).
- **Brag on ship:** after each spec ships (and the stage), run `brag add -t … -d … -T … -p
  crustyimg -k shipped -i …` to log the accomplishment + impact (the user wants this).
- Each merge syncs `main` locally first; delete the merged remote branch (or `git fetch --prune`).

**Constraints that bind STAGE-002:** `ergonomic-defaults` (common task = one short command),
`no-unwrap-on-recoverable-paths`, `every-public-fn-tested`, `clippy-fmt-clean`,
`no-new-top-level-deps-without-decision` (emit a DEC for any new crate; `view` likely needs no
new dep since viuer already exists behind the `display` feature — confirm). `view` must refuse
on a non-tty; `info --json` writes structured output to stdout, diagnostics to stderr.

Start by confirming state (`just status`/`just roadmap`), then scaffold and design SPEC-008
(`view`). Pause for the user before merging anything.
