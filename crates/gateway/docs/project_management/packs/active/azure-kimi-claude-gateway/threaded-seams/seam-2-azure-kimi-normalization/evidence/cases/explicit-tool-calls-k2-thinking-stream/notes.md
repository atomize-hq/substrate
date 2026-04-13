# Explicit Tool Calls - K2 Thinking Stream

- Source log: `~/.claude-code-router/logs/ccr-20260326143921.log` (`req-9`).
- Request evidence: Azure request body at line `11` is a streaming `Kimi-K2-Thinking` chat-completions call with a tool catalog present.
- Response evidence: selected chunk lines `371`, `373`, `583`, `629`, `685`, and `755` show the sequence from reasoning text, to assistant progress text, to explicit `delta.tool_calls`, to terminal `finish_reason: "tool_calls"`.
- Observed behavior: the tool identities and arguments arrive through explicit streamed `tool_calls`; no hidden tool markers are required to recover the `Bash` and `Read` intents for this case.
- Normalized expectation: emit one `action` event for the assistant progress text and two `tool_intent` events for the streamed `Bash` and `Read` calls; do not emit a `final` event before the tool handoff.
