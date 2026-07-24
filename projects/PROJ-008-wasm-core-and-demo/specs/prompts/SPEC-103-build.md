# SPEC-103 — BUILD prompt

Cycle: build. You are NOT the architect who wrote this spec. The spec file is your
primary context; read it in full.

Read files in order:

1. `/AGENTS.md` — conventions (fixtures generated in-process, DCO sign-off, `just` recipes).
2. `/projects/PROJ-008-wasm-core-and-demo/specs/SPEC-103-wire-raw-decode-into-the-demo-behind-a-pixel-gate.md`
   — the spec. Read the ENTIRE `## Implementation Context` and `## Notes for the Implementer`.
3. `/docs/research/proj-008-raw-on-wasm-probe.md` — the design-time probe. It has the
   measured numbers, the verified wiring shape (a working `raw_preview_probe` export was
   prototyped and independently verified), and the ruled-out alternatives. Trust it.
4. `/projects/PROJ-008-wasm-core-and-demo/stages/STAGE-029-demo-launch-quality.md` — the stage.
5. `/projects/PROJ-008-wasm-core-and-demo/brief.md` — the project.
6. The spec's referenced decisions: DEC-055, DEC-063, DEC-064, DEC-065.
7. `/guidance/constraints.yaml` — `untrusted-input-hardening`, `ergonomic-defaults`.

Before coding, mark the build cycle `[~]` in the timeline
(`SPEC-103-…-timeline.md`) — already set. If you hit something needing architect
judgment (constraint conflict, the published API can't stay additive, the gate can't
reject before decode), change it to `[?]` with a one-line reason and STOP.

## What to build (all detail is in the spec — this is the shape)

- **Branch:** `spec-103-raw-on-wasm` off `main`.
- Additive `#[wasm_bindgen]` exports in `src/wasm.rs`: `rawPreview(&[u8]) -> Result<Vec<u8>, JsError>`
  (extract largest embedded preview → PNG bytes) and `isRawExtension(&str) -> bool`
  (thin wrapper over `raw::is_raw_extension`, so JS never copies the list).
- One tunable constant, **`MAX_RAW_PREVIEW_MEGAPIXELS = 40`** (maintainer-locked). The
  gate peeks the largest candidate's declared SOF dimensions and returns a typed
  "too large" error **before** the full-resolution decode/allocation — reuse the
  existing DEC-063 SOF-peek machinery in `raw.rs` (see
  `raw_preview_rejects_oversize_embedded_jpeg_before_decode`). Add whatever minimal
  `pub(crate)` dimension-scan helper this needs to `raw.rs`; keep it bounded by
  `MAX_PREVIEW_CANDIDATES`.
- Demo: `demo.js` routes on `isRawExtension(source.file.name)` → `rawPreview` → existing
  `optimizeDetailed` path; `index.html` `accept` gains the RAW extension tokens.
- **User-facing copy — ship VERBATIM (maintainer-approved):**
  - too-large fallback: `This RAW's built-in preview is very high-resolution — convert it with the crustyimg CLI instead.`
  - no-preview: `Couldn't find a preview image inside this RAW file.`
  Never leak the internal `raw: …` strings or `Tiff is not supported` into the UI.
- **DEC-082** (`/decisions/DEC-082-*.md`): RAW preview on the wasm/demo surface behind a
  demo-specific pixel budget. Record the 40 MP value + the safe/risky bracket
  (2.46 MP OK ↔ 46.7 MP ≈ 320 MB peak), that it's a wasm ceiling below the native
  DEC-063 cap (native unchanged), and that on-device tuning is a post-ship step.

## Make the Failing Tests pass

They're written in the spec's `## Failing Tests` — `tests/wasm_roundtrip.rs` (extract,
gate-fires-before-decode with a mutation check that just-under extracts, no-preview
distinct, `isRawExtension` mirrors the list) and `tests/demo_smoke.mjs` (RAW drops &
converts with 0 network requests, over-threshold shows the fallback copy, `accept`
lists RAW). Use synthetic fixtures (the `raw_blob`/`jpeg_declaring` primitives in
`raw.rs`), not real camera files.

## Non-negotiables

- **No panic on any input path** — a panic aborts the wasm module. Typed `JsError`
  everywhere; no `unwrap`/`expect` on input-derived values.
- **Published API untouched** — `info`/`transform`/`optimize`/`optimizeDetailed`/
  `score`/`version` keep exact signatures; new exports are purely additive.
  `just wasm-npm-smoke` must still pass.
- **No native `src/` behavior change** — the gate is wasm-only; full native gate suite green.
- Build the wasm through **`just wasm-build`** (the size profile — bare
  `cargo build --target wasm32` ships +109 KB). Report the real brotli delta vs the
  probe's +1,214 B; a big jump means a codec got linked that shouldn't be.
- **rtk footgun:** cross-check every grep sweep with raw `grep` + a positive control —
  the rtk hook silently zeroes `rg -c` here.
- All commits DCO-signed (`git commit -s`).

## When done

1. Fill the spec's `## Build Completion` including the three build-phase reflection
   questions (not optional).
2. Append a `build` cost session entry to the spec's `cost.sessions` (best-available
   numbers; null + a note if unavailable).
3. `just validate` green.
4. `just advance-cycle SPEC-103 verify`.
5. Open a PR from `spec-103-raw-on-wasm`; description names PROJ-008, STAGE-029,
   SPEC-103, decisions used, constraints checked, DEC-082.
6. Mark build `[x]` in the timeline with PR number, cost, date.

Do NOT merge. Hold for verify (Opus) + the orchestrator's merge on maintainer go-ahead.
