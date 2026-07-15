# Sanitized Delegation-Link Fixtures

These fixtures are minimal synthetic derivatives of the current raw Codex collaboration shapes.
They preserve only the fields needed to prove the bounded R7 direct-link contract:

- parent `response_item` rows pair a `spawn_agent` `function_call` with its
  `function_call_output` by `call_id`; the output contains only a sanitized `agent_id`;
- child `session_meta.payload.source.subagent.thread_spawn` rows contain a sanitized child
  session id, `parent_thread_id`, `depth`, and optional generic agent metadata; and
- the ordinary single-agent control contains only a sanitized session id.

The matrix covers:

| Directory | Expected semantic posture |
|---|---|
| `reciprocal` | one verified direct parent/child claim |
| `parent_only` | one parent claim with no child artifact |
| `child_only` | one child origin with no parent spawn result |
| `conflict` | parent and child disagree about the parent id |
| `multi_child` | two independently verified direct children |
| `nested_depth` | matching ids at depth 2 remain unsupported residue |
| `single_agent` | no delegation claim or child origin |

No fixture is a copied rollout. Prompts, messages, credentials, working directories, absolute
private paths, unrelated tool calls/output, real session/call ids, and real nicknames are omitted.
`delegation_link_fixtures.rs` enforces the allowed field set and safe-token conventions in addition
to running every JSONL file through the production compactor ingest parser.
