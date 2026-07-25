# SPEC-104 — BUILD prompt

Cycle: build. You are NOT the architect. The spec file is your primary context.

Read in order:
1. `/AGENTS.md` — conventions (in-process fixtures, DCO sign-off, `just` recipes).
2. `/projects/PROJ-008-wasm-core-and-demo/specs/SPEC-104-raise-the-demo-raw-preview-gate-40-to-60-megapixels.md`
   — the whole spec, esp. `## Notes for the Implementer` (the [60, 64] Mpix straddle).
3. The referenced code: `src/wasm.rs:526-597`, `src/image/mod.rs:54-94`, `src/image/raw.rs`
   (`largest_declared_preview_pixels` + its boundary test), `tests/wasm_roundtrip.rs`.
4. `decisions/DEC-082-raw-preview-on-wasm-behind-a-demo-pixel-gate.md` (you amend it).
5. `/guidance/constraints.yaml` — `untrusted-input-hardening`, `ergonomic-defaults`.

## What to build (small — one constant + tests + a DEC amendment)

- Branch `spec-104-raise-raw-gate` off `main`.
- `src/wasm.rs`: `MAX_RAW_PREVIEW_MEGAPIXELS: u64 = 40` → `60`. Rewrite its doc comment
  per the spec (no longer a "framing default"; 60 clears real Leica previews while
  staying below the 64 Mpix native cap so the CLI-fallback stays honest; global raise;
  accepted mobile tradeoff).
- Move the over-threshold gate test: the bomb must now declare **between 60 and 64
  Mpix** (~62 Mpix, e.g. dims whose product is ~62,000,000 and < 67,108,864) so it
  tests the DEMO gate, not the native cap. Keep a mutation check proving the gate
  tracks the constant. Update the `raw.rs` boundary unit test to straddle 60 Mpix.
- Amend DEC-082 (dated note): retuned 40→60 after the maintainer hit it on a real
  desktop Leica DNG; 40 conflated a mobile bound with a global one; 60 stays below the
  64 Mpix native cap; on-device *mobile* verification is still the open item.

## Non-negotiables
- Native `src/` behavior unchanged — do NOT touch `MAX_IMAGE_PIXELS`/`check_pixel_budget`
  or any native decode path. Full native gate suite green.
- No panic on any input path; typed `JsError` only; gate still rejects on the header
  peek before allocation.
- Both fallback messages unchanged and distinct; no `raw:`/`Tiff is not supported` leak.
- Build wasm through `just wasm-build`; report the brotli delta (expect ~0 — it's a
  constant). rtk footgun: cross-check greps with raw `grep` + a positive control.
- DCO-sign every commit.

## When done
1. Fill the spec's `## Build Completion` (incl. the 3 reflection questions).
2. Append a `build` cost session to `cost.sessions` (best-available numbers).
3. `just validate` green.
4. `just advance-cycle SPEC-104 verify`.
5. Open a PR from `spec-104-raise-raw-gate` (name PROJ-008, STAGE-029, SPEC-104,
   DEC-082 amendment). Mark build `[x]` in the timeline with PR #, cost, date.

Do NOT merge. Hold for verify (Opus) + the orchestrator's merge.
Report back to the orchestrator: branch, PR #, brotli delta, criteria met, `just
validate` status, deviations, follow-ups.
