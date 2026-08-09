# SPEC-111 — PUNCH LIST prompt (verify → build)

Cycle: **build** (second pass). Fresh session, own worktree. Verify returned **⚠ PUNCH LIST** on
PR #138.

**Work on the existing branch `feat/spec-111-build-recipes` and push to the same PR #138.** Not a
new spec, not a new PR.

Verify's verdict: *"Every acceptance criterion holds under driving. Two documentation claims are
over-broad. **No code change required.**"* All 11 ACs confirmed against four release binaries,
on bytes rather than extensions, with `build` == `apply --recipe X` byte-for-byte on all three
recipes and both routes. **This pass is record accuracy only — do not touch behaviour.**

There is also an unmerged verify branch `verify/spec-111-build-recipes` (`00bb17b`) carrying the
verify cost session and timeline line. Bookkeeping only; do not duplicate it.

## Item 1 — DEC-087's "complete" claim is driven false

DEC-087's Consequences say:

> *"A recipe saved by `edit` is now a complete, replayable description of what `edit` did."*

Verify drove it: `edit --invert -q 40` → `e3410cc3`; replaying that recipe **without** `-q` →
`0fda01d6`; replaying **with** `-q 40` → `e3410cc3`. **Quality is not a recipe field**, so the
saved recipe is not a complete description of the invocation.

**Fix:** narrow "complete" to the **pixel steps**. AC-8/AC-9 are genuinely met — the orientation
divergence is closed and `auto-orient` is recorded explicitly — so the code is right and only the
scope of the claim is wrong. Do not widen recipes to capture quality; that is a schema question,
not this spec's.

## Item 2 — `src/build/cache.rs`'s module doc justifies the invariant with a false premise

It still asserts the output format:

> *"is a pure function of the input bytes and extension — both already keyed — so a hit implies
> the same format."*

**SPEC-111 falsifies the premise.** The output format is now also a function of the recipe's
terminal step and the target's name template. The *conclusion* still holds — but only because
`target_recipe_hash` folds the plan into the key, which is work this PR did. DEC-087 explicitly
declines to amend this, which leaves the cache's correctness core carrying a **false
justification for the very invariant this PR worked to preserve**.

**Fix:** update the module doc to state the real basis — a hit implies the same format because
the key now covers the plan (recipe + template), not because format is a pure function of input
bytes and extension. Reference `target_recipe_hash`.

## Item 3 — correct an architect error that reached `main`

`projects/…/specs/prompts/SPEC-111-verify.md:63` (merged in #139) describes the cache-collision
bug as *"a real **pre-existing** defect."* **Verify refuted that**: it is not reachable on `main`
at all — a terminal-`optimize` target dies at prepare, and the closest main-reachable shape
(plain recipe, two templates) correctly serves identical bytes. It is a regression **SPEC-111
would have introduced**, caught inside the same change.

DEC-087's own wording was accurate; the orchestrator's relay of it was not. Correct that line so
the archived prompt does not teach the wrong thing, and say in Build Completion that it was the
orchestrator's error, not the build's — the build described it correctly.

## Decide and record (verify raised these; none blocking)

Say what you chose for each. Fixing is optional; **leaving one unrecorded is not.**

- **The build's own AC-7 test is weak.** It uses `--check` without deleting the output first and
  never asserts a hit *occurred*, so it would pass on a rebuild. Verify drove the real hit by
  hand (deleted `dist/`, re-ran, got `1 cached, 0 rebuilt`, same path, same 6755 bytes). The AC is
  met; the committed test under-proves it. Strengthening it is cheap.
  [[a-harness-that-exercises-nothing-reports-green]]
- **Orphaned artifacts.** A content change that flips the decided extension leaves the old file
  in `out/` (`photo.avif` *and* `photo.webp`). `build` has never cleaned, and `--check` catches it
  loudly — pre-existing class, **newly triggerable** by this change, and unnamed in DEC-087's
  Consequences. At minimum, name it there.
- **`name = "{stem}"`** (no extension) exits 4, which `docs/api-contract.md`'s phrasing about
  "naming a literal extension that is not a recognized image format" does not quite cover.

## Do not

- Change behaviour. Verify confirmed all 11 ACs by driving; this pass is documentation.
- Widen recipes to capture quality (Item 1) — schema question, separate spec.
- Fix `src/wasm.rs::transform`. Verify drove it (`unknown operation 'optimize'`) and confirmed it
  is genuinely out of scope: `demo/worker.js:135` builds its own terminal-step-free recipe, so
  the shipped demo never reaches it, and `transform` takes an explicit `out_format`, making the
  fix a design question rather than a strip. It stays named in DEC-087.

## Verify before handing back

Docs-only changes should not move the matrix, but confirm rather than assume — and **through
`rtk proxy` from the first leg**. Reference on the branch: **default 838 / lean 818 /
webp-lossy 844**, 0 failed; `just wasm-test` 30/30; clippy and fmt clean. If you strengthen the
AC-7 test, reconcile the delta.

**Then read the CI legs on the PR** before claiming green.

## Guardrails

Own worktree — several trees are still checked out. `git commit -s`. Never `git reset --hard`.
Cross-check anything load-bearing with `/usr/bin/git` or `python3` plus a positive control.
macOS has no `timeout(1)`. **Do not merge the PR.**

## When you finish

Append a **second build session** to `cost.sessions` (do not overwrite the first). Update
`## Build Completion` with what this pass changed, and the timeline's build line. Close with the
`## Cost readout` block, verbatim, as the last thing you emit.
