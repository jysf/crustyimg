# Cost-session snippet for cycle prompts (paste into build / verify / ship prompts)

> Replaces the old "append a cost session with **null numerics**" line that let
> cost tracking silently go empty (AGENTS.md §4 / docs/cost-tracking.md). Do NOT
> write `null` for build/verify cycles.

**For a build or verify prompt** (the cycle runs as a metered subagent):

```
Append your cycle's entry to the spec front-matter `cost.sessions` with:
  - cycle: <build|verify>
    agent: <claude-sonnet-4-6 | claude-opus-4-8>
    interface: claude-code
    tokens_total: <leave null here>     # the ORCHESTRATOR fills the real number
    estimated_usd: <leave null here>    # from the Agent result at ship
    duration_minutes: <leave null here>
    recorded_at: <YYYY-MM-DD>
    notes: "<one line>"
Do NOT invent token numbers. The orchestrator records the real tokens_total /
duration / estimated_usd from your Agent result during ship bookkeeping.
(If you run this cycle interactively instead of as a subagent, run `/cost` and
write the real numbers here yourself.)
```

**For the orchestrator's ship bookkeeping** (this is where real numbers land):

```
For each metered cycle (build, verify), write the real values into cost.sessions:
  tokens_total    = the Agent result's subagent_tokens
  duration_minutes= round(duration_ms / 60000)
  estimated_usd   = tokens_total × list rate (Opus 4.8 $5/$25, Sonnet 4.6
                    $3/$15 per MTok), ~80/20 input/output, no cache discount;
                    add a one-line note that it's an order-of-magnitude estimate
                    (the harness reports one combined token metric, no I/O split).
Leave design/ship (orchestrator main-loop) numerics null with a
"main-loop, not separately metered" note.
Compute cost.totals: tokens_total = sum of non-null sessions (use 0, not null,
for the placeholder), estimated_usd = sum, session_count = number of cycles.
Then `just cost-audit` must pass before the spec is considered shipped.
```
