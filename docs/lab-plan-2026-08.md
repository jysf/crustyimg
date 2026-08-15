# crustyimg-lab — project plan (2026-08-15)

**Source.** An ideation + roadmap session run against this repo on 2026-08-15, read-only. It takes
`DEC-088` (the lab/workhorse split), `docs/feature-set-triage-2026-08.md` (the 18-item feature set)
and `docs/territory.md` as its inputs, and answers the seven questions DEC-088 left open: what lab
is for, its op set, its interaction model, what "correct" means for it, what the shared core needs,
what already exists, and its dependencies.

**Recorded here with provenance on purpose**, for the same reason the feature-set triage was:
STAGE-036 once carried a candidate list from an untracked draft that no committed record explained,
and it read as a backlog to every session that met it until it was formally declined. A document
with a thesis, a stage table and a DEC recommendation is *more* likely to be mistaken for committed
work than a bare list was.

**What was done to it.** Every load-bearing claim in the input was re-checked against the repo,
against the crates.io API, or by compiling an out-of-crate probe. Eight findings changed the
planning answer and are recorded in §0 with their evidence. The verification standard is the same
one the triage used.

**Status of this document.** A plan, not a backlog. **Nothing here is committed work**, and lab is
not scheduled — it is sequenced behind STAGE-042/043/045 (§9). The consequences of this plan that
*are* now tracked were split out into `docs/backlog.md` and `docs/roadmap.md` rather than left
here. Items still marked "recommend" are recommendations.

**Shelf-life.** Crate probes, the seam-probe result, and stage statuses are stamped 2026-08-15.
Re-run the probes in the appendix before building; do not cite this document's numbers as current.

---

## 0. Findings — what did not survive checking

Listed first because each one changes what the project costs or how it is shaped. This mirrors
the triage document's own §1, and for the same reason: a previous session was handed 11 crates of
which 2 did not exist.

### F1 — The seam is real, and the widening count is **zero**. Measured.

DEC-088's central bet is that lab can ride the shared core. The triage flagged this as unverified
(§4: *"whether `Constructor` and the `Operation` trait are public enough for an out-of-crate
implementor"*). **It is verified now.**

I built a throwaway crate outside the repo depending on `crustyimg` by path and exercised the full
surface a real lab binary needs: load from bytes, pixel access and reconstruction, implement
`Operation`, `register()` into the registry, parse a recipe, `build_pipeline()`, `run()`, emit a
recipe via `Recipe::from_ops` → `to_toml`, `sink::encode_to_bytes`, `quality::score`,
`analysis::Analysis::compute`, and `tiny_skia::Mask`.

`cargo check` → **exit 0. Zero widenings required.**

That green has teeth: a negative control referencing `crustyimg::cli::ops::run_edit` (a
`pub(super)` item) fails with `error[E0603]: module 'ops' is private`, exit 101. The probe can go
red, so its green means something.

> **Note on method:** my first probe run reported exit 0 while actually failing to compile — the
> command was piped through `tail`, so I read the pipe's exit code. Caught by reading the log.
> This is worth carrying into the build prompts (§10).

**Consequence:** DEC-088's stated *"Also wrong if… the shared core needs constant `pub` widening
for lab"* — **passes**, measured at 0. The seam is in the right place. This is the single
strongest signal that the project is architecturally sound.

### F2 — But the core leaks two crates it does not re-export, and the naive fix silently widens lab's attack surface

The first probe run failed with `E0433` on exactly two crates: `image` and `toml`. Both appear in
crustyimg's **public API** but are not re-exported:

- `Image::pixels(&self) -> &::image::DynamicImage` (`src/image/mod.rs:269`)
- `Image::with_pixels(self, DynamicImage) -> Image` (`src/image/mod.rs:321`)
- `OperationParams::from_map(BTreeMap<String, toml::Value>)` (`src/operation/mod.rs:48`)

So every lab op author must declare `image` and `toml` themselves. That is not merely an
ergonomics wart. **Cargo features are additive across the graph**, so a lab author writing the
obvious `image = "0.25.10"` changes crustyimg's own build inside the lab binary. Measured with
`cargo tree -e features`, defaults-off vs defaults-on:

```
+dds  +exr  +ff  +hdr  +pnm  +qoi  +tga  +rayon  +default  +default-formats
```

**Ten features, six of them new decoders.** And they are *reachable*, not merely linked: the
decode path is `ImageReader::new(...).with_guessed_format()` (`src/image/mod.rs:521-522`), generic
dispatch over whatever formats are compiled in. A naive lab `Cargo.toml` therefore makes lab
accept DDS, OpenEXR, Farbfeld, Radiance HDR, PNM, QOI and TGA — formats crustyimg deliberately
excluded and which `tests/hostile_inputs.rs` has never fuzzed.

**Action (2 lines, and it is a safety item, not a nicety):** `pub use ::image;` and
`pub use ::toml;` in `src/lib.rs`, and a lab constraint that lab **must not** declare `image`
directly. These are re-exports, not visibility widenings — the widening count in F1 stays 0.

### F3 — The registry holds **four** ops. This is the largest correction to the cost model.

`OperationRegistry::with_builtins()` registers exactly `identity`, `invert`, `resize`,
`auto-orient` (`src/operation/registry.rs:80-83`). Repo-wide there are **five** non-test
`impl Operation for` blocks — `Identity`, `Invert`, `Resize`, `AutoOrient`, `Watermark`
(`src/operation/mod.rs:161,185,348,588,784`) — and `Watermark` is deliberately **not** registered
(`src/cli/ops.rs:1165`: *"`watermark` is NOT registered in `with_builtins()`"*).

Everything else the CLI does — `thumbnail`, `convert`, `web`, `optimize`, `responsive`, `diff`,
`lint`, `meta`, `build` — lives in `src/cli/` and **is not an `Operation`**. There is **no `crop`
op**; `Gravity` exists (`src/operation/mod.rs:641`) with a doc comment saying it is *"reusable by a
future `crop`"*.

DEC-088 says a lab recipe is *"a SUPERSET of a workhorse recipe — same `[[op]]` array, same
registry."* That is true, and the mechanism works (F1). But **the set being supersetted is four
ops.** Lab does not inherit an editor. It inherits a four-op pipeline and an open seam.

### F4 — `edit` is three flags

`Commands::Edit` takes `--auto-orient`, `--resize-max N`, `--invert` (`src/cli/mod.rs:448-463`).

`docs/territory.md:73-74` — the sentence that carries the deferred-branch permission — reads
*"crustyimg already has `edit` + a recipe pipeline, and geometry/color/effects ops are
editor-adjacent."* Against the code: one geometry op (resize), one colour op (invert), zero
effects ops. The permission is still validly granted; the **base it implies does not exist**.
F3 + F4 together are why the honest v1 scope in §9 is smaller than the triage's seven lab items.

### F5 — The entire mask substrate is already in the tree, free

`resvg` re-exports `tiny_skia` (`resvg-0.47.0/src/lib.rs:17`), and crustyimg **already imports it
by that path**: `use resvg::{tiny_skia, usvg};` (`src/image/svg.rs:55`).

`tiny_skia::Mask` (v0.12.0, in `Cargo.lock`) ships exactly the API §7 needs:

| need (triage §7) | `tiny_skia::Mask` |
|---|---|
| SVG bezier paths as selections | `fill_path`, `intersect_path` |
| grayscale PNG as the "universal escape hatch" | `decode_png`, `from_vec` |
| compose / invert masks | `intersect_path`, `invert`, `clear` |
| 8-bit coverage buffer | `data()`, `data_mut()`, `take()` |

**Named composable masks need zero new dependencies and no new DEC.** The triage billed §7 at
M–L; the substrate half of that is already paid for. This is the biggest cost *reduction* found.

### F6 — The `.cube` in-house estimate is ~2× optimistic, and the licence finding needs correcting

Measured, by downloading the crate and counting: `lut-cube` 0.2.0 is **329 lines**
(`lib.rs` 53 + `cube.rs` 59 + `lut.rs` 217). The triage estimated *"~100 lines to parse, ~50 for
trilinear interpolation."* Against a real comparable that is optimistic by roughly 2×. Budget
**250–350 lines** for parse + trilinear + errors. (Still small; still the right call to in-house.)

**Licence correction:** the triage says lut-cube's *"licence reads non-standard — must clear
`cargo deny` before it is an option."* Its `LICENSE` file is **plain MIT, Copyright (c) 2023 Yury
Korolev**. crates.io reports "non-standard" only because the manifest uses `license-file` rather
than `license`. The practical conclusion survives (`cargo deny` still cannot read it without a
clarification entry) but the stated reason is wrong, and "the licence is unclear" should not be
repeated as a fact.

**A better, independent argument for in-housing** that the triage did not have: lut-cube's only
tests hardcode absolute paths into the author's local DaVinci Resolve install
(`src/lib.rs:28,33` — `/Library/Application Support/Blackmagic Design/…`). They cannot run on any
other machine. That is a supply-chain quality signal, not a licence one.

### F7 — There is no golden-image infrastructure, but two oracle precedents exist and are the right shape

No `insta`, no `proptest`, no `quickcheck` anywhere in `src/` or `tests/`. "Golden" in this repo
means byte-stable JSON/SARIF strings (`src/lint/report.rs:258,343`) and metadata byte comparison
(`tests/metadata.rs:780-883`) — never a picture.

Two existing patterns are, however, exactly what lab needs, and §4 is built on them:

- **`tests/edit.rs:216`** — *"edit output and apply-of-saved-recipe output must be byte-identical."*
  The lab replay oracle, already prototyped on the three-flag `edit`.
- **`src/image/avif.rs:767-793`** — a committed FNV-1a u64 digest of the **decoded RGBA buffer**,
  with the discipline stated in the comment: *"an independent value the code under test cannot
  fabricate."* Golden-image regression without committing images or adding a snapshot crate.

### F8 — Two recorded constraints bite lab harder than they bite the workhorse

`docs/backlog.md:624` records that `resize` resamples in sRGB, not linear light, and that the
pipeline is 8-bit throughout (`to_rgba8()` at `src/operation/mod.rs:197,396,816,817`).

For a resizer that is a quality defect. **For a grading tool it is a correctness defect.** Any
curves / LUT / exposure op lab ships works on 256 levels per channel in a non-linear space:
strong grades band visibly, and a `.cube` baked from a lab expression is baked against the wrong
transfer function. Lab's own tests cannot see this, because both the reference and the candidate
are wrong in the same way. **This is a pre-registered spike question (§12), not a blocker** — but
it must be answered before §2 (LUT) or §8 (expressions) are built, not during.

---

## 1. What lab is for (Q1)

> ### crustyimg-lab is where you find the recipe; crustyimg is where you run it.
>
> Lab's deliverable is **not a picture** — it is a workhorse-runnable artifact (a recipe or a
> `.cube`) that you could not have written by hand, and that the workhorse then executes unchanged
> across a batch.

**Why this is falsifiable rather than marketing:** it makes a mechanical prediction. Lab's emitted
recipe, run through the workhorse, must reproduce lab's preview **byte-for-byte**. That is a test,
not an opinion, and the repo already runs its three-flag ancestor (`tests/edit.rs:216`).

It also yields a sharp feature filter: **if a lab feature's only output is a finished image, it is
the wrong feature.** That single sentence does more scope work than the generalization fence does,
and it is what makes §4 tractable.

```yaml
value:
  thesis: >
    Lab converts a human's per-image judgement into a batch-runnable workhorse artifact.
    Every session ends in a recipe or LUT the workhorse executes unchanged.
  beneficiaries:
    - the maintainer, grading their own Eleventy photo blog before a build
    - photographers/developers who need one look applied across a shoot
    - the workhorse itself — lab is where recipes worth bundling get discovered
    - future RAW-develop users, whose per-image params are lab's first real workload
  success_signals:
    - every lab op has a replay-equivalence test that passes (Tier 0, §4)
    - at least one bundled workhorse recipe was discovered in lab, not hand-written
    - lab-emitted recipes run in the workhorse with zero exit-4s in normal use
    - the cumulative pub-widening count stays at 0 (measured per stage ship)
  risks_to_thesis:
    - lab has no user but the maintainer (top pre-mortem risk, §11)
    - the 8-bit sRGB pipeline (F8) makes lab's grading outputs wrong invisibly
    - "find the recipe" is not actually hard enough to need a second binary
```

---

## 2. The op set and the fence (Q2)

DEC-088's *"right if"* is: **a proposed lab feature can be placed on the correct side of the fence
without a scope argument.** I tested that against all 18 items. The result is a finding, and per
the brief I am reporting it rather than widening the fence.

### 2.1 The fence is sound — within a domain narrower than the feature set

The fence — *lab may produce anything; the workhorse accepts only what generalizes across a
batch* — is **derived, not asserted**, and it is genuinely good at what it does. It cleanly cuts
masks, the item it was derived from: absolute rect → lab, percent/gravity rect → workhorse; seed
flood coordinate → lab, luminance threshold → workhorse.

But note what it is actually testing. It is a test on a **parameter**: *does this value still mean
something when applied to a different image?* It is not a test on a **feature**. And 13 of the 18
items are not ops at all — they are lints, sinks, codecs, interfaces, test harnesses, and a bot.
For those the fence returns "not applicable", not a side.

### 2.2 Three items are placed on the *workhorse* side — contradicting the triage's own assignment

This is stronger than "resists placement," and it is the finding that matters.

| item | what the fence says | where the triage puts it | verdict |
|---|---|---|---|
| **§8 expression filters** | A per-pixel expression like `r*0.5+0.2` is size-independent and means exactly the same thing on every image. **Generalizes → workhorse.** | lab | **contradiction** |
| **§9 parameter sweeps** | Sweeping quality 60→90 is meaningful on any image; the workhorse already does a quality search (`src/quality/mod.rs:255`). **Generalizes → workhorse.** | lab | **contradiction** |
| **§10 watch-preview** | Watching a file and re-running generalizes fine — `build --watch` **already ships in the workhorse** (`watch` feature, default-on). | lab | **contradiction** |

These three are lab-side for real reasons — unbounded evaluation surface, a human-facing contact
sheet, an interactive loop — but **none of those reasons is "it doesn't generalize."** §8 is the
clearest tell: its headline design, *bake to `.cube`*, exists precisely to move it across the
fence, which is an admission that the fence was never what excluded it.

### 2.3 The handed RAW split has the same problem

The brief states RAW's per-image params are *"WB eyedropper (literally a pixel coordinate),
exposure compensation, crop."* The eyedropper is a textbook fence hit — a coordinate, correctly
lab. But:

- **Exposure compensation is a scalar EV value.** `+0.5 EV` is perfectly meaningful applied to a
  thousand images. By the fence it is **workhorse**. (The counter-argument — "the right value
  differs per image" — proves too much: the right *quality* differs per image too, and quality is
  a workhorse parameter the engine searches for.)
- **Crop splits**, exactly as masks do: an absolute rect is lab, a percent/gravity crop is
  workhorse — which is what `Gravity`'s own doc anticipates (`src/operation/mod.rs:641`).

### 2.4 Recommendation: two fences, not a wider one

Do **not** widen the fence. Sharpen it and add an orthogonal one for the non-op surface.

> **Fence A — parameter admissibility (restates DEC-088, does not change it).**
> A parameter is workhorse-admissible if its *meaning* is invariant under substituting a different
> image. Absolute pixel coordinates are not. Ratios, percentages, gravities, and value-space
> thresholds (luminance / chroma / alpha) are. *Whether the best value differs per image is
> irrelevant — that is what a search is for.*

> **Fence B — surface admissibility (new; covers what A cannot reach).**
> The workhorse emits **artifacts a build consumes**. Lab emits **decisions a human acts on** —
> and every lab decision must be expressible as a workhorse artifact (§1).

Fence B is what actually places sweeps (a contact sheet is information), watch-preview (a loop is
information), a TUI, and window display. It is also derived rather than asserted: it falls out of
the thesis in §1.

### 2.5 The 18 items placed

| # | item | Fence A (params) | Fence B (surface) | home |
|---|---|---|---|---|
| §1 | placeholders + dominant colour | n/a — not an op | build artifact | **workhorse** (Wave 4, existing home) |
| §2 | LUT op (`.cube`) | LUT file path + strength: generalizes | build artifact | **workhorse** — and its cache-key story is the differentiator |
| §3 | perceptual dedup lint | n/a | build gate | **workhorse** (`lint`) |
| §4 | lint in wasm | n/a | build target | **workhorse** |
| §5 | threading | n/a | implementation | **workhorse** (premise false — see triage §1.1) |
| §6 | cross-verb invariant harness | n/a | test infra | **workhorse** (STAGE-042) |
| §7 | named composable masks | **splits**: percent/gravity/luma → workhorse; absolute rect + seed flood → lab | — | **both** — the fence's founding case, and it works |
| §8 | expression filters | generalizes → workhorse | **unbounded eval surface → lab** | **lab authors, workhorse consumes the baked `.cube`** |
| §9 | parameter sweeps | generalizes → workhorse | **contact sheet = information → lab** | **lab** (by Fence B only) |
| §10 | watch-preview | generalizes (already ships) | **interactive loop → lab** | **lab** (by Fence B only) |
| §11 | animated GIF → WebP/AVIF | n/a | codec | **workhorse** |
| §12 | ICC transforms (`qcms`) | colour space name generalizes | build artifact | **workhorse** |
| §13 | autofix bot | n/a | separate product | **neither** — own trust model |
| §14 | window display | n/a | information → lab | **lab, but firm-tier blocked** (X11/Wayland) |
| §15 | SVG optimization | n/a | build artifact | **workhorse** |
| §16 | brand consistency gate | n/a | build gate | **workhorse** |
| §17 | MCP measurement server | n/a | interface over existing `--json` | **workhorse** |
| §18 | external tools | n/a | n/a | **not built** (DEC-088 decision 2) |
| — | JPEG XL decode | n/a | codec | **workhorse** |
| — | favicon / OG sets | n/a | build artifact | **workhorse** |
| — | declared convolution kernels | kernel matrix generalizes | build artifact | **workhorse** |
| — | entropy smart crop | generalizes (derived, not absolute) | build artifact | **workhorse** |
| — | op stack with undo | n/a | session state → lab | **lab** |

**Read the column, not the rows:** under both fences honestly applied, **the lab-only set is
small** — masks' absolute half, sweeps, watch-preview, undo stack, and expression *authoring*.
Most of the 18 items are workhorse features that were parked behind "the lab decision" because
lab was the open question, not because they belong to lab. That is good news for sequencing and
bad news for anyone expecting lab to be the big wave.

### 2.6 What to do about DEC-088

Amend, do not reverse — the DEC-087 pattern this repo already uses. A short **DEC-089** recording:
Fence A restated at parameter level; Fence B added for non-op surface; the three contradictions in
§2.2 named as the evidence; and the RAW exposure-compensation misplacement corrected. DEC-088's
confidence (0.86) is about right and should not move much: its mechanism verified (F1), its scope
test needed a companion.

---

## 3. Interaction model (Q3)

**Recommendation: (a) the same recipe model, plus one loop. Not a REPL. Not a TUI in v1.**

The thesis decides this. If lab's product is a recipe, then **session state that is not in the
recipe is state the workhorse cannot replay** — a stateful editor is not merely unnecessary, it is
in tension with the one invariant that makes lab testable (§4, Tier 0).

So the interaction model already exists in embryo: `edit --save-recipe` writes the op chain, and
`tests/edit.rs:216` proves the round-trip. Lab's loop is that, tightened:

```
edit recipe.toml  →  lab watch --recipe recipe.toml photo.jpg  →  view in terminal  →  repeat
                                                              ↘  crustyimg apply --recipe recipe.toml shoot/
```

**Is terminal feedback sufficient?** For iterating a *recipe*, largely yes — `view` already covers
kitty graphics, iTerm2, sixel, and a block fallback, and the maintainer's own terminal is a
first-class target. It is **not** sufficient for judging fine grading work (banding, halos, subtle
colour), and no terminal protocol fixes that.

**The smallest thing that beats it** is not a GUI — it is *removing the round-trip*, i.e. a watch
loop that re-renders on file save. And that is nearly free: `notify` is already a dependency behind
the default-on `watch` feature (`Cargo.toml:210`), and `src/build/watch.rs` already implements the
debounced loop. **Zero new dependencies.**

**Defer the TUI, with the door open.** `ratatui` probed healthy — v0.30.2, MIT, 44,005,259
downloads, updated 2026-06-19 — and `crossterm` 0.29.0 is *already in the lock file*, pulled by
`viuer` behind the default-on `display` feature. So a ratatui surface costs one permissive
top-level dep and no new transitive tail. But build it only on **evidence the recipe loop is
insufficient**, gathered by using it. territory.md already welcomes "the ratatui recipe editor";
that is permission, not a requirement to spend it now.

---

## 4. THE CRUX — what "correct" means for lab, and how it is tested (Q4)

The brief is right that this decides tractability. **The project is tractable, and the reason is
the thesis.**

### 4.1 The reframe

"An editor's output is judgment-bound" is true of *pictures*. Under §1, lab's output is a
**recipe**. So split the question:

- **Is lab correct?** → Does the recipe it emits do what lab showed, and do its ops obey their
  stated algebra? **Fully mechanical.**
- **Is lab's output good?** → Does the grade look right? **The user's job, explicitly not tested.**

Nearly all of lab's surface falls on the mechanical side once the thesis is enforced. Where a
feature does not, that is a signal the feature is wrong (see §4.3).

### 4.2 The oracle ladder — four tiers, every one with existing repo precedent

**Tier 0 — Replay equivalence. The load-bearing invariant.**
> The recipe lab emits, executed by the workhorse, produces bytes identical to what lab previewed.

No judgment, no tolerance, no fixture curation. **Precedent: `tests/edit.rs:216`** already asserts
exactly this shape for the three-flag `edit`. This is lab's constitution, and it is the gate every
lab op must pass before it can be considered built. It also catches the whole class of bugs that
a two-binary split invites: divergent defaults, op-order drift, a lab-only param the workhorse
silently ignores.

**Tier 1 — Algebraic invariants.** Hand-rolled property tests over natively generated fixtures
(AGENTS.md §12 already forbids shelling out for fixtures; `proptest` is optional and not needed
for v1):
- **identity at neutral params** — `posterize(256)`, an all-zero mask, an identity `.cube`, and
  `exposure(0 EV)` must each be a byte-exact no-op. Cheapest, highest-yield test in the set.
- **idempotence where claimed** — `auto-orient` already asserts this (`src/operation/mod.rs:919-922`).
- **commutativity where claimed — and non-commutativity where that is the contract.** Assert the
  negative too, or the test passes on a pipeline that silently reorders.
- **param round-trip** — `from_ops → to_toml → from_toml` yields the same ops; the existing
  `tests/recipe_round_trip.rs` pattern extended to lab ops.
- **bounds** — output dimensions, alpha coverage, value range stay in domain.

**Tier 2 — Digest goldens on decoded RGBA, not encoded bytes.**
**Precedent: `src/image/avif.rs:767-793`** — a committed FNV-1a `u64` over the decoded RGBA buffer.
This is golden-image regression **without** committing image files, without `insta`, and without
cross-OS encoder flakiness (decode is deterministic; encode is not guaranteed to be).

Discipline, and it is the part that usually rots: **the digest must be captured from a source
independent of the code under test** — a reference implementation, a hand-computed expected
buffer, or the pre-change binary — exactly as that AVIF test's comment insists (*"an independent
value the code under test cannot fabricate"*). A digest captured by running the new op proves only
that the op is deterministic.

**Tier 3 — SSIMULACRA2 as a null-hypothesis oracle, never as a quality judge.**
`quality::score(&DynamicImage, &DynamicImage) -> f64` is already public
(`src/quality/mod.rs:99`). Use it for **bounded, directional** claims only:
- an all-zero mask scores ≥ 99.9 against the input (the op did *nothing*);
- `posterize(2)` scores below a floor (the op did *something* — guards against a silently
  no-op'd op, the failure mode a green test suite is worst at catching);
- **bake fidelity** — a `.cube` baked from an expression must score ≥ *N* against the expression
  evaluated directly. This one is genuinely load-bearing; see §4.3.

Never `assert!(score > 80)` as a stand-in for "looks good."

### 4.3 The one genuinely untestable feature — and the design that fixes it

**§8 expression filters have no oracle.** An arbitrary user expression's *result* cannot be
checked; there is nothing to compare it to. Left as-is, it is exactly the "untestable surface" the
brief warns will accumulate.

But its **bake** is testable, by Tier 3. So:

> **Recommendation: make bake-to-`.cube` the only path by which an expression reaches an output.**
> Lab evaluates the expression solely to *build a LUT*; the LUT is what renders, and the LUT is
> what the workhorse consumes.

This converts an untestable feature into a testable one, and it pays three other ways the triage
already identified: the main binary never needs an evaluator, wasm stays small, and the cache key
just hashes a file. It also means the *only* new correctness question is "does the baked LUT match
the expression," which is one bounded SSIMULACRA2 assertion plus a lattice-corner exactness check.

### 4.4 What is explicitly not tested — write this down as an anti-goal

Aesthetic quality. Whether a grade is pleasing, whether a mask is "right", whether a crop is
well-composed. Lab does not assert taste. Naming this now is what stops a future session from
inventing a subjective assertion and calling it coverage.

### 4.5 Verdict

**Tractable.** Tier 0 alone covers most of lab's surface mechanically, and it already exists in
miniature. The gating rule that keeps it true is in §10: **a lab spec that cannot name its oracle
tier does not get built.**

---

## 5. What the shared core needs (Q5)

| change | kind | count | evidence |
|---|---|---|---|
| `pub` widenings | visibility | **0** | probe compiles; negative control E0603 (F1) |
| re-exports (`::image`, `::toml`) | ergonomics **+ safety** | **2** | first probe E0433; feature-unification measurement (F2) |
| `cli` feature gating `clap`/`clap_complete`/`indicatif` | feature | **1** | `Cargo.toml:168,169,183` — unconditional in the native table, as DEC-088 said |

**Verdict against DEC-088's "wrong if":** *"if the shared core needs constant `pub` widening for
lab, that is evidence the seam is in the wrong place."* Measured at **zero**. The seam is in the
right place, and this is the plan's most load-bearing verified premise.

**The honest caveat.** Zero is the count *for the surface I probed* — ops, registry, recipe,
pipeline, encode, quality, analysis, mask. A lab op that needs the decide path
(`src/analysis/decide.rs`), metadata write internals, or `cli` internals would need more. So make
it a measured gate rather than a one-time claim:

> **Checkpoint:** re-run the widening probe at every lab stage ship. If the cumulative count
> exceeds **5**, re-open DEC-088 rather than continuing to widen.

One further note: `required-features` on `[[bin]]` was flagged unverified in triage §4. It is
**moot under the recommended packaging** — lab is its own crate depending on `crustyimg`, and
Cargo does not build a dependency's binary target (DEC-088 established this). No `[[bin]]`
gymnastics are needed.

---

## 6. What already exists that changes the cost (Q6)

| capability | already present? | evidence | cost delta |
|---|---|---|---|
| external op registration | **yes, verified** | probe exit 0 + E0603 control | the hardest-sounding piece is **done** |
| recipe superset mechanism | **yes, verified** | probe: `from_toml`→`build_pipeline`→`run` | done |
| mask substrate (paths, PNG, compose, invert) | **yes** | `resvg::tiny_skia::Mask`, imported at `src/image/svg.rs:55` | **§7 substrate free** |
| SVG parsing for path masks | **yes** | `usvg` 0.47.0 in lock | free |
| debounced watch loop | **yes** | `src/build/watch.rs`, `notify` @ `Cargo.toml:210` | **§10 nearly free** |
| terminal preview | **yes** | `viuer` behind default-on `display` | free |
| perceptual oracle | **yes, public** | `src/quality/mod.rs:99` | free |
| parameter search harness | **yes** | `search_quality` / `search_under_size` (`src/quality/mod.rs:255,267`) | **§9 sweep engine largely free** |
| image feature analysis | **yes, public** | `src/analysis/` (`Analysis::compute`, probed) | free |
| crossterm (if a TUI ever lands) | **yes** | 0.29.0 in lock via `viuer` | ratatui = 1 dep, no new tail |
| replay-equivalence oracle | **yes, in miniature** | `tests/edit.rs:216` | **Tier 0 prototyped** |
| RGBA digest golden pattern | **yes** | `src/image/avif.rs:767-793` | Tier 2 prototyped |
| **`crop` op** | **no** | no `Operation` impl; `Gravity` doc says "future" | must be built |
| **any colour op beyond invert** | **no** | 5 `impl Operation for` total (F3) | must be built |
| golden-image / property test infra | **no** | no insta/proptest/quickcheck (F7) | build on Tiers 1–2 |

---

## 7. Dependencies (Q7)

Probed against the crates.io API on 2026-08-15.

| crate | version | licence | downloads | last publish | verdict |
|---|---|---|---|---|---|
| `ratatui` | 0.30.2 | MIT | 44,005,259 | 2026-06-19 | healthy; **defer** (§3) |
| `image_hasher` | 3.1.1 | MIT OR Apache-2.0 | 821,062 | 2026-02-21 | healthy — but §3 is workhorse |
| `logos` | 0.16.1 | MIT OR Apache-2.0 | 69,093,222 | 2026-01-30 | healthy; only if §8 needs a lexer |
| `oxipng` | 10.2.0 | MIT | 1,775,957 | 2026-08-09 | healthy — workhorse, not lab |
| `zopfli` | 0.8.3 | Apache-2.0 | 104,065,268 | 2025-10-30 | healthy — workhorse |
| `lut-cube` | 0.2.0 | **MIT** (via `license-file`) | 1,969 | 2025-02-22 | **decline** — F6 |
| `wagahai_lut` | 0.1.0 | MIT | 544 | 2026-01-27 | **decline** — one release |
| `show-image` | 0.14.1 | BSD-2-Clause | 270,116 | 2025-02-23 | permissive but **firm-tier blocked** (X11/Wayland) |
| `rustyline` | 18.0.1 | MIT | 45,221,223 | 2026-06-24 | healthy — but a REPL is not recommended (§3) |
| `reedline` | 0.50.0 | MIT | 2,998,972 | 2026-08-15 | same |

### The recommendation: **lab v1 adds zero new top-level dependencies.**

That is not aspiration — it follows from F5 (masks free via `resvg::tiny_skia`), the existing
watch loop, the existing quality/search/analysis modules, and the decision to in-house `.cube`.
It keeps `no-agpl-default-deps` and `single-image-library` trivially satisfied, keeps `just deny`
green with no new exception, and means lab's first release carries **no new supply-chain surface
at all** — a genuinely strong claim for a second binary, and one worth stating publicly.

`no-new-top-level-deps-without-decision` is therefore not triggered in v1.

---

## 8. Anti-goals — what lab is NOT

Named so later decisions are cheap.

1. **Not an interactive GUI.** Unchanged from territory.md's firm tier.
2. **Not a window backend.** X11/Wayland is the heavy system dependency the firm tier calls
   non-negotiable. `view` covers the terminal case.
3. **Not a delegate/external-process host.** DEC-088 decision 2. Do not reopen.
4. **Not a taste oracle.** Lab does not assert whether output looks good (§4.4).
5. **Not stateful beyond the recipe.** Session state the recipe cannot express is state the
   workhorse cannot replay (§3).
6. **Not safe on untrusted input by inheritance.** DEC-088 is explicit. Lab states its own posture,
   and F2 shows the concrete way it could silently be *less* safe than the workhorse.
7. **Not a second pixel library, ever.** `single-image-library` is blocking, and F2 shows lab can
   breach its spirit through feature unification without adding a crate at all.
8. **Not a home for workhorse features that were merely blocked on the lab decision.** §2.5 shows
   most of the 18 items are workhorse work. Do not let lab absorb them.

---

## 9. Stage plan

Sized against F3/F4 (the base is four ops, not an editor), not against the triage's seven items.

| stage | delivers | oracle tier | new deps | gate to exit |
|---|---|---|---|---|
| **L-00 Seam hardening** *(lands in `crustyimg`, not lab)* | `pub use ::image; pub use ::toml;`; the `cli` feature; a committed out-of-crate probe test asserting the seam + widening count | Tier 1 | 0 | probe test in CI with its negative control |
| **L-01 Lab skeleton** | second crate, its own binary, depends on `crustyimg`; `lab apply` runs a recipe; lab-only op reaching the workhorse exits **4** naming the op; lab's own SECURITY posture written | Tier 0 | 0 | Tier 0 green on a workhorse recipe; exit-4 driven, not asserted |
| **L-02 Masks** | `tiny_skia::Mask` substrate; luminance/chroma/alpha + percent/gravity (workhorse-admissible) and absolute rect + PNG/SVG masks (lab-only); the Fence-A split enforced in code | Tier 0+1+2 | 0 | all-zero-mask identity is byte-exact; the workhorse-side half round-trips into a `build` recipe |
| **L-03 Watch loop** | `lab watch` — re-render + `view` on recipe change, reusing `src/build/watch.rs` | Tier 0 | 0 | recipe emitted mid-loop replays byte-identically |
| **L-04 Sweeps** | parameter sweep + contact sheet, over the existing search harness; emits the winning **recipe**, not just a sheet | Tier 0+1 | 0 | the emitted recipe reproduces the chosen cell byte-for-byte |
| **L-05 `.cube` LUT** *(workhorse-side op; lab is the authoring surface)* | in-house parser + trilinear interp (**budget 250–350 lines**, F6); LUT hash joins the build cache key | Tier 1+2+3 | 0 | identity cube is a byte-exact no-op; lattice corners exact |
| **L-06 Expressions → bake only** | expression authoring in lab; **the only output is a `.cube`** (§4.3) | Tier 3 | 0–1 (`logos`, only if needed) | bake fidelity ≥ threshold vs direct evaluation |

**Sequencing against the repo.** Lab is downstream of PROJ-010 closing. Current state, read from
the stage files: STAGE-044 **shipped**; STAGE-042, 043, 045 **active**; STAGE-041 **proposed**; no
open PRs. **Do not start L-00 until 042/043/045 land** — L-00 touches `src/lib.rs` and the feature
set, and STAGE-042's conformance matrix is exactly what would catch a regression there. Doing it
first would mean editing the core while its safety instrument is still being built.

**Spike before L-05/L-06:** the colour-space question (F8, §12).

---

## 10. Automating the build with quality checkpoints

You asked for this specifically, and crustyimg has paid for the lessons. Concretely, for the
forked repo's prompt set:

**Structural rules**

1. **Orchestrator does not build.** Design and ship in the main session; hand build and verify to
   separate CLI sessions via a written prompt. Run them foreground — background subagents do not
   get Bash.
2. **Checkpoint commits are mandatory.** SPEC-113's build ran ~3h and ~$40 with **zero commits**.
   Every build prompt must say: *push a WIP commit as soon as it compiles, before the matrix.*
3. **One worktree per concurrent session**, and one `CARGO_TARGET_DIR` per feature leg. Never both
   shared and parallel — differently-featured builds corrupt a shared target dir.
4. **The orchestrator re-runs a clean full-matrix verify.** A stale incremental build is a false
   green; the builder's own green does not count.

**Evidence rules — these are what actually enforce quality**

5. **Never read an exit code through a pipe.** `cargo test | tail` turns a red leg green. Redirect
   to a file and read `$?`. *I hit this in this very session* — the first seam probe reported exit
   0 while failing to compile.
6. **Every acceptance criterion needs a negative control.** Prove the test can go red. The seam
   probe in §F1 is the model: the E0603 control is what makes its green mean anything.
7. **A fixture generated by the code under test cannot fail.** Tier 2 digests must come from an
   independent source (§4.2).
8. **Cite the grep, and treat its scope as a claim too.** Mechanical sweeps need mechanical checks;
   cross-check counted lists with `/usr/bin/grep` and a positive control.

**Lab-specific gates — the new ones**

9. **Every lab spec names its oracle tier (0–3) in its acceptance criteria. A spec that cannot
   name one does not get built.** This is the single rule that keeps §4's answer true over time.
10. **Tier 0 is a release gate, not a spec gate.** No lab release ships with any op lacking a
    passing replay-equivalence test.
11. **The widening probe runs at every stage ship**, and its count goes in the stage's completion
    table. Cumulative > 5 re-opens DEC-088 (§5).
12. **Diff the completion table against the acceptance list at verify.** A criterion nobody claims
    is a criterion nobody checks.

**Model split:** Sonnet for mechanical build cycles, Opus for verify — measured on this repo,
Sonnet loses only on sweep thoroughness, which is exactly what verify is for. Capture real
`tokens_total` per metered cycle (AGENTS.md §4); `just cost-audit` fails a ship without it.

---

## 11. Pre-mortem — how this fails

Ordered by likelihood. As in the RAW session, **the top risk is not technical.**

1. **Lab has no user but the maintainer.** The workhorse's territory is won on *automatic, batch,
   CI*. Lab is manual and single-image — the opposite axis. It is entirely possible to build all
   six stages well and have nobody, including the maintainer, reach for it twice. **Mitigation:**
   L-01 through L-03 must be dogfooded on the Eleventy photo blog before L-04 is framed. If the
   maintainer does not reach for lab unprompted after L-03, stop and reconsider.
2. **The fence contradiction erodes into "everything is lab."** §2.2 shows three items already sit
   on the wrong side of the stated fence. If DEC-089 is not written, the next session resolves each
   case by argument, which is precisely how an anti-goal erodes silently. **Mitigation:** write
   DEC-089 before the first lab spec, exactly as the triage said about DEC-088.
3. **The 8-bit sRGB pipeline makes lab's grading output wrong invisibly (F8).** Lab's own tests
   cannot detect it — reference and candidate are wrong identically. **Mitigation:** the §12 spike,
   before L-05.
4. **Two binaries double the release surface.** Conformance matrix, three-OS CI, Homebrew,
   crates.io, cargo-dist, the npm size claim. The engineering cost of the *split* may exceed the
   cost of the ops. **Mitigation:** L-01's exit criteria include a full release dry-run, not just a
   build.
5. **Lab silently becomes less safe than the workhorse.** F2 is a live path to this today, and no
   existing test would catch it. **Mitigation:** L-00's re-exports plus a lab constraint forbidding
   a direct `image` declaration, with a test that asserts the resolved feature set.
6. **Scope absorption.** §2.5 shows most of the 18 items are workhorse work parked behind the lab
   question. Lab is an attractive place to put them and the wrong one. **Mitigation:** anti-goal 8.

---

## 12. Pre-registered spike questions — answer before any code

Each must be answered with a measurement, not a judgement.

1. **Colour space (blocks L-05/L-06).** Does SSIMULACRA2 score a linear-light downscale *better*
   than the current sRGB-space one on a representative image? `docs/backlog.md:624` says measure
   this first, and lab makes it load-bearing rather than nice-to-have. **If linear-light does not
   win, the premise is wrong and both the backlog item and lab's grading ops need rethinking.**
   Related: does the build cache key need a colour-pipeline-version component so old and new
   renders cannot collide?
2. **Does a lab-only op actually exit 4 in the workhorse?** DEC-088 asserts it; nothing implements
   it. Drive the failure path — a claimed failure mode is as unproven as a claimed success.
3. **Does Tier 0 hold across encoders?** `tests/edit.rs:216` proves byte-identity for PNG. Does it
   hold for AVIF and lossy WebP, whose encoders may not be bit-reproducible across platforms? If
   not, Tier 0 needs a decoded-RGBA-digest form (Tier 2) for those formats — **decide before L-01,
   not during.**
4. **What does `tiny_skia::Mask::fill_path` cost on a 24MP image?** F5 says the substrate is free;
   free to *link* is not free to *run*. Measure.
5. **Does the `cli` feature actually shrink anything?** Measure the lab binary with and without.
   If `clap` is pulled in anyway by some other path, the feature is churn.
6. **How many lines is the in-house `.cube` reader, really?** F6 budgets 250–350 against a measured
   comparable. Confirm at L-05 design and record the delta — the repo's in-housing precedent
   (`little_exif` → 718-line `tiff.rs`) is only credible if the estimates keep proving honest.

---

## Appendix — reproducing the probes

All probes ran outside the repo, in a scratch crate with its own `CARGO_TARGET_DIR`. Nothing in
`crustyimg` was modified.

```
labprobe/Cargo.toml   crustyimg = { path = ... }, image (default-features = false), toml, resvg
labprobe/src/main.rs  implements Operation out-of-crate; registers; from_toml → build_pipeline →
                      run; from_ops → to_toml; encode_to_bytes; quality::score;
                      Analysis::compute; tiny_skia::Mask::new
```

- **Seam probe:** `cargo check` → exit 0.
- **Negative control:** add `crustyimg::cli::ops::run_edit` → `error[E0603]`, exit 101.
- **Feature unification:** `cargo tree -e features -i image@0.25.10`, defaults off vs on →
  10-feature delta.
- **Crate probes:** crates.io API, 2026-08-15.
- **`lut-cube` measurement:** `static.crates.io` tarball, `wc -l` → 329 lines; `LICENSE` read
  directly.
