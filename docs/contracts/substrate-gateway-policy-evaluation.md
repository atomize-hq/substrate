# Substrate Gateway Policy Evaluation

This document is the durable canonical contract reference for Substrate gateway policy evaluation.
It mirrors the feature-local boundary contract and names the owned policy-evaluation surface without
expanding into a full decision matrix.

## Contract

The owned policy-evaluation surface is the gateway-integration flow over existing ADR-0027 inputs.

Owned contract points:
- the no-host-fallback rule when in-world execution is required
- the host-to-world secret delivery boundary
- the precedence rule between allowlisted env auth material and allowlisted host credential file material
- the rule that gateway-local config, admin, and persistence surfaces are not trusted policy inputs
- the distinction between invalid integration state, dependency unavailability, and policy denial
- the rule that policy explanations remain Substrate-owned

Governed inputs:
- `llm.gateway.mode`: determines whether execution must stay in-world or may remain host-only.
- `llm.fail_closed.routing`: determines whether host fallback is allowed when world placement is required.
- `llm.secrets.env_allowed`: determines which host environment variable names may be read for secret delivery.
- `agents.host_credentials.read.allowed_backends`: determines which backend identities may participate in the host-side credential read path.

Concrete rules:
- If `llm.gateway.mode` requires in-world execution and the world boundary is unavailable, the policy result is denial rather than host fallback.
- If `llm.fail_closed.routing` is true, absence of the world boundary is treated as a fail-closed condition.
- Host secret sourcing is only a policy-gated preparation step for Substrate-owned delivery and only for env names and backend identities allowed by the governed inputs above.
- If a selected backend has complete allowlisted env auth material for the integrated handoff, that env material is the authoritative source for the handoff.
- Host credential file reads are permitted only as a host-side fallback when the required env auth handoff is absent; file material must not override, supplement, or merge with an already-present env handoff.
- If env auth material is present but blocked by `llm.secrets.env_allowed`, the result is policy denial; host credential file fallback must not be used to bypass that denial.
- If partial env auth material is present for the selected backend, the request is invalid integration and must fail closed rather than backfilling missing fields from a host credential file read.
- Those precedence rules govern the content of the integrated handoff, not the durable host-to-world carrier. The landed integrated carrier is a Substrate-owned secret-channel auth bundle delivered over an inherited FD (`SUBSTRATE_LLM_AUTH_BUNDLE_FD`), and the in-world gateway must consume that pointer once without depending on secret-bearing child env vars.
- Backend-specific auth renderers may differ, but they must preserve the same precedence and
  fail-closed rules above; carrier choice does not change policy authorization.
- Gateway-local config, admin mutation surfaces, and token persistence remain implementation details of `substrate-gateway`; they do not become trusted policy inputs.
- Policy explanations remain part of the Substrate operator surface and must not be delegated to gateway-local state.
- Invalid integration state, dependency unavailability, and policy denial are separate outcomes and must not be collapsed into a single failure bucket.

Not defined here:
- a full decision table
- exhaustive routing examples
- implementation-specific transport or adapter behavior
- config schema ownership already covered elsewhere
- `client_wiring.*` status-schema detail

## Boundary rules

- fail closed when in-world execution is required and no world boundary is available
- do not trust gateway-local control-plane surfaces as policy inputs
- host secret sourcing and delivery remain policy-gated and Substrate-owned
- treat complete allowlisted env auth material as primary and host credential files as fallback-only when env auth is absent
- do not merge partial env auth material with host credential file reads; fail as invalid integration instead
- keep auth-source precedence separate from carrier choice; the landed integrated transport is the secret-channel FD/auth-bundle path, and carrier choice does not relax precedence or fail-closed rules
- policy evaluation remains part of the Substrate operator surface
- keep invalid integration state distinct from dependency unavailability and policy denial
