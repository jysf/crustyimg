---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-157
  type: bug                        # epic | story | task | bug | chore
  cycle: design  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-063
repo:
  id: crustyimg

agents:
  architect: claude-opus-4-7
  implementer: claude-opus-4-7     # usually same Claude, different session
  created_at: 2026-11-22

references:
  decisions:
    - DEC-035
    - DEC-057
    - DEC-059
    - DEC-060
    - DEC-061
  constraints:
    - untrusted-input-hardening
  related_specs:
    - SPEC-065
    - SPEC-066
    - SPEC-067
    - SPEC-068
    - SPEC-154
    - SPEC-158
  external:
    - docs/reviews/2026-11-10-external-code-review.md

# One sentence on what this spec contributes to its stage's
# value_contribution. For plumbing: "infrastructure enabling
# STAGE-063's <capability>". Optional; null is acceptable.
value_link: "STAGE-063's success criterion that `build` never overwrites a declared source and never records two outputs for one file"

# Self-reported AI cost per cycle. Each cycle (design, build, verify,
# ship) appends one entry to sessions[]. Totals are computed at ship.
# Record a REAL tokens_total for metered cycles (build/verify): the
# orchestrator fills it from the Agent result's subagent_tokens at ship
# (or /cost interactively). Only un-metered cycles (design/ship main-loop)
# may be null-with-note. `just cost-audit` enforces this on shipped specs.
# See AGENTS.md §4 and docs/cost-tracking.md. interface: claude-code |
# claude-ai | api | ollama | other.
cost:
  sessions:
    - cycle: frame
      date: 2026-11-22
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 7131617
      tokens_breakdown:
        input: 104
        cache_creation: 153369
        cache_read: 6939542
        output: 38602
      estimated_usd: 5.39
      note: >
        Measured from the session JSONL, found by content ("SPEC-157 — FRAME prompt"), deduped by
        message.id (52 messages; max output_tokens per id), 0 subagent transcripts. Measured after the
        push, before this cost commit's own turns.
        Rates (Opus): $5/MTok input, $6.25 cache_creation (x1.25), $0.5 cache_read
        (x0.10), $25 output.
  totals:
    tokens_total: 0
    estimated_usd: 0
    session_count: 0
---

# SPEC-157: build destinations have one identity

## Context

STAGE-063 entry SPEC-157 (reserved 2026-11-10), from two P1 findings in
`docs/reviews/2026-11-10-external-code-review.md`. PROJ-011's last stage holds 0.8.3 for it.

`build` decides where each output goes from the *spelling* of `out`. The pre-write collision key
(`output_collision_key`, `src/cli/build.rs:376`) only drops `.` components, and the post-write check
(`lock_output_path`, `:473`) compares the same spellings after the files are written. The sink then
canonicalizes the directory (`safe_join`, `src/sink/mod.rs:455`), so two spellings of one directory
reach one file. Nothing compares a destination with what the build reads. A user who points `out`
at the source folder, or at the recipe's own file name, loses that file and gets exit 0.

## Goal

Every `build` destination is resolved once, before anything is written, to one identity that the
planner, the collision check, the sink, the lockfile and the `--watch` exclusion all use. A build
whose destinations collide under that identity, or whose destination is a file the build reads, is
refused at planning with exit 2 and writes nothing.

## Frame

**Verdict: GO**, one spec, [M]. Needs a design cycle and a new DEC (content below). Its build
starts after SPEC-154 has merged.

### Premise check (driven)

Release build of `main` at `ff3c3fc` (`crustyimg 0.8.2`), fresh `CARGO_TARGET_DIR` in the session
scratchpad, macOS arm64, APFS (case-insensitive). Recipe `r.toml` is a `resize mode = "exact"` to
1×1 (`r2.toml`: 2×2); sources are 2×2 PNGs. Every case below is a fresh temp tree.

- **P1-a, `out` is the source folder: reproduced.** Manifest:
  `source = "src/photo.png"`, `recipe = "r.toml"`, `out = "src"`. Before: `src/photo.png 2x2`.
  `crustyimg build` → **exit 0**, `built 1 target, 1 output (0 cached, 1 rebuilt)`. After:
  `src/photo.png 1x1`. Lock: `path = "src/photo.png"`, `source = "src/photo.png"`.
  `crustyimg build --check` → **exit 7**, `drift: "src/photo.png": inputs changed (key 07515fd2e306 → c5624d3e8aa8)`.
- **P1-b, aliased directory: reproduced, and worse than reported.** Targets
  `{source = "src/result.png", recipe = "r.toml", out = "dist"}` and
  `{source = "src2/result.png", recipe = "r2.toml", out = "alias/../dist"}`, with `alias/`
  existing. `build` → **exit 0**, `built 2 targets, 2 outputs`. Files: one `dist/result.png`
  (2×2, sha256 `0f7765…`). Lock: two records, `alias/../dist/result.png` hash `0f7765…` and
  `dist/result.png` hash `9ba3d6…`, so **the record for the real path names a hash the file does not
  have**. `build --check` straight after → **exit 0**, `lockfile crustyimg.build.lock is up to date`.
  The review's precondition is not needed: with **no** `alias/` directory the result is the same
  (exit 0, two records), because `create_dir_all` makes `alias/`.

Neighbours (same setup, one change each):

- `./dist`, `dist/`, `dist/.`, `dist//` against `dist`: **caught**, exit 2, `output collision: "dist/result.{ext}" written by both …`.
  The lexical `CurDir` filter works; only `..` and links slip past.
- `out = "DIST"` against `out = "dist"`: **exit 0**, one directory `dist/`, lock records `DIST/result.png` and `dist/result.png`.
- One target, sources `src/result.png` and `src2/Result.png`, `out = "dist"`: **exit 0**, one file
  `dist/Result.png`, lock records both `dist/Result.png` and `dist/result.png` (same hash, so it hides).
- `out = "link"` with `link → dist` (in tree), against `out = "dist"`: **exit 0**, one file, two records.
- `out = "link/x"` with `link → ../outside`: **exit 0**, the PNG is written to `../outside/x/result.png`
  (DEC-061's accepted residual, reproduced).
- `..` past a symlink: `link → deep/a`, `out = "link/../dist"` against `out = "deep/dist"`: **exit 0**,
  one file `deep/dist/result.png`, two records. The kernel resolves `link/..` to `deep`; a lexical
  clean would say `dist`. **So a purely lexical identity is wrong here, not just incomplete.**
- `out = "."` beside `photo.png`: exit 0, `photo.png` 2×2 → 1×1 (`.` is an accepted `out`).
- Across targets: target 0 `src/a.png → dist`, target 1 `dist/a.png → src` with `name = "a.{ext}"`.
  From a clean tree: exit 3 (`input not found or unreadable: dist/a.png`), because sources resolve
  before anything is written. With `dist/a.png` present from an earlier run: **exit 0**, and target 1
  overwrote target 0's source `src/a.png` (2×2 → 1×1); `--check` then exits 7.
- Watermark asset: recipe `watermark image = "logo.png"`, `out = "."`, `name = "logo.{ext}"`:
  **exit 0**, `logo.png` sha1 `f675cd…` → `f6fdbe…` (the asset is replaced by the output).
- Recipe file: `out = "."`, `name = "r.toml"`: **exit 0**, `r.toml` now starts `89 50 4e 47` (a PNG).
  The next build fails to parse its recipe.
- `--watch` with `source = "src/*.png"`, `out = "outlink"`, `outlink → src/gen`: **19 builds in 4 s**
  (1 rebuilt, 18 cached) with no edits. The exclusion is `lexical_clean(out)` (`src/build/watch.rs:119`)
  and the watcher reports the real path, so the build's own writes wake it. An unbounded loop.
- `--watch` with `out = "src"` (output inside a watched source root, distinct name): no loop (the
  `out` exclusion covers it). One driven edit of `src/a.png` did produce a rebuild, but its summary
  read `1 cached`, so whether a source edit inside an excluded `out` is ever missed is **not settled**;
  design drives it.

### Answers

1. **Premise:** both findings reproduce on `main` (above). The class is wider than the review: `..`,
   links, case, an `out` of `.`, recipe files, watermark assets, other targets' sources, and a
   `--watch` loop.
2. **One identity.** Recommend `resolve_destination(out, file_name)`: (a) lexically drop `.`
   components; (b) walk from the full path to the longest prefix that **exists**, and
   `fs::canonicalize` it (this resolves links, `..` after a link the way the kernel does, and on
   macOS returns the on-disk case: `realpath DIST` printed `…/dist`); (c) join the non-existent
   tail after a lexical `..`/`.` clean (safe: a component that does not exist cannot be a link, and
   `create_dir_all` builds exactly that path); (d) strip the canonical build root (canonicalized cwd,
   DEC-057's root) to get a root-relative identity, rendered with `/`. Case: the existing part takes
   its on-disk case from (b); the tail cannot be known, so **collision and overlap compare a
   case-folded key** on every OS (over-detection, the same safe direction as `EXT_SENTINEL`; a
   manifest relying on `a.png` vs `A.png` breaks on a macOS or Windows checkout anyway). Windows:
   `canonicalize` returns `\\?\C:\…`; since the root is canonicalized the same way, step (d)'s
   `strip_prefix` is between like forms, and drive-letter case is normalized by the OS call. Design
   must confirm both on the Windows leg, not by reading. Used by: planning (`prepare_target` /
   `check_output_injective`), the overlap pass, the sink (write to the resolved path, so the plan and
   the write cannot disagree), the lockfile `path` (the root-relative identity), and the `--watch`
   exclusion.
3. **Source overlap.** A destination overlaps when its identity equals the identity of any file the
   build reads: any resolved input of **any** target, any target's recipe file, any recipe asset
   (`registry.asset_keys`, the same list `recipe_asset_paths` builds, resolved with
   `resolve_recipe_assets`'s resolver), the manifest, and the lockfile; or when it lies under
   `.crustyimg/`. That also refuses chaining (one target's output as another's source), which today
   fails from a clean tree with exit 3 and silently overwrites on a warm one: a maintainer question
   below. An `out` *directory* inside a source root is **not** refused (`photos/` → `photos/web/` is a
   normal layout; directory sources are non-recursive, DEC-010). `--watch`: overlap alone does not
   close the driven loop (the loop writes to no source, it writes under a watched root through
   another spelling); excluding the **resolved** `out` does. Fold that in.
4. **Refusal.** At planning, after every target is prepared and before the lockfile load, cache open
   or any write, the same place as `check_output_injective`. **Exit 2**: the existing
   `OutputCollision` and `..`-escape (`CliError::Build`) are exit 2, and both are manifest errors
   found before execution; exit 5 stays the sink's last-line refusal. Messages (design fixes the
   wording, plain, no ids):
   - `output collision: "dist/result.{ext}" is written by target #0 (src/result.png) and target #1 (src2/result.png, out "alias/../dist")`
   - `target #1 would overwrite "src/a.png", which target #0 reads as a source`
   - `target #0 would overwrite "r.toml", the recipe of target #0` (and `a watermark image of …`)
   - `target #0's out "link/x" resolves outside the build tree (through the link "link")` (if ruled so).
   DEC-106's rule (check before any file is read) is the precedent for "refuse before work".
5. **Symlinked output directories.** Options:
   - *Keep accepting (DEC-061 status quo).* Breaks nothing today; keeps the out-of-tree write
     (reproduced), and still needs the resolved identity for in-tree link aliases and the watch loop.
   - *Refuse any link in the `out` path.* Simplest rule; breaks in-tree conveniences (`dist → build/dist`)
     and every out-of-tree target (`dist → ramdisk`) with no workaround, since an absolute `out` is refused.
   - *Accept a link only when the resolved `out` stays inside the canonical build root.* Closes the
     reviewer's write and is the fix DEC-061 itself names (STAGE-024's filed item); in-tree links keep
     working and are de-aliased by the identity. Breaks `dist → /Volumes/ramdisk`; that user's
     workaround is to build in-tree and copy, or a later opt-in.
   **Recommend the third.** New DEC content: (i) a build destination's identity is the resolution in
   answer 2, computed once at planning and used by planning, sink, lockfile and watch; (ii)
   collisions and overlaps compare the case-folded identity; (iii) a destination may not be a file the
   build reads (the list in answer 3) nor under `.crustyimg/`; (iv) an `out` that resolves outside the
   canonical build root is refused, exit 2, closing DEC-061's symlink residual (amend DEC-061's
   "Residual (symlink)" and accepted-risk 3 to point at the new DEC); (v) the lockfile records the
   root-relative identity, so `./dist`, `dist/` and `a/../dist` all lock as `dist/…` (a lockfile written
   with an odd spelling drifts once, on paths only); (vi) DEC-035's per-file symlink refusal stays as
   the sink's second layer. Its `affected_scope` (block-list form) includes `src/cli/build.rs`,
   `src/build/mod.rs`, `src/build/watch.rs`, `src/sink/mod.rs`, `docs/cli-reference.md`, `SECURITY.md`.
6. **`--check` and the lockfile.** Driven: `--check` runs the whole build. It **writes every output
   and repopulates the cache** (an output deleted and `.crustyimg/` removed before `--check` were both
   back after it, exit 0), it **never writes the lockfile** (sha1 unchanged), and it compares the
   keys and hashes of *the bytes this run produced* with the lockfile. It never reads the outputs
   already on disk, which is why P1-b's `--check` passed while `dist/result.png` disagreed with its
   record. `docs/cli-reference.md` says only "re-runs the build … never modifies the lockfile". This
   spec documents what it writes. Making `--check` read-only, or comparing the files on disk, is a
   behaviour change: filed separately (below). After the fix, the lockfile holds **one** record per
   destination, keyed by the root-relative identity.
7. **Split or not: no split.** Overlap and alias need the same resolver (`out = "./src"` or a link
   to `src` is only an overlap once resolved), so splitting them builds the resolver twice. Fold the
   watch exclusion (a call-site change once the resolver exists). From SPEC-158 fold **nothing**: the
   source snapshot (hash one read, decode the same bytes), `LOCK_MAX_BYTES` and the `encode_one`
   clone are about *bytes*, not *paths*; they touch `cache_key_for`/`encode_one`, not the planner.
   The boundary: SPEC-157 owns "where", SPEC-158 owns "what". Order: SPEC-157 first, so SPEC-158's
   snapshot hangs off the already-resolved plan.
8. **Tests.** Integration on a temp tree (every OS leg): P1-a, P1-b with and without `alias/`, the
   four `CurDir` spellings (regression), `.`-as-`out`, cross-target source, recipe file, watermark
   asset (image and font, two rows), manifest and lockfile names, `.crustyimg/` path, `dist` vs `DIST`
   and `a.png` vs `A.png` (case-folded, so they assert refusal on Linux too), the lockfile path for
   `./dist` and `a/../dist`, a refused build leaves every file byte-identical and writes no lock.
   `#[cfg(unix)]` (links): in-tree link alias, `..` past a link, out-of-tree link refused, in-tree
   link accepted. `--watch` (`tests/build_watch.rs`, unix): a linked `out` under a watched root does
   not rebuild itself (count summaries in a window). Windows CI only: resolver unit rows for
   `dist\` vs `dist`, and the root-prefix strip under `\\?\`. A case-insensitive volume is the macOS
   dev box and the Windows leg (NTFS); the case-fold rule makes the case rows meaningful on Linux too.
   Mutation rows (each one edit; the red set is read off a full `cargo test --no-fail-fast` of every
   target, AGENTS §15):
   - M1, resolver skips step (b) (lexical only) → in-tree link alias and `..`-past-link go red.
   - M2, step (c) keeps `..` in the tail → P1-b without `alias/` goes red.
   - M3, drop the case fold → `dist`/`DIST` and `a.png`/`A.png` go red.
   - M4, overlap ignores asset paths → watermark image and font rows go red.
   - M5, overlap ignores recipe files → recipe row goes red.
   - M6, overlap compares only within one target → cross-target row goes red.
   - M7, overlap runs after phase 2 → "refused build leaves files byte-identical" goes red.
   - M8, lock `path` uses `target.out` spelling → `./dist`/`a/../dist` lock-path rows go red.
   - M9, watch exclusion keeps `lexical_clean(out)` → the watch-loop row goes red.
   - M10, drop the root-containment check → the out-of-tree link row goes red.
9. **Docs.** There is no `docs/build.md`; `build` is documented in `docs/cli-reference.md` (§build,
   ~lines 375–465), which says nothing about `out` resolution, collisions, overlaps or links, and
   describes `--check` without saying it writes outputs. After: one paragraph on `out` (resolved
   inside the build tree; two targets resolving to one file, or an output that is a file the build
   reads, are refused with exit 2 before anything is written; a link in `out` must resolve inside the
   tree), and the `--check` paragraph states it writes outputs and the cache, compares the bytes it
   produced, and never writes the lockfile. `SECURITY.md` (:42–46, the table row :87) gains the
   out-dir containment. `skills/crustyimg/SKILL.md:335` (exit 5 row) is unchanged unless design
   picks exit 5. `README.md`/`docs/USAGE.md:165` mention only the per-file symlink refusal: unchanged.
10. **Fold-ins.** None of the open STAGE-063 entries fit: the `build_watch` timeout diagnosis is a
    different symptom (tests timing out waiting for a rebuild, not an extra rebuild); leave it, but
    its diagnoser should know about this loop. Watermark strict mode, SPEC-148/150 leftovers,
    `optimize` stdout, finding A and `docs/recipes.md:148` are unrelated. Outside the stage, PROJ-007
    STAGE-024's "canonicalize-contain the `out` dir" item is this spec's answer 5; marked superseded.
11. **Size: M, confirmed.** The code is one resolver plus five call sites and one overlap pass. The
    cost is verification: four axes (link, `..`, case, cross-target/asset), three OS legs where two
    are case-insensitive and one only CI can run, the watch loop, a DEC that amends DEC-061, and a
    lockfile path change. If design finds the sink cannot take a pre-resolved path without changing
    `write_encoded`/`Sink::Dir` for `apply` too, split the watch exclusion into its own [S].

### Scope

- **In:** the resolver; collision and overlap at planning on the resolved, case-folded identity;
  symlinked-`out` containment (per the ruling); the sink writing the resolved path; one lock record
  per destination with the root-relative path; `--watch` excluding the resolved `out`; the new DEC
  and DEC-061's amendment (design writes them); `docs/cli-reference.md` and `SECURITY.md`.
- **Out:** everything in SPEC-158; making `--check` read-only or comparing on-disk outputs (filed);
  `apply`/single-verb `-o`/`--out-dir` resolution; an opt-in for out-of-tree output links; pruning
  orphaned outputs.

### Rough build file list

- `src/cli/build.rs`: resolver call sites, overlap pass, `lock_output_path`, `output_collision_key`.
- `src/build/mod.rs`: `find_output_collision` on folded keys; maybe the resolver itself.
- `src/build/watch.rs`: `watch_roots` exclusion on the resolved `out`.
- `src/cli/mod.rs`: new error variant(s) and their exit code.
- `src/cli/common.rs` / `src/sink/mod.rs`: only if the sink takes the resolved path.
- `tests/build_destinations.rs` (new), `tests/build_watch.rs`, unit rows in `src/cli/build.rs`.
- `docs/cli-reference.md`, `SECURITY.md`, `decisions/` (new DEC, DEC-061 amendment), `CHANGELOG.md`.

### Maintainer questions

- Symlinked `out` that resolves outside the tree: refuse? **Recommend refuse** (third option).
- Chaining (one target's output as another's source): refuse? **Recommend refuse**; it already fails
  from a clean tree (exit 3).
- Case: fold on every OS, refusing `a.png` + `A.png` even on Linux? **Recommend yes.**
- Exit code for the new refusals: 2 (manifest error) or 5 (write refused)? **Recommend 2.**
- Lockfiles with odd `out` spellings drift once on paths after upgrade: CHANGELOG note enough?
  **Recommend yes.**

## Inputs

- **Files to read:** `path/to/file.ext` — why
- **External APIs:** <name, docs link, auth>
- **Related code paths:** `src/some/module/`

## Outputs

- **Files created:** `path/to/new.ext` — purpose
- **Files modified:** `path/to/existing.ext` — what changes
- **New exports:** <names and signatures>
- **Database changes:** <migrations>

## Acceptance Criteria

Testable outcomes. Cover happy path, error cases, edge cases.

- [ ] Criterion 1 (testable)
- [ ] Criterion 2 (testable)

## Failing Tests

Written during **design**, BEFORE build. The implementer's job in
**build** is to make these pass.

- **`path/to/test.file`**
  - `"test description 1"` — asserts: ...

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Decisions that apply

- `DEC-NNN` — <one-line summary of why this matters here>
- `DEC-MMM` — <one-line summary>

### Constraints that apply

These constraints apply to the paths touched by this task (see
`/guidance/constraints.yaml` for full text):

- `constraint-id-1` — <one-line summary>
- `constraint-id-2` — <one-line summary>

### Prior related work

- `SPEC-YYY` (shipped) — <one-line summary, if relevant>
- `PR #NNN` — <link, if relevant>

### Out of scope (for this spec specifically)

Explicit list of what this spec does NOT include. If any of these feel
necessary during build, create a new spec rather than expanding this one.

- SPEC-158's items: hashing one read and decoding another, the lockfile size contract, the
  `encode_one` clone.
- Making `build --check` read-only, or comparing the outputs on disk (filed in STAGE-063).
- Destination resolution for `apply` and the single-file verbs (`-o`, `--out-dir`).
- An opt-in for `out` links that resolve outside the build tree.
- Pruning orphaned outputs (PROJ-007 STAGE-024 item).

## Notes for the Implementer

Gotchas, style preferences, reuse opportunities.

---

## Build Completion

*Filled in at the end of the **build** cycle, before advancing to verify.*

- **Branch:**
- **PR (if applicable):**
- **All acceptance criteria met?** yes/no
- **New decisions emitted:**
  - `DEC-NNN` — <title> (if any)
- **Files this diff touches** — **list every one**, with a phrase on what changed
  in each. Not "src/foo and tests": the enumeration is what a verify cycle diffs
  the real change against, and what tells you whether every affected `DEC-*` has
  the path in its `affected_scope`. SPEC-122's Deviations claimed "`src/operation`
  and `tests/` only" and was wrong by two `scripts/` files, which left
  `affected_scope` blind; SPEC-124's build added this section by hand after that
  lesson and still listed 8 of its 9 files. It is a checklist because it is easy
  to get wrong from memory — build it from `git diff --name-only`, not recall.
  - `path/to/file` — <what changed>
- **Deviations from spec:**
  - [list]
- **Follow-up work identified:**
  - [any new specs for the stage's backlog]

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — <answer>

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — <answer>

3. **If you did this task again, what would you do differently?**
   — <answer>

---

## Reflection (Ship)

*Appended during the **ship** cycle. Outcome-focused reflection, distinct
from the process-focused build reflection above.*

1. **What would I do differently next time?**
   — <answer>

2. **Does any template, constraint, or decision need updating?**
   — <answer>

3. **Is there a follow-up spec I should write now before I forget?**
   — <answer>
