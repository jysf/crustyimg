# SPEC-104 — VERIFY prompt

Cycle: verify. You are NOT the architect or builder. Verify the SPEC-104 build on its PR
branch **adversarially** — a one-constant retune, so the risk is a test that now proves
the wrong thing, or a native-path regression.

## Setup
- Repo: `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg`.
- This prompt is on `main` (read it now, before switching). Build is on **`spec-104-raise-raw-gate`**
  (PR #112). Check it out: `git checkout spec-104-raise-raw-gate`. Do NOT merge, do NOT push to `main`.
- DCO-sign any verify commit; `git rebase --signoff main` if one lands unsigned.

## Read first
1. `projects/PROJ-008-wasm-core-and-demo/specs/SPEC-104-raise-the-demo-raw-preview-gate-40-to-60-megapixels.md`
2. `decisions/DEC-082-*.md` (the amendment)
3. `git diff main...spec-104-raise-raw-gate`

## Verify — with evidence, not by re-reading the build's asserts
1. **The constant is 60** and `MAX_RAW_PREVIEW_PIXELS == 60_000_000`.
2. **The demo-gate test tests the DEMO gate, not the native cap.** The over-threshold
   bomb must declare **strictly between 60 Mpix (60,000,000) and the native 64 Mpix cap
   (67,108,864)** — the build says 62.4 Mpix. Confirm the number, and MUTATION-test it:
   temporarily lower `MAX_RAW_PREVIEW_MEGAPIXELS` and confirm a mid-window case flips;
   temporarily raise it toward/over 64 and confirm the bomb would then be caught by the
   NATIVE cap instead (proving the current test isolates the demo gate). Restore.
3. **The happy path holds at 60** — a real sub-60 MP preview still extracts and converts
   (independent-decode the output dims, don't trust the assert).
4. **The regenerated fixture is sound.** The build regenerated
   `tests/fixtures/raw/oversize_preview.dng` (50.4 → 62.4 Mpix) because 50.4 fell under
   the new gate. Confirm: the new fixture actually declares ~62.4 Mpix (in-window),
   `no_preview.cr2` is unchanged/equivalent, and `demo-smoke` exercises the rejection
   for real (not a timeout masquerading as a pass).
5. **Native untouched.** `MAX_IMAGE_PIXELS`/`check_pixel_budget` and every native decode
   path are byte-unchanged; full native suite green (`cargo test`).
6. **Fallback copy unchanged and distinct**; no `raw:`/`Tiff is not supported` leak.
7. **Brotli ~0** (`just wasm-build`) — a constant edit shouldn't move code size.
8. `just validate`, `just wasm-test`, `just demo-smoke`, `just wasm-npm-smoke`,
   `cargo build --no-default-features` all green.
9. **DEC-082 amendment** honestly records the retune + that mobile verification is still open.

## When done
- VERDICT: CLEAN or NOT-CLEAN, each finding (real / severity / evidence). Fix small
  defects minimally + DCO-signed, or escalate to the architect and stop.
- If CLEAN: `just advance-cycle SPEC-104 ship` (note: the build flagged that advance-cycle
  can mis-target `specs/prompts/*.md` — verify it edited the real spec's `task.cycle`, fix
  by hand if not), mark verify `[x]` in the timeline. Do NOT merge.
- Report to the orchestrator: verdict, findings with evidence, what you re-drove by hand,
  brotli delta, gate status, and this session's real cost numbers.
