# Azure Kimi Variant Notes

## K2.5 Hidden Marker Evidence

- Source log: `~/.claude-code-router/logs/ccr-20260326143921.log` (`req-36`, request line `23603`, response line `23604`).
- Observation: `Kimi-K2.5` can also return hidden tool markers entirely inside `reasoning_content` while leaving `tool_calls: null` in the non-streaming Azure response. The sampled payload uses `<|tool_calls_section_begin|>` with a `functions.Write:34` call.
- Implication for `C-02`: hidden-marker parsing must stay Azure-Kimi-family aware rather than hard-coding `Kimi-K2-Thinking` as the only hidden-tool variant.

## Streaming Hidden Marker Evidence

- Source log: `~/.claude-code-router/logs/ccr-20260326143921.log` (`req-x`, request line `4256`, selected response lines `4319`, `4325`, and `4447`).
- Observation: `Kimi-K2-Thinking` can emit hidden tool markers piecewise across streaming `reasoning_content` deltas; the reconstructed reasoning text includes `<|tool_calls_section_begin|>` and two hidden tool intents even though no explicit `delta.tool_calls` arrive.
- Implication for `C-02`: downstream seams must not assume hidden-marker recovery is limited to non-streaming completions.

## Fixture Scope Decision

- The regression corpus now lands five core fixtures: explicit streamed tool calls, streamed hidden markers, hidden non-streaming markers, mixed explicit-plus-reasoning, and one no-tool control.
- The additional `Kimi-K2.5` hidden-marker sample remains recorded here as a stale-trigger input instead of becoming a sixth fixture so the seam can keep the regression corpus narrow while still naming the variant risk for later revalidation.
