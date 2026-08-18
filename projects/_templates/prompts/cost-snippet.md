# Cost-session snippet for cycle prompts (paste into build / verify / ship prompts)

> Replaces the old "append a cost session with **null numerics**" line that let
> cost tracking silently go empty (AGENTS.md §4 / docs/cost-tracking.md). Do NOT
> write `null` for build/verify cycles.

**For a build or verify prompt** (the cycle runs as a metered subagent):

```
Measure your own cost — do not estimate it, and do not leave it for someone else.

Your session transcript records per-message `usage`, so the number is available to
you whether or not you were dispatched as a subagent. Sum it:

  ~/.claude/projects/<cwd-slug>/<session-id>.jsonl

  Each line with `.message.usage` contributes input_tokens, output_tokens,
  cache_creation_input_tokens and cache_read_input_tokens. `tokens_total` is the
  sum of all four. Take duration from the first and last `timestamp`, and the
  model from `.message.model`.

  (The session id is the last path component of your scratchpad directory. If the
  transcript is genuinely unreadable, say so explicitly in the readout and write
  `tokens_total: null` — a stated gap is fine, a made-up number is not.)

Append your cycle's entry to the spec front-matter `cost.sessions`:
  - cycle: <build|verify>
    agent: <model id from the transcript>
    interface: claude-code
    tokens_total: <measured sum>
    duration_minutes: <measured>
    recorded_at: <YYYY-MM-DD>
    tokens_breakdown: {input, output, cache_creation, cache_read}
    estimated_usd: <see the pricing note below>
    note: "<one line; say MEASURED or say why not>"

END YOUR RETURN MESSAGE with this block, verbatim, as the last thing you emit —
the orchestrator reads cost from your return, not by going hunting for it:

  ## Cost readout
  cycle:            <build|verify>
  spec:             SPEC-NNN
  agent:            <model id>
  tokens_total:     <n>
  breakdown:        in <n> / out <n> / cache-write <n> / cache-read <n>
  duration_minutes: <n>
  estimated_usd:    <n>
  source:           transcript sum over <n> assistant messages | subagent_tokens
                    | UNAVAILABLE (<reason>)

If you WERE dispatched as a subagent, the orchestrator cross-checks your
`tokens_total` against the Agent result's `subagent_tokens`. Two independent
numbers that agree is the point; if they disagree, the Agent result wins and the
gap gets recorded.
```

**Budget in MESSAGE COUNT, not minutes — measured, 2026-08-16.**

```
Cost scales with the SQUARE of message count, because every message re-reads the
accumulated context. Across four builds in one wave (cache reads 97-99% of tokens
in all four), relative to SPEC-116:

  spec       msgs    total tokens   msgs^2   minutes      $
  SPEC-120   0.41x       0.40x       0.17x    0.21x     8.69
  SPEC-116   1.00x       1.00x       1.00x    1.00x    11.91
  SPEC-117   1.57x       2.18x       2.47x    0.60x    23.06
  SPEC-119   2.29x       4.99x       5.23x    0.59x    51.24

MINUTES ANTI-CORRELATE. SPEC-116 ran 104 minutes for $11.91; SPEC-119 ran 61 for
$51.24. Every prompt in that wave carried a wall-clock budget and not one fired.

So write the stop condition as an exchange count -- "past ~250 exchanges without
having started the matrix, checkpoint and report" -- which would have fired on
SPEC-119 and correctly stayed silent on SPEC-116's slow, cheap run.
```

**Watching CI settle is NOT free — measured 2026-08-17 on SPEC-123's build.**

```
On a cycle that is ~98% cache reads, every poll re-reads the whole accumulated
context, so a quiet wait costs about as much as real work.

  SPEC-123 build: $5.80 -- 13% of its $46.17 -- went on observing a CI matrix it
  had already triggered and could not influence.

Its cost was measured three times, and the "almost done" reading was the worst:

  $32.80 at 242 messages   <- PR already open, only CI left to watch
  $40.37 at 285
  $46.17 at 318            <- final, 29% above the "almost done" reading

So: PREFER ONE LONG WAIT TO MANY SHORT POLLS, and take the cost reading AFTER CI
has settled, not when the PR opens. A cycle that reports cost at the moment it
thinks it is finished will under-report it.
```

**Pricing — do not apply the 80/20 rule blind.**

```
estimated_usd prices each component separately, at the list anchors of the model
that ACTUALLY RAN — the one you record in `agent`, read from `.message.model` in
your own transcript. Do not use the anchors a prompt happens to name.

  Opus    $5 / $25 per MTok (input / output)
  Sonnet  $3 / $15 per MTok

  input           x1.00 input rate
  output          x1.00 output rate
  cache_creation  x1.25 input rate
  cache_read      x0.10 input rate

State the anchors you used, next to the agent. THIS MISMATCH HAS ALREADY
HAPPENED: SPEC-108's build and verify both recorded `agent: claude-sonnet-5` and
priced at the Opus anchors their prompt named, overstating that spec's total by
~67% ($104 recorded against ~$62 at Sonnet rates). The token counts were right;
only the dollar figure was wrong, and it was wrong because the rule named rates
without tying them to the model.

The older "tokens_total x list rate, ~80/20 in/out, no cache discount" shortcut is
only sound when cache reads are a small share of volume. On a long agentic cycle
they dominate — SPEC-109's build measured 98.7% cache reads, where the shortcut
returned $588 against a component-accurate $43.21, a 14x overstatement. Price the
components; note the rate anchors used.
```

**For the orchestrator's ship bookkeeping** (this is where real numbers land):

```
For each metered cycle (build, verify), the numbers should ALREADY be in the spec —
the cycle measured them and closed with a `## Cost readout` block. Your job is to
check them, not to source them:
  - the readout's tokens_total matches what landed in cost.sessions
  - if the cycle ran as a subagent, cross-check against the Agent result's
    subagent_tokens; on disagreement the Agent result wins, and record the gap
  - a `source: UNAVAILABLE` readout is the ONLY acceptable route to a null, and
    it must carry a reason
If a cycle returned no readout at all, ask for one before shipping rather than
filling the number in yourself — a cost the executor did not measure is a guess
wearing a measurement's clothes.
  duration_minutes= round(duration_ms / 60000), or first→last transcript timestamp
  estimated_usd   = per-component pricing (see the pricing note above), NOT the
                    flat 80/20 shortcut.
Leave design/ship (orchestrator main-loop) numerics null with a
"main-loop, not separately metered" note.
Compute cost.totals: tokens_total = sum of non-null sessions (use 0, not null,
for the placeholder), estimated_usd = sum, session_count = number of cycles.
Then `just cost-audit` must pass before the spec is considered shipped.
```
