# SPEC-102 timeline

Architect appends as cycles are designed. Executors update status as
they go. Status markers: `[ ]` not started · `[~]` in progress · `[x]` complete · `[?]` blocked.

Cycle prompts live in `prompts/SPEC-102-<cycle>.md`.

## Instructions
- [x] design — framed build-ready 2026-07-22. **The gap SPEC-083 surfaced: the distributed binary has
  no AVIF.** cargo-dist builds DEFAULT features only, `avif` is off by default (SPEC-018/DEC-020), so
  Homebrew, the Releases binaries and plain `cargo install crustyimg` all lack the flagship path —
  BENCHMARKS.md itself has to tell readers to `cargo install crustyimg --features avif`. **Fix is ONE
  LINE: `default = ["display","watch","avif"]` in `Cargo.toml` — NOT a `features` key in
  `dist-workspace.toml`**, which would miss `cargo install` (brew and cargo users would get different
  binaries) and erode the DEC-052 guard keeping `heic` out of distributed builds. **It is a BEHAVIOR
  change** — `Mode::Fast` can then admit AVIF, so `web`/`optimize` produce different output files for
  existing users → emits **DEC-081** (reversing DEC-020's rationale) + a headline CHANGELOG entry.
  Must MEASURE rather than assume: binary-size delta, clean compile-time delta, and that the **MSRV job
  still passes** (rav1e may floor above the declared rust-version — a finding, not a quiet bump); lean
  `--no-default-features` build + cargo-deny stay green. Docs sweep is **mechanical — cite the grep and
  hit count** (README feature table, cli-reference, BENCHMARKS' install line, the Cargo.toml comment,
  the dist-config note). Honest limit: the *prebuilt* artifact can only be fully proven by cutting an
  irreversible tag, so verify everything provable pre-tag and mark the released-binary check as
  post-tag. Sonnet build / Opus verify. Complexity S.
- [x] build — `Cargo.toml` `default = ["display", "watch", "avif"]` + both AVIF comment blocks rewritten;
  `DEC-081` written (supersedes `DEC-020`, which got `superseded_by: DEC-081`). **Measured:** binary
  +2,878,672 B (+22.4%), clean release compile +5.71 s (+23%), MSRV (1.90.0) still passes, byte-parity
  confirmed (SHA-256 identical `-q80` AVIF vs a pre-spec `--features avif` build). Docs sweep by grep:
  61→59 hits after fixing a genuinely missed `README.md` example line. **Found + fixed one thing the
  spec didn't call out:** `justfile`'s `_wasm_features` relied on cargo's `default` list being AVIF-less
  to make the SPEC-074 lean/no-AVIF wasm comparison lean — pinned it to `--no-default-features --features
  avif` explicitly so this native-facing change couldn't silently break it. **Deviated** from the spec's
  suggested "remove the test's `#[cfg(feature = "avif")]` gate" — that would break the lean CI job;
  proved the same behavior change via the same test's pass/fail transition across commits instead (0
  tests filtered on parent → 1 passed here). All gates green: `cargo test` default (783) + lean (763),
  clippy/fmt/deny clean, `dist-workspace.toml` confirmed still feature-free. Full detail in the spec's
  Build Completion section. → verify.
- [x] build — Sonnet, 2026-07-22. The one-line `default = ["display","watch","avif"]` flip, DEC-081
  (superseding DEC-020), a CHANGELOG headline, and a docs sweep. **Measured, not assumed:** release
  binary +2,878,672 B (+22.4%), clean compile +5.71s, MSRV 1.90.0 unaffected, AVIF output SHA-256
  identical to a pre-spec `--features avif` build. **Caught beyond the spec's list:** the justfile's
  `_wasm_features` implicitly relied on cargo's defaults being AVIF-less to keep SPEC-074's lean wasm
  comparison honest — this spec would have silently invalidated it. ~$3.25.
- [x] verify — Opus, 2026-07-23. **⚠ NOT-CLEAN (4 findings).** Re-measured the size delta independently
  and reproduced it BIT-EXACTLY twice, plus the dist profile that actually ships (+22.49%); ran the real
  CI msrv legs; DROVE the DEC-052 heic guard rather than reading it. **The real miss: the cited sweep's
  `--include` filter was structurally blind to `.py`/`.mjs`/`.yaml`/`justfile`, so a live "AVIF is
  opt-in / `--features avif`" overclaim survived in `scripts/bench-compare.py` — inside the harness that
  produces BENCHMARKS.md's numbers.** Plus a DEC-081 profile mislabel, a justfile override that didn't
  parse, and two load-flaky `web` tests. Caught and redid its own false-positive byte-parity result.
  ~$8.
- [x] build (fix) — Sonnet, 2026-07-23. Cleared all four. Re-ran the sweep with the missing file types
  and **independently re-triaged every hit rather than trusting the prior pass**, which found a further
  miss (`examples/gen_avif_fixture.rs`'s "no-op without --features avif" doc comment, false once avif
  defaults on) — verified by deleting and regenerating the fixtures with a bare `cargo run`. Rewrote the
  flaky tests to assert AVIF's ADMISSION via the `--json` explain trace instead of a byte race. ~$2.40.
- [x] verify (re-verify) — Opus, 2026-07-23. ✅ **CLEAN.** Re-derived the sweep independently rather than
  confirming it. **Proved the rewritten tests NON-VACUOUS by mutating the admission site while keeping
  the feature on** — both failed on exactly the admission assertions. Flagged a newly-reachable latent
  trap (a nested `just wasm-size` doesn't inherit `--set`, so a lean build mislabels its own banner) and
  that the rtk hook silently corrupted `rg` counts twice. ~$3.50.
- [x] build (fix 2) — Sonnet, 2026-07-24. **CI went red on the `webp-lossy` leg** — a feature
  combination no local gate covered, because making avif DEFAULT meant a test began running where a
  lossy-WebP encoder also competes. **Diagnosed before fixing, and it was the test, not the engine:**
  `--json` showed source 3621 B → avif 525 B, **webp 372 B**, jpeg 3861 B, so lossy WebP genuinely
  out-encodes fixed-quality AVIF on that tiny synthetic gradient. `pick_winner`'s contract is "smallest
  admitted candidate that beats the source", never "AVIF always wins". Rewritten to assert admission +
  a modern-format winner; **no engine code touched.** Ran the full local feature matrix (default/lean/
  avif/webp-lossy/heic/avif+webp-lossy) — the gap that let this reach CI. ~$1.75.
- [x] ship — squash-merged PR #110 (**afd9f4e**) 2026-07-24, CI CLEAN across the full matrix.
  `default = ["display","watch","avif"]` is on `main`; `dist-workspace.toml` still carries **zero**
  features keys, so DEC-052's heic guard is intact. Orchestrator verified the final test rewrite's
  non-vacuity **inline** rather than dispatching a session — mutating `avif: cfg!(feature="avif")` to
  false made it fail on the right assertion with the candidate list collapsed to JPEG-only.
  **~$19.4 / 6 sessions.** ⚠ **Nothing has reached a user yet — the released binaries and the Homebrew
  formula are still 0.5.0 (AVIF-less). Cutting 0.6.0 is the maintainer's irreversible tag push, and
  confirming AVIF in the prebuilt artifact is a POST-TAG check.**
