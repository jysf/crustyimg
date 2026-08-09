# SPEC-112 — BUILD prompt

Cycle: **build**. You are NOT the architect. The design is settled; implement it.

**One-line summary:** `wasm::transform` is the last call site that hands a terminal `optimize`
step to `build_pipeline`, so none of the three recipes crustyimg ships with can run through it
— and the README says they can. Strip the marker.

**This blocks the 0.7.0 cut.** The README renders on the crates.io crate page, so releasing
without this publishes the false claim wider.

## Read in order

1. **The spec** —
   `/projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-112-wasm-transform-runs-bundled-recipes.md`,
   in full. **10 acceptance criteria, 7 pre-written failing tests, one negative control.**
2. **The code** — `src/wasm.rs:155-170` (`transform`) and `:86-92` (`parse_format`);
   `src/cli/optimize.rs:25-45` (`OPTIMIZE_STEP_OP` + `split_terminal_optimize`, the helper to
   reuse); `demo/worker.js`'s `geometryRecipe()` (why the demo is unaffected — do not change it).
3. **`decisions/DEC-087`** — it names this as an out-of-scope exception. Amending it is AC-7.
4. **`/AGENTS.md`** — §4 cost, §6 commands, §12 testing, §13 git/PR.

## The driven facts

`transform`'s exact call chain, run natively at design against all three shipped recipes:

| bundled recipe | result |
|---|---|
| `web` | `Err("unknown operation 'optimize'")` |
| `gallery` | `Err("unknown operation 'optimize'")` |
| `product` | `Err("unknown operation 'optimize'")` |

The demo escapes it only because `geometryRecipe()` hand-builds a terminal-step-free recipe.
The published `crustyimg-wasm` npm package does not escape it.

## The design question SPEC-111 had does NOT exist here

`build` needed something to choose the output format once the marker was stripped — that was
the trap, and getting it wrong would have silently written the source format.

**`transform` has no such fork.** It takes `out_format`, and `parse_format` resolves it through
`format_from_extension`, which cannot accept `"auto"` or an empty string — only a concrete
format. **The caller has always pinned it.** So: strip the marker, run the pixel steps, encode
to `out_format`. That is exactly DEC-087's pinned branch and `apply`'s before it.

`optimizeDetailed` stays the decide-path counterpart. **Do not give `transform` an `"auto"`
mode** — that is new capability and needs its own spec.

## Two traps

1. **AC-3.** A strip that removes the *whole recipe*, or runs no steps, still returns bytes in
   the right format and passes AC-1 and AC-2. Only asserting the output **dimensions** changed
   where the recipe resizes will catch it.
2. **AC-4.** The live demo sends markerless recipes. Those must stay **byte-identical to
   `main`** — not "still work". That is the guard that this change cannot break the demo.

## Notes

- **Reuse `split_terminal_optimize`; do not copy it.** It is `pub(super)` in
  `src/cli/optimize.rs` after SPEC-111. Widening to `pub(crate)` or moving it somewhere neutral
  (it is really a *recipe* concern, not a *cli* one) are both fine — **say which you chose and
  why.** A second copy in `wasm.rs` is the one outcome to avoid.
- **`wasm.rs` compiles for wasm32 and for native tests.** `just wasm-check` is the fast gate;
  run it before the full `just wasm-test`.
- **AC-5**: the strip must key on the reserved name — an unknown terminal op still errors, and
  an `optimize` step *not last* still errors, matching `build`'s behaviour.
- **AC-7 is real work.** DEC-087 currently records this as out of scope. Amend it, dated,
  stating that the exception is closed and *why it was reopened*: the demo reasoning held, the
  README's claim did not. Do not silently delete the exception — the record should show the
  call and its correction.
- **Do not fix the README by weakening it** (AC-8). The claim is a good one and the code should
  meet it. If you find you cannot make it true, stop and report rather than editing the sentence.
- **Do not bump the version.** The 0.7.0 cut is STAGE-040's separate chore.

## Verify before handing back

Full matrix, fresh per-leg `CARGO_TARGET_DIR`, sequentially, **through `rtk proxy` from the
first leg** (it has collapsed `cargo test` output and deleted the `Compiling crustyimg` line —
treat a missing one as a tooling failure first). Reference on `main`: **lean 818 / default 841 /
webp-lossy 844** passed, `just wasm-test` 30/30. Reconcile your delta against the tests you add.

**Then read the CI legs on your PR before claiming green.** SPEC-107 shipped a red Windows leg
behind a "full matrix clean" claim from a local macOS run.

Run AC-9's negative control and record it: revert the strip, confirm `transform_runs_every_bundled_recipe`
goes RED, restore. Prove the revert reached the built artifact, not just the source — a changed
hash shows a rebuild happened; driving shows the change took effect.

**Enumerate before claiming completeness** (spec's last Note). SPEC-111's verify established
that `registry.build(&step.op, …)` is the only route from an op name to an `Operation`, so
`transform` should be the last unstripped site — confirm with your own grep and state its scope
as a claim rather than inheriting that.

## Repo guardrails

`git commit -s` (DCO enforced). Never `git reset --hard`. Cross-check anything load-bearing with
`/usr/bin/git` or `python3` plus a positive control. macOS has no `timeout(1)`. Own git
worktree; several are already checked out. **Do not merge the PR.**

## When you finish

Fill in `## Build Completion` and the three reflection questions. Update the timeline's build
line. Amend DEC-087 (AC-7) — a new DEC is **not** expected; if you find a reason to deviate from
the pinned-format rule, that is a finding to report, not a second rule to invent.

### Cost

Follow `projects/_templates/prompts/cost-snippet.md` on `main`. Price **per component** at the
anchors of the model that **actually ran** (`.message.model` from your own transcript). Close
with the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not do as clearly as what you did.**
