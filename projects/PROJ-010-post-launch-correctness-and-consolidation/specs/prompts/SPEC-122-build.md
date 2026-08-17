# SPEC-122 — BUILD prompt

Cycle: **build**. You are NOT the architect. The premise was measured before this spec existed;
implement the fix.

**One-line summary:** `Resize::apply` resamples non-linear sRGB values as if they were linear.
Against an independent reference the shipped downscale scores **70.45** and **84.45** where a
linear-light prototype scores ~100. Linearize → resample → re-encode, inside the op.

## ⛔ Hard gate — check this before anything else

**SPEC-121 must be merged to `main` before you start.** It rewrites the same function you are
about to rewrite (`Resize::apply`), and you branch from `main` *after* it lands.

```bash
git log --oneline main | grep -i 'spec-121'
```

**No SPEC-121 commit on `main` → stop and report.** Do not branch from SPEC-121's branch, and do
not reimplement its narrowing yourself. This is the third of three serial specs; the sequencing is
deliberate and the pair shares one decision.

## Read in order

1. **The spec** — `projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-122-resize-resamples-in-linear-light.md`,
   in full. **10 ACs, 5 settled design calls, 4 failing tests already written.**
2. **DEC-092** — the premise verdict, the numbers, and the alpha refutation. **Binding.**
3. **SPEC-121's merged diff and its DEC-095** — you are amending that decision, not writing a new
   one, and you are editing the function it just changed.
4. **The prototype** — `examples/spec120_linear_probe.rs` and `scripts/spec120_linear_light.py`.
5. **The code** — `src/operation/mod.rs`, `Resize::apply` (post-SPEC-121; the line numbers in the
   spec predate that merge — find it by name).
6. **`/AGENTS.md`** — §4 cost, §6 commands, §12 testing, §13 git/PR, **§15**.

## Do not re-litigate the premise. Two halves of it are already settled.

**The linear-light half HOLDS.** SPEC-120 measured it and the falsification gate fired:

| case | source → target | mean signed luma err | today's SS2 | linear prototype |
|---|---|---|---|---|
| synthetic worst case (positive control) | 2048²→256² | −0.104350 (−88.07%) | −63.85 | 100.00 |
| `graphic_large.png` | 512²→128² | −0.001386 (−0.44%) | **70.45** | 100.00 |
| `photo_forest_cc0.jpg` | 800×532→200×133 | −0.004920 (−2.63%) | **84.45** | 99.41 |

⚠ **Read those correctly.** The prototype's ~100 and the Δ column are **partly self-fulfilling** —
a linear-light Lanczos prototype scored against a linear-light Lanczos reference should agree.
**The claim that survives is the third column: the shipped path scores 70.45 and 84.45 against an
independent reference** (ImageMagick 7.1.2-29 Q16-HDRI, `-colorspace RGB`). Frame your before/after
the same way; do not quote the Δ as though it were the finding.

The positive control is what makes those numbers readable rather than an uninterpretable null: an
−88% physical error registered as a **163.85-point swing**, so the instrument can see this defect.

**⚡ The premultiplied-alpha half is FALSE.** `fast_image_resize` 6.0.0's
`ResizeOptions::default()` sets `mul_div_alpha: true` (`src/resizer.rs:52-60`), and
`ResizeOptions::new()` *is* `Default::default()` (`:63-64`) — `Resize::apply` overrides only the
algorithm. **It has always premultiplied. Do not add alpha handling.** AC-5 exists to prove
nothing moved there.

## The design calls are SETTLED — do not reopen them

1. **Convert inside `Resize::apply`.** No pipeline-wide colour management, no new `Image` field,
   no 16-bit-throughout project. `fast_image_resize` 6.0.0 already ships `F32x4` and `U16x4`
   (`src/pixels.rs:26,21`) — the backend is in place.
2. **Start from `examples/spec120_linear_probe.rs`.** It is the code that produced the numbers
   above; starting anywhere else means re-earning them. It is an `examples/` throwaway —
   productionizing it is this spec's work. **Say whether you deleted it or kept it**, and why.
3. **Reuse SPEC-120's harness as the acceptance test.** Do not invent a new oracle. Re-run
   `scripts/spec120_linear_light.py` against the **branch** binary on the same three cases.
4. **sRGB is assumed; ICC-aware conversion is NOT this spec.** crustyimg keeps ICC profiles but
   does not interpret them. Assume the sRGB transfer function and **say so in the DEC**. An image
   with a non-sRGB profile gets resampled under an assumption that is wrong for it — better than
   today's, still an assumption. Full colour management is its own project.
5. **The migration already exists** (shared with SPEC-121): `cache_key_for` includes
   `crate::version()` (`src/cli/build.rs:294`) and the lockfile never promised output-hash
   stability across versions (`src/build/lock.rs:32-36`). **AC-8 drives it; it does not design
   it** — key changes, `--frozen` fails, regeneration succeeds, no stale cache hit. **If the
   contract does not hold, stop and report.**

## Keep SPEC-121's fix intact

You are editing the function it just fixed. Its colour-type and bit-depth tests
(`tests/colour_type_preservation.rs`) are now **regression guards on your change**, not background
noise. An f32 linear pipeline that returns RGBA8 unconditionally re-introduces the exact defect
that shipped one spec earlier.

**Run that file explicitly and report it green**, alongside your own.

## The controls — three, and each answers a different way this could be fake

- **AC-4 — the reference stays independent.** Regenerate it with the same external tool. Do **not**
  substitute crustyimg's own output: a fixture derived from the code under test cannot fail, and it
  would make this untestable forever.
- **AC-3 — the positive control still fires.** The synthetic worst case must still show the large
  swing. A fix that improves only the realistic cases is suspicious, not reassuring.
- **AC-7 — the negative control.** Revert the linearization; the three cases return to **70.45 /
  84.45 / −63.85**. AGENTS §15: **the behavioural flip is the evidence, not a binary hash** — a
  debug rebuild from byte-identical source produced a different binary on this repo, so a hash
  proves only that cargo relinked.

**AC-2: report both metrics — SSIMULACRA2 and mean signed luminance error. If they disagree, that
disagreement is the finding.** Do not reconcile them into a tidier story than the data supports.

## AC-9 — performance, measured and reported

f32 resampling is more work than u8, `resize` is the most-used op, and **nobody asked at design.**
Not a gate — but measure it with controls (repeats, same machine, same input set) and report it
plainly. If it is bad enough to matter, that is a finding for the architect, not a reason to
abandon the fix or to quietly optimize your way out of the spec.

## The matrix — AC-10

Clean full matrix, **fresh per-leg `CARGO_TARGET_DIR`**, **sequential**, through `rtk proxy`:
default, `--no-default-features`, `--features webp-lossy`. Clippy (`--all-targets -- -D warnings`)
and `cargo fmt --check` each. ⚠ **Never both shared-and-parallel** — concurrent
differently-featured builds sharing one target dir corrupt it, measured on this repo. Then **read
the CI legs individually**; a green summary is not a matrix.

AC-6 needs upscale and no-op resize **byte-identical to `main`** — build your own baseline rather
than trusting a remembered number.

## Guardrails

- **Own git worktree**, branch `fix/spec-122-resize-resamples-in-linear-light`, from `main` **after
  SPEC-121 merges**.
- **⚡ AMEND DEC-095. Do not mint a new DEC.** SPEC-121 wrote it to cover the wave; you add the
  linear-light change and Call 4's sRGB assumption to it. **Do not run `next_id`** — it scans only
  the working tree and has already produced one collision in this project (SPEC-119 and SPEC-120
  both minted DEC-092). If DEC-095 does not exist on `main`, that means SPEC-121 did not land as
  specified — stop and report rather than improvising an ID.
- **⚡ Checkpoint early.** Push a WIP commit **as soon as the branch compiles**, before the harness
  runs and before the matrix. SPEC-113's build ran three hours and $40 with **zero commits**.
- **Budget in exchanges, not minutes.** This is an **M** — past **~250 exchanges** without having
  started the matrix, checkpoint and report. Cost scales with the *square* of message count and
  anti-correlates with wall clock.
- **A piped command reports the pipe's exit code** — redirect and read `$?`.
- macOS has no `timeout(1)`. `git commit -s` (DCO). Never `git reset --hard`.
- **Do not merge the PR. Do not bump the version.**

## When you finish, in this order

1. Fill in the spec's `## Build Completion`, including its three reflection questions.
2. Append a build cost session entry to `cost.sessions` (see below).
3. **Amend DEC-095** — the linear-light change, the sRGB assumption, the shared migration posture.
   Confirm `affected_scope` still covers `src/operation/**`.
4. Run `just advance-cycle SPEC-122 verify`, and **CONFIRM it moved** — `git diff` on the spec
   should show the `cycle:` line change. It reports success even when it changes nothing.
5. Open the PR. **Do not merge it.**

### Cost

Follow `projects/_templates/prompts/cost-snippet.md`. Identify your transcript by something only
your session emitted — **never by "the newest `.jsonl`."** Price per component at the anchors of
the model `.message.model` actually reports (you are expected to be **Sonnet**: $3/$15 per MTok;
cache_creation ×1.25 input, cache_read ×0.10 input). **Do not price at Opus anchors because a
prompt named them** — SPEC-108 did exactly that and overstated its total by ~67%.

**Measure at session end, not mid-session.** Mid-session readings have run 40–49% low, measured
twice. Re-measure as the last thing you do.

Close with the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not do as clearly as what you did.**
