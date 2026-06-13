#!/usr/bin/env bash
# scripts/new-spec.sh — scaffold a new spec.
# Usage: new-spec.sh "short title" STAGE-NNN [PROJ-NNN]

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_lib.sh"

require_initialized

TITLE="${1:-}"
STAGE_ID="${2:-}"
PROJECT_ID="${3:-}"

if [ -z "$TITLE" ] || [ -z "$STAGE_ID" ]; then
    die "Usage: just new-spec \"title\" STAGE-NNN [PROJ-NNN]"
fi

if [ -z "$PROJECT_ID" ]; then
    PROJECT_ID=$(get_active_project | sed -E 's/-.*//')
    # get_active_project returns PROJ-001-slug; we only want PROJ-001
    PROJECT_ID=$(get_active_project | awk -F- '{print $1"-"$2}')
fi

PROJECT_DIR=$(find "${REPO_ROOT}/projects" -maxdepth 1 -type d -name "${PROJECT_ID}-*" | head -n1)
if [ -z "$PROJECT_DIR" ]; then
    die "Project not found: ${PROJECT_ID}"
fi

# Verify stage exists in this project
STAGE_FILE=$(find "${PROJECT_DIR}/stages" -type f -name "${STAGE_ID}-*.md" 2>/dev/null | head -n1)
if [ -z "$STAGE_FILE" ]; then
    die "Stage not found in ${PROJECT_ID}: ${STAGE_ID}"
fi

SPEC_ID=$(next_id SPEC "${PROJECT_DIR}/specs")
SLUG=$(slugify "$TITLE")
SPEC_FILE="${PROJECT_DIR}/specs/${SPEC_ID}-${SLUG}.md"
VARIANT=$(get_variant)

if [ -f "$SPEC_FILE" ]; then
    die "Spec file already exists: ${SPEC_FILE}"
fi

# Choose template based on variant
if [ "$VARIANT" = "claude-plus-agents" ]; then
    TEMPLATE="${REPO_ROOT}/projects/_templates/spec.md"
else
    TEMPLATE="${REPO_ROOT}/projects/_templates/spec.md"
fi

if [ ! -f "$TEMPLATE" ]; then
    die "Template not found: ${TEMPLATE}. Did init run correctly?"
fi

# Copy template, substitute placeholders
cp "$TEMPLATE" "$SPEC_FILE"

# Use sed to substitute. Portable across macOS/Linux using a wrapper.
sed_inplace() {
    if [ "$(uname)" = "Darwin" ]; then
        sed -i '' "$@"
    else
        sed -i "$@"
    fi
}

# Escape user-controlled values before substituting them into the
# template (see sed_escape_replacement in _lib.sh). The ID/date values
# are tool-generated or validated, but escaping is harmless and keeps
# the substitutions uniform.
TITLE_ESC=$(sed_escape_replacement "$TITLE")
REPO_ID_ESC=$(sed_escape_replacement "$(get_repo_id)")

sed_inplace "s|SPEC-XXX|${SPEC_ID}|g" "$SPEC_FILE"
sed_inplace "s|STAGE-XXX|${STAGE_ID}|g" "$SPEC_FILE"
sed_inplace "s|PROJ-XXX|${PROJECT_ID}|g" "$SPEC_FILE"
sed_inplace "s|<Short Title>|${TITLE_ESC}|g" "$SPEC_FILE"
sed_inplace "s|__TODAY__|$(today)|g" "$SPEC_FILE"
sed_inplace "s|__REPO_ID__|${REPO_ID_ESC}|g" "$SPEC_FILE"

# Scaffold the timeline file alongside the spec. Architect appends
# cycle lines as it designs them; executors update status markers.
TIMELINE_TEMPLATE="${REPO_ROOT}/projects/_templates/timeline.md"
TIMELINE_FILE="${PROJECT_DIR}/specs/${SPEC_ID}-${SLUG}-timeline.md"
if [ -f "$TIMELINE_TEMPLATE" ]; then
    cp "$TIMELINE_TEMPLATE" "$TIMELINE_FILE"
    sed_inplace "s|SPEC-XXX|${SPEC_ID}|g" "$TIMELINE_FILE"
else
    # Fallback: inline minimal timeline so new-spec never hard-fails on
    # a freshly-cloned repo whose _templates/ is incomplete.
    cat > "$TIMELINE_FILE" <<EOF
# ${SPEC_ID} timeline

Status markers: \`[ ]\` not started · \`[~]\` in progress · \`[x]\` complete · \`[?]\` blocked.

## Instructions

_(Timeline will be populated as the architect writes each cycle's prompt.)_
EOF
fi

# Ensure prompts/ exists as a sibling to the spec file. The architect
# writes one prompt file per dispatched cycle here.
PROMPTS_DIR="${PROJECT_DIR}/specs/prompts"
mkdir -p "$PROMPTS_DIR"
[ -f "${PROMPTS_DIR}/.gitkeep" ] || touch "${PROMPTS_DIR}/.gitkeep"

success "Created ${SPEC_FILE}"
success "Created ${TIMELINE_FILE}"
echo ""
echo "Next steps:"
echo "  1. Fill in the spec with Claude (use Prompt 2b: SPEC from FIRST_SESSION_PROMPTS.md)"
echo "  2. Update the stage's backlog in ${STAGE_FILE}"
echo "  3. Architect will write cycle prompts to ${PROMPTS_DIR#$REPO_ROOT/}/"
echo "     and append lines to the timeline as cycles are designed."
echo "  4. When ready for build, run:"
echo "       just advance-cycle ${SPEC_ID} build"
