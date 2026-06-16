# Handoff — finish STAGE-003 in a fresh Opus session

Paste the block below into a new Opus session (in the `crustyimg` repo) to
continue the orchestration identically. It carries the current state plus the
gotchas learned the hard way across STAGE-002/003 — read them; they will save
you a recovery.

---

You are the ORCHESTRATOR / architect for **crustyimg**, a Rust image CLI being
rebuilt spec-driven. Repo root:
`/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg` (its own git
repo, remote `git@github.com:jysf/crustyimg.git`, `gh` authed).

**First, read to orient:** `AGENTS.md`,
`projects/PROJ-001-crustyimg-mvp/brief.md`,
`projects/PROJ-001-crustyimg-mvp/stages/STAGE-003-transform-and-output.md`,
`docs/api-contract.md`, `docs/data-model.md`, `guidance/constraints.yaml`, and
the decisions in `decisions/` (esp. **DEC-014** operation-params mechanism,
**DEC-015** output-format preservation + partial-batch exit 6, **DEC-016**
encode-quality policy, DEC-008 resize backend, DEC-004 codec policy, DEC-003
metadata dual-lane). Read the shipped `src/cli/mod.rs` and `src/sink/mod.rs` in
full — they are the surfaces the remaining commands extend. Run `just status`
and `just roadmap`. Your auto-memory has entries you should heed:
`push-design-before-build-branches`, `isolate-build-when-background-task-active`,
`verify-test-existence-not-just-gate-count`, `crustimg-rebuild`.

**Where we are (2026-06-15):** STAGE-001 (foundation, 7 specs) and STAGE-002
(view & info, 2 specs) are SHIPPED. **STAGE-003 (transform & output) is 4/6
shipped:** SPEC-010 (`resize` Operation + the OperationParams mechanism,
DEC-014), SPEC-011 (`resize` CLI + multi-input fan-out, DEC-015), SPEC-012
(`thumbnail`), SPEC-013 (`shrink` + the `-q` quality-aware Sink encode, DEC-016).
`main` is clean and synced, **no open PRs, 181 tests, 3-OS CI green**. Live
commands: `view`, `info`, `resize`, `thumbnail`, `shrink`, `apply`. The app is
usable now (`just run …`, `just view photo.png`, `just install-display`).

**IMPORTANT — the CI gate changed:** CI now runs
`cargo clippy --all-targets -- -D warnings` (not plain clippy). AGENTS §5/§6/§11
and the `clippy-fmt-clean` constraint are updated. **Every build prompt must use
`cargo clippy --all-targets -- -D warnings` as the clippy gate.**

**Your job: finish STAGE-003 — two specs left, then ship the stage.**
- **SPEC-014 `convert`** — `convert <INPUT...> --format FMT [-q Q]`: re-encode
  between the pure-Rust core formats (JPEG/PNG/GIF/BMP/TIFF/ICO). `--format` is
  REQUIRED. No pixel transform — decode then re-encode to the target format,
  honoring `-q` (reuse the DEC-016 quality path) and writing via the shared
  `run_pixel_op` fan-out. Output format = the `--format` target (overrides
  DEC-015 source-preserve). An unsupported/unbuilt codec (e.g. AVIF without the
  feature) → **exit 4** (DEC-004). Likely CLI-only (an empty/`Identity`
  pipeline; the work is the forced output format + quality). Probably complexity
  S–M, no new Operation, no new DEC.
- **SPEC-015 `auto-orient`** — `auto-orient <INPUT...>`: read the EXIF
  orientation (kamadak-exif is already a dep) and bake the corresponding
  rotation/flip into the pixels; the pixel-lane re-encode then drops the
  (now-satisfied) orientation tag inherently. This IS a new `Operation`
  (`AutoOrient`) registered in the registry (recipe-usable) — the first op that
  reads container metadata to drive a pixel transform. Map EXIF orientation
  values 1–8 to `DynamicImage` rotate/flip ops. Complexity ~M. Reuses
  `run_pixel_op` for the CLI.
- **Then run the STAGE-003 STAGE SHIP:** stage-level reflection in the stage
  file, `status: shipped` + `shipped_at`, update the PROJ-001 `brief.md` stage
  plan (STAGE-003 → shipped, count), and `brag add` the stage.

**The orchestration model (follow exactly):**
- This thread orchestrates; each cycle runs as a SEPARATE fresh `Agent`
  (general-purpose) you give a self-contained prompt to. ONE design agent per
  spec.
- **Model routing:** DESIGN and VERIFY agents → Opus (no model override). BUILD
  agents → Sonnet 4.6 (`model: "sonnet"`); make build prompts highly
  prescriptive — exact signatures, exact test names, exact module layout, the
  gates. **Mirror the latest build prompt** `projects/PROJ-001-crustyimg-mvp/specs/prompts/SPEC-013-build.md`
  as your template (it bakes in the lessons below).
- **Per spec:** `just new-spec "<title>" STAGE-003` (assigns next id, SPEC-014+);
  Design agent fills the spec + a precise `## Failing Tests` section +
  Implementation Context + writes `prompts/SPEC-NNN-build.md` (+ a DEC if a
  genuinely new durable decision arises — none expected for convert; maybe not
  for auto-orient either); **commit the design to `main` AND PUSH it** (see
  gotcha #1); Build agent (Sonnet) implements on a `feat/spec-NNN-<slug>` branch,
  runs the 4 gates, opens a GitHub PR; Verify agent (Opus, READ-ONLY) reviews the
  PR cold and returns ✅/⚠/❌; on ⚠ punch list, send a focused Sonnet follow-up to
  fix it on the same branch, then re-confirm.
- **Cadence:** run design→build→verify autonomously, then PAUSE for the user's OK
  before each merge/ship.
- **Git/PR (AGENTS.md §13):** design+specs commit directly to `main`; build work
  goes through a squash-merged PR; verify is read-only; ALL build/verify/ship
  bookkeeping (timeline marks, reflections, cost totals, archive to
  `specs/done/`, stage backlog + Count) is applied by YOU on `main` after merge.
  PR title carries the spec id; PR body uses the §13 structured sections + the
  Claude Code footer. Build agents' commits use the Sonnet 4.6 Co-Authored-By
  trailer; your `main` commits use the Opus trailer.
- **Cost:** each cycle appends a `cost.sessions` entry to the spec (subagent →
  null numerics + a note); compute totals at ship (`session_count: 4` typical).
- **Brag on ship:** `brag add -t … -d … -T … -p crustyimg -k shipped -i …` after
  each spec ships and after the stage ships (the user wants this).

**Hard-won gotchas — DO NOT relearn these:**
1. **Push the design commit to `origin/main` BEFORE dispatching the build
   agent.** The build agent's STEP 0 does `git checkout main && git pull && git
   checkout -b feat/…`; if the design is unpushed, the build branch is based on
   it and the build PR's squash folds the design in (and local `main` diverges →
   you must reset). Commit design → `git push origin main` → THEN dispatch build.
2. **Verify the working-tree branch before every commit YOU make.** The working
   directory is shared; a background "task chip" once checked out its own branch
   here and your commit landed on it. Run `git branch --show-current` before
   `git add`/`commit`. (If a background task is active, consider dispatching
   agents with `isolation: "worktree"`.)
3. **A green test COUNT does not prove the spec's tests were written.** A dropped
   build once shipped with its entire integration suite missing — gates were
   green because the unit tests passed and absent tests just don't run. Verify
   caught it. So: build prompts MUST instruct the agent to (a) commit
   incrementally and (b) before finishing, confirm EVERY test named in the spec's
   `## Failing Tests` exists and runs (check each off). And the verify prompt MUST
   independently grep for each named test.
4. **The API drops subagent sessions intermittently (socket errors).** Seen on a
   build (recovered) and twice on one design (the orchestrator then authored that
   design directly — DEC-016 + spec + build prompt — which worked cleanly). On a
   BUILD drop: check `git status`/branch, finish the gates + bookkeeping + PR
   yourself, but re-verify the test suite matches the spec. On repeated DESIGN
   drops: author the design yourself (you have the context), or retry once.
5. **Branch protection: "require branches up to date" is ON and auto-merge is
   DISABLED.** If `main` advanced since the PR branch was cut, `gh pr merge`
   fails `BEHIND`. Do NOT `--admin` bypass. Instead: `gh pr update-branch <N>`,
   wait for CI to go green on the new head, then `gh pr merge <N> --squash
   --delete-branch`. A background bash loop that polls `gh pr checks <N>` until
   conclusive and then merges works well (run it with `run_in_background: true`).
6. **`just advance-cycle`/`archive-spec` MIS-GLOB** with multiple `SPEC-NNN*`
   files — do ship bookkeeping BY HAND: hand-edit `task.cycle`, append cost
   sessions, fill the ship reflection, `git mv` the spec + timeline to
   `specs/done/`, update the stage backlog line + Count.
7. **Accurate timeline wording:** build marks the line `[x]` with "PR #N opened"
   — never "merged"/"approved" (verify/ship record those later).
8. **Derive `Debug` on any new public type; don't `{:?}`-format non-Debug types**
   (a Sonnet build hit two compile cycles on this).

**Reusable patterns the remaining specs build on (read the code to confirm):**
- `fn run_pixel_op(pipeline: Pipeline, inputs: &[String], global: &GlobalArgs,
  quality: Option<u8>) -> Result<(), CliError>` in `src/cli/mod.rs` — the shared
  fan-out: resolve→flatten→single-vs-multi `--out-dir`→per-input
  `output_format_for`→sink→partial-batch exit 6. `resize`/`thumbnail`/`shrink`
  all call it; `convert`/`auto-orient` should too.
- Ops are built via `OperationRegistry::with_builtins().build("<name>", &params)`
  (DEC-014; `OperationParams` is a `BTreeMap<String, toml::Value>` newtype, each
  op parses its own params). `auto-orient` registers a new `"auto-orient"` op.
- DEC-015 output-format precedence: `--format` > `-o` ext > preserve source;
  partial-batch → exit 6, single-input failure keeps its natural code.
- DEC-016 quality: `-q` → JPEG quality via `JpegEncoder::new_with_quality`,
  ignored for lossless formats; threaded `run_pixel_op → Sink::write →
  encode_to_bytes`. `convert` reuses this verbatim.
- `CliError` exit codes: 2 usage (clap `ArgGroup`/`CliError::Usage`), 3
  not-found, 4 unsupported format, 5 write-refused, 6 partial batch. `convert`
  needs **4** for an unsupported/unbuilt target codec — confirm whether a new
  `CliError`/`SinkError` arm + an `exit_code_mapping_is_total` extension is
  needed, or an existing arm already maps to 4.

**Constraints binding STAGE-003:** `ergonomic-defaults`, `single-image-library`
(use `image` + `fast_image_resize` only — no second pixel lib), `decode-once-no-
per-op-disk`, `no-unwrap-on-recoverable-paths`, `every-public-fn-tested`,
`clippy-fmt-clean` (now `--all-targets`), `no-new-top-level-deps-without-decision`,
`untrusted-input-hardening`. Metadata WRITE / selective preserve / `--keep-gps`
is the STAGE-004 container lane (`img-parts`/`little_exif`, not deps yet) — keep
it out of STAGE-003; `auto-orient` only READS orientation (kamadak-exif) and the
pixel re-encode drops the tag inherently.

Start by confirming state (`just status`/`just roadmap`), then scaffold and
design **SPEC-014 (`convert`)**. Pause for the user before merging anything.
