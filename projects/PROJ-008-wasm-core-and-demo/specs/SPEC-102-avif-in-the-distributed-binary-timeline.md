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
