# Substrate Gateway Backend Adapter Schema

This document is the durable canonical contract reference for the gateway adapter schema
boundary. It defines the adopted capability advertisement subset, the adopted run extension
subset, the bounded event/completion payload shapes, the bounded adapter error shape, and the
bounded session-handle facet.

## Contract

The gateway-local adapter schema owns:

- capability advertisement for the adopted cross-backend capability subset
- the adopted run extension-key subset and its closed `.v1` payload shapes
- the bounded request payload shape passed into adapter execution
- the bounded event and completion payload shapes emitted from adapter execution
- the bounded adapter error shape
- the bounded session-handle facet shape

### Adopted Capability Advertisement Subset

The adopted cross-backend capability ids are:

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

- Gateway-facing capability advertisement uses only the adopted `agent_api.*` ids above as stable
  contract truth.
- Backend-specific capability ids remain implementation detail unless a later contract revision
  promotes them into this document.
- Non-run MCP management capability ids are outside the current adapter-execution schema contract.
- Dangerous or topology-sensitive execution policy extensions remain out of the current adopted
  subset until the external owner docs and runtime boundary agree on them as stable contract truth.

### Adopted Run Extension Subset

The adopted run extension keys are:

- `agent_api.exec.non_interactive`
- `agent_api.exec.add_dirs.v1`
- `agent_api.session.resume.v1`
- `agent_api.session.fork.v1`

Rules:

- Unsupported extension keys fail closed before spawn.
- `agent_api.exec.non_interactive` is a boolean and defaults to the owner-doc default when absent.
- `agent_api.exec.add_dirs.v1` is a closed object owned by the Unified Agent API extensions spec;
  unknown keys are invalid.
- `agent_api.session.resume.v1` and `agent_api.session.fork.v1` are closed objects with:
  - `selector: "last" | "id"`
  - `id` required only when `selector == "id"`
  - `id` absent when `selector == "last"`
- Resume and fork selectors are mutually exclusive in one request.

### Adapter Request Payload

The bounded gateway-local adapter request payload is:

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

- `backend_id` must already satisfy the stable backend-id contract.
- `prompt` is required and must be non-empty after trimming.
- `working_dir`, `timeout_ms`, and `env` remain optional request metadata and must not redefine
  backend identity or policy.
- `extensions` is limited to the adopted run extension subset above.
- Backend-specific integrated auth handoff material, when required to realize adapter execution,
  is bounded request input rather than backend-selection or policy input.

### Adapter Event and Completion Payloads

The bounded gateway-local event payload shape is:

```text
{
  kind: "text_output" | "tool_call" | "tool_result" | "status" | "error" | "unknown"
  channel?: string
  text?: string
  message?: string
  data?: object
}
```

The bounded gateway-local completion payload shape is:

```text
{
  exit_status: integer
  final_text?: string
  data?: object
}
```

Rules:

- Event and completion data must stay bounded and metadata-only.
- Tool metadata may use the bounded structured tools facet when the adapter advertises
  `agent_api.tools.structured.v1`.
- `final_text` is optional and appears only when the backend can extract it deterministically.

### Session-Handle Facet

The adopted bounded session-handle facet is:

```json
{
  "schema": "agent_api.session.handle.v1",
  "session": { "id": "string" }
}
```

Rules:

- `session.id` is an opaque backend-defined string.
- `session.id` must be non-empty after trimming.
- `session.id` must not exceed 1024 UTF-8 bytes; oversize ids are omitted rather than truncated.
- The facet may appear on one early status event and on completion metadata when known.
- The facet must come from typed backend event models rather than raw stdout or stderr parsing.

### Adapter Error Shape

The bounded adapter error payload is:

```text
{
  kind: "unknown_backend" | "unsupported_capability" | "invalid_request" | "backend"
  message: string
  capability?: string
}
```

Rules:

- `message` must be safe and redacted.
- `kind: "unsupported_capability"` may include the rejected capability id in `capability`.
- `kind: "invalid_request"` covers closed-schema or contradiction failures for the adopted
  extension subset.
- `kind: "invalid_request"` also covers missing or incomplete required integrated auth handoff
  material after policy has permitted the relevant host-side sourcing path.
- `kind: "backend"` covers runtime rejections and bounded backend-owned failure messages,
  including the pinned cancelled completion outcome and the reserved safe session-selection failure
  messages.
- Raw provider payloads, raw backend lines, stack traces, and secret-bearing strings are out of
  contract.

## Boundaries

- This schema does not redefine the top-level structured-event envelope or trace vocabulary. Those
  remain owned by ADR-0017 and ADR-0028.
- This schema does not define the durable host-to-world carrier for integrated auth material.
  Env-based delivery, file-backed delivery, or a future secret-channel handoff remain outside this
  schema as long as the published policy precedence and bounded request validation rules stay
  unchanged.
- This schema does not redefine machine-readable gateway status output. That remains owned by
  `docs/contracts/gateway/status-schema.md`.
- This schema does not make backend-specific capability ids, provider quirks, or raw transport
  details part of the stable Substrate-facing contract.

## Verification Surfaces

The implementation and verification surfaces for this contract are expected to stay aligned across:

- `docs/adr/implemented/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- the Unified Agent API capability, extension, run-protocol, and event-envelope specs cited by
  ADR-0041
- the built-in backend capability and session-handle tests cited by the seam-local schema spec
