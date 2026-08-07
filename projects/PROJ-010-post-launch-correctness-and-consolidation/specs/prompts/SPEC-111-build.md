# SPEC-111 — BUILD prompt

Cycle: **build**. You are NOT the architect. Both design calls are made; your job is to
implement them. This is the **last launch-gating repo item**.

**One-line summary:** `build` cannot run any recipe the binary ships with. Strip the reserved
terminal `optimize` step — and, crucially, thread the format the decision picks through to the
output, because stripping alone writes the source format and silently discards the whole point.

## Read in order — deliberately short

1. **The spec** —
   `/projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-111-build-runs-bundled-recipes.md`,
   in full. **11 acceptance criteria, 10 pre-written failing tests, one negative control.**
2. **The code** — `src/cli/common.rs:33-56` (`encode_one`, and the `img.source_format()` at
   `:52` this spec retires), `src/cli/build.rs:85` / `:106-115` / `:215-234` / `:317` / `:575`,
   `src/cli/optimize.rs:25-45` (the helper to reuse) and `:80-102` (the precedent to copy).
3. **`recipes/web.toml`** — note it already names `op = "auto-orient"` explicitly. That is the
   precedent for design question 2.
4. **`/AGENTS.md`** — §4 cost, §6 commands, §12 testing, §13 git/PR. Skip the rest.

## The driven facts, inlined

Release build at `3dd8fa7`, real manifest (`version = 1`), one PNG source:

| invocation | exit | result |
|---|---|---|
| `build`, `recipe = "web.toml"` | **1** | `error: unknown operation 'optimize'` |
| `build`, `recipe = "web"` (by name) | **1** | identical |
| `apply --recipe web`, same source | **0** | writes a real AVIF (`ftypavif`) |
| `build`, plain pixel recipe | **0** | `built 1 target, 1 output` |

The two controls prove the fault is exactly the terminal marker.

## The trap: stripping alone is a WORSE bug

`encode_one` (`src/cli/common.rs`) does:

```rust
let pipeline = recipe.build_pipeline(registry)?;   // :46 — dies on the terminal step
let fmt = img.source_format();                      // :52 — "no --format override in batch path v1"
```

It **preserves the source format**. The terminal `optimize` step exists so the fast decision
*chooses* one (AVIF for photos, lossless WebP for graphics, SPEC-085). Strip the step and stop
there, and `build` runs the pixel pipeline then writes the source format — silently discarding
the modernization. **That fails quietly, where today it fails loudly.** Do not ship the half fix.

Encouraging: **`build` already anticipates this.** `EXT_SENTINEL` (`build.rs:115`) exists
because *"the real output extension is only knowable after a decode"*, and `lock_output_path`
(`:221`) already takes the real `ext` as a parameter. The naming and lockfile layers are shaped
for it; only `encode_one`'s format choice is not.

## Decision 1 — what picks the format (made; do not re-litigate)

Copy `apply`'s rule. `run_apply` (`optimize.rs:80-102`) skips the decision when the format is
**pinned**, so `apply --recipe web hero.jpg -o hero.png` matches `web hero.jpg -o hero.png` —
"a real PNG of the downscaled image, not AVIF-in-a-`.png`".

**`build` uses the name template as the pin:**
- template names a **literal extension** (`name = "{stem}.png"`) → pinned; honour it, skip the
  decision.
- template uses **`{ext}`** (including the default `{stem}.{ext}`) → the decision chooses, and
  `{ext}` expands to the chosen format.

One rule across `apply` and `build`. Two rules is what SPEC-110 paid three cycles to avoid.

## Decision 2 — the recipe divergence (made; do not re-litigate)

SPEC-110 made `edit` bake orientation on the CLI path but not record it in `--save-recipe`
output, so a saved recipe no longer reproduces what `edit` did (driven at verify: `edit` gives
800×1200, the replay gives 1200×800). **SPEC-110 introduced this** — DEC-086 says so; do not
re-describe it as pre-existing.

**Fix: `edit --save-recipe` records `auto-orient` explicitly.** Not an implicit prefix on
`apply`/`build` — that would make a recipe no longer a complete description of its own
behaviour, which is exactly the pattern SPEC-110 removed. `recipes/web.toml` already names
`auto-orient` as an explicit step; saved recipes should match.

## Notes

- **Reuse `split_terminal_optimize` (`optimize.rs:39`); do not copy it.** It already documents
  the "an `optimize` step anywhere but last stays an error" rule that AC-5 pins. `pub(super)` is
  the intended move.
- **The format thread is the work.** Whatever `encode_one` returns must reach
  `lock_output_path`'s `ext`, or AC-7 fails silently and the lockfile names a file that does not
  exist.
- **`encode_one` is shared with `apply`.** Changing its signature touches both callers — AC-6
  guards that the plain path is byte-identical.
- **Assert on bytes, not extensions.** AVIF-in-a-`.png` passes every extension check.
- **Check `--check` / `--frozen` / `--locked` / `--watch`** (AC-10) — the output extension now
  varies with content, and those paths compare recorded paths.

## Three process warnings, all paid for by SPEC-110

1. **Enumerate; do not trust the roster.** SPEC-110's design table omitted one verb; that hole
   propagated through the build and cost a full extra cycle. Before claiming this is complete,
   enumerate **every** code path that builds a pipeline from a `Recipe` and classify each. Cite
   the grep and state its scope as a claim.
2. **If your sweep and this spec disagree, the sweep wins.** Fix what it finds, or name the
   exception in the DEC. Do not file it and ship a universal claim — that is precisely what went
   wrong last time.
3. **Read the CI legs.** SPEC-107 shipped a red Windows leg behind a "full matrix clean" claim
   from a local macOS run.

## Verify before handing back

Full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequentially, **through `rtk proxy` from the
first leg** (it has collapsed `cargo test` output and deleted the `Compiling crustyimg` line —
treat a missing one as a tooling failure first). Reference on `main`: **lean 805 / default 824 /
webp-lossy 831 passed, 0 failed**, `just wasm-test` 30/30. Reconcile your delta against the
tests you add.

Run AC-4's negative control and record it: make the strip drop the last step unconditionally →
`build_still_rejects_an_unknown_terminal_op` must go RED. Prove the mutation reached the
**binary**, not just the source — a changed MD5 shows a rebuild happened, not that the change
took effect. Drive it.

## Repo guardrails

`git commit -s` on every commit (DCO enforced). Never `git reset --hard`. Cross-check anything
load-bearing with `/usr/bin/git` or `python3` plus a positive control. macOS has no
`timeout(1)`. Own git worktree; check `git branch --show-current` before committing. **Do not
merge the PR.**

## When you finish

Fill in `## Build Completion` and the three reflection questions. Update the timeline's build
line. Create the new DEC covering **both** design calls, with `affected_scope` filled in.

### Cost

Follow `projects/_templates/prompts/cost-snippet.md` on `main`. Price **per component** at the
anchors of the model that **actually ran** (`.message.model` from your own transcript). Close
with the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not do as clearly as what you did.**
