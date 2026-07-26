# SPEC-105 — sub-agent readouts (maintainer review)

Build/verify ran as dedicated Opus sub-agents. Prompts: `SPEC-105-{build,verify}.md`.
⚠ Both reported the feature matrix passing, but that was a **stale-incremental-build
false green** — a clean CI build caught real no-AVIF-leg test breakage, fixed in the ship
CI-parity pass (see the spec's Ship reflection). The classifier feature itself was sound.

---

## build — Opus (`spec-105-entropy-photo-classify`, 2026-07-25)

**Prompt:** `prompts/SPEC-105-build.md` · **Cost:** ~640k tok est (main-loop)

> Added `PHOTO_ENTROPY_STRONG = 4.0` + a strong-entropy → `Photograph` rule (after EXIF/Document, before
> the graphic gates) in the shared `classify()`. Calibration (all via `optimize --explain=json`,
> EXIF-stripped): 48 real photo crops floor **4.58**; hard-edged graphics ≤ ~1.6; dithers ≤ 3.43 (committed
> 8-colour Floyd–Steinberg 3.03). Threshold 4.0 sits in the (3.43, 4.58] gap. Symptom fixed end-to-end:
> Leica DNG → `photograph`, AVIF 63,894 B (was 843,252 B lossless WebP), native + wasm. Committed 4 real
> fixtures. DEC-047 amended. **Reported all gates green** — but via incremental builds (the stale-artifact
> trap the verify agent also hit and CI later exposed).

---

## verify — Opus (`spec-105-entropy-photo-classify`, 2026-07-25)

**Prompt:** `prompts/SPEC-105-verify.md` · **Cost:** 143,805 tok (real)

> **VERDICT: CLEAN (on the feature — the classifier).** ★ Decoded + EXIF-stripped **64 real Nikon D3300
> RAWs** and ran every one through the fixed classifier — all 64 → `photograph` → AVIF, entropy floor 5.87
> (well above 4.0). Mutation proof: with the rule disabled, the high-flat shots fall to graphic-logo →
> lossless WebP — reproducing the maintainer's exact "Auto → WebP not AVIF" bug. Grayscale Leica → AVIF.
> Dither 3.03 stays lossless (differentiator preserved). Threshold mutation-checked both directions.
> ⚠ **Its feature-matrix "green" was a stale-incremental-build false pass, and it left uncommitted
> codec-agnostic test fixes in the checkout** — both surfaced only at CI. Real breakage the ship pass then
> fixed cleanly. The classifier verdict itself held: the feature is correct.
