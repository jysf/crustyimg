# SPEC-122 — PUNCH LIST prompt

Cycle: **build** (punch-list return). Verify returned **⚠ PUNCH LIST** on PR #182 and confirmed the
fix independently — all three cases reproduced, AC-7's numbers to the digit, the wasm guard
relaxation **ruled legitimate** after being forced red. **Do not re-litigate the fix.** 5 items.

Branch `fix/spec-122-resize-resamples-in-linear-light`, head `90a7167`. **Own worktree.** `cycle:`
stays at `verify` — do not advance it.

## 1 — The DEC-092 correction, and ⚡ the correction itself is wrong

**(a) It never landed.** The `decisions/` diff touches only DEC-095. **`DEC-092:127-128` still reads
*"i.e. Lanczos ringing at hard corners"*** — refuted, unamended, no forward pointer. And
`decisions-audit` names DEC-092 as governing `src/operation/mod.rs`, so the next builder is pointed
straight at the record carrying the refuted mechanism. Add an `### Amended` section, matching the
convention DEC-095 already uses.

**(b) The tool still prints it.** `scripts/spec120_linear_light.py:312-313` hard-codes *"the max is
Lanczos ringing at hard corners, not a halo"* — and now emits it beside `max alpha err 0`. That line
is where DEC-092's wording came from; fix it at the source, not just in the record.

**(c) ⚡ The replacement mechanism is refuted by this build's OWN control — do not write it down.**
Build Completion says *"8-bit quantization inside `fir`'s premultiply/divide round-trip"*. But:

- **alpha is never premultiplied or divided**, and the harness shows the alpha channel itself going
  27 → 0;
- **control C4, with premultiplication OFF, still reads `max alpha err 27` on the u8 arm.**

So the residual is **8-bit quantization in the integer resampling path generally — the alpha
channel's own convolution included — not the premultiply/divide round-trip specifically.** Right in
kind, wrong in the specific. **Cite C4 as the evidence when you write it**; a correction to a wrong
mechanism that is itself wrong is worse than the original.

## 2 — AC-9 measured time and never measured memory, and half the cost is free

`F32x4` multiplies the resize working set. Measured by verify:

| case | peak RSS before → after | |
|---|---|---|
| 4000×2660 `--max 400` | 166 → **465 MB** | 2.8× |
| 512² → 6000×6000 | 266 → **1407 MB** | **5.3×** |

- **Zero mentions of memory** in the spec or DEC-095. Record it.
- ⚠ **`MAX_AREA`'s comment (`src/operation/mod.rs:867-869`) is now false** — it describes a 512 MiB
  untrusted-input allocation bound whose float intermediates are now **4× that**. Untrusted input is
  the whole point of that bound. Fix the comment; if you believe the bound itself needs to move,
  **say so and stop** rather than changing it here.
- ✅ **Half of it is free:** `resample_linear_f32x4` returns `dst.pixels().to_vec()` — a full second
  copy of the destination float buffer (**576 MB** in that test). Eliminate the copy and re-measure.

The remaining half is the working type itself, which is the **same** `F32x4` decision behind the
3.83× slowdown. **Do not change the working type here** — it is a maintainer decision, and both
findings feed it. Report the post-fix numbers so that decision has data.

## 3 — The AC-10 webp-lossy row is a false green

Reported *"835 passed / 28 suites"*. **39 suites is invariant** — 36 test files + lib + bin + doc,
zero required-features — so that row matches nothing. Verify measured `--features webp-lossy` =
**933/39** and `--no-default-features --features webp-lossy` = **911/39**, both green, clippy exit 0.

**Re-run that leg properly and report the real counts.** The other two rows reproduce exactly; this
one was not a complete run [[a-harness-that-exercises-nothing-reports-green]].

## 4 — `affected_scope` was confirmed on a false premise

Build Completion says *"the diff touches `src/operation/mod.rs` and `tests/` only"*. It also touches
**`scripts/lib/wasm-artifact.mjs`** and **`scripts/demo-assemble.mjs`**. DEC-095 records why
`WASM_BROTLI_BASELINE` moved, but nothing surfaces that record when someone next edits it — **the
exact failure DEC-095's own scope comment warns about.** Correct the Deviations line and extend
`affected_scope`.

## 5 — Minor, pre-existing, newly load-bearing

The lean arm prints `── .wasm size (features: --no-default-features --features avif) ──` three lines
above the guard failure, because `@just wasm-size` is a **fresh `just` invocation** that does not
inherit `--set _wasm_features`. The rewritten guard message tells the reader to *"check the
wasm-pack line above actually carries `--features avif`"* — and **the nearest feature line in that
log contradicts the true one.** Make the message and the log agree.

## When you finish

1. Update `## Build Completion` — Deviations (item 4), the memory numbers (item 2).
2. **Amend DEC-092** (item 1a) and **DEC-095** (items 2, 4). Item 1c's wording must cite C4.
3. **Append a cost session** for this punch-list cycle — do not overwrite the build's.
4. **Do NOT run `advance-cycle`.** `cycle:` stays at `verify`.
5. Push to the same branch. **Do not merge.**

## Guardrails

- **⚡ NEVER POLL CI, and do not re-read a backgrounded watcher's output while it runs.**
  `gh pr checks 182 --watch --interval 30`, then leave it alone. **Measured on this spec's build:
  ~$60 of $103.60 went on CI polling** — over half. Take the cost reading once, at the end.
- **Budget ~150 exchanges.** The build ran 608.
- Full matrix again — item 3 requires it, and items 2 and 5 change code.
- macOS has no `timeout(1)`. `git commit -s`. A piped command reports the pipe's exit code.
