---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-081
  type: decision
  confidence: 0.85
  audience:
    - developer
    - agent
    - operator

agent:
  id: claude-sonnet-5
  session_id: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-22
supersedes: DEC-020
superseded_by: null

affected_scope:
  - Cargo.toml
  - dist-workspace.toml
  - .github/workflows/ci.yml
  - justfile

tags:
  - avif
  - codec
  - ravif
  - feature-gate
  - distribution
  - modern-formats
---

# DEC-081: AVIF encode moves into the default feature set (supersedes DEC-020)

## Decision

`avif` (AVIF output via `image/avif` → `ravif` → `rav1e`) is now a **default Cargo
feature** (`default = ["display", "watch", "avif"]`). Every distributed channel —
Homebrew, the Releases-page binaries, the shell/PowerShell installers, and a plain
`cargo install crustyimg` — now ships AVIF encode with no extra flag. This
**supersedes DEC-020**, which kept `avif` off by default for compile time, binary
size, and encode speed.

`--no-default-features` (the lean/headless build) still drops it: `convert
--format avif` there exits 4 with a "rebuild with `--features avif`" hint
(DEC-004), same as today. `heic` **remains off by default** and
`dist-workspace.toml` still carries **no** `features`/`all-features` key — this
decision changes nothing about DEC-052's guard (see Consequences).

## Context

DEC-020 (2026-06-16) gated AVIF because it was one candidate format among
several, and `rav1e` was measured as a real compile-time/binary-size/speed cost
for a feature most invocations wouldn't touch. That calculus has changed:

- **SPEC-084/DEC-069** made fixed-quality AVIF the **default `optimize`/`web`
  decision** (`Mode::Fast` admits AVIF as a bucket predicate, one encode, no
  search) — AVIF is no longer an opt-in extra, it is what the default command
  now reaches for on lossy-family content.
- **SPEC-083** benchmarked crustyimg's AVIF path as the headline number against
  sharp/ImageMagick/squoosh/cwebp, and had to caveat every result with "built
  `--features avif`, off in the default distributed binary" — the flagship
  path the benchmark measures is invisible to anyone who installs the normal
  way.
- **The wasm artifact has shipped the AVIF encoder unconditionally since
  SPEC-073/DEC-065** — "PNG → AVIF in the browser" is the client-side demo's
  headline, so the project has already accepted `rav1e`'s cost on the size-
  sensitive wasm target. The native default being the one build *without* it
  had become the odd one out, not the norm.

The gap this closes: `BENCHMARKS.md` measures `crustyimg web` producing AVIF,
but a `brew install jysf/tap/crustyimg` or plain `cargo install crustyimg` user
got a binary where `web`/`optimize` silently fall back to non-AVIF output and
an explicit `--format avif` exits 4.

## The measured costs (DEC-020 gated this for real reasons — reporting plainly)

Measured on the release profile (`[profile.release]`, thin-LTO via `Cargo.toml`'s
own release settings; Apple M4 Pro, macOS, clean `cargo clean --release` +
`cargo build --release` each time, Homebrew rustc 1.94.1):

| | before (no avif in default) | after (avif in default) | delta |
|---|---|---|---|
| **Binary size** | 12,841,632 B (12.2 MiB) | 15,720,304 B (15.0 MiB) | **+2,878,672 B (+22.4%)** |
| **Clean release compile** | 24.77 s real (227.93 s user) | 30.48 s real (274.55 s user) | **+5.71 s real (+23%)** |

This is the DEC-020 cost, realized: every distributed binary is ~2.7 MiB larger
and every from-source build (dev machine or CI's `check` job) is ~5-6 s slower.
Accepted deliberately — see Alternatives Considered.

**MSRV floor is unchanged.** `re_rav1d`/`avif-parse` (the AVIF *decode* path,
native-only) already floor the declared `rust-version = "1.90.0"`
(`avif-parse` 2.1.0) and were **already unconditional** on native default before
this spec — they are not behind the `avif` feature. The open question this
spec had to check (not assume) was whether `ravif`/`rav1e` (the *encode* path,
newly default) float that floor any higher: verified via
`rustup run 1.90.0 cargo build --features avif` — **exit 0, clean build**. The
declared MSRV does not move.

**Byte parity confirmed.** `convert --format avif -q 80` on an identical input,
compared between a binary built from the pre-spec `Cargo.toml` with
`--features avif` explicitly and a binary built from this spec's `Cargo.toml`
with plain defaults: **SHA-256 identical**. Turning the feature on by default
does not touch the encoder — this is a distribution change, not a behavior
change to the codec itself.

## Alternatives Considered

- **Keep DEC-020's gate; only make the RELEASE/dist config default to
  `--features avif`.** Rejected — and explicitly warned against in the spec:
  `dist-workspace.toml` deliberately carries no `features` key (DEC-052's
  guard against `heic` ever reaching a distributed binary). Adding one there
  would (a) still leave a plain `cargo install crustyimg` without AVIF, so
  brew/release users and `cargo install` users would get different binaries,
  and (b) erode the exact guard DEC-052 depends on — the next person who
  wants a different feature in "just the dist build" has a precedent to point
  at. The one honest lever is `Cargo.toml`'s own `default` list, which is
  build-tool-agnostic.
- **Wait for a `--speed` knob / AVIF decode before defaulting encode.**
  Rejected: those are DEC-020's own deferred v1 scope-outs, unrelated to
  whether the existing output-only encoder should be on by default. SPEC-084
  already made it the default *decision*; not making it the default *feature*
  was the remaining inconsistency.
- **Leave it opt-in and just fix the docs to stop overclaiming.** Rejected:
  that keeps the flagship path — the one SPEC-083 benchmarks and SPEC-084
  optimizes for — absent from every binary a normal install produces. Honest
  docs about an opt-in feature don't close the gap; only shipping it does.

## Consequences

**Good**

- Every distributed channel (Homebrew, Releases, shell/PowerShell installers,
  `cargo install crustyimg`) now produces a binary where `web`/`optimize` can
  actually pick AVIF — matching what `BENCHMARKS.md` measures and what
  SPEC-084's default decision already assumes.
- Native and wasm converge: both now ship AVIF encode unconditionally (the
  wasm build already did, via DEC-065).
- `heic` is untouched — **still** an off-by-default `heic` feature, **still**
  never in a distributed binary. `dist-workspace.toml` **still** carries no
  `features`/`all-features` key. DEC-052's two independent blockers (the AGPL
  wall and the HEVC patent pool) are orthogonal to AVIF's royalty-free,
  pure-Rust codec and are not affected by this decision in any way — a reader
  should not infer "AVIF went default, so HEIC might too." It won't, absent a
  separate legal clearance (DEC-052's own revisit condition).

**Bad / risky**

- **Every distributed binary is ~2.7 MiB larger and every default build is
  ~5-6 s slower to compile from source**, on this machine's clean-build
  measurement (see table above). This is the literal cost DEC-020 was
  avoiding; SPEC-084/DEC-069 already having made AVIF the default *decision*
  is judged to outweigh it now, but it is a real, ongoing cost of every
  future `cargo build`/CI run on the default feature set, not a one-time
  price.
- **This is a behavior change for existing users, not just a build change.**
  With `avif` compiled in, `Mode::Fast` can admit AVIF as a candidate for
  lossy-family content, so `web` and `optimize` on an upgraded binary can
  produce a **different output file** (a smaller AVIF instead of the previous
  best non-AVIF candidate) than the same command produced before. This must
  be — and is — headlined in `CHANGELOG.md` under `[0.6.0]`, not buried as a
  build-only note.
- **AVIF encode speed** (AV1 is slow at high quality) is now something every
  default-build user pays for on lossy content, not just users who opted in.
  `Mode::Fast`'s single fixed-quality encode (DEC-069, `FAST_LOSSY_QUALITY =
  85`) keeps this bounded — no repeated-encode search runs by default — but
  it is still a slower codec than JPEG/lossless-WebP per encode.

**Neutral**

- The CI `avif` feature job (`.github/workflows/ci.yml`) becomes largely
  redundant with the default `check` job (both now build/test/lint with AVIF
  on) — kept as an explicit belt-and-suspenders pin on `--features avif`
  alone, not removed. The `lean` job (`--no-default-features`) is the
  meaningfully differentiated one now, and is unchanged.
- The wasm build's `_wasm_features` (justfile) is pinned to
  `--no-default-features --features avif` explicitly rather than relying on
  cargo's `default` list, specifically so this native-facing decision cannot
  silently change what "lean" means for the wasm size-comparison recipe
  (SPEC-074's no-AVIF baseline). Without that pin, `just --set _wasm_features
  "" wasm-build` would have quietly stopped being a lean/no-AVIF build the
  moment `avif` joined `default`.

## Validation

Right if: `cargo build --release` with **no feature flags** produces a binary
whose `convert --format avif -o out.avif` exits 0 and writes a container that an
independent decoder (`sips`) confirms is AVIF; `dist-workspace.toml` still has
no `features`/`all-features` key and a default build still refuses `.heic` with
the typed exit-4 error; `--no-default-features` still builds and tests green;
`cargo deny check licenses` stays green with only the existing scoped
`libfuzzer-sys` exception (unchanged surface — same tree as any
`--features avif` build before this spec); the MSRV job (pinned to 1.90.0)
still passes. Revisit if: a future measurement shows the compile-time or
binary-size cost has become a real problem for a target environment (then
consider `opt-level`/codegen-units tuning before re-gating, per this
decision's own measured-cost table); or if `rav1e`'s upstream MSRV climbs
above the declared floor (re-run the 1.90.0 check named above).

**Honest limit:** this decision is verified against a **local** default-feature
build. The *prebuilt* Homebrew/Releases artifact can only be fully confirmed by
cutting the 0.6.0 tag — an irreversible act out of this spec's scope. Treat the
distributed-artifact confirmation as a **post-tag** check, not something this
decision already proved.

## References

- Supersedes: DEC-020 (AVIF output via a feature-gated `ravif` codec, off by
  default)
- Related specs: SPEC-102 (this), SPEC-018 (DEC-020's spec — added AVIF behind
  the feature), SPEC-084/DEC-069 (fixed-quality AVIF as the default `optimize`
  decision), SPEC-083 (benchmarked AVIF as the headline, surfaced this gap),
  SPEC-073/DEC-065 (wasm ships AVIF encode unconditionally)
- Related decisions: DEC-052 (why `dist-workspace.toml` has no `features` key —
  untouched by this decision), DEC-004 (codec policy — pure-Rust default,
  native codecs feature-gated; `avif` stays pure-Rust either way), DEC-027 (the
  direct precedent — `display` moved from off-by-default (DEC-011) to
  on-by-default for the same class of reason: a feature the shipped product
  needs by default shouldn't need a flag)
- Related constraints: `pure-rust-codecs-default` (unaffected — `ravif`/`rav1e`
  are pure Rust either way)
