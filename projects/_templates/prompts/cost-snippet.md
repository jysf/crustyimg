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
