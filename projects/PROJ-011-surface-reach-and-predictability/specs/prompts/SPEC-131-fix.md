# SPEC-131 — FIX prompt: the verify punch list (follow-up PR)

Cycle: **build** (punch-list follow-up). Fresh CLI session, own worktree, **Sonnet** (three small,
fully specified items):

```
just wt fix/spec-131-punch-list -- --model sonnet
```

You did not design, build or verify SPEC-131. SPEC-131 is already on `main` (`ba8b740`, PR #200).
Its retroactive verify returned a punch list, recorded in the spec's `## Verify Readout`. This PR
fixes exactly those three items and nothing else. The maintainer pasted this prompt into a fresh
session and will paste your final message back to the orchestrator, so it must stand on its own.

## Step 0
`git fetch origin && git rebase origin/main`. Read `## Verify Readout` in
`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-131-typed-per-operation-parameter-structs.md`,
then DEC-102, then `src/operation/registry.rs` (`ParamKind::Number`'s check, ~lines 150-172).

## The three items

### 1 — Regression: a tiny positive number passes the range check, then reads as 0
The schema compares the `f64` against `min`/`max`, but the ops read the value as `f32`, and the
build removed the constructors' own range checks. So `1e-50 > 0` passes while `1e-50 as f32 ==
0.0`. Measured by verify, release `apply --recipe`, before SPEC-131 vs after:
- `percent = 1e-50`: exit 1 → **exit 0, writes a 1×1 image**
- `size = 1e-50`: exit 1 → **exit 0, no watermark** (byte-equal to an identity recipe)
- `scale = 1e-50`: exit 1 → **exit 0**

**Fix:** apply the `min` / `min_inclusive` / `max` comparison to the value **as the op will read
it**, i.e. `(n as f32) as f64`, as well as to `n`. Keep the existing finiteness checks. The error
keeps today's shape (`'key' must be …, got 1e-50`, showing the value as written).
**Tests:** add the three `1e-50` rows to `a_number_past_f32_range_is_refused` (or a sibling table
next to it) in `tests/recipe_strict_params.rs`, with the table's row-count assertion updated.
Write them first, run on unmodified `src/`, and confirm all three are **RED because the recipe
loads**. Then fix, then GREEN.
**Negative control:** revert only the `as f32` comparison, rebuild, confirm exactly these rows go
RED and nothing else; restore.

### 2 — Missing coverage: a one-step recipe whose only step is a keyed `optimize` marker
Mutation M10 (skip the terminal-marker check on a one-step recipe) turned no test red; the only
marker test uses three steps. `main` behaves correctly today, so this is coverage only. Add a row
to `terminal_optimize_marker_with_a_key_is_refused`: a recipe with exactly one `[[step]]`,
`op = "optimize"`, `quality = "max"` → refused at `Recipe::from_toml`, naming `optimize` and
`quality`. It is GREEN on `main` (a guard). **Prove it is a real guard:** apply M10 yourself
(make the marker check skip when the recipe has one step), confirm the new row goes RED, restore.

### 3 — CHANGELOG Breaking line wording
The SPEC-131 Breaking line in `CHANGELOG.md` `[Unreleased]` says such recipes "used to run,
ignoring the key". That is untrue for wrong-typed **required** keys (already refused, with a
message that did not name the key), and it omits keys on the terminal `optimize` step. Reword it
to cover both, e.g. "…used to run ignoring the key, or fail with a message that did not name it;
this includes keys on a recipe's final `optimize` step." Edit only that line.

## Boundaries
- Touch only `src/operation/registry.rs`, `tests/recipe_strict_params.rs` (and `src/recipe/mod.rs`
  only if the marker test lives there), and that one `CHANGELOG.md` line. No other `src/`, no DEC
  change (DEC-102 already says values are checked as the op reads them; if it does **not**, say so
  in your report rather than editing it).
- Do not edit the spec or its timeline: they live on `main` and the orchestrator keeps them.
- SPEC-134's build is running in parallel on `src/cli/**` / `src/sink/**`; you share only
  `CHANGELOG.md`. Rebase onto `main` before opening the PR if it moved.

## Finish
1. Clean matrix: default, `--no-default-features`, `--features webp-lossy`, each in a **fresh**
   `CARGO_TARGET_DIR`, **sequential** (a reused target dir silently compiled stale code during
   SPEC-131's verify); tests + `clippy --all-targets -D warnings`; `fmt --check`; `just wasm-test`.
2. Drive the three `1e-50` recipes on a release binary: each now exits 1 with nothing written.
3. Every commit `git commit -s` (DCO). Open the PR **as a draft**:
   `gh pr create --draft --title "fix(recipe): check numbers as the op reads them; punch list (SPEC-131)"`,
   body per AGENTS §13 (Summary of the three items, Spec metadata, Decisions referenced: DEC-102,
   New decisions: none), ending with the Claude Code footer. ⛔ Never mark it ready yourself: a
   separate re-verify session approves it first.
4. **Never poll CI.** Run `gh pr checks <PR> --watch` in the background; when it exits, read a
   direct snapshot at the true head SHA and check every leg.
5. `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to
   a file. A piped command reports the **pipe's** exit code. zsh does not word-split unquoted
   parameters. macOS has no `timeout(1)`.
6. **Budget ~60 exchanges.**

## Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/` and `.` replaced by `-`. Identify it **by content, not
recency**: `grep -l "SPEC-131 — FIX prompt" ~/.claude/projects/<slug>/*.jsonl`. ⚠ **Dedup by
`.message.id`**: one JSONL line per *content block*; lines sharing an id repeat identical
`input_tokens` / `cache_creation_input_tokens` / `cache_read_input_tokens` while `output_tokens`
grows. Take those three once per id and the **MAX** `output_tokens`. Price each component at the
model that actually ran (Sonnet $3/$15 per MTok, Opus $5/$25; cache_creation ×1.25 the input rate,
cache_read ×0.10), never flat. Report it; the orchestrator records it on `main`.

## Report back (your final message — it is pasted to the orchestrator)
First line: `SPEC-131 fix: draft PR #NNN at <head SHA> — CI <green | red: legs>`. Then: the three
items (done / not done); the three `1e-50` rows RED before and GREEN after; the two mutation flips
(the `as f32` revert, M10); files changed; whether DEC-102 already covers item 1; your cost
(`tokens_total`, `tokens_breakdown`, `estimated_usd`, rate anchors).
