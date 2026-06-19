# Handoff — STAGE-009 shipped; kick off STAGE-004's first spec (metadata lane)

Paste the block below into a fresh **Opus** session in the `crustyimg` repo. STAGE-009
(web-prep power & differentiator surface) just shipped end-to-end; the user has
**already chosen** the next move: STAGE-004 (compose & metadata), first spec =
**the metadata lane (`strip` + `clean --gps`)**. This session activates STAGE-004 and
designs/builds that first spec under the usual per-spec cycle. Process mechanics are
unchanged — read them, don't relearn their lessons (most are also in auto-memory).

---

You are the ORCHESTRATOR / architect for **crustyimg**, a pure-Rust, permissive
(`MIT OR Apache-2.0`) image CLI rebuilt spec-driven. Repo root:
`/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg` (own git repo,
remote `git@github.com:jysf/crustyimg.git`, `gh` authed). `main` is clean at
`295ddc6`.

## Orient (read first)
- `AGENTS.md` (esp. §4 cost, §5/§6 gates, §13 git/PR, §15 build cycle, §18 pointers);
  `projects/PROJ-001-crustyimg-mvp/brief.md` (stage plan: **5 shipped / 0 active /
  4 pending**; STAGE-004 is NEXT).
- `docs/moat.md` — the strategic moat (built vs pending). The metadata lane is the
  start of the **verifiable-privacy axis** and unlocks `optimize`'s selective-preserve
  upgrade (DEC-024 revisit).
- `guidance/constraints.yaml`; `guidance/license-watchlist.yaml` (`just watchlist`);
  `guidance/questions.yaml` (note the open `metadata-icc-coverage` question).
- **`decisions/DEC-003-metadata-dual-lane.md`** — THE governing decision for this
  spec: pixel lane vs **container lane**; read via `kamadak-exif` (already a dep,
  read-only), edit/preserve via **`img-parts`** (EXIF/ICC segments) + **`little_exif`**
  (tag write); metadata-only commands (`strip`/`clean --gps`/`set`/`copy-metadata`)
  **never go through the pixel encode path**; default-preserve policy (keep
  orientation+ICC+copyright, drop GPS unless `--keep-gps`).
- Recent DECs: DEC-024 (optimize), DEC-025 (diff + exit 7), DEC-026 (responsive),
  DEC-027 (display default-on, supersedes DEC-011), DEC-028 (benchmarking).
- The stage you're starting: `projects/PROJ-001-crustyimg-mvp/stages/STAGE-004-compose-and-metadata.md`
  (currently `proposed` — flip to `active`). Backlog: watermark + the metadata lane
  (`strip`/`clean --gps`/`set`/`copy-metadata`).
- Run `just status` / `just specs-by-stage`.

## Where we are (2026-06-18)
**STAGE-008 + STAGE-009 SHIPPED.** Live commands: `view` (display now ON by default,
DEC-027), `info`, `resize`, `thumbnail`, `shrink`, `convert`, `auto-orient`,
`optimize`, `diff`, `responsive`, `apply`. The differentiator engine (perceptual
auto-quality + byte budgets + AVIF/WebP) and its surface (optimize/diff/responsive +
a criterion benchmark net) are done. ~290 tests; CI = 3-OS matrix + `avif` +
`webp-lossy` + `lean` (--no-default-features) + cargo-deny + cost-audit, all green.

## Current relevant state (verified this session)
- `main` clean at `295ddc6`. Next spec id: **SPEC-026**.
- **`src/metadata/` does NOT exist yet** — the container lane is net-new code for this
  stage. (The `Image` model captures a metadata bundle at load — check `src/image/`
  for what's captured: raw EXIF/ICC segments. SPEC-002/DEC-003.)
- `strip`/`clean` are **clap stubs**: `Commands::Strip { inputs: Vec<String> }` and
  `Commands::Clean { inputs, gps: bool }` in `src/cli/mod.rs`, both returning
  `CliError::NotImplemented(...)` from `dispatch`. Wire them to real handlers.
- **`img-parts` and `little_exif` are NOT in `Cargo.toml`** — adding them needs a DEC
  (`no-new-top-level-deps-without-decision`). DEC-003 pre-names them but pre-naming
  isn't a dep-add decision → emit **DEC-029** pinning the exact versions, confirming
  pure-Rust + permissive license (run `just deny` after adding), per the
  feature-gated/dep-add pattern used all stage.

## Your immediate task: activate STAGE-004 + design SPEC-026 (metadata lane v1)
The user chose **the metadata lane (`strip` + `clean --gps`)** as the first spec
(highest moat value — the verifiable-privacy axis; upgrades `optimize`). Scope it as a
clean v1:
- **`strip <inputs…>`** — remove ALL container metadata (EXIF/ICC/XMP) **without
  re-decoding pixels** (container lane, DEC-003). Output via the usual sink/fan-out.
- **`clean --gps <inputs…>`** — remove ONLY GPS/location tags, preserving everything
  else (the selective, privacy-by-default operation).
- Both operate on the container, not the pixel pipeline. Decide the crate split
  (`img-parts` for segment-level strip/rewrite; `little_exif` for tag-level GPS
  removal) and which formats v1 covers (JPEG certainly; PNG/WebP/TIFF as the crates
  allow — be honest in the spec about coverage and exit cleanly on unsupported).
- Likely **out of scope for THIS spec** (own later specs): `set` (write tags),
  `copy-metadata`, `watermark`, the EXIF **audit-linter**, and wiring the
  selective-preserve policy into `optimize` (note as a DEC-024 follow-up). Keep v1 to
  strip + clean.
- Emit **DEC-029** (pin `img-parts`/`little_exif`; pure-Rust + permissive; the
  read=kamadak / write=img-parts+little_exif division). Verify the metadata-edit path
  in design (a small probe), not just at build — the riskiest part per DEC-003 is
  reliable cross-format preserve; v1 strip/clean is lower-risk than ICC-preserve but
  still verify the write crates do what's expected on a real JPEG.

Then: scaffold (`just new-spec "..." STAGE-004 PROJ-001`), author the spec (with
`## Failing Tests` + `## Implementation Context`), flip STAGE-004 `proposed`→`active`,
**push design + DEC-029 to `main` before dispatching the build**, then build → verify →
**pause for the user before merge** → ship.

## Orchestration model + this-repo gotchas (do NOT relearn — also in auto-memory)
- Per spec: design (Opus — you) → build (Sonnet prescriptive prompt, or
  orchestrator-direct) → verify (independent read-only `/code-review` or an **Explore**
  subagent — `code-reviewer` agent type is NOT available; use `Explore`) → ship.
  **Pause for the user before every merge.** Push design+DECs to `main` before the
  build so the build PR doesn't fold the design in.
- **Background subagents can't get Bash here** — run build/verify in the main loop, or
  a FOREGROUND agent. Read-only Explore finders work as subagents.
- **Cost capture is enforced** (CI `cost-data` / `just cost-audit`): a SHIPPED spec
  needs **positive `tokens_total` on its build AND verify cycles**. Since build/verify
  run in the main loop, record **labeled order-of-magnitude estimates** (Opus $5/$25
  per MTok, ~80/20 in/out); design/ship stay `null`. Totals = sum non-null (0 for
  null) + `session_count: 4`.
- **Ship bookkeeping by hand on `main`** (helpers mis-glob): flip `task.cycle`→ship,
  fill cost sessions + `## Reflection (Ship)`, mark the timeline, `git mv` spec +
  timeline to `specs/done/`, update the stage backlog/Count, `brag add … -t "<title>"
  -p crustyimg -k shipped` (NOTE: `-t` is title, `-i` is impact — not interface).
- **⚠ VERIFY THE GIT INDEX BEFORE EVERY SHIP COMMIT.** Editor/linter file-state churn
  silently staged STALE spec content on multiple ship commits (dropped cost edits →
  main cost-audit went red once). Standing fix: after `git mv`, **re-`git add` the
  `done/` paths**, then `git show :<file>` to confirm `cycle: ship` + positive
  build/verify cost are STAGED, commit, then `git show HEAD:<file>` to confirm, and
  after push **confirm the `cost-capture audit` job is green on the main CI run.**
- **Merge dance** (branch protection: require-up-to-date ON, auto-merge OFF): branch
  off current `main` (behind 0) → clean squash via `gh pr merge <N> --squash
  --delete-branch`.
- **`cargo fmt` trap:** a global `cargo fmt` reformats already-committed files; run
  `cargo fmt` then `git add -u` before committing (local `--check` passes on the
  working tree but CI checks the commit).
- **`just deny`** (license gate, DEC-018, all-features) must pass after adding
  `img-parts`/`little_exif`. If either trips, scope an exception or a watchlist entry.
- **Verify the working-tree branch before every commit** (`git branch --show-current`);
  an unrelated `reports/daily|weekly/*.md` may appear untracked — don't commit it.
- New fan-out commands **reuse the global flags** (`--out-dir`/`--format`/`-q`/`-y`) —
  declaring locals collides (the SPEC-024 lesson). `strip`/`clean` already have their
  clap variants; just wire handlers + reuse the `run_pixel_op`-style fan-out shape (or
  a metadata-lane analog that doesn't decode pixels).

Start by confirming state (`just status` / `just specs-by-stage`), flip STAGE-004 to
active, then scaffold + design SPEC-026 (strip + clean --gps) with DEC-029. **Pause for
the user before merging.**
