# PROJ-011 — orchestration handoff

**Rewritten 2026-09-06.** Supersedes the 2026-08-23 version entirely — that one was written
before STAGE-049 existed as shipped work and before the cost ledger was known to be wrong.

> **This file deliberately does NOT restate repo state.** `just status`, `just backlog`,
> `just roadmap` and `just specs-by-stage` all report correctly — **trust them over any summary,
> including this one.** What follows is only what the tooling cannot show.

---

## Read first

`/AGENTS.md`, then `just status` and `just backlog`. Then this file.

**You orchestrate; you do not build.** Build and verify go to separate CLI sessions via a persisted
prompt in `specs/prompts/`, pushed to `main` **before** the branch is cut.

---

## Where this stands

**SPEC-126, SPEC-127 and SPEC-128 all shipped** (2026-09-03 → 09-06). STAGE-049 is **shipped**;
STAGE-050 is **active** with 2 shipped / 1 closed / **5 pending**.

### The one sequencing rule that matters

⛔ **Everything in PROJ-011 is byte-changing on a shipped verb, so the whole project carries ONE
lockfile migration and ships as ONE release.** The key is a function of `crate::version()`, so what
makes it one migration is landing in the same *release*, not the same PR. **Merging is fine and
expected; tagging is not.** Version is still **0.7.1** and must stay there until STAGE-050 closes.

### What a recipe can express now, which is the whole point of the stage

```toml
version = "2"          # required by format/quality (SPEC-127)
format  = "jpeg"

[[step]]
op = "watermark"       # SPEC-128
image = "assets/logo.png"
gravity = "southeast"
opacity = 0.6
```

---

## ⚡ The one thing I would spec next

**`build` serves a stale watermark from cache.** Reproduced at SPEC-128's verify with a verified
control: change the overlay's bytes, and `build` reports *"1 cached, 0 rebuilt"* and emits
byte-identical pre-edit output. **`apply` is unaffected.** Filed on STAGE-050, visible to
`just backlog`.

It is a **silent wrong-output path that arrived with the watermark capability** — the cache key does
not cover a step's resolved assets. Everything else pending in STAGE-050 is a feature; this is a
correctness bug in a shipped verb.

---

## Open, waiting on the maintainer

Batching these in one sitting unblocks more than anything an orchestrator can do alone.

- **The watermark coverage threshold.** ⚡ **No longer an open question — it is now a choice between
  two numbers.** Measured 2026-09-03 with a geometric measure (decode both to RGBA8, count differing
  pixels; SSIMULACRA2 is the WRONG instrument — it is non-monotonic at small canvases, scoring 24 px
  as *less* damaged than 64 px). The backlog's **~25 % lands at a 32 px canvas** and would pass
  48 px (17 %), 64 px (13 %) and 128 px (7 %), all visibly ruined. **The visual read puts it nearer
  2–5 %.** Full curve on STAGE-050. A **clipping** predicate may be the better primary rule — it
  needs no threshold at all.
- **The `--name-template` pin ruling** — warn / honour-the-template / document-and-keep. Same family
  as the `-o`-extension pin ruling, which moved to PROJ-013 STAGE-037. Worth ruling together.
- **STAGE-041's real status.** The repo has understated it since 2026-08-16. **Do not re-plan it
  against what the repo says.**
- **`mp4-atom` DEC** — blocks PROJ-012 from being specced at all.
- **The two GitHub Action releases are still DRAFTS.** `v1.0.1` on `jysf/setup-crustyimg` and
  `jysf/crustyimg-action`, self-tests green, finished since **2026-08-12**. Publishing is a click
  and it is the smallest-effort/highest-visibility item on the board.
- **The jysf.org project page** — a fourth public surface, in no release checklist.

---

## Traps this wave paid for

1. **⚡ MEASURE THE SCENARIO, NOT THE CONTROL.** Three times in three specs I drove a table and the
   row carrying the argument did not use the feature the argument was about. SPEC-127's Call 1 was
   settled on a `version = "2"` recipe **with no `format` key** — the one case where v2 buys nothing
   — so AC-5's second half was unachievable and the rationale was wrong. **When a claim rests on a
   driven table, the row carrying it must use the feature.**
2. **⚡ AN AC NAMING TWO THINGS NEEDS A TEST PER THING.** SPEC-128's AC-5 said "overlay/**font**" and
   only overlay was tested — half the asset mechanism shipped unguarded. Count the nouns in each AC
   against the tests listed; it is free at design time.
3. **⚡ THE COST LEDGER WAS ~2× WRONG AND IS NOW FIXED — do not regress it.** Claude Code writes one
   JSONL line **per content block**. Lines sharing a `.message.id` repeat identical
   `input`/`cache_creation`/`cache_read` while `output_tokens` **grows**. Correct method: dedup by
   id, take those three from the group, take **MAX** output. Summing every line over-counts ~2×;
   taking the first line's output under-counts ~11×. **Both mistakes are on record here.** 32 records
   were corrected in place; 10 remain flagged because no transcript reproduces them.
   📌 **Transcripts live in a project dir PER WORKTREE** (`~/.claude/projects/<slug>/`), and a
   subagent's is under `<session-id>/subagents/`. Searching only the main dir wrongly concluded 22
   records were unrecoverable; 12 were in sibling dirs.
4. **VERIFY BEFORE YOU COMMIT — read back what you wrote and diff it against what you intended.** A
   regex using a lazy `(?:.*\n)*?` spanned across YAML block boundaries and scattered 20 corrections
   onto the wrong entries. It reached `main` because I checked that the edit *succeeded* and not that
   it was *right*. Reverted and redone with a block parser. This one rule has since caught a phantom
   backlog bullet, an invented DEC filename, and an invented `tokens_breakdown`.
5. **`just check` is weaker than CI in a way CLAUDE.md does not list:** it never runs the **wasm
   bundle-size gate**. SPEC-128's local matrix was clean while the PR was red on a real 10.6 %
   regression. Filed on STAGE-053. DCO sign-off is the other CI-only catch — **4th recurrence**.
6. **A moved size gate is a purchase and belongs in DEC-066's ledger** — the record of what was cut,
   paid for, and refused. It had *declined* to drop `ssimulacra2` for 23,540 B; SPEC-128 bought 5×
   that and would have left no row. DEC-066's own `affected_scope` omitted the gate file, so the
   audit could not have flagged it. Both fixed.
7. **`scripts/decisions-audit.sh` silently drops an INLINE-ARRAY `affected_scope`.** `DEC-015` is
   invisible to `--changed`, forever, and DEC-015 governs `docs/api-contract.md`. Root cause is
   `decisions/_template.md:33`, whose example uses the unreadable form. **Always write the
   block-list form.** Filed, PROJ-013 STAGE-047.
8. **PROJ-013 is invisible to `just backlog`, including `--all`** — 54 items, because the project is
   still `proposed`. Anything filed there is git-safe and framework-invisible.

## What is working, and worth keeping

- **Read the code before writing the spec.** SPEC-128's backlog title was wrong — `Watermark` was
  already an `Operation`, deliberately unregistered. A design cycle that only read the item would
  have specced the wrong thing.
- **Ask what the user actually sees.** "What would this look like in a TOML file?" found a defect
  two design reviews had missed (text mode wrote the *text* under the `image` key).
- **Verify earns its cost every single time — five waves running.** It has now refuted an
  orchestrator hypothesis, established the repo's cost method, caught an unachievable AC, a
  non-discriminating control, and a half-tested mechanism. **SPEC-128's verify BUILT the alternative
  it was asked about** rather than arguing for it, which is what turned a judgement call into a
  decision.
- **Reserve DEC ids in the prompt.** `next_id` scans only the working tree. **Highest is DEC-100.**
- **Size the VERIFICATION, not just the code.** SPEC-127 was marked M off its implementation while
  its AC matrix was L-shaped; the builder said so, and was right.
- **Inline the cost-measurement rule in the dispatch prompt** — do not reference `cost-snippet.md`.
  SPEC-127's build had to reconstruct its cost post-hoc because the prompt only pointed at it.
