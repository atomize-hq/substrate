---
slice_id: S1
seam_id: SEAM-1
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the route contract changes the field-classification matrix, minimal header contract, or typed-message/image rules after this slice starts
    - auth-handoff planning changes the consumed account-id input shape in a way that requires revisiting request builder assumptions
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-14
contracts_produced:
  - C-14
contracts_consumed: []
open_remediations: []
---
### S1 - Unify Codex Route Request Shaping And Header Contract

- **User/system value**: the provider sends one explicit ChatGPT Codex wire contract instead of a generic Responses approximation with sync/stream divergence and silent compatibility drift.
- **Scope (in/out)**:
  - In: unify sync and streaming on `/codex/responses`, enforce `stream = true` and `store = false`, emit only the minimal header set plus the normalized auth input, translate accepted message/image shapes into the Codex wire form, and deterministically reject unsupported caller-visible controls.
  - Out: semantic stream assembly and sync-drain failure handling, downstream auth-handoff ownership, and whole-pack conformance ownership.
- **Acceptance criteria**:
  - `send_message()` and `send_message_stream()` use the same Codex endpoint when OAuth routing selects this transport
  - the provider emits only `Authorization`, `ChatGPT-Account-ID`, and `Content-Type` unless a later ADR revalidates more headers
  - the serializer uses typed `message` items, flat function tools, and flat explicit `tool_choice`
  - unsupported controls reject deterministically instead of being silently stripped or degraded
  - image inputs remain supported through typed `message` items with upstream `image_url` content parts
- **Dependencies**: `S00`, `crates/gateway/src/providers/openai.rs`, `crates/gateway/src/models/mod.rs`, `THR-14`
- **Verification**:
  - positive tests prove endpoint parity, minimal headers, typed-message translation, flat tool serialization, flat explicit `tool_choice`, and accepted image-part behavior
  - negative tests prove rejected fields and shapes fail before the upstream call
  - pass condition: a reviewer can trace one Codex-routed request from normalized input through the provider builder without seeing generic Responses drift
- **Rollout/safety**: keep all route-specific shaping below public ingress and do not let auth ownership bleed into this slice.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`) and `review.md` (`Likely mismatch hotspots`)

#### S1.T1 - Unify Sync And Streaming On The Codex Endpoint

- **Outcome**: both provider paths use one upstream transport contract.
- **Inputs/outputs**: inputs are `C-14`, current provider request builders, and OAuth route selection; outputs are endpoint-selection and request-builder updates for sync and streaming.
- **Thread/contract refs**: `THR-14`, `C-14`
- **Implementation notes**: remove `/responses` streaming drift, force `stream = true`, and keep sync as a stream-drain adaptation rather than a separate upstream behavior.
- **Acceptance criteria**: there is one route builder and one endpoint decision for Codex OAuth, shared by sync and streaming.
- **Test notes**: add assertions for endpoint parity, forced `stream` and `store`, and route selection behavior.
- **Risk/rollback notes**: partial endpoint unification will keep sync and streaming semantically inconsistent.

Checklist:
- Implement: route sync and streaming through one Codex endpoint builder
- Test: assert endpoint parity and forced `stream` / `store`
- Validate: confirm sync still uses stream drain rather than a second upstream mode
- Cleanup: remove stale `/responses` streaming assumptions

#### S1.T2 - Enforce The Minimal Header And Request-Shaping Contract

- **Outcome**: request shaping matches the owned Codex route contract instead of generic Responses defaults.
- **Inputs/outputs**: inputs are `C-14`, current serializer code, normalized request/tool/image shapes, and consumed auth context; outputs are serializer and header-emission updates plus request validation behavior.
- **Thread/contract refs**: `THR-14`, `C-14`
- **Implementation notes**: emit only minimal headers, use typed `message` items, keep image translation explicit, serialize flat function tools and flat explicit `tool_choice`, and reject unsupported fields/shapes deterministically.
- **Acceptance criteria**: accepted controls preserve caller-visible semantics, rejected controls fail loudly, and no extra parity headers remain on the route by default.
- **Test notes**: add request-shape coverage for typed messages, images, tool definitions, explicit `tool_choice`, and rejected field families.
- **Risk/rollback notes**: if shaping stays permissive, later conformance work will need to reverse-engineer silent behavior loss.

Checklist:
- Implement: update serializer and header emission to the minimal Codex contract
- Test: cover accepted and rejected request controls plus header assertions
- Validate: confirm account-id consumption remains explicit but non-owning
- Cleanup: remove generic Responses assumptions not allowed on this route
