---
stage:
  id: STAGE-052
  status: proposed
  priority: high
  target_complete: null

project:
  id: PROJ-013
repo:
  id: crustyimg

created_at: 2026-08-23
shipped_at: null

value_contribution:
  advances: >
    shipped verbs that silently return less than they were given.
  delivers: []
  explicitly_does_not:
    - "Re-open the shipped stage it continues — that stage closed in place with its specs"
---

# STAGE-052: Output Fidelity on Shipped Verbs, Continued

## What This Stage Is

⚠ **A continuation, not a re-home.** The stage this carries — `STAGE-046-output-fidelity-on-shipped-verbs` —
**shipped in PROJ-010 and closes there**, because a stage with shipped specs closes in place rather
than moving. Only its **open** items came here.

**7 items carried, unchanged**, on 2026-08-23. Their evidence, measurements and rulings
came with them; nothing was summarised away.

## Spec Backlog

- [ ] (not yet written) — [M] ⚠ **PRIORITY: multi-page TIFF and multi-size ICO are SPEC-119's
  defect on two axes SPEC-119 never covered — driven on `main` at `dcd43c8`, 2026-08-21.**
  SPEC-119 fixed *animation* (GIF / APNG / animated WebP). **Animation is not the only multi-image
  case**, and the two that remain behave exactly the way SPEC-119's own `value_link` described the
  bug it fixed: *"accepts a valid file, discards every frame but the first, reports the loss as a
  win … and — through `lint` — actively recommends the command that does it."*

  Fixtures built independently of the code under test and verified before conversion (3 IFDs in the
  TIFF chain; 3 entries in the icon directory):

  | input | contains | `convert` output | kept | **lost** | exit | stderr | `lint` | `optimize` says |
  |---|---|---|---|---|---|---|---|---|
  | 3-page TIFF | greys 70/140/210 | 8×8 png, pixel = **70** | page 1 | **2 pages** | 0 | **none** | 0 warn | **"542 → 78 B (86% smaller)"** |
  | 3-size ICO | 16/32/64 = R/G/B | **64×64** png, pixel = **(0,0,255)** | the 64px | **16px + 32px** | 0 | **none** | 0 warn | **"460 → 118 B (74% smaller)"** |

  **The output pixel value is the proof, not the exit code.** Grey 70 is page 1; blue is the 64px
  entry. Both verbs exit 0, print nothing to stderr, and `lint` reports `0 error · 0 warn · 0 info`.
  `optimize` presents the data loss as a large size win.

  **Mechanism** (from the #177 sweep, re-confirmed): `image` implements `AnimationDecoder` for
  exactly three decoders — `GifDecoder`, `ApngDecoder`, `WebPDecoder` — which is why the rule name
  `format/animated-gif` and SPEC-119's scope both stop there. **TIFF** exposes no multi-page API
  through `image` at all (`TiffDecoder` has `fn new` only), so pages 2..N are *unreachable and
  undetectable* without using the `tiff` crate directly. **ICO**'s `IcoDecoder::new` calls
  `best_entry()`, scoring on `(bits_per_pixel, width × height)` and **discarding every other entry**.

  ⚠ **These are two different problems wearing one description.** ICO is detectable and refusable
  today — the icon directory is right there in the header. TIFF is not, without a new parse path.
  Scope them separately; a spec that promises both will discover that mid-build.

  ⚠ Related but distinct, seen while driving this: `optimize` reports `source_format` as **`other`**
  for both, so it cannot name TIFF or ICO. SPEC-115 was *"never passes through bytes it cannot
  name"* and SPEC-117 taught `source_format` the real container for svg/heic/raw. Whether `other` is
  correct here or the same gap on two more formats is worth one look before specing.

  📌 **Filed here because `docs/backlog.md` is read by NO command.** The measurement first landed
  there via PR #177 on 2026-08-17 and was invisible to `just backlog` for four days
  [[a-document-is-not-a-backlog-unless-tooling-reads-it]].
- [ ] (not yet written) — [M] **Three writing paths still silently flatten animated input.**
      SPEC-119 fixed `convert`/`optimize`/`web`/`build`(Decide)/`apply`(terminal-optimize).
      **Driven by its verify, 2026-08-16:** `responsive anim.gif --widths 16` writes a 1-frame
      `anim-16w.gif` from a 4-frame source, exit 0, **empty stderr** — same for APNG and WebP.
      `apply --recipe <plain pixel recipe>` and `build` with a plain recipe are silent too.
      **The stage's own Goal — "no shipped verb silently discards frames" — is therefore not yet
      met.** Not a regression: `run_responsive` has its own `Image::load`
      (`src/cli/optimize.rs:1744`) and **misses the truncated-JPEG warning as well**, so this seam
      drops *both* diagnostics. That makes it evidence for STAGE-042's conformance matrix
      (SPEC-118) as much as a fix in its own right — a verb-by-diagnostic matrix would have
      surfaced it mechanically instead of at verify.
- [ ] (not yet written) — [S] **`info` describes an animated file as a still.** It is the one
      verb whose entire job is reporting, and `run_info` (`src/cli/report.rs:240-275`) checks
      `is_truncated_jpeg()` and **never calls `is_animated_input()`** — confirmed by reading, and
      surfaced by SPEC-119's punch list. Two consequences, the second sharper than the first:
      it prints no animation warning where every pixel verb now does; and its report is
      internally inconsistent — `file_size_bytes` covers **all frames** while `decoded_bytes`,
      `width`, `height` and `color_type` come from `img.info()`, i.e. **frame 1 only**. The two
      size fields describe different things without saying so. The flag already exists and
      `Image` already carries it, so this is a report field plus a warning, not new detection.

> **Moved from STAGE-042, 2026-08-16.** Same class as the rest of this stage: the tool
> silently delivers less than it was given and exits 0.
- [ ] (not yet written) — [S] **`size/truncated-or-corrupt` does not fire on a truncated file, and
  the rule roster is 11 not 9.** From the read-only CLI-surface audit (F5). **A design question,
  not a patch** — it comes back as a proposal.

  Every pixel-lane verb warns on a truncated JPEG (`info` prints *"missing end-of-image marker
  (FF D9)"*), while `lint --select size` over the same files reports `0 error · 0 warn · 0 info`,
  exit 0. **Mechanism confirmed at source** (`src/lint/mod.rs`, `TruncatedOrCorrupt::check`): the
  rule keys on `target.decoded()` returning `Err`, and a JPEG missing EOI **decodes successfully
  with a warning** — `Ok` — so the rule never sees it.

  ⚠ **This is a naming/scope mismatch more than a bug, and the code already knows it.** The rule
  carries two reasoned carve-outs (`CodecNotBuilt`, `LimitsExceeded`) whose comments name the same
  structural limit: its severity is fixed at **Error**, so it cannot carry a softer verdict. Those
  comments already propose `meta/not-inspected` (Info) and `size/over-decode-budget` as follow-ups
  — **a `size/decodes-with-warnings` rule would be the third of that family** and is probably the
  right shape. **Do not widen the existing rule's scope; propose the new one.**

  **Why it matters:** a photographer running `lint` over an archive gets an all-clear on files the
  tool already knows are damaged.

  ⚡ **Fix the roster count while in here — it is load-bearing.** The roster is **11 rules, not 9**:
  `privacy/gps-metadata-leak` and `size/truncated-or-corrupt` live in `src/lint/mod.rs`, **not**
  `src/lint/rules.rs` — **and they are the two Error-severity rules, i.e. the ones that gate CI.**
  Any sweep or doc scoped to `rules.rs` misses exactly the load-bearing half
  [[mechanical-sweeps-need-a-mechanical-check]]. Authoritative:
  `grep -rn 'fn id(&self)' -A1 src/lint/`.
- [ ] (not yet written) — [S] ⚠ **PRIORITY: the `IMAGE_EXTENSIONS` gap silently defeats the
  strict gate that a maintainer decision rests on.** Not a new defect — the *consequence* of the
  item below, and it is why that item is no longer routine.

  SPEC-119's Call 1 (animated input **warns and proceeds** rather than refusing) was accepted on
  2026-08-16 on one argument: **`lint --max-warnings 0` is the strict path**, so a pipeline that
  must never flatten an animation has a way to say so. Driven by SPEC-119's verify:

  ```
  lint --max-warnings 0 <dir containing anim.webp>   → exit 0   "1 scanned · 0 warn"
  lint --max-warnings 0 <dir>/anim.webp              → exit 7
  optimize <dir>/*.webp                              → warns; 408 → 240 B, 4 frames → 1
  ```

  **Directory mode — the shape CI actually uses — returns a false green.** Naming the file or
  piping stdin both work. `docs/api-contract.md` states `lint --max-warnings 0` "fails on any of
  the three formats" **with no qualifier**, which is now false as written.

  Two things follow: the contract sentence needs its qualifier (SPEC-119 punch list), and the
  `IMAGE_EXTENSIONS` fix should be **specced rather than left in the backlog**, because a
  maintainer ruling now depends on it.
- [ ] (not yet written) — [S] **`webp` is missing from `IMAGE_EXTENSIONS`, so directory and glob
  discovery silently skips `.webp` files.** `src/source/mod.rs:105-113` lists 30+ extensions —
  jpg/png/gif/bmp/tif/ico/avif/svg, eleven RAW families, heic/heif — and **not `webp`**, which is
  a supported input *and* an output format the tool writes by default. So `crustyimg web ./dir/`
  processes everything except the files crustyimg itself produced. **Reproduces on `main`,
  confirmed 2026-08-16**; found by SPEC-119's build and recorded in DEC-093, which is not a
  backlog anyone reads. The repo already knows this hazard class — `src/lint/mod.rs:217` cites
  "the IMAGE_EXTENSIONS-exposes-every-decode-caller lesson" by name. Adding an extension changes
  every decode caller, so **audit each caller and its `Err(_)` arm** rather than editing the list
  alone.
- [ ] (not yet written) — [S] **Probe whether `U16x4` recovers linear-light `resize`'s speed and
  memory without moving the output.** SPEC-122 resamples in linear `F32x4` — **16 B/px, 4× RGBA8**
  — which is correct and is the point of the spec, but it costs **3.83×** wall clock (169 → 649 µs)
  and **2.8×–5.3×** peak RSS (measured: 166 → 465 MB downscale; 266 → **1407 MB** on a 512²→6000²
  upscale). ⚠ **Verify's decomposition is the load-bearing number: 76% of the added time is the
  WORKING TYPE, not the transfer function** — so gamma math is not where the cost is, and no
  cheap fix exists inside the current type.
  **The candidate is `U16x4`** (8 B/px, shipped by `fast_image_resize` 6.0.0): halves the memory
  delta, and 16-bit linear is ample precision for 8-bit output. **Unmeasured on all three axes** —
  does it recover the time, what does it do to a 16-bit source, and does the output move?
  ⚡ **This is a probe, not a fix — do not scope it as one.** SPEC-123 cost $60.33 largely because
  a measurement whose premise might be wrong was sized `[S]` and asserted in advance
  [[a-measurement-specs-cost-lives-in-the-refutation]].
  **Deliberately NOT in 0.7.1** (maintainer, 2026-08-20). No correctness defect; 649 µs is
  sub-millisecond and the memory is transient and input-bounded. It is an optimization, so it can
  carry its own lockfile migration later — unlike SPEC-121/122/124, which share one because they
  are correctness fixes landing together.
  **Related and separate:** `MAX_AREA` bounds the OUTPUT buffer, not the peak, and still does its
  documented job (`src/operation/mod.rs:870-885`). Whether that bound should move for untrusted
  input is its own decision, recorded in the comment and in DEC-095 — **not a regression.**

**Count:** 0 shipped / 0 active / **7 pending** — re-derive with a grep you just ran.

## Stage-Level Reflection

*Filled in when status moves to shipped.*
