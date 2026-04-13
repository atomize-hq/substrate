# Hidden Markers - K2 Thinking Stream

- Source log: `~/.claude-code-router/logs/ccr-20260326143921.log` (`req-x`).
- Request evidence: Azure request body at line `4256` is a streaming `Kimi-K2-Thinking` chat-completions call with the normal tool catalog present.
- Response evidence: selected chunk lines `4319`, `4325`, and `4447` show the streamed reasoning prefix, the hidden marker section begin, and the terminal `finish_reason: "stop"` while no explicit `delta.tool_calls` ever arrive.
- Observed behavior: Azure can stream hidden tool intent entirely through piecewise `reasoning_content` deltas and still terminate with `finish_reason: "stop"`.
- Normalized expectation: emit one `action` event for the recovered reasoning prefix plus `tool_intent` events for the hidden `Read` and `Skill` calls; normalize the terminal stop into `stop_reason = tool_use`.
