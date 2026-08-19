# SPEC-121 — PUNCH LIST prompt

Cycle: **build** (punch-list return). Verify returned **⚠ PUNCH LIST** on PR #181. The central
claim survived — `resize`, `thumbnail`, `edit --invert` and `web` genuinely preserve colour type
and bit depth, reproduced on independent fixtures. **Do not re-litigate the fix.** Seven items.

Branch `fix/spec-121-ops-preserve-colour-type-and-bit-depth`, head `4391e06`. **Own worktree.**
`cycle:` stays at `verify` — do not advance it; the orchestrator re-approves.

## Read first

1. **The spec** — `.../specs/SPEC-121-ops-preserve-colour-type-and-bit-depth.md` (Call 1, Call 2,
   AC-4, AC-6, AC-7).
2. **DEC-095** — you will be correcting it, and **SPEC-122 amends it next**.
3. **`/AGENTS.md`** §12, §13, **§15**.

---

## 1 — ⚡ The sweep, and item (a) is URGENT

Verify's grep (all 798 tracked files) found **four** live premises, not the one the build fixed.
**Amend, do not delete** — SPEC-121 preserves input depth, it does **not promote**, so for an
8-bit source a grade is still quantized to 256 levels. The quantization half is now *conditional*;
the transfer-function half is untouched and still live for SPEC-122.

- **(a) `docs/backlog.md:678-682` — do this first.** It sits inside **SPEC-122's own backlog
  entry** and reads *"The `Operation` pipeline stays 8-bit… convert back to 8-bit on the way out…
  16-bit PNG/TIFF inputs are truncated today — every op calls `to_rgba8()`."* **A SPEC-122 builder
  following it will re-break SPEC-121.**
- **(b) `:823`** — the LUT gate (*"a `.cube` applied in an 8-bit, non-linear pipeline"*).
- **(c) `:933-934`** — the effects scope guard (*"All of it is bounded by the 8-bit sRGB pipeline"*).

**Leave alone** (verified correct as historical/defect-describing): the spec `:115`/`:193`, the
build prompt, `docs/backlog.md:997` and the whole Live-defect section, DEC-095's own description.

**Cite your grep and its scope.** `STAGE-046:246` should stay unmarked until this is done — note
it is prose, not a `- [ ]`, so nothing tracks it mechanically.

## 2 — ⚠ Watermark: the substantive one

**`watermark --text` on an opaque photo still returns RGBA8**, so the +12.4% wasted channel this
spec exists to remove is still paid on the verb's commonest invocation — the one the spec's own
defect table measured.

Why: source-over onto a fully opaque base is **mathematically always opaque**. The observed
histogram is `{255: 988, 254: 36}` — those 254s are f32 rounding plus a truncating cast in
`image`'s `Rgba::blend`, not genuine transparency. 26 pixels in 65,536 defeat the narrow.

- **Fix it so a genuinely opaque composite narrows**, without destroying real alpha. Call 2 stands:
  RGBA only when the overlay *actually contributed non-opaque samples*.
- ⚠ **Both existing AC-4 tests use uniform overlays** that hit `blend`'s fast paths
  (`alpha == MAX` / `alpha == 0`) and never exercise the float path. **`--text` is untested.**
  Add a test that drives the float path.
- **Keep the control:** a genuinely translucent overlay must still keep RGBA. A tolerance that
  swallows real transparency is a worse bug than the one you are fixing.
- Verify flagged a **latent CI break**: an `image` release that rounds instead of truncating flips
  the current test red. Your fix should not depend on that behaviour either.

## 3 — A filing, and a FALSE citation

`tests/colour_type_preservation.rs:215` says the lossless-WebP 16-bit gap is *"filed in
`docs/backlog.md`"*. **That file is untouched by this PR and contains no such entry.** The finding
lives only in DEC-095's Consequences prose.

It is a **default-path silent downgrade** (no feature flag), so file it as a **`- [ ]` checkbox in
STAGE-042** — the same treatment its AC-8 sibling correctly got — bump the count, and **confirm
`just backlog` surfaces it**. Then correct the citation in the test comment.

## 4 — The AC-7 test named in the spec does not exist

`## Failing Tests` names `convert_optimize_auto_orient_bytes_unchanged`. No such test exists
anywhere. AC-7 itself is genuinely met (verify re-confirmed byte-identity across 5 fixtures × 3
verbs with a positive control) — **but nothing pins it going forward.** Write it.

## 5 — Record the unlisted deviation

`src/image/mod.rs` is touched (1-line `fn` → `pub(crate) fn` + doc). The change is right, but it is
in neither `## Build Completion`'s Deviations nor DEC-095's `affected_scope`. **Add both.**
Measured consequence of leaving it: `decisions-audit --changed` flags 11 unrelated decisions
because of that file, and DEC-095 will **not** surface when `src/image/mod.rs` changes later.

## 6 — DEC-095 describes a rule the code does not implement

Two measured gaps. **Fix the code or fix the description — and say which you chose, and why.**

- **Grayscale.** `Gray8 → resize → RGB8`. Narrowing is lossless (0/1024 px with R≠G≠B) but the type
  is not preserved: `convert` 783 B vs branch `resize` 2078 B (**+165%**; `main` was 2334 B). That
  is the spec's own clean-verb-vs-op-verb asymmetry, still open, at **13× the relative cost** of
  the RGB→RGBA case that motivated the spec. **AC-1/AC-2 name RGB only, so the ACs are literally
  met — Call 1 is not** (AGENTS §15: an AC may not transfer).
  **Preferred: fix it**, if it is the same narrowing mechanism. If it needs a materially different
  code path, scope it out **explicitly** — amend DEC-095, file a STAGE-046 `- [ ]`, and state the
  measured cost of deferring.
- **All-opaque RGBA input stays RGBA8**, because `narrow_rgba8` also requires
  `!original_color.has_alpha()` — which DEC-095 never mentions. **The behaviour is right** (do not
  strip a channel the user supplied); the description is wrong. Fix the description.

## 7 — Two tests that measure the wrong thing

- **AC-6's test compares an RGB encode to an RGBA encode and never runs an op** — it passes
  identically on `main`, so it measures the PNG encoder, not the fix
  [[fixtures-from-the-code-under-test-cannot-fail]]. Rewrite it to run the op. The real win, from
  verify: `rgb8` resize **1579 → 1323 B (−16.2%)**, `gray8` **−11.0%**.
- **The downgrade warning says "16-bit" for any >8-bit source** — a 32-bit-float source hits the
  same branch. Make the message true.

## 8 — Release notes

16-bit sources get materially **bigger** outputs (`rgb16` resize: 1579 → 3443 B, **+118%**). That
is restored fidelity, not bloat — but users must be told. Add it where the wave's release note
lives, and keep it consistent with SPEC-122/SPEC-124 landing in the same release.

---

## When you finish

1. Update `## Build Completion` — Deviations (item 5) and what you chose for item 6.
2. Amend **DEC-095** — `affected_scope`, the grayscale ruling, the `has_alpha` rule.
3. **Append a cost session** for this punch-list cycle (see `cost-snippet.md`) — do not overwrite
   the build's.
4. **Do NOT run `advance-cycle`.** `cycle:` stays at `verify` for re-approval.
5. Push to the same branch. **Do not merge.**

## Guardrails

- Full matrix again (default / `--no-default-features` / `--features webp-lossy`), fresh per-leg
  `CARGO_TARGET_DIR`, sequential. Items 2, 4, 6 and 7 all change tests.
- **Budget ~150 exchanges.** ⚠ The build ran **555** against ~250 and cost **$58.50**; the
  checkpoint never fired. If you pass 150 without item 2 resolved, checkpoint and report.
- **Never poll CI** — background it: `gh pr checks 181 --watch --interval 30`. Cost reading after.
- macOS has no `timeout(1)`. `git commit -s`. A piped command reports the pipe's exit code.
