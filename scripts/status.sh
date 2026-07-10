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

# --- JSON mode: emit the same declared state as a single JSON object ---------
# All gathering reuses the same helpers as the human report below (project_status,
# get_spec_cycle, find_all_specs, is_grandfathered_cost, spec_missing_cost_cycles),
# so the two can't drift. Ruby is used ONLY as a safe JSON encoder, fed a simple
# tab-delimited stream — no logic lives in it.
emit_json() {
    command -v ruby >/dev/null 2>&1 \
        || die "status --json needs \`ruby\` (stdlib json). Use \`just status\` for the human report, or install ruby."
    {
        printf 'meta\tvariant\t%s\n' "$VARIANT"
        printf 'meta\tactive_project\t%s\n' "$ACTIVE_PROJECT"

        # All projects: id + declared status.
        for p in "${REPO_ROOT}"/projects/PROJ-*; do
            [ -d "$p" ] || continue
            pstatus="unknown"
            [ -f "$p/brief.md" ] && pstatus=$(project_status "$p/brief.md")
            printf 'project\t%s\t%s\n' "$(basename "$p")" "${pstatus:-unknown}"
        done

        # Active project: stages (id + status).
        if [ -d "${ACTIVE_PROJECT_DIR}/stages" ]; then
            for s in "${ACTIVE_PROJECT_DIR}/stages"/STAGE-*.md; do
                [ -f "$s" ] || continue
                sstatus=$(awk '/^---$/{f=!f; next} f && /^[[:space:]]+status:/{print $2; exit}' "$s" 2>/dev/null || echo "unknown")
                printf 'stage\t%s\t%s\n' "$(basename "$s" .md)" "${sstatus:-unknown}"
            done
        fi

        # Active project: specs by cycle (id + archived flag); done/ counts as ship.
        if [ -d "${ACTIVE_PROJECT_DIR}/specs" ]; then
            for cycle in frame design build verify ship; do
                for f in "${ACTIVE_PROJECT_DIR}/specs"/SPEC-*.md; do
                    [ -f "$f" ] || continue
                    sc=$(get_spec_cycle "$f" 2>/dev/null || echo "")
                    [ "$sc" = "$cycle" ] && printf 'spec\t%s\t%s\t0\n' "$cycle" "$(basename "$f" .md)"
                done
            done
            if [ -d "${ACTIVE_PROJECT_DIR}/specs/done" ]; then
                for f in "${ACTIVE_PROJECT_DIR}/specs/done"/SPEC-*.md; do
                    [ -f "$f" ] || continue
                    printf 'spec\tship\t%s\t1\n' "$(basename "$f" .md)"
                done
            fi
        fi

        # Low-confidence decisions (< 0.7): id + confidence.
        if [ -d "${REPO_ROOT}/decisions" ]; then
            for d in "${REPO_ROOT}/decisions"/DEC-*.md; do
                [ -f "$d" ] || continue
                conf=$(awk '/^---$/{f=!f; next} f && /^[[:space:]]+confidence:/{print $2; exit}' "$d" 2>/dev/null || echo "")
                [ -n "$conf" ] || continue
                low=$(awk -v c="$conf" 'BEGIN { print (c + 0 < 0.7) ? "1" : "0" }')
                [ "$low" = "1" ] && printf 'lowconf\t%s\t%s\n' "$(basename "$d" .md)" "$conf"
            done
        fi

        # Possibly-stale specs (build/verify, file mtime > 7 days): id + cycle + age.
        if [ -d "${ACTIVE_PROJECT_DIR}/specs" ]; then
            for f in "${ACTIVE_PROJECT_DIR}/specs"/SPEC-*.md; do
                [ -f "$f" ] || continue
                cyc=$(get_spec_cycle "$f" 2>/dev/null || echo "")
                if [ "$cyc" = "build" ] || [ "$cyc" = "verify" ]; then
                    if [ "$(uname)" = "Darwin" ]; then
                        age=$(( ( $(date +%s) - $(stat -f %m "$f") ) / 86400 ))
                    else
                        age=$(( ( $(date +%s) - $(stat -c %Y "$f") ) / 86400 ))
                    fi
                    [ "$age" -gt 7 ] && printf 'stale\t%s\t%s\t%s\n' "$(basename "$f" .md)" "$cyc" "$age"
                fi
            done
        fi

        # Shipped specs missing build/verify cost (same rule as `just cost-audit`).
        for f in $(find_all_specs "$ACTIVE_PROJECT_DIR"); do
            case "$f" in *-timeline.md) continue ;; esac
            name=$(basename "$f" .md)
            shipped=0
            case "$f" in
                */specs/done/*) shipped=1 ;;
                *) [ "$(get_spec_cycle "$f")" = "ship" ] && shipped=1 ;;
            esac
            [ "$shipped" = "1" ] || continue
            is_grandfathered_cost "$name" && continue
            missing=$(spec_missing_cost_cycles "$f")
            [ -n "$missing" ] && printf 'missingcost\t%s\t%s\n' "$name" "$missing"
        done

        # Summary counts.
        printf 'summary\ttotal_specs\t%s\n' "$(find "${ACTIVE_PROJECT_DIR}/specs" -name "SPEC-*.md" 2>/dev/null | wc -l | tr -d ' ')"
        printf 'summary\tshipped_specs\t%s\n' "$(find "${ACTIVE_PROJECT_DIR}/specs/done" -name "SPEC-*.md" 2>/dev/null | wc -l | tr -d ' ')"
        printf 'summary\ttotal_decisions\t%s\n' "$(find "${REPO_ROOT}/decisions" -name "DEC-*.md" 2>/dev/null | wc -l | tr -d ' ')"
    } | ruby -rjson -e '
        data = {
          "schema" => 1,
          "variant" => nil,
          "active_project" => nil,
          "projects" => [],
          "active" => {
            "stages" => [],
            "specs_by_cycle" => {"frame"=>[], "design"=>[], "build"=>[], "verify"=>[], "ship"=>[]},
            "low_confidence_decisions" => [],
            "stale_specs" => [],
            "specs_missing_cost" => [],
          },
          "summary" => {},
        }
        STDIN.each_line do |line|
          p = line.chomp.split("\t")
          case p[0]
          when "meta"        then data[p[1]] = p[2]
          when "project"     then data["projects"] << {"id"=>p[1], "status"=>p[2]}
          when "stage"       then data["active"]["stages"] << {"id"=>p[1], "status"=>p[2]}
          when "spec"        then (data["active"]["specs_by_cycle"][p[1]] ||= []) << {"id"=>p[2], "archived"=>(p[3]=="1")}
          when "lowconf"     then data["active"]["low_confidence_decisions"] << {"id"=>p[1], "confidence"=>p[2].to_f}
          when "stale"       then data["active"]["stale_specs"] << {"id"=>p[1], "cycle"=>p[2], "age_days"=>p[3].to_i}
          when "missingcost" then data["active"]["specs_missing_cost"] << {"id"=>p[1], "missing"=>p[2].split(/\s+/)}
          when "summary"     then data["summary"][p[1]] = p[2].to_i
          end
        end
        puts JSON.pretty_generate(data)
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
