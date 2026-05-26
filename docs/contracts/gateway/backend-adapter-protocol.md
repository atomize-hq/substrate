# Substrate Gateway Backend Adapter Protocol

This document is the durable canonical contract reference for the gateway adapter protocol
boundary. It defines the deterministic lifecycle from a selected stable backend id to
adapter execution, normalized event emission, and completion without widening external ownership.

## Contract

The adapter protocol owns:

- adapter registry lookup after a stable backend id has already been selected and allowlisted
- pre-spawn validation order for adopted run extensions and required capabilities
- request normalization before adapter execution
- response and event translation inside the gateway-local adapter boundary
- explicit handoff to the external structured-event and trace owners

Concrete rules:

- Adapter dispatch starts only after the stable `<kind>:<name>` backend id from the selection
  contract has been accepted as a fixed input.
- Prompt-bearing host orchestration and world-member execution must reach adapter dispatch through
  the gateway-owned runtime seam rather than by rebuilding direct backend-registration tables in
  shell-local or world-local runtime code.
- One selected backend id maps to one adapter dispatch target for the duration of a run. Internal
  provider or wrapper mechanics remain hidden behind that adapter identity.
- If adapter resolution fails for a backend id that has already passed selection and allowlist
  validation, the outcome is dependency unavailable rather than invalid selection.
- The protocol lifecycle is:
  1. resolve the adapter for the already-selected backend id
  2. load the adapter capability advertisement for the adopted capability subset
  3. fail closed on unsupported extension keys or missing required capabilities before spawn
  4. validate adopted extension payloads only after the capability gate passes
  5. normalize request fields and session-selector intent before adapter execution starts
  6. execute the adapter and emit normalized events in backend order
  7. surface one completion result and any bounded metadata attached to that completion
- Unsupported extension keys and unsupported required capabilities fail closed before any adapter
  process is started.
- Extension-value validation uses closed `.v1` schema rules where the adopted Unified Agent API
  owner docs pin them.
- Session resume and fork selectors are mutually exclusive.
- Session-handle metadata is gateway-contract data only. It must not be treated as policy input,
  backend selection input, provider identity, or operator identity.
- Explicit cancellation, when the adapter advertises it, is best-effort and resolves the run with
  the safe cancelled backend error contract rather than silently hanging or reopening approval
  semantics.

## External Ownership Boundary

Gateway-local adapter translation owns:

- adapter lookup and dispatch ordering
- adopted capability and extension-key validation
- request normalization
- mapping typed backend events into the bounded gateway-local event and completion shapes
- surfacing bounded session-handle metadata and bounded adapter failure detail

The adapter protocol does not own:

- the top-level structured event envelope, output-class routing, or correlation fields required for
  concurrent rendering and routing; those remain owned by ADR-0017
- canonical trace vocabulary, join keys, and trace append semantics; those remain owned by
  ADR-0028
- raw provider stream framing; the standalone gateway keeps provider transport normalization behind
  its own structured-event contract

The local-to-external handoff is explicit:

- local adapter work stops at the bounded gateway-local event and completion payloads
- any later envelope wrapping, routing attribution, or trace persistence must defer to ADR-0017
  and ADR-0028 rather than redefining them here

## Boundaries

- This contract does not redefine the stable backend-id grammar or allowlist order. That remains
  owned by `docs/contracts/gateway/backend-adapter-selection.md`.
- This contract consumes the upstream invalid-selection, dependency-unavailable, and policy-denial
  buckets; adapter lookup must not reinterpret backend-id grammar or allowlist policy locally.
- This contract does not widen the machine-readable status boundary. That remains owned by
  `docs/contracts/gateway/status-schema.md`.
- This contract does not define provider-specific routing strategy, planner/executor role splits,
  raw SSE framing, or provider error payloads as public contract truth.
- Repository topology is not part of the protocol contract. Moving the gateway into the Substrate
  monorepo or vendoring related repos through submodules does not change this contract unless the
  normative docs or runtime behavior themselves change.

## Verification Surfaces

The implementation and verification surfaces for this contract are expected to stay aligned across:

- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md`
- `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `crates/gateway/src/adapter_runtime.rs`
- `crates/shell/src/execution/prompt_fulfillment.rs`
- `crates/world-service/src/prompt_fulfillment.rs`
- the Unified Agent API normative specs and backend harnesses cited by ADR-0041
- the standalone gateway structured-event normalization surfaces cited by ADR-0041
