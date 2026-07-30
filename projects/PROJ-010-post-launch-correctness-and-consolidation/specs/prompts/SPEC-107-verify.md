# SPEC-107 — VERIFY prompt

Cycle: **verify**. Fresh session, **Opus**, your own git worktree. You are not the builder and
not the architect. Your job is to find what both of them missed.

Output one of: **✅ APPROVED** / **⚠ PUNCH LIST** / **❌ REJECTED**.

**The build under review is PR #127**, branch `feat/spec-107-hostile-input-pass`. It reports all
11 acceptance criteria met. The report is unusually candid and self-critical — which is a reason
to read it closely, not a reason to trust it. A build that volunteers four deviations has
probably surfaced its own weak points honestly; it has *not* thereby proven them harmless.

## Re-derive, do not inherit

The design cycle's findings and the build's evidence are both **claims**. Drive the corpus
yourself, on your own builds of the **branch** and of **`main`**. A number you did not produce is
not a measurement. [[a-number-from-an-unproven-path-is-not-a-measurement]]

Read in this order:
1. The spec's `## Build Completion` — the per-AC evidence, the four deviations, and the two
   follow-ups. It is your map of where to dig.
2. `decisions/DEC-085` — the F1 decision, including the `--quiet` call the builder made that the
   spec did not pin down.
3. The diff: `src/image/mod.rs`, `src/cli/{report,ops,optimize}.rs`, `tests/hostile_inputs.rs`,
   `tests/common/mod.rs`, `tests/wasm_roundtrip.rs`, `tests/fixtures/hostile/`.

## The five things most likely to be wrong

These are ranked. Spend your effort here before running a generic checklist.

**1. The build found an existing test that never tested what it claimed — and left it that way.**
This is the most important item on the list. Per the deviations, `png_header_declaring` in
`tests/wasm_roundtrip.rs` (bare signature + IHDR + CRC, no IDAT/IEND) is insufficient to reach
`check_pixel_budget`: the dimension peek fails earlier with a generic
`Decode("unexpected end of file")`. Because
`optimize_detailed_rejects_oversize_without_panic` only asserts `Err(_)` **generically**, it has
apparently **never exercised the `LimitsExceeded` path it exists to pin** — it passes on the
wrong error. The design cycle's coverage matrix marked that case ✅; if this is right, **the
matrix was wrong and so was I.**

The builder fixed its **own copy** in `tests/common/mod.rs` and deliberately did not touch the
wasm original (`one-spec-per-pr`). So the vacuous wasm test is **still vacuous on this branch**.

Verify: (a) confirm the claim empirically — feed the bare shape to the wasm path and read which
error you actually get; (b) decide whether leaving it is a punch-list item or a correctly-scoped
follow-up. Do not accept "out of scope" without checking whether AC-7's own new tests inherited
the same weakness — a new test that asserts `Err(_)` generically is the same bug wearing a new
name. [[a-harness-that-exercises-nothing-reports-green]]

**2. Is AC-5's coverage honestly bounded, or quietly partial?** The warning is wired on
`info`/`web`/`convert`/`resize` (+`optimize`, `thumbnail`, `auto-orient` incidentally) and
**not** on `view`, `watermark`, `edit`, `diff`, `apply`/`build`, `responsive` — all of which also
call `Image::load`/`from_bytes` and will still silently succeed on a truncated JPEG. The builder
names this and calls it in-scope-complete, which matches AC-5's literal wording.

Your call is whether the **spec's** bound was the right one, and whether the gap is recorded
where someone will act on it rather than only in a Build Completion section that gets archived.
Sweep the load sites yourself and **cite the grep with its scope stated as a claim** — do not
take the list of six verbs on trust. [[mechanical-sweeps-need-a-mechanical-check]]

**3. AC-6's carve-out: is it actually narrow?** The claim is that it matches the upstream banner
by name (`"bad parser state bytes left"`), not a blanket debug-mode skip, and that a *new* panic
would still fail the suite. That second half is the load-bearing part and it is a claim about a
counterfactual. **Drive it:** inject a different panic on a hostile input under
`debug_assertions` and confirm the suite goes RED. A carve-out nobody has seen reject something
is not proven narrow. [[a-guards-advertised-reach-is-a-claim]]

**4. The negative controls — re-run them, do not read them.** The build reports both run for real
and both undone before the final matrix. Re-run at least the F1 one yourself: revert
`jpeg_missing_eoi` to `false` and confirm the tests go RED, then confirm your revert is fully
undone. Note the trap this repo has already hit: **reverting source does not rebuild the binary**
— the mutation survives its own revert until something triggers a rebuild.
[[reverting-source-does-not-rebuild-the-binary]] · [[a-control-you-never-verified-applied-is-not-a-control]]

**5. The `--quiet` decision.** DEC-085 makes the F1 warning unconditional, against an established
codebase convention that `--quiet` gates advisories. The reasoning (gating it would reopen the
silent-corruption gap) is sound, but check the *convention* claim is stated accurately and that
nothing else in the CLI now behaves inconsistently. This is a user-visible behaviour choice made
during build on a question the spec left open — exactly the kind of thing verify exists to catch,
whether or not it turns out to be right.

## Also check

- **The +13 delta reconciles.** Claimed: lean 797 / default 816 / webp-lossy 823, +13 identical on
  all three legs, against `main`'s 784 / 803 / 810. Re-measure both sides yourself and reconcile
  the delta against the tests actually added — 9 in `tests/hostile_inputs.rs` (incl. 1 ignored
  generator), 3 unit tests in `src/image/mod.rs`, 1 ignored generator in `src/image/avif.rs`.
  [[verify-test-existence-not-just-gate-count]]
- **Run the matrix through `rtk proxy` from the first leg.** The build hit a **new** rtk failure
  mode: `cargo test` output collapsed to a one-line summary with **no `Compiling crustyimg` line
  at all** — deleting the very control that proves a build was not incremental. Treat a missing
  `Compiling` line as a tooling failure first, a build-state question second.
  [[rtk-can-silently-corrupt-grep-counts]] · [[a-stale-incremental-build-is-a-false-green]]
- **AC-4's profile logic.** The empty-OBU fixture must exit 1 through the CLI, and the
  `debug_assertions` leg is the one that proves the guard — a `debug_abort()` is not an unwind, so
  a thread boundary will not catch it. Confirm the *debug* leg specifically ran.
  [[a-thread-boundary-does-not-catch-abort]]
- **The corpus fixtures assert their own properties.** The builder found the design's ⅓ truncation
  ratio does not transfer across images (boundary between 48–50% on the actual fixture; used 60%).
  Confirm `truncated.jpg` really is in the silent-success regime and not accidentally in the
  hard-error regime — if it drifted, AC-5's whole point evaporates while staying green.
  [[fixtures-from-the-code-under-test-cannot-fail]]
- **AC-9's doc edits are claims, not code.** Read `docs/launch-readiness.md` and
  `docs/api-contract.md` as text: is the closed item's stated outcome actually what was driven, is
  the browser remainder genuinely still on the board, and did the exit-4 row lose its universal
  quantifier? [[documentation-has-no-green]]
- `just decisions-audit --changed` for drift; `just validate`; `just cost-audit`.
- **The cost readout was checked by the orchestrator and reconciles exactly** — component sum
  218,031,873 and $81.04 at Sonnet anchors, 98.6% cache reads, `agent: claude-sonnet-5` matching
  the pinned `implementer`. You do not need to re-derive it; flag only if the spec's
  `cost.sessions` entry disagrees with the readout.

## Guardrails

Own worktree — the build's worktree is still checked out on the build branch, and two sessions in
one tree have corrupted this project's work twice. `git commit -s`. Never `git reset --hard`.
Cross-check anything load-bearing with `/usr/bin/git` or `python3` plus a positive control.
macOS has no `timeout(1)`.

**Do not merge the PR.** Report your verdict; the merge is the maintainer's call.

## When you finish

Append your verify cost session to the spec's `cost.sessions` (see
`projects/_templates/prompts/cost-snippet.md` — price per component at the anchors of the model
that **actually ran**, read from `.message.model` in your own transcript). Update the timeline's
`verify` line. Close with the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not check as clearly as what you did.** If you could not drive something,
say so — a stated gap is worth more than a green tick that quietly skipped it.
