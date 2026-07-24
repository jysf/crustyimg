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
