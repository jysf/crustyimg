# SPEC-103 — VERIFY prompt

Cycle: verify. You are NOT the architect or the builder. Verify the SPEC-103 build on
its PR branch **adversarially** — your job is to find where the build's claims don't
hold, not to confirm them.

## Setup

- Repo: `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg`.
- The build is on branch **`spec-103-raw-on-wasm`** (PR #111), off `main`. Check it out
  and work there: `git checkout spec-103-raw-on-wasm`. Do NOT merge. Do NOT push to `main`.
- If you make verify-cycle commits, DCO-sign every one (`git commit -s`). If a commit
  lands unsigned, fix with `git rebase --signoff main` before finishing.

## Read first

1. `projects/PROJ-008-wasm-core-and-demo/specs/SPEC-103-wire-raw-decode-into-the-demo-behind-a-pixel-gate.md`
   — the spec (Acceptance Criteria, Failing Tests, Implementation Context, and the
   build's `## Build Completion` claims).
2. `docs/research/proj-008-raw-on-wasm-probe.md` — the probe (ground truth for the
   mechanism and the measured numbers).
3. `decisions/DEC-082-*.md` — the new decision.
4. The diff: `git diff main...spec-103-raw-on-wasm`.

## Verify these — with independent evidence, not by re-reading the build's asserts

1. **The RAW output is a real image, checked by a decoder the build did NOT write.**
   Drive `rawPreview` (or the demo path) on a synthetic RAW fixture and confirm the
   returned PNG decodes to the expected dimensions with an INDEPENDENT decoder — macOS
   `sips`/`file`, not just a magic-byte sniff ([[verify-wasm-output-with-an-independent-decoder]]).
2. **The gate fires BEFORE the full decode — re-drive it, don't trust the test name.**
   The claim is a preview declaring > 40 MP is rejected on the SOF header peek, before
   the multi-hundred-MB allocation. Confirm the rejection path allocates ~nothing.
   Then run the NEGATIVE CONTROL: a preview just UNDER 40 MP must extract normally — a
   gate that rejects everything (or nothing) is vacuous. The build's bomb declares
   50.4 MP (between the 40 MP demo gate and the 64 MP native DEC-063 cap) so only the
   new gate can be catching it — verify that framing actually holds (mutate the
   threshold and prove the 50.4 MP case flips).
3. **"Too large" vs "no decodable preview" are genuinely distinct**, and NEITHER leaks
   `raw:` or `Tiff is not supported` into the demo UI. Check the exact user-facing
   strings match the maintainer-approved copy verbatim.
4. **`isRawExtension` mirrors `RAW_EXTENSIONS` with no second copy in JS.** Grep the
   demo for a hand-copied extension list; cross-check the grep with raw `grep` + a
   positive control (the rtk hook silently zeroes `rg -c` here —
   [[rtk-can-silently-corrupt-grep-counts]]).
5. **The published API is genuinely untouched** — `info`/`transform`/`optimize`/
   `optimizeDetailed`/`score`/`version` signatures byte-identical; the new exports are
   purely additive. `just wasm-npm-smoke` green.
6. **No native `src/` behavior change** — the gate helper must not alter native decode.
   Confirm the full native gate suite passes and that the new `raw.rs` helper is
   correctly cfg-scoped (the build says `cfg(any(wasm32, test))` — verify it doesn't
   compile into native non-test builds).
7. **Re-measure the brotli delta** through `just wasm-build`; confirm it's within a
   small margin of the probe's +1,214 B (build claims +1,262 B) and no unexpected
   codec was linked.
8. **Zero network requests during conversion** still holds on the RAW path.
9. `just validate` green; `cargo build --no-default-features` (lean) green.

## Also check the completeness traps this project has been burned by

- Diff the build's Build-Completion "all criteria met" table against the spec's
  Acceptance list — a criterion with no row is presumed NOT met
  ([[a-criterion-nobody-claims-is-a-criterion-nobody-checks]]).
- The Failing Tests were prose in the spec; confirm the build actually wrote tests that
  can FAIL (mutation-test at least the gate boundary) — a green test that exercises
  nothing is not evidence ([[a-plausible-test-result-is-not-a-checked-one]]).

## When done

- Give a clear VERDICT: CLEAN or NOT-CLEAN, with each finding as (real / severity /
  evidence). If you fix small defects, keep the fixes minimal and DCO-signed; note what
  you changed. If a finding needs the architect, state it and stop.
- If CLEAN: `just advance-cycle SPEC-103 ship`, mark verify `[x]` in the timeline, and
  report back to the orchestrator with the verdict + the real cost numbers for this
  session. Do NOT merge — the orchestrator merges on maintainer go-ahead.
- Your final message is a report to the orchestrator: the verdict, the findings with
  evidence, what you re-drove by hand vs. read, and the cost.
