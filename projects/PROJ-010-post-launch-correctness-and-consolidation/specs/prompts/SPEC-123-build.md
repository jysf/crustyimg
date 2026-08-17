# SPEC-123 — BUILD prompt

Cycle: **build**. You are NOT the architect. The design is settled; run the measurement.

**One-line summary:** three shipped things — `build --frozen`, the lockfile's `hash`, and the
cache key — assume AVIF output is byte-stable on one machine. Thread count is in none of their
qualifying lists. Find out whether varying it changes the bytes.

**This spec ships no behaviour.** Its deliverable is a table of hashes, a verdict, and a decision
record. AC-7 requires `git diff main` to show **no functional `src/` change** — if you are keeping
a source edit, you have left the spec.

## This spec is self-contained, and nothing is blocked behind you

You run in isolation, first of three serial specs. SPEC-121 and SPEC-122 follow you and touch
`src/operation/mod.rs` — **you touch no `src/` at all**, so there is zero overlap in either
direction. Finishing early only unblocks two roadmap items (`par_iter run_pixel_op`, encoder
threading); it does not put anyone on hold.

## Read in order

1. **The spec** — `projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-123-avif-byte-determinism-across-thread-counts.md`,
   in full. **8 ACs, 4 settled design calls, no failing tests.**
2. **`docs/backlog.md`**, the AVIF-determinism entry — the claim, and its "measure before claiming
   either way" instruction.
3. **The code** — `src/build/lock.rs:28-40` (what `hash` promises and what it explicitly does
   not); `src/cli/build.rs:294-300` (`cache::compute_key`'s six components); `src/sink/mod.rs:667-684`
   (the AVIF encode arm).
4. **DEC-077 / SPEC-091** — why AVIF *decode* is pinned to one thread. **Different code path.**
   Read it so your write-up does not conflate encode determinism with the decode race.
5. **`/AGENTS.md`** — §4 cost, §6 commands, §12 testing, §13 git/PR, **§15**.

## ⚡ The lever is not where the spec assumes — verified at design time

**Do not spend an hour discovering this.** Three facts, read out of the source today:

1. **crustyimg never calls `with_num_threads`.** `src/sink/mod.rs:679` constructs
   `AvifEncoder::new_with_speed_quality(&mut cursor, s, q)` and nothing else. `image` 0.25.10
   exposes `with_num_threads` (`codecs/avif/encoder.rs:90`) and documents the default as
   *"use all threads in the default `rayon` thread pool."* We take that default everywhere.
2. **So the thread count is the rayon pool's size, resolved at encode time.** `ravif` 0.13.0
   `av1encoder.rs:653` does `p.threads.unwrap_or_else(rayon::current_num_threads)`. That is your
   observable: the encoder reads the ambient pool.
3. **The two levers reach different verbs, and one of them is a trap.**
   - `RAYON_NUM_THREADS` sizes the *global* pool → the lever for the serial verbs.
   - `--jobs` builds a *scoped* pool and `install`s the work — read in exactly two places,
     `src/cli/build.rs:661` and `src/cli/optimize.rs:177`.
   - ⚠ **`--jobs` is silently ignored by six verbs, `convert` among them** (STAGE-042 backlog
     item, filed 2026-08-16). `crustyimg convert -j 1` accepts the flag, warns nothing, and
     changes no pool. **A matrix built on `--jobs` for `convert` measures one thread count three
     times and reports a confident false "deterministic".**

**Use `RAYON_NUM_THREADS` as the primary lever.** It reaches all three verbs uniformly, including
`convert`, where `--jobs` is inert. Run `--jobs` as a **second, confirming leg on `optimize` with a
single input** — batch size 1, so the only thing varying is the encoder's pool and not the file
fan-out. Both call sites do `pool.install(..)` (`optimize.rs:181`, `build.rs:666`), so nested rayon
work should inherit the scoped pool; confirm that it does rather than assuming it.

**Establish which lever reaches each verb before you collect a single hash.**

## ⚡ Thread count is an ENCODER PARAMETER, not a scheduling detail

Read `ravif` 0.13.0 `av1encoder.rs:651-655` before you design the matrix:

```rust
let tiles = {
    let threads = p.threads.unwrap_or_else(rayon::current_num_threads);
    threads.min((p.width * p.height) / (p.speed.min_tile_size as usize).pow(2))
};
```

**The ambient thread count sets the AV1 tile count.** Tiles are a bitstream-level partitioning —
tile boundaries reset entropy-coding contexts — so a different tile count is a different bitstream
**by construction**, before rav1e's nondeterminism bug (#2781) enters the picture at all.

So the expected verdict is **non-deterministic, for a structural reason**. That is a prior, not a
result: **you still have to measure it.** A plausible answer is not a checked one, and this repo
has been wrong about exactly that before. But budget for AC-6 firing — the correction sweep is the
likely path, not the unlikely one.

### The third false-null mechanism — the clamp

That `.min(..)` is a trap the spec does not list. `tiles = min(threads, (w*h) / min_tile_size²)`,
and at crustyimg's default speed 6, `min_tile_size` is **128 or 256** depending on ravif's
`high_quality` gate (`quantizer > quality_to_quantizer(80.)`, `:544`) — which the quality search in
`optimize`/`web` will move around under you.

Computed for the corpus:

| input | size-term at 128 | size-term at 256 |
|---|---|---|
| `graphic_large.png` (512²) | 16 | **4** |
| `photo_forest_cc0.jpg` (800×532) | 25 | **6** |

On an 8-core machine, a 1 / 4 / 8 matrix over `graphic_large.png` in the 256 case clamps to tiles
**1 / 4 / 4** — two legs byte-identical, the timing control fires anyway, and the table reads as
"deterministic above 4 threads" when it is only the clamp.

**So: compute the size-term for each input and quality you use, report it next to the hashes, and
pick inputs where the thread term binds across your whole range.** A hash table without its clamp
column is not interpretable.

### Record OUTPUT SIZE beside the hash — one column, and it answers a second question

You are already producing these artifacts at each thread count. **Record their byte size too.**

Tiles are coded independently, so more tiles should cost compression efficiency — ravif's own
comment concedes it: *"AV1 needs all the CPU power you can give it, except when it'd create
inefficiently tiny tiles."* If that holds, **crustyimg's quality-per-byte today varies with the
machine's core count**, which matters rather a lot on this tool.

That is a prediction from reading the source, **not a measurement**. Your table converts it into
one for the cost of a `wc -c`. Report it whichever way it comes out — including "no material size
difference", which is equally useful and would close the question.

## Call 4 is load-bearing, not the cheap adjacent extra

The spec frames run-to-run stability at a fixed thread count as one extra loop. **It now decides
something.** If tiling explains the variance, then pinning the thread count would remove it — and
whether that is *sufficient* depends entirely on whether output is stable run-to-run once the count
is fixed. If it is, a pin is a real fix; if it is not, there is residual nondeterminism underneath
(the #2781 shape) and a pin only narrows the problem.

**Run it with enough repeats to mean something** — three is not a stability claim — and report it as
its own result, not a footnote.

## Call 1 in detail, because the whole spec rests on it

A "deterministic" verdict is the one **most likely to be wrong for a boring reason**: the setting
never reached the encoder. That failure mode and a genuine pass produce *the same output* — an
identical hash across all legs — so the hashes alone cannot tell them apart.

**Prove the variable moved.** Any of these, and say which you used:

- **Wall-clock / CPU-time delta** between 1 thread and all cores on an encode big enough for the
  difference to clear the noise. Report the numbers, with repeats — one timing is not a delta.
- **`rayon::current_num_threads()` observed** in the process that encodes.
- **Process-level thread instrumentation** while the encode runs.

**A single-digit-percent timing wobble is not evidence the variable moved.** If you cannot show
the work changed, the verdict is Call 3's *third* branch, not its first.

## The three outcomes — two of them are findings

| observation | verdict | what it costs |
|---|---|---|
| hashes differ across thread counts | **non-deterministic** | AC-6 fires: **every shipped "reproducible" claim is located and corrected in this PR.** A false claim must not outlive its disproof. |
| hashes identical **and the control fired** | **deterministic** | the language is safe; threading work unblocked on this axis |
| hashes identical, **control did not fire** | **the encoder ignores the thread setting** | moot today, live the moment anyone sets it. **Record it as this, not as "deterministic."** |

Report the hashes first and let the verdict follow from the table. Do not lead with the verdict.

## Coverage

- **At least three thread counts** (1, a middle value, all cores) — AC-1.
- **Three verbs** through the shipped binary: `convert --format avif`, `web`, `optimize` — AC-3.
  Not a library harness; the claim under test is about crustyimg's output, not rav1e's.
- **The lean build** — `--no-default-features` drops the AVIF encoder entirely, so that leg
  cannot produce the artifact at all. Confirm what it does instead; that is part of the answer,
  not a gap in it.
- **Run-to-run at a fixed thread count** — AC-4, Call 4. One extra loop, re-confirms the narrower
  claim the lockfile actually makes.
- Inputs from `bench/corpus/` (`photo_forest_cc0.jpg`, `graphic_large.png`).

## What "done" looks like

- **The hash table**, per verb × thread count, with the control's evidence next to it.
- **A verdict** stated as exactly one of the three rows above.
- **The harness committed** — under `scripts/`, shaped like `scripts/spec120_linear_light.py` — so
  the numbers can be **re-derived rather than trusted**. Re-run it once and confirm it lands in the
  same place (AC-8).
- **DEC-094** (ID reserved for you — see Guardrails). `affected_scope`: `src/sink/**` if the
  answer constrains encoding, `[]` if it is purely a documentation finding.
- **The result appended to `docs/backlog.md`**'s determinism entry.
- If non-deterministic: the corrections, **with the grep cited** — its scope is a claim too.

## Guardrails

- **Own git worktree**, branch `chore/spec-123-avif-byte-determinism`. Do not work in the primary
  checkout; several sessions have been live in this repo.
- **Your DEC is DEC-094. The ID is reserved — do not run `next_id`.** It scans only the working
  tree, so a record on an unmerged branch is invisible to it; SPEC-119 and SPEC-120 both minted
  DEC-092 that way. DEC-095 is reserved for SPEC-121/122. Take 094 and nothing else.
- **Budget in exchanges, not minutes.** This is an **S** — past **~120 exchanges** without a hash
  table in hand, checkpoint and report what you have. Cost scales with the *square* of message
  count and anti-correlates with wall clock: SPEC-116 ran 104 minutes for $11.91, SPEC-119 ran 61
  for $51.24. A slow careful run is cheap; a chatty one is not.
- **A piped command reports the pipe's exit code** — redirect and read `$?`.
- macOS has no `timeout(1)`. `git commit -s` (DCO). Never `git reset --hard`.
- **Do not merge the PR. Do not bump the version.**

## When you finish, in this order

1. Fill in the spec's `## Build Completion`, including its three reflection questions.
2. Append a build cost session entry to `cost.sessions` (see below).
3. Write **DEC-094**, with `affected_scope` set per the verdict.
4. Run `just advance-cycle SPEC-123 verify`, and **CONFIRM it moved** — `git diff` on the spec
   should show the `cycle:` line change. It reports success even when it changes nothing.
5. Open the PR. **Do not merge it.**

### Cost

Follow `projects/_templates/prompts/cost-snippet.md`. Identify your transcript by something only
your session emitted — **never by "the newest `.jsonl`."** A sub-agent once priced the
orchestrator's session as its own. Price per component at the anchors of the model
`.message.model` actually reports (you are expected to be **Opus**: $5/$25 per MTok;
cache_creation ×1.25 input, cache_read ×0.10 input).

**Measure at session end, not mid-session.** Measured twice in this project: SPEC-114 reported
$25.75 mid-run and finished at $34.31; SPEC-117 reported $11.76 mid-run and finished at **$23.06**
— 49% low. Re-measure as the last thing you do.

Close with the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not do as clearly as what you did.** An unmeasured leg named is worth more
than a table that implies coverage it does not have.
