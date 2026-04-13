# No-Tool Control - K2.5 Stream

- Source log: `~/.claude-code-router/logs/ccr-20260327000000_1.log` (`req-3`).
- Request evidence: Azure request body at line `33` is a streaming `Kimi-K2.5` chat-completions call with no tool catalog.
- Response evidence: selected chunk lines `42`, `44`, `1507`, `1517`, and `1520` show reasoning text, final visible content, `finish_reason: "stop"`, and terminal usage with no `tool_calls` or hidden marker section.
- Observed behavior: `Kimi-K2.5` still streams `reasoning_content`, but in this control case it never advertises tool use and ends with a normal final content payload.
- Normalized expectation: emit exactly one `final` event for the returned JSON title text; keep the reasoning-only trace as debug provenance and do not fabricate `action` or `tool_intent` events.
