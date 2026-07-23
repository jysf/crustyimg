# Design-time probe: can the browser demo open RAW files?

Repo: `/Users/jyashinsky/PSeven/experiments/crustimg_redo_plus/crustyimg` @ `b501417` (main, clean).
Nothing in the primary checkout was committed, branched, or modified — confirmed with
`git status`/`git diff --stat` at the end of the session (both clean). All building and
patching happened in a detached-HEAD `git worktree` under the session scratchpad
(`raw-probe-worktree/`), removed after this probe. Probe artifacts (extracted preview
PNGs, thumbnails) are in `raw-probe-out/` next to this file.

**Symptom reproduced by code reading, not by driving the live demo** (the live demo /
`spec-101-demo-pass` branch was left untouched per the constraints). The mechanism found
below fully explains the reported symptom ("could not decode image: The image format Tiff
is not supported") without needing to reproduce it interactively.

---

## Q1 — Confirm the mechanism

**(a) RAW routing is extension-based.** `src/image/raw.rs:88-97`:

```rust
pub(crate) fn is_raw_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| {
            RAW_EXTENSIONS
                .iter()
                .any(|&raw| e.eq_ignore_ascii_case(raw))
        })
        .unwrap_or(false)
}
```

`RAW_EXTENSIONS` (`raw.rs:61-73`) lists `nef nrw cr2 cr3 arw srf sr2 dng raf rw2 orf pef
srw rwl raw`. The doc comment on `is_raw_extension` (`raw.rs:84-87`) states the reason
plainly: "Detection is by **extension**, not content: TIFF-based RAW starts with the TIFF
magic (`II*\0`/`MM\0*`), byte-indistinguishable from a plain `.tif`, so a content sniff
would risk mis-routing legitimate TIFFs."

**(b) `from_bytes` never consults an extension.** `src/image/mod.rs:119-137`:

```rust
/// Decode already-read file `bytes` using the path's EXTENSION to route
/// format detection — the single place the RAW-vs-generic decision lives.
/// ...
/// RAW-via-stdin (`from_bytes`) is a v1 non-goal.
pub fn decode_path(path: impl AsRef<Path>, bytes: &[u8]) -> Result<Image> {
    if raw::is_raw_extension(path.as_ref()) {
        return raw_preview(bytes);
    }
    Image::from_bytes(bytes)
}

/// Detect the format of an in-memory byte slice, decode it, and capture the
/// raw metadata bundle.
pub fn from_bytes(bytes: &[u8]) -> Result<Image> {
    let (pixels, source_format) = decode_with_format(bytes)?;
    ...
}
```

`from_bytes` takes only `&[u8]` — there is no `Path` parameter anywhere in its signature
or in anything it calls. The doc comment on `decode_path` says outright: "RAW-via-stdin
(`from_bytes`) is a v1 non-goal." This was a **documented, deliberate** scope line, not an
oversight.

**(c) Where a DNG falls through to the TIFF decoder.** Every wasm entry point
(`src/wasm.rs`: `info`, `transform`, `optimize`, `optimize_detailed`, `score`) calls
`Image::from_bytes(input)` — never `decode_path`, `Image::load`, or `raw_preview`. Inside
`from_bytes` → `decode_with_format` → `decode_with_limits` (`mod.rs:349-434`), the dispatch
order is: AVIF sniff → SVG sniff → HEIC sniff → generic `ImageReader::with_guessed_format()`
(`mod.rs:409-412`). A DNG's bytes open with the TIFF magic (`II*\0` little-endian, which is
what Adobe DNG / most camera RAW containers use), so `with_guessed_format()` correctly
detects `ImageFormat::Tiff` and the code falls into the generic path expecting a `tiff`
decoder to be linked. On wasm32 it is not (see Q2) — `image`'s error for a
detected-but-not-compiled-in format is exactly `"The image format Tiff is not supported"`,
matching the reported symptom verbatim.

**Verdict: hypothesis confirmed exactly as stated**, and even more precisely documented
than the hypothesis assumed — `decode_path`'s own doc comment already calls out
`from_bytes`'s RAW blindness as a known, intentional v1 gap.

---

## Q2 — Is `extract_preview` already in the wasm binary?

**No `cfg(target_arch)` gate on `raw.rs`, confirmed.** Read the whole file
(`src/image/raw.rs`, 421 lines) — no `#[cfg]` anywhere. `src/image/mod.rs:35`: `mod raw;`
is unconditional (compare `avif`/`heic`, which route through `cfg(feature/target)` deeper
inside, but the *module* itself is always compiled).

**But it is NOT present in the shipped `.wasm` binary today** — dead-code-eliminated,
because nothing in the wasm-reachable call graph (the five `#[wasm_bindgen]` functions,
all calling only `Image::from_bytes`) ever calls `raw::extract_preview` or
`image::raw_preview`. Built the real artifact (`just wasm-build`, `--features avif`, the
production profile) and searched the compiled binary:

```
$ just wasm-build
...
── .wasm size (features: --features avif) ──
  raw:         5.73 MB  (6009090 B)
  gzip:        1.91 MB  (2006210 B)
  brotli:      1.33 MB  (1395239 B)  ← the number that matters

$ strings -a pkg/crustyimg_bg.wasm | grep -i "decodable embedded\|embedded preview\|raw: no\|raw: embedded"
(no output — none of raw.rs's distinctive error strings are present)

$ strings -a pkg/crustyimg_bg.wasm | grep -i "raw.rs"
/Users/…/.cargo/registry/…/aligned-vec-0.6.4/src/raw.rs
/rust/deps/hashbrown-0.17.1/src/raw.rs
/Users/…/.cargo/registry/…/hashbrown-0.17.1/src/raw.rs
(crustyimg's own src/image/raw.rs does NOT appear)

$ strings -a pkg/crustyimg_bg.wasm | grep -i "svg.rs"
src/image/svg.rs
(svg.rs — reachable from Image::from_bytes — DOES appear, as the control)
```

**Cost of wiring it in: negligible, not free but close to it.** Added one throwaway
additive `#[wasm_bindgen]` export in a scratch worktree copy of `src/wasm.rs` that calls
`crate::image::raw_preview` (nothing else changed), rebuilt through the exact
`just wasm-build` profile (`--target web`, `CARGO_PROFILE_RELEASE_LTO=fat`,
`CODEGEN_UNITS=1`, `STRIP=true`, `--features avif`), and diffed against the same build with
that export removed:

| | raw bytes | brotli bytes |
|---|---|---|
| baseline (no RAW export) | 6,009,090 | 1,395,239 |
| with `raw_preview_probe` wired in | 6,011,445 | 1,396,453 |
| **delta** | **+2,355 B** | **+1,214 B (≈ 0.09%)** |

(Baseline numbers reproduce the primary checkout's own `just wasm-build` output exactly —
good cross-check that the worktree build is representative.)

This is because `raw::extract_preview` has **no new codec dependency** — it only calls the
`image` crate's JPEG decoder via `ImageReader`/`set_format(ImageFormat::Jpeg)`, and JPEG is
already linked into the wasm build for every other format. Contrast the TIFF/BMP/ICO trim
(SPEC-074/DEC-066, `wasm.rs:28-31`), which cost −84,327 B brotli precisely because those
*are* separate decoders. RAW wiring is cheap for the opposite reason those were expensive.

**Could not verify:** whether `wasm-opt` (currently off per DEC-066) would change this delta
meaningfully — not tested, and the project's own recipe doesn't run it, so this mirrors what
ships.

---

## Q3 — Does it actually run on wasm?

**Yes — driven against real files inside a real wasm VM, with independently verified
output.**

### In-VM proof (`wasm-bindgen-test-runner`, the project's own harness)

Added two isolated test binaries (each `include_bytes!`-ing exactly one real RAW file, so
memory baselines aren't polluted by the other fixture's static data) and ran them through
`cargo test --target wasm32-unknown-unknown --release` under the project's pinned
`wasm-bindgen-test-runner` (Node-backed), calling `crustyimg::image::raw_preview(bytes)`
directly:

```
RAW PROBE [dng-leica-47mp]: file_bytes=84872192 preview_w=8368 preview_h=5584
  preview_px=46726912 elapsed_ms=301.0
  wasm_mem_before_bytes=87162880 wasm_mem_after_bytes=319488000 wasm_mem_delta_bytes=232325120

RAW PROBE [raf-fuji]: file_bytes=50268160 preview_w=1920 preview_h=1280
  preview_px=2457600 elapsed_ms=86.0
  wasm_mem_before_bytes=52559872 wasm_mem_after_bytes=110493696 wasm_mem_delta_bytes=57933824
```

Both decodes succeeded — `raw_preview` returned `Ok(Image)` for both real files, inside the
actual wasm32 VM the shipped artifact runs in.

### Independent verification (macOS `sips` + `file` — a decoder I did not write)

Built a second, separate `--target nodejs` artifact with a throwaway `rawPreviewProbe`
export (PNG-encodes the extracted preview) and drove it from a plain Node script reading the
real files off disk (`fs.readFileSync`, not `include_bytes!`), writing the PNG output to
disk:

```
[dng-leica-47mp] input=84872192B output=16951661B elapsed_ms=996 -> raw-probe-out/dng-leica-47mp.png
[raf-fuji]        input=50268160B output=2817548B  elapsed_ms=84  -> raw-probe-out/raf-fuji.png

$ sips -g pixelWidth -g pixelHeight -g format raw-probe-out/dng-leica-47mp.png
  pixelWidth: 8368   pixelHeight: 5584   format: png
$ sips -g pixelWidth -g pixelHeight -g format raw-probe-out/raf-fuji.png
  pixelWidth: 1920   pixelHeight: 1280   format: png

$ file raw-probe-out/*.png
dng-leica-47mp.png: PNG image data, 8368 x 5584, 8-bit/color RGB, non-interlaced
raf-fuji.png:        PNG image data, 1920 x 1280, 8-bit/color RGB, non-interlaced
```

`sips` and `file` (both decoders this project didn't write) independently confirm the exact
dimensions the in-VM test reported, and confirm the files are structurally valid PNGs, not
truncated/garbage. Visually inspected downscaled thumbnails of both (`*-thumb.png` in
`raw-probe-out/`) — the DNG decodes to a correctly-oriented black-and-white portrait photo,
the RAF to a correctly-oriented close-up of a succulent. Both are real, coherent
photographs, not noise.

Note the ~1s (DNG) / ~84ms (RAF) Node-script timings include a JS→wasm byte-copy *and* a
full-resolution PNG re-encode on top of the pure `raw_preview` extraction; they are not
directly comparable to the 301ms/86ms extraction-only numbers above. The demo would not
re-encode to lossless PNG in production (more likely feed the decoded pixels straight into
the existing `optimizeDetailed` pipeline — see Q5), so the Node-script number is a rough
upper bound on total pipeline latency, not the number to design around.

### Whether the `tiff` `image` feature is needed — confirmed NOT needed

`Cargo.toml:227` (the wasm32 target's `image` dependency): `features = ["png", "jpeg",
"gif", "webp"]` — no `tiff`. The wasm build that produced the results above was built with
exactly this feature set (no modification), and both real RAW files decoded successfully
anyway, because `extract_preview`'s inner decode (`raw.rs:206-217`,
`decode_jpeg_with_limits`) always calls `peek.set_format(ImageFormat::Jpeg)` /
`reader.set_format(ImageFormat::Jpeg)` explicitly — it never asks the `image` crate to
guess or decode TIFF. The RAW *container* is only ever byte-scanned, never handed to a TIFF
decoder.

**Could not verify:** CR3 (Canon, ISOBMFF-based) and any file whose embedded preview isn't a
baseline-JPEG-compatible stream — no CR3 sample was available in the local corpus or home
directory search. DEC-055 states the same scan mechanism covers CR3 by design (ISOBMFF
containers also embed a plain JPEG stream the scan finds the same way), but that specific
claim is unverified by this probe — it rests on the existing native fuzz/unit test coverage
in `raw.rs`, not on a real CR3 file driven through wasm here.

---

## Q4 — Memory + time: the mobile risk

| | file size (disk) | preview dims | preview px | wasm elapsed | wasm mem before | wasm mem after | wasm mem delta |
|---|---|---|---|---|---|---|---|
| DNG (Leica, full-frame) | 84,872,192 B (80.9 MiB) | 8368×5584 | 46.7 MP (**~full sensor res**) | 301 ms | 87.2 MB | 319.5 MB | 232.3 MB |
| RAF (Fuji, APS-C) | 50,268,160 B (47.9 MiB) | 1920×1280 | 2.46 MP (**screen-res only**) | 86 ms | 52.6 MB | 110.5 MB | 57.9 MB |

Native cross-check (same worktree, native release binary, `crustyimg info`, `/usr/bin/time
-l`):

| | wall time (whole process) | peak RSS / footprint |
|---|---|---|
| DNG | 0.62 s | 320 MB |
| RAF | 0.05 s | 111 MB |

The native peak-memory figures land within a few MB of the isolated wasm-VM `mem_after`
figures — a good consistency check (same JPEG decoder, same algorithm, both measuring the
same underlying allocation). The wasm decode-only time is faster than the native
*whole-process* time, which is unsurprising since the native number includes process
startup and full CLI plumbing the isolated wasm test skips.

**Judgment: this is a real, not hypothetical, mobile risk — and it is highly
camera-dependent, not RAW-dependent.** The two samples bracket a huge range from the SAME
mechanism (largest-embedded-JPEG-preview extraction): a Leica DNG's preview is ~46.7 MP
(essentially the full sensor), while this Fujifilm RAF's preview is only 2.46 MP. Camera
firmware — not this codebase — decides how big the embedded preview is, so "RAW support" on
wasm has no single memory number; it has a distribution set by what cameras are in the wild.
A ~320 MB peak for *just the extraction step* (before any subsequent
downscale/optimize/re-encode pipeline stage, which would add more on top) is a meaningful
fraction of what a memory-constrained mobile Safari tab tolerates before the OS or WebKit
intervenes. It would very likely be fine on a modern, higher-RAM iPhone; it is a plausible
OOM/tab-kill on an older or lower-RAM device.

**Could not verify — and this is the load-bearing gap:** no real iOS device or Safari
session was used (per the probe's own read-only constraint on the live demo, and no device
in hand). All numbers above are Node's V8 wasm engine on a desktop Mac. V8 and Safari's
JavaScriptCore are different wasm engines with different memory-growth and GC behavior, and
iOS Safari's actual per-tab ceiling is a moving target across device/OS-version generations
that no engineering estimate substitutes for a real measurement. **This project has an
established, repeatedly-earned lesson that an unverified device-dependent claim is exactly
the kind of thing that ships wrong** (`a-claimed-failure-mode-is-as-unproven-as-a-claimed-success`,
`never-drive-the-maintainers-live-browser`) — so this number should be read as "credible
evidence of a real risk," not as "proof it's safe" or "proof it's fatal."

---

## Q5 — The cleanest wiring

Reviewed DEC-064 (the wasm boundary is `cfg(target_arch)`, a thin additive `wasm-bindgen`
shim over the shared engine — `src/wasm.rs:1-46`) and DEC-055 (extension routing for RAW is
**required, not a convenience**: TIFF-based RAW — `nef/nrw/cr2/arw/srf/sr2/dng/rw2/orf/pef/
srw/rwl`, 11 of the 12 `RAW_EXTENSIONS` — is byte-indistinguishable from a plain `.tif`).

**(b) Content-sniffing inside `from_bytes` is ruled out for the majority of the format
list** by DEC-055's own stated reason (mis-routing a legitimate `.tif`), which is exactly
why native routes by extension in the first place. Only `.raf` (distinct
`"FUJIFILMCCD-RAW "` magic) and `.cr3` (ISOBMFF, in principle brand-sniffable like AVIF/HEIC
already are in `image/sniff.rs`) are even theoretically content-discriminable — and even for
those two, reliable sniffing wasn't attempted or verified here. A content-only
discriminator does not exist for the format family as a whole; extension is genuinely
required, matching DEC-055's existing native design.

**(a)/(c) converge into one clean, additive design**, and I built and validated a working
version of it:

- `demo.js` already carries `source.file.name` (the dropped/picked `File`'s name, extension
  included) at load time (`demo.js:320,327,471,542` all read `source.file.name` already) —
  **zero new plumbing** is needed to get the extension into JS; it's already there when a
  file lands.
- Add one new additive `#[wasm_bindgen]` export, e.g. `rawPreview(bytes: &[u8]) ->
  Result<Vec<u8>, JsError>`, that calls `crate::image::raw_preview` and PNG-encodes the
  result (exactly what I prototyped as `raw_preview_probe` for Q2/Q3, and confirmed
  correct end-to-end). The demo's JS checks the extension against a RAW list (ideally
  mirroring `RAW_EXTENSIONS` — see below) and calls this new export **instead of** the
  normal decode path when it matches, then feeds the returned PNG bytes into the
  **existing, unmodified** `optimizeDetailed`/`transform`/`info` pipeline.
- This exactly mirrors a precedent already established in this codebase: `wasm.rs`'s own
  doc comment on `score()` (lines 499-504) describes browser-decodes-AVIF-then-hands-PNG-
  back-to-the-engine as the sanctioned pattern for "a capability the wasm build itself
  lacks, bridged via re-encoded bytes." RAW-preview-then-PNG is the same shape, just with
  Rust doing the extraction instead of the browser.
- **Does not touch the published API.** `info`, `transform`, `optimize`,
  `optimize_detailed`, and `score` are all unchanged — purely additive, so this cannot break
  an existing `crustyimg-wasm` npm consumer.
- A small drift risk: JS would need its own copy of the RAW-extension list unless
  `raw::is_raw_extension` (or a name-only variant of it) is also exposed to wasm so the two
  lists can't diverge. `is_raw_extension` is currently `pub(crate)` in `raw.rs:88`, so
  wiring a thin `#[wasm_bindgen] pub fn is_raw_extension(name: &str) -> bool` wrapper in
  `wasm.rs` is a one-line addition once the mechanism above exists.

---

## Q6 — What the demo needs beyond the engine

**`accept` attribute (`demo/index.html:40`):** currently `image/png,image/jpeg,image/gif,
image/webp,image/svg+xml,image/avif,.png,.jpg,.jpeg,.gif,.webp,.svg,.avif` — no RAW
extensions or MIME types. This only gates the OS file-picker dialog; **drag-and-drop
bypasses `accept` entirely** (`demo.js:541-543`, `e.dataTransfer?.files?.[0]` is taken
unconditionally), which is exactly how the reported symptom happened — the drop succeeded
as a browser event and only failed once the bytes hit the decoder. Wiring RAW in for real
should add the RAW extensions to `accept` too, for picker-dialog parity with drop.

**Honest error for a RAW with no decodable preview:** `raw.rs` already types this —
`ImageError::Decode("raw: no decodable embedded JPEG preview")` when no embedded JPEG
survives the scan, `ImageError::LimitsExceeded("raw: embedded preview exceeds decode caps:
{reason}")` when the only candidate is oversize. `demo.js`'s `showError` (line 278) surfaces
`e.message` verbatim: `` `Can't convert ${source.file.name}: ${e.message ?? e}` ``. So today
these messages would read e.g. *"Can't convert IMG_1234.CR2: raw: no decodable embedded
JPEG preview"* in the browser's error banner. Functionally honest and correct, but the
`raw: ` prefix and terse phrasing were written for CLI stderr, not a browser reader — worth
a copy pass against this project's own established bar for user-facing text
(`comments-plain-no-spec-refs`: plain, behavior-first, no internal-symbol flavor) before
shipping, not a blocker to wiring.

**The deliberate `RAW_EXTENSIONS` exclusion:** `.x3f` (Sigma Foveon) is NOT in the list
(`raw.rs:59-60`, documented: "carries no standard baseline-JPEG preview"). A `.x3f` drop
falls through to the generic decoder and gets a normal "unsupported format" error — no
special demo handling needed; this doesn't affect the demo differently than any other
genuinely unsupported format already does.

---

## Q7 — Sizing

**S** for the mechanical wiring itself:
- One new ~15-line `#[wasm_bindgen]` export (prototyped, working, independently verified —
  see Q2/Q3) plus optionally a one-line `is_raw_extension` wrapper.
- Bundle cost: **+1,214 B brotli, measured** — a rounding error against the 1.33 MB budget.
- Demo-side: an extension check (data already in hand via `source.file.name`), an `accept`
  attribute update, a branch to call the new export, and an error-copy pass (Q6).
- Tests: 2-3 new `tests/wasm_roundtrip.rs` cases using **synthetic** RAW-shaped fixtures —
  `raw.rs`'s own unit tests already show the exact pattern (`raw_blob()`: a TIFF header +
  embedded JPEGs built in-process), so no real camera file is needed in the committed test
  suite, consistent with this project's "generate fixtures, never shell out" convention.

**The riskiest part is squarely Q4: the unverified mobile memory ceiling for a
full-resolution embedded preview**, and it is genuinely camera-dependent (46.7 MP for the
Leica sample vs. 2.46 MP for the Fujifilm sample, same mechanism). Given this project's own
repeated experience with unverified device-dependent claims shipping wrong, I would not
recommend landing this as a blanket, un-gated "RAW now supported" claim without either a
real low/mid-range iOS device test, or an engineered bound.

### Recommendation: wire it in, gated by a cheap pre-check — not a binary full-support-vs-fallback choice

Wire the additive path in (it's S-sized, proven correct, and nearly free in bundle size),
but **before calling the expensive extraction**, peek the RAW's largest embedded JPEG
candidate's *declared* dimensions — the same SOF-header peek `decode_jpeg_with_limits`
already performs internally for the DEC-063 budget check (`raw.rs:198-205`) — and if it
exceeds a conservative, demo-specific pixel threshold (well under the measured-risky 46.7 MP
DNG, comfortably above the measured-safe 2.46 MP RAF — the exact number is a product/spec
decision, not this probe's to make), show the honest "this RAW's embedded preview is
high-resolution — convert it with the CLI" fallback message instead of attempting the
decode.

This is deliberately **not** one of the probe's two named options taken straight: a pure
graceful-fallback would throw away a working, cheap, correct capability for what this
sample suggests may be the common case (many cameras embed screen-res previews, not
full-sensor ones); a blanket "full support" claim would repeat the exact unverified-claim
pattern this project has been burned by before. A size gate converts an unverified,
open-ended mobile-OOM risk into a bounded, engineered, and testable one, and it can ship
without waiting on real-device access — the device test then becomes a launch-readiness
check on the THRESHOLD value (does 24 MP survive a low-end iPhone tab, or does it need to be
12 MP?), not a blocker on whether to build the feature at all.

---

## Full "could not verify" list

- CR3 (Canon, ISOBMFF-based RAW) on wasm — no sample file was available locally; DEC-055
  claims the same scan mechanism covers it, unverified here.
- Any real device / real mobile browser (iOS Safari in particular) — all timing/memory
  numbers are Node's V8 on a desktop Mac, not JavaScriptCore on a phone. This is the single
  biggest open question the recommendation above is designed around, not resolve.
- Whether `wasm-opt` (off by default per DEC-066) would change the measured +1,214 B bundle
  delta.
- The exact wording/tone of a browser-shown RAW error message against this project's
  established user-facing-copy bar — flagged, not fixed or user-tested.
- Whether `optimizeDetailed`'s own downstream pipeline stages (downscale, re-encode,
  perceptual search) would meaningfully add to or reduce the peak measured here once a
  RAW-derived preview flows through them — only the bare `raw_preview` extraction step was
  measured in isolation.
