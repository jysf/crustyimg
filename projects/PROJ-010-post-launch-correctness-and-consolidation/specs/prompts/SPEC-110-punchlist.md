# SPEC-110 — PUNCH LIST prompt (verify → build)

Cycle: **build** (second pass). Fresh session, own worktree. Verify returned **⚠ PUNCH LIST** on
PR #133.

**Work on the existing branch `feat/spec-110-orientation` and push to the same PR #133.** Not a
new spec, not a new PR.

Verify's verdict on the change itself: *"correct, safe, and better-tested than the spec asked
for."* AC-1 through AC-6, AC-10 and AC-11 all confirmed by independent re-derivation — release
builds of both sides, driven through **ImageMagick `identify`/`magick` and `exiftool`**, oracles
outside the code under test. It fails on its **Goal**, not its engineering.

There is also an unmerged verify branch `verify/spec-110-orientation` (`72efb2d`) carrying the
verify cost session and timeline line. It touches only bookkeeping files; do not duplicate it.

## Item 1 — BLOCKING. `watermark` is unbaked, so AC-7 and the Goal are false.

Driven on the branch: `watermark --text hi` returns **1200×800** where **800×1200** is correct.
`run_watermark` (`src/cli/ops.rs:1085`) builds `Pipeline::new().push(...)` → `run_pixel_op` — the
**identical shape** to `resize` and `thumbnail`, which were fixed. **The fix is one token.**

Verify's scope claim, which you should treat as the roster: every `Pipeline` construction in
`src/`, every `run_pixel_op` call site, every direct `pipeline.run(…)` — 22 sites, cross-checked
with an independent `python3` walk (22 = 22, so no rtk corruption). That scope has a real hole —
it only finds sites that build a `Pipeline` — so verify closed it by enumerating all **17
subcommands** from `--help` and classifying each. **`watermark` is the only missed pixel-lane
site.** `run_apply` / `recipe::build_pipeline` / `wasm::transform` are the recipe and wasm lanes
(deliberately excluded and documented); `run_auto_orient` is the op itself;
`view`/`info`/`diff`/`meta`/`lint` never re-encode.

**Why filing it was the wrong call — and this is the part that matters more than the verb.**
DEC-086's title, its Decision (*"Every verb that re-encodes pixels … pins the existing auto-orient
operation first"*) and its Consequences (*"no shipped verb can hand back a sideways image"*) are
all **false as shipped**. SPEC-110 exists *because* DEC-003 stopped describing the code. Shipping
DEC-086 already not describing the code recreates that failure mode in a brand-new record on day
one. `src/cli/optimize.rs:782`'s doc comment does it in miniature — it claims "every verb that
re-encodes pixels bakes EXIF orientation first" and then lists watermark as an exception two lines
later.

Note also what happened procedurally: the build **did** find watermark in its sweep, then routed
the finding back through the design's measured table — the very artifact AC-7's mechanical sweep
existed to backstop. **The mechanical check ran and was overruled by the thing it was checking.**
When a sweep and a roster disagree, the sweep wins.

**Fix watermark.** Then add a test for it alongside the existing nine.

## Item 2 — BLOCKING. Make the records true.

After Item 1 they become true on their own; confirm rather than assume:

- **DEC-086** — Decision and Consequences now hold. Also fix its **Consequences claim about
  recipes**, which Item 3 refutes.
- **`src/cli/optimize.rs:782`** — the doc comment's universality claim and its watermark
  exception. Remove the exception.
- **`docs/api-contract.md`** — the `watermark` section is currently **the only pixel verb silent
  on orientation**, which reads as oversight rather than a documented carve-out. State it.

## Item 3 — SHOULD FIX. Item 6 was mis-characterized: this PR *introduces* the recipe divergence.

The build's Follow-up and DEC-086's Consequences both call the `edit --save-recipe` gap
"pre-existing, unchanged by this decision." **Verify drove it and refuted that:**

| | `edit --invert` output | recipe replayed via `apply` |
|---|---|---|
| `main` | 1200×800 | 1200×800 — **consistent** |
| branch | 800×1200 | 1200×800 — **divergent** |

So a recipe round-tripped out of `edit` no longer reproduces what `edit` did. **Correctly out of
scope to fix** — that is SPEC-111's pixel-lane wiring — but it must be described correctly, and it
does leave a shipped surface self-inconsistent. Correct the wording in Build Completion **and** in
DEC-086, and file it as a follow-up that names SPEC-111 as the place it lands.

## Item 4 — SHOULD FIX. `docs/cli-reference.md` contradicts itself.

It keeps *"(byte-pinned to what `apply` of that recipe produces)"* and then contradicts it in the
next sentence. Drop the parenthetical.

## Worth knowing — not blocking, but say what you decide

Verify judged these met-as-written; they are recorded so nobody rediscovers them as defects.

- **AC-5's committed test is half vacuous.** It asserts dimensions only, so for orientations 1–4
  it asserts (40,30) — which an **unbaked** build also produces. Four of eight assertions cannot
  fail; the test would stay green if flips and 180° silently stopped applying. The AC as written
  asked for exactly this, so the AC is met — but it is weaker than its own rationale ("a fix that
  handles only 6 is not a fix"). The behaviour is genuinely correct: verify drove all eight at
  **pixel-content** level against ImageMagick and every one matches. **Strengthening this to a
  content-level assertion is cheap and would close it properly** — do it if it stays small, and
  say so either way. [[a-claim-that-a-test-is-vacuous-needs-driving-too]]
- **AC-3's test proves a weaker proposition than AC-3's wording.** It compares no-EXIF vs
  orientation-1 *within one build*, not branch-vs-main, so it cannot catch a change that shifted
  both equally. Verify's cross-binary drive covered that gap (24/24 byte-identical). Good
  in-branch invariant guard; just not literally "byte-identical to before."
- **`auto_orient_prefix()` has no direct unit test** — `pub(super)`, so
  `every-public-fn-tested` does not strictly bite; covered indirectly by all nine tests.
- One stale *"pure re-encode … pixels unchanged"* remains in
  `docs/research/photo-preset-import-and-photographic-ops.md:536` — a research note quoting old
  code. Arguably fine; leave it or fix it, but say which.

## Verify before handing back

Full matrix from fresh per-leg `CARGO_TARGET_DIR`s, sequentially, **through `rtk proxy` from the
first leg**. Reference after this build: **default 823 / lean 804 / webp-lossy 830 passed, 0
failed**, `just wasm-test` 30/30. Your watermark test (and any AC-5 strengthening) will move
those — reconcile the delta explicitly.

**Then read the CI legs on the PR.** All three OS legs were green last push; keep them green.

On AC-10's method, from verify: **a changed MD5 proves a rebuild happened but not that the revert
took effect** — any unrelated touch changes it. Driving the binary is the sufficient check. Use
that standard if you re-run any control.

## Guardrails

Own worktree — several trees are still checked out. `git commit -s`. Never `git reset --hard`.
Cross-check anything load-bearing with `/usr/bin/git` or `python3` plus a positive control.
macOS has no `timeout(1)`. **Do not merge the PR.**

## When you finish

Append a **second build session** to `cost.sessions` (do not overwrite the first). Update
`## Build Completion` with what this pass changed, and the timeline's build line. Close with the
`## Cost readout` block, verbatim, as the last thing you emit.
