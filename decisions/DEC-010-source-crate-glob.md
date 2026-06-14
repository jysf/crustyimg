---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-010
  type: decision
  confidence: 0.85                   # honest: glob is the right tool; the
                                     # residual risk is cross-platform pattern
                                     # semantics (Windows separators), not the
                                     # choice of crate.
  audience:
    - developer
    - agent

agent:
  id: claude-opus-4-8
  session_id: null

# Decisions are repo-level, but it's useful to track which project
# caused them to be emitted.
project:
  id: PROJ-001
repo:
  id: crustyimg

created_at: 2026-06-14
supersedes: null
superseded_by: null

# Path globs this decision governs.
affected_scope:
  - src/source/**
  - Cargo.toml

tags:
  - dependencies
  - source
  - cli
---

# DEC-010: `glob` for glob-pattern source expansion; `std::fs` for directories

## Decision

The `Source` input abstraction (SPEC-004) adds exactly **one** new top-level
dependency: **`glob` 0.3** (pure-Rust, MIT/Apache-2.0), used only to expand a
glob *pattern* argument (e.g. `photos/*.jpg`) into matching paths. Directory
sources are enumerated with the **standard library** (`std::fs::read_dir`),
**non-recursively** (top-level entries only). `walkdir` — although pre-named in
`docs/architecture.md` as a candidate source crate — is **NOT** added in the
MVP: it earns its place only when recursive directory walking is actually
needed, which it is not for the non-recursive directory behavior decided here.

## Context

SPEC-004 must resolve a single CLI input argument into an ordered, deterministic
list of inputs across four shapes: a single file, a glob pattern, a directory,
and `-` (stdin). The constraint `no-new-top-level-deps-without-decision` requires
a DEC before any new crate lands in `Cargo.toml`. `docs/architecture.md` § "Crate
Choices" pre-names both `glob` 0.3 and `walkdir` 2 as source crates, but pre-naming
is not the same as a decision record — and adding *both* when only one is needed
violates the spirit of the constraint ("dependencies are forever; force
deliberation").

Two sub-questions:

1. **Glob matching** is genuinely hard to do correctly by hand: `*`, `?`,
   character classes, `**`, escaping, and not crossing certain boundaries. A
   hand-rolled matcher is the kind of subtle, security-relevant code we should
   not own. `glob` is the de-facto standard crate, pure-Rust, tiny, and stable.
2. **Directory enumeration** (non-recursive) is a few lines of
   `std::fs::read_dir` and needs no dependency. Pulling `walkdir` for a
   non-recursive listing is overkill (DEC-004's portability spirit: prefer
   `std` when it is sufficient).

Non-recursive directories are the right MVP default: predictable, fast, no
surprise descent into huge trees, and aligned with how most image-prep CLIs
treat a bare directory argument. The door is explicitly left open: a future
`--recursive` flag (its own spec/DEC) can add `walkdir` then, with the depth
and symlink-loop semantics that recursion actually requires.

## Alternatives Considered

- **Option A: add both `glob` and `walkdir` now (as architecture pre-names)**
  - What it is: use `glob` for patterns and `walkdir` for directory listing.
  - Why rejected: `walkdir`'s value is recursive traversal with loop/symlink
    handling; for a *non-recursive* directory listing it is unused weight. Adds
    a forever-dependency for no MVP behavior. Add it when recursion lands.

- **Option B: std-only — hand-roll glob matching too**
  - What it is: no new dependency at all; implement `*`/`?`/classes by hand.
  - Why rejected: correct, secure glob matching is subtle (and a parsing surface
    over untrusted-ish input). Owning it is more risk than a 1-file pure-Rust
    crate. Not worth saving one well-known dependency.

- **Option C (chosen): `glob` for patterns + `std::fs::read_dir` for dirs**
  - What it is: one new dep (`glob`) for the hard part; `std` for the easy part;
    no `walkdir`; directories non-recursive.
  - Why selected: minimal dependency surface, uses the right tool for the hard
    problem, keeps directory behavior simple/portable, and leaves recursion as a
    clean future addition (its own DEC adds `walkdir`).

## Consequences

- **Positive:** Smallest viable dependency add (one pure-Rust crate, no system
  deps — consistent with DEC-004). Glob correctness is delegated to a stable
  crate. Directory behavior is trivial std and easy to reason about for the
  symlink-traversal hardening (constraint `untrusted-input-hardening`).
- **Negative:** Directory sources are top-level only in the MVP; users who want
  recursion must wait for a `--recursive` flag (deliberate, documented).
  `glob`'s pattern semantics differ subtly across platforms (path separators on
  Windows); SPEC-004 must normalize/sort results deterministically and test it.
- **Neutral:** `walkdir` remains pre-named in architecture for when recursion
  arrives; this DEC narrows the MVP to `glob`-only without removing that future.

## Validation

Right if: SPEC-004 ships with `glob` as the only new dependency, directory
enumeration uses `std::fs::read_dir` with no `walkdir`, and the four input
shapes resolve deterministically and safely (symlink-escape entries skipped) on
all three CI OSes. Revisit if: a near-term spec needs recursive directory
walking — then add `walkdir` under its own DEC with depth + symlink-loop +
follow-symlinks policy, rather than retrofitting recursion into this code.

## References

- Related specs: SPEC-004 (Source input abstraction), SPEC-005 (Sink — consumes
  the inputs this yields), SPEC-002 (Image::from_bytes / from_reader — Source
  yields bytes/paths, Image decodes)
- Related decisions: DEC-002 (canonical Image the inputs load into), DEC-004
  (pure-Rust default; `glob` is pure-Rust), DEC-007 (typed errors for resolution
  failures)
- Constraints: `no-new-top-level-deps-without-decision`, `untrusted-input-hardening`
- External docs: https://docs.rs/glob , https://doc.rust-lang.org/std/fs/fn.read_dir.html
- Architecture: `docs/architecture.md` § "Crate Choices" (pre-names glob/walkdir)
