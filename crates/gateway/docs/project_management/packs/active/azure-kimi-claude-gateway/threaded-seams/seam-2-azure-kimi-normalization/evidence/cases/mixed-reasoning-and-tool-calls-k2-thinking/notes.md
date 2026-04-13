# Mixed Reasoning And Tool Calls - K2 Thinking

- Source log: `~/.claude-code-router/logs/ccr-20260326143921.log` (`req-a`).
- Request evidence: Azure request body at line `762` is a non-streaming `Kimi-K2-Thinking` call with the full tool catalog present.
- Response evidence: line `1386` contains assistant `content`, non-empty `reasoning_content`, and three explicit `tool_calls` in the same non-streaming response.
- Observed behavior: Azure can return explicit tool intents and hidden reasoning in one payload; this is the collision path called out by `C-02`.
- Normalized expectation: keep the assistant progress text as one `action`, emit one `tool_intent` per explicit tool call, and retain the hidden reasoning only as debug provenance because it does not add competing tool identity in this sample.
