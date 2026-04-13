# Hidden Markers - K2 Thinking Nonstream

- Source log: `~/.claude-code-router/logs/ccr-20260326143921.log` (`req-k`).
- Request evidence: Azure request body at line `1687` is a non-streaming `Kimi-K2-Thinking` call with the normal tool catalog present.
- Response evidence: line `1688` contains `tool_calls: null` while `reasoning_content` carries `<|tool_calls_section_begin|>` sentinel blocks for two `Read` calls.
- Observed behavior: Azure can hide tool intent entirely inside `reasoning_content` and still return a non-streaming completion shape with no explicit `tool_calls` array.
- Normalized expectation: emit one `action` event for the explanatory prefix and two `tool_intent` events recovered from the hidden marker section; the hidden sentinel syntax remains debug-only provenance.
