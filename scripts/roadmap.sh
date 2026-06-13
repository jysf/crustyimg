#!/usr/bin/env bash
# scripts/roadmap.sh — stage-grained "where is this project going" view.
#
# One row per stage in the active project:
#   - Stage ID + title
#   - Status: shipped / active / upcoming (proposed/on_hold both render
#     as "upcoming" — the user-facing distinction we care about is
#     "done", "happening now", or "later")
#   - Date range:
#       shipped → created_at → shipped_at
#       active  → created_at → ?
#       upcoming → target: target_complete (or "—" if unset)
#   - Spec counts (in-flight / backlog) for active and upcoming stages
#
# Read-only. No writes.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_lib.sh"

require_initialized

ACTIVE_PROJECT=$(get_active_project)
ACTIVE_DIR="${REPO_ROOT}/projects/${ACTIVE_PROJECT}"
STAGES_DIR="${ACTIVE_DIR}/stages"
SPECS_DIR="${ACTIVE_DIR}/specs"

# Count in-flight specs (cycle ≠ ship/archived) for a given stage ID.
count_in_flight_for_stage() {
    local stage_id="$1"
    local n=0 f c sid
    [ -d "$SPECS_DIR" ] || { echo 0; return; }
    for f in "${SPECS_DIR}"/SPEC-*.md; do
        [ -f "$f" ] || continue
        sid=$(awk '
            /^---$/ { fm = !fm; next }
            !fm { exit }
            /^project:/ { in_p = 1; next }
            in_p && /^[a-zA-Z_]/ { in_p = 0 }
            in_p && /^[[:space:]]+stage:/ { print $2; exit }
        ' "$f")
        if [ "$sid" = "$stage_id" ]; then
            c=$(get_spec_cycle "$f")
            case "$c" in
                frame|design|build|verify|ship) n=$((n + 1)) ;;
            esac
        fi
    done
    echo "$n"
}

# Count un-promoted "(not yet written)" bullets in a stage file.
count_backlog_bullets() {
    local f="$1"
    awk '
        /^## Spec Backlog/ { in_b = 1; next }
        in_b && /^## / { in_b = 0 }
        in_b && /\(not yet written\)/ { count++ }
        END { print count+0 }
    ' "$f"
}

echo "${BOLD}Roadmap for ${ACTIVE_PROJECT}${RESET}"
if [ ! -d "$STAGES_DIR" ]; then
    echo "  ${DIM}(no stages/ directory yet)${RESET}"
    exit 0
fi

# Determine the "active stage" once so we can highlight it.
ACTIVE_STAGE_FILE=$(get_active_stage_file "$ACTIVE_DIR" || true)
ACTIVE_STAGE_ID=""
if [ -n "$ACTIVE_STAGE_FILE" ]; then
    ACTIVE_STAGE_ID=$(basename "$ACTIVE_STAGE_FILE" .md | sed -E 's/^(STAGE-[0-9]+).*/\1/')
fi

any_stage=0
for s in "${STAGES_DIR}"/STAGE-*.md; do
    [ -f "$s" ] || continue
    any_stage=1
    sname=$(basename "$s" .md)
    sid=$(echo "$sname" | sed -E 's/^(STAGE-[0-9]+).*/\1/')
    status=$(get_stage_status "$s")

    # User-facing status bucket. Multiple "active" stages are possible
    # in the data; only the first one (per get_active_stage_file) is
    # tagged "active" here. Others fall into "upcoming" so the roadmap
    # renders one happening-now row at a time.
    bucket=""
    case "$status" in
        shipped)   bucket="shipped" ;;
        cancelled) bucket="cancelled" ;;
        active)
            if [ "$sid" = "$ACTIVE_STAGE_ID" ]; then bucket="active"
            else bucket="upcoming"; fi
            ;;
        *)         bucket="upcoming" ;;
    esac

    # Date column.
    date_col=""
    case "$bucket" in
        shipped)
            sa=$(get_stage_shipped_at "$s")
            ca=$(get_stage_created_at "$s")
            date_col="${ca:-?} → ${sa:-?}"
            ;;
        cancelled)
            ca=$(get_stage_created_at "$s")
            date_col="${ca:-?} → cancelled"
            ;;
        active)
            ca=$(get_stage_created_at "$s")
            date_col="${ca:-?} → ?"
            ;;
        upcoming)
            t=$(get_stage_target "$s")
            if [ -n "$t" ]; then date_col="target: $t"
            else date_col="target: —"; fi
            ;;
    esac

    # Spec counts column (only for active and upcoming).
    counts_col=""
    if [ "$bucket" = "active" ] || [ "$bucket" = "upcoming" ]; then
        in_flight=$(count_in_flight_for_stage "$sid")
        backlog=$(count_backlog_bullets "$s")
        counts_col="(${in_flight} in flight, ${backlog} backlog)"
    fi

    # Render. Bold the active row to make "you are here" obvious.
    if [ "$bucket" = "active" ]; then
        printf "  ${BOLD}%-36s %-10s %-22s${RESET} %s\n" \
            "$sname" "$bucket" "$date_col" "$counts_col"
    else
        printf "  %-36s %-10s %-22s %s\n" \
            "$sname" "$bucket" "$date_col" "$counts_col"
    fi
done
if [ "$any_stage" = "0" ]; then
    echo "  ${DIM}(no stages yet)${RESET}"
fi
