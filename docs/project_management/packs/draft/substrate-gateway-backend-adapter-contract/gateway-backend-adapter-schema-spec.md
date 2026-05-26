# Gateway Backend Adapter Schema Spec

This spec is the seam-local execution baseline for `C-04`. The durable contract text for this
surface lives in `docs/contracts/gateway/backend-adapter-schema.md`.

## Adopted Unified Agent API Subset

### Adopted capability ids

The adopted cross-backend capability subset is:

- `agent_api.run`
- `agent_api.events`
- `agent_api.events.live`
- `agent_api.exec.non_interactive`
- `agent_api.exec.add_dirs.v1`
- `agent_api.session.resume.v1`
- `agent_api.session.fork.v1`
- `agent_api.session.handle.v1`
- `agent_api.control.cancel.v1`
- `agent_api.tools.structured.v1`
- `agent_api.tools.results.v1`
- `agent_api.artifacts.final_text.v1`

Rules:

- the adopted capability ids above are the only gateway-facing capability ids in this baseline
- backend-specific capability ids remain implementation detail and must not widen the contract
- unsupported capability requirements fail closed before adapter execution starts
- capability validation uses the adopted subset above as closed contract truth rather than an inferred backend surface

### Explicitly deferred capability ids

These remain outside the current adapter contract baseline:

- `agent_api.exec.external_sandbox.v1`
  - dangerous execution-policy opt-in that changes trust-boundary posture
- `agent_api.config.model.v1`
  - backend-selection-adjacent tuning surface not required for the current adapter baseline
- `agent_api.tools.mcp.list.v1`
- `agent_api.tools.mcp.get.v1`
- `agent_api.tools.mcp.add.v1`
- `agent_api.tools.mcp.remove.v1`
  - non-run management APIs rather than adapter-execution payload semantics
- backend-specific capability ids such as `backend.codex.*` and `backend.claude_code.*`
  - implementation detail until promoted by a later contract revision

## Adopted Extension-Key Subset

The adopted run-extension subset is:

- `agent_api.exec.non_interactive`
- `agent_api.exec.add_dirs.v1`
- `agent_api.session.resume.v1`
- `agent_api.session.fork.v1`

Closed-schema rules:

- unsupported extension keys fail closed before adapter execution starts
- `agent_api.exec.add_dirs.v1`
  - object with required `dirs: string[]`
  - unknown keys are invalid
- `agent_api.session.resume.v1`
  - object with `selector: "last" | "id"`
  - `id` required only when `selector == "id"`
  - `id` absent when `selector == "last"`
- `agent_api.session.fork.v1`
  - object with `selector: "last" | "id"`
  - `id` required only when `selector == "id"`
  - `id` absent when `selector == "last"`
- resume and fork are mutually exclusive in the same request

## Bounded Payload Inventory

### Request payload

```text
{
  backend_id: string
  prompt: string
  working_dir?: string
  timeout_ms?: integer
  env?: object<string, string>
  extensions?: object
}
```

Rules:

- `backend_id` stays the stable `<kind>:<name>` selector from the upstream selection contract.
- `extensions` is limited to the adopted subset above.
- session selectors reuse the closed `.v1` object shapes above rather than inventing a second
  selector surface.

### Event payload

```text
{
  kind: "text_output" | "tool_call" | "tool_result" | "status" | "error" | "unknown"
  channel?: string
  text?: string
  message?: string
  data?: object
}
```

Allowed `data` schemas in the adopted baseline:

- `agent_api.tools.structured.v1`
- `agent_api.session.handle.v1`

### Completion payload

```text
{
  exit_status: integer
  final_text?: string
  data?: object
}
```

Allowed completion metadata in the adopted baseline:

- `agent_api.session.handle.v1`

### Adapter error payload

```text
{
  kind: "unknown_backend" | "unsupported_capability" | "invalid_request" | "backend"
  message: string
  capability?: string
}
```

Pinned rules:

- `unsupported_capability` keeps the rejected capability id bounded in `capability` and applies when a capability outside the adopted subset is requested or advertised
- `invalid_request` covers adopted extension-schema and contradiction failures
- `backend` covers safe runtime rejection messages, including:
  - `"cancelled"`
  - `"no session found"`
  - `"session not found"`
  - bounded safe messages for runtime rejection such as add-dir rejection
- raw provider payloads, stack traces, backend stdout/stderr lines, and secret-bearing values are
  never part of this error payload

### Session-handle facet

```json
{
  "schema": "agent_api.session.handle.v1",
  "session": { "id": "string" }
}
```

Pinned rules:

- `session.id` is opaque and round-trippable into the adopted session-selector objects
- `session.id` must be non-empty after trimming
- `session.id` must not exceed 1024 UTF-8 bytes
- the facet appears at most once on an early status event and on completion metadata when known

## Execution Checklist

### Doc surfaces that define the landed schema baseline

- `docs/contracts/gateway/backend-adapter-schema.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md`

### Runtime-adjacent adoption surfaces to verify against when implementation lands

These are planning and verification anchors only. They describe where later implementation must align with `C-04`; they do not widen this slice into implementation work.

- `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/lib.rs`
- `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/session_selectors.rs`
- `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/codex/backend.rs`
- `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/codex/harness.rs`
- `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/claude_code/backend.rs`
- `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/claude_code/harness.rs`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/transport-api-types/src/lib.rs`

### Verification plan

Keep the following verification surfaces aligned with the adopted subset:

- `codex_backend_reports_required_capabilities`
  in `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/codex/tests/capabilities.rs`
- `claude_backend_reports_required_capabilities`
  in `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/claude_code/tests/capabilities.rs`
- `handle_facet_emitted_once_on_thread_started_and_attached_to_completion`
  and `synthetic_status_is_emitted_if_id_first_seen_on_non_status_event`
  in `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/codex/tests/session_handle.rs`
- `claude_emits_handle_facet_once_when_first_event_is_status` and
  `claude_completion_attaches_handle_facet_when_id_is_known`
  in `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/claude_code/tests/session_handle.rs`
- `resume_v1_invalid_cases_rejected_with_pinned_messages` and
  `fork_v1_invalid_cases_rejected_with_pinned_messages`
  in `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/session_selectors.rs`
- runtime rejection and cancellation coverage in:
  - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/tests/c1_codex_exec_policy.rs`
  - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/tests/c5_claude_add_dirs_runtime_rejection.rs`
  - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/tests/c3_explicit_cancellation.rs`
  - `/Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/tests/c3_explicit_cancellation_claude_code.rs`

Pass/fail conditions:

- pass when both built-in backends advertise the adopted subset and omit the explicitly deferred
  surfaces from the gateway-facing baseline
- pass when closed-schema selector validation and session-handle bounds stay pinned
- pass when safe runtime rejection messages stay bounded and do not leak raw backend output
- fail when backend-specific capability ids, model-selection policy, dangerous external-sandbox
  posture, or MCP management APIs drift into the adopted execution schema without a new contract
