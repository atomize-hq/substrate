# SOW: Secret Handoff Into The World Gateway

Status: implementation-oriented draft. This SOW covers the remaining gateway secret-delivery blocker called out in [AGENT_ORCHESTRATION_GAP_MATRIX.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md:119). Host-side policy gating, auth sourcing, and typed `integrated_auth` request payloads already exist. The missing work is the carrier swap inside the world-owned gateway lifecycle so secret values stop living in the in-world gateway process environment by default.

## Objective

Land one production path where Substrate continues to:

- source auth material on the host under existing policy gates,
- send typed auth material to `world-agent` over the existing lifecycle request seam,
- and launch the in-world gateway under Substrate-owned lifecycle control,

but changes the final handoff from:

- secret-bearing child env vars,

to:

- a read-once auth bundle delivered through an inherited one-time FD/pipe channel, with only a non-secret pointer env var exposed to the gateway process.

The required outcome is not a new gateway control plane. The required outcome is that integrated gateway auth delivery becomes honest about the trust boundary: Substrate still owns secret delivery, but the in-world gateway no longer receives secret values in its process environment by default.

## Why This Is Needed

The current integrated gateway path is already substantially landed:

- the shell resolves allowlisted auth material under the existing config/policy surface in [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs:640),
- lifecycle requests already carry typed `integrated_auth` payloads in [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs:354),
- and `world-agent` already validates backend-specific auth expectations in [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:1101).

The remaining mismatch is narrower and more security-sensitive:

1. `world-agent` resolves the integrated auth payload into concrete secret env vars in [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:1097).
2. The managed gateway launch path then injects those values directly into the child process environment in [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:706).
3. The fake managed-gateway binaries used in runtime tests still assert the presence of secret-bearing env vars in [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:1552).

That means the integrated path currently satisfies policy and lifecycle requirements, but not the preferred secret-channel posture already selected in the governing docs:

- [ADR-0023](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0023-in-world-llm-gateway-front-door.md:72) records the additive upgrade from legacy env injection to FD/pipe auth-bundle delivery.
- [ADR-0040](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md:82) keeps host-to-world secret delivery explicitly Substrate-owned.
- [Substrate Gateway Policy Evaluation](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/gateway/policy-evaluation.md:33) already says the carrier should move away from env-based delivery by default.
- [DR-0018](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/_archived/next/llm_gateway_in_world/decision_register.md:683) explicitly selects the auth-bundle FD/pipe direction.
- [Secrets Delivery Channel Rubric](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/system/standards/shared/SECRETS_DELIVERY_CHANNEL_RUBRIC.md:36) says FD/pipe is the default when Substrate spawns both endpoints.

This is now the remaining security-critical last-mile gap for the in-world gateway path.

## Relationship To Existing Decisions

This SOW is intentionally bounded. It consumes existing decisions without reopening them:

- `ADR-0027` remains the source of truth for:
  - `llm.secrets.env_allowed`
  - `agents.host_credentials.read.allowed_backends`
  - backend allowlists
  - host-side secret sourcing precedence
- `ADR-0040` remains the ownership split:
  - Substrate owns host-to-world secret delivery
  - `substrate-gateway` owns in-world runtime internals
- `ADR-0046` remains the integrated runtime realization seam:
  - selected backend resolution
  - adapter binding
  - typed `integrated_auth` payloads
- `DR-0018` and the shared secrets rubric remain the source of truth for:
  - why FD/pipe is preferred,
  - why secret-bearing env vars should stop being the default,
  - and why a non-secret pointer env var is allowed.

Non-goal reminder:

- this slice does not redesign backend selection,
- does not redesign host-side auth sourcing,
- does not widen the operator command family,
- and does not broaden the request contract into a new public auth API.

## Current Relevant Code Surfaces

### Host-side auth sourcing and lifecycle request construction

- [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs:339)
  - resolves the selected backend
  - derives `integrated_auth`
  - builds `GatewayLifecycleRequestV1`
- [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs:640)
  - `resolve_integrated_auth_payload(...)`
  - `resolve_cli_codex_integrated_auth(...)`
  - `resolve_api_env_integrated_auth(...)`

These are already doing the right policy-gated host sourcing work. This SOW does not redesign them.

### World-agent request preparation and current secret-delivery gap

- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1558)
  - validates the typed gateway lifecycle request and carries `integrated_auth` toward the runtime manager
- [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:706)
  - launches the managed gateway child
  - currently sets secret-bearing env vars on that child
- [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:1097)
  - `ResolvedGatewayAuthHandoff`
  - `resolve_integrated_auth_handoff(...)`
  - `resolve_codex_auth_handoff(...)`
  - `resolve_api_env_auth_handoff(...)`

This is the exact seam that must change.

### Operator and policy contracts that must remain true

- [docs/contracts/gateway/operator-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/gateway/operator-contract.md:7)
- [docs/contracts/gateway/policy-evaluation.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/gateway/policy-evaluation.md:25)

This SOW must preserve those contracts while changing only the durable secret carrier between `world-agent` and the in-world gateway process.

## In Scope

- replace secret-bearing gateway child env injection with auth-bundle FD/pipe delivery
- preserve the existing typed `integrated_auth` lifecycle request seam from shell to `world-agent`
- keep host-side secret sourcing and policy gating unchanged
- define the non-secret pointer contract used to tell the gateway which FD/pipe to read
- update runtime launch, restart, and rotation semantics so re-sync/restart delivers a fresh read-once bundle
- update tests and docs so the integrated path proves absence of secret-bearing env vars in the gateway process by default
- Linux-first implementation and validation, with explicit fail-closed posture where equivalent support is not yet available

## Out Of Scope

- redesigning `GatewayLifecycleRequestV1` into a new public API family
- tuple-policy work from `ADR-0042` / `ADR-0043`
- backend-selection redesign from `ADR-0046`
- gateway-local admin/config redesign
- changing the operator command family (`sync|status|restart`)
- changing host-side secret precedence rules
- changing member-runtime placement or toolbox auth delivery

## Required Semantics And Invariants

### 1. Host-side policy and sourcing rules must not change

The auth-bundle slice must preserve the existing rules already encoded in shell policy evaluation:

- allowlisted env auth remains primary when complete,
- host credential file reads remain fallback-only where permitted,
- partial env auth remains invalid integration,
- and blocked env reads remain policy denial.

Carrier choice must not change authorization semantics.

### 2. The existing typed `integrated_auth` request remains the host→world seam

The shell already sends typed `integrated_auth` payloads to `world-agent`. This SOW should treat that existing lifecycle request as the Substrate-owned secret-channel payload.

Required rule:

- do not invent a second public request family just to carry the same auth material.

### 3. Secret values must not be present in the in-world gateway process environment by default

Once this slice lands, the normal integrated gateway path must stop placing values for:

- `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_*`
- `SUBSTRATE_LLM_BACKEND_AUTH_API_*`
- or raw provider env vars such as `OPENAI_API_KEY`

into the managed gateway process environment by default.

### 4. A non-secret pointer env var is allowed

The gateway process may receive one non-secret pointer env var such as:

- `SUBSTRATE_LLM_AUTH_BUNDLE_FD`

Required rule:

- the pointer env var may identify the inherited FD number,
- but it must never contain secret material itself,
- and it must be safe to print/log if surfaced by diagnostics.

### 5. The auth bundle must be read-once and closed promptly

The managed gateway contract must follow the shared secrets rubric:

- `world-agent` writes the bundle once,
- the gateway reads it once,
- the FD/pipe is closed promptly,
- and it is not forwarded to child processes unless an owning spec explicitly requires and documents that propagation.

### 6. Canonical auth field names must remain stable

The auth bundle payload must continue using the canonical `SUBSTRATE_LLM_BACKEND_AUTH_*` field names for the secret-bearing keys even when the values no longer travel as env vars.

That keeps redaction/caps rules uniform across:

- legacy env-based compatibility paths,
- auth-bundle delivery,
- and any later in-world child propagation rules.

### 7. No silent fallback to env injection on the production integrated path

If the selected integrated gateway runtime cannot consume the auth bundle, the integrated lifecycle must fail closed instead of silently reintroducing secret-bearing env vars as the default path.

Legacy env injection may remain only as an explicit compatibility path if deliberately documented and gated. It must not be the default once this slice lands.

### 8. Restart/rotation must re-deliver a fresh bundle

`substrate world gateway restart` and any `sync`-driven replacement path must deliver a fresh auth bundle for the new process instance.

No persisted auth bundle files are permitted.

## Core Implementation Work

### 1. Replace env-var-oriented auth handoff with an auth-bundle-oriented internal model

`ResolvedGatewayAuthHandoff` in [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:1097) is currently modeled as `env_vars`.

This slice should replace that internal model with something closer to:

- canonical auth field names plus secret values,
- suitable for serialization into a read-once bundle payload,
- without assuming the final carrier is env injection.

The exact type name is implementation-defined. The important change is semantic: `world-agent` should resolve auth material into a bundle payload, not into process env assignments.

### 2. Add bundle creation and inherited-FD spawn plumbing in `GatewayRuntimeManager`

The gateway launch path in [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:706) must:

- create a one-time pipe or equivalent FD-backed channel,
- serialize the auth bundle payload into the write end,
- pass the read end to the gateway child process,
- export only the non-secret pointer env var,
- and avoid setting secret-bearing env vars on the child.

The implementation must ensure:

- correct close discipline on both sides,
- correct error handling if the bundle cannot be created or written,
- and cleanup on spawn failure so no dangling open FDs remain.

### 3. Define the managed gateway consumption contract clearly

This repo owns the Substrate side of the lifecycle seam, but the gateway must still know how to consume the bundle.

Required contract:

- if `SUBSTRATE_LLM_AUTH_BUNDLE_FD` is present, the managed gateway must read the auth bundle from that FD,
- load the values into memory only,
- and not require the corresponding secret-bearing env vars to be present.

This SOW does not require a second gateway control API. It requires one clear startup contract.

### 4. Preserve the existing shell-side secret sourcing boundary

The shell-side code in [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs:640) should continue to:

- read allowed env values,
- read allowed credential files where applicable,
- fail closed on incomplete/blocked inputs,
- and send typed `integrated_auth` to `world-agent`.

This slice should not move secret sourcing into the gateway or into gateway-local config.

### 5. Preserve invalid-integration, dependency-unavailable, and policy-denial distinctions

The current gateway policy contract already requires those buckets to remain distinct in [docs/contracts/gateway/policy-evaluation.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/contracts/gateway/policy-evaluation.md:38).

The auth-bundle work must preserve that separation for cases such as:

- missing host-side secret material,
- blocked env or credential-file reads,
- bundle creation/write failure,
- gateway process that does not honor the pointer contract,
- and transient launch/readiness failures.

### 6. Keep gateway runtime artifacts non-secret

Managed logs, status surfaces, and diagnostics may continue to expose:

- runtime log paths,
- backend id,
- lifecycle state,
- and the non-secret pointer contract when necessary.

They must not expose:

- bundle contents,
- secret-bearing field values,
- or secret-bearing env vars that should no longer exist on the child process.

### 7. Update fake gateway launchers and parity tests to prove the new contract

The managed-gateway test launchers in [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:1540) currently assert secret env presence.

Those tests must be rewritten to prove the new posture:

- the pointer env var is present,
- the auth bundle can be read exactly once,
- secret-bearing env vars are absent by default,
- restart delivers a fresh bundle,
- and failure to read the bundle fails the lifecycle cleanly.

## Required Validation Work

### Unit and integration tests

At minimum this slice should add or update tests for:

- shell-side auth resolution still honoring current precedence and policy gates
- `world-agent` auth-bundle rendering for `cli:codex` and `api:*`
- managed gateway launch proving the absence of secret-bearing env vars by default
- managed gateway launch proving the presence of `SUBSTRATE_LLM_AUTH_BUNDLE_FD`
- read-once behavior and prompt close of the bundle FD
- restart/sync rotation delivering a fresh bundle
- fail-closed behavior when the gateway does not consume the bundle or when the bundle cannot be created

Primary test surfaces:

- [crates/shell/tests/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/world_gateway.rs)
- [crates/world-agent/tests/gateway_runtime_parity.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/gateway_runtime_parity.rs)
- [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)

### Documentation and contract updates

This slice should update the docs that currently describe the preferred auth-bundle posture so they match the landed runtime:

- `AGENT_ORCHESTRATION_GAP_MATRIX.md` should stop treating this item as missing once the carrier swap is real.
- any gateway contract docs that still describe env-based integrated delivery as the only concrete behavior should be updated to reflect auth-bundle-by-default.
- test fixture comments that assume `SUBSTRATE_LLM_BACKEND_AUTH_*` values appear in the gateway env must be rewritten.

## Exit Criteria

This slice is complete when all of the following are true on the primary Linux production path:

1. The shell still resolves and sends typed `integrated_auth` under existing policy gates.
2. `world-agent` no longer resolves that auth into child env assignments as the default managed-gateway path.
3. The managed gateway receives auth through a read-once FD/pipe bundle with only a non-secret pointer env var exposed.
4. The managed gateway process environment no longer contains secret-bearing auth values by default.
5. Restart/sync rotation re-delivers fresh auth material without persistence.
6. Tests prove both the happy path and fail-closed posture.
7. The matrix row for `Secret handoff into the world gateway` can be marked landed.

At that point, the remaining v1 gaps are no longer about secure gateway secret delivery. They move back to operator/control-plane follow-ons such as status ambiguity handling and public session controls.
