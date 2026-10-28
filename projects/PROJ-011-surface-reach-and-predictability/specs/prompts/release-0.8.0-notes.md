# 0.8.0 — RELEASE NOTES prompt: CHANGELOG, recipes guide, agent skill, RELEASING

Cycle: **release prep, part 1 of 2** (docs only). Fresh CLI session, own worktree, **Opus**. The
work is a sweep across ten specs, and the prompt asks you to run every example you write.

```
just wt docs/release-0-8-0-notes -- --model opus
```

The maintainer pasted this prompt into a fresh session and will paste your final message back to
the orchestrator, so that message must stand on its own.

## Where things stand
- **The maintainer ruled (2026-10-27): 0.8.0 is everything on `main` since `v0.7.1`, and nothing
  else.** Every spec in it has shipped. The last one, SPEC-135, merged as `df91f7e` and was
  archived as `3d8e598`.
- **Part 2 is not yours.** A later session runs `just release 0.8.0`, rolls `[Unreleased]` into
  `[0.8.0]`, and writes the release commit.
- ⛔ **Creating and pushing the tag is maintainer-only.** No session does it.

The ten specs since `v0.7.1`:

| spec | title | lives in |
|---|---|---|
| SPEC-126 | apply and build agree on output format | PROJ-011 `specs/done/` |
| SPEC-127 | recipes carry format and quality (`version = "2"`) | PROJ-011 `specs/done/` |
| SPEC-128 | recipes can express watermark | PROJ-011 `specs/done/` |
| SPEC-129 | build cache hashes watermark asset content | PROJ-011 `specs/done/` |
| SPEC-130 | WebP discovered in directory and glob batches | **PROJ-013** `specs/` (not archived) |
| SPEC-131 | recipe step params validated at load | PROJ-011 `specs/done/` |
| SPEC-132 | `build --watch` watches recipe asset paths | PROJ-011 `specs/done/` |
| SPEC-133 | watermark refuses to damage an image | PROJ-011 `specs/done/` |
| SPEC-134 | a `--name-template` extension pins the output format | PROJ-011 `specs/done/` |
| SPEC-135 | watermark refuses a size or scale that is not a sensible number | PROJ-011 `specs/done/` |

Also on `main` since `v0.7.1`, with no spec: `545f931` and `9f1becd`, which change the build
profile only (dev/test optimization). They need no user-facing entry unless you find one that
changes a released binary. Say which.

## Step 0
`git fetch origin && git rebase origin/main`. Run
`git log --oneline v0.7.1..origin/main --no-merges` and check the table above against it. If a
code-bearing commit is missing from the table, add it to your work and report it.

## Read first
- `RELEASING.md`, steps 2 and 2a. These are the rules you are carrying out.
- `CHANGELOG.md`: `## [Unreleased]` in full, and `## [0.7.1]` and `## [0.7.0]` for the house
  style. Entries are bold-lead, behaviour-first, and name the command, the exit code and what the
  user sees.
- For each spec in the table: its Goal, Acceptance Criteria, `## Build Completion`, and the
  CHANGELOG lines its PR wrote (`git show <merge> -- CHANGELOG.md`).
- `skills/crustyimg/SKILL.md`, `recipes/README.md` and `README.md`, in full.
- The relevant DECs: DEC-102 (recipe params at load), DEC-104 (the damage rule), DEC-105 (template
  pins the format) and DEC-106 (flag checks before IO). Also the recipe-format DEC for
  `version = "2"`; find it with `./scripts/decisions-audit.sh` or grep `decisions/`.

## Do, in order

### 1 — CHANGELOG `[Unreleased]`: one entry per user-visible change
Build a table: for each spec, the entry or entries it has in `[Unreleased]`, under which heading,
or **missing**. Known gaps to start from (confirm each; don't trust them):
- **There is no `### Added` section.** SPEC-128 (a recipe `watermark` step) and SPEC-127 (a recipe
  pins `format` and `quality`, with `version = "2"`) are new capabilities. Find where, if
  anywhere, they are described today.
- SPEC-126 and SPEC-129: check each has an entry that says what the user sees.
  - For SPEC-129, say whether editing a watermark image now triggers a rebuild, and whether
    existing caches rebuild once.
  - For SPEC-126, say what output format `apply` and `build` now agree on.

Then:
- **Add what is missing**, in the house style, under the right heading (Added / Breaking /
  Changed / Fixed). Edit an existing entry only where it is wrong or misleading; do not reflow
  entries that are correct. Keep the heading order the file already uses.
- **The 0.8.0 wording ruling.** No line may promise more than "a non-finite or oversized size or
  scale is refused". Nothing may say a watermark is "never invisible" or that every invisible
  mark is refused. Marks that draw nothing on both paths (`opacity = 0`, blank text,
  `size = 0.01`) are still accepted, and they are a 0.8.1 item. Grep for `invisible`, `never`
  and `always`, and fix any overclaim.
- **Drive every new or edited entry** on a release build of `main` (fresh `CARGO_TARGET_DIR`,
  deleted at the end). For each, record the command and the exit code. Documentation has no
  green, so the run is the only check.
- ⛔ Do **not** roll `[Unreleased]` into `[0.8.0]`, change the link references, or bump any
  version. That is part 2.

### 2 — `recipes/README.md`
- **`version = "2"`, `format` and `quality`.** Document what SPEC-127 shipped, as the code does
  (`src/recipe/mod.rs:57`, `:166`, `:218-221`):
  - which keys need `version = "2"`;
  - the error a `"1"` recipe gets if it sets them;
  - how `format` interacts with `-o`, `--format` and a `--name-template` extension, per DEC-105;
  - update the `### File format` table, which says `"1"` today.
- **The `watermark` step.** Check that the step's key table lists every key `WATERMARK_PARAMS`
  accepts (`src/operation/mod.rs`), with each range: `size` > 0 and ≤ 1024, finite; `scale`
  finite and > 0; `opacity` 0–1. Also check:
  - text vs image mode;
  - that `scale` and `tile` are image-only;
  - that the "a watermark that does not fit is left off" section (SPEC-133) matches the binary;
  - the new `the scaled mark does not fit inside the W×H px image` refusal.
- Run every recipe example in the file through `apply --recipe` and `build` on the release
  binary. Record each exit code. A recipe in the guide that fails to load is a bug in the guide.

### 3 — `skills/crustyimg/SKILL.md` (RELEASING step 2a)
- **Remove only the WebP batch-input caveat** at about lines 268-272 ("In crustyimg 0.7.1 and
  earlier, WebP inputs must be named one file at a time…"), together with its shell-loop
  workaround. SPEC-130 fixed it.
- ⛔ **Keep "WebP output is lossless only"** (about line 120). `webp-lossy` is not a default
  feature, so every shipped binary still writes lossless WebP. Check it on the release binary.
- Bump "Written against crustyimg **0.7.x**" to **0.8.x**.
- Walk the `[Unreleased]` section you just finished. For every added or changed flag, changed
  exit code, new refusal or new recipe key, update the skill so an agent using it would not be
  surprised. The skill is for agents, so state what to do, not the history. At least these:
  - recipes can pin `format` and `quality` (`version = "2"`) and can watermark;
  - recipe keys are checked at load (exit 1, naming the step and the key);
  - a `--name-template` extension now pins the format, and `meta` refuses a format change;
  - `watermark` refuses a mark that does not fit or covers over 5 %: exit 2 on one input, a skip
    with a warning in a batch;
  - `--size` is at most 1024, and bad flags exit 2 before any file is read.
- Run **every** command example in the skill on the release binary, not only the ones you
  touched. Record each exit code. A skill example that fails is a bug; fix it.

### 4 — `RELEASING.md`
Step 2 holds a stale note: "This next release batches three AVIF/pixel-lane byte-changing specs on
purpose — SPEC-121…, SPEC-122…, SPEC-124…". Those shipped in 0.7.0 and 0.7.1.
- Remove the batching paragraph.
- **Keep** the lasting rule inside it: diff the CHANGELOG against the specs merged since the
  previous tag before rolling it. 0.7.0 shipped with SPEC-112 missing. Reword it as a standing
  instruction with no spec ids.

### 5 — Anything else that 0.8.0 makes false
Read `README.md` and `docs/` for sentences that the ten specs made false. Two examples:
- a statement that recipes carry only operations;
- a statement that `--name-template` never sets the format.

Fix each one in `README.md` and `docs/`. **The jysf.org project page is out of your reach.** If
you find a claim there that 0.8.0 changes, list it for the maintainer; do not try to edit it.

## Land it
- Branch `docs/release-0-8-0-notes`. Commit with `git commit -s`; **the DCO sign-off is
  mandatory**. Use one commit per numbered section, or fewer.
- Open a **draft** PR: `gh pr create --draft`.
  - Title: `docs(release): 0.8.0 release notes, recipe guide, agent skill`.
  - Body: the CHANGELOG coverage table, the list of examples run with their exit codes, and the
    Claude Code footer.
- ⛔ A draft stays a draft. Never mark it ready and never merge.
- **Never poll CI.** Run `gh pr checks <PR> --watch` in the background. When it exits, read one
  snapshot at the head SHA and check every leg.

## Guardrails
- ⛔ No source or test changes. No version bump, no `Cargo.toml` or `Cargo.lock` change, no tag
  and no release.
- No spec or DEC ids in user-facing text: the CHANGELOG, the skill, `recipes/README.md` and
  `README.md`.
- A piped command reports the pipe's exit code, so redirect output and read `$?`. zsh does not
  word-split unquoted parameters; loop with `while IFS= read -r`. macOS has no `timeout(1)`.
- Keep scratch files in a scratch directory, never in `~`. Delete your scratch target dir at the
  end.
- **Budget ~80 exchanges.**

## Measure your own cost
- Your transcript is a JSONL under `~/.claude/projects/<slug>/`. `<slug>` is your working
  directory's absolute path with every `/`, `.` and `_` replaced by `-`.
- Identify it **by content, not recency**:
  `grep -l "0.8.0 — RELEASE NOTES prompt" ~/.claude/projects/<slug>/*.jsonl`.
- ⚠ **Dedup by `.message.id`.** Claude Code writes one JSONL line per *content block*. Lines that
  share an id repeat the same `input_tokens`, `cache_creation_input_tokens` and
  `cache_read_input_tokens`, while `output_tokens` grows. Take those three once per id and the
  **MAX** `output_tokens`. Summing every line over-counts about 2×; taking the first line's output
  under-counts about 11×.
- Price each component at the model that actually ran (read `.message.model`): Opus $5/$25 per
  MTok, Sonnet $3/$15. cache_creation is ×1.25 the input rate and cache_read ×0.10.
- Report `tokens_total`, the breakdown and `estimated_usd`.

## Report back (your final message — it is pasted to the orchestrator)
First line: `0.8.0 notes: draft PR #NNN at <head SHA> — CI <green | red: legs>`. Then, short:
- the CHANGELOG coverage table (spec → entry, heading, added / edited / already there);
- the overclaim grep and what you changed;
- the `recipes/README.md` changes, and every recipe example's exit code;
- the skill changes, and every skill example's exit code (count, plus any that failed);
- the `RELEASING.md` change;
- any other false sentences you fixed, and any jysf.org claims for the maintainer;
- your cost.
