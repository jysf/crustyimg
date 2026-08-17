# SPEC-119 — PUNCH LIST prompt

Cycle: **build** (punch-list return). Verify returned **⚠ PUNCH LIST** on PR
[#176](https://github.com/jysf/crustyimg/pull/176) with a clear headline: *"The implementation is
correct and I could not break it."*

**All three items are documentation. There is no code defect and no source change.** Verify
independently re-drove all 11 ACs against fixtures built **outside** this repo (ImageMagick, a
pure-Python APNG assembler) and validated with decoders the repo doesn't ship (`magick`,
`webpinfo`, `exiftool`), plus 96 byte-identity pairs against `main`. **Do not re-verify any of
that.**

## Your three items — all in `docs/api-contract.md` and the spec

### P1 — the Goal is not met, and the record says it is

`responsive` still silently flattens. Driven by verify: `responsive anim.gif --widths 16` writes a
**1-frame `anim-16w.gif` from a 4-frame source, exit 0, empty stderr** — same for APNG and
animated WebP. `apply --recipe <plain pixel recipe>` and `build` with a plain recipe are silent too.

Not a regression — `run_responsive` has its own `Image::load` (`src/cli/optimize.rs:1744`) and
misses the **truncated-JPEG** warning as well. **But the spec's Goal says "No shipped verb silently
discards frames", and Build Completion says "All acceptance criteria met? yes"**, which together
read as the Goal being met. It isn't.

**Fix the record, not the code:**
- Add a **`## Known residual`** subsection to Build Completion naming the three silent paths, that
  `run_responsive` drops both diagnostics, and that it is out of this spec's scope.
- Amend the spec's `## Goal` to say what was actually achieved — the five wired verbs — with the
  residual named.
- **Already filed on STAGE-046 by the orchestrator** as a `[M]` follow-up. Do not re-file it; do
  cross-reference it.

### P2 — the new api-contract paragraph is the thing its own neighbour warns against

The paragraph you added names **5 seams**. Verify drove the real map:

- **warns:** `convert`, `optimize`, `web`, `resize`, `thumbnail`, `auto-orient`, `edit`,
  `watermark`, `apply --recipe` (terminal-optimize), `build` (Decide)
- **silent:** `info`, `diff`, `responsive`, `apply`/`build` with a plain recipe

Three lines above yours, the truncated-JPEG paragraph explicitly says *"not an exhaustive-sounding
short list"* and then lists **both** sides. Yours lists neither correctly. **Rewrite it in the
neighbour's shape** — both sides, accurate.

*(Pre-existing and not yours: that older paragraph names `build` as unwired, but `build` warns for
both flags now. Fix it in passing since you are in the file, and say you did.)*

### P3 — the contract states a claim that is false in the shape CI uses

`docs/api-contract.md` says `lint --max-warnings 0` "fails on any of the three formats" **with no
qualifier.** Driven:

```
lint --max-warnings 0 <dir containing anim.webp>   → exit 0   "1 scanned · 0 warn"
lint --max-warnings 0 <dir>/anim.webp              → exit 7
```

Cause is the `IMAGE_EXTENSIONS` gap (`webp` absent from `src/source/mod.rs:105-113`) — **already
filed, do not fix it here.** But this is load-bearing: **Call 1's warn-and-proceed ruling was
accepted precisely because `lint --max-warnings 0` is the strict path**, and directory mode is the
shape a CI pipeline uses. **Add the qualifier** — naming the file or piping stdin works, directory
discovery currently skips `.webp` — and point at the STAGE-042 item.

## Also fix, small and quick

- **The spec's `## Failing Tests` names `animated_gif_still_writes_frame_one_and_exits_zero`,
  which does not exist** — folded into `animated_gif_warns_on_every_pixel_verb`. Correct the list
  and record it under **Deviations**. *(Verify confirmed the APNG/WebP tests never assert an output
  was written, and that it is written — so the coverage is real, the roster is wrong.)*
- **Narrow DEC-093's AVIF claim by one clause.** Verify read `avif-parse` 2.1.0
  `src/lib.rs:742-761`: the rejection keys on **major brand `avis` only**, and `_ =>
  skip_box_content` skips `moov`. A file with major brand `avif` that also carries an image
  sequence would parse as a still. **"PROVEN SAFE" is right for the `avis` case**; say that
  precisely rather than generally.

## Do NOT do these

- **Do not fix `responsive`.** It is filed as its own `[M]` item on STAGE-046.
- **Do not touch `IMAGE_EXTENSIONS`.** Adding an extension changes every decode caller and needs a
  caller-by-caller audit — verify explicitly ruled it right to leave out of this PR.
- **Do not re-run the matrix or the controls.** Verify re-drove all three negative controls and the
  full three-leg matrix independently.
- **Do not rebase.** The PR is green at `d66a357`.

## Two things about state you need to know

1. **Verify's own commit is NOT on the branch.** It ran on a detached HEAD and committed the cycle
   advance + cost as `c920cb9` without pushing. **Leave `cycle:` alone** — the orchestrator applies
   verify bookkeeping on `main` after the merge, per AGENTS §13. Do not run `advance-cycle`.
2. **The DEC is DEC-093, not DEC-092.** The orchestrator renumbered it in `d66a357` after it
   collided with SPEC-120's. If you see `DEC-092` anywhere that means *animated input*, that is a
   miss — report it.

## When you finish, in this order

1. Make the three documentation fixes plus the two small ones.
2. Amend `## Build Completion` with a `## Known residual` subsection and the Deviations entry.
3. **Do not add a cost session entry** — this is a continuation of the same build cycle. If the
   work is material, fold its tokens into the existing build entry and say so in the note.
4. Push to the same branch. **Do not merge. Do not bump the version.**

## Guardrails

- **Own git worktree.** `main` has moved repeatedly; do not work in the primary checkout, and check
  `git branch --show-current` before committing.
- `git commit -s` (DCO). macOS has no `timeout(1)`.
- **Budget: this is documentation. Well under an hour.** If it is taking longer you have misread
  the scope — re-read "Do NOT do these".
