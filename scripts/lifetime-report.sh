#!/usr/bin/env bash
# scripts/lifetime-report.sh — whole-repo lifetime history, two modes.
#
# Unlike `weekly-review` (active project, last 7 days) this looks across
# EVERY project and stage and assembles the raw lifetime data — release
# timeline, per-project dates, specs-by-stage, decision log.
#
# Modes:
#   data    (default) — a self-contained, LLM-free data report: every
#                       deterministic fact about the repo's lifetime.
#   prompt            — the same data wrapped in a synthesis prompt for an
#                       LLM to turn into a narrative arc (same posture as
#                       `just weekly-review`). Use when you want the story.
#
# Usage:
#   just lifetime-data                  # the data report (no LLM needed)
#   just lifetime-report                # the synthesis prompt (data + the ask)
#   just lifetime-save                  # data report -> reports/lifetime/<stamp>.md
#   ./scripts/lifetime-report.sh data
#   ./scripts/lifetime-report.sh prompt | pbcopy
#
# Ported from zany-animal-slots (SPEC-106 / STAGE-038 chore). Two crustyimg
# adaptations vs the original: releases come from CHANGELOG.md + git tags
# rather than a RELEASES.md this repo does not have, and the roadmap is a
# single docs/roadmap.md file rather than a docs/roadmap/ directory.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_lib.sh"

require_initialized

MODE="${1:-data}"
case "$MODE" in
    data|prompt) ;;
    *) echo "usage: lifetime-report.sh [data|prompt]" >&2; exit 2 ;;
esac

REPO_ID=$(get_repo_id)
VARIANT=$(get_variant)
ACTIVE_PROJECT=$(get_active_project)
GENERATED=$(today)

# ---------------------------------------------------------------------------
# emit_data — every deterministic fact about the repo's lifetime.
# No LLM, no prompting; just the aggregated history. Shared by both modes.
# ---------------------------------------------------------------------------
emit_data() {
    # --- Repo-level context (the source files behind the "why") ---
    echo "## Repo-level files (context for the 'why')"
    for f in AGENTS.md CHANGELOG.md README.md docs/roadmap.md docs/moat.md docs/backlog.md; do
        [ -e "${REPO_ROOT}/${f}" ] && echo "- $f"
    done
    for f in "${REPO_ROOT}"/projects/*/brief.md; do
        [ -e "$f" ] && echo "- ${f#"${REPO_ROOT}"/}"
    done
    echo ""

    # --- Per-project timeline (created / shipped / status from frontmatter) ---
    echo "## Projects (created → shipped, from brief frontmatter)"
    for brief in "${REPO_ROOT}"/projects/*/brief.md; do
        [ -e "$brief" ] || continue
        local proj_dir created shipped status
        proj_dir=$(basename "$(dirname "$brief")")
        created=$(grep -m1 -E '^created_at:' "$brief" | sed 's/created_at:[[:space:]]*//' || true)
        shipped=$(grep -m1 -E '^shipped_at:' "$brief" | sed 's/shipped_at:[[:space:]]*//' || true)
        status=$(grep -m1 -E '^[[:space:]]*status:' "$brief" | sed 's/.*status:[[:space:]]*//' | awk '{print $1}' || true)
        echo "- ${proj_dir}: created ${created:-?} → shipped ${shipped:-—}  (status: ${status:-?})"
    done
    echo ""

    # --- Releases. crustyimg has no RELEASES.md; the app's release history is
    #     the tag list (with dates) plus CHANGELOG.md's version headings. ---
    echo "## Releases (git tags, with dates)"
    if command -v git >/dev/null 2>&1 && [ -d "${REPO_ROOT}/.git" ]; then
        git -C "$REPO_ROOT" for-each-ref --sort=creatordate \
            --format='- %(refname:short)  %(creatordate:short)' refs/tags 2>/dev/null || true
    fi
    echo ""
    echo "## Release headings (from CHANGELOG.md)"
    grep -E '^## ' "${REPO_ROOT}/CHANGELOG.md" 2>/dev/null | sed 's|^## |- |' || true
    echo ""

    # --- Decisions across all projects ---
    echo "## Decisions (across all projects)"
    find "${REPO_ROOT}/decisions" -name "DEC-*.md" 2>/dev/null | sort \
        | sed "s|^${REPO_ROOT}/decisions/|- |" || true
    echo ""

    # --- Git span ---
    if command -v git >/dev/null 2>&1 && [ -d "${REPO_ROOT}/.git" ]; then
        echo "## Git span"
        # `|| true`: `git log | head -1` SIGPIPEs git once its output exceeds the
        # pipe buffer; without this, `set -euo pipefail` aborts before the status /
        # specs-by-stage aggregates below — the core of the report.
        local first last count
        first=$(git -C "$REPO_ROOT" log --reverse --format='%ad' --date=short 2>/dev/null | head -1) || true
        last=$(git  -C "$REPO_ROOT" log          --format='%ad' --date=short 2>/dev/null | head -1) || true
        count=$(git -C "$REPO_ROOT" rev-list --count HEAD 2>/dev/null)
        echo "- first commit: ${first:-?}   last commit: ${last:-?}   (${count:-?} commits)"
        echo "  (note: project dates in briefs predate the git history if the repo was re-inited)"
        echo ""
    fi

    # --- The mechanical aggregates the report tables are built from ---
    echo "## Repo status snapshot"
    echo '```'
    "${SCRIPT_DIR}/status.sh"
    echo '```'
    echo ""

    echo "## Specs by stage (full history)"
    echo '```'
    "${SCRIPT_DIR}/specs-by-stage.sh"
    echo '```'
    echo ""
}

# ---------------------------------------------------------------------------
# emit_glance — a deterministic at-a-glance count block (data mode only).
# ---------------------------------------------------------------------------
emit_glance() {
    local n_projects n_stages n_specs n_decisions n_releases first last count d
    n_projects=$(find "${REPO_ROOT}/projects" -maxdepth 2 -name brief.md 2>/dev/null | wc -l | tr -d ' ')
    n_stages=$(find "${REPO_ROOT}/projects" -path '*/stages/STAGE-*.md' 2>/dev/null | wc -l | tr -d ' ')
    n_decisions=$(count_files "${REPO_ROOT}/decisions" 'DEC-*.md')
    n_releases=$(git -C "$REPO_ROOT" tag 2>/dev/null | wc -l | tr -d ' ')

    # Specs across EVERY project, active + archived, excluding -timeline.md
    # companions (they match the same glob and would double every count).
    n_specs=0
    for d in "${REPO_ROOT}"/projects/*/specs; do
        [ -d "$d" ] || continue
        n_specs=$(( n_specs + $(count_specs "$d") + $(count_specs "${d}/done") ))
    done

    echo "## Lifetime at a glance"
    if command -v git >/dev/null 2>&1 && [ -d "${REPO_ROOT}/.git" ]; then
        first=$(git -C "$REPO_ROOT" log --reverse --format='%ad' --date=short 2>/dev/null | head -1) || true
        last=$(git  -C "$REPO_ROOT" log          --format='%ad' --date=short 2>/dev/null | head -1) || true
        count=$(git -C "$REPO_ROOT" rev-list --count HEAD 2>/dev/null)
        echo "- Lifespan:   ${first:-?} → ${last:-?}   (${count:-?} commits)"
    fi
    echo "- Projects:   ${n_projects}"
    echo "- Stages:     ${n_stages}"
    echo "- Specs:      ${n_specs}  (all projects, active + archived)"
    echo "- Decisions:  ${n_decisions}"
    echo "- Releases:   ${n_releases}  (git tags)"
    echo "  (recorded cost per spec: see the specs-by-stage totals below)"
    echo ""
}

if [ "$MODE" = "prompt" ]; then
    cat <<'EOF'
# ============================================================
# Lifetime Report — copy everything below this line into Claude
# ============================================================

Produce a full lifetime report on this repo: the arc of every project,
stage, spec, decision, and release since the first commit. Not a status
snapshot (`just status` already does that) — a narrative of how the tool
got from nothing to where it is now. Use the pre-loaded data below;
read the project briefs and CHANGELOG.md for the "why" behind each wave.

Head the report with `# <Repo> — Lifetime Report` and, directly under it, a
`Generated: <date>` line — use the `Generated:` date given in the context block
below the prompt (this is the date the report was produced; the git-span and
project dates below are the history it covers, which is a different thing).

Structure to produce:
  1. What this is — one paragraph on the product and the lifecycle model.
  2. Lifetime at a glance — a metrics table (lifespan, #projects, #stages,
     #specs shipped/deferred, #decisions, #releases, open low-confidence debt).
  3. Release timeline — version / date / theme / which project shipped it.
  4. The waves — one tight paragraph per project: dates, stage/spec counts,
     what it delivered, and WHY now (pull from each brief's "Why Now").
  5. Arc of the whole thing — the progression across waves in one line,
     plus how each wave made the next cheap.
  6. Decision debt & loose ends — low-confidence DECs, deferred/cut specs,
     stale flags.
  7. What's next — the next unframed wave (see docs/roadmap.md).

Keep it skimmable: tables where the data is tabular, prose only for the arc.
Do not inflate: if a number is not in the data below, say so rather than
estimating it.
EOF
    echo ""
    echo "Generated: ${GENERATED}"
    echo "Repo: ${REPO_ID}   ·   variant: ${VARIANT}   ·   active project: ${ACTIVE_PROJECT}"
    echo ""
    emit_data
    cat <<'EOF'
# ============================================================
# End of Lifetime Report prompt
# ============================================================
EOF
else
    # data mode — a self-contained report, no LLM required.
    echo "# ${REPO_ID} — Lifetime Data Report"
    echo "Generated: ${GENERATED}"
    echo "Repo: ${REPO_ID}   ·   variant: ${VARIANT}   ·   active project: ${ACTIVE_PROJECT}"
    echo ""
    emit_glance
    emit_data
    echo "---"
    echo "Data report only — no narrative. For a written lifetime story, run"
    echo "\`just lifetime-report\` and hand the prompt to an LLM."
fi
