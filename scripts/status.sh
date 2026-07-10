#!/usr/bin/env bash
# scripts/status.sh — print repo state report.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_lib.sh"

require_initialized

# Output mode: human (default) or machine-readable JSON (`--json`), for tools
# that consume declared state (a portfolio / standup tracker). The JSON path is
# the only one that needs `ruby`; the human report stays pure bash.
MODE="human"
for arg in "$@"; do
    case "$arg" in
        --json) MODE="json" ;;
        -h|--help)
            echo "usage: just status [--json]"
            echo "  (no flag)  human-readable repo status report"
            echo "  --json     the same state as JSON on stdout (needs ruby)"
            exit 0
            ;;
        *) die "status: unknown argument '$arg' (try --json or --help)" ;;
    esac
done

VARIANT=$(get_variant)
ACTIVE_PROJECT=$(get_active_project)
ACTIVE_PROJECT_DIR="${REPO_ROOT}/projects/${ACTIVE_PROJECT}"

# --- JSON mode: emit repo state as one JSON object (spec-driven-template shape)
# Envelope: { schema_version, command, generated_at, data:{ variant,
# active_project, specs[], missing_cost_specs[], summary } }. Each spec carries
# the ContextCore-dotted keys task.id / task.cycle / cost.tokens_total /
# cost.estimated_usd plus `shipped` and its `missing_cost` cycles.
#
# All gathering reuses the same helpers as the human report and `just cost-audit`
# (get_spec_cycle, find_all_specs, is_grandfathered_cost, spec_missing_cost_cycles),
# so they can't drift. Ruby is used ONLY as a safe JSON encoder over a
# tab-delimited stream — no logic lives in it. Output is compact (one line), and
# cost.estimated_usd keeps 2 decimals as a raw number via a sentinel.
emit_json() {
    command -v ruby >/dev/null 2>&1 \
        || die "status --json needs \`ruby\` (stdlib json). Use \`just status\` for the human report, or install ruby."

    {
        printf 'meta\tvariant\t%s\n' "$VARIANT"
        printf 'meta\tactive_project\t%s\n' "$ACTIVE_PROJECT"
        printf 'meta\tgenerated_at\t%s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

        # One record per spec (timelines excluded). task.id / task.cycle /
        # cost.totals are read in a single awk pass; `shipped` and `missing_cost`
        # reuse the cost-audit rule (shipped == archived or cycle==ship; missing
        # only counts for shipped, non-grandfathered specs).
        total=0
        shipped_count=0
        while IFS= read -r f; do
            [ -n "$f" ] || continue
            case "$f" in *-timeline.md) continue ;; esac
            total=$((total + 1))
            name=$(basename "$f" .md)

            # id \t cycle \t totals.tokens_total \t totals.estimated_usd
            fields=$(awk '
                /^---$/ { fm = !fm; next }
                !fm { next }
                /^task:/ { in_task = 1; next }
                in_task && /^[a-zA-Z_]/ { in_task = 0 }
                in_task && /^  id:/    { id = $2 }
                in_task && /^  cycle:/ { cyc = $2 }
                /^cost:/ { in_cost = 1; next }
                in_cost && /^[a-zA-Z_]/ { in_cost = 0 }
                in_cost && /^  totals:/ { in_tot = 1; next }
                in_cost && in_tot && /^  [a-zA-Z_]/ { in_tot = 0 }
                in_tot && /^    tokens_total:/  { tok = $2 }
                in_tot && /^    estimated_usd:/ { usd = $2 }
                END {
                    if (tok !~ /^[0-9]+$/) tok = 0
                    if (usd !~ /^[0-9]+(\.[0-9]+)?$/) usd = 0
                    printf "%s\t%s\t%s\t%.2f", id, cyc, tok, usd + 0
                }
            ' "$f")
            IFS=$'\t' read -r id cyc tok usd <<EOF
$fields
EOF
            [ -n "$id" ] || id=$(printf '%s' "$name" | sed -E 's/^(SPEC-[0-9]+).*/\1/')

            shipped=0
            case "$f" in
                */specs/done/*) shipped=1 ;;
                *) [ "$cyc" = "ship" ] && shipped=1 ;;
            esac
            [ "$shipped" = "1" ] && shipped_count=$((shipped_count + 1))

            missing=""
            if [ "$shipped" = "1" ] && ! is_grandfathered_cost "$name"; then
                missing=$(spec_missing_cost_cycles "$f")
            fi

            printf 'spec\t%s\t%s\t%s\t%s\t%s\t%s\n' "$id" "$cyc" "$shipped" "$tok" "$usd" "$missing"
        done < <(find_all_specs "$ACTIVE_PROJECT_DIR")

        printf 'summary\ttotal_specs\t%s\n' "$total"
        printf 'summary\tshipped\t%s\n' "$shipped_count"
        printf 'summary\tdecisions\t%s\n' "$(find "${REPO_ROOT}/decisions" -name "DEC-*.md" 2>/dev/null | wc -l | tr -d ' ')"
    } | ruby -rjson -e '
        env = {
          "schema_version" => 1,
          "command"        => "status",
          "generated_at"   => nil,
          "data" => {
            "variant"            => nil,
            "active_project"     => nil,
            "specs"              => [],
            "missing_cost_specs" => [],
            "summary"            => {},
          },
        }
        d = env["data"]
        STDIN.each_line do |line|
          p = line.chomp("\n").split("\t", -1)
          case p[0]
          when "meta"
            case p[1]
            when "variant"        then d["variant"] = p[2]
            when "active_project" then d["active_project"] = p[2]
            when "generated_at"   then env["generated_at"] = p[2]
            end
          when "spec"
            id, cyc, shipped, tok, usd, missing = p[1], p[2], p[3], p[4], p[5], p[6]
            d["specs"] << {
              "task.id"            => id,
              "task.cycle"         => cyc,
              "shipped"            => (shipped == "1"),
              "cost.tokens_total"  => tok.to_i,
              # Keep 2 decimals as a raw JSON number (unquoted below).
              "cost.estimated_usd" => "@@USD:#{usd}@@",
              "missing_cost"       => (missing.nil? || missing.strip.empty? ? [] : missing.split(/\s+/)),
            }
          when "summary"
            d["summary"][p[1]] = p[2].to_i
          end
        end
        d["missing_cost_specs"] = d["specs"].reject { |s| s["missing_cost"].empty? }.map { |s| s["task.id"] }
        json = JSON.generate(env).gsub(/"@@USD:([0-9]+(?:\.[0-9]+)?)@@"/, "\\1")
        puts json
    '
}

if [ "$MODE" = "json" ]; then
    emit_json
    exit 0
fi

echo "${BOLD}Repo status${RESET}"
echo ""
echo "  Variant:         ${VARIANT}"
echo "  Active project:  ${ACTIVE_PROJECT}"
echo ""

# --- All projects ---
echo "${BOLD}All projects${RESET}"
for p in "${REPO_ROOT}"/projects/PROJ-*; do
    [ -d "$p" ] || continue
    pname=$(basename "$p")
    brief="${p}/brief.md"
    status="unknown"
    if [ -f "$brief" ]; then
        # Grep for "status:" nested under "project:" in the front-matter
        status=$(awk '
            /^---$/ { f = !f; next }
            f && /^project:/ { inproj = 1; next }
            f && inproj && /^[a-zA-Z_]+:/ { inproj = 0 }
            f && inproj && /^[[:space:]]+status:/ { print $2; exit }
        ' "$brief" 2>/dev/null || echo "unknown")
    fi
    marker=" "
    if [ "$pname" = "$ACTIVE_PROJECT" ]; then marker="${GREEN}*${RESET}"; fi
    printf "  %s %-40s  status: %s\n" "$marker" "$pname" "$status"
done
echo ""

# --- Active project: stages ---
echo "${BOLD}Stages in ${ACTIVE_PROJECT}${RESET}"
stages_dir="${ACTIVE_PROJECT_DIR}/stages"
if [ -d "$stages_dir" ]; then
    for s in "${stages_dir}"/STAGE-*.md; do
        [ -f "$s" ] || continue
        sname=$(basename "$s" .md)
        pstatus=$(awk '/^---$/{f=!f; next} f && /^[[:space:]]+status:/{print $2; exit}' "$s" 2>/dev/null || echo "unknown")
        printf "  %-44s  status: %s\n" "$sname" "$pstatus"
    done
else
    echo "  ${DIM}(no stages dir yet)${RESET}"
fi
echo ""

# --- Active project: specs by cycle ---
echo "${BOLD}Specs in ${ACTIVE_PROJECT} by cycle${RESET}"
specs_dir="${ACTIVE_PROJECT_DIR}/specs"
if [ -d "$specs_dir" ]; then
    for cycle in frame design build verify ship; do
        count=0
        names=""
        for f in "${specs_dir}"/SPEC-*.md; do
            [ -f "$f" ] || continue
            spec_cycle=$(awk '/^---$/{f=!f; next} f && /^[[:space:]]+cycle:/{print $2; exit}' "$f" 2>/dev/null || echo "")
            if [ "$spec_cycle" = "$cycle" ]; then
                count=$((count + 1))
                names="${names}    - $(basename "$f" .md)\n"
            fi
        done
        # Also count done/ as ship
        if [ "$cycle" = "ship" ] && [ -d "${specs_dir}/done" ]; then
            for f in "${specs_dir}/done"/SPEC-*.md; do
                [ -f "$f" ] || continue
                count=$((count + 1))
                names="${names}    - $(basename "$f" .md) ${DIM}(archived)${RESET}\n"
            done
        fi
        printf "  ${BOLD}%-8s${RESET} (%d)\n" "$cycle" "$count"
        if [ -n "$names" ]; then
            printf "%b" "$names"
        fi
    done
else
    echo "  ${DIM}(no specs yet)${RESET}"
fi
echo ""

# --- Low-confidence decisions ---
echo "${BOLD}Low-confidence decisions (< 0.7)${RESET}"
decisions_dir="${REPO_ROOT}/decisions"
found_any=false
if [ -d "$decisions_dir" ]; then
    for d in "${decisions_dir}"/DEC-*.md; do
        [ -f "$d" ] || continue
        conf=$(awk '/^---$/{f=!f; next} f && /^[[:space:]]+confidence:/{print $2; exit}' "$d" 2>/dev/null || echo "")
        if [ -n "$conf" ]; then
            # Use awk for float comparison (portable)
            low=$(awk -v c="$conf" 'BEGIN { print (c + 0 < 0.7) ? "1" : "0" }')
            if [ "$low" = "1" ]; then
                printf "  %-42s  confidence: %s\n" "$(basename "$d" .md)" "$conf"
                found_any=true
            fi
        fi
    done
fi
if [ "$found_any" = "false" ]; then
    echo "  ${DIM}(none — or no decisions yet)${RESET}"
fi
echo ""

# --- Stale specs (no commits on their branch in 7 days, approximate) ---
echo "${BOLD}Possibly stale specs${RESET}"
echo "  ${DIM}(heuristic: specs in build/verify with file mtime > 7 days)${RESET}"
found_stale=false
if [ -d "$specs_dir" ]; then
    for f in "${specs_dir}"/SPEC-*.md; do
        [ -f "$f" ] || continue
        cycle=$(awk '/^---$/{fm=!fm; next} fm && /^[[:space:]]+cycle:/{print $2; exit}' "$f" 2>/dev/null || echo "")
        if [ "$cycle" = "build" ] || [ "$cycle" = "verify" ]; then
            # Age in days (portable across macOS and Linux).
            if [ "$(uname)" = "Darwin" ]; then
                age_days=$(( ( $(date +%s) - $(stat -f %m "$f") ) / 86400 ))
            else
                age_days=$(( ( $(date +%s) - $(stat -c %Y "$f") ) / 86400 ))
            fi
            if [ "$age_days" -gt 7 ]; then
                printf "  %-40s  cycle: %-8s  age: %d days\n" "$(basename "$f" .md)" "$cycle" "$age_days"
                found_stale=true
            fi
        fi
    done
fi
if [ "$found_stale" = "false" ]; then
    echo "  ${DIM}(none)${RESET}"
fi
echo ""

# --- Specs missing cost data (shipped, non-grandfathered) ---
echo "${BOLD}Specs missing cost data${RESET}"
echo "  ${DIM}(shipped specs whose build/verify cycles lack tokens_total — run just cost-audit)${RESET}"
found_missing_cost=false
for f in $(find_all_specs "$ACTIVE_PROJECT_DIR"); do
    case "$f" in *-timeline.md) continue ;; esac
    name=$(basename "$f" .md)
    shipped=0
    case "$f" in
        */specs/done/*) shipped=1 ;;
        *) if [ "$(get_spec_cycle "$f")" = "ship" ]; then shipped=1; fi ;;
    esac
    [ "$shipped" = "1" ] || continue
    if is_grandfathered_cost "$name"; then continue; fi
    missing=$(spec_missing_cost_cycles "$f")
    if [ -n "$missing" ]; then
        printf "  %-44s  missing: %s\n" "$name" "$missing"
        found_missing_cost=true
    fi
done
if [ "$found_missing_cost" = "false" ]; then
    echo "  ${DIM}(none)${RESET}"
fi
echo ""

# --- Summary counts ---
total_specs=$(find "${ACTIVE_PROJECT_DIR}/specs" -name "SPEC-*.md" 2>/dev/null | wc -l | tr -d ' ')
shipped_specs=$(find "${ACTIVE_PROJECT_DIR}/specs/done" -name "SPEC-*.md" 2>/dev/null | wc -l | tr -d ' ')
total_decisions=$(find "$decisions_dir" -name "DEC-*.md" 2>/dev/null | wc -l | tr -d ' ')
echo "${BOLD}Summary${RESET}"
echo "  Total specs in ${ACTIVE_PROJECT}:     ${total_specs}"
echo "  Shipped (archived):                   ${shipped_specs}"
echo "  Total decisions (across all projects): ${total_decisions}"
