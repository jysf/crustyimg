# PROJ-011 — orchestration handoff

**Rewritten 2026-09-06.** Supersedes the 2026-08-23 version entirely — that one was written
before STAGE-049 existed as shipped work and before the cost ledger was known to be wrong.

> **This file deliberately does NOT restate repo state.** `just status`, `just backlog`,
> `just roadmap` and `just specs-by-stage` all report correctly — **trust them over any summary,
> including this one.** What follows is only what the tooling cannot show.

---

## Read first

`/AGENTS.md`, then `just status` and `just backlog`. Then this file.

**You orchestrate; you never design, build, or verify yourself.** ⛔ **Maintainer rule
(2026-10-24): every step of every spec's cycle runs in its own separate Claude session** —
**frame**, design, build, verify, and ship bookkeeping each get a fresh session (own worktree).
Code quality is the reason: a clean session per step stops one step's context from leaking into
the next. Each session's brief is a persisted prompt in `specs/prompts/SPEC-NNN-<phase>.md`,
pushed to `main` **before** that session cuts its branch. **The loop:** the orchestrator writes
the prompt and hands it to the maintainer, who pastes it into a fresh CLI session
(`just wt <branch>` from any checkout: it cuts a worktree from a fresh `origin/main` and starts
`claude` in it) and pastes the session's final report back. `just wt-ls` shows every worktree's
state; `just wt-rm <name>` / `just wt-clean --yes` remove the ones that are merged, clean and
not in use. So every prompt must be
self-contained and end with a stand-alone report spec. A sub-session (subagent) is the exception,
not the default. Release-prep work (CHANGELOG, docs, skill) follows the same rule. When a session reports
back, check its actual result (diff, PR, CI legs, completion table) before advancing — a session's
note is a claim, not proof. Never poll CI. Every commit in every session is DCO signed off
(`git commit -s`).

### The cycle at a glance (re-read 2026-10-26 from `AGENTS.md` §8/§9/§15 and `FIRST_SESSION_PROMPTS.md`)

| Cycle | Fresh session does | Lands | Ends with |
|---|---|---|---|
| frame | go/no-go, size, scope, rough file list; creates the stub | `main` directly | `SPEC-NNN frame: GO …` |
| design | ACs, failing tests, implementation context | `main` directly | spec at `cycle: build` |
| build | makes the tests pass, Build Completion, cost | **draft** PR | `… build: draft PR #NNN …` |
| verify | reviews the PR cold, read-only, own mutations | nothing (verdict in report) | `✅ APPROVED` / `⚠ PUNCH LIST` / `❌ REJECTED` |
| *merge* | **the maintainer**, only after ✅ | squash to `main` | |
| ship | ship reflection, cost totals, `just archive-spec` | `main` directly | spec in `done/` |

After the last spec in a stage ships: **Stage Ship** (`FIRST_SESSION_PROMPTS.md` prompt 1d). For
0.8.0: then release prep, then `just release 0.8.0`, then the maintainer tags.

**Rules added 2026-10-26, after SPEC-131 (#200) merged before its verify ran:**
1. ⛔ **Builds open DRAFT PRs.** A draft cannot be merged by accident. Only after verify's ✅ does
   the orchestrator run `gh pr ready <N>` and tell the maintainer "merge #N". The merge gate used
   to live only inside prompts the maintainer never reads.
2. **The orchestrator keeps the timelines on `main`.** `[~]` when it hands out a prompt, `[x]`
   (PR, cost, date) when the report checks out. Builds and verifies do not edit the timeline,
   because AGENTS §13 keeps it off feature branches. Frame and design commit to `main`, so they
   mark their own.
3. **Verify and ship bookkeeping:** the orchestrator records verify's verdict and cost in the spec
   on `main`; a separate ship session writes the ship reflection, totals and archive.
4. **One next action per message to the maintainer.** End every message with one "Your next
   step" block: one command and one paste line. Never bundle a launch command with another
   task or a question; the SPEC-131 verify prompt was missed that way.
5. **At most two builds at once,** never two on the same files. Verify is read-only and may run
   alongside. Remove a session's worktree by name (`just wt-rm <branch>`) once its report checks
   out.
6. **A bug found outside the spec is recorded when it is found** (from SPEC-132's ship). The
   build names it under an **Out-of-spec finding** line in `## Build Completion`: what it is,
   whether released versions have it, and the CHANGELOG / DEC lines it needs. If the build fixes
   it, it writes those lines in the same PR; if not, it files a STAGE-050 backlog item. SPEC-132
   fixed a Linux `--watch` loop in released 0.7.x and recorded it only as a deviation, which cost
   a punch-list round.

---

## Where this stands

**0.8.0 is released (2026-10-28, `v0.8.0` → `139b381`).** It's the one release this project
promised: SPEC-126 to SPEC-135, plus resvg 0.49 (PR #211, which the maintainer folded in at the
end). All five surfaces show 0.8.0: crates.io, the GitHub Release, the Homebrew tap, npm
`crustyimg-wasm` and the jysf.org page. Main's CI on the tagged commit passed all 13 jobs. The
one-lockfile-migration rule this section used to carry is now discharged.

How the release went, for next time:
- Release prep ran as two parts: notes (Opus) → verify → fix, then the cut (Sonnet), each as a
  draft PR. The notes verify caught a false claim (`build --format`); the code bug was filed,
  and the docs were reworded.
- A late dependency PR moves the tag target. Re-verify it like code, and tag its merge commit,
  not the release commit. The #211 verify found the CHANGELOG had named the wrong resvg panic,
  and that 0.7.1 crashes on a hostile `feComposite` SVG (fixed by #211).
- `just wt <branch>` for a branch that exists only on the remote creates a *new* branch from
  `main`. A fix prompt for a maintainer-opened PR must fetch and reset to `origin/<branch>`.
- npm returned **404 on PUT** for a logged-in owner: the saved token could read but not
  publish. `npm login`, then re-run. Publish npm from a detached worktree at the tag
  (`git worktree add ../crustyimg-vX.Y.Z vX.Y.Z`), so a dirty main checkout doesn't block it.

### What is open now
- **STAGE-054:** 1 shipped / 0 active / **16 pending**. These are the 0.8.1 candidates, including
  `build --format` being ignored, the `render_text` library panic, spec ids in `--help` and the
  wasm message, and SVG font families other than Go drawing no text.
- **STAGE-053** (guards, proposed) and **STAGE-055** (the `.cube` LUT op, proposed, homed by
  #212).
- **PROJ-014 / Wave 4** (manifest, `{hash}`, placeholders, favicon) is the post-0.8.0 feature
  plan.

---

## Open, waiting on the maintainer

Batching these in one sitting unblocks more than anything an orchestrator can do alone.

- **The watermark coverage threshold.** ⚡ **No longer an open question — it is now a choice between
  two numbers.** Measured 2026-09-03 with a geometric measure (decode both to RGBA8, count differing
  pixels; SSIMULACRA2 is the WRONG instrument — it is non-monotonic at small canvases, scoring 24 px
  as *less* damaged than 64 px). The backlog's **~25 % lands at a 32 px canvas** and would pass
  48 px (17 %), 64 px (13 %) and 128 px (7 %), all visibly ruined. **The visual read puts it nearer
  2–5 %.** Full curve on STAGE-050. A **clipping** predicate may be the better primary rule — it
  needs no threshold at all.
- **The `--name-template` pin ruling** — warn / honour-the-template / document-and-keep. Same family
  as the `-o`-extension pin ruling, which moved to PROJ-013 STAGE-037. Worth ruling together.
- **STAGE-041's real status.** The repo has understated it since 2026-08-16. **Do not re-plan it
  against what the repo says.**
- **`mp4-atom` DEC** — blocks PROJ-012 from being specced at all.
- **The two GitHub Action releases are still DRAFTS.** `v1.0.1` on `jysf/setup-crustyimg` and
  `jysf/crustyimg-action`, self-tests green, finished since **2026-08-12**. Publishing is a click
  and it is the smallest-effort/highest-visibility item on the board.
- **The jysf.org project page** — a fourth public surface. Updated for 0.8.0 (2026-10-28), but
  it's still in no release checklist. Adding it to `RELEASING.md` step 6 is a one-line change.

---

## Traps this wave paid for

1. **⚡ MEASURE THE SCENARIO, NOT THE CONTROL.** Three times in three specs I drove a table and the
   row carrying the argument did not use the feature the argument was about. SPEC-127's Call 1 was
   settled on a `version = "2"` recipe **with no `format` key** — the one case where v2 buys nothing
   — so AC-5's second half was unachievable and the rationale was wrong. **When a claim rests on a
   driven table, the row carrying it must use the feature.**
2. **⚡ AN AC NAMING TWO THINGS NEEDS A TEST PER THING.** SPEC-128's AC-5 said "overlay/**font**" and
   only overlay was tested — half the asset mechanism shipped unguarded. Count the nouns in each AC
   against the tests listed; it is free at design time.
3. **⚡ THE COST LEDGER WAS ~2× WRONG AND IS NOW FIXED — do not regress it.** Claude Code writes one
   JSONL line **per content block**. Lines sharing a `.message.id` repeat identical
   `input`/`cache_creation`/`cache_read` while `output_tokens` **grows**. Correct method: dedup by
   id, take those three from the group, take **MAX** output. Summing every line over-counts ~2×;
   taking the first line's output under-counts ~11×. **Both mistakes are on record here.** 32 records
   were corrected in place; 10 remain flagged because no transcript reproduces them.
   📌 **Transcripts live in a project dir PER WORKTREE** (`~/.claude/projects/<slug>/`), and a
   subagent's is under `<session-id>/subagents/`. Searching only the main dir wrongly concluded 22
   records were unrecoverable; 12 were in sibling dirs.
4. **VERIFY BEFORE YOU COMMIT — read back what you wrote and diff it against what you intended.** A
   regex using a lazy `(?:.*\n)*?` spanned across YAML block boundaries and scattered 20 corrections
   onto the wrong entries. It reached `main` because I checked that the edit *succeeded* and not that
   it was *right*. Reverted and redone with a block parser. This one rule has since caught a phantom
   backlog bullet, an invented DEC filename, and an invented `tokens_breakdown`.
5. **`just check` is weaker than CI in a way CLAUDE.md does not list:** it never runs the **wasm
   bundle-size gate**. SPEC-128's local matrix was clean while the PR was red on a real 10.6 %
   regression. Filed on STAGE-053. DCO sign-off is the other CI-only catch — **4th recurrence**.
6. **A moved size gate is a purchase and belongs in DEC-066's ledger** — the record of what was cut,
   paid for, and refused. It had *declined* to drop `ssimulacra2` for 23,540 B; SPEC-128 bought 5×
   that and would have left no row. DEC-066's own `affected_scope` omitted the gate file, so the
   audit could not have flagged it. Both fixed.
7. **`scripts/decisions-audit.sh` silently drops an INLINE-ARRAY `affected_scope`.** `DEC-015` is
   invisible to `--changed`, forever, and DEC-015 governs `docs/api-contract.md`. Root cause is
   `decisions/_template.md:33`, whose example uses the unreadable form. **Always write the
   block-list form.** Filed, PROJ-013 STAGE-047.
8. **PROJ-013 is invisible to `just backlog`, including `--all`** — 54 items, because the project is
   still `proposed`. Anything filed there is git-safe and framework-invisible.

9. **⚡ A CHECK REMOVED AS "REDUNDANT" MUST BE RE-PROVEN AT THE TYPE IT GUARDS.** SPEC-131's build
   moved range checks from the typed constructors into a schema and deleted the constructors' own
   checks as redundant. The schema checked the f64; the ops read an f32; `1e-50 > 0` passed and
   read as `0.0` — a silent 1×1 image / unmarked output, exit 0. None of the build's six negative
   controls could see it (each reverted a check that still existed); the retroactive verify's
   own probes did. **When a check moves across a type boundary, add a test row just past the
   boundary of the type the code actually reads** (here: below f32's smallest positive value).

10. **⚡ A HELPER THAT DERIVES A VALUE NEEDS AN INPUT WHERE THE DERIVED AND NAIVE VALUES DIFFER.**
    SPEC-134's fix added `resolved_input_count`, but every N-input row passed literal paths, so
    swapping it for `inputs.len()` stayed green — a second fix + re-verify. SPEC-133 repeated the
    shape: AC-5's `opacity = 0.01` never rounded an alpha to 0, and a resize-then-watermark test
    whose mark fit neither size could not tell the input from the image entering the step. **Every
    build or fix prompt that adds or relies on a derived value (a count, a resolved list, the image
    a step sees) asks for one test row where it differs from the naive value, and a mutation that
    swaps the two.** Trap 9 is the same lesson for a removed check.
    **Formatted output is a derived value too.** In SPEC-135 every message-row value (`nan`, `1025`,
    `1.5`) was exact in `f32`, so printing the value widened to `f64` (`got 1.100000023841858`)
    passed every test. A punch list and a re-verify followed. A test of printed or formatted
    output uses at least one value whose naive and correct renderings differ: an `f32`-inexact
    number such as `1.1`, or one past a cap by less than one step of the narrower type.
11. **⚡ A TEST THAT CAN SKIP MUST FAIL WHERE ITS COVERAGE IS REQUIRED; A PRINTED SKIP IS
    INVISIBLE IN CI.** SPEC-137's system-font row printed a SKIP line on a Linux host with no
    font, and its design asked the build to quote that line from the CI log. libtest hides stderr
    for passing tests, so neither the SKIP line nor the `using <font>` line ever reached the log.
    The build inferred coverage from green CI, and only verify caught it, which cost a punch
    list. **A design never asks for a log line without saying how it reaches the log**
    (`--show-output` on a dedicated step). **A row that may skip locally fails when an
    environment variable says its coverage is required**, and CI sets that variable on the step
    that must prove it ran (`CRUSTYIMG_REQUIRE_SYSTEM_FONT`).
12. **⚡ A MUTATION ROW NAMES WHERE IT IS APPLIED, AND EVERY TEST EXPECTED RED, ON BOTH PATHS.**
    SPEC-138's row "move the guard after the manifest loads" could be applied two ways. The build
    moved it into `run_build` only, which is off the watch path, and FT-11 and FT-12 went red. Verify
    applied it on both paths, and FT-8, FT-9 and FT-12 went red while FT-11 stayed green. Neither
    result matched the table. **A design names the code location for each mutation and lists every
    test expected red, including tests on the other path** (plain vs `--watch`, native vs wasm).
13. **⚡ A RED ROW IS RED FOR ITS REASON ONLY IF ITS FAILURE MESSAGE SAYS SO, AND A GOLDEN GUARDS
    THE BYTES ONLY IF ONE MUTATION TURNS IT RED BY HASH.** SPEC-143's FT-11 and FT-12 failed on
    `main` because their small bases tripped the 5 % coverage cap, not because the mark drew
    nothing. The build recorded them as RED, and FT-12 as "returned Ok", which was false. Its golden
    test (FT-6) went red under every recorded mutation, but only because the mark was refused first.
    `opacity × 0.999` on the 16-bit composite line stayed green. **The baseline quotes each RED row's
    failure message, and the prompt names the likely wrong-reason message** (here, the coverage
    cap). **A golden needs at least one mutation that goes red on a hash mismatch**, not on a
    refusal or an exit code.
14. **⚡ A TEST FOR A FIXED PATH CALLS THE PRODUCTION FUNCTION, AND THE OLD CODE PUT BACK VERBATIM
    TURNS IT RED.** SPEC-141's first punch list fixed a missing-golden panic in `golden_path_for`,
    and its new test (FT-9) drove the report builder through a closure that never called it. The
    re-verify put the old line back unchanged: all 9 tests stayed green. A second punch list and a
    second re-verify paid for it. **The control for a fix is the old code restored exactly**, not a
    new mutation. **A bless or golden guard's design also drives the next user's path:** add a new
    case by the README alone and record where it first goes green. That one row would have found
    SPEC-141's missing dump and unnamed literals at design, not over two verify rounds.

## What is working, and worth keeping

- **Read the code before writing the spec.** SPEC-128's backlog title was wrong — `Watermark` was
  already an `Operation`, deliberately unregistered. A design cycle that only read the item would
  have specced the wrong thing.
- **Ask what the user actually sees.** "What would this look like in a TOML file?" found a defect
  two design reviews had missed (text mode wrote the *text* under the `image` key).
- **Verify earns its cost every single time — five waves running.** It has now refuted an
  orchestrator hypothesis, established the repo's cost method, caught an unachievable AC, a
  non-discriminating control, and a half-tested mechanism. **SPEC-128's verify BUILT the alternative
  it was asked about** rather than arguing for it, which is what turned a judgement call into a
  decision.
- **Reserve DEC ids in the prompt.** `next_id` scans only the working tree. **Highest on `main` is
  DEC-101 (2026-10-24).** Reserved for the 0.8.0 wave: SPEC-131 typed params / DEC-102 ·
  SPEC-132 `build --watch` assets / DEC-103 (released unused: DEC-060 amended instead) · SPEC-133 watermark damage rule / DEC-104 ·
  SPEC-134 `--name-template` pins format / DEC-105 (EXTENDS DEC-087 to every verb; DEC-087 already makes `build`'s template the format pin, and the "names the file only" rule lived only in `docs/api-contract.md`). SPEC-135 `watermark` non-finite size/scale + text-size cap / DEC-106 (STAGE-054; gates 0.8.0). Same for SPEC ids, which
  `just new-spec` also allocates from the working tree.
- **Size the VERIFICATION, not just the code.** SPEC-127 was marked M off its implementation while
  its AC matrix was L-shaped; the builder said so, and was right.
- **Inline the cost-measurement rule in the dispatch prompt** — do not reference `cost-snippet.md`.
  SPEC-127's build had to reconstruct its cost post-hoc because the prompt only pointed at it.
