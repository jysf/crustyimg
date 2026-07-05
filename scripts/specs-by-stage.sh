#!/usr/bin/env bash
# scripts/specs-by-stage.sh — a flat, every-spec ledger grouped by
# stage. Complements `status` (current state), `backlog` (what's next),
# and `roadmap` (stage-grained counts) with the one view none of them
# give: every spec, under its stage, with ship date and complexity.
#
# Unlike roadmap/backlog (which default to the active project), this
# defaults to ALL projects — it's a historical ledger.
#
# Usage:
#   just specs-by-stage              # all projects (default)
#   just specs-by-stage --active     # active/current project only
#   just specs-by-stage PROJ-002     # a specific project (id or full dir name)
#
# Everything is read from authoritative front-matter: a spec's stage
# (project.stage), cycle (task.cycle), complexity (task.complexity),
# and — for archived specs under specs/done/ — the ship date (the
# recorded_at of the `ship` cost session, falling back to the stage's
# shipped_at). Output is human-readable text on stdout.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_lib.sh"

require_initialized

# ---------------------------------------------------------------------
# Spec front-matter readers (kept local to this script).
# ---------------------------------------------------------------------

# project.stage — the STAGE-NNN this spec belongs to.
get_spec_stage() {
    awk '
        /^---$/ { f = !f; next }
        !f { exit }
        /^project:/ { p = 1; next }
        p && /^[a-zA-Z_]/ { p = 0 }
        p && /^[[:space:]]+stage:/ { print $2; exit }
    ' "$1"
}

# task.complexity — S | M | L.
get_spec_complexity() {
    awk '
        /^---$/ { f = !f; next }
        !f { exit }
        /^task:/ { t = 1; next }
        t && /^[a-zA-Z_]/ { t = 0 }
        t && /^[[:space:]]+complexity:/ { print $2; exit }
    ' "$1"
}

# patch.cycle / patch.complexity — patches carry a `patch:` block (DEC-043),
# not a `task:` block, so they need their own readers.
get_patch_cycle() {
    awk '
        /^---$/ { f = !f; next }
        !f { exit }
        /^patch:/ { p = 1; next }
        p && /^[a-zA-Z_]/ { p = 0 }
        p && /^[[:space:]]+cycle:/ { print $2; exit }
    ' "$1"
}
get_patch_complexity() {
    awk '
        /^---$/ { f = !f; next }
        !f { exit }
        /^patch:/ { p = 1; next }
        p && /^[a-zA-Z_]/ { p = 0 }
        p && /^[[:space:]]+complexity:/ { print $2; exit }
    ' "$1"
}

# recorded_at of the `ship` cost session, if any. Indents match the
# rest of the cost-session readers in _lib.sh.
get_spec_ship_date() {
    awk '
        /^---$/ { fm = !fm; next }
        !fm { next }
        /^cost:/ { c = 1; next }
        c && /^[a-zA-Z_]/ { c = 0 }
        c && /^  sessions:/ { s = 1; next }
        c && s && /^  [a-zA-Z_]/ { s = 0 }
        s && /^    - cycle:/ { cyc = $3 }
        s && /^      recorded_at:/ { if (cyc == "ship") { print $2; exit } }
    ' "$1"
}

# Compact token count for display: 201141 -> 201k, 950 -> 950.
fmt_tok() {
    if [ "$1" -ge 1000 ]; then echo "$(( $1 / 1000 ))k"; else echo "$1"; fi
}

# ---------------------------------------------------------------------
# Parse scope flags.
# ---------------------------------------------------------------------

SCOPE="all"
TARGET=""
for arg in "$@"; do
    case "$arg" in
        --active|--current) SCOPE="active" ;;
        --all)              SCOPE="all" ;;
        --*)                die "Unknown flag: $arg (use --active, --all, or a PROJ-NNN id)" ;;
        *)                  SCOPE="one"; TARGET="$arg" ;;
    esac
done

PROJECTS=()
if [ "$SCOPE" = "active" ]; then
    PROJECTS+=("$(get_active_project)")
elif [ "$SCOPE" = "one" ]; then
    dir=$(find "${REPO_ROOT}/projects" -maxdepth 1 -type d -name "${TARGET}*" 2>/dev/null | sort | head -n1)
    [ -n "$dir" ] || die "No project matching '${TARGET}' under projects/."
    PROJECTS+=("$(basename "$dir")")
else
    while IFS= read -r d; do
        [ -n "$d" ] && PROJECTS+=("$(basename "$d")")
    done < <(find "${REPO_ROOT}/projects" -maxdepth 1 -type d -name 'PROJ-*' 2>/dev/null | sort)
fi

[ "${#PROJECTS[@]}" -gt 0 ] || die "No projects found under projects/."

# ---------------------------------------------------------------------
# Render.
# ---------------------------------------------------------------------

SHIPPED=0
INFLIGHT=0
NOTWRITTEN=0
STAGES=0
PATCHES=0
GRAND_USD="0.00"
GRAND_TOK=0

# Patches (DEC-043) have no stage, so they get their own per-project section
# — the flat ledger should still account for every unit of work + its cost.
print_patches() {
    project_dir="$1"
    local rows="" PATCH_USD="0.00" PATCH_TOK=0 any=0
    while IFS= read -r pf; do
        [ -f "$pf" ] || continue
        pid=$(basename "$pf" | sed -E 's/^(PATCH-[0-9]+).*/\1/')
        cyc=$(get_patch_cycle "$pf"); [ -n "$cyc" ] || cyc="?"
        cx=$(get_patch_complexity "$pf"); [ -n "$cx" ] || cx="?"
        u=$(sum_cost_usd_for_spec "$pf"); t=$(sum_cost_tokens_for_spec "$pf")
        PATCH_USD=$(awk -v a="$PATCH_USD" -v b="$u" 'BEGIN{printf "%.2f", a+b}')
        PATCH_TOK=$((PATCH_TOK + t))
        GRAND_USD=$(awk -v a="$GRAND_USD" -v b="$u" 'BEGIN{printf "%.2f", a+b}')
        GRAND_TOK=$((GRAND_TOK + t))
        if [ "$u" = "0.00" ] && [ "$t" -eq 0 ]; then costcol="—"; else costcol="\$${u}  $(fmt_tok "$t")"; fi
        # A patch is "shipped" when it lives under patches/done/ (or cycle: ship).
        case "$pf" in
            */done/*)
                sdate=$(get_spec_ship_date "$pf"); [ -n "$sdate" ] || sdate="—"
                rows+=$(printf "    %-10s  ${GREEN}%-8s${RESET}  %-12s  %-3s  %s\n" "$pid" "shipped" "$sdate" "$cx" "$costcol")$'\n'
                SHIPPED=$((SHIPPED + 1)) ;;
            *)
                rows+=$(printf "    %-10s  %-8s  %-12s  %-3s  %s\n" "$pid" "$cyc" "—" "$cx" "$costcol")$'\n'
                INFLIGHT=$((INFLIGHT + 1)) ;;
        esac
        PATCHES=$((PATCHES + 1)); any=1
    done < <(find "${project_dir}/patches" -type f -name 'PATCH-*.md' -not -path '*/prompts/*' 2>/dev/null | sort)

    [ "$any" = 1 ] || return 0
    printf "  ${BOLD}Patches${RESET}  ${DIM}[no stage — DEC-043]${RESET}\n"
    printf '%s' "$rows"
    if [ "$PATCH_TOK" -gt 0 ] || [ "$PATCH_USD" != "0.00" ]; then
        printf "    ${DIM}patch cost: \$%s · %s${RESET}\n" "$PATCH_USD" "$(fmt_tok "$PATCH_TOK")"
    fi
}

print_stage() {
    local STAGE_USD="0.00" STAGE_TOK=0
    project_dir="$1"
    stage_file="$2"
    stage_base=$(basename "$stage_file" .md)
    stage_id=$(printf '%s' "$stage_base" | sed -E 's/^(STAGE-[0-9]+).*/\1/')
    status=$(get_stage_status "$stage_file"); [ -n "$status" ] || status="?"
    shipped=$(get_stage_shipped_at "$stage_file")
    if [ -n "$shipped" ]; then
        printf "  ${BOLD}%s${RESET}  ${DIM}[%s · shipped %s]${RESET}\n" "$stage_base" "$status" "$shipped"
    else
        printf "  ${BOLD}%s${RESET}  ${DIM}[%s]${RESET}\n" "$stage_base" "$status"
    fi

    any=0
    while IFS= read -r sf; do
        [ -f "$sf" ] || continue
        case "$sf" in *-timeline.md) continue ;; esac   # skip timeline artifacts
        sstage=$(get_spec_stage "$sf")
        [ "$sstage" = "$stage_id" ] || continue
        sid=$(basename "$sf" | sed -E 's/^(SPEC-[0-9]+).*/\1/')
        cyc=$(get_spec_cycle "$sf"); [ -n "$cyc" ] || cyc="?"
        cx=$(get_spec_complexity "$sf"); [ -n "$cx" ] || cx="?"
        u=$(sum_cost_usd_for_spec "$sf"); t=$(sum_cost_tokens_for_spec "$sf")
        STAGE_USD=$(awk -v a="$STAGE_USD" -v b="$u" 'BEGIN{printf "%.2f", a+b}')
        STAGE_TOK=$((STAGE_TOK + t))
        GRAND_USD=$(awk -v a="$GRAND_USD" -v b="$u" 'BEGIN{printf "%.2f", a+b}')
        GRAND_TOK=$((GRAND_TOK + t))
        if [ "$u" = "0.00" ] && [ "$t" -eq 0 ]; then costcol="—"; else costcol="\$${u}  $(fmt_tok "$t")"; fi
        case "$sf" in
            */done/*)
                sdate=$(get_spec_ship_date "$sf")
                [ -n "$sdate" ] || sdate="$shipped"
                [ -n "$sdate" ] || sdate="—"
                printf "    %-10s  ${GREEN}%-8s${RESET}  %-12s  %-3s  %s\n" "$sid" "shipped" "$sdate" "$cx" "$costcol"
                SHIPPED=$((SHIPPED + 1)) ;;
            *)
                printf "    %-10s  %-8s  %-12s  %-3s  %s\n" "$sid" "$cyc" "—" "$cx" "$costcol"
                INFLIGHT=$((INFLIGHT + 1)) ;;
        esac
        any=1
    done < <(find_all_specs "$project_dir" | sort)

    if [ "$STAGE_TOK" -gt 0 ] || [ "$STAGE_USD" != "0.00" ]; then
        printf "    ${DIM}stage cost: \$%s · %s${RESET}\n" "$STAGE_USD" "$(fmt_tok "$STAGE_TOK")"
    fi

    # Un-promoted "(not yet written)" backlog bullets in this stage.
    notwritten=$(grep -cE '^- \[[ x~?]\] \(not yet written\)' "$stage_file" 2>/dev/null || true)
    notwritten=${notwritten:-0}
    if [ "$notwritten" -gt 0 ]; then
        printf "    ${DIM}+ %s not yet written${RESET}\n" "$notwritten"
        NOTWRITTEN=$((NOTWRITTEN + notwritten))
    fi
    if [ "$any" = 0 ] && [ "$notwritten" -eq 0 ]; then
        printf "    ${DIM}(no specs)${RESET}\n"
    fi
}

case "$SCOPE" in
    all)    scope_label="all projects" ;;
    active) scope_label="active project (${PROJECTS[0]})" ;;
    one)    scope_label="${PROJECTS[0]}" ;;
esac

printf "${BOLD}Specs by stage — %s${RESET}\n" "$scope_label"
printf "${DIM}columns: spec · status · ship date · complexity · cost (usd · tokens)${RESET}\n"

for proj in "${PROJECTS[@]}"; do
    project_dir="${REPO_ROOT}/projects/${proj}"
    [ -d "$project_dir" ] || continue
    printf "\n${BLUE}%s${RESET}\n" "$proj"
    found_stage=0
    while IFS= read -r stage_file; do
        [ -f "$stage_file" ] || continue
        STAGES=$((STAGES + 1)); found_stage=1
        print_stage "$project_dir" "$stage_file"
    done < <(find "${project_dir}/stages" -maxdepth 1 -type f -name 'STAGE-*.md' 2>/dev/null | sort)
    if [ "$found_stage" = 0 ]; then
        printf "  ${DIM}(no stages)${RESET}\n"
    fi
    print_patches "$project_dir"
done

printf "\n${BOLD}Totals:${RESET} %d shipped · %d in flight · %d not yet written  ${DIM}(%d stage(s), %d patch(es), %d project(s))${RESET}\n" \
    "$SHIPPED" "$INFLIGHT" "$NOTWRITTEN" "$STAGES" "$PATCHES" "${#PROJECTS[@]}"
printf "${BOLD}Recorded cost:${RESET} \$%s · %s tokens  ${DIM}(only cycles with real numbers — see just cost-audit)${RESET}\n" \
    "$GRAND_USD" "$GRAND_TOK"
