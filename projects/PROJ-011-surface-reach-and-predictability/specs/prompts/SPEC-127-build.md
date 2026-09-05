# SPEC-127 — BUILD prompt

Cycle: **build**. New session, own worktree, branch `feat/spec-127-recipe-format-quality`.
**Sonnet.** You did not design this — the spec carries the context (AGENTS §15).

**What you are building:** `Recipe` gains `format` and `quality`, gated behind a new
`version = "2"`, honoured by one precedence rule across `apply`, `build` and `wasm::transform`.

```
git worktree add -b feat/spec-127-recipe-format-quality \
  ~/PSeven/experiments/crustimg_redo_plus/crustyimg-spec127 main
```

## Read in order, before writing any code

1. **The spec** — `specs/SPEC-127-recipes-carry-format-and-quality.md`. Its
   `## Implementation Context` names the seams; its `## The design calls` are **settled**, not
   open. If you think a call is wrong, say so in Build Completion — do not quietly re-decide it.
2. **DEC-015** (the precedence chain you are extending), **DEC-098** (SPEC-126's ruling that
   `apply` at one input preserves the source), **DEC-087** (a name template's literal extension
   names the file, it does not pin the format), **DEC-005** (recipes round-trip through the
   registry), **DEC-058** (the build cache key).
3. `STAGE-050-recipe-reach.md` and the project `brief.md`.
4. `/guidance/constraints.yaml`.

If anything is genuinely ambiguous, add to `/guidance/questions.yaml` and stop.

## Reserved

📌 **DEC-099 is yours.** `next_id` scans only the working tree, so a record on an unmerged branch
is invisible and the id collides. Highest on `main` is DEC-098. Use the **block-list** form for
`affected_scope`:

```yaml
affected_scope:
  - src/recipe/mod.rs
```

⚠ **Not** the inline-array form. `scripts/decisions-audit.sh` silently drops inline arrays — it is
why DEC-015 is invisible to `--changed` (filed, PROJ-013 STAGE-047). A DEC written inline governs
nothing, and nothing tells you.

## The order that keeps you honest

1. **Baseline first.** Write the seven failing tests from `## Failing Tests`, run them against
   pristine `main`, and **record that all seven are RED and why**. A test that was never seen red
   is not a regression guard.
2. Then implement, smallest seam first: the schema + version gate, then precedence, then wasm.
3. **Push a WIP commit as soon as it compiles**, before the matrix. A previous build in this repo
   ran three hours with zero commits.

## Three things most likely to be got wrong

### 1 — ⚡ `to_toml` must still emit `version = "1"`

For a recipe using neither new field. Emitting `"2"` unconditionally **strands every existing
recipe** on the next `--save-recipe`, and it will look like it works because the new binary reads
both. `v1_still_round_trips_and_stays_v1` is that guard. This is the highest-consequence line in
the spec.

### 2 — Resolve the recipe's values **at the call site**, not inside `encode_one`

`encode_one` already takes `format_override` and `quality`, and **both `apply` and `build` reach
the encoder through it** — SPEC-126 confirmed `build.rs` calls it directly. Read the recipe's
fields in each caller and pass them down. Reading them *inside* `encode_one` would put the
precedence decision below the point where the CLI flags live, and is how the two paths silently
diverge again — which is the exact defect SPEC-126 existed to fix.

Widen `ops::output_format_for` (already `pub(super)`) to take the new rung. **Do not reimplement
the chain beside it.**

### 3 — AC-8 is the one that catches a mistake here

A **v1** recipe through `apply` and `build`, plus `resize`/`thumbnail`/`watermark`/`optimize`/
`web`/`convert`/`responsive`, must be **byte-identical to `main`**. Run it on a real corpus of at
least 4 files across 2 formats, on two binaries, and carry a **positive control** — something you
know differs — so the comparison method is shown able to detect a difference. State the corpus
boundary in DEC-099; SPEC-126's verify had to add that after the fact.

## Also required

- **`docs/api-contract.md` in the same change.** The recipe section and the `apply`/`build`
  entries all describe format resolution. SPEC-126 shipped without it and verify caught it; the
  decisions audit **cannot** warn you, because DEC-015's scope is written inline.
- **AC-7's negative controls: one revert per independent condition.** Call 1, Call 2 and Call 3 are
  independent. Revert each **alone** and confirm only that condition's tests flip. The evidence is
  the **behavioural flip**, never a hash — in the debug profile a rebuild from byte-identical
  source already produces a different binary.
- **AC-9's matrix**: default, `--no-default-features`, `--features webp-lossy`, each in a fresh
  `CARGO_TARGET_DIR`, sequential. Plus **`just wasm-check`** — `src/recipe/` is an engine module
  and compiles for `wasm32`. No `std::fs`, no `clap`.
- ⚠ **`just wasm-test` runs in NO CI job.** A wasm assertion you add is not covered by the
  required matrix. Say that plainly in Build Completion rather than implying the leg is guarded.

## Guardrails

- ⛔ **Byte-changing on the surface: do NOT bump the crate version, do NOT cut a release.** It
  batches into PROJ-011's single lockfile migration with the rest of STAGE-050.
- **Never poll CI.** Background `gh pr checks --watch`; when it exits, read a direct snapshot at
  the **true head SHA** — the watch summary line has been unreliable here.
- `cargo test` fails `display_sink_refuses_non_tty` in an interactive terminal. **Redirect stdout;
  do not "fix" it.** And a piped command reports the **pipe's** exit code — redirect and read
  `$?`. The spec's own Context table got that wrong on its first pass.
- zsh does **not** word-split unquoted parameters — use `while IFS= read -r`, and build argument
  lists explicitly rather than in a variable. Use `/usr/bin/grep`.
- macOS has no `timeout(1)`.
- **Budget ~150 exchanges.** Cycles here have blown their budget five times running without the
  checkpoint firing.

## When you finish

1. Fill in `## Build Completion` — **including every file from `git diff --name-only`, not from
   recall**, and an honest reflection. If a design call turned out wrong, say so there.
2. Append a build cost session entry to `cost.sessions` — **measured**, priced **per component**
   at the anchors `.message.model` reports (DEC-083; a flat rate overstates by ~6× here). Take the
   reading **after CI settles**, not at the "almost done" point.
3. `just advance-cycle SPEC-127 verify`.
4. Create `DEC-099` with a filled `affected_scope` (block-list form).
5. Open the PR — conventional-commit title carrying the spec id, and the body template from
   AGENTS §13.
