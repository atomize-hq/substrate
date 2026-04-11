# Gateway Backend Adapter Protocol Spec

This spec is the seam-local execution baseline for `C-03`. The durable contract text for this
surface lives in `docs/contracts/substrate-gateway-backend-adapter-protocol.md`.

## Scope

This spec owns the seam-local execution checklist for:

- adapter registry lookup and dispatch order after a stable backend id is selected
- fail-closed capability and extension-key validation order
- request normalization before adapter execution
- normalized event and completion emission order
- the exact local-to-external handoff boundary to ADR-0017 and ADR-0028

## Deterministic Lifecycle

The adopted lifecycle is:

1. consume the already-selected stable backend id from the selection contract
2. resolve exactly one adapter for that backend id
3. load the adopted capability advertisement subset
4. apply fail-closed required-capability and extension-key gating before any value validation
5. validate adopted extension payloads and selector contradictions only after the capability gate passes
6. normalize request metadata (`working_dir`, `timeout`, `env`, adopted extension payloads)
7. start adapter execution
8. emit normalized events in backend order
9. surface exactly one completion result with bounded completion metadata

## Local-To-External Owner Line

Local adapter protocol ownership stops at:

- adapter lookup and dispatch
- adopted capability and extension validation
- request normalization
- mapping typed backend events into gateway-local `kind/channel/text/message/data` items
- attaching bounded session-handle metadata and bounded adapter failure detail

ADR-0017 remains the owner of:

- top-level structured-event envelope fields
- output-class routing and PTY-versus-structured rendering semantics
- orchestration attribution and routing hints

ADR-0028 remains the owner of:

- canonical trace vocabulary
- trace join keys and persistence semantics
- trace append ordering and correlation rules

The standalone gateway remains the owner of:

- raw provider transport parsing
- normalized provider-to-gateway event semantics before Substrate-facing wrapping

## Repository Topology Note

The current authority set is split across the standalone gateway repo and the Universal Agent API
repo cited by ADR-0041. Moving the gateway into the Substrate monorepo later, or vendoring it via
submodule, does not change protocol ownership by itself. Repo packaging stays implementation
topology, not contract truth.

## Execution Checklist

### Doc surfaces that define the landed protocol baseline

- `docs/contracts/substrate-gateway-backend-adapter-protocol.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md`
- `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`

### Code surfaces that will need to match this baseline when implementation lands

- `/Users/spensermcconnell/__Active_Code/codex-wrapper/crates/agent_api/src/lib.rs`
- `/Users/spensermcconnell/__Active_Code/codex-wrapper/crates/agent_api/src/backends/session_selectors.rs`
- `/Users/spensermcconnell/__Active_Code/codex-wrapper/crates/agent_api/src/backends/codex/harness.rs`
- `/Users/spensermcconnell/__Active_Code/codex-wrapper/crates/agent_api/src/backends/claude_code/harness.rs`
- `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/gateway/src/structured_events.rs`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-client/src/lib.rs`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/handlers.rs`
- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs`

### Verification plan

The protocol baseline is only considered landed when the following verification surfaces stay
aligned:

- `codex_backend_routes_through_harness_and_does_not_reintroduce_orchestration_primitives`
  in `/Users/spensermcconnell/__Active_Code/codex-wrapper/crates/agent_api/src/backends/codex/tests/backend_contract.rs`
- `claude_backend_routes_through_harness_and_does_not_reintroduce_orchestration_primitives`
  in `/Users/spensermcconnell/__Active_Code/codex-wrapper/crates/agent_api/src/backends/claude_code/tests/backend_contract.rs`
- `resume_v1_invalid_cases_rejected_with_pinned_messages` and
  `fork_v1_invalid_cases_rejected_with_pinned_messages`
  in `/Users/spensermcconnell/__Active_Code/codex-wrapper/crates/agent_api/src/backends/session_selectors.rs`
- event-order and safe-error regression coverage in:
  - `/Users/spensermcconnell/__Active_Code/codex-wrapper/crates/agent_api/tests/c1_codex_exec_policy.rs`
  - `/Users/spensermcconnell/__Active_Code/codex-wrapper/crates/agent_api/tests/c1_codex_stream_exec_adapter.rs`
  - `/Users/spensermcconnell/__Active_Code/codex-wrapper/crates/agent_api/tests/c5_claude_add_dirs_runtime_rejection.rs`
  - `/Users/spensermcconnell/__Active_Code/codex-wrapper/crates/agent_api/tests/c3_explicit_cancellation.rs`
  - `/Users/spensermcconnell/__Active_Code/codex-wrapper/crates/agent_api/tests/c3_explicit_cancellation_claude_code.rs`

Pass/fail conditions:

- pass when no protocol surface bypasses the shared harness-owned gating and completion rules
- pass when unsupported capabilities and unsupported extension keys fail before spawn
- pass when local mapping stops at gateway-local event and completion shapes and does not reopen
  ADR-0017 or ADR-0028 ownership
- fail when provider transport details, backend-specific sub-identities, or trace-envelope fields
  leak into the adapter protocol baseline
