# SPEC-121 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-121-<cycle>.md`.

## Instructions

- [x] **design** — 2026-08-16. Framed **as a pair with its sibling** — SPEC-121 and SPEC-122
      change output bytes for every recipe, share one DEC and one migration, and should be
      sequenced together so that cost is paid once.
      **Design's largest finding: the migration already exists.** `cache_key_for` includes
      `crate::version()` (`src/cli/build.rs:294`) and the lockfile never promised output-hash
      stability across versions (`src/build/lock.rs:32-36`), so the "invalidates every PROJ-007
      lockfile" risk both backlog entries flagged is already within the shipped contract. The
      builds drive it; they do not design it.
- [x] **build** — 2026-08-18, Sonnet, PR #181 (`4391e06`), **$58.50** / 60 min / **555 messages**.
      Cost re-derived to the cent by the orchestrator; 99.2% cache reads. ⚠ **555 messages against
      a ~250 budget** — the checkpoint did not fire. Second-most-expensive build in the wave despite
      running on Sonnet's cheaper anchors; at Opus rates it would have been ~$97.
      Three op bodies now widen to work at the input's own bit depth and narrow back losslessly.
      `Watermark` decides RGBA-vs-RGB from the actual composited alpha (Call 2), both directions
      tested. Call 3's 8-bit-downgrade diagnostic added at the sink. New
      `tests/colour_type_preservation.rs` (11 tests, one op body per test, three real reverts for
      AC-9) plus two `tests/sink.rs` tests. Full matrix clean; AC-7 driven.
      **DEC-095 minted at the reserved id — no collision.** Sweep corrected
      `docs/lab-plan-2026-08.md` and `docs/roadmap.md`.
      ⚖ **AC-8 returned a finding rather than stopping:** the cache-key-changes-on-release safety
      net **only fires on an actual version bump**, driven both ways on a real build target. Filed
      to STAGE-042. **This is load-bearing past this spec** — SPEC-122's Call 5 and SPEC-124 rest on
      the same "the migration already exists" reasoning.

- [x] **verify** — 2026-08-18, Opus, **⚠ PUNCH LIST**, **$11.08** / 12 min / 110 messages
      (cheapest verify in the wave; cost re-derived to the cent). Read-only, no commits.
      Reproduced every headline claim on **independently hand-written PNG fixtures** (pure-Python
      zlib, not the `image` crate under test). **The central fix holds.**
      ✅ Confirmed genuinely good: AC-3's control **can fail** (forced "always narrow" → exactly the
      3 alpha tests red, AC-1/AC-2 green); AC-9's three reverts are a **clean partition** (Invert 2
      red, Resize 4 red incl. `web`, Watermark 1 red); AC-7 byte-identity re-confirmed vs its own
      `main` build with a positive control; AC-5 warning is stderr-only, `-o -` stays pure JPEG.
      ⚖ **Lead rulings:** (1) sweep **incomplete and wider than the orchestrator found** — 4 live
      premises, not 1; (2) `src/image/mod.rs` **justified but unrecorded**; (3) AC-8 **"filed, not
      stopped" was CORRECT** — the contract holds when the version moves; what fails is its
      *precondition* under the spec's own "do not bump the version" guardrail. A release-discipline
      finding, not a broken contract. Residual exposure: source/`main` builders mid-wave get stale
      pre-fix bytes with `build --check` reporting exit 0. Closes at the next tag — **and SPEC-122
      and SPEC-124 land inside that window.**
      ⚠ **Substantive defect (item 2):** `watermark --text` on an opaque photo **still returns
      RGBA8** — source-over onto an opaque base is mathematically always opaque, so the observed
      alpha histogram `{255: 988, 254: 36}` is f32 rounding + a truncating cast in `image`'s
      `Rgba::blend`. 26 px in 65,536 defeat the narrow, and both AC-4 tests use uniform overlays
      that never exercise the float path. `--text` is untested.
      ⚠ Also: a **false citation** in `tests/colour_type_preservation.rs:215`; the AC-7 test named
      in `## Failing Tests` **does not exist**; AC-6's test **never runs an op** so it passes
      identically on `main`; and grayscale `Gray8 → resize → RGB8` leaves Call 1 unmet at **13× the
      relative cost** of the RGB→RGBA case (+165% vs `convert`) while the ACs are literally met.
- [x] **punch list** — 2026-08-18, Opus, `d46ef37`, **$21.59** (re-derived to the cent). All 7
      items closed. CI green on the code commit `8066c24` (18 checks); local matrix 917 / 897 / 923
      passed, clippy and fmt clean per leg.
      **Two behaviour changes.** `watermark --text` now narrows — **66,313 → 53,970 B, −18.6%** on
      an opaque base. Root cause pinned exactly: `image`'s `Rgba::blend` computes result alpha in
      f32 and **truncates** the cast, so `1.0 + a − a = 0.99999994` for **32 of 254** overlay
      alphas — anti-aliased glyph edges hit precisely those, and 36 px in 65,536 defeated the scan.
      Fixed by restoring the alpha the algebra requires, gated on the base having had no alpha:
      **exact, no tolerance, and independent of whether `image` truncates or rounds** — which also
      kills the latent CI break verify flagged.
      **Grayscale is preserved**: `resize --max 16` L8 **852 → 340 B (−60.1%)**, L16 −61.8%, La8
      −53.5% — roughly **4× the relative saving of the RGB→RGBA case that motivated the spec.**
      ⚠ **Deviation, recorded not silent.** Punch-list item 2 (orchestrator-authored) asked for two
      incompatible things: an opaque composite must narrow AND a translucent overlay must keep
      RGBA. Source-over onto an alpha-less base is opaque for **every** overlay alpha, so under the
      second rule the item had no fix. Read as *the composite decides, no numeric tolerance*, and
      `watermark_keeps_alpha_when_overlay_is_translucent` retargeted onto a base with genuine
      transparency. **The old form passed only because 128 is one of the 32 truncating alphas.**
      Orchestrator reviewed and agrees; the contradiction was the prompt's.
      ✅ Orchestrator checked the risk a fresh verify would have chased — the new grayscale
      narrowing has controls in **both** directions: `rgb_input_that_happens_to_be_gray_stays_rgb`
      (do not over-narrow) and `colour_watermark_on_a_gray_base_becomes_rgb` (widen when chroma is
      added), plus `graya_opaque_input_keeps_its_alpha_channel`. Item 4's missing AC-7 test now
      exists as `convert_optimize_auto_orient_bytes_unchanged`.
      ⚠ **The lossless-WebP finding is worse than DEC-095 stated** and is now a STAGE-042 `- [ ]`:
      `convert --format webp` reaches it on the **default path, no feature flag**, and prints
      **`ssim 100.0` while halving the depth** — SSIM is computed on 8-bit renderings, so the
      honest-size line reassures about exactly the loss the metric cannot see.
      *(`cost.sessions` carries no verify entry by design — verify is read-only since 2026-08-18;
      its $11.08 is held above and applied by the orchestrator at ship.)*
- [ ] **re-approval / ship** — `cycle:` held at `verify`. **SPEC-121 to date: $91.17** (build
      $58.50 + punch list $21.59 + verify $11.08).

- [ ] **ship**
