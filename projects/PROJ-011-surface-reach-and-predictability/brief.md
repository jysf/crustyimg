---
# Maps to ContextCore project.* semantic conventions.
# A project is a bounded wave of work against the repo (the app).

project:
  id: PROJ-011
  status: active
  priority: high
  target_ship: null

repo:
  id: crustyimg

created_at: 2026-08-21
shipped_at: null

value:
  thesis: >
    A declared `build` should be able to express what the CLI can already do.
    Today it cannot: the operation registry holds four ops, so a recipe cannot
    carry a watermark, a format or a quality — and `build` binds sources to a
    recipe, so every one of those limits is inherited. The concrete consequence,
    from the maintainer's own site work: **every image on theF11.com should be
    watermarked, and that cannot be done through `build` at all.** The working
    alternative is a hand-driven three-pass pipeline. Closing this makes the
    declarative path as capable as the imperative one, which is the promise
    `build` implies and does not keep.
  beneficiaries:
    - "The maintainer, who wants one `build` to watermark and optimize a whole photo site and currently runs a three-pass pipeline by hand"
    - "Anyone automating crustyimg in CI, where the declarative path is the only one worth wiring up and is today the weaker one"
    - "Anyone who has written a recipe and discovered mid-batch that a parameter was misspelled, because validation happens per-operation at apply time rather than at parse time"
    - "Downstream readers of a `*.build.lock`, which today pins bytes the `apply` spelling of the same recipe cannot reproduce"
  success_signals:
    - "A single `crustyimg build` watermarks and optimizes an entire photo directory from a manifest — the theF11.com case, driven end to end"
    - "A recipe can pin an output format and quality, so `apply --recipe` reaches what `convert` reaches"
    - "`apply` and `build` produce byte-identical output for the same recipe and input, asserted by a test — today they disagree on the default format"
    - "`apply --format` is honoured on a multi-input batch, which it is not today"
    - "A malformed recipe fails at parse time with a message naming the bad key, not partway through image 37 of 50"
  risks_to_thesis:
    - "⚠ Every item here is byte- or behaviour-changing on a shipped verb, so the project carries ONE lockfile migration and must ship as one release — the batching STAGE-046 used. Sequencing it wrong costs users two migrations"
    - "⚠ `watermark --size` is ABSOLUTE PIXELS. A recipe-level watermark only behaves consistently if the recipe normalises dimensions first, so this is a design constraint on the op, not an implementation detail to meet during build"
    - "The registry is documented as 'the single seam new operations register at', but no op with watermark's parameter richness has ever registered. If the seam turns out to need widening, that is a larger change than this brief assumes"
    - "⚠ This project is deliberately NARROW. Nine measured defects were considered and six were left out. The temptation to pull them back in is the main way it loses its shape and its ship date"
---

# PROJ-011: A Declared Build Can Do What the CLI Can

## What This Project Is

`crustyimg build` is the declarative path: a manifest binds sources to a recipe, a lockfile pins
the result, and CI can gate on it. It is the path worth automating — and it is **strictly weaker
than typing the commands by hand.**

The operation registry holds **four** ops (`identity`, `invert`, `resize`, `auto-orient`) plus a
terminal `optimize` marker. So a recipe cannot express a **watermark**, and `Recipe` has no field
for an output **format** or **quality** at all. Everything `build` can do is bounded by that.

The concrete case that motivated this, from the maintainer's own work: **every image on
theF11.com should carry a watermark, and there is no way to say that in a manifest.** The
working alternative is a three-pass pipeline driven by hand — `resize --format png` →
`watermark --format png` → `optimize --verify` — which costs +0.5% bytes and cannot be declared.

When this ships, one `build` does the whole job.

## Why Now

**The gap was found by using the tool for real work, not by auditing it.** That is the same
provenance as every finding that mattered this year, and it is a different signal from a review
batch: it means someone hit the wall while trying to get something done.

**Two defects sit directly in the path and were measured on `main`:**

| invocation | output format |
|---|---|
| `apply` **1** JPEG, no `--format` | **PNG** — the source format is changed |
| `apply` **2** JPEGs, no `--format` | JPEG — the source format is preserved |
| `apply` **1** JPEG, `--format png` | PNG ✅ |
| `apply` **2** JPEGs, `--format png` | **JPEG — the flag is silently ignored** |

`apply`'s multi-input path does no format resolution at all. And **`apply --format` exists
precisely because a recipe cannot carry a format** — it is the workaround for the gap this project
closes, and it is broken on the exact path a site build uses. ⚡ **The two are one wound seen from
both sides**, which is why they belong in one project rather than being fixed twice.

The same divergence means a `*.build.lock` pins bytes the `apply` spelling of the same recipe
cannot reproduce — so two commands the docs present as interchangeable are not.

### What this project is NOT, and why

An earlier draft of PROJ-011 bundled multi-frame input, animated AVIF output and six further
consistency defects. **That drifted into a second correctness project** — PROJ-010 already is the
correctness lane, and it has ~24 actionable items and no end state. This brief was re-cut around a
single user-visible outcome instead. **Six measured defects were deliberately left behind**
(see Out of Scope); they are real, and they are not in the way of this.

## Success Criteria

- **One `crustyimg build` watermarks and optimizes a whole photo directory from a manifest** —
  the theF11.com case, driven end to end rather than reasoned about.
- A recipe can **pin an output format and quality**, so `apply --recipe` reaches what `convert`
  reaches.
- **`apply` and `build` produce byte-identical output** for the same recipe and input, asserted by
  a test. They disagree today.
- **`apply --format` is honoured on a multi-input batch.**
- A malformed recipe **fails at parse time**, naming the bad key — not partway through image 37.

## Scope

### In scope
- A `watermark` operation in the registry, with its parameters.
- `format` and `quality` as `Recipe` fields.
- Typed per-operation parameter structs, so validation happens at parse.
- `apply`'s multi-input format resolution, and the `apply`/`build` default-format disagreement.
- A **targeted** encode-identity test: `apply` vs `build`, and `-o` vs `--out-dir`, for the same
  recipe and input.

### Explicitly out of scope
- ⚡ **Animated AVIF output** — forked to its own design track, becomes **PROJ-012** when specced.
  Its next work is a `mp4-atom` DEC and splitting `docs/research/draft-spec-animated-avif-output.md`
  into buildable specs. That is design work; it runs in parallel and does not compete for build
  sessions or touch this project's migration.
- **The six defects left behind, all back on PROJ-010:** multi-page TIFF and multi-size ICO silent
  data loss; the ICO round-trip defect; `IMAGE_EXTENSIONS` gaps; `info` describing an animation as
  a still; the `-o`-extension pin ruling; `--explain`'s silence under a pin. ⚠ **Real, several of
  them silent data loss — but none is in the way of a declared build watermarking a site.**
- **The full `(command × output-flag)` conformance matrix**, and **SPEC-118**'s
  `bundled_recipe × entry_point` matrix. ⚠ **These are siblings, not duplicates** — SPEC-118 varies
  entry points and would not have caught the `-o` divergence; the output-flag matrix varies flags
  and would not catch a bundled recipe failing through wasm. Both stay on PROJ-010. This project
  builds only the narrow assertion its own changes need.
- **New verbs.** The surface is 18 verbs; this widens what `build` reaches, not the roster.
- **The engineering-quality backlog** — three external review batches, STAGE-042's remainder, the
  `F32x4` cap gap. No user-facing thesis; it belongs in the correctness lane.

## Stage Plan

**Ordered by dependency.** Both stages are byte- or behaviour-changing, so the project carries
**one** lockfile migration and ships as **one** release.

- [ ] **STAGE-049 — `apply` and `build` agree.** Fix `apply`'s multi-input format resolution and
  the `apply`/`build` default-format disagreement (**one defect, not two**), plus the targeted
  encode-identity assertion. ⚡ **First, because it settles the semantics the `Recipe` format field
  must then match.** Adding the field on top of two paths that already disagree would bake the
  disagreement in.
  📌 Controls already run and worth reusing: `resize`, `thumbnail` and `watermark` all honour
  `--format` across the same two inputs, so the defect is specific to `apply`.

- [ ] **STAGE-050 — recipe reach.** A `watermark` op in the registry; `format` and `quality` on
  `Recipe`; typed per-operation parameter structs.
  ⚡ **The design call to make first, and it is now a MEASURED defect, not a hypothetical.**
  `watermark --size` is absolute pixels, and driven on `main` at `fc360c4` against flat fixtures
  (PNG filters undone before comparing — filtered bytes differ everywhere and read as 100 %):

  | input | `--size` | pixels changed | reading |
  |---|---|---:|---|
  | 800×600 | default | **0.32 %** | a sane watermark |
  | 64×64 | default | **16 %** | covers a sixth of the image |
  | 24×24 | default | **47 %** | covers nearly half the image |
  | 24×24 | **200** | **0.00 %** | ⚡ **silently NOT APPLIED — exit 0, file written, no warning** |

  **Two failure modes, both silent, and the second is the one that matters.** Asking for a
  watermark bigger than the image produces a success exit code and an unwatermarked file. A user
  batching a site cannot tell which outputs carry the mark. That is precisely this project's own
  thesis — *never silently do less than you asked* — inside the op it is about to register.

  **So the call is not just "relative or absolute".** It is: what happens when the watermark does
  not fit? Refuse (exit 2), warn and skip, warn and apply, or scale to fit. ⚠ A recipe-level
  watermark makes this sharper, not softer: one recipe over a directory of mixed sizes will hit
  every row of that table in a single run.
  📌 A relative size (percentage of the long edge) answers the batch-consistency half, but **does
  not by itself answer the silent-no-op half** — a 5 % watermark on a 24 px thumbnail is still
  unreadable. Both halves need a ruling.
  ### The ruling — maintainer, 2026-08-23

  **Watermarking exists to deter reproduction of images that have reproduction value. A thumbnail
  has none.** So watermarking a small image is both pointless and destructive. That settles the
  intent; what follows is the design it implies.

  ⚡ **Two rules, and they are not the same rule.** Conflating them is how this gets built wrong:

  | | **Damage rule** | **Value rule** |
  |---|---|---|
  | asks | would the watermark *wreck* this image? | is this image *worth* watermarking? |
  | depends on | watermark size **relative to** the image | the image alone, and the user's business |
  | who can answer | **the tool** — it is pure geometry | **only the user** — the tool cannot know what is worth protecting |
  | so it should be | **enforced by default** | **declared, opt-in** |

  **The tool enforces geometry; the user declares policy.** Hard-coding "images under 400 px never
  get a watermark" is the tool making a business judgement it has no standing to make. Providing
  `--min-size` so the user says it once, in a recipe, is the same outcome without the overreach.

  ### What each rule does

  **Damage rule — default on.** When the watermark would cover more than a threshold fraction of
  the image, it does not silently proceed. ⚠ **The response must differ by invocation**, and this
  is the part most likely to be got wrong:
  - **Single explicit input** (`watermark one.jpg`) → **error, exit 2**, naming the numbers. The
    user pointed at this file. Refusing beats damaging it, and beats today's silent no-op.
  - **Batch or recipe** → **skip that file, report it, continue**, with a summary count.
    ⚠ **Never abort the run.** A mixed directory of 4000 px originals and 400 px thumbnails is the
    normal case, not the exception — erroring there makes one `build` over a real site impossible,
    which is the outcome this whole project exists to deliver.

  **Value rule — opt-in `--min-size`, and settable in a recipe.** Below it, skip and report. This
  is what makes a single `build` over a site correct: originals get the mark, generated thumbnails
  do not, and the manifest says so.

  ### Still to settle before specing

  - **The damage threshold is a number nobody has measured.** Reference points from the drive
    above: 24 px → 47 % coverage (clearly wrecked), 64 px → 16 % (marginal), 800 px → 0.32 %
    (fine). A ~25 % starting point separates those, **but validate it against real output rather
    than adopting it from this note.**
  - **Whether the threshold is a knob.** A fixed constant is simpler; a knob lets a user who wants
    an aggressive mark have one. Default-with-override is the likely answer.
  - ⚠ **`--tile` changes the arithmetic** — a tiled watermark covers the image by design, so the
    coverage test cannot apply to it unmodified.

  📌 Why the gap stayed invisible: **only `edit` has `--save-recipe`**, so the one path that emits
  a recipe covers only ops a recipe can already express.

**Count:** 0 shipped / 0 active / **2 pending** — re-derive with a grep you just ran; never restate
a tally you carried forward.

## Dependencies

### Depends on
- **v0.7.1 shipped** ✅ (2026-08-22, all three channels).
- ⛔ **One design ruling, inside STAGE-050:** how a recipe-level watermark handles absolute-pixel
  `--size`. Nothing else here is blocked on a decision.
- **PROJ-010 stays open** as the correctness lane, and keeps the six defects this project declined.

### Enables
- A `build` manifest that is worth wiring into CI, because it can express the whole job.
- `--save-recipe` becoming defensible on more than one verb, once a recipe can express more.
- **PROJ-012** (animated AVIF) inherits a registry that has absorbed one parameter-rich op — the
  precedent an animated encoder's own configuration will need.

## Project-Level Reflection

*Filled in when status moves to shipped.*
