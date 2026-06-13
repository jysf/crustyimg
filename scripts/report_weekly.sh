#!/usr/bin/env bash
# scripts/report_weekly.sh — weekly report for the active project.
#
# Writes reports/weekly/YYYY-WNN.md covering the ISO week containing
# today (or the explicit date passed as $1, if any). Idempotent:
# re-running overwrites. Graceful on pre-v5.2 data.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_lib.sh"

require_initialized

# Accept an optional YYYY-MM-DD to report on that week; default today.
ANCHOR_DATE="${1:-$(today)}"
WEEK_ID=$(iso_week_number "$ANCHOR_DATE")
# iso_week_bounds emits two lines (start, end). Read both.
# `read` returns non-zero at EOF-without-newline; sink with `|| true`
# so set -e doesn't kill us.
_bounds=$(iso_week_bounds "$ANCHOR_DATE")
WEEK_START=$(echo "$_bounds" | sed -n '1p')
WEEK_END=$(echo "$_bounds" | sed -n '2p')
NOW=$(date "+%Y-%m-%d %H:%M:%S %Z")

OUT_DIR="${REPO_ROOT}/reports/weekly"
mkdir -p "$OUT_DIR"
OUT="${OUT_DIR}/${WEEK_ID}.md"

ACTIVE_PROJECT=$(get_active_project)
ACTIVE_DIR="${REPO_ROOT}/projects/${ACTIVE_PROJECT}"
STAGES_DIR="${ACTIVE_DIR}/stages"
SPECS_DIR="${ACTIVE_DIR}/specs"

# Shipped-this-week = file in specs/done/ whose mtime date is within
# [WEEK_START, WEEK_END]. We use mtime as a pragmatic proxy for ship
# date; archive-spec touches the file when it moves it.
shipped_this_week_files() {
    [ -d "${SPECS_DIR}/done" ] || return 0
    local f d
    for f in "${SPECS_DIR}/done"/SPEC-*.md; do
        [ -f "$f" ] || continue
        d=$(spec_mtime_date "$f")
        if [[ "$d" > "$WEEK_START" || "$d" == "$WEEK_START" ]] && \
           [[ "$d" < "$WEEK_END" || "$d" == "$WEEK_END" ]]; then
            echo "$f"
        fi
    done
}

# Front-matter helper: extract task.<key> for a spec.
fm_task() {
    local file="$1" key="$2"
    awk -v k="$key" '
        /^---$/ { fm = !fm; next }
        !fm { exit }
        /^task:/ { in_t = 1; next }
        in_t && /^[a-zA-Z_]/ { in_t = 0 }
        in_t && $1 == k":" { sub(/^[[:space:]]+[a-zA-Z_]+:[[:space:]]*/, ""); print; exit }
    ' "$file"
}

# Front-matter helper: project.stage
fm_project_stage() {
    awk '
        /^---$/ { fm = !fm; next }
        !fm { exit }
        /^project:/ { in_p = 1; next }
        in_p && /^[a-zA-Z_]/ { in_p = 0 }
        in_p && /^[[:space:]]+stage:/ { print $2; exit }
    ' "$1"
}

# Title from the first "# SPEC-NNN: ..." line
spec_title() {
    awk '/^# SPEC-[0-9]+:/ { sub(/^# SPEC-[0-9]+:[[:space:]]*/, ""); print; exit }' "$1"
}

# ---------- Collect shipped-this-week first (used in many sections) ----------
SHIPPED_FILES=()
while IFS= read -r line; do
    [ -n "$line" ] && SHIPPED_FILES+=("$line")
done < <(shipped_this_week_files)

# ---------- Totals ----------
TOTAL_USD="0.00"
TOTAL_TOKENS=0
for f in "${SHIPPED_FILES[@]:-}"; do
    [ -n "$f" ] || continue
    usd=$(sum_cost_usd_for_spec "$f")
    tok=$(sum_cost_tokens_for_spec "$f")
    TOTAL_USD=$(awk -v a="$TOTAL_USD" -v b="$usd" 'BEGIN{printf "%.2f", a+b}')
    TOTAL_TOKENS=$((TOTAL_TOKENS + tok))
done
SHIPPED_COUNT=${#SHIPPED_FILES[@]}
AVG_USD="0.00"
if [ "$SHIPPED_COUNT" -gt 0 ]; then
    AVG_USD=$(awk -v t="$TOTAL_USD" -v n="$SHIPPED_COUNT" 'BEGIN{printf "%.2f", t/n}')
fi

# ---------- Write report ----------
{
    echo "# Weekly report — ${WEEK_ID}"
    echo ""
    echo "*Generated ${NOW}. Covers ${WEEK_START} → ${WEEK_END}.*"
    echo ""
    echo "- **Project:** ${ACTIVE_PROJECT}"
    echo ""

    # ---------- Summary ----------
    echo "## Summary"
    echo ""
    echo '```'
    printf "  %-28s %d\n" "Specs shipped:" "$SHIPPED_COUNT"
    # Decisions emitted (DEC files whose mtime is within week)
    dec_emitted=0
    if [ -d "${REPO_ROOT}/decisions" ]; then
        while IFS= read -r -d '' f; do
            d=$(spec_mtime_date "$f")
            if [[ "$d" > "$WEEK_START" || "$d" == "$WEEK_START" ]] && \
               [[ "$d" < "$WEEK_END" || "$d" == "$WEEK_END" ]]; then
                dec_emitted=$((dec_emitted + 1))
            fi
        done < <(find "${REPO_ROOT}/decisions" -type f -name "DEC-*.md" -print0 2>/dev/null)
    fi
    printf "  %-28s %d\n" "Decisions touched:" "$dec_emitted"
    fb_touched=0
    if [ -d "${REPO_ROOT}/feedback" ]; then
        while IFS= read -r -d '' f; do
            d=$(spec_mtime_date "$f")
            if [[ "$d" > "$WEEK_START" || "$d" == "$WEEK_START" ]] && \
               [[ "$d" < "$WEEK_END" || "$d" == "$WEEK_END" ]]; then
                fb_touched=$((fb_touched + 1))
            fi
        done < <(find "${REPO_ROOT}/feedback" -maxdepth 1 -type f -name "*.md" -print0 2>/dev/null)
    fi
    printf "  %-28s %d\n" "Feedback captured:" "$fb_touched"
    printf "  %-28s \$%s\n" "Total AI cost:" "$TOTAL_USD"
    printf "  %-28s \$%s\n" "Avg cost per shipped spec:" "$AVG_USD"
    printf "  %-28s %d\n" "Total tokens (shipped):" "$TOTAL_TOKENS"
    echo '```'
    echo ""

    # ---------- Value advancement ----------
    echo "## Value advancement"
    echo ""
    thesis=$(get_project_thesis "$ACTIVE_DIR" || true)
    if [ -n "$thesis" ]; then
        echo "**Project thesis:** ${thesis}"
    else
        echo "**Project thesis:** *(not set)*"
    fi
    echo ""

    # Stages that shipped this week (status moves to shipped OR mtime in range)
    echo "**Shipped stages (mtime in range):**"
    echo ""
    shipped_stages=0
    if [ -d "$STAGES_DIR" ]; then
        for s in "${STAGES_DIR}"/STAGE-*.md; do
            [ -f "$s" ] || continue
            stat=$(awk '/^---$/{f=!f; next} f && /^[[:space:]]+status:/{print $2; exit}' "$s" 2>/dev/null || echo "")
            d=$(spec_mtime_date "$s")
            if [ "$stat" = "shipped" ] && \
               { [[ "$d" > "$WEEK_START" || "$d" == "$WEEK_START" ]] && \
                 [[ "$d" < "$WEEK_END" || "$d" == "$WEEK_END" ]]; }; then
                sid=$(basename "$s" .md | sed -E 's/^(STAGE-[0-9]+).*/\1/')
                adv=$(get_stage_value_contribution "$s" || true)
                echo "- ${sid}: ${adv:-*(no value_contribution set)*}"
                shipped_stages=$((shipped_stages + 1))
            fi
        done
    fi
    if [ "$shipped_stages" -eq 0 ]; then echo "*None this week.*"; fi
    echo ""

    # Value-link coverage on shipped specs
    linked=0
    unlinked=0
    for f in "${SHIPPED_FILES[@]:-}"; do
        [ -n "$f" ] || continue
        vl=$(extract_value_link "$f" || true)
        if [ -n "$vl" ]; then linked=$((linked + 1)); else unlinked=$((unlinked + 1)); fi
    done
    echo "**Shipped-spec value_link coverage:** ${linked} linked, ${unlinked} not."
    if [ "$unlinked" -gt 0 ] && [ "$SHIPPED_COUNT" -gt 0 ]; then
        echo ""
        echo "*Flag:* ${unlinked} shipped spec(s) have null/missing value_link."
    fi
    echo ""

    # ---------- Shipped this week table ----------
    echo "## Shipped this week"
    echo ""
    if [ "$SHIPPED_COUNT" -gt 0 ]; then
        echo "| Spec | Title | Stage | Ship date | Cost (\$) | Value link |"
        echo "|---|---|---|---|---|---|"
        for f in "${SHIPPED_FILES[@]}"; do
            [ -n "$f" ] || continue
            name=$(basename "$f" .md | sed -E 's/^(SPEC-[0-9]+).*/\1/')
            title=$(spec_title "$f")
            stage=$(fm_project_stage "$f")
            shipd=$(spec_mtime_date "$f")
            usd=$(sum_cost_usd_for_spec "$f")
            vl=$(extract_value_link "$f" || true)
            vl_short="${vl:0:60}"
            [ -n "$vl_short" ] || vl_short="—"
            echo "| ${name} | ${title} | ${stage:-?} | ${shipd} | ${usd} | ${vl_short} |"
        done
    else
        echo "*No specs shipped this week.*"
    fi
    echo ""

    # ---------- Cost breakdown ----------
    echo "## Cost breakdown"
    echo ""
    echo "- **Total this week:** \$${TOTAL_USD}"
    echo "- **Total tokens:** ${TOTAL_TOKENS}"
    echo ""

    # Cost by cycle — walk shipped specs' sessions and bucket
    echo "**By cycle (shipped specs):**"
    echo ""
    echo '```'
    for cycle in design build verify ship; do
        c_usd="0.00"
        for f in "${SHIPPED_FILES[@]:-}"; do
            [ -n "$f" ] || continue
            sum=$(awk -v cy="$cycle" '
                /^---$/ { fm = !fm; next }
                !fm { next }
                /^cost:/ { in_cost = 1; next }
                in_cost && /^[a-zA-Z_]/ { in_cost = 0 }
                in_cost && /^  sessions:/ { in_s = 1; next }
                in_cost && in_s && /^  [a-zA-Z_]/ { in_s = 0 }
                in_s && /^    - cycle:/ { cur = $3 }
                in_s && /^      estimated_usd:/ {
                    if (cur == cy && $2 ~ /^[0-9]+(\.[0-9]+)?$/) total += $2
                }
                END { printf "%.2f", total+0 }
            ' "$f")
            c_usd=$(awk -v a="$c_usd" -v b="$sum" 'BEGIN{printf "%.2f", a+b}')
        done
        printf "  %-8s \$%s\n" "$cycle:" "$c_usd"
    done
    echo '```'
    echo ""

    # Cost by interface
    echo "**By interface (shipped specs):**"
    echo ""
    if [ "$SHIPPED_COUNT" -gt 0 ]; then
        echo '```'
        for f in "${SHIPPED_FILES[@]}"; do
            awk '
                /^---$/ { fm = !fm; next }
                !fm { next }
                /^cost:/ { in_cost = 1; next }
                in_cost && /^[a-zA-Z_]/ { in_cost = 0 }
                in_cost && /^  sessions:/ { in_s = 1; next }
                in_cost && in_s && /^  [a-zA-Z_]/ { in_s = 0 }
                in_s && /^      interface:/ { iface = $2 }
                in_s && /^      estimated_usd:/ {
                    if ($2 ~ /^[0-9]+(\.[0-9]+)?$/ && iface != "") {
                        sums[iface] += $2
                    }
                }
                END {
                    for (k in sums) printf "%s %.2f\n", k, sums[k]
                }
            ' "$f"
        done | awk '{ sums[$1] += $2 } END { for (k in sums) printf "  %-15s $%.2f\n", k":", sums[k] }'
        echo '```'
    else
        echo "*No cost data this week.*"
    fi
    echo ""

    # Top 3 most expensive shipped specs
    if [ "$SHIPPED_COUNT" -gt 0 ]; then
        echo "**Top 3 most expensive:**"
        echo ""
        for f in "${SHIPPED_FILES[@]}"; do
            [ -n "$f" ] || continue
            usd=$(sum_cost_usd_for_spec "$f")
            name=$(basename "$f" .md | sed -E 's/^(SPEC-[0-9]+).*/\1/')
            echo "${usd} ${name}"
        done | sort -rn | head -n3 | while read -r usd name; do
            echo "- ${name}: \$${usd}"
        done
        echo ""
    fi

    # ---------- Decision activity ----------
    echo "## Decision activity"
    echo ""
    if [ "$dec_emitted" -gt 0 ]; then
        echo "**Touched this week:**"
        if [ -d "${REPO_ROOT}/decisions" ]; then
            while IFS= read -r -d '' f; do
                d=$(spec_mtime_date "$f")
                if [[ "$d" > "$WEEK_START" || "$d" == "$WEEK_START" ]] && \
                   [[ "$d" < "$WEEK_END" || "$d" == "$WEEK_END" ]]; then
                    conf=$(awk '/^---$/{fm=!fm; next} fm && /^[[:space:]]+confidence:/{print $2; exit}' "$f" 2>/dev/null || echo "")
                    echo "- $(basename "$f" .md) (confidence: ${conf:-?})"
                fi
            done < <(find "${REPO_ROOT}/decisions" -type f -name "DEC-*.md" -print0 2>/dev/null)
        fi
    else
        echo "*No decisions touched this week.*"
    fi
    echo ""

    # ---------- Reflection themes ----------
    echo "## Reflection notes (shipped specs)"
    echo ""
    if [ "$SHIPPED_COUNT" -gt 0 ]; then
        for f in "${SHIPPED_FILES[@]}"; do
            [ -n "$f" ] || continue
            name=$(basename "$f" .md | sed -E 's/^(SPEC-[0-9]+).*/\1/')
            refl=$(awk '
                /^## Reflection/ { in_r = 1; next }
                /^## [A-Z]/ && in_r { in_r = 0 }
                in_r { print }
            ' "$f" | sed 's/^/  /' | head -n 12)
            if [ -n "$refl" ]; then
                echo "**${name}:**"
                echo ""
                echo "$refl"
                echo ""
            fi
        done
    else
        echo "*No shipped specs this week.*"
    fi
    echo ""

    # ---------- Flags ----------
    echo "## Flags"
    echo ""
    # Reuse daily's checks but aggregated across the week
    stalled=()
    if [ -d "$SPECS_DIR" ]; then
        while IFS= read -r -d '' f; do
            c=$(get_spec_cycle "$f")
            if [ "$c" = "build" ] || [ "$c" = "verify" ]; then
                if [ "$(uname)" = "Darwin" ]; then
                    age=$(( ( $(date +%s) - $(stat -f %m "$f") ) / 86400 ))
                else
                    age=$(( ( $(date +%s) - $(stat -c %Y "$f") ) / 86400 ))
                fi
                if [ "$age" -gt 7 ]; then
                    stalled+=("$(basename "$f" .md) (${c}, ${age}d)")
                fi
            fi
        done < <(find "$SPECS_DIR" -type f -name "SPEC-*.md" -not -path '*/done/*' -print0 2>/dev/null)
    fi
    if [ "${#stalled[@]}" -gt 0 ]; then
        echo "**Stalled specs:**"
        for s in "${stalled[@]}"; do echo "- ${s}"; done
    else
        echo "**Stalled specs:** none."
    fi
    echo ""

    # Missing-cost shipped specs
    missing_cost_shipped=()
    for f in "${SHIPPED_FILES[@]:-}"; do
        [ -n "$f" ] || continue
        sc=$(count_cost_sessions "$f")
        if [ "$sc" = "0" ]; then
            missing_cost_shipped+=("$(basename "$f" .md)")
        fi
    done
    if [ "${#missing_cost_shipped[@]}" -gt 0 ]; then
        echo "**Shipped without cost data (agent-discipline signal):**"
        for s in "${missing_cost_shipped[@]}"; do echo "- ${s}"; done
    else
        echo "**Shipped without cost data:** none."
    fi
    echo ""
    echo "---"
    echo "*Re-run with \`just report-weekly\` (or pass a date as the first arg).*"
} > "$OUT"

echo "${GREEN}✓${RESET} wrote ${OUT#$REPO_ROOT/}"
echo ""
cat "$OUT"
