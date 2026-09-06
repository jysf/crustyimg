# SPEC-129 — VERIFY prompt

Cycle: **verify**. New session, **read-only**. You did not build this.

**What it claims:** `build`'s per-target cache key now covers the content of every asset a
recipe's steps name; a recipe with no asset-bearing steps hashes identically to `main`;
nothing else changes bytes. The one-file surface (`src/cli/build.rs`) is the whole point.

## ⚠ The PR is OPEN and must NOT merge

Base is `main`. Branch is `feat/spec-129-build-cache-asset-hash`.

```
git worktree add --detach ~/PSeven/experiments/crustimg_redo_plus/crustyimg-spec129-verify feat/spec-129-build-cache-asset-hash
```

⛔ **Do not merge, do not bump the version, do not cut a release.** Batches with the rest of
STAGE-050 as PROJ-011's single lockfile migration. **Make no commits** — verdict and
`## Cost readout` go in the return message (AGENTS §13).

## Six specific things

### 1 — ⚡ THE OVER-INVALIDATION GUARD — check this before anything else

**The property that decides whether existing users pay for this fix**: a recipe with no
asset-bearing steps must hash identically to `main`. AC-4 is that guard, and its unit test
(`target_recipe_hash_matches_recipe_hash_for_watermarkfree_recipe`) is the single assertion
that keeps every existing `.crustyimg/cache/` entry valid.

**Drive it yourself.** Build both binaries as release, run `build` against a `resize`-only
recipe on `main` first (populates a cache), then on the branch: the second run must report
`(N cached, 0 rebuilt)`. A regression here would mean every existing plain-recipe build silently
rebuilds on upgrade — a real user cost — while the "fixes a watermark bug" framing hides it.

### 2 — ⚡ The DEC-101 departure from DEC-058

**DEC-058's Revisit clause says bump `CACHE_SCHEMA_VERSION` when an output-affecting input is
discovered outside the seven.** This spec deliberately does NOT bump. DEC-101 must:

- name DEC-058's clause explicitly (not just "amends DEC-058"),
- state the specific reason for the departure (the new input applies only to a **subset** of
  recipes; bumping over-invalidates every existing plain-recipe entry),
- cite AC-4 as the guard,
- state the corpus boundary of AC-6's sweep.

If any of the four is missing, the audit cannot check the design; call it out.

### 3 — The bug is reproduced on `main`, refuted on the branch

The SPEC-128-verify reproduction is the positive control. Drive both binaries:

- **main**: build once with `logo.png`, mutate `logo.png` (same path, different bytes), build
  again → `(1 cached, 0 rebuilt)`, output byte-identical to first run. **The bug.**
- **branch**: same sequence → `(0 cached, 1 rebuilt)`, output reflects new overlay bytes.

Confirm this yourself; it is what makes AC-1's test a real guard and not a plausible one.

### 4 — AC-3: the `font` half of the mechanism has its own test

SPEC-128's AC-5 shipped half-tested because "overlay/font" was tested only on overlay. Read
`build_rebuilds_when_font_bytes_change` and `build_hits_when_bundled_font_is_used`; drive both.
The `font` key must produce the same behavior as `image`, and a text watermark with **no**
`font` key (bundled default) must produce cache hits (nothing to resolve, nothing to hash) —
this last is the subcase that would tell you whether the hasher misfires on an absent asset.

### 5 — AC-6's sweep, and whether its boundary is stated

The 41/41-style byte-identical sweep. ⚠ **Confirm the sweep followed the CALL GRAPH, not the
file** — `run_convert`, `run_optimize` and `run_web` all reach `run_pixel_op`, which is what
the orchestrator got wrong on SPEC-127. Confirm the positive control is real (a case known to
differ, shown differing — the SPEC-128 reproduction), and that DEC-101 states the corpus
boundary rather than implying a whole-surface sweep.

### 6 — AC-7: hashed once per target, not once per input

**The performance property Call 1 rests on.** If the asset is hashed once per input (N inputs
= N SHA-256 passes over the overlay), the fix is unnecessarily expensive for a big photo tree.
Read the test and confirm the assertion counts calls, not just correctness — one target with
N=10 inputs sharing one overlay must trigger exactly ONE overlay hash. The call site
(`prepare_target`, once per target) already gives this by construction; the test is the pin.

## Also check

- **DEC-058 is the premise this spec amends** (via DEC-101). Confirm DEC-101 either names it in
  `supersedes` or explicitly amends a specific clause — a decision silently contradicted is
  worse than one reopened.
- **`CACHE_SCHEMA_VERSION` is unchanged.** `git diff src/build/cache.rs` on the branch should
  not touch that const. If it does, Call 5 was contradicted.
- **The whole change is under `src/cli/build.rs`.** AC-9 is a hard boundary — a grep for
  `std::fs`/`Image::load`/`File::open` under `src/operation/` must stay empty. `just wasm-check`
  must pass.
- **The wasm bundle-size gate did NOT move.** SPEC-128's build moved it 10.6 %; this spec has
  no reason to touch wasm and `WASM_BROTLI_BASELINE` in `scripts/lib/wasm-artifact.mjs` should
  be unchanged. Diff it. **If it moved, DEC-066's ledger needs a row** — check DEC-101 for it.
- **Decision drift:** `./scripts/decisions-audit.sh --changed main` — **pass the base ref**, or
  a clean checkout exits 0 on a green that cannot go red. ⚠ It **cannot see DEC-015** (inline
  `affected_scope`, filed PROJ-013 STAGE-047).
- **Every file in Build Completion**, from `git diff --name-only`, not recall.
- **AC-10's matrix**: default, `--no-default-features`, `--features webp-lossy`, fresh
  `CARGO_TARGET_DIR` each, sequential; clippy + `fmt --check`; plus `just wasm-check`,
  `just wasm-test`, and `just demo-build`.
- **`docs/api-contract.md`**: the one added paragraph is a testable claim about a shipped
  binary. Drive it. Documentation has no green.
- ⚡ **The measure that matters, once more:** two `.crustyimg/cache/` runs — plain-recipe
  hits on main and branch (AC-4 property), watermark-recipe miss on branch when overlay
  changes (AC-1 property). These are the two behaviors that keep the fix from being either
  ineffective or expensive.

## Guardrails

- **Read-only. No commits. Do not fix what you find.** Do not merge or bump the version.
- **Budget ~120 exchanges.**
- `cargo test` fails `display_sink_refuses_non_tty` interactively — redirect stdout, do not
  "fix" it. A piped command reports the **pipe's** exit code — redirect and read `$?`. zsh
  does not word-split unquoted parameters. Use `/usr/bin/grep`. macOS has no `timeout(1)`.
- ⚠ **Do not run a backgrounded feature build while editing source for a revert** — SPEC-127's
  build contaminated a leg that way.

## When you finish

**Measure your own cost from your transcript** at
`~/.claude/projects/<cwd-slug>/<session-id>.jsonl` (session id = last path component of your
scratchpad dir). ⚠ **Dedup by `.message.id`**: one JSONL line per *content block*; lines
sharing an id repeat identical `input`/`cache_creation`/`cache_read` while `output_tokens`
grows. Take those three from the group and **MAX** output. Summing every line over-counts ~2×;
taking the first line's output under-counts ~11×. **Emit a `tokens_breakdown`** — SPEC-128's
original build entry lacked one and couldn't be checked. Price per component at Opus anchors
($5/$25 per MTok, cache_creation ×1.25, cache_read ×0.10), never flat.

End with **✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED**, leading with **item 1** — whether the
over-invalidation guard (AC-4) holds and is directly driven.
