#!/usr/bin/env bash
# scripts/wt.sh — create, list and remove per-session git worktrees.
#
#   wt.sh new NAME [--no-claude]   worktree + branch NAME from fresh origin/main, then start claude
#   wt.sh ls                       list managed worktrees: branch, dirty, merged, in use
#   wt.sh rm NAME [--force]        remove a worktree and its branch, if that is safe
#   wt.sh clean [--yes]            remove every managed worktree that is safe to remove
#
# Worktrees always live under <main checkout>/.claude/worktrees/, whichever worktree you run
# this from. Only those are ever listed or removed; the main checkout is never touched.
# NAME is the branch name; slashes become dashes in the directory name, so
# `feat/spec-131-typed-params` lives at .claude/worktrees/feat-spec-131-typed-params.

set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "${SCRIPT_DIR}/_lib.sh"

usage() {
    sed -n '3,7p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
    exit 1
}

# The main checkout is the parent of the shared .git directory.
COMMON_DIR="$(git rev-parse --path-format=absolute --git-common-dir)"
MAIN_ROOT="$(dirname "$COMMON_DIR")"
WT_ROOT="${MAIN_ROOT}/.claude/worktrees"

dir_for() { echo "${WT_ROOT}/${1//\//-}"; }

branch_of() { git -C "$1" symbolic-ref --quiet --short HEAD 2>/dev/null || echo "(detached)"; }

is_dirty() { [ -n "$(git -C "$1" status --porcelain 2>/dev/null)" ]; }

# A worktree is in use when any process (a claude session, a shell, an editor) has its
# current directory inside it. Removing it would pull the floor out from under that process.
in_use() {
    local dir="$1" cwds
    # lsof exits non-zero when some processes are unreadable; that must not read as "not in use".
    cwds="$(lsof -d cwd -Fn 2>/dev/null || true)"
    # Not grep -q: quitting early breaks the pipe, and pipefail would then report "not in use".
    printf '%s\n' "$cwds" | sed -n 's/^n//p' | grep -e "^${dir}\$" -e "^${dir}/" >/dev/null
}

# Echoes whether removing the branch would lose commits: "merged" (its tip is on origin/main),
# "squash-merged" (its tip is contained in the head of a PR GitHub reports as merged), or "no".
# A fresh branch with no commits counts as merged; the in-use and dirty checks protect a
# session that has just started.
merge_state() {
    local branch="$1"
    [ "$branch" = "(detached)" ] && { echo "no"; return; }
    if git merge-base --is-ancestor "$branch" origin/main 2>/dev/null; then
        echo "merged"; return
    fi
    # Squash merges leave no ancestry, so ask GitHub. Trust it only when every local commit is
    # inside the merged PR's head; commits added after the merge would otherwise be lost.
    if command -v gh >/dev/null 2>&1; then
        local oid
        for oid in $(gh pr list --head "$branch" --state merged --json headRefOid --jq '.[].headRefOid' 2>/dev/null); do
            git cat-file -e "${oid}^{commit}" 2>/dev/null || git fetch --quiet origin "$oid" 2>/dev/null || continue
            if git merge-base --is-ancestor "$branch" "$oid" 2>/dev/null; then
                echo "squash-merged"; return
            fi
        done
    fi
    echo "no"
}

managed_dirs() {
    git worktree list --porcelain | sed -n 's/^worktree //p' | while IFS= read -r d; do
        case "$d" in "${WT_ROOT}"/*) echo "$d" ;; esac
    done
}

# Finds a managed worktree by branch name or by the folder name `ls` prints.
resolve_dir() {
    local name="$1" d
    while IFS= read -r d; do
        if [ "$d" = "$(dir_for "$name")" ] || [ "${d#"${WT_ROOT}"/}" = "$name" ] ||
           [ "$(branch_of "$d")" = "$name" ]; then
            echo "$d"; return 0
        fi
    done < <(managed_dirs)
    return 1
}

cmd_new() {
    local name="" launch=1
    for a in "$@"; do
        case "$a" in --no-claude) launch=0 ;; -*) usage ;; *) name="$a" ;; esac
    done
    [ -n "$name" ] || usage
    local dir; dir="$(dir_for "$name")"

    if [ -d "$dir" ]; then
        info "Reusing existing worktree ${dir} (branch $(branch_of "$dir"))"
    else
        info "Fetching origin/main"
        git fetch --quiet origin main
        mkdir -p "$WT_ROOT"
        if git show-ref --verify --quiet "refs/heads/${name}"; then
            git worktree add --quiet "$dir" "$name"
            info "Checked out existing branch ${name}"
        else
            # --no-track: a branch cut from origin/main must never push to main by default.
            git worktree add --quiet --no-track -b "$name" "$dir" origin/main
        fi
        success "Created ${dir} on ${name} at $(git -C "$dir" log -1 --format='%h %s')"
    fi

    if [ "$launch" -eq 1 ] && [ -t 0 ] && command -v claude >/dev/null 2>&1; then
        cd "$dir" && exec claude
    fi
    echo "cd ${dir} && claude"
}

cmd_ls() {
    local any=0
    printf '%-44s %-40s %-6s %-14s %s\n' "WORKTREE" "BRANCH" "DIRTY" "MERGED" "IN USE"
    while IFS= read -r d; do
        any=1
        local b; b="$(branch_of "$d")"
        printf '%-44s %-40s %-6s %-14s %s\n' "${d#"${WT_ROOT}"/}" "$b" \
            "$(is_dirty "$d" && echo yes || echo no)" "$(merge_state "$b")" \
            "$(in_use "$d" && echo yes || echo no)"
    done < <(managed_dirs)
    [ "$any" -eq 1 ] || info "No worktrees under ${WT_ROOT}"
}

# Prints why a worktree is unsafe to remove, or nothing when it is safe.
removal_blocker() {
    local d="$1" b="$2" m="$3"
    case "$PWD/" in "$d"/*) echo "you are inside it"; return ;; esac
    in_use "$d" && { echo "a process is working inside it"; return; }
    is_dirty "$d" && { echo "it has uncommitted changes"; return; }
    case "$m" in
        merged|squash-merged) ;;
        *) echo "its branch has commits main does not have" ;;
    esac
}

remove_one() {
    local d="$1" b="$2" force="$3"
    if [ "$force" -eq 1 ]; then
        git worktree remove --force "$d"
    else
        git worktree remove "$d"
    fi
    if [ "$b" != "(detached)" ]; then
        git branch --quiet -D "$b"
    fi
    success "Removed ${d#"${WT_ROOT}"/} and branch ${b}"
}

cmd_rm() {
    local name="" force=0
    for a in "$@"; do
        case "$a" in --force) force=1 ;; -*) usage ;; *) name="$a" ;; esac
    done
    [ -n "$name" ] || usage
    local d; d="$(resolve_dir "$name")" || die "No managed worktree named ${name} (see: just wt-ls)"
    git fetch --quiet origin main
    local b m why; b="$(branch_of "$d")"; m="$(merge_state "$b")"
    why="$(removal_blocker "$d" "$b" "$m")"
    if [ -n "$why" ] && [ "$force" -eq 0 ]; then
        if [ "$m" = "no" ]; then
            warn "Commits on ${b} that origin/main does not have:"
            git log --oneline origin/main.."$b" | head -20 >&2
        fi
        case "$why" in
            "a process"*|"you are"*) die "Not removing ${name}: ${why}. Close that session or shell first." ;;
            *) die "Not removing ${name}: ${why}. Re-run with --force to remove it anyway." ;;
        esac
    fi
    # --force can discard work, but never a folder a live session is standing in.
    case "$PWD/" in "$d"/*) die "You are inside ${d}; cd out first." ;; esac
    in_use "$d" && die "A process is working inside ${d}; close that session or shell first."
    remove_one "$d" "$b" "$force"
}

cmd_clean() {
    local yes=0
    for a in "$@"; do case "$a" in --yes) yes=1 ;; *) usage ;; esac; done
    git fetch --quiet origin main
    local found=0
    while IFS= read -r d; do
        local b m why; b="$(branch_of "$d")"; m="$(merge_state "$b")"
        why="$(removal_blocker "$d" "$b" "$m")"
        if [ -n "$why" ]; then
            echo "${DIM}keep   ${d#"${WT_ROOT}"/}: ${why}${RESET}"
            continue
        fi
        found=1
        if [ "$yes" -eq 1 ]; then remove_one "$d" "$b" 0; else echo "remove ${d#"${WT_ROOT}"/} (${b}, ${m})"; fi
    done < <(managed_dirs)
    if [ "$found" -eq 1 ] && [ "$yes" -eq 0 ]; then
        info "Dry run. Re-run with --yes (just wt-clean --yes) to remove the lines marked remove."
    fi
}

sub="${1:-}"; shift || true
case "$sub" in
    new) cmd_new "$@" ;;
    ls) cmd_ls ;;
    rm) cmd_rm "$@" ;;
    clean) cmd_clean "$@" ;;
    *) usage ;;
esac
