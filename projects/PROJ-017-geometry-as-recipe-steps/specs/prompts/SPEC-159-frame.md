# SPEC-159 — FRAME prompt: `pad` with a transparent fill, and the no-alpha output rule

Cycle: **frame**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-159-frame -- --model opus
```

**Reserved: SPEC-159.** PROJ-011's STAGE-063 reserves SPEC-154 to SPEC-158, and SPEC-160 is
reserved for `edit --pad`/`--trim`, which follows this spec. Before you allocate, confirm SPEC-159 is
unused everywhere:

```
for b in $(git for-each-ref --format='%(refname)' refs/heads refs/remotes); do /usr/bin/git grep -l 'SPEC-159' "$b" -- . ; done
```

The expected hits are this prompt and STAGE-058. If a spec file or another reservation uses
SPEC-159, stop and report. Don't pick another number. ⚠ `just new-spec` skips any id named in prose,
so it may offer SPEC-161 or later; take SPEC-159 anyway (see "Allocate the id").

Your item is the STAGE-058 Spec Backlog entry "**`pad` with a transparent fill, and what
`transparent` does in a format with no alpha.**" Read it in full, with its two rulings. Other entries
bear on it and are named below; touch no item except as this prompt says.

## Why this spec exists
STAGE-058 promises a `pad` step "filled with a colour or transparent". SPEC-151 shipped `pad` with an
opaque fill only and refuses the rest at load (`src/operation/pad.rs`, about :149: "a fill with
transparency is not supported yet"). The maintainer ruled on 2026-11-10 that this spec ships
**before** `edit --pad`/`--trim` (SPEC-160), so `--pad-color` takes `transparent` the day that flag
ships, and SPEC-160 then closes the stage.

**Two rulings are settled (2026-11-06, SPEC-151's Q1 and Q4); don't re-open them:**
- transparency is its own spec (this one);
- `transparent` into an output format with no alpha is **refused per input, exit 4, naming the step
  and the format**, not flattened and not left to the encoder. Drive GIF (1-bit transparency) too.

The frame's job is everything those rulings leave open.

## What already exists (read it, don't assume it)
- **`pad`** (`src/operation/pad.rs`, `tests/pad.rs` or wherever its tests live; grep): the aspect
  and border forms, the fill, how it builds the output buffer at the source's colour type, and the
  load-time refusal of `transparent` and `#RRGGBBAA` below `ff`.
- **`parse_color`** (`src/text/mod.rs`): what it returns for `RRGGBBAA`, and whether `transparent`
  is a keyword it knows or one `pad` checks itself. (Its `+f+f+f` sign bug is folded into SPEC-160,
  not here. Don't fix it; say if this spec's work would collide with it.)
- **DEC-095**: ops keep the colour type and bit depth, with the narrow rule that a channel with
  non-opaque samples is kept. Say exactly which widenings this spec needs (RGB8 → RGBA8, RGB16 →
  RGBA16, L8 → LA8, L16 → LA16; and an image that's already RGBA or LA), and whether DEC-095 already
  allows them or needs amending.
- **`output_format_for`** (`src/cli/ops.rs`, about :214) and its ladder (`--format`, `-o` extension,
  recipe `format`, preserved source). The entry says the format is known per input after decode and
  before the pipeline runs. Confirm it on each surface: `apply` with one input and with a batch,
  `build`, wasm `transform`. Name any surface where the output format is **not** known when the check
  must run.
- **`trim`** (SPEC-152): it already treats a fully transparent pixel as matching a transparent
  `color`, whatever colour sits under it. Say whether pad-then-trim with `transparent` round-trips.
- **`watermark`**: it already composites alpha. Say whether its colour handling is a model for the
  fill, or a trap.
- **The docs that state the opaque-only rule:** `docs/recipes.md` (about :116),
  `docs/api-contract.md` (about :478), `skills/crustyimg/SKILL.md`, and `pad`'s doc comment. Grep
  for `opaque` and `transparent` across `docs/`, `skills/`, `README.md` and `src/`.

## Questions this frame must answer
1. **The fill.** What `transparent` puts in each pixel: which colour under zero alpha
   (`#00000000`? the edge colour?), and why it matters (premultiplied encoders, later resize, `trim`
   matching). What a partial alpha such as `#ff000080` means: straight alpha, stored as given. And at
   16 bits: how an 8-bit `AA` scales (`AA * 257`?), consistent with how `trim` scales its colour.
2. **Widening.** For every source colour type, the output colour type when the fill has any alpha
   below opaque, and when it is opaque (no change from today: say what pins that). Whether an RGBA
   source with an opaque fill stays RGBA (it should).
3. **The no-alpha rule, made exact.** The ruling says refuse. Make it precise:
   - Which formats have no alpha, which have 1-bit alpha (GIF), and which keep full alpha. **Drive
     each** (see "What a good frame does here").
   - GIF: refuse, or allow only fully transparent or fully opaque fills? Recommend one, with what GIF
     does to a `#ff000080` pixel today.
   - What triggers the refusal: the recipe has a `pad` with a non-opaque fill, **and** the output
     format for this input can't hold it. Say whether a later step that makes the image opaque again
     (a second `pad` with an opaque fill over it? nothing does today?) lifts it, or whether the rule
     is a static check of the recipe. Recommend the static check unless the evidence says otherwise.
   - **The consistency question:** an RGBA PNG source written to JPEG with no `pad` at all. Drive
     what it does today. If it silently drops alpha, the refusal applies only when `pad` brought the
     transparency, and the docs must say so. Recommend whether that asymmetry is acceptable, and
     file the source-alpha case as its own entry if it isn't. Don't fold it in.
   - Where the check lives on each surface, and its message (naming the step index, `'pad'`, the
     fill and the format). In a batch: does one refused input stop the batch or skip that input,
     consistent with how other per-input refusals behave today? Read the STAGE-058 entry "The batch
     summary says 'watermark skipped' for any refusing op": this refusal would be a second op that
     triggers it. Recommend whether that entry folds in.
4. **Is it a DEC?** This is the first step whose validity depends on the output format. Recommend
   whether that is a DEC (the next free id is DEC-115), what it says, and whether DEC-095 is amended
   or stands.
5. **What doesn't change.** Every opaque `pad` output is byte-identical, and so is every recipe that
   has no `pad`. Say what pins that today (the recipe corpus guard, SPEC-141; `pad`'s byte tests) and
   what more is needed.
6. **wasm.** `transform` must agree with native (DEC-110). Say whether the wasm build writes a
   no-alpha format at all, and whether wasm needs the refusal or can't reach it. Name any wasm row
   or count that changes.
7. **Fold-ins.** Read every open STAGE-058 entry and recommend fold or leave for each, with why. Fold
   only what doesn't change the size. Don't fold the `parse_color` sign fix (ruled into SPEC-160).
8. **Docs.** Every place that states the opaque-only rule or what `pad` fills with, and what each
   says after.
9. **Out:** say where each lives: `edit --pad` and `--pad-color` (SPEC-160), flattening against a
   colour (ruled out), the source-alpha-into-JPEG case if you file it.
10. **Size.** The backlog says [S]. Fill semantics × colour types × formats × surfaces may make it
    [M]. If it is, say what you would cut to keep it [S], and recommend.

---

## Rules every frame session follows

**You are the frame cycle only.** Your job is go/no-go plus a correctly sized, correctly scoped
spec stub. You do **not** write acceptance criteria, failing tests or Implementation Context, and
you do **not** write any DEC. You recommend its content. Separate design, build and verify
sessions follow. The orchestrator only sequences.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back
to the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables**: wide tables have lost lines in transit.

### Step 0 — get onto current `main`
You should be in your own worktree (made with `just wt <branch>`). Run `git fetch origin`, then
`git rebase origin/main`. If `git status` shows you are in the maintainer's main checkout rather
than a worktree, stop and say so.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 backlog, §13 git, §15 (its two newest verify rules bind
   every cycle: "A bound or 'can't happen' is a claim until its worst input is driven" and
   "Classify every finding") and §16 habit 6 (no CI wait loops).
2. `projects/PROJ-017-geometry-as-recipe-steps/brief.md`: the thesis, the build order, the risk on
   `pad`, and "Explicitly out of scope".
3. `projects/PROJ-017-geometry-as-recipe-steps/stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`:
   your item, every open entry, the `delivers` line for `pad` ("filled with a colour or transparent"),
   the exit criteria and the design notes.
4. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for"
   (traps 10, 12, 13, 14 and 17). PROJ-017 has no HANDOFF of its own, so these bind here. If the
   file isn't in your checkout, say so.
5. DEC-095, DEC-106, DEC-110, DEC-112, DEC-113, DEC-114.
6. SPEC-151, SPEC-152 and SPEC-153 in `projects/PROJ-017-…/specs/done/`, with their
   ship reflections. SPEC-152's proposes that a mutation row quote its exact mutant edit and that a
   count carry its derivation; SPEC-153's that build drive every doc sentence that states behaviour.
   Apply them in your stub where they arise.
7. `/guidance/constraints.yaml`.

### What a good frame does here
- **Check the premise against the code.** On a release build of `main` (fresh `CARGO_TARGET_DIR`,
  deleted at the end), a recipe `pad` with `color = "transparent"` and with `"#ff000080"`: record the
  exact refusal and exit code. Then drive today's alpha-into-JPEG behaviour on an RGBA PNG source
  with no `pad` at all (`apply` with `-o out.jpg`, and a batch with `--format jpeg`): quote what comes
  out, and the exit code.
- **Drive every output format** the CLI writes (read the list from the code, don't assume it): which
  keep alpha, which keep 1-bit alpha (GIF), which drop it. Quote each.
- **Size the verification, not just the code** (AGENTS §2): fill forms × source colour types (8/16-bit,
  grey, grey+alpha, RGB, RGBA) × output formats × surfaces (`apply` one input and a batch, `build`,
  wasm `transform`) × format sources (`-o` extension, `--format`, recipe `format`, preserved source).
- **Scope boundary:** what is in, what is out, and where each out item lives.
- **Rough build file list** (paths), and any fold-in's files.
- **Questions only the maintainer can answer:** one line each, with your recommendation.

### Allocate the id
Run `just new-spec "<title>" STAGE-058 PROJ-017`. If it gives anything other than SPEC-159,
`git mv` the spec and its timeline to SPEC-159 and fix the id inside both. Then run
`just advance-cycle SPEC-159 frame`. If you recommend a DEC, it takes the next free id (DEC-115 on
`main` today); recommend its content, don't write it.

### What to write into the stub
- Front matter: `type`, `priority`, `complexity` (your sizing), `value_link`, `references`
  (DEC-095, DEC-106, DEC-110, DEC-112, DEC-113, DEC-114, SPEC-151, SPEC-152, SPEC-153), and
  `cycle: frame`.
- `## Context` and `## Goal`: what a CLI user can't do today, and what they can when this ships.
- A `## Frame` section directly after `## Goal`. It holds:
  - the verdict (go / no-go / split);
  - the premise check, with evidence;
  - the format table, driven, and the refusal rule you recommend (and a DEC's content, if one);
  - the fill semantics: what `transparent` and `#RRGGBBAA` put in each widened colour type;
  - where each failure happens (load, before the pipeline, apply) and its exit code;
  - sizing and why;
  - scope in/out, including fold-ins;
  - the rough build file list;
  - open maintainer questions.
- `### Out of scope (for this spec specifically)`, filled from your scope boundary.
- Leave Acceptance Criteria, Failing Tests and Implementation Context as template placeholders.
- Timeline: mark **frame** `[x]` with a 2–4 line summary, and add `[ ]` lines for design
  (`prompts/SPEC-159-design.md`), build, verify and ship.

### Landing the frame
Frame commits go to `main` directly (AGENTS §13), with no PR.
1. On **go**, run `just advance-cycle SPEC-159 design`. On no-go or split, leave `cycle: frame`.
2. In STAGE-058's backlog, change your item's `- [ ] (not yet written)` marker to
   `- [ ] **SPEC-159** (framed <date>)`. If you recommend folding an entry, mark it
   `(folded into SPEC-159)` the way SPEC-152's folded entry was. On a split, file the second half
   as a new `- [ ] (not yet written)` entry right after yours. File any out-of-spec finding as a new
   `- [ ] (not yet written)` entry at the end of the stage it belongs to, and update that stage's
   `**Count:**` line with a grep you've just run. Something with no stage goes in
   `guidance/ideas.yaml` (AGENTS §10).
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Use this message:
   `docs(SPEC-159): frame pad with a transparent fill`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back what you pushed: `git show --stat HEAD` and the STAGE-058 diff.
   ⚠ `just backlog` reads only the **active** project (PROJ-011), so it won't list this spec.
   Don't change the active project to make it. Use `just status` and a grep instead.

### Guardrails
- ⛔ No code, no tests, no DEC, no version bump, no tag. A probe you drive at frame lives in a
  scratch directory and is never committed.
- Other sessions may be working at the same time. Never share a `CARGO_TARGET_DIR`. If
  `git rebase origin/main` conflicts in a file outside PROJ-017, stop and report.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate `grep` and
  `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Keep scratch files in your session's scratchpad, never in `~` or `/tmp`.
- **Budget ~40 exchanges.**

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`/usr/bin/grep -l "SPEC-159 — FRAME prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
`cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
**MAX** `output_tokens`.

Price each component at the model that actually ran (read `.message.model`), never at a flat
rate: Opus $5/$25 per MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate, and
cache_read is ×0.10. Measure at your last turn, after the push, and append a `frame` entry to the
spec's `cost.sessions` with `tokens_total`, a `tokens_breakdown`, `estimated_usd` and the rate
anchors in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-159 frame: GO | NO-GO | SPLIT`, then the commit SHA on `main`.
Then, short:
- the premise check (confirmed or wrong, with one line of evidence);
- today's alpha-into-JPEG behaviour with no `pad`, quoted;
- the format table: each output format and what it does with alpha, driven;
- the fill semantics for `transparent` and `#RRGGBBAA`, per source colour type;
- the refusal rule you recommend, where it is checked on each surface, and any DEC's content;
- refusals: where each fails, its exit code and message;
- whether any existing output changes (it should not);
- the fold-in candidates: folded or not, and why;
- size and why, scope in/out, and any split;
- the rough build file list;
- maintainer questions, each with your recommendation;
- any out-of-spec findings, and where you filed them;
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
