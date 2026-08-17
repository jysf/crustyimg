# SPEC-119 — VERIFY RE-READ prompt

Cycle: **verify** (second pass, after a punch list). **Deliberately narrow.**

The first verify returned ⚠ PUNCH LIST with the headline *"the implementation is correct and I
could not break it."* It re-drove **all 11 ACs** against fixtures built outside this repo, three
independent negative controls, 96 byte-identity pairs against `main`, and a full three-leg matrix.
**None of that is reopened.** The punch list was documentation-only, and `d9f2d8c` touched exactly
three files: `docs/api-contract.md`, the spec, and DEC-093 — **no `src/`, no `tests/`** (confirmed).

**Your job is to close three items and one new factual claim. Nothing else.**

## Do NOT redo

- The ACs, the negative controls, the matrix, the byte-identity sweep, the CI legs.
- The cost arithmetic. Build **$51.2365** vs `$51.24` ✓ and verify **$30.9889** vs `$30.99` ✓ were
  both re-derived by the orchestrator.
- The DEC-092 → DEC-093 renumber. Already confirmed coherent by the first pass and again after
  the punch list.

**If you find yourself running `cargo test`, stop and re-read this section.** The only reason to
build is item 4 below, and only if you cannot settle it by reading.

## The four things to close

### 1. P1 — is the record now honest?

The Goal claimed "no shipped verb silently discards frames"; `responsive`, and `apply`/`build`
with a plain recipe, still do. The punch list amended `## Goal` to what was actually wired and
added a `## Known residual` subsection to Build Completion.

**Read both. Does a reader now learn the true scope without hunting?** The residual is filed as a
`[M]` on STAGE-046 — confirm it is cross-referenced, not re-filed.

### 2. P2 — are the two verb maps accurate?

The rewritten `docs/api-contract.md` paragraph lists **warns** and **silent** for the
animated-input warning, and says it is **not the identical set** the truncated-JPEG warning uses.
The punch list also corrected the older truncated-JPEG paragraph in passing, since SPEC-116 moved
`build`'s Decide arm out of its silent list.

**Both maps are now factual claims in a contract document. Check them against the source** — that
is the whole point of this pass. The first verify drove one map; the punch list asserts two.

### 3. ⚠ NEW CLAIM — `info` warns for one flag and not the other

The punch list states: **`info` warns for truncated-JPEG but NOT for animated-input**, because it
decodes via `Image::decode_path`/`from_bytes` directly, checks `is_truncated_jpeg()`, and never
calls `is_animated_input()`.

**Nobody has verified this.** It appeared during the punch list, not the verify. It is also
exactly the asymmetry the original defect was made of — one code path knowing something another
does not — so it deserves the same scrutiny.

**Settle it by reading `src/cli/report.rs`'s `run_info`**, and say whether it is (a) true and
correctly documented, (b) true but *itself a defect worth filing*, or (c) wrong. **Option (b) is
live**: if `info` reports on an image, staying silent about discarded frames while warning about
truncation is arguably its own inconsistency.

### 4. P3 — is the qualifier correct and complete?

The new text says `lint --max-warnings 0` holds for a directly-named file and for stdin, but not
for animated WebP in **directory-discovery mode**, because `webp` is absent from
`IMAGE_EXTENSIONS` — and that **GIF and APNG are unaffected** since both extensions are present.

Confirm the exemption claim by reading `src/source/mod.rs:105-113`. **If you want to drive it, one
`lint` run on a directory is enough** — do not rebuild the world for this.

## Also confirm

- **DEC-093's AVIF narrowing.** "PROVEN SAFE" now scoped to major brand `avis`; an `avif`-branded
  file carrying an embedded sequence is stated as unproven. Is that the precise form?
- **The `## Failing Tests` roster** no longer names the test that does not exist, and the swap is
  recorded under Deviations.
- **`cycle:` is still `verify`** — the punch list was told not to advance it. You advance it.

## When you finish, in this order

1. **Do NOT add a second verify cost entry.** This is a continuation of the same verify cycle. If
   this pass is material, fold its tokens into the existing verify entry and say so in the note;
   the first pass measured **$30.99 / 47,880,996 / 28.3 min**, and its commit `c920cb9` is local
   on a detached worktree, unpushed — the orchestrator applies the block on `main` at ship.
2. Run `just advance-cycle SPEC-119 ship`, and **CONFIRM it moved** (`git diff` shows the `cycle:`
   line change; it reports success even when it changes nothing).
3. Give the verdict: ✅ APPROVED / ⚠ PUNCH LIST / ❌ REJECTED — with items 1–4 each ruled on.

## Guardrails

- **Own git worktree**, `--detach` at the PR tip. `main` has moved repeatedly today.
- **Do not fix what you find.** Report it. A new defect (option 3b) gets filed, not patched.
- **Do not merge. Do not bump the version.**
- `git commit -s` (DCO). **A piped command reports the pipe's exit code.**
- **Budget: well under 50 exchanges.** This is a documentation re-read plus one source check. If
  it is growing, you have reopened something the first pass already closed.
