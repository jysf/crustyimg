# SPEC-111 — VERIFY prompt

Cycle: **verify**. Fresh session, **Opus**, your own git worktree. You are not the builder and
not the architect.

Output one of: **✅ APPROVED** / **⚠ PUNCH LIST** / **❌ REJECTED**.

Under review: **PR #138**, branch `feat/spec-111-build-recipes`. **All 27 CI checks green,
`mergeState: CLEAN`** — the orchestrator confirmed that directly, so do not re-derive it. This
is the **last launch-gating repo item** apart from the `docs/data-model.md` chore.

**Word contested conclusions "confirm or refute", not "confirm."** Two prompts ago that framing
error put a thumb on the scale; a build deviation that contradicted the design turned out to be
wrong, and the design right.

## Already resolved by the orchestrator — do not spend time here

- **Cost reconciles exactly**: components sum to 144,391,578, $56.84 at Sonnet anchors, 98.12%
  cache reads, `agent` matches the pinned `implementer`. Flag only if `cost.sessions` disagrees
  with the readout.
- **The non-uniform test delta is explained and legitimate.** lean +13 / default +14 /
  webp-lossy +13 looks wrong at first — `--features webp-lossy` is a superset of default, so a
  test in default should also be there. The cause is `tests/build.rs:573`:
  `#[cfg(all(feature = "avif", not(feature = "webp-lossy")))]` on
  `build_writes_the_decided_format_not_the_source_format`. lean lacks `avif`; webp-lossy fails
  the `not(...)`. The arithmetic closes. **But see item 2 — it raises a real coverage question.**

## The four things most likely to be wrong

**1. Did the trap actually get avoided, end to end?** The spec's central warning was that
stripping the terminal step without threading the decided format is a *worse* bug than the
original — it writes the source format and silently discards the modernization. The build says
the format is threaded through a new `format_override` param and
`encode_one_optimize_decided` to the write, the lockfile **and** the cache.

Drive it, do not read it: run `build` on a photographic source through `web` and assert the
output **bytes** are AVIF (`ftypavif`), not just the extension — AVIF-in-a-`.png` passes every
extension check. Then confirm the byte-for-byte claim against `apply --recipe web` on the same
input. Then confirm **AC-7**: the lockfile entry names the file that was actually written, and a
cache **hit** reproduces the same path as the miss that filled it.

**2. AC-2's headline test runs on exactly one feature leg.** Because of the `cfg` above, the
"decided format, not source format" behaviour is pinned **only** on default — it is unasserted
on `webp-lossy`, which is precisely the configuration where the decision has a *different*
correct answer (lossless WebP for graphics). Judge: is a webp-lossy counterpart needed, or is
the single-leg pin sufficient? The gate itself is defensible; the question is whether the
criterion is tested where it applies. [[test-the-guard-where-the-criterion-applies]]

**3. DEC-087 names two out-of-scope items — check it does not also make a claim they falsify.**
The build reports `src/wasm.rs::transform` shares the same unstripped-terminal-`optimize` defect
class, and that `build`'s new auto-decide path does not thread SPEC-107's truncated-JPEG warning.
Naming them in the DEC is exactly what the build prompt asked for, so this is compliant behaviour
— **but SPEC-110 shipped a DEC whose universality claims its own named exception falsified, and
it took two rounds to fix.** Read DEC-087 as text: does its Decision or Consequences assert
anything ("every recipe path", "all bundled recipes work") that the wasm exception makes false?
If so, the wording needs narrowing, not the exception hiding.

Also worth a judgement: the wasm one is the same *defect class* on a shipped surface. Confirm it
is genuinely out of scope rather than convenient — `wasm::transform` is reachable from the live
demo.

**4. The cache-collision bug the build found and fixed.** It reports that decision 1 exposed a
real pre-existing defect — two targets sharing a recipe file with different name templates could
serve each other's cached bytes. That is a **behaviour fix beyond the spec's acceptance
criteria**, in cache code, on the launch path. Confirm: that the bug was real (drive the
collision on `main`), that the fix is correct, that it is tested, and that it did not weaken
SPEC-065's injective-output guarantees. An unasked-for fix in cache code deserves more scrutiny
than an asked-for one, not less.

## Also check

- **AC-1 by both routes** — bundled recipe by name *and* by path, for all three of `web`,
  `gallery`, `product`. Both routes failed identically before; confirm both work now.
- **AC-3** the literal-extension template pins the format (real PNG bytes, decision skipped) —
  the `build` twin of `apply --recipe web -o hero.png`.
- **AC-4 / AC-5** the strip keys on the reserved name: an unknown terminal op still errors, and
  an `optimize` step *not last* still errors. Re-run AC-4's negative control yourself rather than
  reading it; the build reports driving it against the compiled binary with a changed SHA, which
  is the right standard — a changed hash proves a rebuild, driving proves the change took effect.
- **AC-6** a plain pixel recipe through `build` is byte-identical to before, and **`apply` is
  unchanged** — `encode_one` is shared and its signature changed, so both callers are in play.
- **AC-8 / AC-9** the recipe divergence closes: `edit --invert --save-recipe` on an
  Orientation=6 JPEG replays via `apply` to the *same* dimensions, and the saved recipe **names
  `auto-orient` explicitly**. Assert on the recipe TOML, not only the replay — a replay that
  agrees for the wrong reason still passes AC-8.
- **AC-10** `--check` / `--frozen` / `--locked` / `--watch`, now that the output extension varies
  with content. Name which you drove.
- **Enumerate, do not trust the roster.** The build says it swept every path that builds a
  pipeline from a `Recipe`. Re-run that sweep yourself, cite the grep, and state its scope as a
  claim — including what the scope would miss. SPEC-110's roster omitted a verb and cost a full
  extra cycle. [[mechanical-sweeps-need-a-mechanical-check]]
- Run your matrix through **`rtk proxy` from the first leg**; treat a missing
  `Compiling crustyimg` line as a tooling failure first. Reference on `main`: 805 / 824 / 831.
- `just decisions-audit --changed`, `just validate`, `just cost-audit`.

## Guardrails

Own worktree — several trees are still checked out. `git commit -s`. Never `git reset --hard`.
Cross-check anything load-bearing with `/usr/bin/git` or `python3` plus a positive control.
macOS has no `timeout(1)`. **Do not merge the PR.**

## When you finish

Append your verify cost session to `cost.sessions` (per component, at the anchors of the model
that **actually ran**, read from `.message.model` in your own transcript). Update the timeline's
`verify` line. Close with the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not check as clearly as what you did.**
