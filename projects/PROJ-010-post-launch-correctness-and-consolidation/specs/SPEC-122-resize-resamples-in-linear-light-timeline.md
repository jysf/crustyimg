# SPEC-122 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-122-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-16. Framed **as a pair with its sibling** — SPEC-121 and SPEC-122
      change output bytes for every recipe, share one DEC and one migration, and should be
      sequenced together so that cost is paid once.
      **Design's largest finding: the migration already exists.** `cache_key_for` includes
      `crate::version()` (`src/cli/build.rs:294`) and the lockfile never promised output-hash
      stability across versions (`src/build/lock.rs:32-36`), so the "invalidates every PROJ-007
      lockfile" risk both backlog entries flagged is already within the shipped contract. The
      builds drive it; they do not design it.
- [x] **build** — 2026-08-18, **Opus** (not the Sonnet the spec and prompt both name — the
      orchestrator's dispatch used `--model opus` and the prompt was never updated to match; the
      cycle priced at Opus anchors correctly and flagged the mismatch itself). PR #182,
      **$103.60** / 66 min / **608 messages**, re-derived to the cent; **99.44% cache reads**.
      `Resize::apply` linearizes → resamples in `F32x4` → re-encodes at the input's own bit depth.
      Alpha untouched (coverage, not light); a same-size resize short-circuits before conversion.
      **Against the same regenerated independent reference:** synthetic −63.85 → **100.00**,
      `graphic_large` 70.45 → **100.00**, `photo_forest` 84.45 → **99.41**; mean signed luma error
      → **0.000000** on all three. Both oracles agree. Reverting returns the harness to exactly
      −63.85 / 70.45 / 84.45. Two AFTER runs byte-identical. CI 16 pass / 0 fail.
      ⚖ **Four things the spec did not predict:** (1) **AC-6's upscale half cannot be met** — an
      upscale IS a resample and was defective the same way (65.93 → 100.00, 89.16 → 98.44);
      direction-gating would put a discontinuity at 100% and has no answer for `fill`/`cover`.
      **Orchestrator ruled: the AC was imprecise, fixing upscale is correct** (AGENTS §15, an AC may
      not transfer); the no-op half is met at four colour types. (2) **`resize` is 3.83× slower**
      (169 → 649 µs; 1.5–2.5× end to end) and **72% is the `F32x4` working type, not the transfer
      function** — the cheap fix does not exist; recovering the time means changing the working
      type. **Maintainer's call, filed as a follow-on.** (3) **AC-5 improved where it predicted a
      null** — translucent-edge error 27/255 → **0**; DEC-092's reading of that residual was WRONG
      (8-bit quantization inside `fir`'s premultiply round-trip, not Lanczos ringing), and DEC-092
      is a shipped decision that needs correcting. (4) **The wasm bundle shrank 16.9%**, turning a
      size-floor guard red; the build **moved the baseline** after checking the floor still catches
      a missing AVIF encoder, and fixed the guard's message which had asserted the wrong cause.
      ⚠ **~$60 of the $103.60 was CI polling, not work** — the same transcript measured $32.01 at
      231 messages with the build already finished. **The prompt was the cause: it carried no CI
      instruction at all.** SPEC-121's punch list had one; SPEC-122's build prompt was written
      before the lesson existed and tightened twice afterwards without it being added. The cycle had
      the right instinct (backgrounded watchers) and undercut it by re-reading their output.
- [x] **verify** — 2026-08-18, Opus, **⚠ PUNCH LIST**, **$15.82** / 19 min / 149 messages
      (re-derived to the cent). Read-only, no commits, detached at `90a7167`.
      **Everything re-measured independently.** All three cases reproduced (100.0000 / 100.0000 /
      99.4101, luma error 0.000000); Revert A returns **−63.8488 / 70.4507 / 84.4524 and alpha edge
      27 — AC-7's numbers to the digit**, Δ 163.8488. ⚡ That revert is also the strongest proof of
      AC-4: *a crustyimg-derived reference could not have scored the reverted binary at 70.45.*
      AC-6's no-op half confirmed **harder than the in-repo test does** — verify's first fixture
      failed its own positive control at 64→32, so it re-ran on high-contrast fixtures where all
      four colour types differ at 64→32 and are identical at 64→64.
      ⚖ **Item 2 ruling — the wasm guard relaxation STANDS.** Forced red: lean no-AVIF build trips
      it at 865,980 B against the 1,087,675 B floor, 20.4% clear. The band moved for a measured
      cause unrelated to what the guard protects, and the floor was checked **before** the move.
      ✅ Item 1's ruling basis holds (upscale is a measured improvement, independently reproduced);
      item 4's decomposition holds and was **understated** — verify measured 76% of the added time
      is the `F32x4` working type, against the build's 72%.
      ⚠ **5 items. The sharpest is 1c: the CORRECTION to DEC-092's wrong mechanism is itself
      wrong.** The build wrote *"8-bit quantization inside `fir`'s premultiply/divide round-trip"* —
      but alpha is never premultiplied or divided, and **control C4 with premultiplication OFF still
      reads `max alpha err 27`**. The residual is 8-bit quantization in the integer resampling path
      generally, the alpha channel's own convolution included. Right in kind, wrong in the specific,
      and about to be written down as a correction.
      Also: DEC-092 was never amended at all (and `decisions-audit` names it as governing
      `src/operation/mod.rs`, so the next builder is pointed at the refuted record);
      `scripts/spec120_linear_light.py:312-313` still **prints** the wrong mechanism beside
      `max alpha err 0`; **AC-9 measured time and never memory** — peak RSS 166 → 465 MB (2.8×) and
      266 → **1407 MB (5.3×)**, with `MAX_AREA`'s comment still describing a 512 MiB untrusted-input
      bound whose float intermediates are now 4× that, and **half of it free** (`to_vec` makes a
      second 576 MB copy of the destination buffer); the **AC-10 webp-lossy row is a false green**
      (reported 835/28 where 39 suites is invariant — the real counts are 933/39 and 911/39); and
      `affected_scope` was confirmed on a false premise (the diff also touches two `scripts/` files).
- [ ] **punch list** — prompt: `prompts/SPEC-122-punchlist.md` (2026-08-18). Opus, own worktree,
      same branch. 5 items. `cycle:` stays at `verify`. 💰 Verify cost to apply at ship:
      21,844,228 tokens / 19 min / **$15.82**. **SPEC-122 to date: $119.42.**

- [ ] **ship**
