# SPEC-131 — VERIFY prompt: recipe step params are validated at load

Cycle: **verify**. Fresh CLI session, own worktree, **Opus**, **read-only**. You did not design or
build this.

```
just wt verify/spec-131 -- --model opus
```

⚡ **This verify is retroactive.** PR #200 was **squash-merged to `main` on 2026-10-25 as
`ba8b740`** before verify ran (its head `48185da` differs from the build's reported `c98b032` only
by an "Update branch" merge of docs commits — no `src/` or `tests/` change). The fresh worktree
`just wt` gives you is already on `origin/main`, which contains the change: confirm
`git log --oneline | grep ba8b740` and record the `main` SHA you verified. The PR's individual
commits (e.g. `dff5159`, `d4af450`) are still reachable with
`git fetch origin pull/200/head:refs/remotes/pr/200`.

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

**What the build claims:** an unknown, wrong-typed, out-of-range (incl. `nan`/`inf`/past-f32) or
mode-unused key in a recipe step is refused at load, naming step index, op and key, on
`apply --recipe`, `build` and wasm `transform`; every valid recipe still loads with a byte-identical
`to_toml()`; CI is green on all 16 legs; all 15 ACs done; six negative controls flipped cleanly.

## ⚠ Already merged — so a finding becomes a follow-up fix, not a blocked merge
⛔ No version bump, no release (0.8.0 is not tagged, so anything you find is still fixed before
users see it). **Make no commits and push nothing** — your verdict and cost readout go in your
final message; the orchestrator turns a punch list into a follow-up fix PR. Do not fix what you
find; report it, with the evidence and a suggested fix.

## Read first
The spec (`projects/PROJ-011-surface-reach-and-predictability/specs/SPEC-131-typed-per-operation-parameter-structs.md`,
on `main`: all of it, including `## Build Completion`), DEC-102 (new), DEC-061 (accepted
risk #1 and its alternative, now marked superseded), DEC-005, DEC-058/DEC-101, and HANDOFF.md's
"Traps this wave paid for".

## Eight specific things — in this order

### 1 — ⚡ Nothing valid stopped working (the property that decides whether users pay)
- **The goldens are not fixtures from the code under test.** Check out **`ba8b740^`** (`main` just before the merge) into a
  scratch worktree, run the bless test there (`CRUSTYIMG_BLESS=1 cargo test bless_recipe_corpus_goldens
  -- --ignored`, per the spec), and diff the output against the PR's committed `.golden` files.
  Any difference means the goldens were captured after a `src/` change and AC-11(a) is hollow.
- **A plain recipe still hits the existing build cache.** Build both binaries as release (fresh
  `CARGO_TARGET_DIR` each). Run `build` on a resize-only project with the **pre-merge (`ba8b740^`)** binary
  (populates `.crustyimg/cache/`), then the **current `main`** binary on the same project: it must report
  every output cached, 0 rebuilt. Repeat with a valid watermark recipe. A miss means `to_toml()`
  changed and every existing cache entry is invalidated on upgrade.
- Drive every bundled recipe and every `recipes/README.md` example through the current `main` binary;
  each must load.

### 2 — ⚡ The build's deviation reaches the CLI verbs (not in the spec's ACs)
The build made **the constructors run the schema check** too, because `Resize::from_params` is
public and called directly. Find every caller outside the recipe path (grep `from_params` under
`src/cli/**`). If the `resize` / `watermark` / `thumbnail` verbs build their params through those
constructors, their flag errors changed. Drive, on **pre-merge (`ba8b740^`) vs current `main`**, at least:
`resize --width 0`, `resize --percent 0`, `watermark --opacity 1.5`, `watermark --opacity -0.1`,
`watermark --margin` with a large value, `watermark --size 0`, and a flag combination the new
unused-by-mode rule might now refuse. Record exit code + stderr for each on both binaries. Any
change on a direct CLI verb is **out of the spec's scope** (frame: "the direct CLI verbs' flags —
clap already types them") and must be listed: harmless improvement, or regression.

### 3 — Your own mutations, beyond the builder's six
The builder ran C1–C6. Run them again yourself (revert one, rebuild, run, restore), **and** add at
least three mutations of your own the table does not name, e.g.: let `Number` accept non-finite
values; make `UInt` accept negative `i64`; drop the step index from `build_pipeline`'s error;
make the terminal-marker check skip when the recipe has one step. For each, say which tests go
red. A mutation no test catches is a hollow spot — name it. ⚠ Reverting source does not rebuild
the binary.

### 4 — Count the nouns against the tests
AC-3 is 14 keys, AC-4 is 17 rows (plus the build's added past-f32 rows), AC-5 is 6 refused + its
accepted interplay rows, AC-7 is single **and** batch, AC-12 is five parts (a)–(e). Open each
table test and confirm its rows match the AC's list one for one and that it asserts its row count.
The build says 8 optional watermark keys silently defaulted before the merge, not the spec's 7: confirm
`font` is a row.

### 5 — The RED baseline is real
Check out commit `dff5159` (tests added, logic not yet changed) and run the new tests: each must
fail for the reason the spec gives, and each guard must pass. Confirm that `dff5159`'s edits to
`src/operation/registry.rs` and `src/recipe/mod.rs` are **test-module only** (no non-test code).

### 6 — The error a user sees
Drive `apply --recipe` and `build` with: a misspelt key (`opactiy`), a wrong type
(`opacity = "0.5"`), `size = nan`, and an unused key (`height` with `mode = "max"`). Each: exit 1,
nothing written, exactly one `error:` line naming `recipe step N ('op')` and the key, 0-based step
numbers. Then `just wasm-test` and confirm `transform_refuses_an_unknown_step_key` asserts the
same wording.

### 7 — Boundaries
- `git show --stat ba8b740` (the squash commit): the build reported 36 files, the PR diff had 37 —
  reconcile it against Build Completion's list. `src/cli/**`, `src/wasm.rs`, `src/build/**`,
  `tests/apply_batch.rs`, `tests/build.rs` must not appear.
- `tests/registry_seam.rs`'s pre-existing test is **unedited** (diff it).
- `src/operation/**` gains no `std::fs` / `std::path` (grep).
- **The wasm bundle gate did not move**: `WASM_BROTLI_BASELINE` in `scripts/lib/wasm-artifact.mjs`
  unchanged; run `just demo-build` yourself (build reported +2,803 B, inside the gate).
- `./scripts/decisions-audit.sh --changed ba8b740^` (pass the pre-merge base ref); DEC-102 uses the block-list
  `affected_scope`, supersedes **only** DEC-061's accepted risk #1, and DEC-061 carries the two
  markers.

### 8 — Docs have no green
Every claim the PR adds to `recipes/README.md`, `docs/data-model.md`, `docs/api-contract.md` and
`CHANGELOG.md` is a claim about the shipped binary. Drive each one you can (the key tables, types,
ranges, the "unused by mode" rule, the exit-1 one-line contract). List any that is false or
untestable. The CHANGELOG has exactly one Breaking and one Fixed line for this spec.

## Also
- **CI:** read `gh pr checks 200` (head `48185da`) and the `main` CI run for `ba8b740`; confirm every leg, including
  windows-latest, lean, MSRV, the feature legs and "build + browser smoke". Do not poll.
- **Matrix, once, yourself** (it is cheap insurance against a stale incremental build): default,
  `--no-default-features`, `--features webp-lossy`, each in a fresh `CARGO_TARGET_DIR`,
  **sequential**; tests + `clippy --all-targets -D warnings`; `fmt --check`; `just wasm-check`.

## Guardrails
- **Read-only. No commits. Do not fix what you find.** Use a scratch directory for every driven
  project and for the pre-merge worktree; remove that scratch worktree when done
  (`git worktree remove`).
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal: redirect stdout to
  a file. A piped command reports the **pipe's** exit code: redirect and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`.
- Never run a background build while editing source for a mutation.
- **Budget ~120 exchanges.** If you are past it, stop and report what you covered.

Do **not** edit the spec or its timeline: the orchestrator records your verdict and cost on
`main`.

## Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/` and `.` replaced by `-`. Identify it **by content, not
recency**: `grep -l "SPEC-131 — VERIFY prompt" ~/.claude/projects/<slug>/*.jsonl`. ⚠ **Dedup by
`.message.id`**: Claude Code writes one JSONL line per *content block*; lines sharing an id repeat
identical `input_tokens` / `cache_creation_input_tokens` / `cache_read_input_tokens` while
`output_tokens` grows. Take those three once per id and the **MAX** `output_tokens`. Summing every
line over-counts ~2×; taking the first line's output under-counts ~11×; both mistakes are on
record. Price each component at the model that actually ran (Opus $5/$25 per MTok, Sonnet $3/$15;
cache_creation ×1.25 the input rate, cache_read ×0.10), never flat. Report `tokens_total`, a
`tokens_breakdown`, `estimated_usd` and the rate anchors.

## Report back (your final message — it is pasted to the orchestrator)
First line, exactly one of:
- `✅ APPROVED — SPEC-131 as merged (ba8b740) holds at main <SHA>; no follow-up needed.`
- `⚠ PUNCH LIST — SPEC-131 needs a follow-up fix PR before 0.8.0 is tagged.`
- `❌ REJECTED — <reason>.`
Then lead with **item 1** (goldens re-blessed on `ba8b740^`: identical or not; plain + watermark
cache hits on upgrade: yes or no), then **item 2** (every CLI-verb behaviour change, pre-merge vs `main`). Then items
3–8, short. A punch list is numbered, each item with the evidence and a suggested fix. End with
your cost readout.
