---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-092
  type: decision
  confidence: 0.88
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5
  session_id: 7ae3add2-8f00-4435-a9b4-714576c628d8

project:
  id: PROJ-010
repo:
  id: crustyimg

created_at: 2026-08-16
supersedes: null
superseded_by: null

# Path globs this decision governs. The verdict is *proceed*, so the resize op
# is in scope; the two harness files are listed so a later change to the
# measurement surfaces this record too.
affected_scope:
  - "src/operation/**"
  - "scripts/spec120_linear_light.py"
  - "examples/spec120_linear_probe.rs"

tags:
  - resize
  - linear-light
  - gamma
  - ssimulacra2
  - premultiplied-alpha
  - measurement
  - falsification-gate
---

# DEC-092: the linear-light premise holds — measured; the premultiplied-alpha half of the same entry does not

## Decision

**The linear-light premise holds. Spec the fix.** `Resize::apply` resamples non-linear sRGB
values as if they were linear, and against an **independent** linear-light reference the shipped
downscale measures worse on **both** oracles, on **every** case tried — the perceptual metric the
repo already owns (SSIMULACRA2, DEC-019) and the physical quantity the premise actually names
(mean linear luminance). STAGE-046 should carry a fix spec against `src/operation/mod.rs`'s
resize path.

Two sub-decisions fall out of the same measurement:

1. **SSIMULACRA2 is a valid gate for this question — proven, not assumed.** A positive control
   (thin bright hairlines on black, downscaled 8×) produces a physical error of −88.07% of the
   reference's mean linear luminance, and the metric registers it as a **163.85-point** swing
   (−63.85 for the shipped path vs 100.00 for a linear-light prototype). The instrument can see
   gamma-incorrect resampling, so the realistic-case numbers mean what they say. Had it not
   fired, the verdict would have been *"wrong instrument"*, not *"premise false"* — the two
   readings lead to opposite decisions, which is why the control came first.

2. **The premultiplied-alpha half of the same backlog entry is FALSE, and is closed here.**
   `fast_image_resize` 6.0.0's `ResizeOptions::default()` sets `mul_div_alpha: true`
   (`src/resizer.rs:41-59`), and `ResizeOptions::new()` *is* `Default::default()`, so
   `Resize::apply`'s `ResizeOptions::new().resize_alg(...)` — which overrides only the algorithm —
   **already premultiplies alpha**. Measured against an independent premultiplied reference the
   shipped path shows a max premultiplied-RGB edge error of **27/255 (mean 0.364/255)**, while
   the same code with `use_alpha(false)` measures **68/255 (mean 18.34/255)** — 50× worse on the
   mean. **The fix spec must not carry the alpha half.** The backlog entry flagged this claim as
   "Not confirmed — only two files were grepped"; the grep was of the wrong files. The behaviour
   lives in the dependency's *default*, where no grep of `src/` could find it.

## Context

`docs/backlog.md`'s linear-light entry set its own falsification gate — *"does SSIMULACRA2 score
the linear-light output better than the current output on a representative downscale? … Measure
that first."* SPEC-120 is that measurement, and it ships no behaviour: `git diff` against `main`
shows no functional change to `src/`.

### The experiment had to be reshaped before it could be run

SSIMULACRA2 requires equal dimensions (`src/cli/report.rs:329`), so the obvious form of the gate —
score the 256px output against the 2048px source — **errors rather than answering**. The runnable
shape supplies a reference at the *target* size and scores both candidates against it:

```
source ─┬─► crustyimg today   (sRGB U8x4 Lanczos3 — the shipped binary)
        ├─► prototype         (linear f32 Lanczos3 — same crate, same filter)
        └─► REFERENCE         (ImageMagick, explicit linear colorspace)
```

The reference is generated **outside this codebase** on purpose: a reference produced by the code
under test cannot fail the code under test. Tool and version:
**ImageMagick 7.1.2-29 Q16-HDRI aarch64 b919b37fd:20260727**, invoked as
`magick SRC -colorspace RGB -filter Lanczos -resize 'WxH!' -colorspace sRGB -depth 8 -strip DST`.
The Q16-HDRI build carries the intermediate in float, so nothing is quantized between the two
colorspace conversions. Measured with `crustyimg 0.7.0` and `fast_image_resize 6.0.0`.

### The numbers

Reference = the independent linear-light downscale. Luminance is BT.709 relative luminance
computed on **linearized** channel values; the signed mean is the direction the premise predicts
(negative = darker).

| case | source → target | today: mean signed luma err | as % of ref | today SSIMULACRA2 | prototype SSIMULACRA2 | Δ |
|------|-----------------|----------------------------:|------------:|------------------:|----------------------:|----:|
| synthetic worst case (positive control) | 2048² → 256² (8×) | −0.104350 | **−88.07%** | **−63.85** | 100.00 | **+163.85** |
| `graphic_large.png` | 512² → 128² (4×) | −0.001386 | −0.44% | **70.45** | 100.00 | **+29.55** |
| `photo_forest_cc0.jpg` | 800×532 → 200×133 (4×) | −0.004920 | −2.63% | **84.45** | 99.41 | **+14.96** |

Every candidate is darker than the reference, on every case, exactly as the premise predicts.

**Alpha (its own oracle, not SSIMULACRA2).** A 1024² hard-edged opaque shape over a
fully-transparent surround carrying maximally contrasting RGB ("dirty alpha", the classic halo
trigger), downscaled 8× to 128². Both sides resampled in sRGB space so the only variable is
premultiplication. Over the 6301-pixel anti-aliased edge band (any pixel where either image has
`0 < alpha < 255`), the maximum per-channel difference in **premultiplied** 8-bit RGB — which is
the visible composite error, and is background-independent when the two alphas agree:

| arm | max premul RGB err | mean premul RGB err |
|-----|-------------------:|--------------------:|
| **crustyimg today** | **27 / 255** | **0.364 / 255** |
| control: same code, `use_alpha(false)` | 68 / 255 | 18.336 / 255 |

The residual 27 is not a halo: the two implementations' alpha channels disagree by 0.42/255 on
average with a 27/255 peak, and the peak premultiplied error tracks it. ⚠ **The mechanism this
record originally named for that 27 — "Lanczos ringing at hard corners" — is refuted. See
*Amended 2026-08-20* under Consequences; the sentence is left standing rather than rewritten so
the correction is visible, but do not carry the ringing explanation forward.**
`max_straight_rgb_err` is 255 for **both** arms — a reminder that unassociated RGB is
meaningless where alpha ≈ 0, which is why the premultiplied form is the correct oracle here.

### The two metrics agree in direction but are not interchangeable

Worth carrying into the fix spec: on `graphic_large.png` the *mean* luminance error is only
−0.44% while the perceptual penalty is 29.5 points. The physical error is concentrated at edges —
max local |luma err| **0.213** against a mean absolute of 0.0023, ~90×. **Mean luminance
understates the defect on graphics**, which is precisely the content class the premise says is
worst hit. A fix spec that gates on mean luminance alone would under-report its own win.

## Alternatives Considered

- **Score the downscale against its source (rejected — not runnable).** SSIMULACRA2 requires equal
  dimensions; this errors rather than returning a number. Settled at design, restated here because
  it is the form the backlog's gate was written in.

- **Generate the reference with `fast_image_resize` under different flags, or with crustyimg
  itself (rejected).** Both are the code under test. The whole validity of the measurement rests
  on the reference being independent, which is why an outside tool was used even though AGENTS §12
  forbids ImageMagick for *test fixtures* — fixtures must be hermetic; a one-off measurement
  harness must be independent. Different rule, different reason.

- **Trust a null result without a positive control (rejected — this is what the spec exists to
  prevent).** A null has two readings, *premise false* and *instrument wrong*, and they lead to
  opposite decisions. In the event the control fired hugely, so this branch was not taken — but it
  was proven to be capable of firing before any other number was believed.

- **Route the alpha half through SSIMULACRA2 (rejected).** SSIMULACRA2 consumes 8-bit sRGB via
  `to_rgb8()` (`src/quality/mod.rs:68`) and never sees the alpha channel, so it is structurally
  incapable of measuring a transparent-edge halo. The premultiplied-edge oracle above is the
  correct instrument, and it was likewise proven able to fire (the `use_alpha(false)` control)
  before its near-null on the shipped path was believed.

- **Add the synthetic worst case to `bench/corpus/` (rejected).** That directory is what
  `just bench` measures and what `bench_corpus_is_license_clean` scans; adding a file would change
  published bench output for an unrelated reason. The synthetic is generated deterministically by
  the committed harness instead, which is strictly better for re-derivation.

## Consequences

- **STAGE-046 gains a fix spec** for `Resize::apply`: linearize to `f32`, resample, re-encode to
  8-bit sRGB on the way out. `fast_image_resize` 6.0.0 already supports `F32x4`, and the prototype
  confirms the backend handles it — no new dependency, no 16-bit pipeline required.
- **The fix spec is one premise, not two.** Premultiplied alpha is already correct; carrying it
  would be scope invented from an unconfirmed claim.
- **It is still a breaking change.** Fixing the resampling changes output bytes for every existing
  recipe and invalidates every PROJ-007 build lockfile. That consequence is unchanged by this
  measurement — it is now a *justified* breaking change rather than a speculative one. The open
  sub-question stands: does the build cache key need a colour-pipeline-version component so old
  and new renders cannot collide?
- **DEC-019 is confirmed, not narrowed.** SSIMULACRA2 remains the repo's perceptual oracle, and is
  now measured to be sensitive to a defect class outside the compression artifacts it was tuned
  for. That sensitivity is established for *this* defect only; it is not a general licence.
- **The harness stays committed** (`scripts/spec120_linear_light.py`,
  `examples/spec120_linear_probe.rs`) so the fix spec can re-derive these numbers as its own
  before/after, rather than trusting this record.
- **Prototype ≠ production.** The prototype scores ~100 against the reference partly because it
  implements the *same* algorithm as the reference. The load-bearing measurement is
  **crustyimg-today's score against a correct reference** (−63.85 / 70.45 / 84.45), not the exact
  magnitude of the delta. A production linear-light resize will not necessarily score 100.

### Amended 2026-08-20 — the residual 27 is 8-bit quantization, not Lanczos ringing

**SPEC-122 fixed the resampling and the residual went to zero, which refutes the mechanism this
record named for it.** The record read the 27/255 alpha peak as *"Lanczos ringing at hard
corners"*. It is not ringing, and it is not premultiplication either — it is **8-bit quantization
in the integer resampling path, the alpha channel's own convolution included.**

Three pieces of evidence, all re-derived on the same harness rather than taken from the fix's
write-up:

| arm | max premul RGB err | max **alpha** err | mean alpha err |
|---|---:|---:|---:|
| `main` — `U8x4`, premultiplication **ON** | 27 | **27** | 0.4203 |
| **C4** — `U8x4`, premultiplication **OFF** | 68 | **27** | 0.4203 |
| SPEC-122 — `F32x4`, premultiplication ON | 0 | **0** | 0.0000 |

- **C4 is the discriminating control.** Toggling premultiplication moves the premultiplied-RGB
  error (27 → 68) and leaves the alpha statistics **bit-identical** — same 27 peak, same 0.4203
  mean. A residual that does not move when the variable moves is not caused by that variable, so
  it is not the premultiply/divide round-trip.
- **The dependency's source says the same thing.** `fast_image_resize`'s
  `alpha::u8x4::multiply_alpha_pixel` writes `[mul_div_255(r, a), mul_div_255(g, a),
  mul_div_255(b, a), alpha]` — the alpha channel is copied through untouched, and `divide_alpha`
  is symmetric. Alpha is **never premultiplied or divided**, so its own error cannot come from
  that round-trip. Its only arithmetic is the convolution.
- **Widening the working type is what removes it.** SPEC-122 changed nothing about
  premultiplication or the filter; it moved the convolution from `U8x4` to `F32x4`, and the alpha
  error went 27 → 0 along with the RGB error. Quantization of the intermediate is the only
  variable that moved.

**The correction that first replaced the ringing claim was itself wrong** and is recorded here so
it is not re-adopted: SPEC-122's build wrote the residual up as *"8-bit quantization inside
`fast_image_resize`'s premultiply/divide round-trip"* — right in kind, wrong in the specific, and
falsified by C4 in the build's own harness output. Right in kind still matters: the fix and its
justification are unaffected, and DEC-092's verdict (proceed; premultiplication is already
correct) stands unchanged. Only the mechanism sentence moves.

`scripts/spec120_linear_light.py`'s printed footnote carried this wording too — it is where the
record's phrasing came from — and is corrected at the source in the same change, so a future
reader of the harness output is not told the refuted thing again.

## Validation

Five controls run on every invocation of the harness and are reported, not assumed:

- **C1 — the prototype reproduces the shipped binary pixel-exactly.** The prototype's sRGB arm is
  a replica of `Resize::apply`'s backend call, and its output is **pixel-identical** to
  `crustyimg resize --exact` on all three cases (`identical: true`, max RGBA err 0). So the linear
  arm's delta is attributable to the transfer function and to nothing else.
- **C2 — the reference tool's variable actually moved.** ImageMagick's own sRGB-space resize
  differs from its linear-space resize by −88.17% / −0.44% / −2.63% of mean linear luminance,
  matching the crustyimg-vs-linear gaps. `-colorspace RGB` did what it claims.
- **C3 — the reference is a fair oracle.** crustyimg today vs ImageMagick's *sRGB-space* resize:
  mean |luma err| 0.000236 / 0.000013 / 0.000344. The two independent Lanczos3 implementations
  agree, so the arm-vs-reference gaps are gamma, not filter drift.
- **C4 — the alpha oracle can fire.** `use_alpha(false)` measures 68 max / 18.34 mean against the
  premultiplied reference, 50× the shipped path's mean. Its **alpha-channel** error — 27 max /
  0.4203 mean, identical to the premultiplying arm's — is the evidence for *Amended 2026-08-20*
  above, and was in this control's output all along.
- **C5 — the shipped binary is not the non-premultiplied arm.** Its output differs from that arm
  in 10,510 pixels (max RGBA err 255), independently confirming from behaviour what the
  dependency's source says about `mul_div_alpha`.

**Reproducibility.** The harness was run twice from a clean working directory; the two reports are
byte-identical (`diff` exit 0). Re-derive with:

```sh
cargo build --release && cargo build --release --example spec120_linear_probe
python3 scripts/spec120_linear_light.py          # table
python3 scripts/spec120_linear_light.py --json   # machine-readable
```

Requires ImageMagick 7 (HDRI) on `PATH`; the harness refuses to run without it rather than
silently substituting an in-repo reference.

## References

- **SPEC-120** — `projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-120-measure-the-linear-light-premise.md`
- **SPEC-122 / DEC-095** — the fix this record authorised. It closed the defect and, in doing so,
  refuted this record's explanation of the alpha residual: see *Amended 2026-08-20* under
  Consequences before quoting the 27.
- **DEC-019** — SSIMULACRA2 as the perceptual oracle (confirmed for this question by C-control)
- **DEC-008** — `fast_image_resize` as the resize backend
- **DEC-074** — the committed-bench contract (why the synthetic stays out of `bench/corpus/`)
- **SPEC-088** — `bench/corpus/` and `scripts/bench.py`
- `docs/backlog.md` — `## Open — resize resamples in sRGB, not linear light`, where the result is
  appended
- `src/operation/mod.rs:395-527` (`Resize::apply`), `src/quality/mod.rs:25-100` (the scorer),
  `src/cli/report.rs:329` (the equal-dimensions rule)
