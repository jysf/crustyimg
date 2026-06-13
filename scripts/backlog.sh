#!/usr/bin/env bash
# scripts/backlog.sh — spec-grained "what should I pick up next" view.
#
# Surfaces three things `just status` deliberately doesn't:
#   1. In-flight specs (cycle ≠ ship/archived) grouped by cycle, scoped
#      to the active stage by default.
#   2. Stage backlog bullets that haven't been promoted to specs yet —
#      the `(not yet written)` rows in the active stage's `## Spec
#      Backlog`. This is the main new value vs. status.
#   3. Counts (not contents) of un-promoted bullets in upcoming stages.
#
# `--all` expands #1 and #2 to non-active stages too.
#
# Read-only. No writes.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_lib.sh"

require_initialized

SHOW_ALL=0
for arg in "$@"; do
    case "$arg" in
        --all) SHOW_ALL=1 ;;
        -h|--help)
            cat <<EOF
Usage: just backlog [--all]

  --all   Include in-flight specs and un-promoted bullets across all
          stages, not just the active one. Counts for upcoming stages
          stay rolled up.
EOF
            exit 0
            ;;
        *) die "Unknown argument: $arg (try --help)" ;;
    esac
done

ACTIVE_PROJECT=$(get_active_project)
ACTIVE_DIR="${REPO_ROOT}/projects/${ACTIVE_PROJECT}"
STAGES_DIR="${ACTIVE_DIR}/stages"
SPECS_DIR="${ACTIVE_DIR}/specs"

ACTIVE_STAGE_FILE=$(get_active_stage_file "$ACTIVE_DIR" || true)
ACTIVE_STAGE_ID=""
ACTIVE_STAGE_NAME=""
if [ -n "$ACTIVE_STAGE_FILE" ]; then
    ACTIVE_STAGE_NAME=$(basename "$ACTIVE_STAGE_FILE" .md)
    ACTIVE_STAGE_ID=$(echo "$ACTIVE_STAGE_NAME" | sed -E 's/^(STAGE-[0-9]+).*/\1/')
fi

# --- Helpers --------------------------------------------------------

# Read complexity (S/M/L) from spec front-matter.
get_spec_complexity() {
    awk '
        /^---$/ { fm = !fm; next }
        !fm { exit }
        /^task:/ { in_t = 1; next }
        in_t && /^[a-zA-Z_]/ { in_t = 0 }
        in_t && /^[[:space:]]+complexity:/ { print $2; exit }
    ' "$1"
}

# Read project.stage from a spec.
get_spec_stage_id() {
    awk '
        /^---$/ { fm = !fm; next }
        !fm { exit }
        /^project:/ { in_p = 1; next }
        in_p && /^[a-zA-Z_]/ { in_p = 0 }
        in_p && /^[[:space:]]+stage:/ { print $2; exit }
    ' "$1"
}

# Extract un-promoted "(not yet written)" bullets from a stage's
# ## Spec Backlog section. One bullet per line.
# Convention: `- [ ] (not yet written) — <summary>` with optional
# `[S]/[M]/[L]` complexity tag anywhere on the line.
extract_unpromoted_bullets() {
    awk '
        /^## Spec Backlog/ { in_b = 1; next }
        in_b && /^## / { in_b = 0 }
        in_b && /\(not yet written\)/ { print }
    ' "$1"
}

# Trim a bullet line to "summary [complexity]" form. Strips the
# leading `- [ ] (not yet written) — ` and surfaces a complexity
# tag if present. Best-effort; the convention isn't enforced.
format_unpromoted_bullet() {
    local line="$1"
    local summary
    summary=$(echo "$line" \
        | sed -E 's/^[[:space:]]*-[[:space:]]*\[[ x~?]\][[:space:]]*//' \
        | sed -E 's/\(not yet written\)[[:space:]]*—[[:space:]]*//' \
        | sed -E 's/\(not yet written\)[[:space:]]*-[[:space:]]*//')
    # Best-effort complexity extraction: a bracketed S/M/L token.
    local complexity=""
    if [[ "$summary" =~ \[([SML])\] ]]; then
        complexity="${BASH_REMATCH[1]}"
        summary=$(echo "$summary" | sed -E 's/[[:space:]]*\[[SML]\][[:space:]]*//')
    fi
    summary=$(echo "$summary" | sed -E 's/[[:space:]]+$//')
    if [ -n "$complexity" ]; then
        printf "%-50s [%s]\n" "$summary" "$complexity"
    else
        printf "%s\n" "$summary"
    fi
}

# Count un-promoted bullets in a stage file.
count_unpromoted_bullets() {
    extract_unpromoted_bullets "$1" | wc -l | tr -d ' '
}

# --- Output ---------------------------------------------------------

echo "${BOLD}Backlog for ${ACTIVE_PROJECT}${RESET}"
if [ -n "$ACTIVE_STAGE_NAME" ]; then
    echo "  ${DIM}Active stage:${RESET} ${ACTIVE_STAGE_NAME}"
else
    echo "  ${DIM}(no active stage in this project yet)${RESET}"
fi
echo ""

# 1) In-flight specs --------------------------------------------------
echo "${BOLD}Specs in flight${RESET}"
if [ -d "$SPECS_DIR" ]; then
    in_flight=()
    for f in "${SPECS_DIR}"/SPEC-*.md; do
        [ -f "$f" ] || continue
        cycle=$(get_spec_cycle "$f")
        # ship cycle still counts as "in flight" only until archived;
        # archived specs live in done/ and we already excluded that.
        case "$cycle" in
            frame|design|build|verify|ship)
                stage_id=$(get_spec_stage_id "$f")
                if [ "$SHOW_ALL" = "1" ] || [ "$stage_id" = "$ACTIVE_STAGE_ID" ]; then
                    name=$(basename "$f" .md)
                    complexity=$(get_spec_complexity "$f")
                    in_flight+=("$cycle|$name|${complexity:-?}|$stage_id")
                fi
                ;;
        esac
    done
    if [ "${#in_flight[@]}" -gt 0 ]; then
        # Sort by cycle order, then by name.
        printf "  ${DIM}(%d total)${RESET}\n" "${#in_flight[@]}"
        for cycle in frame design build verify ship; do
            for entry in "${in_flight[@]}"; do
                IFS='|' read -r c n cx s <<< "$entry"
                if [ "$c" = "$cycle" ]; then
                    if [ "$SHOW_ALL" = "1" ]; then
                        printf "    %-32s cycle: %-7s complexity: %-2s ${DIM}(%s)${RESET}\n" "$n" "$c" "$cx" "$s"
                    else
                        printf "    %-32s cycle: %-7s complexity: %s\n" "$n" "$c" "$cx"
                    fi
                fi
            done
        done
    else
        if [ "$SHOW_ALL" = "1" ]; then
            echo "  ${DIM}(none across all stages)${RESET}"
        else
            echo "  ${DIM}(none in active stage; try --all)${RESET}"
        fi
    fi
else
    echo "  ${DIM}(no specs/ directory yet)${RESET}"
fi
echo ""

# 2) Un-promoted bullets in active stage (or all) ---------------------
echo "${BOLD}Stage backlog (not yet specced)${RESET}"
if [ -d "$STAGES_DIR" ]; then
    if [ "$SHOW_ALL" = "1" ]; then
        any=0
        for s in "${STAGES_DIR}"/STAGE-*.md; do
            [ -f "$s" ] || continue
            sid=$(basename "$s" .md | sed -E 's/^(STAGE-[0-9]+).*/\1/')
            bullets=$(extract_unpromoted_bullets "$s")
            if [ -n "$bullets" ]; then
                any=1
                echo "  — ${sid}"
                while IFS= read -r line; do
                    echo "    $(format_unpromoted_bullet "$line")"
                done <<< "$bullets"
            fi
        done
        if [ "$any" = "0" ]; then echo "  ${DIM}(none)${RESET}"; fi
    else
        if [ -n "$ACTIVE_STAGE_FILE" ]; then
            bullets=$(extract_unpromoted_bullets "$ACTIVE_STAGE_FILE")
            if [ -n "$bullets" ]; then
                echo "  ${DIM}— ${ACTIVE_STAGE_ID}${RESET}"
                while IFS= read -r line; do
                    echo "    $(format_unpromoted_bullet "$line")"
                done <<< "$bullets"
            else
                echo "  ${DIM}(no un-promoted bullets in ${ACTIVE_STAGE_ID})${RESET}"
            fi
        else
            echo "  ${DIM}(no active stage)${RESET}"
        fi
    fi
else
    echo "  ${DIM}(no stages/ directory yet)${RESET}"
fi
echo ""

# 3) Upcoming-stage rollup --------------------------------------------
echo "${BOLD}Upcoming stages (counts only)${RESET}"
if [ -d "$STAGES_DIR" ]; then
    upcoming_any=0
    for s in "${STAGES_DIR}"/STAGE-*.md; do
        [ -f "$s" ] || continue
        sid=$(basename "$s" .md | sed -E 's/^(STAGE-[0-9]+).*/\1/')
        # Skip the active stage and shipped stages.
        if [ "$sid" = "$ACTIVE_STAGE_ID" ]; then continue; fi
        status=$(get_stage_status "$s")
        if [ "$status" = "shipped" ] || [ "$status" = "cancelled" ]; then continue; fi
        sname=$(basename "$s" .md)
        count=$(count_unpromoted_bullets "$s")
        printf "  %-44s %d backlog item%s\n" "$sname" "$count" "$([ "$count" = "1" ] && echo "" || echo "s")"
        upcoming_any=1
    done
    if [ "$upcoming_any" = "0" ]; then echo "  ${DIM}(no upcoming stages)${RESET}"; fi
else
    echo "  ${DIM}(no stages/ directory yet)${RESET}"
fi
