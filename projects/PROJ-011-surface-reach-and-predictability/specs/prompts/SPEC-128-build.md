# SPEC-128 — BUILD prompt

Cycle: **build**. New session, own worktree, branch `feat/spec-128-recipe-watermark`. **Sonnet.**
You did not design this — the spec carries the context (AGENTS §15).

**What you are building:** a recipe can express `watermark` in both modes, because the registry
seam learns to construct an operation whose parameters name a **file** — without
`src/operation/**` ever touching the filesystem.

```
git worktree add -b feat/spec-128-recipe-watermark \
  ~/PSeven/experiments/crustimg_redo_plus/crustyimg-spec128 main
```

## Read in order, before any code

1. **The spec** — `specs/SPEC-128-recipes-can-express-watermark.md`. Its four design calls are
   **settled**. If one is wrong, say so in Build Completion; do not quietly re-decide it.
2. **DEC-031** — the premise. It says *why* watermark is unregistered. This spec exists to remove
   that constraint properly, not to ignore it.
3. **DEC-064** (engine/shell split, the wasm32 rule), **DEC-005** (recipe round-trip),
   **DEC-002** (decode-once), **DEC-099** (SPEC-127's precedence chain — do not disturb it).
4. `STAGE-050-recipe-reach.md`, the project `brief.md`, `/guidance/constraints.yaml`.

## Reserved

📌 **DEC-100 is yours.** Highest on `main` is DEC-099. Use the **block-list** form:

```yaml
affected_scope:
  - src/operation/registry.rs
```

⚠ **Not** an inline array — `scripts/decisions-audit.sh` silently drops those, which is why DEC-015
is invisible to `--changed` (filed, PROJ-013 STAGE-047). A DEC written inline governs nothing and
nothing warns you.

## Order

1. **Baseline first.** Write the eight failing tests, run them against pristine `main`, and record
   that all eight are RED **and why**. A test never seen red is not a guard.
2. Implement smallest-seam-first: the resolve step, then watermark registration, then the wasm
   refusal.
3. **Push a WIP commit as soon as it compiles**, before the matrix.

## The four things most likely to go wrong

### 1 — ⚡ Bytes must never reach `params()` or `to_toml`

`to_toml` emits the **path**. If loaded overlay/font bytes round-trip into the params, every recipe
becomes enormous and `from_toml(to_toml(r)) == r` breaks. This is the same shape as SPEC-127's
`to_toml`-must-emit-`"1"` guard and it is the highest-consequence line here.
`watermark_recipe_round_trips_with_path_not_bytes` is that test.

### 1b — ⚡ Text mode's `params()` is WRONG today; fixing it is in scope

`watermark_overlay` (`src/cli/ops.rs:1220-1262`) returns the overlay pixels **plus a label**, and
for `--text` that label **is the text**. `Watermark::params()` writes it under the key **`image`**
— the field that means *file path*. Nothing has broken because watermark is unregistered; this
spec is what makes it break.

Give the two modes **distinct, non-overlapping keys**: image mode keeps `image = "<path>"`; text
mode emits `text`, `font` (optional), `size`, `color`, and **must not emit `image`**. `Watermark`'s
single `overlay_path: String` slot cannot express both — **widening that struct is part of this
spec, not a follow-up.** A step with both sources, or neither, is a typed error at build-pipeline
time, matching the CLI's existing XOR.

⚠ **AC-2 asserts the round-tripped step renders the same PIXELS**, not merely that the TOML parses
— a parseability-only test passes on today's broken behaviour.

### 2 — `apply` and `build` must use the SAME resolver

Two paths resolving the same thing separately is exactly how they drifted before SPEC-126. One
resolver, called from both. AC-4 is what catches a violation.

### 3 — `src/operation/**` stays filesystem-free

It compiles for **wasm32** (DEC-064). Verified at design time: a grep for `std::fs`, `Image::load`
and `File::open` under `src/operation/` returns **nothing today** — AC-8 is a guard on that, so
keep it empty and run `just wasm-check`.

### 4 — AC-10's sweep must follow the CALL GRAPH, not the file

⚠ SPEC-127's verify caught the orchestrator narrowing a sweep wrongly: `run_convert`,
`run_optimize` and `run_web` all reach `run_pixel_op` even though their own hunks sit elsewhere.
**Being unchanged in one file says nothing about what it calls in another.** Sweep every verb that
reaches the code you touched, on ≥4 files across 2 formats, two binaries, with a **positive
control** proving the comparison can detect a difference. State the corpus boundary in DEC-100.

## Also required

- **`docs/api-contract.md` and `docs/data-model.md` in the same change.** SPEC-127's build did this
  correctly — match it. The decisions audit cannot warn you if you forget.
- **AC-9's negative controls: one revert per independent condition** (Calls 1, 2, 3), each reverted
  **alone**, each flipping only its own tests. Evidence is the **behavioural flip**, never a hash.
- **AC-7 is the seam's real test:** register a *fixture* asset-bearing op through the same resolve
  path and assert the seam did not have to change for it. Not the real LUT op — that is explicitly
  out of scope (Call 4).
- **AC-11's matrix**: default, `--no-default-features`, `--features webp-lossy`, fresh
  `CARGO_TARGET_DIR` each, **sequential**; clippy + `fmt --check` each; plus `just wasm-check`.

## Guardrails

- ⛔ **Do NOT bump the crate version, do NOT cut a release.** Batches into PROJ-011's single
  lockfile migration with the rest of STAGE-050.
- ⚠ **Never run a backgrounded feature build while editing source for a revert.** SPEC-127's build
  contaminated a leg exactly that way and had to re-run it from a clean tree.
- **Never poll CI.** Background `gh pr checks --watch`; when it exits, read a direct snapshot at the
  **true head SHA** — the watch summary line has been unreliable here.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: **redirect stdout**,
  do not "fix" it. A piped command reports the **pipe's** exit code — redirect and read `$?`.
- zsh does **not** word-split unquoted parameters — build argument lists explicitly, use
  `while IFS= read -r`, and prefer `/usr/bin/grep`. macOS has no `timeout(1)`.
- **Budget ~250 exchanges.** Sized honestly: the code here is M, the verification is heavier than
  the code. Blowing the budget openly beats rushing a control.

## When you finish

1. Fill `## Build Completion` — **every file from `git diff --name-only`, not recall** (SPEC-127's
   list was short by 3), plus an honest reflection naming any design call that turned out wrong.
2. **Measure your own cost and put it in your return message.** Your transcript is at
   `~/.claude/projects/<cwd-slug>/<session-id>.jsonl`, session id = the last path component of your
   scratchpad dir. ⚠ **Dedup by `.message.id`**: Claude Code writes one line per *content block*,
   and lines sharing an id repeat identical `input`/`cache_creation`/`cache_read` while
   `output_tokens` grows — take those three from the group and **MAX** output. Summing every line
   over-counts ~2×; taking the first line's output under-counts ~11×. Both mistakes are on record.
   Price per component (Sonnet $3/$15 per MTok, cache_creation ×1.25, cache_read ×0.10), never flat.
3. `just advance-cycle SPEC-128 verify`; create DEC-100; open the PR with a conventional-commit
   title carrying the spec id and the AGENTS §13 body template.
