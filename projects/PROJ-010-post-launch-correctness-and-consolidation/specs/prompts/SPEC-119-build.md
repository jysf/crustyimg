# SPEC-119 — BUILD prompt

Cycle: **build**. You are NOT the architect. The design is settled; implement it.

**One-line summary:** `crustyimg lint anim.gif` tells the user to run
`crustyimg convert --format webp anim.gif`. That command silently discards 3 of 4 frames and
reports the loss as `72% smaller · ssim 100.0`. Make the tool say so — on **three** formats, not
one — and stop the linter recommending it.

## Read in order

1. **The spec** — `projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-119-animated-input-is-never-silently-flattened.md`,
   in full. **11 acceptance criteria, 4 design calls, 9 pre-written tests.** Read its Call 1
   ruling block and the revised Call 4 carefully — **both changed after the spec was first
   written**, and the revision is not cosmetic.
2. **`docs/backlog.md`**, `## ⚠ Live defect — animated input is silently flattened` — the driven
   evidence. **Read it; do not re-derive it.**
3. **The code** — `src/image/mod.rs:164-178` (the `Image` fields, and `truncated_jpeg` at `:174`
   which is your template), `:461` (`decode_with_limits`, the seam), `:551`;
   `src/lint/rules.rs:232-310` (the rule, its `fix:` string, and `gif_is_animated`);
   `src/cli/ops.rs:328-338` (the emit shape to copy).
4. **SPEC-116's shipped diff** (`git log --oneline --grep SPEC-116`) — it threaded exactly this
   kind of flag through `build`, and its `apply`-vs-`build` string-equality test is the right
   shape for AC-1.
5. **DEC-085** and `docs/api-contract.md`'s exit table.

## The two rulings that changed the spec — do not work from a stale reading

### Call 1 is RULED: warn and proceed. Do not refuse.

Maintainer-confirmed 2026-08-16, with a recorded reservation. Exit stays 0, frame 1 is still
written, and the warning is **unconditional** (not `--quiet`-gated — DEC-085).

**Do not implement a refusal, and do not add a flag to opt into one.** If you believe refuse is
right, say so in Build Completion and stop; it changes the frozen exit-code surface and is not a
build-cycle decision.

### Call 4 is REVISED, and it is now the larger half of this spec

The original said the lint rule could stay GIF-only. **That is no longer true**, and the reason
matters: the answer to *"warn-and-proceed still loses data"* is that
**`crustyimg lint --max-warnings 0` already exits non-zero** — `lint` is the gate, `convert` is
the tool (`src/lint/mod.rs:49-53`, `src/cli/report.rs:474`).

That only works for formats the rule detects. **A GIF-only rule leaves APNG and animated-WebP
users with no strict option at all.** So the rule must cover what the defect covers.

- **Keep `format/animated-gif` firing for GIF** — existing config and output must not break.
- **Add coverage for APNG and animated WebP.** Broadened rule under a new id with the old kept as
  an alias, or two sibling rules — **your call, justified in Build Completion.**
- **If it needs a config migration, STOP and report** rather than shipping one here.

## The sweep is done and BINDING — do not redo it, do not narrow it

Run at design against the pinned `image` 0.25.10:

```
grep -rn "impl.*AnimationDecoder.*for" ~/.cargo/registry/src/*/image-0.25.10/src/
  codecs/gif.rs:426           GifDecoder
  codecs/png.rs:514           ApngDecoder
  codecs/webp/decoder.rs:104  WebPDecoder
```

All three features are enabled here. **Three formats, not one.**

| format | detection API — cheap, no frame decode |
|---|---|
| GIF | `into_frames().take(2).count() >= 2` (already used at `rules.rs:302`) |
| **APNG** | `PngDecoder::is_apng() -> ImageResult<bool>` (`png.rs:160`) — returns a Result, **do not `unwrap`** |
| **animated WebP** | `WebPDecoder::has_animation() -> bool` (`decoder.rs:31`) |

TIFF, ICO, BMP, JPEG have no `AnimationDecoder` impl and are out.

**AVIF is the honest gap.** crustyimg decodes AVIF via `re_rav1d` (`src/image/avif.rs`), not
`image`'s codec, so this sweep says nothing about it. **Determine whether that path can receive a
sequence and report it. If you cannot settle it, say so and state that AVIF is therefore
unproven** — do not imply coverage this spec did not earn.

## Call 2: the carrier already exists — do not invent one

`Image` already carries a per-decode degradation flag for exactly this purpose
(`src/image/mod.rs:174`), set at decode and turned into a stderr warning by the CLI layer. **Add a
sibling field.** The four emit sites already exist (`ops.rs:336`, `:431`, `optimize.rs:1473`,
`:1522`, plus `build.rs`'s Decide arm from SPEC-116) — you are adding a second condition at those
sites, not a new plumbing route.

Constructors that cannot produce a multi-frame source (`raw_preview`, SVG rasterization) set it
`false` with the same one-line rationale `truncated_jpeg: false` carries at `:579`.

## The trap: your obvious oracle cannot fail

**Do not assert on the quality score.** SSIMULACRA2 compares decoded-source to output, and both
are frame 1 — the quantity it measures is *preserved by the bug*. A test asserting "the score
stayed high" is **vacuous by construction** and AC-6 will reject it.

**Assert structurally**: count `ANMF` chunks, or decode the output and count frames.

## Negative controls (AC-9) — three, not one

Revert the detection for GIF, APNG and WebP **independently**; confirm each format's test goes RED
and the static controls stay green. **Three detection sites are three independent claims.**

**The evidence is the behavioural flip, not a binary hash.** Measured in this repo 2026-08-16: a
rebuild from byte-identical source produces a *different* binary, so a changed hash proves only
that a relink happened. The test going RED is what proves the revert reached the artifact.

## AC-7b is the one people will skip

`lint --max-warnings 0` must **exit non-zero on each of the three families**, driven — with a
static counterpart of each staying clean. This is the answer the spec gives to the maintainer's
reservation about warn-and-proceed. **An answer nobody drove is not an answer.**

## Fixtures

Build them natively with `image`'s own encoders, the way `src/lint/rules.rs:361` already does for
the animated GIF. **Do not commit binary fixtures and do not shell out.** If you cannot construct
a valid animated WebP or APNG fixture natively, **say so and declare that format untested** rather
than shipping a test that cannot fail.

## Verify before handing back (AC-10)

Full matrix, fresh per-leg `CARGO_TARGET_DIR`, **sequentially**, through `rtk proxy` from the
first leg: default, `--no-default-features`, `--features webp-lossy`. Clippy
(`--all-targets -- -D warnings`) and `fmt --check` each. **Establish your own `main` baseline.**
Then read the CI legs individually (`gh pr checks <PR>`).

**A piped command reports the pipe's exit code** — redirect and read `$?`.

## Repo guardrails

- **Own git worktree**, branch `fix/spec-119-animated-input-never-silently-flattened`, off current
  `main`. Do not work in the primary checkout.
- **Checkpoint early** — push a WIP once it compiles, before the matrix.
- **Budget:** past ~2 hours without the matrix started, stop and report. Note that cost tracks
  **rebuilds**, not minutes: SPEC-117 cost $23 in 62 minutes on control rebuilds alone, and its
  mid-session reading was 49% low.
- `git commit -s` (DCO). Never `git reset --hard`. macOS has no `timeout(1)`.
- **`rtk` can corrupt grep counts** — cross-check with `/usr/bin/git` or raw `grep` plus a
  positive control.
- **Do not merge the PR. Do not bump the version.**

## Out of scope

- **Animated output encode.** ⚠ The spec's own out-of-scope note was **corrected 2026-08-16** —
  it previously called `webp-animation` "verified and filed", and both halves were wrong
  (it depends on non-optional `libwebp-sys2`, a C wrapper, so it fails `pure-rust-codecs-default`;
  and it was filed nowhere). Read the corrected block. **You need none of it** — this spec closes
  the destructive path, it does not add the capability.
- The other three STAGE-046 defects. They share `Resize::apply`; this one shares nothing with them.

## When you finish, in this order

1. Fill in `## Build Completion`, including the three reflection questions — and **say explicitly
   what you decided about the lint rule id, and whether AVIF is proven or unproven.**
2. Append a build cost session entry to `cost.sessions` (see below).
3. Write the DEC the spec expects — Call 1's warn-not-refuse policy for a whole input class —
   with `affected_scope` covering `src/image/**` and `src/cli/**`.
4. Run `just advance-cycle SPEC-119 verify`, and **CONFIRM it moved** (`git diff` shows the
   `cycle:` line change; it reports success even when it changes nothing).
5. Open the PR. **Do not merge it.**

### Cost

Follow `projects/_templates/prompts/cost-snippet.md`. Identify your transcript by something only
your session emitted — **never by "the newest `.jsonl`."** Price per component at the anchors of
the model `.message.model` reports (you are expected to be Sonnet: $3/$15 per MTok;
cache_creation ×1.25, cache_read ×0.10).

**Measure at session end.** Measured twice here: SPEC-114 reported $25.75 mid-run against $34.31
final; SPEC-117 reported $11.76 against **$23.06** — 49% low.

Close with the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not do as clearly as what you did.**
