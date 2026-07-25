# SPEC-103 — sub-agent readouts (maintainer review)

Each build/verify cycle runs as a dedicated sub-agent (separate context + model). This
file captures the **readout** each sub-agent reported back to the orchestrator, so the
maintainer can review what happened without reading the full session log. The matching
**prompt** (the input) is the sibling file named for the cycle
(`SPEC-103-<cycle>.md`).

---

## build — Sonnet (`spec-103-raw-on-wasm`, 2026-07-24)

**Prompt:** `prompts/SPEC-103-build.md` · **Cost:** 301,224 tokens (real, from the
sub-agent usage object)

> **Branch:** `spec-103-raw-on-wasm` (off `main`, not merged). **PR:** #111 — OPEN, held for verify.
>
> **Brotli delta (real `just wasm-build`):** +1,262 B vs the probe's baseline
> (1,395,239 → 1,396,501 B) — within margin of the probe's +1,214 B estimate; no codec pulled in.
>
> **Acceptance criteria:** all met. Key proof points:
> - RAW opens end-to-end in a real headless-Chrome smoke run (drop → preview → convert, 0 network requests).
> - Gate fires *before* decode: a bomb declaring 50.4 Mpix (between the new 40 Mpix demo gate and the
>   native 64 Mpix DEC-063 cap, so only the new gate can catch it) with ~16×12 real entropy is rejected;
>   a real preview under threshold still extracts (mutation check). Boundary math (±5,650 px around
>   40,000,000) pinned by a cheap native unit test.
> - "Too large" vs "no decodable preview" are distinct, maintainer-approved verbatim messages; neither
>   leaks `raw:`/`Tiff is not supported`.
> - `isRawExtension` mirrors `RAW_EXTENSIONS` (no second JS array — grep with a positive control).
> - Published API untouched (`just wasm-npm-smoke` green); native `src/` unchanged (new `raw.rs` helper
>   is `pub(crate)` + `cfg(any(wasm32, test))`, so it doesn't compile into native non-test builds).
>
> **Test/lint — all green:** `just wasm-test` 25/25 (4 new); `just demo-smoke` (incl. new RAW section);
> `just wasm-npm-smoke`; full native `cargo test` 32/32 binaries; clippy `-D warnings`, `cargo fmt --check`,
> `cargo build --no-default-features`, `just validate`.
>
> **Deviations:** the spec's Failing Tests were prose → wrote the actual Rust/JS tests; added a fixture
> generator (`examples/gen_raw_gate_fixtures.rs`) + 2 committed fixtures (JS has no JPEG encoder); the
> over-threshold bomb intentionally sits between the two ceilings so it exercises the *new* gate; the
> "just under threshold" mutation check uses a real 3 Mpix preview (building a real ~40 Mpix JPEG inside
> the wasm VM would itself allocate the memory the gate avoids) — precise boundary math covered natively.
>
> **New decision:** DEC-082. **Follow-ups:** on-device threshold tuning (out of scope, launch-readiness);
> CR3 still unverified on wasm against a real file (same gap the probe recorded).

---

## verify — Opus (`spec-103-raw-on-wasm`, 2026-07-24)

**Prompt:** `prompts/SPEC-103-verify.md` · **Cost:** 152,014 tokens (real, from the
sub-agent usage object)

> **VERDICT: CLEAN.** Checked out `spec-103-raw-on-wasm` (PR #111) and verified adversarially. Every
> load-bearing claim holds, most re-driven by hand rather than read.
>
> **Re-driven BY HAND:**
> - **RAW output is a real image** — decoded the extracted PNG with `sips` + `file` (decoders the build
>   didn't write): valid 64×48 PNG, matches the wasm test's dims.
> - **Gate fires before decode, non-vacuous** — mutation test: raised `MAX_RAW_PREVIEW_MEGAPIXELS` 40→60,
>   reran the wasm suite; the 50.4 MP bomb test *flipped to FAIL* (others pass), proving the gate is driven
>   by the constant, not vacuous. Bomb fixture <1 KB → rejection can only be the header peek. Restored to 40.
> - **Approved strings verbatim, no leak** — real headless-Chrome banner reads the approved copy exactly;
>   no `raw:`, no `Tiff is not supported`; "too large" and "no preview" distinct.
> - **No hand-copied JS ext list** — raw `grep` + positive control; routing goes through `isRawExtension`.
> - **API additive-only** — freshly-built `.d.ts` + `just wasm-npm-smoke`: 6 signatures unchanged, 2 new
>   additive, zero-deps.
> - **No native behavior change** — clippy clean (helper correctly `cfg(any(wasm32,test))`); 786 native
>   tests pass, 0 failures.
> - **Brotli delta** — `just wasm-build`: 1,396,501 B = +1,262 B, exactly the build's claim.
> - **Zero network on RAW path** — confirmed.
>
> **Gates:** `just validate`, lean build, `just wasm-test` 25/25, `just demo-smoke`, `just wasm-npm-smoke`
> — all green.
>
> **Findings:** none, real or minor. Completeness traps checked (build-completion table has a row per
> acceptance criterion; Failing Tests genuinely can fail — proven by the mutation flip).
>
> **Commit:** `c36fcac verify(SPEC-103): adversarial pass CLEAN` — DCO-signed; timeline verify `[x]`;
> cycle advanced build→ship. Not merged — held for maintainer go-ahead.
