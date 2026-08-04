# SPEC-110 — BUILD prompt

Cycle: **build**. You are NOT the architect. The design is settled; your job is to implement it.

**One-line summary:** seven verb invocations hand back a sideways image *and* drop the EXIF that
described the rotation. Pin the existing `auto-orient` operation first on every pixel-lane verb,
so one rule is true across the whole surface.

## Read in order — deliberately short

1. **The spec** —
   `/projects/PROJ-010-post-launch-correctness-and-consolidation/specs/SPEC-110-orientation-baked-on-every-pixel-lane-verb.md`,
   in full. **11 acceptance criteria, 9 pre-written failing tests, one negative control.**
2. **The code** — `src/cli/optimize.rs:781-798` (`optimize_pipeline()`, the prefix to reuse),
   `:507-542` (`run_convert`, the empty `Pipeline::new()` at `:538`), `src/cli/ops.rs`
   (`run_pixel_op` and the `resize`/`thumbnail`/`edit` handlers), and the `responsive` handler.
3. **`tests/common/mod.rs:102-172`** — `jpeg_with_orientation` and `wrap_with_orientation_app1`.
   Use these. Do not hand-roll a TIFF.
4. **`/AGENTS.md`**, these sections only — §4 cost, §6 commands, §12 testing, §13 git/PR.

## The measured facts, inlined

Release build at `d854038`. A JPEG stored **1200×800** with a real one-entry `Orientation=6`
IFD; correct **display** size is **800×1200**.

| verb | output | expected if baked | | EXIF kept? |
|---|---|---|---|---|
| `convert --format png` / `jpeg` | 1200×800 | 800×1200 | **not baked** | no |
| `resize --max 600` | 600×400 | 400×600 | **not baked** | no |
| `thumbnail --size 300` | 300×200 | 200×300 | **not baked** | no |
| `edit --invert` | 1200×800 | 800×1200 | **not baked** | no |
| `edit --resize-max 600` | 600×400 | 400×600 | **not baked** | no |
| `responsive --widths 600` | 600×400 | 600×**900** | **not baked** | no |
| `web` / `optimize` / `auto-orient` / `edit --auto-orient` | 800×1200 | 800×1200 | baked ✓ | no |

**`resize` is the worst case for users, not `convert`.** The `--max` bound is applied to the
wrong axis today, so the output is the wrong *size*, not merely mis-rotated. Lead your PR
description with that.

**The decision is made — do not re-litigate it.** Bake, do not preserve. The rejected
alternatives and the reasoning are in the spec's Context. The sub-decision is that
`edit --auto-orient` becomes an accepted, documented no-op (the CLI surface is frozen, so it
cannot be removed), and **no opt-out flag is added**.

## The three traps in this change

1. **Double rotation.** `web`/`optimize` already push `orient` via `optimize_pipeline()`. If a
   shared prefix is applied on top of that existing push, a 90° case comes back 180° off. AC-2
   is the guard.
2. **A square fixture makes the entire spec vacuous.** It cannot distinguish baked from
   not-baked from baked-twice. Use non-square throughout (AC-4).
3. **`responsive` is width-pinned**, so a baked result is 600×**900** — not a dimension swap. A
   test copied from the `resize` case asserts the wrong thing.

## Notes

- **Reuse, do not copy.** `optimize_pipeline()` already builds the prefix. Factor it so exactly
  one place knows "pixel-lane pipelines start with auto-orient" (AC-7) — six copies is how the
  next verb gets added without it.
- **Orientation 1 / no-EXIF must be byte-identical to before** (AC-3), asserted on **bytes**,
  not dimensions. That is what makes this change safe for the overwhelming majority of inputs.
- **All eight orientation values** get driven through one verb (AC-5): 5/6/7/8 swap dimensions,
  1/2/3/4 do not. A fix that handles only 6 is not a fix.
- **AC-9 is real work, not a nicety.** DEC-003 asserts *"Orientation/ICC survive transforms"*
  and writes its success test as *"Right if: a resize preserves orientation…"*. That is now
  false. Amend the record (dated, with reasoning) and correct `AGENTS.md:448`'s glossary line.
- **Drive every verb; do not infer from the call graph.** SPEC-107's follow-up list was wrong in
  **both** directions because it reasoned from `run_pixel_op` membership instead of running the
  binary. The table above was produced by driving it.
- **If you find ICC is also being dropped against DEC-003, report it — do not fix it**
  (`one-spec-per-pr`). Same for any classification change; classification only runs on the
  `optimize`/`web` decide path, which this spec does not touch, so a change there is a finding.

## Verify before handing back

Clean full matrix, fresh per-leg `CARGO_TARGET_DIR`, **sequentially** (never both shared and
parallel), **every leg through `rtk proxy` from the first one**:

```bash
cargo test --no-default-features && cargo test && cargo test --features webp-lossy
cargo clippy --all-targets --no-default-features -- -D warnings
cargo clippy --all-targets -- -D warnings
cargo clippy --all-targets --features webp-lossy -- -D warnings
cargo fmt --check
just wasm-test
```

Confirm each log says `Compiling crustyimg`. rtk has collapsed `cargo test` output and deleted
that exact line — treat a missing `Compiling` line as a tooling failure first, a build-state
question second. Reference totals on `main`: **lean 797 / default 816 / webp-lossy 823**,
`just wasm-test` 30/30. Reconcile your delta against the tests you add.

**Then read the CI legs on your PR before claiming the matrix is clean.** SPEC-107 shipped a red
Windows leg behind a "full matrix clean — all matching the reference exactly" claim; the local
macOS run was green and nobody looked. A local matrix is not the required matrix.

Also run AC-10's negative control and record it: remove the `auto-orient` prefix from `convert`,
confirm a test goes RED, restore. **Reverting source does not rebuild the binary** — prove the
revert reached the artifact, not just the file.

## Repo guardrails

- **`git commit -s`** on every commit. DCO is enforced.
- **Never `git reset --hard`.**
- **`rtk` silently and intermittently corrupts output** — cross-check anything load-bearing with
  `/usr/bin/git` or `python3` plus a positive control that must return nonzero.
- **macOS has no `timeout(1)`.**
- **Own git worktree**; check `git branch --show-current` before any commit.
- **Do not merge the PR.**

## When you finish

Fill in `## Build Completion` and the three reflection questions. Update the timeline's build
line. Create the new DEC for "bake on every pixel-lane verb", with `affected_scope` filled in.

### Cost

Follow `projects/_templates/prompts/cost-snippet.md` on `main` — it is current. Price **per
component** at the anchors of the model that **actually ran** (read `.message.model` from your
own transcript). Close with the `## Cost readout` block, verbatim, as the last thing you emit.

**Report what you could not do as clearly as what you did.**
