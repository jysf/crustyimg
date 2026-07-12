---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-064
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

project:
  id: PROJ-008
repo:
  id: crustyimg

created_at: 2026-07-12
supersedes: null
superseded_by: null

affected_scope:
  - Cargo.toml
  - .cargo/config.toml
  - src/lib.rs
  - src/main.rs
  - src/wasm.rs
  - src/image/mod.rs
  - src/image/avif.rs
  - src/image/sniff.rs
  - src/error.rs
  - src/sink/mod.rs
  - tests/wasm_roundtrip.rs
  - justfile

tags:
  - wasm
  - build-targets
  - dependencies
  - avif
  - pure-rust-codecs-default
---

# DEC-064: the WASM boundary is `cfg(target_arch)`, not a cargo feature — and AVIF decode is out

## Decision

crustyimg compiles its **pure engine** to `wasm32-unknown-unknown` as a `cdylib`
with a thin `wasm-bindgen` surface (`src/wasm.rs`). Three choices, together:

1. **The native/wasm boundary is the TARGET, not a feature.** Modules split with
   `#[cfg(not(target_arch = "wasm32"))]` / `#[cfg(target_arch = "wasm32")]`, and
   dependencies split into `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]`
   and `[target.'cfg(target_arch = "wasm32")'.dependencies]`. There is **no `wasm`
   cargo feature**.

2. **`wasm-bindgen` is the JS boundary** (pinned `=0.2.126`, wasm-target-only), with
   `wasm-bindgen-test` as a wasm-only dev-dep.

3. **AVIF DECODE is not in the wasm build.** `re_rav1d` cannot compile to bare
   wasm32, so it (and `avif-parse`) are native-only deps, and an AVIF input to the
   wasm surface returns a typed `ImageError::CodecUnavailableOnTarget` — never a
   panic. Every other default input format survives, **including SVG**.

## Context

PROJ-008 (Wave 3) compiles the already-I/O-agnostic core to wasm so the demo page
runs client-side with no backend. A design-time probe (2026-07-12) compiled the whole
dependency tree to wasm32 and found exactly one blocker, `re_rav1d`: it imports libc
POSIX types bare wasm32 does not have (`off_t`, `ptrdiff_t`/`intptr_t`/`uintptr_t`,
errno `ENOENT`/`EIO`/`EINVAL`) and its thread-task module fails const-eval (`E0080`).
`resvg`/`usvg`/`tiny-skia`, `image`, `fast_image_resize`, `ssimulacra2`, `skrifa`,
`zeno`, `rayon`, `img-parts`, `kamadak-exif`, `toml`, `serde` all compiled.

The architecture had already earned this: the transform core (`operation`, `pipeline`,
`recipe`, `analysis`, `quality`, `text`, `metadata`) has **zero** imports of the
filesystem/CLI shell, and `sink::encode_to_bytes` is a pure bytes-in/bytes-out
function (DEC-053/DEC-054 anticipated it). So the cut is along a seam that already
existed — this decision records where the knife goes, not a new architecture.

## Rationale

### Why the target, not a `wasm` feature

A feature would have to be *remembered*: every wasm build would need `--features wasm
--no-default-features`, and forgetting it yields a confusing link error instead of a
clear one. `cfg(target_arch)` is **automatic and unforgeable** — `--target wasm32-…`
selects the right module set and the right dependency set with no flags at all.

Decisively: it also means **the native feature matrix does not move**. The
native-only dependency table is unconditionally active on every native target, exactly
as `[dependencies]` was, so `default` / `--no-default-features` / `avif` / `heic` /
`webp-lossy` resolve to byte-identical dependency graphs. Had we made these deps
optional behind a new feature instead, every existing native combination would have
become a new, untested combination. The wasm build is a **new target**, not a new
configuration of the old one, and the manifest should say so.

The one place this leaks is a default-ON feature whose dep is native-only: on wasm,
`display` is enabled while `viuer` is absent. Handled by gating the *code* on
`all(feature = "display", not(target_arch = "wasm32"))` — two `cfg`s in `sink`. That
leak is small, local, and preferable to reshaping the native feature set.

### Why AVIF decode is dropped rather than fought

Options were: (a) drop it, (b) patch/fork `re_rav1d` for wasm, (c) find another
decoder, (d) block the wave on it. We chose (a) **for this spec only**, because it is
the one that lets the rest be *measured*: the wasm build ships now with SVG + raster
conversion — a real, demoable capability — and the AVIF-on-wasm strategy gets its own
spec (SPEC-073) with real numbers to argue from instead of a guess. Dropping it is
reversible; blocking the wave on a codec port is not free.

What makes the drop survivable is that **SVG rasterization survives** — resvg compiles
to wasm cleanly. Vector→raster in the browser with no backend is arguably the more
striking demo than AVIF decode anyway.

### Why a NEW error variant, gated to wasm

`CodecNotBuilt` (DEC-056) exists for exactly this shape — but its message is "rebuild
with `--features X`", which is **a lie in a browser**: no feature flag brings AVIF back
on wasm32, and the user cannot rebuild anything. So
`ImageError::CodecUnavailableOnTarget` says the true thing (convert the file first).

The variant is itself `#[cfg(target_arch = "wasm32")]`. That is deliberate: SPEC-061
and SPEC-062 both taught that a new `ImageError` variant forces an audit of every
decode caller and every exit-code `match` (`IMAGE_EXTENSIONS exposes every decode
caller`). Gating the variant to the target that can construct it means the native
exit-code map — which must stay total — does not change at all.

A panic would have been the lazy alternative, and is the one genuinely unacceptable
option: **a panic in wasm aborts the module** and takes the page's instance with it.
`untrusted-input-hardening` matters *more* in a browser, not less.

## Consequences

**Good**

- The wasm build runs the REAL engine: same `Recipe` TOML, same `OperationRegistry`,
  same `Pipeline`, same `encode_to_bytes`. A recipe tuned in the terminal replays in
  the browser because it is the same code, not a port.
- Native builds are untouched — same deps, same features, same behavior. Verified:
  full suite (714 tests), lean build, clippy, and native AVIF decode all green.
- `just deny` needs **no new license exception**: the `wasm-bindgen` tail is
  MIT/Apache-2.0, so `pure-rust-codecs-default` (DEC-004) holds unchanged.
- The size baseline is on the record for SPEC-074: **4.29 MB raw / 1.64 MB gzip /
  1.19 MB brotli** after `wasm-opt`.

**Costs / risks**

- **No AVIF decode in the browser** until SPEC-073 decides otherwise. An `.avif` drop
  on the demo page will show a clear error, not a picture. This is the wave's most
  visible gap and it is deliberate.
- **The bundle is big** (1.19 MB brotli). The whole codec set — including the SVG text
  stack and SSIMULACRA2 — is linked in eagerly. SPEC-074 owns this.
- **A default-ON feature with a native-only dep needs a `not(wasm32)` conjunct in the
  code.** Today that is only `display`. A future default-ON feature whose dep is
  native-only must remember the same gate, or the wasm build breaks. This is the
  sharpest edge the decision leaves behind.
- **`crate-type` now includes `cdylib`**, so native builds also emit an unused dynamic
  library next to the rlib. Harmless, but it is new output in `target/`.
- **The wasm tests need `wasm-bindgen-test-runner`** (from `wasm-bindgen-cli`, version-
  matched to `wasm-bindgen`) rather than `wasm-pack test` — see `.cargo/config.toml`
  for why (`wasm-pack test` hardcodes `--tests`, which drags all ~20 native
  CLI-driving integration tests into the wasm build).

## Alternatives considered

| Option | Why not |
|---|---|
| A `wasm` cargo feature | Must be remembered; and it would turn every existing native feature combination into a new untested one. The target already carries the information. |
| Extract the engine into a separate `crustyimg-core` crate | The cleanest long-term shape, and probably right eventually. But it is a large mechanical refactor whose payoff (a smaller wasm dep graph) is exactly what SPEC-074 will measure — do it when there's a number arguing for it, not before. Filed as a follow-up. |
| Patch/fork `re_rav1d` for wasm32 | A codec port is a project, not a spec. SPEC-073 decides with the baseline in hand. |
| Panic / `unimplemented!()` on AVIF in wasm | Aborts the wasm module and kills the page's instance. Violates `untrusted-input-hardening`. |
| Reuse `CodecNotBuilt` for the wasm AVIF case | Its message advises a `--features` rebuild — advice a browser user cannot act on and that is not true on this target. |
| Build wasm with `--no-default-features` to dodge the `display`/`viuer` leak | Makes the plain `cargo build --target wasm32-…` fail confusingly, and silently drops `watch`/`display` semantics from the story. Two `cfg` conjuncts are cheaper and more honest. |

## Revisit when

- **SPEC-073** decides the AVIF-on-wasm path (rav1e encode feasibility; whether
  `re_rav1d` decode can be restored). If it can, the `CodecUnavailableOnTarget` arm in
  `image/mod.rs` and the sniff split in `image/sniff.rs` are the two places to undo.
- **SPEC-074** attacks the bundle size. If the answer is "split the engine out of the
  CLI crate", that supersedes the module-level `cfg` partition here with a crate
  boundary — a strictly better shape that this decision deliberately defers.
- A **second default-ON feature with a native-only dep** appears. Two is a pattern; at
  that point consider a `native_only!` cfg alias instead of open-coding the conjunct.
