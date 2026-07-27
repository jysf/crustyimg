# SPEC-108 — BUILD prompt

Cycle: **build**. You are NOT the architect. The design is settled; your job is to implement it.
This is an **engine change to the shared content classifier** — the highest-risk change in
PROJ-010 and the launch gate the whole project exists for.

**Precondition — check before starting.** SPEC-109 (PR #114, merged 2026-07-27 as `408b0f9`)
must be on your `main`, because AC-3 needs its two boundary specimens. Verify by **name**, not
by counting the directory:

```bash
ls tests/fixtures/classify/photo_entropy_floor.png tests/fixtures/classify/dither_32color.png
```

Both must exist. If either is missing, stop — you are not on an up-to-date `main`.

**One-line summary of the job:** classification currently runs on the *output* of the resize
pipeline, so `--max` decides an image's content class. Make it run on the source.

## Read in order — deliberately short

Everything you need that lives elsewhere is inlined below. **Do not go reading the wider
evidence trail unless something here contradicts what you find in the code** — if it does, stop
and report the contradiction rather than resolving it yourself.

1. **The spec** —
   `/projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-108-classification-placement-and-scale-aware-entropy.md`,
   in full. It is your contract: 9 acceptance criteria, 6 pre-written failing tests. Do not skip
   `### What SPEC-109 established that changes this spec's picture`.
2. **The code** — `src/cli/optimize.rs` (the pipeline → analyse → decide sequence),
   `src/analysis/mod.rs` (the `classify()` cascade and its constants), `src/analysis/decide.rs`.
3. **`/AGENTS.md`, these sections only** — §4 cost tracking, §5 tech stack, §6 commands,
   and the git/PR conventions. Skip the rest.

Optional, only if you need provenance for something: `decisions/DEC-047-*.md` (the SPEC-105
amendment section and its two 2026-07-26 corrections) and
`docs/research/pr113-classifier-review-findings.md` §"Re-derivation (2026-07-25)". You should
not need either — everything load-bearing from them is below.

## The measured facts, inlined

Every number here is from a release build; the re-derivation of 2026-07-25 supersedes the
original review's figures, which were wrong on magnitude.

**The defect.** Classification runs on the resize *output*: `pipeline.run`
(`src/cli/optimize.rs:989`) → `Analysis::compute` (`:1013`) → `decide::format_shortlist`
(`:1026`). So `--max` decides an image's content class. A 1-bit halftone goes from **45,527 B
lossless passthrough at native size to 844,492 B lossy AVIF at the default `--max 2048`** —
18.5× larger, SSIMULACRA2 69.2. At `--max 2560` it reaches 35×.

**The committed fixture, at four scales** (`tests/fixtures/classify/dithered_graphic.png`):

| `--max` | class | entropy | edge | flat | unique_colors |
|---|---|---|---|---|---|
| 4096 / 512 | `graphic-logo` | 3.03 | 0.28 | 0.49 | 9 |
| **256** | **`photograph`** | **7.08** | 0.05 | 0.27 | **217** |
| 128 | `icon` | 7.15 | 0.07 | 0.36 | 207 |

**The other fixtures** (`--max 8192`, no resize): leica 6.07, canon 6.83, fuji 6.37 — all
`photograph`, all with `flat_ratio` **0.76–0.83** and `edge_ratio` 0.00. `photo_entropy_floor`
4.5176, `flat_ratio` **1.00**. `dither_32color` 3.6414. `checker_graphic` 2.78.

**Constants:** `ICON_MAX_EDGE` 128 · `PALETTE_COLORS` 256 · `FLAT_GRAPHIC_RATIO` 0.60 ·
`GRAPHIC_EDGE_MAX` 0.08 · `DOC_ENTROPY_MAX` 4.5 · `PHOTO_ENTROPY` 5.0 ·
`PHOTO_ENTROPY_STRONG` 4.0 · `PHOTO_FLAT_MAX` 0.25.

**Blast radius:** dithered and halftoned sources whose long edge exceeds ~2048 by more than
~20%. **Not screenshots** (four measured, entropy 0.80–2.37, all stay lossless) and **not
favicons** (sub-129 px hits the `Icon` rule). A screenshot-only fixture corpus would go green
against this defect — scope your fixtures to dither and halftone.

## The design decision is settled — do not re-litigate it

**Classify the source image, not the pipeline output.** The narrower alternative (delete rule
3.5's early return, gate rule 4's clauses on `entropy < PHOTO_ENTROPY_STRONG`) was evaluated at
design and **refuted by measurement**: at `--max 256` the committed fixture has **217 unique
colours ≤ `PALETTE_COLORS` 256**, so `few_colors` is TRUE and `many_colors` FALSE — which makes
rules 5 and 6 unreachable — and it falls to rule 7, whose bias is `Photograph`. It changes which
line returns `Photograph`, not the answer. The full trace is in the spec's Context.

If you find yourself concluding the narrow fix is better, you have probably missed the
`few_colors` step. Re-read the trace before proposing it.

## Before you change anything: reproduce the baseline

```bash
cargo build --release --locked
CB=./target/release/crustyimg
for m in 4096 512 256 128; do
  $CB web tests/fixtures/classify/dithered_graphic.png --max $m --json -o /dev/null
done
```

Expect: **3.03 `graphic-logo`** at 4096 and 512; **7.08 `photograph`** at 256; **7.15 `icon`**
at 128. If your build disagrees, stop and reconcile — do not build on a disagreement.

## What to build

Branch `spec-108-classify-the-source-image` off `main`.

Compute `Analysis` from the **source** image before `pipeline.run` and thread it to the decision
site. Then resolve the cascade contradictions the same change stands in — AC-5 through AC-8 in
the spec. Emit a **new DEC** recording "classify the source, not the pipeline output", including
the measured refutation of the narrow alternative so it is not re-proposed.

Nine acceptance criteria and six failing tests are in the spec. They are the contract.

## Traps — every one of these has already bitten someone on this stage

- **`--max 128` looks like a pass and is not.** It returns `icon` → lossless because the Icon
  rule fires on *size*, before entropy is consulted. A test asserting "lossless at 128" goes
  green on the broken build. **Assert the class.**
- **Moving the analysis earlier means EXIF is present where it previously was not.** The
  pipeline bakes orientation and drops metadata (DEC-017); rule 2 keys on `has_exif` and returns
  `Photograph` immediately. Classifying pre-pipeline changes behaviour for every EXIF-bearing
  input. **This is the most likely place for this spec to surprise you** — measure the fallout,
  do not assume it is inert.
- **Do NOT weaken or reorder rule 3.5.** Confirmed load-bearing from two independent directions:
  the three photo fixtures measure `flat_ratio` 0.76–0.83 and `photo_entropy_floor.png` measures
  **1.00** — all above `FLAT_GRAPHIC_RATIO` 0.60 with `edge_ratio` ≈ 0.00. Rule 4b would claim
  every one of them if 3.5 did not fire first. Its unconditional early return is the only thing
  holding real photographs off the lossless path while the flat detector stays scale-broken.
- **Rule 6's dead code does not fall out of this fix.** Under placement, rule 3.5 keeps its early
  return, so rules 5 and 6 stay unreachable. AC-6 is real work: make rule 6 reachable **or**
  delete it, and leave no inert constants (`PHOTO_ENTROPY`, `PHOTO_FLAT_MAX`). Deleting is an
  acceptable and probably correct outcome — say which you chose and why in the DEC. This breaks
  the **blocking** `clippy-fmt-clean` constraint today, and `-D warnings` cannot see it.
- **Do not treat 4.0 as validated headroom.** The margin above the graphic class is **0.16 bits**
  (Canon-frame dither 3.8396), not the 0.36 the record implied, and the true ceiling is unknown
  and filed in `docs/backlog.md`. Do not move the threshold — but do not lean on it either.
- **Check both `pipeline.run` sites.** `:989` is the one the findings name; there is another at
  `:160`. If it also feeds a classify path, fixing one leaves a second door open. Cite the grep
  when you claim it does not ([[mechanical-sweeps-need-a-mechanical-check]]).
- **SPEC-084 makes no never-bigger-than-source promise on the metadata-forced branch.** Read
  `src/cli/optimize.rs:1043-1059` before assuming otherwise; this cost the SPEC-109 build two red
  tests to learn from the code.
- **Reverting a mutation does not rebuild the binary.** After any threshold experiment,
  `./target/**/crustyimg` stays mutated until something triggers a rebuild — SPEC-109's verify
  hit this and briefly saw *every* photo fixture report `graphic-logo`. Rebuild and confirm
  `Compiling crustyimg` before taking any measurement. A result that dramatic is a build-state
  symptom, not a classifier symptom.

## Verify before handing back

**Clean full matrix from a fresh `CARGO_TARGET_DIR`** — this is shared engine code and an
incremental build false-greens here (it cost this repo about a day on SPEC-105):

```bash
cargo test --no-default-features && cargo test && cargo test --features webp-lossy
cargo clippy --all-targets --no-default-features -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features webp-lossy -- -D warnings
cargo fmt --check
```

Confirm the log says `Compiling crustyimg` on each leg. Reference totals after #114:
**776 / 796 / 803** passed. Your numbers should exceed these; report them.

**Also re-run SPEC-109's mutation control.** With `PHOTO_ENTROPY_STRONG = 5.5` the analysis
suite must still go RED. If your change makes that guard green again, you have broken the
instrument, and that is a blocking defect regardless of how the fixtures behave.

## Repo guardrails

- **Every commit signed off (`git commit -s`).** DCO is enforced and has gone red three times.
- **Never `git reset --hard`.**
- **`rtk` silently corrupts output** — it has dropped the newest commit from `git log`, returned
  "0 matches" against files that plainly match, and mangled `ls`/`cargo`. It is *intermittent*,
  so a clean comparison proves nothing about the next call. Cross-check anything load-bearing
  with `python3` or plain `git`, plus a positive control that must return nonzero.
- **`just advance-cycle` / `just archive-spec` mis-target `specs/prompts/*.md`** — `git mv` by hand.
- **Work in a git worktree if any other session is open on this repo**, and check `git status`
  before assuming the tree is yours. This was violated once already this project and cost a
  verify session a corrupted 7-minute window.
- **Do not open or merge the PR.** Maintainer's call.

## When you finish

Fill in `## Build Completion` in the spec and the three reflection questions. Update the
timeline's `build` line.

### Cost — follow THIS, not `cost-snippet.md`

⚠ **`projects/_templates/prompts/cost-snippet.md` on `main` is stale and its instructions are
unsatisfiable.** It says to leave `tokens_total` null "because the orchestrator fills it from
the Agent result's `subagent_tokens`" — but an interactive cycle has no `subagent_tokens`, and
`just cost-audit` rejects a null on a metered cycle at ship. Following it leaves inventing a
number as the only way to satisfy both rules, which is exactly what happened on SPEC-109's build
before it was caught. The corrected snippet and its decision record (**DEC-083**) are on the
unmerged branch `chore/cost-measurement-methodology`. **Ignore the version you find on `main`
and use the following instead.**

**Measure, do not estimate.** Your session transcript carries per-message `usage`:

```
~/.claude/projects/<cwd-slug>/<session-id>.jsonl
```

Sum `input_tokens`, `output_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens` over every line with `.message.usage`; `tokens_total` is all four.
Duration comes from the first and last `timestamp`, model from `.message.model`. The session id
is the last path component of your scratchpad directory. If the transcript is genuinely
unreadable, write `tokens_total: null` **and say so explicitly** — a stated gap is fine, an
invented number is not.

**Price by component, not by a flat rate.** At Opus anchors ($5 input / $25 output per MTok):

```
input          x1.00 input rate
output         x1.00 output rate
cache_creation x1.25 input rate
cache_read     x0.10 input rate
```

A flat `tokens_total × list rate` overstates a long agentic cycle by ~14× — cache reads
dominate volume (SPEC-109's build: 98.7% cache reads, $588 flat vs $43.21 by component; its
verify: 96.3%, $190.37 vs $17.76). Record the anchors you used.

**Close your return message with this block, verbatim, as the last thing you emit:**

```
## Cost readout
cycle:            build
spec:             SPEC-108
agent:            <model id>
tokens_total:     <n>
breakdown:        in <n> / out <n> / cache-write <n> / cache-read <n>
duration_minutes: <n>
estimated_usd:    <n>
source:           transcript sum over <n> assistant messages | subagent_tokens | UNAVAILABLE (<reason>)
```

**Report what you could not do as clearly as what you did.** A stated gap is worth more than a
green tick that quietly skipped something.
