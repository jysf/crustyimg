#!/usr/bin/env bash
# scripts/test.sh — end-to-end happy-path test for the template.
#
# Copies the repo into a temp dir, runs `just init` + the full cycle
# (new-stage → new-spec → advance-cycle × 4 → archive-spec), and
# asserts the invariants that previous bugs tripped over:
#
#   - init is one-shot and refuses to re-run
#   - advance-cycle preserves the cycle legend comment
#   - archive-spec refuses to archive an already-archived spec
#   - weekly-review emits only repo-relative paths
#
# No external test framework needed. Prints PASS / FAIL per check.
# Exits 0 if everything passes, 1 on the first failure.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEMPLATE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# --- Colors (off if not a TTY) ---
if [ -t 1 ] && command -v tput >/dev/null 2>&1; then
    GREEN=$(tput setaf 2 2>/dev/null || printf '')
    RED=$(tput setaf 1 2>/dev/null || printf '')
    DIM=$(tput dim 2>/dev/null || printf '')
    RESET=$(tput sgr0 2>/dev/null || printf '')
else
    GREEN=''; RED=''; DIM=''; RESET=''
fi

pass_count=0
fail_count=0

pass() {
    pass_count=$((pass_count + 1))
    echo "${GREEN}✓${RESET} $*"
}

fail() {
    fail_count=$((fail_count + 1))
    echo "${RED}✗${RESET} $*" >&2
    # Bail on first failure — later checks usually depend on earlier state.
    echo "" >&2
    echo "${RED}FAILED${RESET}  (${pass_count} passed before this one)" >&2
    echo "Scratch dir left at: ${SCRATCH}" >&2
    exit 1
}

assert_eq() {
    local actual="$1" expected="$2" msg="$3"
    if [ "$actual" = "$expected" ]; then
        pass "$msg"
    else
        fail "$msg (expected: '$expected', got: '$actual')"
    fi
}

assert_file() {
    if [ -f "$1" ]; then pass "file exists: $1"; else fail "missing file: $1"; fi
}

assert_no_file() {
    if [ ! -e "$1" ]; then pass "absent: $1"; else fail "unexpected path: $1"; fi
}

assert_contains() {
    local file="$1" pattern="$2" msg="$3"
    if grep -qE "$pattern" "$file"; then
        pass "$msg"
    else
        fail "$msg (pattern '$pattern' not found in $file)"
    fi
}

assert_cmd_fails() {
    local msg="$1"; shift
    if "$@" >/dev/null 2>&1; then
        fail "$msg (expected non-zero exit, got 0)"
    else
        pass "$msg"
    fi
}

# --- Set up scratch dir ---
SCRATCH=$(mktemp -d 2>/dev/null || mktemp -d -t 'template-hardening-test')
# Copy template into scratch/repo, then delete .git so the scratch acts
# like a fresh `Use this template` clone.
cp -R "$TEMPLATE_ROOT" "$SCRATCH/repo"
rm -rf "$SCRATCH/repo/.git"

cd "$SCRATCH/repo"
echo "${DIM}scratch: $SCRATCH${RESET}"
echo ""

# ============================================================
# 1) init: happy path
# ============================================================
printf "1\n" | just init >/dev/null 2>&1 \
    || fail "just init (claude-only) exited non-zero"
assert_file "AGENTS.md"
assert_file ".variant"
assert_eq "$(cat .variant)" "claude-only" "variant marker is claude-only"
assert_no_file "variants"
pass "init: scaffolded claude-only successfully"

# ============================================================
# 2) init: re-run guard
# ============================================================
assert_cmd_fails "re-running init (AGENTS.md present) fails" just init
rm AGENTS.md
assert_cmd_fails "init with variants/ gone also fails" bash -c 'printf "1\n" | just init'
# Restore AGENTS.md by rerunning init cleanly from a fresh scratch for the next checks.
# Simpler: copy AGENTS.md back from the TEMPLATE_ROOT's variant.
cp "$TEMPLATE_ROOT/variants/claude-only/AGENTS.md" ./AGENTS.md
pass "init: re-run guards work in both states"

# ============================================================
# 3) new-stage + new-spec scaffold correctly
# ============================================================
# Simulate the user replacing the REPLACE'd repo id in .repo-context.yaml
# so we can verify the scaffold picks it up.
sed_inplace_portable() {
    if [ "$(uname)" = "Darwin" ]; then sed -i '' "$@"; else sed -i "$@"; fi
}
sed_inplace_portable 's|id: my-app|id: bragfile-test|' .repo-context.yaml

just new-stage "Test Stage" >/dev/null
STAGE_FILE="projects/PROJ-001-example-mvp/stages/STAGE-002-test-stage.md"
assert_file "$STAGE_FILE"
# created_at should be today (not the __TODAY__ placeholder).
today=$(date +%Y-%m-%d)
assert_contains "$STAGE_FILE" "^created_at: ${today}\$" "stage.md created_at filled with today"
# target_complete comment should still say YYYY-MM-DD (not substituted).
assert_contains "$STAGE_FILE" "# optional: YYYY-MM-DD" "stage.md comment placeholder untouched"
# repo.id should come from .repo-context.yaml, not the hardcoded default.
assert_contains "$STAGE_FILE" "^  id: bragfile-test\$" "stage.md repo.id picks up from .repo-context.yaml"

just new-spec "Test Spec" STAGE-002 >/dev/null
SPEC_FILE="projects/PROJ-001-example-mvp/specs/SPEC-002-test-spec.md"
assert_file "$SPEC_FILE"
assert_contains "$SPEC_FILE" "id: SPEC-002" "spec ID set"
assert_contains "$SPEC_FILE" "stage: STAGE-002" "spec parent stage set"
assert_contains "$SPEC_FILE" "^  created_at: ${today}\$" "spec created_at filled"
assert_contains "$SPEC_FILE" "^  id: bragfile-test\$" "spec.md repo.id picks up from .repo-context.yaml"

# ============================================================
# 4) advance-cycle preserves the cycle legend comment
# ============================================================
just advance-cycle SPEC-002 build >/dev/null
assert_contains "$SPEC_FILE" "^  cycle: build.*# frame \| design \| build \| verify \| ship" \
    "advance-cycle build: cycle updated AND comment preserved"

just advance-cycle SPEC-002 verify >/dev/null
assert_contains "$SPEC_FILE" "^  cycle: verify.*# frame \| design" \
    "advance-cycle verify: cycle updated AND comment still present"

just advance-cycle SPEC-002 ship >/dev/null
assert_contains "$SPEC_FILE" "^  cycle: ship.*# frame \| design" \
    "advance-cycle ship: cycle updated AND comment still present"

# ============================================================
# 5) archive-spec: happy path + double-archive refusal
# ============================================================
archive_out=$(just archive-spec SPEC-002 2>&1)
ARCHIVED="projects/PROJ-001-example-mvp/specs/done/SPEC-002-test-spec.md"
assert_file "$ARCHIVED"
assert_no_file "$SPEC_FILE"
# The stage-shipped message must be an observation, not a completion
# claim — the stage's backlog may still list unwritten specs.
if printf '%s\n' "$archive_out" | grep -qE "All specs for .* are shipped\."; then
    fail "archive-spec prints false-positive 'All specs … are shipped' claim"
else
    pass "archive-spec does not claim stage completion"
fi
if printf '%s\n' "$archive_out" | grep -qE "No active specs remain for STAGE-002"; then
    pass "archive-spec reports observation (no active specs remain)"
else
    fail "archive-spec missing expected 'No active specs remain' message"
fi

# Second archive must fail and must NOT create done/done/...
assert_cmd_fails "double-archive of SPEC-002 fails" just archive-spec SPEC-002
assert_no_file "projects/PROJ-001-example-mvp/specs/done/done"

# advance-cycle on an archived spec must also fail.
assert_cmd_fails "advance-cycle on archived spec fails" just advance-cycle SPEC-002 build

# ============================================================
# 6) weekly-review emits only repo-relative paths
# ============================================================
review_out=$(just weekly-review 2>&1)
# The script's output should contain the scratch dir nowhere in path lines.
# It's OK for the scratch name to appear in shell echoes (it doesn't), but
# any `- /foo/...` bullet is a path bullet that must be relative.
if printf '%s\n' "$review_out" | grep -E "^- ${SCRATCH}" >/dev/null; then
    fail "weekly-review still prints absolute paths"
else
    pass "weekly-review: all bullet paths are repo-relative"
fi
# Sanity-check that it found the archived spec (relative).
if printf '%s\n' "$review_out" | grep -qE "^- projects/PROJ-001-example-mvp/specs/done/SPEC-002-test-spec\.md"; then
    pass "weekly-review: includes archived spec as relative path"
else
    fail "weekly-review: archived spec missing from output"
fi

# ============================================================
# 7) status runs cleanly post-archive
# ============================================================
just status >/dev/null 2>&1 || fail "just status exited non-zero after archive"
pass "status: clean exit after archive"

# ============================================================
# 8) v5.2 value/cost blocks exist in scaffolded specs
# ============================================================
# Scaffold a fresh spec (SPEC-002 is archived; create a fresh one in
# the stage we still have) to verify the v5.2 shape lands correctly.
just new-spec "Second Test Spec" STAGE-002 >/dev/null
SPEC_V52="projects/PROJ-001-example-mvp/specs/SPEC-003-second-test-spec.md"
assert_file "$SPEC_V52"
assert_contains "$SPEC_V52" "^value_link: null\$" "spec scaffold has value_link: null"
assert_contains "$SPEC_V52" "^cost:\$" "spec scaffold has cost: block header"
assert_contains "$SPEC_V52" "^  sessions: \[\]" "cost.sessions is empty list by default"
assert_contains "$SPEC_V52" "^    tokens_total: 0" "cost.totals.tokens_total is 0"
assert_contains "$SPEC_V52" "^    session_count: 0" "cost.totals.session_count is 0"

# ============================================================
# 9) v5.2 value blocks in brief and stage templates are exposed in
#    the scaffolded copies (example brief still ships pre-v5.2)
# ============================================================
# The template's project-brief and stage markdowns should carry the
# v5.2 blocks — confirm by scaffolding a new stage and inspecting.
just new-stage "V52 Test Stage" >/dev/null
STAGE_V52_PATH=$(ls projects/PROJ-001-example-mvp/stages/STAGE-00*-v52-test-stage.md 2>/dev/null | head -n1 || true)
if [ -n "$STAGE_V52_PATH" ]; then
    assert_file "$STAGE_V52_PATH"
    assert_contains "$STAGE_V52_PATH" "^value_contribution:\$" "new stage has value_contribution: block"
    assert_contains "$STAGE_V52_PATH" "^  advances: null" "value_contribution.advances starts null"
    assert_contains "$STAGE_V52_PATH" "^  delivers: \[\]" "value_contribution.delivers starts []"
else
    fail "new-stage did not produce the expected scaffold file"
fi

# ============================================================
# 10) AGENTS.md (post-init, claude-only) has Business Value and
#     Cost Tracking sections
# ============================================================
assert_contains "AGENTS.md" "^## 3\\. Business Value" "AGENTS.md has Business Value section"
assert_contains "AGENTS.md" "^## 4\\. Cost Tracking Discipline" "AGENTS.md has Cost Tracking section"

# ============================================================
# 11) just report-daily writes a file and prints output
# ============================================================
daily_out=$(just report-daily 2>&1)
daily_file="reports/daily/$(date +%Y-%m-%d).md"
assert_file "$daily_file"
# Output should start with the header
if printf '%s\n' "$daily_out" | grep -q "^# Daily report — "; then
    pass "report-daily prints header to stdout"
else
    fail "report-daily did not print expected header"
fi
# Sections present in the written file
assert_contains "$daily_file" "^## Snapshot\$" "daily report has Snapshot section"
assert_contains "$daily_file" "^## Value\$" "daily report has Value section"
assert_contains "$daily_file" "^## Cost activity\$" "daily report has Cost activity section"
assert_contains "$daily_file" "^## Flags\$" "daily report has Flags section"
# Graceful fallback on pre-v5.2 example brief (no value.thesis)
assert_contains "$daily_file" "Project thesis:.*not set" \
    "daily report handles missing project thesis gracefully"

# Re-run overwrites (not append)
lines_before=$(wc -l < "$daily_file" | tr -d ' ')
just report-daily >/dev/null 2>&1
lines_after=$(wc -l < "$daily_file" | tr -d ' ')
assert_eq "$lines_after" "$lines_before" "report-daily re-run overwrites, doesn't grow the file"

# ============================================================
# 12) just report-weekly writes a file and degrades gracefully
# ============================================================
just report-weekly >/dev/null 2>&1
# Determine the expected ISO week filename in the same way the
# script does, so macOS/Linux branches agree with the test.
if [ "$(uname)" = "Darwin" ]; then
    iso_week=$(date -j -f "%Y-%m-%d" "$(date +%Y-%m-%d)" +"%G-W%V")
else
    iso_week=$(date -d "$(date +%Y-%m-%d)" +"%G-W%V")
fi
weekly_file="reports/weekly/${iso_week}.md"
assert_file "$weekly_file"
assert_contains "$weekly_file" "^## Summary\$" "weekly report has Summary section"
assert_contains "$weekly_file" "^## Value advancement\$" "weekly report has Value advancement section"
assert_contains "$weekly_file" "^## Shipped this week\$" "weekly report has Shipped this week section"
assert_contains "$weekly_file" "^## Cost breakdown\$" "weekly report has Cost breakdown section"
# No specs shipped this week in the scratch flow except SPEC-002
# (just-archived today). So the table shouldn't be empty.
if grep -q "SPEC-002" "$weekly_file"; then
    pass "weekly report includes freshly-archived SPEC-002"
else
    fail "weekly report missing freshly-archived SPEC-002"
fi

# ============================================================
# 13) Reports tolerate a spec without cost/value_link blocks
# ============================================================
# The example spec SPEC-001-example-project-logger ships in the
# template pre-v5.2, so it has no cost or value_link. Both reports
# must still run without error — they did above. Add an explicit
# assertion that the daily report surfaces "no cost data yet" when
# the active spec has no cost.sessions entries at all.
if grep -q "no cost data yet" "$daily_file" || grep -q "Specs with no cost data" "$daily_file"; then
    pass "daily report flags pre-v5.2 specs as missing cost data"
else
    fail "daily report did not flag specs without cost data"
fi

# ============================================================
# 14) v5.3 instruction-timeline convention
# ============================================================
# SPEC-003 was scaffolded in §8; new-spec should have created a
# timeline file alongside it, substituting the SPEC ID into the
# template's SPEC-XXX placeholder.
TIMELINE_V53="projects/PROJ-001-example-mvp/specs/SPEC-003-second-test-spec-timeline.md"
assert_file "$TIMELINE_V53"
assert_contains "$TIMELINE_V53" "^# SPEC-003 timeline\$" "timeline header names the spec"
assert_contains "$TIMELINE_V53" '\[ \].*not started' "timeline legend documents [ ] not started"
assert_contains "$TIMELINE_V53" '\[~\].*in progress' "timeline legend documents [~] in progress"
assert_contains "$TIMELINE_V53" '\[x\].*complete' "timeline legend documents [x] complete"
assert_contains "$TIMELINE_V53" '\[\?\].*blocked' "timeline legend documents [?] blocked"

# prompts/ directory should exist as a peer to the spec files so
# the architect's first cycle-prompt write lands in a ready place.
if [ -d "projects/PROJ-001-example-mvp/specs/prompts" ]; then
    pass "prompts/ directory created alongside specs"
else
    fail "prompts/ directory missing after new-spec"
fi

# AGENTS.md (post-init, claude-only) has the Instruction Timeline
# section and documents all four markers. If the legend drifts the
# convention erodes, so each marker gets its own assertion.
assert_contains "AGENTS.md" "^## 9\\. Instruction Timeline\$" \
    "AGENTS.md has Instruction Timeline section"
assert_contains "AGENTS.md" '`\[ \]` not started' \
    "AGENTS.md documents [ ] not-started marker"
assert_contains "AGENTS.md" '`\[~\]` in progress' \
    "AGENTS.md documents [~] in-progress marker"
assert_contains "AGENTS.md" '`\[x\]` complete' \
    "AGENTS.md documents [x] complete marker"
assert_contains "AGENTS.md" '`\[\?\]` blocked' \
    "AGENTS.md documents [?] blocked marker"

# archive-spec should co-move the timeline file into done/, keeping
# the spec and its cycle history paired. SPEC-002 was archived in §5;
# verify its timeline (if one ever existed — SPEC-002 was scaffolded
# by new-spec in this test, so it got one) is also in done/.
ARCHIVED_TIMELINE="projects/PROJ-001-example-mvp/specs/done/SPEC-002-test-spec-timeline.md"
assert_file "$ARCHIVED_TIMELINE"
assert_no_file "projects/PROJ-001-example-mvp/specs/SPEC-002-test-spec-timeline.md"

# ============================================================
# 15) just daily-status-report writes reports/daily/<date>-status.md
# ============================================================
just daily-status-report >/dev/null 2>&1
status_snap="reports/daily/$(date +%Y-%m-%d)-status.md"
assert_file "$status_snap"
assert_contains "$status_snap" "^# Daily status - $(date +%Y-%m-%d)\$" \
    "daily-status-report header names the date"

# ============================================================
# 16) just backlog (spec-grained "what's next" view)
# ============================================================
backlog_out=$(just backlog 2>&1)
if printf '%s\n' "$backlog_out" | grep -q "^Backlog for "; then
    pass "backlog prints header for active project"
else
    fail "backlog header missing"
fi
# Active stage's un-promoted bullets in the example project should
# include the four "(not yet written)" entries from STAGE-001 (the
# example project ships these). Surface at least one to confirm
# parsing works.
if printf '%s\n' "$backlog_out" | grep -qE "Typed error classes|Env-var loader|Health check"; then
    pass "backlog surfaces un-promoted bullets from active stage"
else
    fail "backlog did not surface un-promoted bullets"
fi
# In-flight section should mention SPEC-001 (the example spec is in
# design cycle in the example project).
if printf '%s\n' "$backlog_out" | grep -q "SPEC-001"; then
    pass "backlog lists in-flight specs"
else
    fail "backlog missing in-flight spec"
fi

# --all flag should not error out and should still produce output.
just backlog --all >/dev/null 2>&1 || fail "backlog --all exited non-zero"
pass "backlog --all exits cleanly"

# ============================================================
# 17) just roadmap (stage-grained view)
# ============================================================
roadmap_out=$(just roadmap 2>&1)
if printf '%s\n' "$roadmap_out" | grep -q "^Roadmap for "; then
    pass "roadmap prints header for active project"
else
    fail "roadmap header missing"
fi
# STAGE-001 should appear with its date range. The example ships
# with status: active, so we expect the active bucket.
if printf '%s\n' "$roadmap_out" | grep -qE "STAGE-001-foundational-infra.*active"; then
    pass "roadmap renders the active stage with bucket"
else
    fail "roadmap did not render active stage correctly"
fi
# Spec counts should appear for the active row (1 in flight, 4 backlog
# from the example project).
if printf '%s\n' "$roadmap_out" | grep -qE "1 in flight, 4 backlog"; then
    pass "roadmap shows correct spec counts for active stage"
else
    fail "roadmap spec counts wrong or missing"
fi

# ============================================================
# security: titles with sed metachars are escaped, not injected
# ============================================================
INJECT_MARKER="/tmp/spec-driven-template-inject-$$"
rm -f "$INJECT_MARKER"
NASTY_TITLE="Pwn|e touch ${INJECT_MARKER} & a\\b"
if just new-stage "$NASTY_TITLE" >/dev/null 2>&1; then
    pass "new-stage accepts a title containing sed metacharacters"
else
    fail "new-stage failed on a title with sed metacharacters (should escape, not break)"
fi
if [ -e "$INJECT_MARKER" ]; then
    fail "SECURITY: sed injection — marker file was created from a hostile title"
    rm -f "$INJECT_MARKER"
else
    pass "no command injection from a hostile title"
fi
NASTY_STAGE=$(ls projects/PROJ-001-example-mvp/stages/STAGE-*pwn* 2>/dev/null | head -n1 || true)
if [ -n "$NASTY_STAGE" ] && grep -qF 'Pwn|e touch' "$NASTY_STAGE"; then
    pass "hostile title is rendered verbatim in the generated file"
else
    fail "hostile title was not rendered verbatim"
fi

# ============================================================
# specs-by-stage: flat ledger across scopes
# ============================================================
sbs_all=$(just specs-by-stage 2>&1)
if printf '%s\n' "$sbs_all" | grep -qE "Specs by stage — all projects"; then
    pass "specs-by-stage defaults to all projects"
else
    fail "specs-by-stage default header wrong: $sbs_all"
fi
if printf '%s\n' "$sbs_all" | grep -qE "^Totals: [0-9]+ shipped"; then
    pass "specs-by-stage prints a totals line"
else
    fail "specs-by-stage totals line missing: $sbs_all"
fi
if printf '%s\n' "$sbs_all" | grep -qE "STAGE-001-foundational-infra"; then
    pass "specs-by-stage groups specs under their stage"
else
    fail "specs-by-stage did not render STAGE-001: $sbs_all"
fi
sbs_active=$(just specs-by-stage --active 2>&1)
if printf '%s\n' "$sbs_active" | grep -qE "active project \(PROJ-001"; then
    pass "specs-by-stage --active scopes to the active project"
else
    fail "specs-by-stage --active header wrong: $sbs_active"
fi
assert_cmd_fails "specs-by-stage rejects an unknown flag" \
    just specs-by-stage --bogus

# ============================================================
# decisions-audit: lint + scope auditing
# ============================================================
# Clean state: the example DEC-001 is well-formed and has an
# affected_scope, so a plain audit should pass and report "clean".
audit_out=$(just decisions-audit 2>&1)
if printf '%s\n' "$audit_out" | grep -qE "clean: structure valid"; then
    pass "decisions-audit reports clean on the example decision"
else
    fail "decisions-audit did not report clean: $audit_out"
fi

# A structurally broken decision (missing created_at + insight.type and
# a dangling supersedes) must make the audit exit non-zero.
cat > decisions/DEC-666-broken.md <<'BROKEN'
---
insight:
  id: DEC-666
supersedes: DEC-999
superseded_by: null
---

# DEC-666: Intentionally broken
BROKEN
assert_cmd_fails "decisions-audit exits non-zero on a broken decision" \
    just decisions-audit
rm -f decisions/DEC-666-broken.md

# --changed maps pending edits to the decisions that govern them.
# Needs a git repo (scratch had its .git removed at setup), so init one.
git init -q >/dev/null 2>&1
git add -A >/dev/null 2>&1
git commit -qm "scratch baseline" >/dev/null 2>&1
mkdir -p src/lib
echo "// touched" >> src/lib/log.ts
changed_out=$(just decisions-audit --changed 2>&1)
if printf '%s\n' "$changed_out" | grep -qE "DEC-001"; then
    pass "decisions-audit --changed flags DEC-001 for an edit to src/lib/log.ts"
else
    fail "decisions-audit --changed missed DEC-001: $changed_out"
fi

# ============================================================
# Done
# ============================================================
echo ""
echo "${GREEN}PASS${RESET}  ${pass_count} checks"
echo "${DIM}(scratch dir removed: ${SCRATCH})${RESET}"
rm -rf "$SCRATCH"
exit 0
