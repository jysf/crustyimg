# SPEC-113 — BUILD prompt

Cycle: **build**. You are NOT the architect. The design is settled; implement it.

**One-line summary:** `optimize x.jpg -o out.jpg` on an already-compressed JPEG returns a file
**2.02× larger, exit 0, empty stderr** — because the pinned branch never reaches the never-bigger
guarantee the auto path has. Keep the source, and say so.

**This ships before the launch post.** The post names `optimize`, and a silently doubled file is a
top comment rather than a bug report.

## Read in order

1. **The spec** — `projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-113-optimize-never-silently-grows-a-pinned-output.md`,
   in full. **10 acceptance criteria, 7 pre-written failing tests, one negative control.**
2. **The code** — `src/cli/optimize.rs:584-640` (the fork at `:616-631`);
   `src/analysis/decide.rs:843` + `pick_winner` (the comparison to reuse and the note's voice);
   `src/cli/ops.rs` `run_pixel_op`.
3. **`docs/cli-reference.md:140,152,412`** — the documented claims. AC-10 is real work.
4. **`/AGENTS.md`** — §4 cost, §6 commands, §12 testing, §13 git/PR.

## The driven fact

On the released 0.7.0 binary, source made with `sips -s formatOptions 15` (an independent tool):

```
crustyimg optimize low.jpg -o out.jpg     41,862 B → 84,586 B   exit 0, stderr EMPTY
crustyimg optimize low.jpg --out-dir auto  41862 → 52713 B (26% larger) + an explanatory note
```

Same input, same engine. The auto path reports; the pinned path does not, and does not guard.

## The design call is SETTLED — do not reopen it

**`--profile preserve` stays exempt; the pin does not.** `preserve` is the documented engine-off
regression anchor (DEC-048) whose job is to reproduce format-preserving behaviour *exactly*; giving
it a never-bigger guard would destroy the property it exists to provide. A pinned `-o out.jpg` is
not an anchor — it is a user asking for a JPEG.

**No new flag. Do not implement both behaviours.** AC-4 pins `preserve`'s exemption with a test, so
the decision is claimed rather than assumed.

## The trap in this spec — find it BEFORE writing the guard

The auto path's note lists reasons the source could not ship verbatim — *metadata stripped /
orientation baked / resized to the requested bound*. **If any of those apply, the source bytes are
not a valid output and keeping them would be wrong.** `optimize` bakes orientation and drops
metadata via `optimize_pipeline`. So determine whether the pinned path can be in that state; if it
can, the guard's condition is **narrower than "output > source"**.

Getting this wrong produces a fix that is quieter and worse than the bug — exactly what SPEC-111
nearly shipped. Work it out first.

## Two more traps

1. **AC-6.** "Always keep the source" passes AC-1 and destroys the verb. Assert that a genuinely
   smaller re-encode still writes the **new** bytes.
2. **AC-5.** `--format jpeg` and `-o out.jpg` are two spellings of the same pin and both reach
   `:616`. A fix applied to one is the unenumerated-cell defect this project keeps finding.

## Notes

- **The comparison already exists** in `decide::pick_winner`. Reuse the judgement; do not drag the
  whole decision engine into the pinned branch — a pin has no candidates, only re-encode vs source.
- **Match the auto path's voice** in the message. Plain, behaviour-first, **no SPEC/DEC references
  in user-facing strings**.
- **At least one new test must FAIL on `HEAD`.** If they all pass before your fix, they do not cover
  the bug.
- Commit the fixture, and **generate it with an independent tool** — never with crustyimg.

## Verify before handing back

Full matrix, fresh per-leg `CARGO_TARGET_DIR`, **sequentially**, **through `rtk proxy` from the
first leg** (it has deleted the `Compiling crustyimg` line and mangled binary through `cat` — treat
a missing one as a tooling failure first; use `/bin/cat` for binary). Reference on `main`:
**lean 821 / default 841 / webp-lossy 847**. Reconcile your delta against the tests you add.

**A piped command reports the pipe's exit code** — `cargo test | tail` turns a red leg green.
Redirect to a file and read `$?`.

Run AC-8's negative control and record it: revert the guard, confirm the AC-1 test goes RED,
restore. **Prove the revert reached the built artifact** — a changed binary hash shows a rebuild,
driving shows the change took effect.

**Then read the CI legs on your PR individually before claiming green.** SPEC-107 shipped a red
Windows leg behind a "full matrix clean" claim from a local macOS run.

## Repo guardrails

`git commit -s` (DCO enforced). Never `git reset --hard`. **Own git worktree — two other sessions
are live in this repo right now** (a docs/recipes session and a demo look-and-feel session), so do
not work in the primary checkout and do not assume `target/` is yours. macOS has no `timeout(1)`.
Cross-check anything load-bearing with `/usr/bin/git` or `python3` plus a positive control.
**Do not merge the PR. Do not bump the version.**

## When you finish

Fill in `## Build Completion` and the three reflection questions.

### Cost

Follow `projects/_templates/prompts/cost-snippet.md`. **Identify your transcript by something only
your session emitted — a probe symbol, your agent id — never by "the newest `.jsonl` in the
directory."** SPEC-112's build did the latter, read the parent orchestrator's session, and reported
the wrong model *and* the wrong volume while confidently flagging a mismatch that did not exist.
Price **per component** at the anchors of the model `.message.model` actually reports. Close with
the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not do as clearly as what you did.**
