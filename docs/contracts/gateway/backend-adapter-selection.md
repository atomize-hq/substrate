# Substrate Gateway Backend Adapter Selection

This document is the durable canonical contract reference for backend selection at the
Substrate-to-gateway boundary. It records the stable backend-id semantics, trusted-input
boundary, and pre-dispatch failure taxonomy without expanding into adapter protocol,
payload, or parity detail.

## Contract

The owned backend-selection surface covers:

- the stable backend-id grammar used for selection and allowlisting
- the rule that one selected backend id maps to one adapter identity
- the ordered use of existing config, policy, and inventory inputs before adapter dispatch
- the distinction between invalid selection, dependency unavailability, and policy denial
- the rule that gateway-local config, admin, persistence, and session state do not authorize execution

Concrete rules:

- Backend ids use the stable `<kind>:<name>` format.
  - `<kind>`: lowercase ASCII `[a-z0-9_]+`
  - `<name>`: lowercase ASCII `[a-z0-9_-]+`
- The selected backend id is an adapter selector only. It must not be overloaded with router,
  provider, auth-authority, protocol, planner, executor, or wrapper identity.
- One backend id maps to one adapter identity at the Substrate boundary. Internal adapter
  mechanics may vary inside `substrate-gateway`, but the Substrate-facing backend id stays stable.
- Selection consumes the existing ADR-0027 surfaces only:
  - `llm.routing.default_backend`
  - `llm.allowed_backends`
  - the existing one-file-per-backend backend inventory rooted at:
    - `$SUBSTRATE_HOME/agents/<agent_id>.yaml`
    - `<workspace_root>/.substrate/agents/<agent_id>.yaml`
- Effective backend inventory resolution uses the existing ADR-0027 precedence rules for the
  `agents/` inventory roots:
  - workspace inventory overrides global inventory per `id`
  - built-in defaults, if any, remain lower precedence than workspace and global inventory
- Backend inventory identity rules are:
  - the filename-derived `<agent_id>` must match the YAML field `id` exactly
  - the derived backend id is `<kind>:<agent_id>`, where `<kind>` is the inventory item's
    `config.kind`
  - for gateway-backed LLM selection, a backend id is eligible only when it resolves to exactly one
    effective inventory item whose `config.capabilities.llm` is `true`
- Selection order is:
  1. resolve the requested or default backend id from the existing config and inventory surfaces
  2. validate backend-id grammar and inventory identity consistency
  3. apply deny-by-default allowlist policy before adapter dispatch
  4. hand the allowed backend id to the gateway adapter/runtime boundary
- Failure classes remain distinct:
  - invalid selection: malformed backend id, unknown backend id, inventory mismatch, or otherwise invalid selection input
  - dependency unavailable: the selected backend id is valid and allowed, but the required adapter or runtime component is unavailable or unsupported
  - policy denial: policy or safety rules deny execution for the selected backend id or required placement
- Secrets must not appear in backend-id fields or selection surfaces.
- Gateway-local config, admin mutation surfaces, token persistence, and session state are not trusted policy inputs for backend authorization.
- This contract must not introduce a second Substrate control plane.

## Boundaries

- This contract does not define `status --json` fields. That remains owned by
  `docs/contracts/gateway/status-schema.md`.
- This contract does not define policy decision tables or trust-boundary mechanics beyond the
  published selection boundary. That remains owned by
  `docs/contracts/gateway/policy-evaluation.md`.
- This contract does not define adapter request/response payloads, capabilities, extension keys,
  session handles, event envelopes, or trace vocabulary.
- This contract does not widen operator command ownership beyond the existing gateway command family.

## Verification surfaces

The implementation and verification surfaces for this contract are expected to stay aligned across:

- `crates/broker/src/policy.rs`
- `crates/broker/src/policy/tests.rs`
- `crates/broker/src/effective_policy.rs`
- `crates/shell/src/execution/config_model.rs`
- `crates/shell/src/execution/policy_model.rs`
- `docs/reference/policy/contract.md`
- `docs/reference/policy/schema.md`
- `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
