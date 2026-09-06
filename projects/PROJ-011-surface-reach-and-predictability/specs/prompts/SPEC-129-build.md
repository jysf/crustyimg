# SPEC-129 — BUILD prompt

Cycle: **build**. New session, own worktree, branch `feat/spec-129-build-cache-asset-hash`.
**Sonnet.** You did not design this — the spec carries the context (AGENTS §15).

**What you are building:** `build`'s per-target cache key covers the content of every asset a
recipe's steps name (watermark's `image`/`font` today), so editing an overlay on disk always
misses. A recipe with no asset-bearing steps must hash exactly as before.

```
git worktree add -b feat/spec-129-build-cache-asset-hash \
  ~/PSeven/experiments/crustimg_redo_plus/crustyimg-spec129 main
```

## Read in order, before any code

1. **The spec** — `specs/SPEC-129-build-cache-hashes-watermark-asset-content.md`. Its seven
   design calls are **settled**. If one is wrong, say so in Build Completion; do not quietly
   re-decide it.
2. **DEC-058** — the cache-key composition this spec's DEC-101 amends. Its **"Revisit if"** clause
   names exactly this case, and Call 5 explains why we depart from its recommended remedy.
3. **DEC-100** — SPEC-128's seam. This spec is the promised follow-up; the mechanism it exposed
   (`OperationParams::resolved_bytes`) is the mechanism this spec reads from.
4. **DEC-031** (why `src/operation/**` cannot open files — the constraint that keeps the fix out
   of that tree), **DEC-064** (engine/shell split), **DEC-005** (recipes round-trip through the
   parsed form).
5. `STAGE-050-recipe-reach.md`, the project `brief.md`, `/guidance/constraints.yaml`.

## Reserved

📌 **DEC-101 is yours.** Highest on `main` is DEC-100. Use the **block-list** form:

```yaml
affected_scope:
  - src/cli/build.rs
  - docs/api-contract.md
```

⚠ **Not** an inline array — `scripts/decisions-audit.sh` silently drops those, which is why
DEC-015 is invisible to `--changed` (filed, PROJ-013 STAGE-047). A DEC written inline governs
nothing and nothing warns you.

DEC-101 **amends DEC-058 clause 4**, not supersedes it — the seven-input key composition is
unchanged; the definition of "canonical recipe hash" widens to include resolved-asset content.
`supersedes: null`; add a line in the body that names DEC-058's clause and explains **why the
schema version is NOT bumped** (the remedy DEC-058 suggests): the new input applies only to
asset-bearing recipes, and bumping would over-invalidate every existing plain-recipe cache
entry. AC-4 pins that.

## Order

1. **Baseline first.** Write the eight failing tests, run them against pristine `main`, and
   record that all eight are RED **and why**. A test never seen red is not a guard. **AC-1's
   RED baseline is the SPEC-128-verify reproduction** — the same shape reproduces here.
2. Implement smallest-seam-first: the helper, then the branch merge in `target_recipe_hash`,
   then the signature widening at the one caller (`prepare_target`).
3. **Push a WIP commit as soon as it compiles**, before the matrix.

## The three things most likely to go wrong

### 1 — ⚡ A watermark-free recipe MUST hash the same as before

This is the property that keeps every existing plain-recipe cache entry valid. Assert it
directly with `target_recipe_hash_matches_recipe_hash_for_watermarkfree_recipe` — for a
`resize`-only recipe, the new `target_recipe_hash(&recipe, Preserve, &registry)` returns the
**same digest** as `cache::recipe_hash(&recipe)` today.

If this test is red, the fix over-invalidated and Call 5 is broken. **This is the single line
that decides whether existing users pay for this fix.** Test it before AC-1.

### 2 — ⚡ Read the resolved bytes from `OperationParams`, do NOT re-read the file

`resolve_recipe_assets` at `src/cli/common.rs:207-228` reads the overlay/font once per target
and attaches the bytes via `set_resolved_bytes`. The hasher must call `resolved_bytes(key)`,
not `std::fs::read` — a second read is a wasted syscall AND a race (a designer editing
`logo.png` between the resolver and the hasher would poison the key with bytes different from
the ones the recipe will actually render with).

### 3 — AC-6's sweep follows the CALL GRAPH, not the file

Same SPEC-128 discipline: `run_convert`, `run_optimize`, `run_web` all reach `run_pixel_op`
even though their hunks sit elsewhere, and the sweep must cover every verb that reaches the
code you touched. **Being unchanged in one file says nothing about what it calls in another.**
Sweep every pixel-lane verb on ≥4 files across 2 formats, both binaries as release with a
fresh `CARGO_TARGET_DIR` each, with a **positive control** proving the comparison can detect a
difference — the SPEC-128-verify reproduction is that control (main caches → stale; branch
rebuilds → new bytes). State the corpus boundary in **DEC-101**, not just Build Completion.

## Also required

- **`docs/api-contract.md` in the same change.** One paragraph under the content-addressed cache
  prose (~`docs/api-contract.md:559`), naming resolved-asset content as part of the cache key
  for asset-bearing recipes. **Do NOT touch `docs/data-model.md`** — the recipe schema is
  unchanged.
- **AC-8's negative control: one revert per independent condition.** The change has ONE
  testable condition — the added asset absorption in `target_recipe_hash`. Revert **only** that
  and confirm AC-1 and AC-3 flip RED while AC-2/AC-4/AC-5/AC-6 stay GREEN. Evidence is the
  **behavioural flip**, never a hash.
- **AC-10's matrix**: default, `--no-default-features`, `--features webp-lossy`, fresh
  `CARGO_TARGET_DIR` each, **sequential**; clippy + `fmt --check` each; plus `just wasm-check`,
  `just wasm-test`, and `just demo-build` (the wasm bundle-size gate SPEC-128's build got wrong
  — this spec should NOT move it, but run the guard).
- **AC-7: the perf property.** The asset is hashed **once per target**, not once per input. A
  target with N=10 inputs sharing one overlay must trigger exactly ONE overlay hash. Test it by
  asserting `target_recipe_hash` is called once per target (it already is — the call site is
  `prepare_target`, not `build_one`), or by counting invocations of a wrapped hasher.

## Guardrails

- ⛔ **Do NOT bump the crate version, do NOT cut a release.** Batches into PROJ-011's single
  lockfile migration with the rest of STAGE-050.
- ⛔ **Do NOT bump `CACHE_SCHEMA_VERSION`.** Call 5 is the specific ruling here, and AC-4 is its
  guard. DEC-101 explains the departure from DEC-058's suggested remedy.
- ⚠ **Never run a backgrounded feature build while editing source for a revert.** SPEC-127's
  build contaminated a leg exactly that way and had to re-run it from a clean tree.
- **Never poll CI.** Background `gh pr checks --watch`; when it exits, read a direct snapshot
  at the **true head SHA** — the watch summary line has been unreliable here.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: **redirect
  stdout**, do not "fix" it. A piped command reports the **pipe's** exit code — redirect and
  read `$?`.
- zsh does **not** word-split unquoted parameters — build argument lists explicitly, use
  `while IFS= read -r`, and prefer `/usr/bin/grep`. macOS has no `timeout(1)`.
- **Budget ~120 exchanges.** Sized honestly: the code here is S (one helper, one branch merge,
  one signature widening), the verification is heavier than the code but not L-heavy — AC-6's
  sweep is the same 41/41 shape SPEC-128 already ran.

## When you finish

1. Fill `## Build Completion` — **every file from `git diff --name-only`, not recall** (SPEC-127's
   list was short by 3, SPEC-128's was short by 1), plus an honest reflection naming any design
   call that turned out wrong.
2. **Measure your own cost and put it in your return message.** Your transcript is at
   `~/.claude/projects/<cwd-slug>/<session-id>.jsonl`, session id = the last path component of
   your scratchpad dir. ⚠ **Dedup by `.message.id`**: Claude Code writes one line per *content
   block*, and lines sharing an id repeat identical `input`/`cache_creation`/`cache_read` while
   `output_tokens` grows — take those three from the group and **MAX** output. Summing every
   line over-counts ~2×; taking the first line's output under-counts ~11×. Both mistakes are on
   record. Price per component (Sonnet $3/$15 per MTok, cache_creation ×1.25, cache_read ×0.10),
   never flat. **Emit a `tokens_breakdown`** — SPEC-128's original build entry lacked one and
   couldn't be checked.
3. `just advance-cycle SPEC-129 verify`; create DEC-101; open the PR with a conventional-commit
   title carrying the spec id and the AGENTS §13 body template.
