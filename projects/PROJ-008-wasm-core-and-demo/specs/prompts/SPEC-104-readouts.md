# SPEC-104 — sub-agent readouts (maintainer review)

Build/verify ran as dedicated sub-agents (separate context + model). This file captures
each cycle's **readout**; the matching **prompt** is `SPEC-104-<cycle>.md`.

---

## build — Sonnet (`spec-104-raise-raw-gate`, 2026-07-24)

**Prompt:** `prompts/SPEC-104-build.md` · **Cost:** 151,255 tokens (real)

> **Branch:** `spec-104-raise-raw-gate`. **PR:** #112 (held for verify).
> **Brotli delta:** −147 B vs a same-tree `main` baseline — noise, as expected for a one-constant edit.
> **Acceptance:** all 6 met. `MAX_RAW_PREVIEW_MEGAPIXELS`/`_PIXELS` = 60 / 60,000,000; sub-60 MP extracts;
> the 60–64 Mpix window is rejected before decode by the demo gate specifically (bomb moved to 62.4 Mpix,
> native cap untouched); both fallback messages unchanged/distinct, no `raw:`/`Tiff` leak; native `src/`
> and `MAX_IMAGE_PIXELS` untouched, full native suite green.
> **Gates:** `cargo test` 786 pass; lean build clean; `just wasm-test` 25/25; `just demo-smoke` green;
> `just wasm-npm-smoke` green; `just validate` 230 blocks.
> **Deviations:** (1) after raising the constant, `just demo-smoke` failed — the committed fixture
> `oversize_preview.dng` declared 50.4 Mpix, over the *old* 40 gate but under the *new* 60, so it stopped
> triggering rejection and the smoke timed out. Regenerated it at 62.4 Mpix via `gen_raw_gate_fixtures`
> (reproducible) and updated stale prose comments. (2) `just advance-cycle` silently targeted
> `specs/prompts/SPEC-104-build.md` (no `cycle` field) instead of the real spec — `find_spec()`'s glob in
> `scripts/_lib.sh` also matches `specs/prompts/*.md`; fixed `task.cycle` by hand, filed the script bug
> separately (now in `docs/repo-tooling-backlog.md`).
> **DEC-082:** amended in place (dated note; original kept as history).

---

## verify — Opus (`spec-104-raise-raw-gate`, 2026-07-25)

**Prompt:** `prompts/SPEC-104-verify.md` · **Cost:** 95,240 tokens (real)

> **VERDICT: CLEAN.** A faithful one-constant retune; every load-bearing check passed under adversarial,
> evidence-based verification.
> **Re-driven by hand:** constant/window math (bomb 8000×7800 = 62,400,000 sits strictly in the
> (60,000,000, 67,108,864) isolation window); **MUTATION test** — raising `MAX_RAW_PREVIEW_MEGAPIXELS`→63
> (above the bomb, below the native cap) made the rejection test FAIL while the happy path still passed,
> proving the *demo gate* — not the native cap — rejects; lowering the native boundary test's constant
> made its under-arm fail (non-vacuous). Fixture: independent SOF0 scan confirms 62.4 Mpix, byte-identical
> sha256 across a fresh `gen_raw_gate_fixtures` run. Native `MAX_IMAGE_PIXELS`/`check_pixel_budget`
> byte-unchanged; fallback strings byte-unchanged and distinct; `demo-smoke` exercises the real rejection
> (not a timeout). Brotli −147 B, raw `.wasm` byte-identical.
> **Process note flagged:** hit an mtime/incremental-compile stale-object race during mutation testing (a
> `mv`-restore gave the file an older mtime than the compiled object → cargo reused stale output); forced
> recompile → green. Not a code defect; a real footgun for mutation testing.
> **Gates:** native `cargo test` 440 pass; `just wasm-test` 25; validate / demo-smoke / npm-smoke / lean
> all green. Advanced cycle→ship (fixed the find_spec mis-target by hand), DCO-signed. Not merged.
