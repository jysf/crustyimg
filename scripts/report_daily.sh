#!/usr/bin/env bash
# scripts/report_daily.sh — daily report for the active project.
#
# Reads spec/stage/brief front-matter + git log; writes a quantitative
# snapshot to reports/daily/YYYY-MM-DD.md. Idempotent: re-running
# overwrites. Degrades gracefully on pre-migration data (no cost
# block, no value_link, no value: block).

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_lib.sh"

require_initialized

TODAY=$(today)
NOW=$(date "+%Y-%m-%d %H:%M:%S %Z")

OUT_DIR="${REPO_ROOT}/reports/daily"
mkdir -p "$OUT_DIR"
OUT="${OUT_DIR}/${TODAY}.md"

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

# --- Write report ---
{
    echo "# Daily report — ${TODAY}"
    echo ""
    echo "*Generated ${NOW}. Re-run \`just report-daily\` to refresh.*"
    echo ""
    echo "- **Project:** ${ACTIVE_PROJECT}"
    if [ -n "$ACTIVE_STAGE_NAME" ]; then
        echo "- **Active stage:** ${ACTIVE_STAGE_NAME}"
    else
        echo "- **Active stage:** (none — no stages in this project yet)"
    fi
    echo ""

    # ---------- Snapshot ----------
    echo "## Snapshot"
    echo ""
    echo "**Specs by cycle (active project):**"
    echo ""
    echo '```'
    if [ -d "$SPECS_DIR" ]; then
        for cycle in frame design build verify ship; do
            count=0
            ids=""
            for f in "${SPECS_DIR}"/SPEC-*.md; do
                [ -f "$f" ] || continue
                c=$(get_spec_cycle "$f")
                if [ "$c" = "$cycle" ]; then
                    count=$((count + 1))
                    id=$(basename "$f" .md | sed -E 's/^(SPEC-[0-9]+).*/\1/')
                    if [ -z "$ids" ]; then ids="$id"; else ids="${ids}, ${id}"; fi
                fi
            done
            if [ "$cycle" = "ship" ] && [ -d "${SPECS_DIR}/done" ]; then
                for f in "${SPECS_DIR}/done"/SPEC-*.md; do
                    [ -f "$f" ] || continue
                    count=$((count + 1))
                done
            fi
            if [ -n "$ids" ]; then
                printf "  %-7s %d  (%s)\n" "$cycle:" "$count" "$ids"
            else
                printf "  %-7s %d\n" "$cycle:" "$count"
            fi
        done
    else
        echo "  (no specs/ directory yet)"
    fi
    echo '```'
    echo ""

    # Project progress: shipped vs all in project
    total_specs=0
    shipped_specs=0
    if [ -d "$SPECS_DIR" ]; then
        total_specs=$(find "$SPECS_DIR" -type f -name "SPEC-*.md" 2>/dev/null | wc -l | tr -d ' ')
        shipped_specs=$(find "${SPECS_DIR}/done" -type f -name "SPEC-*.md" 2>/dev/null | wc -l | tr -d ' ')
    fi
    active_specs=$((total_specs - shipped_specs))
    if [ "$total_specs" -gt 0 ]; then
        pct=$(awk -v s="$shipped_specs" -v t="$total_specs" 'BEGIN{printf "%.0f", (s/t)*100}')
        echo "**Project progress:** ${shipped_specs} shipped / ${total_specs} scaffolded (${pct}%). ${active_specs} active."
    else
        echo "**Project progress:** no specs scaffolded yet."
    fi
    echo ""

    # ---------- Value ----------
    echo "## Value"
    echo ""
    thesis=$(get_project_thesis "$ACTIVE_DIR" || true)
    if [ -n "$thesis" ]; then
        echo "**Project thesis:** ${thesis}"
    else
        echo "**Project thesis:** *(not set — project brief has no value.thesis)*"
    fi
    echo ""
    if [ -n "$ACTIVE_STAGE_FILE" ]; then
        advances=$(get_stage_value_contribution "$ACTIVE_STAGE_FILE" || true)
        if [ -n "$advances" ]; then
            echo "**${ACTIVE_STAGE_ID} advances:** ${advances}"
        else
            echo "**${ACTIVE_STAGE_ID} advances:** *(value_contribution.advances not set)*"
        fi
        echo ""
    fi

    # value_link population across active specs
    linked=0
    unlinked=0
    if [ -d "$SPECS_DIR" ]; then
        for f in "${SPECS_DIR}"/SPEC-*.md; do
            [ -f "$f" ] || continue
            vl=$(extract_value_link "$f" || true)
            if [ -n "$vl" ]; then linked=$((linked + 1)); else unlinked=$((unlinked + 1)); fi
        done
    fi
    echo "**value_link population (active specs):** ${linked} with, ${unlinked} without."
    echo ""

    # ---------- Activity today ----------
    echo "## Activity today"
    echo ""
    # Specs modified today
    modified_today=()
    if [ -d "$SPECS_DIR" ]; then
        while IFS= read -r -d '' f; do
            mtime=$(spec_mtime_date "$f")
            if [ "$mtime" = "$TODAY" ]; then
                modified_today+=("$f")
            fi
        done < <(find "$SPECS_DIR" -type f -name "SPEC-*.md" -print0 2>/dev/null)
    fi
    if [ "${#modified_today[@]}" -gt 0 ]; then
        echo "**Specs touched today:**"
        echo ""
        for f in "${modified_today[@]}"; do
            name=$(basename "$f" .md)
            c=$(get_spec_cycle "$f")
            echo "- ${name} (cycle: ${c:-unknown})"
        done
        echo ""
    else
        echo "*No specs touched today.*"
        echo ""
    fi

    # Decisions added today
    dec_today=0
    if [ -d "${REPO_ROOT}/decisions" ]; then
        while IFS= read -r -d '' f; do
            mtime=$(spec_mtime_date "$f")
            if [ "$mtime" = "$TODAY" ]; then dec_today=$((dec_today + 1)); fi
        done < <(find "${REPO_ROOT}/decisions" -type f -name "DEC-*.md" -print0 2>/dev/null)
    fi
    echo "**Decisions touched today:** ${dec_today}"
    echo ""

    # Feedback captured today
    fb_today=0
    if [ -d "${REPO_ROOT}/feedback" ]; then
        while IFS= read -r -d '' f; do
            mtime=$(spec_mtime_date "$f")
            if [ "$mtime" = "$TODAY" ]; then fb_today=$((fb_today + 1)); fi
        done < <(find "${REPO_ROOT}/feedback" -maxdepth 1 -type f -name "*.md" -print0 2>/dev/null)
    fi
    echo "**Feedback files touched today:** ${fb_today}"
    echo ""

    # ---------- Cost activity today ----------
    echo "## Cost activity"
    echo ""
    today_sessions=0
    today_usd=0
    wip_usd=0
    missing_cost=()
    if [ -d "$SPECS_DIR" ]; then
        while IFS= read -r -d '' f; do
            s=$(sessions_recorded_on "$f" "$TODAY")
            today_sessions=$((today_sessions + s))
            sc=$(count_cost_sessions "$f")
            if [ "$sc" = "0" ]; then
                name=$(basename "$f" .md)
                missing_cost+=("$name")
            fi
        done < <(find "$SPECS_DIR" -type f -name "SPEC-*.md" -not -path '*/done/*' -print0 2>/dev/null)

        # WIP cost: sum of sessions across active specs
        while IFS= read -r -d '' f; do
            usd=$(sum_cost_usd_for_spec "$f")
            wip_usd=$(awk -v a="$wip_usd" -v b="$usd" 'BEGIN{printf "%.2f", a+b}')
        done < <(find "$SPECS_DIR" -type f -name "SPEC-*.md" -not -path '*/done/*' -print0 2>/dev/null)
    fi
    echo "- **Cost sessions recorded today:** ${today_sessions}"
    echo "- **WIP accumulated cost (active specs):** \$${wip_usd}"
    if [ "${#missing_cost[@]}" -gt 0 ]; then
        echo "- **Specs with no cost data yet** (${#missing_cost[@]}):"
        for s in "${missing_cost[@]}"; do echo "  - ${s}"; done
    else
        echo "- Specs with no cost data yet: none."
    fi
    echo ""

    # ---------- Flags ----------
    echo "## Flags"
    echo ""
    # Stalled specs: build/verify, mtime > 7 days
    stalled=()
    if [ -d "$SPECS_DIR" ]; then
        while IFS= read -r -d '' f; do
            c=$(get_spec_cycle "$f")
            if [ "$c" = "build" ] || [ "$c" = "verify" ]; then
                if [ "$(uname)" = "Darwin" ]; then
                    age_days=$(( ( $(date +%s) - $(stat -f %m "$f") ) / 86400 ))
                else
                    age_days=$(( ( $(date +%s) - $(stat -c %Y "$f") ) / 86400 ))
                fi
                if [ "$age_days" -gt 7 ]; then
                    name=$(basename "$f" .md)
                    stalled+=("${name} (${c}, ${age_days}d)")
                fi
            fi
        done < <(find "$SPECS_DIR" -type f -name "SPEC-*.md" -not -path '*/done/*' -print0 2>/dev/null)
    fi
    if [ "${#stalled[@]}" -gt 0 ]; then
        echo "**Stalled specs** (build/verify, idle >7 days by mtime):"
        for s in "${stalled[@]}"; do echo "- ${s}"; done
    else
        echo "**Stalled specs:** none."
    fi
    echo ""

    # Low-confidence decisions not touched in 14 days
    low_conf_stale=()
    if [ -d "${REPO_ROOT}/decisions" ]; then
        while IFS= read -r -d '' f; do
            conf=$(awk '/^---$/{fm=!fm; next} fm && /^[[:space:]]+confidence:/{print $2; exit}' "$f" 2>/dev/null || echo "")
            if [ -n "$conf" ]; then
                low=$(awk -v c="$conf" 'BEGIN{print (c+0<0.7)?"1":"0"}')
                if [ "$low" = "1" ]; then
                    if [ "$(uname)" = "Darwin" ]; then
                        age=$(( ( $(date +%s) - $(stat -f %m "$f") ) / 86400 ))
                    else
                        age=$(( ( $(date +%s) - $(stat -c %Y "$f") ) / 86400 ))
                    fi
                    if [ "$age" -gt 14 ]; then
                        low_conf_stale+=("$(basename "$f" .md) (conf=${conf}, ${age}d)")
                    fi
                fi
            fi
        done < <(find "${REPO_ROOT}/decisions" -type f -name "DEC-*.md" -print0 2>/dev/null)
    fi
    if [ "${#low_conf_stale[@]}" -gt 0 ]; then
        echo "**Low-confidence decisions (<0.7) stale >14d:**"
        for d in "${low_conf_stale[@]}"; do echo "- ${d}"; done
    else
        echo "**Low-confidence decisions stale >14d:** none."
    fi
    echo ""

    # Open questions older than 14 days
    q_file="${REPO_ROOT}/guidance/questions.yaml"
    if [ -f "$q_file" ]; then
        if [ "$(uname)" = "Darwin" ]; then
            q_age=$(( ( $(date +%s) - $(stat -f %m "$q_file") ) / 86400 ))
        else
            q_age=$(( ( $(date +%s) - $(stat -c %Y "$q_file") ) / 86400 ))
        fi
        if [ "$q_age" -gt 14 ]; then
            echo "**questions.yaml** untouched for ${q_age} days — review for resolved items."
        fi
    fi
    echo ""

    # ---------- Git activity ----------
    echo "## Git activity (last 24h)"
    echo ""
    if git -C "$REPO_ROOT" rev-parse --git-dir >/dev/null 2>&1; then
        commits=$(git -C "$REPO_ROOT" log --since='24 hours ago' --oneline 2>/dev/null | wc -l | tr -d ' ')
        reverts=$(git -C "$REPO_ROOT" log --since='24 hours ago' --oneline --grep='^Revert' 2>/dev/null | wc -l | tr -d ' ')
        echo "- Commits: ${commits}"
        echo "- Reverts: ${reverts}"
    else
        echo "*(not a git repo)*"
    fi
    echo ""
    echo "---"
    echo "*Re-run with \`just report-daily\`.*"
} > "$OUT"

# Print path + show the output for immediate visibility
echo "${GREEN}✓${RESET} wrote ${OUT#$REPO_ROOT/}"
echo ""
cat "$OUT"
