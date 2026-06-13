# Hardening Session Report — 2026-04-20

A narrative record of the Claude Code session that hardened this
template repo before it ships to other humans. Written for the next
Claude session that will review this work and plan the follow-on repo.

If you're that session: **start here**, then read `CHANGELOG.md` for
the condensed what-changed and `KNOWN_LIMITATIONS.md` for what's
explicitly left unfixed.

---

## 1. What this session was

**Trigger:** the user had a fresh template repo (delivered at commit
`df74e57 initial: template as delivered from v5`) and wanted it
polished on macOS before real use. The on-boarding doc for the session
is `CLAUDE_CODE_ONBOARDING.md` at the repo root — it defines the
hierarchy, known weak spots, and the questions Section 7 told me to
ask upfront.

**User answers to the Section 7 questions:**
1. No specific bugs hit yet — but curious what would happen if `just init` re-runs.
2. Claude-only variant.
3. macOS.
4. Bug-fix only. No new features (no `just new-project`).
5. Add tests if cheap; otherwise manual verification is fine.
6. Leave cross-variant dedup alone.

**Agreed scope:** find bugs by exercising the scripts on macOS, fix
them one commit at a time, add a cheap test harness, leave `just
new-project` and any prompt/doc rewrites to a later session.

---

## 2. Phase 1 — Triage (bugs found by exercising the scripts)

Steps taken: copy the repo into a temp dir, run `just init` picking
claude-only, then run the full happy path (`new-stage` → `new-spec`
→ `advance-cycle` × 4 → `archive-spec` → `weekly-review` → re-init).
Every surprise became a bug in the plan.

### Blocking bugs

1. **`just init` re-run silently half-initializes.** The AGENTS.md
   guard worked, but the hint said "remove AGENTS.md first"; doing
   that and re-running made `cp -r variants/…` fail (variants/ was
   already deleted on first init), but the recipe kept going — `\ ;`
   chain — and printed `✓ Done`, wrote `.variant`, left a broken
   half-repo.

2. **`update_frontmatter_scalar` strips inline comments.** First
   `advance-cycle` erased the `# frame | design | build | verify | ship`
   legend from the `cycle:` line. Flagged in the on-boarding doc,
   confirmed on reproduction.

3. **Archiving a spec twice nested it.** `find_spec` returned the
   already-archived copy, so `archive-spec SPEC-NNN` run a second
   time produced `specs/done/done/SPEC-NNN-…`. No guard anywhere.

4. **`weekly-review` emitted absolute paths.** Every file it
   discovered via `find` came out as `/tmp/.../projects/…` while
   hand-listed entries were relative. The prompt text promised
   "paths relative to repo root"; the output broke that contract.

### Medium

5. **`YYYY-MM-DD → today` substitution was too aggressive.** In
   `stage.md`, the documentation comment `# optional: YYYY-MM-DD`
   became `# optional: 2026-04-20`, which reads like a real target
   date.

6. **`just new-project` is referenced but doesn't exist.** The
   `die()` message in `_lib.sh` and the example `brief.md` in both
   variants pointed users at a phantom command.

### Noted but left alone

- `new-spec.sh` dead if/else with identical branches.
- `just init` is interactive-only (breaks in CI/piped shells).
- `next_id` assumes 3-digit padding.
- `new-spec` / `archive-spec` don't auto-update the parent stage's
  backlog — documented as intentional in the on-boarding.

---

## 3. The plan the user approved

- Phase 2: fix the four blockers, one commit each.
- Phase 3: fix the two mediums, one commit each.
- Phase 4: add `scripts/test.sh` + `just test` covering every fix.
- Phase 5: write `CHANGELOG.md` and `KNOWN_LIMITATIONS.md`.
- Two-place rule: any fix in a shared file applies to both variants.

---

## 4. Fixes shipped

### `fix(init): make re-run fail safely with accurate guidance` — `d4df426`

- Chained recipe steps with `&&` so failed `cp` aborts.
- Added a second guard: fail early if `variants/` is missing.
- Rewrote the hint: init is one-shot; to start over, restore from git
  or re-clone. The old "remove AGENTS.md first" advice was actively
  misleading.

### `fix(lib): preserve inline comments in update_frontmatter_scalar` — `cae2940`

- Awk replacement now finds any trailing `# …` in the value tail and
  reattaches it after the new value.
- Column alignment is sacrificed (two-space gap before `#`) but the
  comment content survives and output is stable across repeated calls.
- Scalar values in our front-matter are barewords, so treating the
  first `#` in the tail as the comment start is safe.

### `fix(lib): find_spec must not return archived specs` — `55b10f8`

- `find_spec` now uses `-not -path '*/done/*'`, so:
  - `archive-spec SPEC-NNN` run twice fails with "Spec not found"
    instead of nesting into `done/done/`.
  - `advance-cycle` on an archived spec also fails loudly instead of
    silently mutating the `done/` copy.
- Used `-not -path` rather than `grep -v` because `grep -v` returns 1
  on no-match, which trips `pipefail` and would make `find_spec`
  silently abort the caller under `set -e` without printing `die()`.
  I confirmed this in debugging — first attempt using grep produced
  no error output; switching to `find -not -path` fixed it.

### `fix(weekly-review): emit repo-relative paths consistently` — `9424e61`

- Strip `$REPO_ROOT/` from each find-derived path via `sed` before
  the bullet prefix is added.
- The done-specs block now splits the work: awk for the mtime sort,
  a final `sed` to strip the prefix, a final `sed` to add "- ".

### `fix(templates): use __TODAY__ placeholder so comments aren't mangled` — `904b03e`

- Real placeholders (where scaffolding needs today's date substituted)
  switched to `__TODAY__` in:
  - `variants/claude-only/projects/_templates/spec.md`
  - `variants/claude-only/projects/_templates/stage.md`
  - `variants/claude-plus-agents/projects/_templates/stage.md`
- `new-spec.sh` and `new-stage.sh` substitute `__TODAY__` now.
- Format comments like `# optional: YYYY-MM-DD` stay as YYYY-MM-DD
  (they serve as format documentation for the user).
- The plus-agents `spec.md` uses `handoff.created_at: null` (filled
  in when a HANDOFF is written), so no change there. Template files
  for `project-brief.md`, `handoff.md`, and `decisions/_template.md`
  aren't currently script-processed, so they keep human-readable
  YYYY-MM-DD.

### `fix(docs): remove dangling 'just new-project' references` — `edccded`

- `_lib.sh` `get_active_project` die message: now tells users to
  copy `projects/_templates/project-brief.md` into
  `projects/PROJ-NNN-<slug>/brief.md`.
- Example `brief.md` in both variants: same update.
- The command itself isn't added — user scoped this session to bug-
  fix only, and Claude-driven scaffolding during PROJECT BRIEF is
  the documented path.

### `test: add scripts/test.sh happy-path harness and 'just test'` — `63df653`

- End-to-end test that exercises every fix above in a `mktemp` dir.
- No new deps (no bats, no python). Pure bash + coreutils, macOS and
  Linux compatible (`mktemp -d` fallback, `uname`-conditional `stat`
  / `sed -i` handled via _lib.sh).
- Prints PASS/FAIL per assertion, exits non-zero on first failure,
  leaves scratch dir for triage.
- Exposed as `just test` — for template maintainers, since after
  `just init` runs, `variants/` is gone and this test fails at the
  first check.

### `docs: add CHANGELOG.md and KNOWN_LIMITATIONS.md` — `fc55fd1`

- CHANGELOG: one entry per fix, session-dated.
- KNOWN_LIMITATIONS: what we deliberately didn't touch and why.

---

## 5. Mid-session feedback from the bragfile user

Partway through, the user came back with three items from a
downstream Claude session using this template to build `bragfile`:

**Item 1 — `archive-spec` stage-shipped false-positive** (priority 1,
same-session fix). Archiving the last active spec printed "All specs
for STAGE-X are shipped. Consider running the Stage Ship prompt." That
fired even when the stage's Spec Backlog still listed unwritten specs
(template explicitly shows `- [ ] (not yet written) — …`). The cheap
honest fix: reword to an observation ("No active specs remain for
STAGE-X. If the stage's Spec Backlog is fully complete, run the Stage
Ship prompt…"). Can't check the backlog reliably since it's manually
maintained markdown and typically stale right after archive.

Shipped as `341af4f`.

**Item 2 — `__REPO_ID__` placeholder drift** (priority 2, same-session
fix, same bug-class as `__TODAY__`). Every template hardcoded
`id: my-app` in its `repo:` block. `.repo-context.yaml` had a
"REPLACE" comment but nothing read that file. Even after the user
replaced it there, every `just new-spec` and `just new-stage` still
stamped `my-app` into the scaffolded frontmatter.

Fix:
- Templates use `__REPO_ID__` now (9 files across both variants:
  spec.md, stage.md, project-brief.md, handoff.md, decisions/_template.md).
- New `get_repo_id` helper in `_lib.sh` parses `metadata.repo.id`
  from `.repo-context.yaml` via portable awk (no yq dependency).
- `new-spec.sh` and `new-stage.sh` call it and substitute.
- Fallback: `my-app` if the file/key is missing, so behavior never
  regresses on a pristine clone.
- `.repo-context.yaml` itself still has `id: my-app` — that's the
  user's source of truth and the REPLACE comment drives the human
  edit. Example files under `PROJ-001-example-mvp/` and DEC-001
  also keep `my-app` since they're meant to be deleted.

Shipped as `e11b872`; tests extended in `28ecdc4`; CHANGELOG
updated in `0be94c8`.

**Item 3 — "Fresh session is weaker than it looks with a single
agent"** (priority 3, deferred). This is prompt/onboarding copy, not
a script bug. Out of scope for bug-fix session. Left as an open item
for the next session to address — see Section 8 below.

---

## 6. Test harness — what `just test` covers

30 assertions, all passing at session close. From `scripts/test.sh`:

| # | Area | Assertion |
|---|---|---|
| 1-5 | init happy path | AGENTS.md, .variant, variant marker, variants/ removed |
| 6-8 | init re-run guards | fails with AGENTS.md present; fails with AGENTS.md removed + variants gone |
| 9-12 | new-stage scaffold | stage.md exists; created_at is today; format comment preserved; repo.id from .repo-context.yaml |
| 13-17 | new-spec scaffold | spec.md exists; SPEC ID, stage ref, created_at, repo.id |
| 18-20 | advance-cycle | cycle updated AND legend comment preserved, across build → verify → ship |
| 21-22 | archive happy path | file moved to done/; source gone |
| 23-24 | stage-shipped wording | no false "All specs… are shipped" claim; new observation message present |
| 25-27 | archive guards | double-archive fails; no done/done/ created; advance-cycle on archived spec fails |
| 28-29 | weekly-review paths | no absolute paths in output; archived spec present as relative path |
| 30 | status | exits clean after archive |

To run: `just test`. Needs to run from the template root (pre-init),
so run it in the template repo itself, not inside an init'd user repo.

---

## 7. Files changed

13 commits touching 21 files. See `git log --oneline df74e57..HEAD`
for the full sequence. Quick map:

- `justfile` — init recipe hardened, `test` target added.
- `scripts/_lib.sh` — `update_frontmatter_scalar` preserves comments;
  `find_spec` excludes done/; `get_repo_id` helper added.
- `scripts/archive-spec.sh` — stage-shipped message reworded.
- `scripts/new-spec.sh`, `scripts/new-stage.sh` — __TODAY__ and
  __REPO_ID__ substitutions.
- `scripts/weekly-review.sh` — relative paths.
- `scripts/test.sh` — new test harness.
- `variants/*/projects/_templates/*.md` — __TODAY__ and __REPO_ID__
  placeholders.
- `variants/*/decisions/_template.md` — __REPO_ID__ placeholder.
- `variants/*/projects/PROJ-001-example-mvp/brief.md` — removed
  phantom `just new-project` reference.
- `CHANGELOG.md`, `KNOWN_LIMITATIONS.md`, and this session report
  (originally at the repo root as `SESSION_REPORT.md`; moved to
  `docs/sessions/` in a later session) — new docs.

Initial commits in scope: `df74e57` (delivered template) and `2d9d49f`
(on-boarding doc) were not modified.

---

## 8. How to verify (for the next session)

From the repo root, with `just` installed:

```bash
# 1. Full happy-path test — should print "PASS  30 checks".
just test

# 2. Exercise init manually in a scratch dir:
tmp=$(mktemp -d); cp -R . "$tmp/repo"; rm -rf "$tmp/repo/.git"; cd "$tmp/repo"
printf "1\n" | just init
# Try every command: just info, just status, just new-stage "foo",
# just new-spec "bar" STAGE-002, just advance-cycle SPEC-002 build, etc.

# 3. Verify re-init guards work (from the same scratch):
just init                    # should fail with "Already initialized"
rm AGENTS.md && just init    # should fail with "variants/ is missing"

# 4. Grep sanity — no dangling phantom commands:
git grep 'just new-project'  # should only match CLAUDE_CODE_ONBOARDING.md
```

All of #1 is automated. Items #2–4 reproduce the actual user flow.

---

## 9. Known limitations — kept, not fixed

See `KNOWN_LIMITATIONS.md` for the durable copy. Summary of what's
left alone and why:

1. **No `just new-project`** — Claude creates projects during the
   PROJECT BRIEF step per GETTING_STARTED.md.
2. **Parent stage backlog not auto-updated on new-spec/archive-spec**
   — markdown-list formatting is judgment-laden; deliberate.
3. **`just init` is interactive-only** — fine for the intended use
   (one human, once).
4. **No cross-platform CI** — the test harness runs on whoever runs
   it. Scripts have `uname`-conditional branches for `stat` and
   `sed -i` and those get exercised.
5. **Scripts assume 3-digit zero-padded IDs** — won't matter at
   normal scale.
6. **`get_active_project` is lexical-first** — `ACTIVE_PROJECT` env
   var override exists; documented via `just info`.
7. **Templates exist in two copies** — user explicitly scoped dedup
   out for this pass. Maintainability hazard to watch.

---

## 10. Open items for the next repo / session

### Directly from user feedback

**Item 3 from the bragfile session: "Fresh session is weaker than
it looks with a single agent."** This is prompt/onboarding copy.
The claude-only variant leans heavily on the "start a new Claude
session for each cycle" discipline (see `GETTING_STARTED.md`
Step 7/8 and `AGENTS.md`). The downstream user's signal suggests
this discipline alone doesn't buy as much separation-of-concerns
as the framework assumes when only one agent is involved. Next
session may want to:
- Investigate what the downstream session actually experienced
  (was it memory leakage across sessions? Confirmation bias?
  Not actually opening new sessions consistently?).
- Consider whether the claude-only variant's promise is honest or
  whether it needs explicit hedging in GETTING_STARTED / AGENTS.
- Potentially design the multi-agent variants with this in mind.

### Noted by me during the pass, not fixed

- **Cost tracking integration** (from on-boarding §9a). The template
  reserves a natural slot at the bottom of spec front-matter for a
  `cost:` block. The YAML helpers in `status.sh` are still narrow —
  they'd need generalizing when that block arrives. This session
  didn't touch front-matter structure, so that slot is still open.

- **Reports (`just report-daily` / `just report-weekly`)** from
  on-boarding §9a — not built; the `weekly-review.sh` path-relative
  fix should carry forward cleanly to the report scripts.

- **`get_active_project` heuristic** — lexical-first with an
  undocumented-outside-`info` env var override. Works today; may
  deserve surfacing in GETTING_STARTED if multi-project usage gets
  common.

- **Two-copies-of-templates maintenance cost** — nine files touched
  in both variants during this pass just for the two placeholder
  fixes. Worth revisiting if the follow-on session adds more shared
  structure.

### Things I considered and decided not to do

- Add `just new-project` — user scoped to bug-fix only.
- Port to a different shell/lang — out of scope per on-boarding §9.
- Add bats test framework — user said manual verification was fine
  if tests didn't make sense; the bash-only harness covers the
  fixes without adding a dependency.
- Dedupe shared files across variants — user said leave it.

---

## 11. Commit log (reference)

```
0be94c8 docs(changelog): record archive-spec rewording and __REPO_ID__ fix
28ecdc4 test: cover __REPO_ID__ substitution and stage-shipped rewording
e11b872 fix(templates): __REPO_ID__ placeholder, substituted from .repo-context.yaml
341af4f fix(archive-spec): stage-shipped message reports observation, not claim
fc55fd1 docs: add CHANGELOG.md and KNOWN_LIMITATIONS.md
63df653 test: add scripts/test.sh happy-path harness and 'just test'
edccded fix(docs): remove dangling 'just new-project' references
904b03e fix(templates): use __TODAY__ placeholder so comments aren't mangled
9424e61 fix(weekly-review): emit repo-relative paths consistently
55b10f8 fix(lib): find_spec must not return archived specs
cae2940 fix(lib): preserve inline comments in update_frontmatter_scalar
d4df426 fix(init): make re-run fail safely with accurate guidance
2d9d49f docs: add hardening session onboarding          # pre-session
df74e57 initial: template as delivered from v5          # pre-session
```

End of session, 2026-04-20.
