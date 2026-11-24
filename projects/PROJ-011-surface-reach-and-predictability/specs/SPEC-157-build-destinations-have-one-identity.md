---
# Maps to ContextCore task.* semantic conventions.
# This variant assumes Claude plays every role. The context normally
# in a separate handoff doc lives in the ## Implementation Context
# section below.

task:
  id: SPEC-157
  type: bug                        # epic | story | task | bug | chore
  cycle: verify  # frame | design | build | verify | ship
  blocked: false
  priority: high
  complexity: M                    # S | M | L  (L means split it)

project:
  id: PROJ-011
  stage: STAGE-063
repo:
  id: crustyimg

agents:
  architect: claude-opus-5-5
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
    - cycle: design
      date: 2026-11-22
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 19146877
      tokens_breakdown:
        input: 188
        cache_creation: 259277
        cache_read: 18773662
        output: 113750
      estimated_usd: 13.85
      note: >
        Measured from the session JSONL, found by content ("SPEC-157 — DESIGN prompt"), deduped by
        message.id (94 messages; max output_tokens per id), 0 subagent transcripts. Measured after the
        second push, before this cost commit's own turns. Most of the wall time was 16 full
        `cargo test --no-fail-fast` runs for the mutation table (local CPU, not tokens).
        Rates (Opus): $5/MTok input, $6.25 cache_creation (x1.25), $0.5 cache_read
        (x0.10), $25 output.
    - cycle: build
      date: 2026-11-23
      model: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 13458758
      tokens_breakdown:
        input: 150
        cache_creation: 218799
        cache_read: 13168879
        output: 70930
      estimated_usd: 5.84
      note: >
        Measured from the session JSONL, found by content ("SPEC-157 — BUILD prompt"), deduped by
        message.id (70 messages; max output_tokens per id), 0 subagent transcripts. Measured before
        the final advance-cycle commit's own turns. Wall time was mostly 17 full
        `cargo test --no-fail-fast` mutation runs (local CPU, not tokens).
        Rates (Sonnet): $3/MTok input, $3.75 cache_creation (x1.25), $0.3 cache_read
        (x0.10), $15 output.
    - cycle: verify
      date: 2026-11-23
      model: claude-opus-5-5
      interface: claude-code
      tokens_total: 14331153
      tokens_breakdown:
        input: 186
        cache_creation: 429830
        cache_read: 13797124
        output: 104013
      estimated_usd: 12.19
      note: >
        Figures handed over by the orchestrator in the punch-list prompt, not re-measured here.
        Rates (Opus): $5/MTok input, $6.25 cache_creation (x1.25), $0.5 cache_read
        (x0.10), $25 output.
    - cycle: punchlist
      date: 2026-11-23
      model: claude-sonnet-5-5
      interface: claude-code
      tokens_total: 9223618
      tokens_breakdown:
        input: 146
        cache_creation: 157061
        cache_read: 9013842
        output: 52569
      estimated_usd: 4.08
      note: >
        Measured from the session JSONL, found by content ("SPEC-157 — PUNCH LIST prompt"), deduped by
        message.id, max output_tokens per id. Measured before the final spec commit and the CI wait, so
        the true total is slightly higher. Wall time was mostly cold builds: one mutation series run
        twice (the first run shared a target dir with a second copy of the script and was discarded),
        three fresh-target matrix legs and `just wasm-check`.
        Rates (Sonnet): $3/MTok input, $3.75 cache_creation (x1.25), $0.3 cache_read
        (x0.10), $15 output.
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

### Maintainer rulings (2026-11-22)

All five recommendations accepted:
1. **A symlinked `out` that resolves outside the build root is refused** (the third option). In-tree
   links keep working and are de-aliased. The message says why and that the workaround is to build
   in the tree and copy.
2. **Chaining is refused:** one target's output may not be another target's source.
3. **Collision and overlap compare a case-folded identity on every OS,** so `a.png` and `A.png` are
   refused on Linux too.
4. **The new refusals exit 2,** like the existing collision and `..`-escape errors.
5. **A CHANGELOG note covers lockfiles with odd `out` spellings** that report path drift once after
   the upgrade.

Build order (orchestrator): SPEC-155 and SPEC-157 both change `src/cli/build.rs` and maybe
`src/sink/mod.rs`, so their builds run one after the other, SPEC-155 first. SPEC-157's sink change
sits on SPEC-155's staged-write helper.

## Design

Designed 2026-11-22 (Opus), on `main` at `54fcc79`. Every claim below was driven on a release build of
`main` (`crustyimg 0.8.2`, fresh `CARGO_TARGET_DIR` in the session scratchpad, macOS arm64, APFS
case-insensitive) and on a never-pushed prototype of this design, unless marked *reasoned*.

### Design corrections to the frame

1. **The resolver resolves the directory, not the file.** The frame's `resolve_destination(out, file_name)`
   would canonicalize an existing output file too, which follows a link planted at the file name. The
   design resolves only `out`, then joins the expanded file name as one lexical component (the sink's
   `safe_join` already refuses a name with a separator or `..`). A link at the file name stays DEC-035's
   job: refused by the sink, exit 5, even with `--yes`.
2. **Overlap cannot compare `{ext}` names as spelled.** `out = "src"` with the default `{stem}.{ext}` plans
   `src/a.{ext}`, which equals no file. The design compares an `{ext}` name against every extension the sink
   can write (`OUTPUT_EXTENSIONS`, the range of `sink::extension_for_format`), unless the target's format
   is pinned (literal extension or recipe `format`), in which case it compares only that extension.
   Over-detection, the same direction as `EXT_SENTINEL` (`src/cli/build.rs:350-367`).
3. **The collision message does not change.** The frame drafted a new wording. The existing
   `OutputCollision` text is pinned by `tests/build_injective.rs`; it already names both sources. Only its
   `output` changes, to the resolved root-relative path. Less churn, one fewer message to verify.
4. **No change to `src/sink/mod.rs` and none to `apply`.** `build` stops calling `write_encoded`
   (`src/cli/common.rs:139`, shared with `apply`) and calls a new `write_planned` in `src/cli/build.rs`,
   which creates the resolved directory, re-resolves it, and writes through `Sink::File`'s arm of
   `write_bytes` (`src/sink/mod.rs:642`). The frame's split trigger (the sink cannot take a resolved path
   without changing `apply`) does not fire.
5. **The `--watch` question is settled: a source edit inside an excluded `out` is missed, on `main` too.**
   Driven (below). It is caused by the `out` exclusion, so it is folded in.
6. **A residual the frame did not list: Unicode normalization.** Two sources `s1/café.png` (NFC) and
   `s2/café.png` (NFD) into one `out` on APFS: **exit 0, one file, two lock records** (driven on the
   prototype). Closing it needs Unicode normalization tables, which `std` lacks and nothing in
   `Cargo.lock` provides. Not folded: filed as a follow-up (below), named in the DEC.
7. **`src/build/watch.rs` does not change.** The resolved `out` is added to the exclusion set where the
   watch set is built in `src/cli/build.rs` (`watch_set_for`, `:1087`), which already touches the
   filesystem. `watch_roots` stays lexical and filesystem-free. `src/build/mod.rs` changes only a comment.

### What was driven

- **Case-folding rule.** APFS is case- and normalization-insensitive: writing `cafe\u0301.png` after
  `caf\u00e9.png` replaced the first file's bytes, and `ls` showed one name. **Rule: `str::to_lowercase()`**
  (std, the Unicode default lowercase mapping), applied to the whole path string, no normalization, no new
  dependency. It refuses on Linux what macOS and Windows would merge by case: `dist`/`DIST`,
  `a.png`/`A.png`, `é.png`/`É.png` (all three driven as refusals on the prototype). It over-refuses on Linux
  any pair of names that differ only in letter case. It does **not** catch NFC vs NFD (finding 6), and full
  case folding (`ß` vs `ss`) is not attempted (not driven; APFS and NTFS case rules were not measured for it).
- **The build root** is `std::env::current_dir()` canonicalized (DEC-057: manifest paths resolve against the
  working directory). Driven: a build run from `/tmp/...` (a link to `/private/tmp/...`) locks
  `path = "dist/a.png"` on `main` and on the prototype. On Unix `getcwd` already returns the physical path,
  so skipping the canonicalize is an equivalent mutant there (M11 below, green on every Unix row). On Windows
  `current_dir()` can carry an 8.3 short name (`RUNNER~1`) while `canonicalize` returns `\\?\C:\Users\runneradmin\…`,
  so the canonicalize is load-bearing there (*reasoned*; CI's Windows leg is the evidence).
- **The longest existing prefix** (`main`, then prototype):
  - `out` is a regular file (`out = "f"`): exit 6, `could not create output directory f: File exists`,
    nothing written. Unchanged by this spec.
  - `out` is a link to a file: the same, exit 6. Unchanged.
  - A dangling link mid-`out` (`dl → ../e3-outside/new`, `out = "dl/x"`): exit 6, nothing written anywhere.
    `canonicalize("dl")` fails, so the resolver treats `dl/x` as not existing and stays in-tree; the sink's
    `create_dir_all` then fails on the dangling link. If the link's target appeared before the write, the
    sink's re-resolution (below) refuses it.
  - `..` in the non-existent tail after a link (`lnk → deep/a`, `out = "lnk/new/../d"`): the kernel writes
    `deep/a/d/a.png` (driven on `main`), and the resolver gives `deep/a/d` (canonical `lnk` = `deep/a`,
    then `new/..` lexically; *reasoned* from the algorithm, not driven on the prototype). Lexical `..` is exact in the tail because a directory that does not exist
    cannot be a link.
- **Between planning and the write.** Planning resolves once. `write_planned` runs `create_dir_all` on the
  resolved directory, canonicalizes it again, and refuses the write (exit 5, `SinkError::Traversal`) when the
  folded result differs from the plan. Pinned by a unit row that turns the planned `dist` into a link to
  another temp dir after planning (prototype: refused, the other dir stays empty). DEC-035's per-file link
  refusal stays (`reject_symlink_destination` in the `Sink::File` arm). What a race can still do: swap a
  directory for a link after the re-check and before the open (or SPEC-155's rename). That needs a local
  process with write access to the tree racing the build; accepted, as today.
- **Every input is known before any work.** `prepare_target` (`src/cli/build.rs:279`) resolves each
  target's sources (globs and folders included, `source::resolve_all`, `:332`) and reads each recipe asset
  (`resolve_recipe_assets`, `src/cli/common.rs:254`) before `check_output_injective` runs (`:751`), and that
  is before the lockfile load (`:756`) and `Cache::open` (`:773`). The built-in watermark font is embedded
  bytes (`src/text/mod.rs:26`, `include_bytes!`), not a file, so it has nothing to overlap; a bundled recipe
  name is not a file either (`load_recipe`, `src/cli/common.rs:203`). Driven: target 0 writes `dist/a.png`,
  target 1 reads `dist/*.png`: refused on a warm tree; on a clean tree the glob is empty and the build stops
  at exit 3 (`no images matched 'dist/*.png'`), as on `main`.
- **Cache key and lockfile key.** `cache_key_for` (`:424`) hashes the version, features, recipe hash,
  quality, input extension and source bytes: **no output path**, so the cache is untouched and
  `CACHE_SCHEMA_VERSION` (`src/build/cache.rs:81`) stays 1. The lockfile's primary key is `path`
  (`src/build/lock.rs:144-153`); its schema does not change, only the value, so `SUPPORTED_LOCK_VERSION`
  (`:71`) stays 1. Which spellings move: `./dist` and `dist/` already lock as `dist/…` on `main`
  (`to_slash` drops `.`), so they do not drift. `a/../dist`, a link spelling and a case that differs from
  the directory on disk (`DIST` over `dist/`, prototype locks `dist/a.png`) drift once. Driven on an
  in-tree link: `main` wrote `path = "dist/a.png"`; the prototype's `--check` exits 7 with
  `drift: "build/dist/a.png": not in the lockfile` and `drift: "dist/a.png": in the lockfile, not built`;
  one plain `build` rewrites it, and `--check` then exits 0.
- **`--check` plans too.** A refused manifest under `--check`: exit 2, every file and the lockfile
  byte-identical (sha256 before and after), no `.crustyimg/` created. The refusal runs before `load_lock`.
- **`--watch`** (background watchers, 4–5 s lifetimes, PIDs killed; none left running):
  - linked `out` under a watched source root (`outlink → src/gen`, `source = "src/*.png"`): `main` **19 builds
    in 4 s**; prototype **1**.
  - source edit inside `out` (`source = "src/a.png"`, `out = "src"`, `name = "{stem}_w.{ext}"`, edit
    `src/a.png` after 2 s): `main` 2 builds, the second `(1 cached, 0 rebuilt)`, so it ran before the edit
    and the edit was **missed**; prototype with the exclusion only: 1 build, missed. With the fix: rebuilt.
  - control, source outside `out`: 2 builds, both `(0 cached, 1 rebuilt)`, on both binaries.
  The fix: an event inside an excluded directory still triggers when it names a file the build reads. That
  is safe because the build now refuses an output that is a file it reads, so its own writes never match.
- **Windows** (*reasoned*, CI only). `std::fs::canonicalize` on Windows returns the verbatim form
  (`\\?\C:\…`); the root is canonicalized the same way, so `starts_with`/`strip_prefix` compare like forms.
  NTFS is case-insensitive, so the case rows assert what the filesystem does. The Windows-only unit rows
  assert `resolve_out_dir(root, "dist\\")` equals `root.join("dist")` and that `root_relative` of it is
  `"dist"` under a `\\?\` root. The build reads the Windows leg's red set off its PR.
- **Order of errors.** First failure wins, in this order: manifest validation (`..`/absolute `out`,
  `Target::validate`, exit 2) → per target in manifest order, `prepare_target` (recipe, assets, sources:
  exit 1/2/3) → per target, out-of-tree link → collision over every output → overlap per output in target
  then input order (`.crustyimg/` first, then read files). Pinned by two rows: a collision and an overlap in
  one manifest reports the collision; a `..` escape and an overlap reports the escape.

## Inputs

- **Files to read:** `src/cli/build.rs` (`prepare_target` :279, `output_collision_key` :376,
  `check_output_injective` :393, `cache_key_for` :424, `to_slash` :458, `lock_output_path` :473,
  `lock_record` :489, `build_one` :548 with `write_encoded` calls at :572 and :625, `run_build` :738 with
  the check at :751, the post-write collision check :893-912, `recipe_asset_paths` :1064, `watch_set_for`
  :1087, `watch_impl` :1099 with the exclusion filter at :1132 and the refresh at :1146);
  `src/cli/common.rs` (`write_encoded` :139, `load_recipe` :203, `resolve_recipe_assets` :254);
  `src/sink/mod.rs` (`safe_join` :426, `write_bytes` :632, `Sink::File` arm :642); `src/cli/mod.rs`
  (`OutputCollision` :816, `code()` :885, `exit_code_mapping_is_total` :1416); `src/build/mod.rs`
  (`Target::validate` :249, `find_output_collision` :342); `src/build/watch.rs` (`watch_roots` :105,
  `is_excluded` :148); `tests/build_watch.rs` (the `spawn_watch` harness :95 and `poll_until` :130).
- **External APIs:** none.
- **Related code paths:** `src/cli/build.rs`, `src/cli/mod.rs`.

## Outputs

- **Files created:** `tests/build_destinations.rs`: every integration row below.
- **Files modified:**
  - `src/cli/build.rs`: the resolver, `check_destinations`, `write_planned`, the lock path, the watch
    exclusion and read carve-out, unit rows.
  - `src/cli/mod.rs`: three error variants, their `code()` arms, `exit_code_mapping_is_total`, the `build`
    and `--check` help text.
  - `src/build/mod.rs`: the `Target::validate` comment's "RESIDUAL" sentence (now closed).
  - `tests/build_watch.rs`: two watch rows.
  - `docs/cli-reference.md`, `SECURITY.md`, `skills/crustyimg/SKILL.md`, `CHANGELOG.md`.
  - `decisions/DEC-<next>-a-build-destination-has-one-identity-resolved-before-anything-is-written.md` (new)
    and `decisions/DEC-061-*.md` (amendment), both at **ship** (see Implementation Context).
- **New exports:** none (`pub(super)`/private only).
- **Database changes:** none.

## Acceptance Criteria

Each AC names its tests (FT ids under Failing Tests). Messages are printed by `run()` as `error: <message>`.

- [ ] **AC-1 `out` is the source's own folder: refused.** `source = "src/a.png"`, `out = "src"`: exit 2,
  `target #0 would overwrite "src/a.png", which target #0 reads as a source`, every file byte-identical, no
  lockfile, no `.crustyimg/`. (FT-1)
- [ ] **AC-2 `out = "."` beside its source: refused** with the same message for `a.png`. (FT-2)
- [ ] **AC-3 `..` aliases collide, with and without the intermediate folder.** Targets `dist` and
  `alias/../dist`: exit 2, `output collision: "dist/a.{ext}" written by both "src/a.png" and "src2/a.png" — two
  inputs map to one output (disambiguate the name template, e.g. {parent}_{stem})`, nothing written, and with
  no `alias/` beforehand none is created. (FT-3, FT-4)
- [ ] **AC-4 The four `.` spellings still collide** (`./dist`, `dist/`, `dist/.`, `dist//` against `dist`),
  same message. (FT-5)
- [ ] **AC-5 Letter case is ignored on every OS,** for a folder (`dist`/`DIST`), an ASCII file name
  (`a.png`/`A.png`) and a non-ASCII one (`é.png`/`É.png`): each is the collision message, exit 2. (FT-6,
  FT-7, FT-8)
- [ ] **AC-6 One target's output may not be another target's source,** named directly or found through a
  glob: exit 2, `target #0 would overwrite "dist/a.png", which target #1 reads as a source; one target's output
  cannot be another target's source`. (FT-9, FT-10)
- [ ] **AC-7 The recipe file is refused:** `target #0 would overwrite "r.toml", which is the recipe of target #0`.
  (FT-11)
- [ ] **AC-8 A watermark image is refused:** `… "logo.png", which target #0's recipe reads as its watermark image`.
  (FT-12)
- [ ] **AC-9 A watermark font is refused:** `… "go.ttf", which target #0's recipe reads as its watermark font`.
  (FT-13)
- [ ] **AC-10 The manifest is refused:** `… "crustyimg.build.toml", which is the build manifest`. (FT-14)
- [ ] **AC-11 The lockfile is refused:** `… "crustyimg.build.lock", which is the build lockfile`. (FT-15)
- [ ] **AC-12 `.crustyimg/` is refused:** `target #0 would write ".crustyimg/x/a.{ext}" inside .crustyimg/,
  the build's cache directory`. (FT-16)
- [ ] **AC-13 A refused build changes nothing,** including an existing lockfile, cache and outputs from an
  earlier build (FT-17), and under `--check` it refuses with the same exit 2 and message and writes nothing
  (FT-18). Every refusal row (FT-1 to FT-16, FT-19, FT-20, FT-23 to FT-25) also snapshots the whole tree
  before and after.
- [ ] **AC-14 Order of errors:** a collision is reported before an overlap (FT-19); a `..` escape before an
  overlap (FT-20).
- [ ] **AC-15 The lockfile records the root-relative path:** `out = "./dist/"` and `out = "a/../dist"` lock as
  `dist/a.png` and `dist/b.png`, and no path holds `..` (FT-21); an in-tree link locks the real folder
  (FT-26); a build started through a linked working directory locks `dist/a.png` (FT-27).
- [ ] **AC-16 An older lockfile drifts once, on paths only:** a lock naming `a/../dist/a.png` gives `--check`
  exit 7 with `drift: "dist/a.png": not in the lockfile` and `drift: "a/../dist/a.png": in the lockfile, not
  built`; one plain `build` rewrites it to the same bytes as a fresh build; `--check` then exits 0. (FT-22)
- [ ] **AC-17 Links (Unix):** an in-tree link alias collides (FT-23); `..` past a link collides on the
  kernel's path, `deep/dist/a.{ext}` (FT-24); an `out` that resolves outside the tree through a link is refused
  with exit 2, `target #0's out "link/x" resolves outside the build tree through the link "link"; build inside
  the tree and copy the outputs where you need them`, and nothing is created inside or outside (FT-25); an
  in-tree link builds (FT-26).
- [ ] **AC-18 The sink re-resolves the folder before writing:** a planned folder that became a link
  elsewhere is refused and the other folder stays empty (FT-28, unit, Unix).
- [ ] **AC-19 `--watch` (Unix):** a linked `out` under a watched source root builds once in a 3 s quiet
  window, not in a loop (FT-29); an edit to a source inside an `out` folder rebuilds it (FT-30).
- [ ] **AC-20 Exit 2 for every new refusal;** `exit_code_mapping_is_total` lists each new variant. (FT-31)
- [ ] **AC-21 The case fold covers non-ASCII letters** and `OUTPUT_EXTENSIONS` covers every extension
  `extension_for_format` returns. (FT-32, FT-33)
- [ ] **AC-22 Windows:** `dist\` resolves to `root\dist` and renders `dist` under a `\\?\` root. (FT-34,
  Windows only, CI)
- [ ] **AC-23 Every manifest without an alias, overlap or out-of-tree link builds as before,** byte-identical
  outputs and the same lockfile: every existing target in `tests/build.rs`, `tests/build_lock.rs`,
  `tests/build_cache.rs`, `tests/build_injective.rs`, `tests/build_watermark_cache.rs`,
  `tests/build_watch.rs` and `tests/build_refuses_output_flags.rs` stays green unchanged, and the existing
  unit rows `collision_key_*` and `lock_output_path_expands_the_real_ext_and_slashes` keep their literal
  expected values (their call shape may change). No new pin is needed: `lock_output_path_*` and
  `tests/build_lock.rs` already pin paths by literal.
- [ ] **AC-24 Docs:** each sentence under "Docs" below is present verbatim, and verify drives each.

## Failing Tests

All in `tests/build_destinations.rs` unless named. Expected values are literals in the test, never computed by
the code under test. Fixtures: a 2×2 PNG `src/a.png`; recipe `r.toml` = `resize mode = "exact"` to 1×1; the
font row copies `assets/fonts/Go-Regular.ttf` (the repo's own) into the temp tree. A helper `refused(dir,
args, message)` snapshots every file (bytes; links recorded as links, not followed) before and after, asserts
exit 2 and that stderr contains `error: <message>\n`, and asserts the snapshots are equal.

Every OS:
- **FT-1** `out_is_the_source_folder_is_refused` (AC-1).
- **FT-2** `out_dot_beside_its_source_is_refused` (AC-2).
- **FT-3** `dotdot_alias_without_the_alias_dir_is_a_collision`: also asserts `alias/` does not exist after (AC-3).
- **FT-4** `dotdot_alias_with_the_alias_dir_is_a_collision` (AC-3).
- **FT-5** `cur_dir_spellings_still_collide`: four spellings, a fresh tree each (AC-4).
- **FT-6** `out_dirs_differing_only_in_case_collide` (AC-5).
- **FT-7** `file_names_differing_only_in_case_collide`: `src/a.png` and `src2/A.png` into `dist` (AC-5).
- **FT-8** `non_ascii_names_differing_only_in_case_collide`: `s1/é.png`, `s2/É.png` (U+00E9, U+00C9) (AC-5).
- **FT-9** `another_targets_source_is_refused`: warm `dist/a.png` exists; target 1 `source = "dist/a.png"` (AC-6).
- **FT-10** `another_targets_glob_source_is_refused`: target 1 `source = "dist/*.png"` (AC-6).
- **FT-11** `the_recipe_file_is_refused`: `out = "."`, `name = "r.toml"` (AC-7).
- **FT-12** `a_watermark_image_is_refused`: recipe `watermark image = "logo.png"`, `name = "logo.{ext}"` (AC-8).
- **FT-13** `a_watermark_font_is_refused`: recipe `watermark text = "x" font = "go.ttf"`, `name = "go.ttf"` (AC-9).
- **FT-14** `the_manifest_is_refused` (AC-10). **FT-15** `the_lockfile_is_refused` (AC-11).
- **FT-16** `the_cache_dir_is_refused`: `out = ".crustyimg/x"` (AC-12).
- **FT-17** `a_refused_build_keeps_the_lockfile_and_cache_as_they_were`: build `out = "dist"` (exit 0), then
  switch the manifest to `out = "src"` and refuse (AC-13).
- **FT-18** `check_refuses_the_same_way_and_writes_nothing`: as FT-17, delete `dist/` and `.crustyimg/`,
  then `build --check` (AC-13).
- **FT-19** `a_collision_is_reported_before_an_overlap`: targets `src2/a.png → src` and `src/a.png → src`
  (both collide on `src/a.{ext}` and overlap `src/a.png`); expects the collision message with first
  `"src2/a.png"` (AC-14).
- **FT-20** `a_dotdot_escape_is_reported_before_an_overlap`: targets `src/a.png → src` and `→ ../x`; expects
  `` `out` escapes the build tree via `..`: ../x `` (AC-14).
- **FT-21** `the_lockfile_names_the_root_relative_path` (AC-15).
- **FT-22** `a_lockfile_with_an_old_spelling_drifts_once_on_paths`: build `out = "a/../dist"`, then
  rewrite the lock's `path = "dist/a.png"` to `path = "a/../dist/a.png"` (what `main` writes for that `out`;
  driven), then `--check`, `build`, compare the lock with the fresh one, `--check` (AC-16).

`#[cfg(unix)] mod links` in the same file:
- **FT-23** `an_in_tree_link_alias_is_a_collision`: `link → dist` (AC-17).
- **FT-24** `dotdot_past_a_link_is_a_collision`: `link → deep/a`, `out = "link/../dist"` vs `deep/dist` (AC-17).
- **FT-25** `a_link_out_of_the_tree_is_refused_and_nothing_is_written`: `link →` a second `TempDir`; asserts that
  dir is still empty (AC-17).
- **FT-26** `an_in_tree_link_builds_and_locks_the_real_path`: `dist → build/dist`; exit 0,
  `build/dist/a.png` exists, lock holds `path = "build/dist/a.png"` (AC-15, AC-17).
- **FT-27** `a_build_started_through_a_linked_path_locks_root_relative_paths`: run with `current_dir` = a link
  to the tree (AC-15).

`src/cli/build.rs` unit rows:
- **FT-28** `write_planned_refuses_a_dir_that_became_a_link_after_planning` (`#[cfg(unix)]`): plan `dist`
  (absent), then `symlink(other_tempdir, root/dist)`, call `write_planned`; asserts `Err` and the other dir
  is empty (AC-18).
- **FT-32** `fold_case_lowers_every_letter_not_only_ascii`: `"DIST/A.png"` → `"dist/a.png"`;
  `"\u{c9}t\u{c9}.PNG"` → `"\u{e9}t\u{e9}.png"` (AC-21).
- **FT-33** `output_extensions_cover_every_format_the_sink_names`: every `ImageFormat::all()` maps to a member
  (AC-21).
- **FT-34** `#[cfg(windows)] resolve_out_dir_on_windows`: with `root = canonicalize(tempdir)` (starts `\\?\`),
  `resolve_out_dir(&root, "dist\\") == root.join("dist")` and `root_relative(&root, &that) == "dist"` (AC-22).

Punch list rows (`tests/build_destinations.rs` unless noted; each is a pin except FT-35 to FT-42, red on
`9733cd8`):
- **FT-35** `macos_sharp_s_over_its_own_source_is_refused`, **FT-36** `macos_nfd_name_over_an_nfc_source_is_refused`
  (`cfg(macos)`, skip with a printed reason when the filesystem does not merge the names).
- **FT-37** `links::a_hard_link_to_a_source_at_the_output_name_is_refused` (an end-to-end row, not a unit row:
  `build_root()` reads the working directory, which a unit test cannot move safely).
- **FT-38** `a_glob_source_that_would_read_its_output_is_refused_on_the_first_build`, **FT-39**
  `a_folder_source_…`, **FT-40** `a_glob_that_does_not_match_its_output_still_builds_again` (green on `9733cd8`),
  **FT-41** `every_spelling_of_a_listing_source_and_an_out_is_refused`, **FT-42**
  `links::a_glob_through_a_link_and_an_out_through_a_link_are_refused`.
- **FT-43** `a_non_png_extension_overlap_is_refused_and_a_pinned_other_one_is_not`, **FT-44**
  `the_cache_dir_is_refused_in_any_letter_case`, **FT-45** `the_manifest_is_refused_when_given_by_a_non_default_path`,
  **FT-46** `an_existing_lockfile_is_refused`, **FT-47** `links::a_link_planted_at_an_output_file_makes_build_exit_6_and_is_not_followed`,
  **FT-48** `links::an_out_that_is_itself_a_link_out_of_the_tree_is_refused`, **FT-49**
  `links::an_out_that_is_a_link_to_the_source_folder_overlaps_the_source`, **FT-50**
  `an_out_reached_as_x_dotdot_src_overlaps_the_source`, **FT-51** `an_out_in_another_letter_case_overlaps_the_source`
  (skips where `src` and `SRC` are two folders).
- **FT-52** `readable_extensions_match_the_source_filter` (`src/cli/build.rs` unit row).

`src/cli/mod.rs` unit row:
- **FT-31** `exit_code_mapping_is_total` gains one row per new variant, each `== 2` (AC-20).

`tests/build_watch.rs` (`#![cfg(feature = "watch")]`, Unix rows behind `#[cfg(unix)]`), on its
`spawn_watch`/`poll_until` harness:
- **FT-29** `a_linked_out_under_a_source_root_does_not_rebuild_itself`: `outlink → src/gen`,
  `source = "src/*.png"`, `out = "outlink"`; wait for the first `built ` line, then a 3 s window; asserts
  exactly one `built ` line (AC-19). Negative control on `main`: 19 builds in 4 s (driven).
- **FT-30** `a_source_edit_inside_out_rebuilds`: `source = "src/a.png"`, `out = "src"`,
  `name = "{stem}_w.{ext}"`; after the first build, write a 3×3 `src/a.png`; `poll_until` `src/a_w.png`'s
  summary reads `(0 cached, 1 rebuilt)` a second time (AC-19). On `main` the second summary is
  `(1 cached, 0 rebuilt)` or absent (driven).

### Baseline the build records before changing code (trap 13)

Run FT-1 to FT-30 against `main`'s code (tests added, `src/` untouched) and quote each RED row's failure.
Expected (driven: the Trap-14 row below): FT-1 to FT-4, FT-6 to FT-18, FT-21 to FT-26, FT-29 and FT-30 red
**for their reason** (exit 0 or a different message, or the snapshot differs; FT-22 fails at its `assert_ne`,
because `main` already writes the old spelling). FT-5, FT-19 and FT-20 are **green on `main`** (regressions
and an order that already hold; FT-19 and FT-20 pin that the new passes do not change it). FT-27 is green on
`main` (`getcwd` is physical) and pins that the identity keeps it so. FT-28 and FT-31 to FT-34 do not compile on `main` (new functions); that is
their red. A row red for another reason (a fixture that fails to decode, a watermark that does not fit) is a
broken row: fix the row, not the code.

### Mutation table (trap 12): apply each alone, record every test red

Driven on the prototype (macOS arm64): each mutant applied alone to `src/cli/build.rs`, then one full
`cargo test --no-fail-fast` of every target with stdout redirected to a file; the red set below is every
failing test name read off that run (`grep -E '^    [A-Za-z_:]+$'` of the failure summary), not reasoned.
The prototype held FT-29/FT-30 in `tests/build_destinations.rs` (`mod watch`); the build puts them in
`tests/build_watch.rs`. FT-31 and FT-34 were not prototyped. Unless listed, every other test stayed green
(the baseline full run was rc=0).

- **M1, the resolver skips canonicalizing (lexical only).** In `resolve_out_dir`:
  `for n in (1..=parts.len()).rev() {` → `for n in (1..=root.components().count()).rev() {`.
  Red: FT-23, FT-24, FT-25, FT-26. Green: FT-3, FT-4 (no link involved). *Driven.*
- **M2, the tail keeps `..`.** In `resolve_out_dir`, the tail's `Component::ParentDir => { resolved.pop(); }`
  → `Component::ParentDir => { resolved.push(".."); }`. Red: FT-3, FT-21, FT-22. Green: FT-4 (with `alias/`
  present, `alias/..` is part of the canonicalized prefix). *Driven.*
- **M3, no case fold.** `fold_case`: `path.to_lowercase()` → `path.to_owned()`. Red: FT-6, FT-7, FT-8,
  FT-32. *Driven* (APFS; on Linux the same rows red, since they compare strings: *reasoned*).
- **M3b, ASCII-only fold** (the rule not picked). `path.to_lowercase()` → `path.to_ascii_lowercase()`.
  Red: FT-8, FT-32. Green: FT-6, FT-7. *Driven.*
- **M4, overlap ignores recipe assets.** In `reads_of`: `for step in &p.recipe.steps {` →
  `for step in p.recipe.steps.iter().take(0) {`. Red: FT-12, FT-13. *Driven.*
- **M5, overlap ignores recipe files.** In `reads_of`: `if Path::new(&p.target.recipe).is_file() {` →
  `if false {`. Red: FT-11. *Driven.*
- **M6, chaining allowed** (overlap skips other targets' sources; also the frame's "only within one
  target"). In `check_destinations`: `if let Some(what) = reads.get(&fold_case(&abs.to_string_lossy())) {`
  → `… reads.get(…).filter(|w| !w.ends_with("reads as a source") || w.contains(&format!("#{i} "))) {`.
  Red: FT-9, FT-10. *Driven.*
- **M7, the refusals run after phase 2.** In `run_build`:
  `check_destinations(&mut prepared, &registry, Path::new(path))?;` →
  `let late = check_destinations(&mut prepared, &registry, Path::new(path));` and `late?;` inserted after
  `bar.finish_and_clear();`. Red (24): FT-1 to FT-19, FT-23, FT-24, FT-25, and the existing
  `tests/build_injective.rs` rows `colliding_stems_in_one_target_are_rejected` and
  `collision_across_two_targets_is_rejected`. Each FT row fails at the snapshot assertion ("a refused build
  must leave every file as it was"), its reason. Green: FT-20 (manifest validation runs first), FT-21,
  FT-22, FT-26, FT-27. *Driven.*
- **M8, the lock `path` uses the `out` spelling.** In `lock_record`: `lock_output_path(&prepared.rel_dir,` →
  `lock_output_path(&to_slash(Path::new(&prepared.target.out)),`. Red: FT-21, FT-22, FT-26. *Driven.*
- **M9, the watch exclusion stays lexical.** In `watch_set_for`: `set.excluded.push(dir);` → `let _ = dir;`.
  Red: FT-29. *Driven.*
- **M10, no root containment.** In `check_destinations`: `if !dir.starts_with(&root) {` → `if false {`.
  Red: FT-25. *Driven.*
- **M11, the root is not canonicalized.** In `build_root`: `std::fs::canonicalize(cwd).map_err(…)` →
  `Ok(cwd)`. Red: **none** on macOS. `getcwd` returns the physical path on Unix (FT-27 runs through a linked
  cwd and stays green), so this is an equivalent mutant on Unix. *Driven* (green); on Windows *reasoned*:
  `current_dir()` can be `C:\Users\RUNNER~1\…` while `canonicalize` gives `\\?\C:\Users\runneradmin\…`, so
  `starts_with` fails and every `tests/build*.rs` build row on that leg would be refused as out-of-tree. The
  build reads that leg once with the mutant applied, if CI time allows; FT-34 is the standing row.
- **M12, the out-of-tree check runs after the sink creates directories.** `if !dir.starts_with(&root) {` →
  `if false {`, and in `write_planned` after the canonicalize:
  `if !now.starts_with(build_root()?) { return Err(CliError::Usage("outside".into())); }`.
  Red: FT-25 (the outside dir is no longer empty, and the exit is 6), and FT-28 *(corrected at punch list)*: `build_root()` is the
  test process's working directory, not the temp project, so the mutant's added check returns its exit-2
  usage error before `write_planned` reaches the exit-5 link refusal. *Driven.*
- **M13, no read carve-out in `--watch`.** In `watch_impl`: `&& !is_read(p, &reads))` → `)`.
  Red: FT-30. *Driven.*
- **M14, no re-resolution before the write.** In `write_planned`:
  `if fold_case(&now.to_string_lossy()) != fold_case(&out_dir.to_string_lossy()) {` → `if false {`.
  Red: FT-28. *Driven.*
- **M15, the identity check removed** (punch list). In `check_destinations`:
  `reads.by_file.get(&(meta.dev(), meta.ino()))` → `….filter(|_| false)`. Red: FT-35, FT-36, FT-37.
- **M16, the listing check removed** (punch list). `if lister_reads(lister, &rel, &p.out_dir, &name) {` →
  `if false && …`. Red: FT-38, FT-39, FT-41, FT-42. M16glob, the glob arm removed and the folder arm kept
  (`if matches!(lister, Lister::Folder { .. }) && …`): red FT-38, FT-41, FT-42; FT-39 stays green.
- **M17, `{ext}` checks only png** (punch list). `_ => OUTPUT_EXTENSIONS,` → `_ => &["png"],`. Red: FT-43.
- **M18, the `.crustyimg` check is case-sensitive** (punch list). `fold_case(&first) == ".crustyimg"` →
  `first == ".crustyimg"`. Red: FT-44.
- **M19, the manifest is always the default path** (punch list). `note(manifest_path, …)` →
  `note(Path::new("crustyimg.build.toml"), …)`. Red: FT-45.
- **Trap 14, `main`'s `src/cli/build.rs` restored verbatim** (`git show origin/main:src/cli/build.rs`; the
  prototype's `src/cli/mod.rs` kept, so it compiles). Red (25): FT-1, FT-2, FT-3, FT-4, FT-6 to FT-18,
  FT-21 to FT-26, FT-29, FT-30. Green: FT-5, FT-19, FT-20, FT-27 (behaviour `main` already has). FT-28,
  FT-32, FT-33 do not exist (they live in the replaced file's test module). *Driven.*

Windows (*reasoned*, CI only): FT-34 is the only Windows-only row. On the Windows leg, FT-6 to FT-8 assert
what NTFS does anyway; FT-23 to FT-27 and FT-28 are `#[cfg(unix)]` and do not run. Expect the Windows leg's
red set at baseline to be the Trap-14 set minus the Unix rows.

## Implementation Context

*Read this section (and the files it points to) before starting
the build cycle. It is the equivalent of a handoff document, folded
into the spec since there is no separate receiving agent.*

### Design calls (settled)

**The code shape.** All in `src/cli/build.rs` unless named; line numbers are `main` at `54fcc79`. A prototype
of exactly this shape passed the full `cargo test --no-fail-fast` on macOS (rc=0) with FT-1 to FT-33 green;
the build writes its own code from this text.

1. **`build_root() -> Result<PathBuf, CliError>`** (new, beside `check_output_injective` :393):
   `fs::canonicalize(env::current_dir()?)`. An error is `CliError::Image(ImageError::Io(_))` (exit 3), the
   existing mapping for an unreadable path.
2. **`fold_case(&str) -> String`** (new): `s.to_lowercase()`. The one comparison rule for collisions,
   overlaps and the sink's re-check. Comment says why: macOS and Windows merge names by case.
3. **`resolve_out_dir(root: &Path, out: &str) -> PathBuf`** (new): drop `CurDir` components of `out`;
   `full = root.join(rest)`; walk `n` from `full.components().count()` down to 1, and at the first prefix
   whose `fs::canonicalize` succeeds, take that canonical path and push each remaining component, popping
   on `ParentDir`. Returns an absolute path. Comment the lexical-tail argument (a directory that does not
   exist cannot be a link).
4. **`root_relative(root, path) -> String`** (new): `to_slash(path.strip_prefix(root))`, `""` for the root.
   **`join_rel(dir, name)`** (new): `name` when `dir` is empty, else `dir/name`.
5. **`first_link(root, out) -> String`** (new): the first prefix of `out`, as spelled, whose
   `symlink_metadata` is a link; used only in the message.
6. **`PreparedTarget`** (:222) gains `out_dir: PathBuf` (resolved, absolute) and `rel_dir: String`
   (`root_relative`). `prepare_target` (:279) leaves them empty; `check_destinations` fills them.
7. **`output_collision_key`** (:376) becomes `(rel_dir: &str, template: &str, input) -> String` =
   `join_rel(rel_dir, expand_template(template, stem, EXT_SENTINEL, path))`. Its doc comment drops "NOT
   canonicalized".
8. **`check_output_injective`** (:393) takes `root` too, keys `find_output_collision` on
   `fold_case(output)`, and reports the first claimant's unfolded output (a `HashMap<folded, shown>`).
   `find_output_collision` (`src/build/mod.rs:342`) is unchanged.
9. **`check_destinations(prepared: &mut [PreparedTarget], registry, manifest_path: &Path)`** (new),
   replacing the call at :751, before the lockfile load (:756) and `Cache::open` (:773). In order:
   (a) per target, `resolve_out_dir`; if `!dir.starts_with(&root)` → `OutputOutsideTree`; else set
   `out_dir`/`rel_dir`; (b) `check_output_injective`; (c) build the read set (10) and, per target, per
   input, the planned name `join_rel(rel_dir, expand(…, EXT_SENTINEL, …))`: if its folded first component
   is `.crustyimg` → `OutputInCacheDir`; else for each extension (the pinned one for
   `OutputFormatPlan::Pinned(fmt)`, `extension_for_format(fmt)`; otherwise every entry of
   `OUTPUT_EXTENSIONS`) look up `fold_case(root.join(name with {ext} replaced))` in the read set →
   `OutputOverlap`.
10. **The read set** (new fn `reads_of`): `HashMap<folded absolute path, clause>`, first insert wins, in this
    order: the manifest (`which is the build manifest`), `crustyimg.build.lock` (`which is the build
    lockfile`), then per target in manifest order: each resolved `Input::Path` (`which target #j reads as
    a source`), the recipe when `Path::new(&target.recipe).is_file()` (`which is the recipe of target
    #j`), and each `registry.asset_keys(op)` value on the prepared recipe's steps (`which target #j's recipe
    reads as its {op} {key}`, so `watermark image` / `watermark font`). Each path is
    `fs::canonicalize(p)` or, when it does not exist (the lockfile), `root.join(p)`. When the clause is a
    source clause and `j` is not the writing target, append `; one target's output cannot be another
    target's source`.
11. **`OUTPUT_EXTENSIONS: &[&str]`** (new const): `png jpg gif bmp tiff ico webp pnm tga dds hdr exr ff avif
    qoi bin`, the range of `sink::extension_for_format` (`src/sink/mod.rs:332`). FT-33 pins it.
12. **`write_planned(bytes, ext, input, out_dir, template, overwrite)`** (new, replaces both `write_encoded`
    calls in `build_one`, :572 and :625; `build_one` reads `prepared.out_dir`): `create_dir_all(out_dir)`
    (error → `SinkError::OutDirCreate`, exit 5 as today); `now = canonicalize(out_dir)`; if
    `fold_case(now) != fold_case(out_dir)` → `SinkError::Traversal(out_dir)` (exit 5); `path =
    safe_join(out_dir, expand_template(template, stem, ext, path))`; `Sink::File { path, format: None }
    .write_bytes(bytes, …, ext, overwrite, &mut io::sink())`. That arm runs `reject_symlink_destination` and
    `guard_overwrite` (DEC-035) and, after SPEC-155, its staged-write helper: **the helper receives the
    resolved file path, and its temp goes in `path.parent()`, the resolved directory.** `write_encoded`
    stays for `apply` (`src/cli/common.rs:129`), unchanged.
    SPEC-155's design landed on `main` during this design (`c16a202`, re-read after the rebase): its
    `write_bytes` `Sink::File` arm (`src/sink/mod.rs:645-652` on `main`) keeps `reject_symlink_destination`
    and `guard_overwrite` and then calls `publish_file(path, bytes)`, whose `publish` stages in
    `path.parent()`. So `write_planned` needs no change for it: the staged temp lands in the resolved
    directory. Its new `SinkError::WriteFailed`/`DirNotWritable` reach `build` through this arm (exit 5).
    After rebasing onto SPEC-155's merge, re-check only that the `Sink::File` arm of `write_bytes` still
    calls `publish_file`.
13. **`lock_output_path`** (:473) becomes `(rel_dir, template, input, ext)`; `lock_record` (:489) passes
    `&prepared.rel_dir`. The post-write collision check (:893-912) keys on `fold_case(r.path)` too.
14. **`--watch`.** `watch_set_for` (:1087) appends each target's `resolve_out_dir(build_root, out)` to
    `set.excluded` (keep `watch_roots`' lexical entries). New `watch_reads(manifest, manifest_path,
    registry) -> Vec<String>`: folded canonical paths of the manifest, each target's resolved sources
    (`source::resolve_all(…, OnEmpty::Skip)`, errors ignored), recipe files and `recipe_asset_paths`. New
    `is_read(event, &reads)`: canonicalize the event path (or keep it) and fold. The filter at :1132
    becomes `batch.iter().all(|p| is_excluded(p, &set.excluded) && !is_read(p, &reads))`; `reads` is
    computed at start (beside :1109) and refreshed with the set (:1146). Comment: the build refuses an
    output that is one of these files, so its own writes never match.
15. **Errors** (`src/cli/mod.rs`, after `OutputCollision` :816), each with a `code()` arm `=> 2` beside :885
    and a row in `exit_code_mapping_is_total` (:1416):
    - `OutputOverlap { target: usize, output: String, what: String }`,
      `#[error("target #{target} would overwrite \"{output}\", {what}")]`;
    - `OutputInCacheDir { target: usize, output: String }`,
      `#[error("target #{target} would write \"{output}\" inside .crustyimg/, the build's cache directory")]`;
    - `OutputOutsideTree { target: usize, out: String, link: String }`,
      `#[error("target #{target}'s out \"{out}\" resolves outside the build tree through the link \"{link}\"; build inside the tree and copy the outputs where you need them")]`.
    Paths are printed with `Display`, never `{:?}` (the Windows `\\` lesson in `OutputCollision`'s doc).
16. `src/build/mod.rs`, `Target::validate` comment (:286-289): replace the "RESIDUAL: a pre-existing symlink
    … (DEC-061)" sentences with "A link inside `out` is resolved at planning, and an `out` that resolves
    outside the tree is refused there." (No id in the comment.) `Target.out`'s doc (:178) is already true.

**Every message, verbatim** (stderr line `error: <message>`, exit 2 unless noted):
- collision (two inputs, any targets; unchanged text, resolved output):
  `output collision: "dist/a.{ext}" written by both "src/a.png" and "src2/a.png" — two inputs map to one output (disambiguate the name template, e.g. {parent}_{stem})`
- a source of the same target: `target #0 would overwrite "src/a.png", which target #0 reads as a source`
- chaining: `target #0 would overwrite "dist/a.png", which target #1 reads as a source; one target's output cannot be another target's source`
- recipe file: `target #0 would overwrite "r.toml", which is the recipe of target #0`
- watermark image: `target #0 would overwrite "logo.png", which target #0's recipe reads as its watermark image`
- watermark font: `target #0 would overwrite "go.ttf", which target #0's recipe reads as its watermark font`
- manifest: `target #0 would overwrite "crustyimg.build.toml", which is the build manifest`
- lockfile: `target #0 would overwrite "crustyimg.build.lock", which is the build lockfile`
- under `.crustyimg/`: `target #0 would write ".crustyimg/x/a.{ext}" inside .crustyimg/, the build's cache directory`
- out-of-tree link: `target #0's out "link/x" resolves outside the build tree through the link "link"; build inside the tree and copy the outputs where you need them`
- a glob source that would list the output on the next build (punch list): `target #0 would overwrite "src/a_w.png", which target #0's source pattern `src/*.png` would read on the next build`
- a folder source that would list it (punch list): `target #0 would overwrite "src/a_w.png", which target #0's source folder `src` would read on the next build`
- another target's glob or folder (punch list): the same clause with that target's number, then `; one target's output cannot be another target's source`
- the sink's re-check (exit 5 from `apply`; exit 6 from `build`, which counts it as a failed input *(corrected at punch list)*): `output path escapes the target directory: <resolved dir>` (`SinkError::Traversal`, `src/sink/mod.rs:229`).
No message names a spec, stage or DEC.

**What pins "every other manifest builds the same"** (AC-23): the existing build suites, unchanged and green
on the prototype's full run (`tests/build.rs`, `build_lock.rs`, `build_cache.rs`, `build_injective.rs`,
`build_watermark_cache.rs`, `build_watch.rs`, `build_refuses_output_flags.rs`), plus the literal-valued unit
rows `collision_key_ignores_output_ext_but_not_the_rest`, `collision_key_normalizes_the_out_dir`,
`lock_output_path_expands_the_real_ext_and_slashes`. `cache_key_for` does not read the output path, so cache
keys and `key = …` lock values cannot move. The lock `path` moves only for `..`, link and on-disk-case
spellings (AC-16). No new golden.

### The new DEC (ship writes the file; take the next free id at ship time)

File: `decisions/DEC-<next>-a-build-destination-has-one-identity-resolved-before-anything-is-written.md`.
SPEC-155's design also adds a DEC and others run alongside, so **do not reserve a number now**: at ship,
`ls decisions/` and take the next free id. Front matter `affected_scope` in **block-list form** (HANDOFF
trap 7):

```yaml
affected_scope:
  - src/cli/build.rs
  - src/cli/mod.rs
  - src/build/mod.rs
  - tests/build_destinations.rs
  - tests/build_watch.rs
  - docs/cli-reference.md
  - SECURITY.md
  - skills/crustyimg/SKILL.md
```

Body (`## Decision`):

> A `build` destination has one identity, computed once at planning, before the lockfile is read, the cache
> is opened or anything is written. Each target's `out` is resolved against the build root (the working
> directory, canonicalized): `.` components are dropped, the longest leading part that exists is
> canonicalized (links followed, `..` after a link applied as the kernel applies it, on-disk case taken), and
> the part that does not exist yet is appended with `..` applied lexically. The output's file name is joined
> onto it as written and is never followed: planning compares it as written (so a name `src/a.{ext}` is
> checked as `<out>/src/a.png` and can be refused as an overlap, exit 2), and `safe_join` refuses a name
> with a separator at write time (exit 6) *(corrected at punch list)*. Planning, the collision check, the overlap check, the sink, the
> lockfile and the `--watch` exclusion all use this identity.
>
> 1. Collisions and overlaps compare the identity after `str::to_lowercase`, on every OS. Two outputs that
>    differ only in letter case are one output, even on a case-sensitive filesystem. `to_lowercase` does not
>    merge `ß` and `SS`; see "Not covered" *(corrected at punch list)*.
> 2. A destination may not be a file the build reads: any target's resolved source, any target's recipe
>    file, any file a recipe step names through an asset key (a watermark's `image` or `font`), the manifest,
>    or the lockfile. So one target's output may not be another target's source. A destination may not lie
>    under `.crustyimg/`. An output name ending in `{ext}` is compared under every extension the build can
>    write, unless the target's format is pinned. An output that already exists and is the same file as one of
>    these (device and inode, links followed; Unix) is refused whatever its name, so `ß` for `SS`, NFC for
>    NFD and a hard link are caught for a file that exists *(corrected at punch list)*. A target's glob or folder source may not
>    list one of its planned outputs on the next build *(corrected at punch list)*.
> 3. An `out` that resolves outside the build root is refused. A link to a directory inside the tree is
>    allowed and is de-aliased by the identity. There is no opt-in for an out-of-tree link; build inside the
>    tree and copy.
> 4. Every refusal above exits 2 and writes nothing, the same class as a malformed manifest.
> 5. The lockfile records each output by its root-relative identity with `/` separators. A lockfile
>    written before this decision with a `..`, link or off-case spelling drifts once, on paths only.
> 6. The sink creates the resolved directory, resolves it again, and refuses the write if it moved (exit 6
>    from `build`, which counts it as a failed input; exit 5 is the sink's code on `apply`) *(corrected at punch list)*.
>    DEC-035's per-file link refusal stays the last layer. A swap after that re-check is not covered.
> 7. `--watch` excludes the resolved `out` of every target, and an event on a file the build reads always
>    triggers a rebuild, even inside an excluded directory (rule 2 guarantees the build's own writes are never
>    such a file).
>
> Not covered: two outputs that land in one new file because their names differ only in Unicode
> normalization (NFC and NFD) or in a case `to_lowercase` does not fold (`ß` and `SS`) are one file on macOS
> and two identities here, because neither file exists at planning time; that needs normalization tables
> `std` does not have. The output-to-output collision is all that is left: an output over a file the build
> reads is caught by file identity *(corrected at punch list)*. Destinations of `apply` and the
> single-file verbs (`-o`, `--out-dir`) are not resolved this way.

`## Context`: the two P1 findings (`docs/reviews/2026-11-10-external-code-review.md`), the frame's driven
premise (`out = "src"` replaced the source at exit 0; `alias/../dist` gave one file, two lock records and a
passing `--check`). `## Consequences`: the four CHANGELOG "Changed" items; DEC-061's symlink residual and
the exception in its accepted risk 3 are closed. `## Alternatives considered`: lexical-only identity (wrong:
`link/..` is `deep`, not `.`, driven); refusing any link in `out` (breaks in-tree conveniences); keeping
out-of-tree links (keeps the reviewer's write); full Unicode case folding or normalization (needs a new
dependency). Links: `related: DEC-035, DEC-057, DEC-059, DEC-060, DEC-061`.

### DEC-061's amendment (ship applies it; `main` stays true until the code lands)

In `decisions/DEC-061-*.md`:
- After the "**Residual (symlink) — un-caught, accepted, filed.**" bullet's last sentence ("… filed as
  backlog item #10, not done here."), append:
  > **Closed by DEC-<next> (<ship date>):** `build` now resolves each `out` with links followed before
  > anything is written and refuses one that resolves outside the canonical build root (exit 2). In-tree
  > links still work. `dist → ramdisk` is refused; build inside the tree and copy.
- In accepted risk 3, replace the sentence beginning "**One accepted exception:** a symlinked *out dir* still
  escapes both layers" through "filed as backlog item #10." with:
  > **The symlinked-*out*-dir exception is closed by DEC-<next>:** an `out` that resolves outside the tree
  > through a link is refused at planning.
- Add `superseded_by` nothing (DEC-061 still stands); add `DEC-<next>` to its related list if it has one.

### Docs (exact changes; the build drives every behaviour sentence, verify drives them again)

1. `docs/cli-reference.md:368-370` (§build, "Unlike `apply` …"). Replace the paragraph with:
   > Unlike `apply`, `build` **overwrites its own declared outputs without `--yes`** — a build owns its output
   > tree and must be re-runnable. It never overwrites a file the build reads.
   >
   > Before anything is written, each target's `out` is resolved inside the build tree (the working
   > directory), with `..` applied and links followed. The build is refused with exit `2`, and writes
   > nothing, when:
   >
   > - two inputs resolve to one output file, however `out` is spelled (`dist`, `./dist`, `a/../dist`, or a
   >   link to `dist`). Letter case is ignored on every OS as `str::to_lowercase` folds it, so `a.png` and
   >   `A.png` in one folder collide on Linux too, while `ß` and `SS` stay two names. Two outputs that land in
   >   one new file by names only a Mac's filesystem merges (`ß`/`SS`, NFC/NFD) are not caught;
   > - an output would replace a file the build reads: a source of any target, a recipe file, a file a
   >   recipe names (a watermark's image or font), the manifest, or the lockfile. So one target's output
   >   cannot be another target's source. An output that already exists and is the same file as one of
   >   these is refused too, whatever its name (a hard link; on a Mac, `ß` for `SS` or NFC for NFD);
   > - a target's source is a glob or a folder that would list one of its outputs on the next build
   >   (`source = "src/*.png"`, `out = "src"`, `name = "{stem}_w.{ext}"`), which would feed the output back in;
   > - an output would land in `.crustyimg/`;
   > - an `out` resolves outside the build tree through a link. A link to a folder inside the tree is fine;
   >   to publish outside the tree, build inside it and copy.
   >
   > When a name ends in `{ext}` and the format is not pinned, every extension the build can write is checked.
   Both new sentences marked *(corrected at punch list)*. Exit paths named: exit 2 for each bullet (FT-1 to FT-16, FT-23 to FT-25, FT-35 to FT-40); "a link inside the tree is
   fine": exit 0 (FT-26).
2. `docs/cli-reference.md:400-402` (§`--watch`): replace "and the build's own outputs, `.crustyimg/`, and the
   lockfile are ignored — so a rebuild never triggers itself." with:
   > and the build's own outputs (wherever `out` resolves, links included), `.crustyimg/`, and the lockfile
   > are ignored — so a rebuild never triggers itself. A change to a file the build reads still triggers a
   > rebuild, even inside an `out` folder.
   Exit paths: none (behaviour; FT-29, FT-30).
3. `docs/cli-reference.md:435-436` (§lockfile, "It records one entry per output: the file it wrote, …"):
   change "the file it wrote," to "the file it wrote (its path from the build root, so `./dist`,
   `a/../dist` and a link to `dist` record where the file really is),". Exit paths: exit 0 (FT-21, FT-26).
4. `docs/cli-reference.md:442-445` (§lockfile, "`build --check` re-runs …"): replace with:
   > `build --check` re-runs the build and compares what it produced against the committed lockfile instead
   > of refreshing it. It exits **0** when they agree and **7** on drift, naming each output that moved. It
   > **never modifies the lockfile**, but it does write: every output is written again and the cache is
   > refilled, as in a plain build, and it compares the bytes this run produced, not the files already on
   > disk. A build refused before it starts (exit `2`) writes nothing under `--check` either. `--frozen` and
   > `--locked` are aliases (there is no network to lock out). A missing lockfile under these flags is drift,
   > not an error.
   Exit paths: 0, 7 (existing `tests/build_lock.rs`), 2 with nothing written (FT-18). "every output is
   written again and the cache is refilled": driven at frame (an output and `.crustyimg/` deleted before
   `--check` were back after, exit 0); the build drives it once more and records the result in Build
   Completion (no test: a later STAGE-063 entry may change it).
5. `docs/cli-reference.md:52` (exit-code table, row `2`): "Usage error (bad args) — clap's standard code." →
   "Usage error (bad args) — clap's standard code; also a `build` refused before it writes (a malformed
   manifest, or outputs that collide, replace a file the build reads, or resolve outside the tree)."
6. `SECURITY.md:38-47` (threat 3). Before "On input, directory/glob sources …", insert:
   > `build` resolves each target's `out` once, before writing, with links followed: an `out` that resolves
   > outside the build tree is refused (exit 2), as is an output that would replace a file the build reads
   > or two outputs that are one file; the folder is resolved again just before each write, and the write is
   > refused (exit 6: `build` counts it as a failed input) if it moved.
   And in the same threat's sentence on the sink's link refusal: "(`SinkError::Traversal`, exit 5;
   `build` aggregates per-input failures, so it exits 6)" *(corrected at punch list)*. Exit paths: 2 (FT-25, FT-1, FT-3), 6 (FT-28
   for the folder; FT-47 end to end for a link at the output file).
7. `SECURITY.md:87` (row 3). Append to the mitigations cell: "; `build` outputs resolved and contained in
   the tree (links followed), never over a file the build reads"; add `src/cli/build.rs` to the location
   cell.
8. `skills/crustyimg/SKILL.md:301` ("`build` overwrites its own outputs without `-y`.") → "`build`
   overwrites its own outputs without `-y`, but never a file it reads (a source, recipe, watermark file,
   manifest or lockfile). That, two outputs that are one file (letter case ignored), a glob or folder source
   that would list its own output next time, or an `out` that links outside the project is refused with exit 2
   and nothing written." *(corrected at punch list)* Exit paths: 2 (FT-1, FT-6, FT-25, FT-38).
9. `src/cli/mod.rs:536-541` (`Build` doc, the `--help` long text): append "It never overwrites a file the
   build reads, and refuses (exit 2, nothing written) outputs that collide or resolve outside the tree."
   And `:119-120` (`--check`): "… instead of refreshing it: exit 7 on drift, and never modify the lockfile
   (outputs and the cache are still written)." Both pass `tests/help_has_no_internal_ids.rs`.
10. Not changed (checked): `README.md:292-294`, `docs/USAGE.md:165` ("symlinked output paths are refused",
    still true), `SKILL.md:334` (exit 5 row), `docs/api-contract.md` (exit 2 already means usage).

### CHANGELOG (`## [Unreleased]`, verbatim)

```
### Fixed
- `build` no longer overwrites a file it reads or writes two outputs to one file. An `out` that pointed at a
  source folder (`out = "src"`), at the recipe or a watermark file by name, or that reached another
  target's `out` through `..` or a link replaced that file or recorded one file twice in
  `crustyimg.build.lock`, and exited 0. Each target's `out` is now resolved before anything is written, and
  such a build is refused with exit 2. So is a source that would list its own output on the next build:
  `source = "src/*.png"` or `"src"` with `out = "src"` and `name = "{stem}_w.{ext}"` built once and then
  exited 2 on every later run; it is now refused on the first. *(corrected at punch list)*
- `build --watch` no longer rebuilds in a loop when `out` is a link into a watched source folder, and no
  longer misses an edit to a source that sits inside an `out` folder.

### Changed
- `build` refuses an `out` that resolves outside the project through a link (exit 2). Build inside the
  project and copy the outputs.
- `build` refuses one target's output being another target's source (exit 2). Before, this failed on a clean
  tree and silently overwrote the source on a warm one.
- `build` ignores letter case when it checks that outputs are distinct, on every OS: `a.png` and `A.png`
  written to one folder are refused on Linux too. Only what `str::to_lowercase` folds counts as one name
  (`ß` and `SS` do not), so two outputs that land in one new file by names only a Mac's filesystem merges
  (`ß`/`SS`, NFC/NFD) are not caught; an output that is already a file the build reads is, by file identity. *(corrected at punch list)*
- `crustyimg.build.lock` records each output's path from the build root. A lockfile written with an `out`
  spelled through `..`, a link, or a case that differs from the folder on disk reports path drift once
  under `build --check`; run `crustyimg build` once to refresh it.
```

### Overlap with other open work (from file lists, 2026-11-22)

`gh pr list --state open --json number,title,files` and `git branch -r`:
- **PR #226** (agents docs) touches `src/cli/mod.rs`: its hunks are the module doc (`:2-12`) and
  `dispatch`'s doc comment (`:888`). This spec changes the `CliError` enum (after `:816`), `code()` (`:885`),
  `exit_code_mapping_is_total` and the `Build`/`--check` doc comments. No shared hunk; a rebase is textual.
- **PR #237** (heifer) touches `CHANGELOG.md`. Keep both sides of `## [Unreleased]`.
- PRs #240 and #229: no shared file.
- **No pushed `feat/spec-15[4-8]*` branch exists.** The remote `feat/spec-150/151/152/153/159` branches are
  shipped specs (SPEC-150 and SPEC-159 are in `specs/done/`; their diffs against `main` are `main` moving
  ahead), not open work.
- **SPEC-155** (no branch yet; from its frame's rough build file list): `src/sink/mod.rs` (helper, four arms),
  `src/cli/ops.rs`, `src/cli/build.rs` (`write_lock` :677 only), `SECURITY.md`, `docs/cli-reference.md`
  (`:55`, `:39`), `src/cli/mod.rs` (maybe the `-y` doc), `CHANGELOG.md`, `tests/atomic_write.rs`. Shared
  with this spec: `src/cli/build.rs` (SPEC-155: `write_lock`; this spec: everything in "code shape", not
  `write_lock`), `SECURITY.md` (different sentences of threat 3 and row 3), `docs/cli-reference.md`
  (SPEC-155: exit table row 5 and `-y`; this spec: row 2 and §build), `src/cli/mod.rs`, `CHANGELOG.md`.
  SPEC-155 builds first; this build rebases onto it.
- On a conflict in STAGE-063's file: keep both sides and recount `**Count:**` with a fresh `/usr/bin/grep`.
  On a conflict in any other file: stop and report. Never edit `projects/PROJ-017-geometry-as-recipe-steps/`.

### Size: [M] holds

The prototype's diff (`git diff --stat origin/main` in the prototype worktree): `src/cli/build.rs` +359/−37
(of which ~70 are unit-test lines), `src/cli/mod.rs` +39/−0, `tests/build_destinations.rs` +372 new
(`git diff --numstat origin/main HEAD -- src tests`). Production code is one resolver (5 small fns), one planning pass, one writer,
three variants, and the watch carve-out (2 fns): about 300 lines. The split trigger (the sink cannot take a
resolved path without changing `apply`) does not fire (correction 4). It is not S: four axes (link, `..`,
case, reads), three OS legs, a watch change, a DEC and DEC-061's amendment, and 34 tests.

### What the build prompt must carry

- **Checkpoints:** push after (1) the tests file red against `main`'s code with the baseline quoted, (2) the
  resolver, planning pass and variants green, (3) `write_planned`, lock path, watch, (4) docs.
- **CI legs:** the Windows leg is the only evidence for FT-34 and for the case rows on NTFS; read its red
  set off the PR. The Linux leg is where the case rows over-refuse (that is the assertion). The lean
  `--no-default-features` leg compiles out `--watch`: `watch_reads`/`is_read` are `#[cfg(feature = "watch")]`.
- Run `cargo test --no-fail-fast` with stdout redirected to a file (the `display_sink_refuses_non_tty` row
  fails in an interactive terminal); read `$?`, not a pipe's.
- Re-check SPEC-155's helper (code shape 12) after rebasing.

### Decisions that apply

- `DEC-057` — manifest paths resolve against the working directory; the build root here.
- `DEC-035` — a link at the destination file is refused even with `--yes`; stays the sink's last layer.
- `DEC-059` — the lockfile's `path` is its primary key; the value changes, the schema does not.
- `DEC-060` — `--watch`'s exclusion set; extended, not replaced.
- `DEC-061` — the symlink residual and accepted risk 3; amended at ship.
- `DEC-010` — directory sources are non-recursive, so an `out` folder inside a source folder is not refused.
- `DEC-106` — check before any file is read or written: the precedent for refusing at planning.

### Constraints that apply

- `untrusted-input-hardening` — a manifest is reviewed like code but its paths are untrusted; every path
  from it is resolved and contained before a write.
- No new dependency (std only); no version bump.

### Prior related work

- SPEC-065 (shipped) — the injective source→output check this extends.
- SPEC-066 (shipped) — the lockfile and `--check`.
- SPEC-067 (shipped) — `--watch` and its exclusion.
- SPEC-068 (shipped) — the lexical `out` containment and DEC-061.

### Out of scope (for this spec specifically)

- SPEC-158's items: hashing one read and decoding another, the lockfile size contract, the `encode_one` clone.
- Making `build --check` read-only, or comparing the outputs on disk (filed in STAGE-063).
- Destination resolution for `apply` and the single-file verbs (`-o`, `--out-dir`).
- An opt-in for `out` links that resolve outside the build tree.
- Pruning orphaned outputs (PROJ-007 STAGE-024 item).
- **Unicode normalization (NFC vs NFD) and `ß`/`SS` in the identity,** for two outputs that land in one new
  file. Filed below; needs a dependency decision. An output over a file the build reads is caught by file
  identity *(corrected at punch list)*.
- A new file matching a source glob inside an excluded `out` does not wake `--watch` until another change
  does (only files that existed when the watch set was computed count as reads). Unchanged from `main`.

### Follow-up filed by this design

- STAGE-063 backlog: "[S] `build` identity ignores Unicode normalization and `ß`/`SS`: `café.png` in NFC and
  NFD, or `ß.png` and `SS.png`, from two folders into one `out` is one file on macOS and two lock records
  (driven 2026-11-22, SPEC-157 design). Only this output-to-output collision is left: an output over a file
  the build reads is refused by file identity. Closing it needs NFC/NFD tables (`unicode-normalization` or
  hand-rolled); a dependency decision." *(corrected at punch list)*

## Notes for the Implementer

- The resolver is filesystem-touching; keep `watch_roots` (`src/build/watch.rs`) lexical and do the
  resolving in `src/cli/build.rs`.
- `prepared.out_dir` is set only by `check_destinations`; `build_one` must never fall back to `target.out`.
- `refused()`'s snapshot records a link as a link (it must not follow `link` in FT-25, or it walks outside).
- The font row needs a parseable font: copy `assets/fonts/Go-Regular.ttf` with `include_bytes!`.
- Comments read behaviour-first, with no spec or DEC ids.

---

## Build Completion

- **Branch:** `feat/spec-157-build-destinations`
- **PR (if applicable):** #270 (draft)
- **All acceptance criteria met?** yes (AC-22's Windows row is evidenced only by the Windows CI leg)
- **New decisions emitted:** none (the new DEC and DEC-061's amendment are ship's)
- **Files this diff touches** (9 files, not counting this spec file or its cost/cycle edit; from `git diff --name-only origin/main...HEAD`):
  - `src/cli/build.rs` — resolver (`build_root`, `fold_case`, `resolve_out_dir`, `root_relative`, `join_rel`, `first_link`, `OUTPUT_EXTENSIONS`), `check_destinations`/`reads_of`, `write_planned`, root-relative lock path, `--watch` exclusion and read carve-out, unit rows (FT-28, 32, 33, 34)
  - `src/cli/mod.rs` — three exit-2 variants, `code()` arms, `exit_code_mapping_is_total` rows, `build` and `--check` help text
  - `src/build/mod.rs` — `Target::validate` comment
  - `tests/build_destinations.rs` — new: FT-1 to FT-27
  - `tests/build_watch.rs` — FT-29, FT-30
  - `docs/cli-reference.md` — §build, §--watch, lockfile paragraphs, exit-code row 2
  - `SECURITY.md` — threat 3 sentence and row 3
  - `skills/crustyimg/SKILL.md` — the `build` overwrite sentence
  - `CHANGELOG.md` — Changed and Fixed entries, verbatim
  - `Cargo.toml`, `Cargo.lock`, `CACHE_SCHEMA_VERSION`, lock version: untouched (empty `git diff --stat`).
- **RED baseline** (main's code, src untouched): FT-5, FT-19, FT-20, FT-27 green; 23 other destination rows red for their reason (exit 0 or exit 7 instead of 2, or lock path unchanged); FT-22 failed at its `assert_ne`; FT-29 saw 15 builds in a 3 s window; FT-30 saw only one `(0 cached, 1 rebuilt)`. FT-28, FT-31 to FT-34 did not exist.
- **Behaviour drives** (real binary): collision via `alias/../dist` exit 2; `out = "src"` exit 2; `.crustyimg/x` exit 2; out-of-tree link exit 2 with the outside dir empty; in-tree link exit 0 and lock `build/dist/a.png`; `./dist/` and `a/../dist` lock as `dist/...` exit 0; `--check` exit 0, exit 7 on drift, exit 2 with nothing written; `--check` after deleting an output and `.crustyimg/` wrote both back (exit 0) and left the lock sha unchanged; sink re-check exit 5 (FT-28 asserts `code() == 5`).
- **Upgrade drive:** main's binary built through an in-tree link locked `dist/a.png`; this branch's `--check` exited 7 with `drift: "build/dist/a.png": not in the lockfile` and `drift: "dist/a.png": in the lockfile, not built`; one plain build rewrote it to `build/dist/a.png`; `--check` then exited 0.
- **Mutations** (each in a scratch worktree, clean rebuild, full suite): M1, M2, M3, M3b, M4, M5, M6, M7 (24 red, exactly the table's set), M8, M9, M10, M13, M14 match the table. M11 all green (equivalent on Unix). **M12 differs:** FT-28 (`write_planned_refuses_...`) is also red, because the mutant's inserted containment check in `write_planned` fires on that test's temp dir; the table lists only FT-25. Trap 14 (main's `build.rs`): the 25 rows in the table red, FT-5, 19, 20, 27 green.
- **Matrix:** default, `--no-default-features`, `--features webp-lossy`, each a fresh target dir: tests, clippy `-D warnings`, `fmt --check` all rc=0. `just wasm-check` rc=0 (one pre-existing dead-code warning on `MANIFEST_DROPPED_WARNING`, not this change).
- **Overlap after last rebase:** open PRs #240, #237, #229, #226 touch none of my files except #226 on `src/cli/mod.rs` (module doc only) and #237 on `CHANGELOG.md`.
- **Deviations from spec:**
  - Stages 2 and 3 were committed and pushed together (one commit), not separately.
  - `check_output_injective` does not take `root` (not needed once `rel_dir` is set).
  - The new variants and the CHANGELOG are as specified; no DEC file written.
- **Follow-up work identified:** none beyond the spec's filed NFC/NFD item.

### Punch list (verify)

Starting point: PR #270 head `9733cd8` (as the prompt said). Everything is on `feat/spec-157-build-destinations`.

**Fix 1: an existing output that is a read file is refused by file identity.** `reads_of` now returns `Reads`,
with a second index `by_file` (device and inode, from `std::fs::metadata`, links followed; `cfg(unix)`). After the
folded-string lookup misses, `check_destinations` stats the planned output and refuses on a hit with that read's
clause. Windows skips it (NTFS does not merge `ß`/`SS` and does not normalize, so the folded name is the identity
there; *reasoned, not driven*).
- Red on `9733cd8` (driven): FT-35, FT-36, FT-37 each exit 0 with the source replaced (`left: Some(0), right: Some(2)`).
- Green after: all three. FT-35 and FT-36 ran on this APFS volume (not skipped; checked with `--nocapture`).

**Fix 2: a glob or folder source that would list its output is refused on the first build.** `listers_of` builds,
per target, a `Lister` for each glob or folder source argument; the pass in `check_destinations` refuses with
`which target #j's source pattern `…` would read on the next build` (or `source folder`). The glob's literal prefix is
normalized the way `out` is (`resolve_out_dir`, then `root_relative`), the rest is matched as written, with `glob`'s
walk options except `require_literal_separator: true` (the walk matches one component at a time). The extension
filter uses `READABLE_EXTENSIONS`, a copy of the source module's private list (`src/source/` may not change);
FT-52 drives it against the real filter so drift fails a test.
- Red on `9733cd8` (driven by hand): `src/*.png`, `out = "src"`, `{stem}_w.{ext}`: first build exit 0, second build
  exit 2 (`target #0 would overwrite "src/a_w.png", which target #0 reads as a source`). FT-38, FT-39, FT-41, FT-42 red
  (exit 0); FT-40 green (pin).
- Green after: all of them, and FT-40 builds twice.

Shapes table (every cell driven on this branch with the real binary, `name = "{stem}_w.{ext}"`; each cell exit 2
with the new clause, tree unchanged):
- Sources `src/*.png`, `./src/*.png`, `src/**/*.png`, `src`, `src/`: each against out `src`, `./src`, `src/`,
  `a/../src` (FT-41).
- Sources `src/*.png`, `lnk/*.png` against out `src`, `lnk` (`lnk` a link to `src`) (FT-42).
- `SRC/*.png` against every out: refused on this APFS volume (driven by hand); on a case-sensitive filesystem `SRC`
  matches nothing and the build stops earlier with "no images matched", so no row (reason, not a row).
- `x/../src/*.png`: `x` does not exist, so the glob lists nothing: "no images matched" before planning. No row.
- A glob that does not match the output (`{stem}.webp` against `src/*.png`): FT-40.
- A folder source is non-recursive: an output in a sub-folder of the source folder is not refused (by
  `lister_reads`, `!name.contains('/')` and the folder compare); no row, *reasoned*.

**Fix 3: rows for the uncovered cells** are FT-43 to FT-51, all green on `9733cd8` (pins). Mutants driven one at a time:
M17 red FT-43; M18 red FT-44; M19 red FT-45.

**Fix 4: wording.** Exit 6 for a moved destination in `SECURITY.md` and the spec; line 42's exit 5 is the sink's
link refusal, which is exit 5 on `apply` and exit 6 on `build` (driven: FT-47, a link planted at the output file,
exits 6 with `1 of 1 inputs failed`, the link's target unchanged). The moved-folder race cannot be driven
deterministically end to end (no hook between planning and the write), so FT-28 stays its row. M12's cell now
lists FT-28 and why. The "one component" sentence is corrected in the DEC body, the `write_planned` doc comment and
the build reflection: `name = "src/a.{ext}"` with `out = "."` exits 2 (overlap with `src/a.png`), with `out = "dist"`
exits 6 (`safe_join` at write time) (both driven).

Mutations (each alone, `cargo clean -p crustyimg`, full `cargo test --no-fail-fast`, stdout redirected, own
`CARGO_TARGET_DIR`; red set read off the run, every run rc=101):
- **M15** (3 red): FT-37, FT-35, FT-36.
- **M16** (4 red): FT-38, FT-39, FT-41, FT-42. **M16glob** (3 red): FT-38, FT-41, FT-42.
- **M17** (1 red): FT-43. **M18** (1 red): FT-44. **M19** (1 red): FT-45.
- **M7** (39 red, was 24): every refusal row, including the 15 new ones (FT-35 to FT-39, FT-41, FT-42, FT-43 to FT-46,
  FT-48 to FT-51) and `colliding_stems_in_one_target_are_rejected`, `collision_across_two_targets_is_rejected`.
  Still red where the table says; FT-20, FT-21, FT-22, FT-26, FT-27, FT-40, FT-47 green.
- **TRAP14** (40 red): the 25 of the table, plus the new refusal rows that `main` cannot make. FT-5, FT-19, FT-20,
  FT-27 still green.

Upgrade re-drive: `main`'s binary built through the link `dist → build/dist` wrote `path = "dist/a.png"`; this branch's
`build --check` exited 7 with `drift: "build/dist/a.png": not in the lockfile` and
`drift: "dist/a.png": in the lockfile, not built`; one build ran with the warm cache hitting (`1 cached, 0 rebuilt`)
and rewrote the lock to `build/dist/a.png`; `--check` then exited 0.

Matrix (each leg a fresh `CARGO_TARGET_DIR`): default, `--no-default-features`, `--features webp-lossy`: `fmt --check`,
`clippy --all-targets -D warnings` and the full `cargo test --no-fail-fast`, all rc=0. `just wasm-check` rc=0.

CI snapshot: CI_SNAPSHOT_PLACEHOLDER

### Build-phase reflection (3 questions, short answers)

Process-focused: how did the build go? What friction did the spec create?

1. **What was unclear in the spec that slowed you down?**
   — Nothing blocking. Message wording and mutation edits were exact enough to apply as written; the only gap was M12's red set missing FT-28.

2. **Was there a constraint or decision that should have been listed but wasn't?**
   — No. Windows `\\?\` paths reject `/`, so the read-set lookup joins the name onto `out_dir` rather than a `/` string; worth one line in the design.
   The name is not always one component: `name = "src/a.{ext}"` reaches `out_dir.join` as written and is refused
   as an overlap (exit 2) when `out_dir/src/a.png` is a read, and by `safe_join` at write time (exit 6)
   otherwise *(corrected at punch list)*.

3. **If you did this task again, what would you do differently?**
   — Commit stages 2 and 3 separately for the checkpoint trail. No design call turned out wrong.

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
