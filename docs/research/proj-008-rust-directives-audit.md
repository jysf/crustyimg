# PROJ-008 — Audit: six proposed "Senior Engineering Directives" (D1–D6)

**Status:** read-only audit. No production code, `Cargo.toml`, `constraints.yaml`, or
decision was changed by this session. This document is the sole artifact; it exists
for maintainer review **before** any remediation is framed or any directive is codified.

**Scope of evidence:** every verdict below is grounded in `file:line` references against
the working tree at branch `rust-directives-audit` (off `main`, commit `56a4df7`).
The audit was deliberately adversarial toward both the directives and the
orchestrator's stated prior — each falsifiable claim was confirmed or refuted with
evidence, not rubber-stamped.

---

## Summary verdict table

| Directive | Verdict | One-line evidence |
|---|---|---|
| **D1** checked-math-on-dimensions | **SATISFIED** (with a naming correction) | Every buffer-sizing multiply is downstream of a pixel/alloc cap (`check_pixel_budget` 64 Mpx, `MAX_AREA` 128 Mpx, per-decoder `check_caps`); worst case 64 Mpx × 4 = 268 M < 32-bit `usize::MAX`. The cap variant is **`ImageError::LimitsExceeded(String)`** (`error.rs:39`), **not** `DimensionsTooLarge`. |
| **D2** zero-allocation-pipeline | **N-A-false-premise** | Per-op buffers are real (`Invert` `operation/mod.rs:197`, resize `:526`) but bounded by the decode cap and dwarfed by the measured bottleneck (AVIF encode, 33.6 s @ 12 Mpx). A ~48 MB alloc+memcpy is sub-millisecond → <0.01 % of wall-clock. |
| **D3** miette-cli-diagnostics | **DESIGN-CHANGE** (not a violation) | Current design is deliberate `thiserror` (DEC-007); `miette`/`anyhow` are **not** direct deps. Recipe TOML spans are flattened at `recipe/mod.rs:247` (`Parse(e.to_string())`), but `toml` 0.8 already renders line/column + caret in that string. Real but incremental UX win, at the cost of a new dep + boundary rework. |
| **D4** semver-in-toml | **DECISION-NEEDED** (map below) | 30 exact `=` pins, a deliberate convention (AGENTS.md §5, DEC-011/013) serving the PROJ-007 reproducible-build thesis. **No downstream Cargo consumer exists today** — crustyimg is not on crates.io (DEC-040/041: backlog #5), and the npm package ships a *compiled* `.wasm`, not a Cargo tree. Pins only bite *if/when* the library is published to crates.io. **[Corrected 2026-07-19 — DEC-079: this "not on crates.io" premise was false; crustyimg has published since v0.1.0 (now 0.4.0, `has_lib: true`), so the caret migration shipped in SPEC-099 rather than being deferred.]** |
| **D5** static-dispatch-in-hot-loops | **N-A-false-premise** | `Box<dyn Operation>` dispatches **once per recipe step**, not per pixel: `Pipeline::run` (`pipeline/mod.rs:53–55`) folds `op.apply(Image) -> Image` over the whole image. ~3 vtable calls/image. Prior confirmed. |
| **D6** tile-based-parallelism | **N-A-false-premise** | rayon parallelizes **across images** (`cli/mod.rs:1313, 1337` `all.par_iter()` → `apply_one` per input). There is **no intra-image parallelism at all** — no stripes and no tiles — so the "stripes vs tiles" choice does not exist (DEC-006). Prior confirmed. |

---

## Disposition (maintainer decision, 2026-07-19)

This audit landed on `main` on 2026-07-19. The maintainer reviewed it and dispositioned each finding:

- **Adopted → framed as specs (STAGE-031):**
  - The structural finding (`src/cli/mod.rs`, 6,483 lines) → **SPEC-097** — decompose into a `cli/`
    submodule tree + dedup `escape_json`, behind a byte-identity gate.
  - **D4** (semver-in-toml) → **SPEC-098 / DEC-078**, corrected by **SPEC-099 / DEC-079** — DEC-078's "not
    on crates.io" premise was false (crustyimg has published since v0.1.0, currently 0.4.0, `has_lib: true`),
    so the caret relaxation of the library-public deps shipped in SPEC-099 rather than being deferred to a
    future publish; the committed `Cargo.lock` keeps the binary reproducible.
- **Shelved → no action (do not re-raise):**
  - **D1** — SATISFIED; only optional belt-and-suspenders remained (the two plain-`usize` sites are
    proven-bounded). Not worth churn.
  - **D2** — N/A; per-op allocation is not the bottleneck (encode dominates).
  - **D3** — a design change vs DEC-007, not pursued now.
  - **D5**, **D6** — N/A, false premise for this codebase.

This section is the durable record of that call; the per-directive detail below is the evidence base.

---

## D1 — checked-math-on-dimensions → **SATISFIED** (naming correction)

**Directive claim:** never `width * height * channels` with plain `*`; use `checked_*`
or cast to `u64`; reject oversize with a typed `ImageError::DimensionsTooLarge`.
**Prior:** mostly satisfied via DEC-034/038/063; hunt for gap sites.

### The typed cap variant is misnamed in the directive

There is **no `DimensionsTooLarge` variant**. The real, single cap error is:

- `ImageError::LimitsExceeded(String)` — `src/error.rs:39`

Every decode-cap rejection funnels through it (`image/avif.rs:200/207/215`,
`image/heic.rs`, `image/raw.rs`, and `check_pixel_budget` at `image/mod.rs:82`).
The resize op uses a *different* typed error on the same concern:
`OperationError::Apply { op: "resize", .. }` (`operation/mod.rs:494/502`). Any spec
that references `DimensionsTooLarge` would not compile.

### Every buffer-sizing multiply, classified

| Site | Expression | Guard | Class |
|---|---|---|---|
| `operation/mod.rs:501` | `(tw as u64) * (th as u64) > MAX_AREA` | `MAX_AREA = 128 Mpx`, `MAX_EDGE = 50_000` (`:487/491`) | **guarded** (u64 + cap, rejects before alloc) |
| `analysis/mod.rs:214` | `(w as u64) * (h as u64)` | u64 cast | **guarded** |
| `analysis/mod.rs:222` | `w as usize * h as usize` (`Vec::with_capacity`) | bounded upstream by decode cap (≤64 Mpx) | plain `usize` multiply, **proven bounded** |
| `image/avif.rs:213` | `(w as u64) * (h as u64) * 4` | inside `check_caps`, compared to `max_alloc` | **guarded** |
| `image/avif.rs:179` | `(w as usize) * (h as usize) * 3` | `check_caps` at `:161` runs first | **guarded upstream** |
| `image/avif.rs:438` | `vec![0u8; (w as usize)*(h as usize)*4]` | `check_caps` on decoded dims | **guarded upstream** |
| `image/svg.rs:220` / `:163` | `* 4` alloc | `check_caps` at `svg.rs:151` | **guarded upstream** |
| `image/heic.rs:199` | `(w as u64) * (h as u64) * 4` | `check_caps`, `max_alloc` | **guarded** |
| `image/raw.rs:154` | `(img.width() as usize) * (img.height() as usize)` | post-successful `decode_jpeg_with_limits` (caps already enforced) | **guarded upstream** |

### Why no 32-bit-WASM overflow gap exists

The directive's rationale (a 100 000 × 100 000 image wraps `usize` on 32-bit wasm)
does not reach a live site here:

1. **Two independent pre-decode caps** bound every path before a channel multiply:
   `check_pixel_budget` rejects > `MAX_IMAGE_PIXELS = 64 * 1024 * 1024` px
   (`image/mod.rs:71/82`), and the `image` crate's own `Limits` are configured with
   `max_alloc = 512 MiB` + `MAX_IMAGE_DIMENSION = 65_535` (`image/mod.rs:42/47/327`).
2. **Worst-case arithmetic after the cap:** 64 Mpx × 4 channels = 268 435 456, which
   is < `u32::MAX` (4 294 967 295). So even the plain-`usize` sites
   (`analysis/mod.rs:222`, `raw.rs:154`) cannot wrap a 32-bit `usize`.
3. **The wasm decoder set is the safest subset.** AVIF/HEIC/RAW decode are all
   `cfg(not(target_arch = "wasm32"))` (`Cargo.toml:130` table). On wasm the only
   decoders are the `image`-crate formats + SVG, both capped as above. The wasm entry
   points (`src/wasm.rs`) do no raw dimension arithmetic — they delegate to the
   guarded core.

**Verdict: SATISFIED.** The directive is already the de-facto policy; the plain
`usize` multiplies at `analysis/mod.rs:222` and `raw.rs:154` are stylistic-only (they
cannot overflow given the caps) and do **not** constitute a gap. If the maintainer
wants belt-and-suspenders, those two lines are the only candidates — but there is no
correctness defect to fix.

**Prior correction:** the prior was right ("mostly satisfied"); the audit upgrades it
to fully satisfied and flags that the directive's assumed variant name is wrong.

---

## D2 — zero-allocation-pipeline → **N-A-false-premise**

**Directive claim:** pipeline ops must not allocate a fresh `Vec`/`ImageBuffer` per
step; use a caller `&mut [u8]` or a pipeline scratch buffer.
**Prior:** allocation is not the bottleneck; verify before anyone refactors.

### The per-op allocation is real…

The `Operation` trait is by-value move: `fn apply(&self, img: Image) -> Result<Image, OperationError>`
(`operation/mod.rs:150`). Concretely:

- `Invert::apply` allocates a fresh RGBA8 buffer via `img.pixels().to_rgba8()`
  (`operation/mod.rs:197`) and returns a new `Image`.
- `Resize::apply` allocates a new destination `fast_image_resize::Image`
  (`operation/mod.rs:526`).
- `Pipeline::run` moves the `Image` through each op (`pipeline/mod.rs:52–56`) with **no
  intermediate clone** (that part of DEC-002 already holds), but each op that touches
  pixels does produce one new buffer.

### …but it is nowhere near the bottleneck

The measured bottleneck is **AVIF encode**, ~33.6 s at 12 Mpx (STAGE-030 bench;
`scripts/bench.py`). The cost model there is *megapixels of AVIF encode*, not buffer
churn. A single RGBA buffer for a 12 Mpx image is ~48 MB; allocating + copying it is
sub-millisecond on any target — well under **0.01 %** of the encode wall-clock. The
per-op buffer is also bounded (decode cap ≤ 64 Mpx), so it cannot fragment the heap
unboundedly.

A zero-copy `&mut [u8]` rewrite would also fight the design: ops legitimately change
dimensions (resize) and color type (RGBA promotion), so a single caller-owned buffer
cannot be reused across steps without reallocation anyway.

**Verdict: N-A-false-premise.** The premise ("per-op buffers tank throughput") is false
for this workload. No spec is warranted; record as N/A so it is not re-raised.

**Prior confirmed.**

---

## D3 — miette-cli-diagnostics → **DESIGN-CHANGE** (not a violation)

**Directive claim:** keep engine errors `thiserror`; wrap them in `miette::Report` at
the CLI boundary for source spans/help/chains.
**Prior:** a real UX win, but a design change vs DEC-007, not a violation.

### Current design (DEC-007), confirmed

- Library errors are matchable `thiserror` enums; **`anyhow` and `miette` are not
  direct dependencies** (`grep` of `Cargo.toml` finds only DEC-007 comments in
  `error.rs:4/17`; `anyhow` appears in `Cargo.lock` only transitively; `miette` is
  absent from the lock entirely).
- The CLI maps typed errors to exit codes at the boundary (`cli/mod.rs:706`,
  `.code()` on `CliError`), matching DEC-007.

### What miette would actually add — and what already exists

The directive's motivating example ("line/column pointers, not `Error: invalid params`")
is **partially already delivered**. Recipe parsing does:

```
toml::from_str(s).map_err(|e| RecipeError::Parse(e.to_string()))   // recipe/mod.rs:247
```

`toml` 0.8's error `Display` already renders `TOML parse error at line N, column M`
plus a source snippet with a caret. Flattening to `String` at `:247` **preserves that
rendered text** — it only discards the *structured* span (`toml::de::Error::span()`
→ `Range<usize>`) for programmatic reuse.

So miette's genuine marginal value is:

1. **Uniform** source-linked diagnostics across *all* error types (not just TOML —
   e.g. `RecipeError::InvalidOperation`, `UnknownOperation`, which today carry no
   span), with a `#[help(...)]` line.
2. Rendered **error chains** (`caused by`) for wrapped sources.

### Cost

- New dependency (`miette`, plus its rendering stack) — a `DEC-*` per the
  `no-new-top-level-deps-without-decision` constraint.
- Boundary rework: thread the recipe source text + byte offsets into a `miette`
  `Diagnostic` at the binary. `RecipeError::Parse(String)` would need to grow a span
  field, touching every construction/match site.
- Philosophical collision with DEC-007's "thiserror internal, thin binary boundary"
  posture — miette at the boundary is anyhow-adjacent.

**Verdict: DESIGN-CHANGE.** Not a violation of anything. If pursued it is a scoped
UX spec (recipe-diagnostics), explicitly reconciled with DEC-007, and its payoff
should be weighed against the fact that TOML — the directive's own example — is
already ~80 % handled by `toml`'s native rendering.

**Prior confirmed** (with the sharpening that the flagship example is already mostly
solved).

---

## D4 — semver-in-toml → **DECISION-NEEDED** — THE MAP

> **Correction (2026-07-19, DEC-079):** the "not on crates.io" premise below was
> false when checked against crates.io directly — crustyimg has been published
> since v0.1.0 (2026-07-04) and auto-publishes every tag. See DEC-079, which
> supersedes DEC-078 (the decision this section's finding fed into).

**Directive claim:** never exact-pin (`=x.y.z`); use caret; reproducibility comes
from committing `Cargo.lock`, not from pins, which cause consumer dependency hell.
**Prior:** genuine tension; the map decides it.

### The decisive fact: there is no downstream Cargo consumer today

D4's entire harm model ("dependency hell for consumers") requires a **Cargo** consumer
that resolves crustyimg's dependency tree. Today there is none:

- **Not on crates.io.** DEC-040/041: crates.io publish is *backlog #5*, a separate
  maintainer-authorized future spec; the release pipeline "does NOT publish to
  crates.io." `AGENTS.md:140` lists crates.io as a *target*, i.e. aspirational.
- **The npm package ships a compiled artifact, not a Cargo tree.** `npm/` /`pkg/`
  contain `crustyimg_bg.wasm` (5.7 MB) + JS glue + `.d.ts`. An npm consumer never
  resolves a Cargo dependency graph, so **no `=` pin is visible to them**.
- **Binary distribution** (cargo-dist releases, `cargo install --locked`) consumes the
  committed `Cargo.lock` directly — pins add nothing and cost nothing there.

### Where the directive is *technically correct*

For a **library published on crates.io**, `Cargo.lock` is ignored by consumers, and
exact `=` pins genuinely do force version-unification conflicts. So D4's underlying
Rust fact is right — it simply has **no live target** in this repo yet. It becomes real
*only* on the day backlog #5 (crates.io library publish) lands.

### Impact table — all 30 `=` pins by reach

Reach classes: **BIN** = native binary + native lib only (`cfg(not(wasm32))`), never in
any published tree today. **WASM-LIB** = compiled into `crustyimg_bg.wasm` (shared
`[dependencies]` or wasm-target table) — shipped as bytes, not as a resolvable pin.
**DEV** = dev/bench only, never shipped.

| Dep (`Cargo.toml` line) | Section | Reach | Would caret matter downstream *today*? |
|---|---|---|---|
| `fast_image_resize =6.0.0` (47) | `[dependencies]` | WASM-LIB + BIN | No (compiled into `.wasm`; no Cargo consumer) |
| `thiserror =2.0.18` (48) | `[dependencies]` | WASM-LIB + BIN | No |
| `serde =1.0.228` (49) | `[dependencies]` | WASM-LIB + BIN | No |
| `toml =0.8.23` (50) | `[dependencies]` | WASM-LIB + BIN | No |
| `kamadak-exif =0.6.1` (61) | `[dependencies]` | WASM-LIB + BIN | No |
| `img-parts =0.4.0` (61) | `[dependencies]` | WASM-LIB + BIN | No |
| `skrifa =0.44.0` (69) | `[dependencies]` | WASM-LIB + BIN | No |
| `zeno =0.3.3` (70) | `[dependencies]` | WASM-LIB + BIN | No |
| `ssimulacra2 =0.5.1` (74) | `[dependencies]` | WASM-LIB + BIN | No |
| `webp =0.3.1` (81, optional) | `[dependencies]` | BIN (feature) | No |
| `libheif-rs =2.7.0` (120, optional) | `[dependencies]` | BIN (`heic` feature; C dep, not wasm) | No |
| `image =0.25.10` (138) | native table | BIN | No |
| `resvg =0.47.0` (141) | native table | BIN | No |
| `re_rav1d =0.1.3` (162) | native table | BIN | No |
| `avif-parse =2.1.0` (163) | native table | BIN | No |
| `clap =4.6.1` (167) | native table | BIN | No |
| `clap_complete =4.6.5` (168) | native table | BIN | No |
| `glob =0.3.3` (171) | native table | BIN | No |
| `rayon =1.12.0` (181) | native table | BIN | No |
| `indicatif =0.18.6` (182) | native table | BIN | No |
| `sha2 =0.11.0` (190) | native table | BIN | No |
| `viuer =0.11.0` (195, optional) | native table | BIN | No |
| `notify =8.2.0` (209, optional) | native table | BIN | No |
| `image =0.25.10` (227) | wasm table | WASM-LIB | No (compiled bytes) |
| `resvg =0.47.0` (243) | wasm table | WASM-LIB | No (compiled bytes) |
| `wasm-bindgen =0.2.126` (249) | wasm table | WASM-LIB | No |
| `wasm-bindgen-test =0.3.76` (254) | wasm dev table | DEV | No |
| `serde_json =1.0.150` (291) | `[dev-dependencies]` | DEV | No |
| `tempfile =3.27.0` (301) | `[dev-dependencies]` | DEV | No |
| `criterion =0.8.2` (306) | `[dev-dependencies]` | DEV | No |

**Every row is "No" today.** Not one `=` pin currently reaches a Cargo dependency
resolver outside this repo.

### The tension, framed for the maintainer

- **Pins vs the reproducible-build thesis (PROJ-007):** D4 correctly notes reproducibility
  is delivered by the *committed `Cargo.lock`*, not by the manifest pins. For the
  **binary** (the only shipped Rust artifact today), the lock already guarantees
  reproducibility; the `=` pins are redundant-but-harmless there.
- **Pins vs a future crates.io library:** the day backlog #5 lands, the `=` pins on
  the **shared `[dependencies]` + wasm-lib rows** (those that end up in the published
  crate's *public* dependency tree — `thiserror`, `serde`, `image`, `toml`, etc.)
  become a real downstream-resolution liability. The BIN-only and DEV rows never do
  (a `[[bin]]` crate's pins and dev-deps don't constrain library consumers).

### Decision framing (no edit made — the map informs it)

Three coherent options for the maintainer:

1. **Keep all pins (status quo).** Zero downstream harm today; defer the question to
   the crates.io-publish spec, which must relax the *library-public* pins anyway.
2. **Relax now, but only the library-public deps** (`[dependencies]` + wasm table) to
   caret, keeping BIN/DEV pinned. Pre-pays the crates.io cost; mild churn; the
   committed lock preserves build reproducibility regardless.
3. **Relax everything to caret + rely on `--locked`.** Cleanest semver hygiene; loses
   the "manifest documents the exact tested version" property that AGENTS.md §5 and the
   DEC-011/013 pattern were written for.

**Recommendation:** fold D4 into the crates.io-publish backlog item (#5) rather than a
standalone spec — that is the exact moment the pins start to matter, and relaxing them
outside that context is churn against a deliberate, currently-harmless convention.

**Verdict: DECISION-NEEDED.** The map shows the directive is directionally sound Rust
advice with **no live target yet**; do not treat it as a violation.

---

## D5 — static-dispatch-in-hot-loops → **N-A-false-premise**

**Directive claim:** no `Box<dyn Trait>` inside pixel/tile/chunk loops.
**Prior:** ~3×/image (per recipe step), so N/A — confirm or refute.

`Box<dyn Operation>` is stored in `Pipeline.ops: Vec<Box<dyn Operation>>`
(`pipeline/mod.rs:20`), built once per recipe step via the registry
(`operation/registry.rs`, `Constructor = fn(&OperationParams) -> Result<Box<dyn Operation>, _>`
at `:25`). The dispatch site is:

```
for op in &self.ops {          // pipeline/mod.rs:53
    current = op.apply(current)?;   // one vtable call per OP, whole-image in/out
}
```

`apply` takes and returns an entire `Image`; the per-pixel work happens **inside** each
op's concrete body (e.g. `Invert`'s `for pixel in buf.pixels_mut()` at
`operation/mod.rs:198`, monomorphized, no `dyn`). So vtable dispatch is **per recipe
step** (~3/image), never per pixel or per tile.

**Verdict: N-A-false-premise.** No `dyn` dispatch exists in any inner loop. Record as
N/A. **Prior confirmed.**

---

## D6 — tile-based-parallelism → **N-A-false-premise**

**Directive claim:** parallelize image work in cache-friendly tiles, never
vertical/horizontal stripes.
**Prior:** across-images batch fan-out; no intra-image stripes → N/A.

rayon is used in exactly one place — the batch apply path — and it fans out **across
images**:

```
all.par_iter().map(|input| { apply_one(&recipe, &registry, input, ...) })  // cli/mod.rs:1313 & :1337
```

Each rayon task processes **one whole input image** (`apply_one`); the code comment at
`cli/mod.rs:1172` notes each task rebuilds its own registry because `Operation` is not
`Send`. There is **no intra-image parallelism whatsoever** — grep for
`par_chunks`/`par_bridge`/intra-image rayon finds nothing but this batch fan-out. So
neither stripes nor tiles exist; the choice the directive legislates is not present in
the codebase (consistent with DEC-006: rayon for batch, no async).

**Verdict: N-A-false-premise.** Record as N/A. **Prior confirmed.**

---

## Recommendation

### Warrant a targeted spec — *maybe one, and only after maintainer review*

- **D3 (miette recipe diagnostics)** is the *only* directive describing a genuine,
  non-existent-today capability (uniform source-linked diagnostics + help across all
  error types). It is a **scoped UX spec**, not a bug fix, and must be reconciled with
  DEC-007 (new dep → `DEC-*`). Weigh it against the fact that its flagship example
  (TOML spans) is already ~80 % delivered by `toml`'s native error rendering — the
  incremental win is the non-TOML error types and a `help:` line, which may not clear
  the new-dependency bar.

### Record as explicit N/A decisions (so they are not re-raised)

- **D2** — false premise: per-op allocation is <0.01 % of the AVIF-encode-bound
  wall-clock; a zero-copy rewrite also fights dimension/color-type changes.
- **D5** — false premise: `dyn Operation` dispatches per recipe step (~3/image), never
  per pixel.
- **D6** — false premise: parallelism is across-images batch fan-out (DEC-006); no
  intra-image stripes or tiles exist to convert.
- **D1** — already satisfied de-facto by DEC-034/038/063 + `check_pixel_budget`; the
  only residual is two *stylistic* plain-`usize` multiplies
  (`analysis/mod.rs:222`, `raw.rs:154`) that are provably bounded and cannot overflow.
  Close as "no gap"; optionally note the two lines as a cosmetic-only follow-up.

### D4 decision framing for the maintainer

D4 is directionally correct Rust advice with **no live downstream target today** (not
on crates.io; npm ships compiled bytes; the binary uses the committed lock). Do **not**
treat it as a violation. Fold the pin-relaxation question into the **crates.io library
publish backlog item (#5)** — that is the precise moment the *library-public* pins
(`[dependencies]` + wasm-table rows) start to matter; the BIN-only and DEV pins never
do. Relaxing pins outside that context is churn against the deliberate AGENTS.md §5 /
DEC-011/013 convention.

---

## What the orchestrator's prior got wrong

1. **D1 variant name.** The prior (echoing the directive) assumed a typed
   `ImageError::DimensionsTooLarge`. It does not exist. The real cap variant is
   `ImageError::LimitsExceeded(String)` (`error.rs:39`); resize uses
   `OperationError::Apply` (`operation/mod.rs:502`). A spec written against
   `DimensionsTooLarge` would not compile.
2. **D3 scope over-stated.** The prior framed miette's win as turning `Error: invalid
   params` into line/column pointers. For the directive's own example (malformed TOML),
   `toml` 0.8 *already* renders line/column + caret; `recipe/mod.rs:247` preserves that
   text and discards only the structured span. The real (smaller) win is uniform
   diagnostics across the *non-TOML* error variants.
3. **D4 "genuine tension" needed sharpening.** The tension is real but **latent**: it
   has zero live downstream target today (crates.io publish is unshipped backlog #5;
   npm ships a compiled artifact). The prior implied an active consumer-facing problem;
   there isn't one yet. **[Corrected 2026-07-19, DEC-079: this "unshipped backlog #5"
   premise was false — crustyimg was already published (0.4.0, `has_lib: true`), so the
   consumer-facing problem was live; the caret fix shipped in SPEC-099.]**
4. **D1 fully, not "mostly," satisfied.** The prior said "mostly satisfied … hunt for
   gap sites." The hunt found no overflow gap: every channel multiply is downstream of
   a cap, and the worst case (64 Mpx × 4 = 268 M) fits a 32-bit `usize`. The correct
   verdict is SATISFIED, not "mostly."

The prior was **correct** on D2, D5, and D6 (all N/A), and correct that D3 is a design
change rather than a violation and D4 a decision rather than a violation.

---

*Audit only. Nothing here is remediated, codified, or merged. Framing of any fix or
decision happens after maintainer review of this document.*
