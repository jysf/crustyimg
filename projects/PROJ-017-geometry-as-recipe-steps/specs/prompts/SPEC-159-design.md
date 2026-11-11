# SPEC-159 — DESIGN prompt: `pad` with a transparent fill, and the no-alpha output rule

Cycle: **design**. Fresh CLI session, own worktree, **Opus**:

```
just wt docs/spec-159-design -- --model opus
```

This session starts from the stub the frame pushed to `main` (`53319f5`, cost in `496359a`, rulings
recorded after). **SPEC-159 writes DEC-115.** Confirm DEC-115 is still free
(`ls decisions/ | /usr/bin/grep DEC-115`, and a `git grep DEC-115` across every branch); if anything
else uses it, stop and report rather than pick another number.

**Read the stub's `## Frame` section first.** It's your binding scope. It holds:
- the premise check, and today's silent alpha drop into JPEG with no `pad`, driven;
- the format table over all eight output formats, driven;
- the fill semantics, the widening table (DEC-095 stands), and the no-alpha rule;
- DEC-115's recommended content;
- where the output format is known on each surface, read from the code;
- where each failure happens and its exit code;
- the verification sized (~80 unit cells, ~20 integration rows), scope, fold-ins (none), the docs
  list and the file list.

The maintainer will paste this prompt into a fresh CLI session and paste your final message back to
the orchestrator, so your final message must stand on its own. **Write it as plain lists, not
box-drawn tables.**

## Rulings (settled — do not re-open)
The maintainer accepted all four of the frame's recommendations on 2026-11-10 (recorded in the stub
under `### Maintainer rulings`):
1. **GIF:** fill alpha `00` and an opaque fill are allowed; `01`–`fe` is refused.
2. **`trim` accepts `transparent`** as `#00000000` in this spec, through one helper shared with `pad`.
   `parse_color` is **not** changed: SPEC-160 fixes its `+f+f+f` sign bug, so don't touch it.
3. **Source alpha is out.** Only alpha a `pad` brings is refused; an RGBA source written to JPEG still
   drops alpha silently, and the docs say so in one line.
4. **[M], uncut.**

Settled before that (2026-11-06): `transparent` into a format that can't hold it is **refused per
input, exit 4, naming the step and the format**. Flattening against a colour is ruled out.

Also accepted from the frame: `transparent` means `#00000000`; `#RRGGBBAA` is straight alpha stored
as written; 16-bit channels are `c × 257` and float `c / 255`; the check is **static** (recipe ×
format), not a scan of the output pixels.

## Another orchestrator may be building in nearby files
PROJ-011's STAGE-063 reserved SPEC-154 to SPEC-158 (an external code review). Its specs change
`src/sink/`, `src/metadata/`, `src/cli/build.rs`, and possibly `src/cli/common.rs`, `src/cli/ops.rs`
and `CHANGELOG.md`: three of those are in this spec's file list. At your Step 0, run
`gh pr list --state open --json number,title,files` and list every file another open PR or pushed
`feat/spec-15[4-8]*` branch touches that this spec also touches. Write that map into Implementation
Context **from the PRs' file lists, not from their specs' text**. Tell the build to rebase onto
`main` before each push and to re-check the map after each rebase.

## Things in the frame to check before you build on them
SPEC-152's design corrected eight things in a frame whose model was right. Drive these on a release
build of `main` (and on your prototype where the claim is about the change):
- **The auto-decide path.** The frame says no check is needed there because the shortlist drops JPEG
  for any alpha colour type and never offers GIF, and asks design to confirm the "passthrough (source
  kept)" branch can't fire after a `pad`. Drive it: a JPEG source, recipe `[pad transparent,
  optimize]` with no format pinned, through `apply` and `build`. Quote the output format and its
  alpha. Then find every way the decided format could still be JPEG or GIF (a source-kept branch, a
  size-budget fallback) and say what stops each.
- **Moving `output_format_for` before `pipeline.run`** in single-input `apply`. Read every input it
  takes. If any depends on the processed image (an output-name template that uses the output's
  dimensions, a format chosen from the colour type), the move changes behaviour: drive it and say.
- **The batch pre-check.** "When every input gets one format, check once before reading any input,
  exit 4, nothing written." Drive which spellings give one format (`--format`, a template extension,
  `recipe.format`) and what a mixed batch does today on another exit-4 refusal (a bad `--format`), so
  the new refusal matches an existing shape. Say what's on disk afterwards.
- **GIF at the threshold.** Drive `01` and `fe` on the binary through a prototype, not by argument
  (trap 17). Also `00` with a non-black colour (`#ff000000`): GIF stores alpha 0, but what colour
  comes back, and does `trim transparent` still match it?
- **TIFF is classed "full".** The frame drove that libtiff and ImageMagick read crustyimg's RGBA TIFF
  as opaque (no `ExtraSamples` tag; filed as `tiff-alpha-extrasamples`). A transparent pad into TIFF
  will look like a black border in those readers. Keep TIFF as "full" unless you find it wrong for
  crustyimg's own decoder, and make the docs line honest. If you think TIFF should be refused, don't
  decide it: put it in your report as a maintainer question.
- **16-bit and float fills.** Drive `#ff000080` on an RGB16 and an Rgb32F source and read the alpha
  back (32896 and `128/255`). Check `trim`'s scaling agrees, by a pad-then-trim round trip at 16 bits.
- **Static means static.** An image already at the aspect gets no fill pixel, but the recipe is
  still refused into JPEG. Make it a row. With two `pad` steps (opaque then translucent, and the
  reverse) the message names the first step that can't be stored: make that a row too.

## Design calls that are yours
- **DEC-115.** Write it in `decisions/` from `_template.md`: the static rule, the alpha capacity of each
  format, where it's checked per surface, the exit codes, the scope (alpha a step introduces; source
  alpha out), flattening rejected, and why the auto-decide path needs no check. Word it so a future
  op that can introduce alpha only adds itself to the check, with no amendment. Say how it relates to
  DEC-095 (unamended) and DEC-110 (wasm refuses the same pairs with the same line).
- **Where the "can this step write alpha?" question lives.** One function over the recipe (which step,
  which alpha class), one over the format (its capacity), one place that compares them and builds the
  message, called from every surface. Name them and where they live.
- **The shared keyword helper** for `pad` and `trim`, and what `trim`'s existing messages print when the
  colour was given as `transparent` (the keyword, or `#00000000`?). Pick one and pin it.
- **The exact messages.** Both refusal lines (JPEG, GIF), the wasm error, and the batch summary line,
  each verbatim, each naming the step index, `'pad'`, the fill as written and the format. Cite
  `file:line` for any wording you reuse. One AC row and one test per message. No message names a
  spec, stage or DEC id.
- **The byte pin.** Opaque `pad` output must not change. Beyond the SPEC-141 corpus guard, record
  decoded-pixel hashes on main **now** for an opaque pad on L8, RGB8, RGBA8 and RGB16 (seed the
  fixture independently of the code under test), write them in the spec as literals, and have build
  assert them before and after.
- **wasm.** Measure whether `WASM_BROTLI_BASELINE` moves on your prototype, and design the
  `tests/ddphotos_parity.mjs` rows (the frame recommends one transparent PNG row and one refused JPEG
  row).
- **Docs.** Re-check each surface in the frame's list at its `file:line`, find any it missed, and write
  what each will say. Each is an AC that verify checks **by driving the sentence's claim**, not only
  by reading it (SPEC-153's only defects were two doc sentences nobody drove).
- **Size.** The frame says [M]. Say whether that holds, with the count's derivation next to it.

## Tests: each one must be able to fail
HANDOFF traps 10, 12, 13, 14 and 17 and AGENTS §15 bind here. **Drive every mutation row at design on
a prototype; don't reason them.** Each row **quotes its exact mutant edit**, names the code location,
every test expected red and every test expected green, and is marked *driven* or *reasoned*. Read the
red set **off the prototype's run of the final test list**, not off the reasoning (SPEC-153's design
drove every row and still mis-stated three red sets, all in rows whose reds depended on which tests
use which inputs). A row marked "expected all green" names the search that found no splitting input.
At least:
- **The capacity table:** GIF as full; GIF as none; JPEG as binary.
- **The GIF threshold:** `alpha == 0` replaced by `alpha < 2`, and by `alpha <= 0xfe`.
- **A surface without the check:** drop it from single-input `apply`, from the batch's per-input path,
  from the batch pre-check, from `run_pixel_op`, from `build`, from wasm. Each needs its own red row.
- **The check made pixel-based** (scan the output for alpha instead of reading the recipe): which row
  reds? (The already-at-aspect row should.)
- **The 16-bit alpha scale** `× 256` instead of `× 257`.
- **Widening:** a grey translucent fill on L8 giving RGBA8 instead of LA8.
- **The keyword:** `trim` not using the shared helper.
- **Trap 14:** restore `main`'s `pad.rs` verbatim and list everything that goes red.

**Every count in the spec carries its derivation** (the command, or the sum), next to the number.

## Rules every design session follows

**You are the design cycle only.** You write Acceptance Criteria, Failing Tests, Implementation
Context and DEC-115. You don't write the code. Separate build and verify sessions follow: one builds
the spec from your text alone, and another verifies it, so everything the builder needs must be in
the spec (AGENTS §15).

### Step 0 — get onto current `main`
Run `git fetch origin`, then `git rebase origin/main`. If `git status` shows you are in the
maintainer's main checkout rather than a worktree, stop and say so. Then build the overlap map above.

### Read first
1. `/AGENTS.md`: §2 sizing, §8 cycles, §10 cross-refs, §12 tests, §13 git, §15 and §16 habit 6 (no
   CI wait loops).
2. `projects/PROJ-011-surface-reach-and-predictability/HANDOFF.md`: "Traps this wave paid for".
   PROJ-017 has no HANDOFF of its own, so these bind.
3. `projects/PROJ-017-geometry-as-recipe-steps/brief.md` and
   `stages/STAGE-058-lossless-geometry-crop-rotate-flip-pad-trim.md`.
4. `/guidance/constraints.yaml`, DEC-095, DEC-102, DEC-106, DEC-110, DEC-113, DEC-114.
5. SPEC-151, SPEC-152 and SPEC-153 in `specs/done/`, as the shape to match: Design, ACs, exact
   messages, mutation tables, Build Completion and all three ship reflections.

### How to design well here
- **Read the code before writing the spec**, and cite `file:line`: `pad.rs` (the load refusal, `wide()`,
  `unit()`, the output-type match), `trim.rs` (its colour read and `step`), `output_format_for`
  (`src/cli/ops.rs`), single-input `apply` and the batch in `src/cli/optimize.rs`, `encode_one` in
  `src/cli/common.rs`, `run_pixel_op`, `src/cli/build.rs`, `src/wasm.rs` `transform`, and the
  auto-decide shortlist (`decide.rs`, `has_alpha` in `src/image/mod.rs`).
- **Drive the current behaviour** on a release build of `main`, in a fresh `CARGO_TARGET_DIR` that
  you delete at the end. Give every measurement a negative control.
- **Count the nouns in every AC against the Failing Tests list.** An AC naming two things needs a
  test per thing.
- Comments and user-facing text read behaviour-first, with no spec or DEC ids in them.
- No version bump. Say in an AC whether `CACHE_SCHEMA_VERSION` or the lockfile format must change
  (a recipe that loaded as an error now loads: does the cache key care?), with the reason.

### Ids
Don't run `just new-spec`. Fill in Acceptance Criteria, Failing Tests and Implementation Context,
and keep `## Frame`. If you find the frame wrong, say so in a short `### Design corrections to the
frame` subsection at the top of `## Design`, and in your report.

### Landing the design
Design commits go to `main` directly (AGENTS §13), with no PR.
1. Run `just advance-cycle SPEC-159 build`.
2. In STAGE-058's backlog, change SPEC-159's marker from `(framed 2026-11-10)` to
   `(designed <date>)`, keeping `[M]` (or the size you found). Mark design `[x]` in the timeline with a
   2–4 line summary, and add `prompts/SPEC-159-build.md` to the build line.
3. Run `git commit -s`. **The DCO sign-off is mandatory.** Message:
   `docs(SPEC-159): design pad with a transparent fill; DEC-115 alpha the output can't store`.
4. Run `git fetch origin && git rebase origin/main`, then `git push origin HEAD:main`.
5. Read back `git show --stat HEAD`. `just backlog` reads only the active project (PROJ-011), so use
   `just status` and a grep instead.

### Guardrails
- ⛔ No code changes outside the spec, the timeline, the stage file and DEC-115. No pushed probe
  branch. No version bump, no tag. Never edit PROJ-011's files.
- In an interactive terminal, `cargo test` fails `display_sink_refuses_non_tty`, so redirect stdout
  to a file. A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does
  not word-split unquoted parameters. macOS has no `timeout(1)`. `rtk` can truncate `grep` and
  `git log` output, so use `/usr/bin/grep` and `/usr/bin/git` for anything you count.
- Never share a `CARGO_TARGET_DIR` with another session. Remove the prototype worktree and its target
  dir before you finish. Write scratch files in your session's scratchpad, never `/tmp`.
- **Budget ~60 exchanges** (driven mutation rows, the format drives and the byte pin are the extra).

### Measure your own cost
Your transcript is a JSONL under `~/.claude/projects/<slug>/`, where `<slug>` is your working
directory's absolute path with every `/`, `.` and `_` replaced by `-`. **Identify it by content,
not recency:**
`/usr/bin/grep -l "SPEC-159 — DESIGN prompt" ~/.claude/projects/<slug>/*.jsonl`.

⚠ **Dedup by `.message.id`.** Lines that share an id repeat the same `input_tokens`,
`cache_creation_input_tokens` and `cache_read_input_tokens`, while `output_tokens` grows. Take those
three once per id and the **MAX** `output_tokens`. Include any `subagents/` transcripts.

Price each component at the model in `.message.model`: Opus $5/$25 per MTok, Sonnet $3/$15.
cache_creation is ×1.25 the input rate, and cache_read is ×0.10. Measure at your last turn, after the
push, and append a `design` entry to the spec's `cost.sessions` in a final small commit.

### Report back (your final message — it is pasted to the orchestrator)
Plain lists only. First line: `SPEC-159 design: DONE | SPLIT | BLOCKED`, then the commit SHA on
`main`. Then, short:
- DEC-115's file name and its rule in one line per part;
- any correction to the frame, with what you drove (the auto-decide path, moving
  `output_format_for`, the batch pre-check, GIF at 01 and fe, TIFF, 16-bit and float fills);
- the byte-pin literals and how the fixture is seeded;
- the AC list (one line each);
- the Failing Tests and the mutation table, each row with its quoted edit, red and green tests, and
  driven or reasoned;
- every exact message, with its exit code;
- the docs surfaces, each with `file:line`;
- the overlap map with other open PRs and branches, from their file lists;
- size with its derivation, and anything the build prompt must carry (checkpoints, CI legs);
- your cost: input, cache_creation, cache_read, output, total tokens and dollars, each on its own
  line.
