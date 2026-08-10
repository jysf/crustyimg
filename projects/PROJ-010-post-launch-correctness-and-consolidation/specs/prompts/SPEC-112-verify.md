# SPEC-112 — VERIFY prompt

Cycle: **verify**. Fresh session, **Opus**, your own git worktree. You are not the builder and
not the architect.

Output one of: **✅ APPROVED** / **⚠ PUNCH LIST** / **❌ REJECTED**.

Under review: **PR #144**, branch `feat/spec-112-wasm-bundled-recipes`, two commits on top of
`main` at 462b829. **This is the last repo item before the 0.7.0 cut**, which is STAGE-040's
other half and is blocked on it.

**Word contested conclusions "confirm or refute", not "confirm."** The build's readout is
unusually good — it self-reported a deviation and a discrepancy rather than hiding them. Do not
let that buy it the benefit of the doubt on the things it did *not* check.

## Already resolved by the orchestrator — do not spend time re-deriving these

1. **The `build_pipeline` enumeration is settled.** I grepped independently, the build grepped
   independently, and we agree on all five non-wasm sites (`cli/optimize.rs:135`/`:165` in
   `run_apply`'s markerless branch; `cli/common.rs:57` receives a stripped recipe;
   `cli/build.rs:181` is SPEC-111's fixed site; `recipe/bundled.rs:97` is inside a `#[test]`).
   `src/wasm.rs` was the last unstripped production site. **You may still widen the sweep if you
   think its scope misses something** — state what the scope would miss — but do not just redo it.

2. **The native test-count "discrepancy" is a non-issue and is explained.** The build measured
   lean 821 / default 841 / webp-lossy 847 against a stated reference of 818 / 841 / 844. All 7
   new tests are inside `tests/wasm_roundtrip.rs`'s `#[cfg(target_arch = "wasm32")] mod wasm`
   block (opens at line 31), which native `cargo test` never compiles — I confirmed the gating
   and the count directly off the diff. Native delta from this change is **0** by construction,
   so the stale reference does not bear on the change. The build verified this for lean with a
   stash-to-base positive control and skipped the same control for webp-lossy. Given the cfg
   gating that skip is now harmless — **do not spend a webp-lossy recompile on it.**

3. **Cost: the build's committed numbers are WRONG, and I have already corrected them.** Do not
   re-derive; do not re-price the build session. The build read the **parent orchestrator's**
   transcript instead of its own, and so recorded `agent: claude-opus-5`,
   `tokens_total: 8119424`, `estimated_usd: 6.75`. Its own subagent transcript
   (`.../subagents/agent-a0a1ffb97d8cbdd9d.jsonl`) is **320 assistant messages, all
   `claude-sonnet-5`**. The true figures, components summing exactly to the total:

   | component | tokens | at Sonnet anchors |
   |---|---:|---:|
   | input | 640 | $0.0019 |
   | output | 84,632 | $1.2695 |
   | cache_creation (×1.25 in) | 3,966,512 | $14.8744 |
   | cache_read (×0.10 in) | 77,699,018 | $23.3097 |
   | **tokens_total** | **81,750,802** | **$39.46** |

   The orchestrator writes this into `cost.sessions` at **ship** (AGENTS §4). **Your job here is
   only your own verify session's cost** — and read `.message.model` from *your own* transcript,
   not the newest file in the project directory. That is exactly the trap the build fell into.

4. **CI on PR #144 is settled and fully green**: **27 pass, 6 skipped, 0 pending, 0 failing** — I
   read every leg individually off `gh pr checks 144`, including both Windows legs and the
   `build + browser smoke` leg. `mergeStateStatus` is BLOCKED only for want of a review. You do
   not need to re-poll CI. **Do re-check it if you push a commit to the branch** (item 4 likely
   means you will), because that starts a fresh run — and SPEC-107 shipped a red Windows leg
   behind a "full matrix clean" claim from a local macOS run.

## The four things most likely to be wrong

**1. Drive the fix, do not read it.** The whole spec is one line —
`let pixel_recipe = split_terminal_optimize(&recipe).unwrap_or(recipe);`. Confirm or refute, by
driving the **real wasm surface** (not the native call chain), that `web`, `gallery` and `product`
all now succeed through `transform`, and that they all failed before. The design's driven table
says all three returned `Err("unknown operation 'optimize'")` on `main`; reproduce at least one of
those failures yourself so the "before" is a measurement and not an inherited claim.

**2. AC-4 is byte-identity against `main`, and the build substituted something else.** The spec
and the build prompt both said the markerless (demo-shape) recipe must be **byte-identical to
`main`**, "not merely still works". The build's `transform_leaves_a_markerless_recipe_unchanged`
does **not** diff against a checked-out `main`; it reconstructs the expected bytes in-process by
calling `build_pipeline` + `encode_to_bytes` on the unstripped recipe, arguing that
`split_terminal_optimize` is a no-op on a markerless recipe so the reconstruction *is* what `main`
did.

That argument looks sound to me on the diff — the only change to `transform` is the one line, and
`None` feeds the original recipe through unchanged. **But the test is now partly self-referential**
(it exercises the same `build_pipeline`/`encode_to_bytes` the code under test uses), and a
self-referential control cannot detect a broken pipeline
[[a-self-referential-control-cannot-detect-a-broken-pipeline]]. Judge it: is the substitution
adequate, or does AC-4 need a genuine `main`-vs-branch byte diff? **If you think it needs the real
diff, run it** — check out `main` in a second worktree, run `transform` on the identical input and
recipe, and compare bytes. That is cheap and it is the thing that actually guards the live demo.

**3. The build made a structural finding — confirm it, because it changes what the spec said was
possible.** The spec offered two equally-reasonable options: widen `split_terminal_optimize` to
`pub(crate)` in `src/cli/optimize.rs`, or move it to a neutral module. The build moved it to
`src/recipe/mod.rs` and reports the widen option was **never viable**: `src/lib.rs` gates
`pub mod cli;` behind `#[cfg(not(target_arch = "wasm32"))]` and `pub mod wasm;` behind
`#[cfg(target_arch = "wasm32")]`, so a `cli`-hosted item at any visibility is not compiled into
the artifact `wasm::transform` lives in. I read `src/lib.rs:51,62-65,72-73` and it supports this.
Confirm or refute. If it holds, it is worth more than the fix — it means the design named an
impossible option — and it belongs in the record, which the build says it put in DEC-087's
amendment. Check that it is actually there and reads as a finding, not an aside.

**4. A wrong claim in a doc comment, which is this wave's recurring failure mode.**
`src/recipe/mod.rs`'s new doc block on `split_terminal_optimize` says the `build` caller reaches it
"via `cli::optimize`'s re-export". It does not: `src/cli/build.rs:13` imports it directly from
`crate::recipe`, and the only `pub use` in `cli/mod.rs` is `WEB_DEFAULT_LONG_EDGE` — I checked both.
Small, and purely documentation, but the engineering has been sound nearly every round this wave
while **claims about the code** drifted, so it should not ship. Confirm the inaccuracy and fix the
sentence. While you are in that doc block, read the rest of it as text against what the code does.

## Also check

- **AC-5 keys on the reserved name, tested where the criterion applies.** An unknown terminal op
  still errors, and an `optimize` step *not last* still errors. Both AC-5 tests assert only that
  the error message is non-empty (`assert!(!msg.is_empty())`). Judge whether that is enough, or
  whether they should pin *which* error — a test that passes on any error would also pass if the
  strip started eating unknown terminal ops and something else failed downstream.
  [[test-the-guard-where-the-criterion-applies]]
- **AC-3, the trap.** `transform_actually_runs_the_pixel_steps` asserts 1800×1200 → 1600×1067 via
  `product`. Confirm those are the right numbers for that recipe rather than numbers that happen to
  match, and that the assertion would fail if the strip dropped the whole recipe.
- **AC-9's negative control.** The build reports GREEN→RED→GREEN with matching test results
  (37/0 → 34/3 → 37/0) and matching artifact SHA-256s. That is the right standard. Re-run it
  yourself rather than reading it. [[a-claimed-failure-mode-is-as-unproven-as-a-claimed-success]]
- **AC-7** — DEC-087's amendment: dated, states the exception is closed and *why it was reopened*
  (the demo reasoning held, the README's claim did not), original text preserved not deleted. Also
  read DEC-087 as text for a universality claim its own remaining exceptions falsify — SPEC-110
  shipped exactly that bug and it took two rounds.
- **AC-8** — `README.md:34-36` as text against what `transform` now does. The build says it also
  scanned `README.md:261-276` and found no other overstatement of the wasm surface. Spot-check
  that. **The claim must not have been fixed by weakening it** — confirm the README is unedited.
- **`demo/worker.js` is unchanged**, and reason about whether the live demo can regress.
- **`optimizeDetailed` is untouched** — the decide-path counterpart stays the decide path.
- Run your matrix through **`rtk proxy` from the first leg**, fresh per-leg `CARGO_TARGET_DIR`,
  sequentially; treat a missing `Compiling crustyimg` line as a tooling failure first. Plus
  `just wasm-check` then `just wasm-test`. Per item 2 you already know the native delta should be
  0 and `wasm-test` should be exactly +7 (30 → 37).
- `just decisions-audit --changed`, `just validate`, `just cost-audit`.

## Guardrails

Own worktree — create it yourself under `../crustyimg-spec112-verify`; the tree at
`../crustyimg-spec112` belongs to the build and the primary checkout belongs to the orchestrator.
`git commit -s` (DCO enforced). Never `git reset --hard`. Cross-check anything load-bearing with
`/usr/bin/git` or `python3` plus a positive control. macOS has no `timeout(1)`. A piped command
reports the pipe's exit code — redirect and read `$?`. **Do not merge the PR.** **Do not bump the
version** — the 0.7.0 cut is the orchestrator's separate chore.

Per AGENTS §13, verify bookkeeping lands on `main`, not the feature branch — so if you need to fix
something on the branch (item 4), commit it to `feat/spec-112-wasm-bundled-recipes`; anything that
belongs to `main` (timeline, cost) I will land in the ship PR. **Do not ask the build branch to
correct a file that lives on `main`** — that caused an add/add conflict on an otherwise finished PR
last cycle.

## When you finish

Append your verify cost session to `cost.sessions` (per component, at the anchors of the model that
**actually ran**, read from `.message.model` in **your own** transcript — see item 3). Update the
timeline's `verify` line. Close with the `## Cost readout` block, verbatim, as the last thing you
emit.

**Report what you could not check as clearly as what you did.**
