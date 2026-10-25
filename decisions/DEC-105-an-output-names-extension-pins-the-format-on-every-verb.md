---
# Maps to ContextCore insight.* semantic conventions.

insight:
  id: DEC-105
  type: decision
  confidence: 0.9
  audience:
    - developer
    - agent

agent:
  id: claude-opus-5-5
  session_id: null

project:
  id: PROJ-011
repo:
  id: crustyimg

created_at: 2026-10-25
# Extends DEC-087; does not replace it. DEC-087's rule (a `build` terminal-`optimize`
# target's literal-extension template is its format pin) stays in force, unchanged.
supersedes: null
superseded_by: null

affected_scope:
  - src/cli/ops.rs
  - src/cli/optimize.rs
  - src/cli/common.rs
  - src/cli/build.rs
  - src/cli/mod.rs
  - src/sink/mod.rs
  - docs/api-contract.md
  - docs/cli-reference.md
  - docs/data-model.md

tags:
  - output-format
  - name-template
  - cli
  - build
  - cache
  - metadata
  - dec-087
  - dec-099
---

# DEC-105: an output name's literal extension pins the format on every verb that encodes

## Decision

**DEC-087's rule, extended from one `build` plan to every verb that encodes.** DEC-087 made a
`build` terminal-`optimize` target's literal-extension `name` its format pin. This decision makes
the same rule hold for `--name-template` on every pixel-lane verb at both arities, and for a plain
recipe `build` target's `name`:

1. **The pin.** A **recognised** literal extension in the name the output is written under pins the
   output format, exactly as a recognised `-o` extension already does. "Recognised" is exactly
   `crate::sink::format_from_extension`'s set (the one `-o` uses). A template containing `{ext}`
   (including the default `{stem}.{ext}`) never pins; `{name}` is not a literal extension (it
   expands to the source's own file name). The template is consulted only when it is the name the
   output is written under, so when `-o` is given or the output goes to stdout, the template does
   not pin.
2. **One precedence chain, extending DEC-099:** `--format` > the output name's recognised extension
   (`-o` when given, else the template) > `recipe.format` > the verb's default (preserve the source
   format, or the automatic choice on `web` / `optimize` / `apply --recipe web` / a `build`
   terminal-`optimize` target). A disagreement is **not an error**: `--format` wins silently,
   because that is what `--format webp -o x.png` does today (driven 2026-10-25: WebP bytes in
   `x.png`, exit 0), and the template takes `-o`'s rung, not a new one.
3. **A pin skips the automatic choice** on `web`, `optimize` and `apply --recipe web` (maintainer
   ruling, 2026-10-25), as `-o` and DEC-087 already do. `{ext}` or no extension keeps the
   automatic choice.
4. **An unrecognised or missing extension names the file only**, unchanged, on every surface
   except a `build` terminal-`optimize` target, which keeps DEC-087's exit 4. This matches `-o`,
   which falls through silently on an unrecognised extension on the same verbs.
5. **`meta strip` / `meta clean` / `meta set` refuse, exit 4, before writing anything**, when the
   template names a recognised format different from an input's container (maintainer ruling,
   2026-10-25). The metadata lane cannot convert without re-encoding pixels
   (`metadata-not-via-pixel-encode`); the message says to convert first. Same format, `{ext}`, or
   an unrecognised extension: unchanged.
6. **`build`'s cache:** a plain target whose `name` pins moves from the `Preserve` plan to the
   existing `Pinned` plan, so its key changes. That is required, not tolerated: two plain targets
   sharing one recipe file with different templates already produce different bytes. It rides
   0.8.0's single lockfile migration (the key already folds in the crate version); no
   `CACHE_SCHEMA_VERSION` change.

## Context

`docs/api-contract.md` (the `apply` entry, written by SPEC-126) said: "A literal extension in
`--name-template` does **not** pin the format; it names the file only." **No DEC held that
sentence.** SPEC-126's verify notes, SPEC-127 and the STAGE-050 backlog attributed it to DEC-087,
which says the opposite for the one surface it governs. This decision records the rule the repo
actually adopts and retires the undocumented one.

Driven on a release build of `main` (SPEC-134's frame at `8471fb1`; re-driven at `1145c31`), a
JPEG source, outputs identified by magic bytes: every encoding verb (`resize`, `thumbnail`,
`auto-orient`, `watermark`, `edit`, plain `apply` at 1 and N inputs, plain-recipe `build`) wrote
JPEG into `{stem}.png`; `web`, `optimize` and `apply --recipe web` wrote **AVIF** into `.png` /
`.webp` names while stderr reported "jpeg → avif"; and a `build` target with `recipe = "web"`,
`name = "{stem}.png"` already wrote a real PNG. The same recipe and template gave two answers
depending on the verb. The maintainer ruled on 2026-10-24: honour the template.

## Alternatives Considered

- **Keep "names the file only".** Rejected by the maintainer (2026-10-24): a file named `.png`
  must contain a PNG, and `build` already disagreed.
- **Warn on a mismatch instead of pinning.** Rejected for the same reason: the file is still
  mislabelled.
- **Exit 4 on an unrecognised template extension everywhere** (DEC-087's terminal-`optimize`
  answer). Rejected: it would make the template stricter than `-o` on the same verb, and it turns
  a plain `build` target with `name = "{stem}"` or `"{stem}.txt"`, exit 0 today, into exit 4.
- **Error when `--format` and the template disagree.** Rejected for now: `-o` has the same
  disagreement and resolves it silently. A warning for both is a follow-up, not part of this rule.
- **`meta` writes the source container under the template's name.** Rejected (ruling 3): that is
  the mislabel this decision exists to remove.

## Consequences

- **Positive:** one rule on every surface: the name you write is the format you get, and the
  stderr summary never names a codec the file does not hold.
- **Byte-changing** for any literal-extension template whose extension differs from today's
  output: `{stem}.png` on a JPEG source on every pixel verb, and every literal template on `web` /
  `optimize` / `apply --recipe web`. `{ext}` templates, including the default, are byte-identical.
- **New exit 2:** `--json` / `--timing` with a template pin on `web`, `optimize` and
  `apply --recipe web` is now a usage error, as it already is with an `-o` pin: there is no
  automatic choice to report.
- **`optimize --profile preserve`** with a literal template now pins, as it already does with `-o`.
- **The template-pinned report** is the `-o`-pinned one: no per-file decision line. The pinned
  path's quality (85 → 80) and its missing report are PROJ-013 / STAGE-037's `-o` item, not this
  decision's.
- **Not covered, filed:** `meta … -o x.png` on a JPEG still writes JPEG into `x.png`; `responsive`
  accepts `--name-template` and ignores it.
- **Unchanged:** `build` still never removes a target's previous output when its extension
  changes (DEC-087 Consequences).

## Validation

Right if SPEC-134's table test (`tests/name_template_pins_format.rs`) passes with every row's
output identified by magic bytes; `{ext}` outputs are byte-identical to the no-template run; plain
`build` with `{stem}` / `{stem}.txt` still exits 0; two plain `build` targets sharing a recipe
get distinct keys; and `meta` writes nothing when it refuses. Revisit if a user needs a template
extension that names the file without pinning (then add an explicit opt-out, not a silent
fall-through).

## References

- Related specs: SPEC-134 (this rule), SPEC-111 (DEC-087), SPEC-126 (wrote the retired api-contract
  sentence), SPEC-127 (`recipe.format`, DEC-099).
- Related decisions: DEC-087 (extended), DEC-099 (precedence chain extended), DEC-015 (preserve
  default, exit codes), DEC-058 / DEC-101 (cache key), DEC-048 (`--profile preserve`).
- Constraint: `metadata-not-via-pixel-encode`.
