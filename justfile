# Spec-driven repo template — command runner
#
# This justfile works BOTH before and after `just init`:
# - Before init: only `init` and `list-variants` are expected to work.
# - After init: all the daily commands work (status, new-spec, etc.)
#
# Run `just --list` to see everything.

# Show all commands
default:
    @just --list

# ----------------------------------------------------------------------------
# ONE-TIME SETUP
# ----------------------------------------------------------------------------

# Initialize the repo: pick a variant and scaffold files to the root.
init:
    @echo "Spec-driven repo template — init"
    @echo ""
    @if [ -f AGENTS.md ]; then \
        echo "⚠  Already initialized (AGENTS.md exists at repo root)."; \
        echo "   Init is one-shot: it consumes variants/ when it runs."; \
        echo "   To start over, restore the repo from git or re-clone."; \
        exit 1; \
    fi
    @if [ ! -d variants ]; then \
        echo "⚠  variants/ directory is missing."; \
        echo "   This repo was already initialized (or the template was"; \
        echo "   modified). Restore from git or re-clone to re-init."; \
        exit 1; \
    fi
    @echo "Pick a variant:"
    @echo "  1) claude-only         (Claude plays every role; simpler)"
    @echo "  2) claude-plus-agents  (Claude architects, separate agent implements)"
    @echo ""
    @printf "Enter 1 or 2: "
    @read variant_choice && \
    if [ "$variant_choice" = "1" ]; then \
        VARIANT="claude-only"; \
    elif [ "$variant_choice" = "2" ]; then \
        VARIANT="claude-plus-agents"; \
    else \
        echo "Invalid choice: $variant_choice"; exit 1; \
    fi && \
    echo "" && \
    echo "Scaffolding $VARIANT to repo root..." && \
    cp -r "variants/$VARIANT/." . && \
    rm -rf variants/ && \
    echo "$VARIANT" > .variant && \
    echo "" && \
    echo "✓ Done. Your variant: $VARIANT" && \
    echo "" && \
    echo "Next steps:" && \
    echo "  1. Open GETTING_STARTED.md" && \
    echo "  2. Work through the PROJECT FRAME prompt in FIRST_SESSION_PROMPTS.md" && \
    echo "  3. Commit the scaffolded repo:" && \
    echo "       git add . && git commit -m 'chore: initialize spec-driven scaffold'"

# List the available variants (useful before init)
list-variants:
    @echo "Available variants:"
    @echo "  claude-only         — Claude plays every role; no handoff documents"
    @echo "  claude-plus-agents  — Claude architects, separate agent implements; adds /handoffs/"
    @echo ""
    @echo "Run 'just init' to pick one."

# ----------------------------------------------------------------------------
# DAILY COMMANDS (work after `just init`)
# ----------------------------------------------------------------------------

# Print repo state: active project, stage, specs by cycle, stale items
status:
    @./scripts/status.sh

# Scaffold a new spec. Usage: just new-spec "short title" STAGE-NNN [PROJ-NNN]
new-spec TITLE STAGE_ID PROJECT_ID="":
    @./scripts/new-spec.sh "{{TITLE}}" "{{STAGE_ID}}" "{{PROJECT_ID}}"

# Scaffold a new stage. Usage: just new-stage "short title" [PROJ-NNN]
new-stage TITLE PROJECT_ID="":
    @./scripts/new-stage.sh "{{TITLE}}" "{{PROJECT_ID}}"

# Advance a spec's cycle. Usage: just advance-cycle SPEC-NNN verify
advance-cycle SPEC_ID NEW_CYCLE:
    @./scripts/advance-cycle.sh "{{SPEC_ID}}" "{{NEW_CYCLE}}"

# Archive a shipped spec: move to done/ and update stage backlog.
# Usage: just archive-spec SPEC-NNN
archive-spec SPEC_ID:
    @./scripts/archive-spec.sh "{{SPEC_ID}}"

# Print the Weekly Review prompt with recent activity pre-loaded
weekly-review:
    @./scripts/weekly-review.sh

# Generate today's daily report under reports/daily/YYYY-MM-DD.md
report-daily:
    @./scripts/report_daily.sh

# Generate this week's weekly report under reports/weekly/YYYY-WNN.md.
# Pass a YYYY-MM-DD to report on the ISO week containing that date.
report-weekly DATE="":
    @./scripts/report_weekly.sh "{{DATE}}"

# Capture today's `just status` output to reports/daily/YYYY-MM-DD-status.md.
# Lighter than report-daily — a snapshot of current state with no curation.
daily-status-report:
    @mkdir -p reports/daily
    @D="$(date +%Y-%m-%d)"; \
        { echo "# Daily status - $D"; echo; ./scripts/status.sh; } > "reports/daily/$D-status.md"; \
        echo "✓ Wrote reports/daily/$D-status.md"

# Spec-grained "what's next?" view: in-flight specs in the active
# stage, un-promoted bullets in the active stage's backlog, and
# counts in upcoming stages. Pass --all to widen scope.
backlog *FLAGS:
    @./scripts/backlog.sh {{FLAGS}}

# Stage-grained "where is this project going" view: one row per
# stage in the active project with status, date range, and (for
# active/upcoming) spec counts.
roadmap:
    @./scripts/roadmap.sh

# Flat ledger of every spec grouped by stage, with ship date and
# complexity. Defaults to ALL projects (history); pass `--active` for
# the current project or a `PROJ-NNN` id for a specific one.
specs-by-stage *FLAGS:
    @./scripts/specs-by-stage.sh {{FLAGS}}

# Audit decisions: structural lint + scope-conflict warnings (zero
# deps; a native take on LineSpec-style provenance auditing). Lints
# front-matter and supersession links across all DEC-* files. Pass
# `--changed [BASE]` to flag which decisions govern your pending changes.
decisions-audit *FLAGS:
    @./scripts/decisions-audit.sh {{FLAGS}}

# ----------------------------------------------------------------------------
# HELPERS
# ----------------------------------------------------------------------------

# Print the active project and variant
info:
    @./scripts/info.sh

# Run the template's end-to-end happy-path tests (uses a temp dir).
# Intended for template maintainers, not end users. Works from the
# pre-init template root only — after `just init` runs, variants/ is
# gone and this test would fail at the first check.
test:
    @./scripts/test.sh
