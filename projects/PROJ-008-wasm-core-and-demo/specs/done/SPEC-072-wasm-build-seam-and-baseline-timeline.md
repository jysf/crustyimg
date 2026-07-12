# SPEC-072 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

## Instructions

- [x] **design** (2026-07-12, orchestrator main loop) — framed build-ready. Grounded in the
  2026-07-12 design-time WASM compile probe (only `re_rav1d` blocks wasm32; the transform core
  is shell-free). Acceptance criteria + failing tests (`tests/wasm_roundtrip.rs`, `#[wasm_bindgen_test]`)
  specified; gating strategy = `cfg(target_arch = "wasm32")` + target-scoped dep tables.
- [x] **build** (2026-07-12, PR #80, ~400k tok est.) — **7/7 wasm round-trip tests green in Node** on the
  real `.wasm` (PNG+resize → bytes decoding to 32×24; SVG rasterizes; AVIF errors cleanly, no panic).
  Native unaffected: 714 tests + lean build + clippy + native AVIF decode all green; `just deny` needed
  **no new exception**. Emitted **DEC-064**. Size baseline **4.29 MB raw / 1.64 MB gzip / 1.19 MB brotli**
  → `docs/research/proj-008-wasm-build.md`. NOTE: the round-trip runs via
  `cargo test --target wasm32 --test wasm_roundtrip` + a `.cargo/config.toml` runner, **not**
  `wasm-pack test` — the latter hardcodes `--tests` and drags all ~20 CLI-driving native integration
  tests into the wasm build. Also carries one **out-of-scope** `chore(deny)` commit (RUSTSEC-2026-0206:
  `main` was already red before this branch — verified on a clean worktree).
- [x] **verify** (2026-07-12, fresh adversarial session) — ✅ **CLEAN / ready to ship.** Every claim
  reproduced on the real artifact, none taken on the build's word.
  - **Round-trip re-driven:** `just wasm-test` → **7/7 green** in a real wasm VM (Node). The tests
    assert on real output BYTES, not on `Ok`: PNG+resize → bytes fed back through `info` decode to
    **32×24**; SVG → 40×30 raster → resized 32×24; AVIF → typed `Err` naming AVIF with **no**
    `--features` advice; no panic/abort.
  - **Native unaffected (the historically-missed check):** `cargo build` ✅, `cargo build
    --no-default-features` (LEAN, run explicitly) ✅, `cargo clippy -D warnings` ✅ clean,
    `cargo test` **714 green**. Native AVIF decode driven on the **real binary**:
    `crustyimg info tests/fixtures/avif/solid_16x16.avif` → 16×16, `format: avif`, exit 0.
  - **No native tests silently lost:** main = 713 passed / 28 suites, branch = 714 / 29. The delta is
    exactly the new `wasm_roundtrip` suite's 1 native test; the 3 `is_avif` unit tests moved intact
    `avif.rs` → `sniff.rs`. (Guards the "verify test existence, not just gate count" trap.)
  - **`just deny` green** — advisories/bans/licenses/sources all ok. Verified **non-vacuously**:
    `[graph]` has `all-features = true` and **no target filter**, and `cargo deny list` shows
    `wasm-bindgen@0.2.126` + its whole tail (`-macro`/`-macro-support`/`-shared`, `js-sys`) under
    MIT **and** Apache-2.0. The wasm dep tail really is scanned; "no new exception" is real.
  - **Size baseline reproduced exactly:** `just wasm-build` → **4.29 MB raw / 1.64 MB gzip /
    1.19 MB brotli**, matching `docs/research/proj-008-wasm-build.md` to the digit. Stable toolchain,
    no nightly; the Homebrew-rustc gotcha is genuinely handled by the recipes' pinned `RUSTC`.
  - **The 3 flagged items all hold up.** (a) The out-of-scope `chore(deny)` RUSTSEC-2026-0206 commit is
    a **genuine pre-existing-main-red fix** — confirmed by running `cargo deny check` on a clean `main`
    worktree: `advisories FAILED`, `rustybuzz` unmaintained, same usvg/resvg text stack as the
    already-ignored -0192. It masks nothing SPEC-072 introduced. (b) The wasm-pack-test divergence does
    **not** weaken anything: still a real wasm VM, still byte-asserting, and the `.cargo/config.toml`
    runner is target-scoped (`[target.wasm32-unknown-unknown]`) so the native run is untouched —
    proven by the 714-green native suite. (c) `optimize`'s partiality is **honest**: it never claims to
    pick the best, and the "takes the shortlist's first candidate, does not comparison-shop encodings"
    caveat propagates into the **generated `pkg/crustyimg.d.ts`**, so a JS consumer reads it too.
  - **Adversarial probes the build tests did not cover** (written fresh this cycle, run in the wasm VM,
    then removed): empty input, 4 KB of garbage bytes, 5 truncation points of a PNG, a **hostile
    oversized PNG** (IHDR rewritten to 100 000 × 100 000 with a fixed CRC), malformed/huge/external-href
    SVG, 5 bogus recipe TOMLs, 6 bogus `out_format` strings (incl. `avif`, `heic`, `../../etc/passwd`,
    an embedded NUL), and a garbage-payload AVIF. **Every one returned a typed `Err` — zero panics,
    zero module aborts.** Critically, the **DEC-034/063 decode cap DID carry into wasm**: the
    100 000² PNG is refused at decode with the dimension-limit error, not allocated.
  - Only truncation quirk found: a PNG missing its final byte still decodes — but **native does exactly
    the same** (verified with a matching native probe), so it is pre-existing `image`-crate leniency,
    not a wasm regression or a SPEC-072 defect.
  - **Fixed on branch (1 cosmetic):** the module docstring of `tests/wasm_roundtrip.rs` still said the
    tests run under `wasm-pack test --node` — the one thing deviation #4 replaced. Corrected to name the
    real command + why. (`.cargo/config.toml` and the research doc were already right.)
  - **Carry to ship (not a blocker):** the 4 follow-ups live only in the spec's `## Build Completion`.
    Prior specs (SPEC-061/062) carried theirs into `docs/roadmap.md`; ship should do the same for the
    shared-`optimize`-engine seam, the wasm CI job, the SPEC-074 size work, and the
    "default-ON feature + native-only dep needs a `not(wasm32)` conjunct" cfg-alias note.
- [x] **ship** (2026-07-12) — **SHIPPED.** Out-of-scope deny fix split to **PR #81** (`0c6506a`,
  merged first → main green independently); SPEC-072 rebased (dropped the dup deny commit),
  DCO fixed via `git rebase --signoff` (the 2 verify commits lacked `-s`), squash-merged as
  **PR #80** (`c3813a5`). Cost totals filled (520k tok / $4.70, labelled estimates), ship
  reflection appended, 4 follow-ups → `docs/roadmap.md`, spec archived. `just validate` +
  `just cost-audit` green. DEC-064 on main.
