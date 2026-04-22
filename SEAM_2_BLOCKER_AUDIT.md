# SEAM-2 Blocker Audit

Date: 2026-04-22
Scope: source-inspection audit only; no code changes, no test execution
Pack: `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse`

## Purpose

This document captures step 1 of the `SEAM-2` unblock path: a concrete audit of why `THR-02` is still unpublished and why `SEAM-3` promotion is correctly blocked on the current tree.

## Files audited

- [SEAM-2 brief](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/seam-2-runtime-realization-and-artifacts.md)
- [threading.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threading.md)
- [SEAM-2 seam.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/seam.md)
- [SEAM-2 review.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/review.md)
- [slice-1-binding-lookup-and-capability-gates.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/slice-1-binding-lookup-and-capability-gates.md)
- [slice-2-request-auth-and-runtime-artifacts.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/slice-2-request-auth-and-runtime-artifacts.md)
- [slice-3-lifecycle-conformance-and-drift-guards.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/slice-3-lifecycle-conformance-and-drift-guards.md)
- [slice-99-seam-exit-gate.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/slice-99-seam-exit-gate.md)
- [seam-2-closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/seam-2-closeout.md)
- [seam-3-closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/seam-3-closeout.md)
- [remediation-log.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/remediation-log.md)
- [crates/world-agent/src/gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs)
- [crates/world-agent/src/service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs)
- [crates/agent-api-types/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs)
- [crates/shell/src/builtins/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs)
- [crates/world-agent/tests/gateway_runtime_parity.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/gateway_runtime_parity.rs)
- [crates/shell/tests/world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/world_gateway.rs)

## Executive summary

`SEAM-3` is blocked for a real upstream reason, not because the promotion skill lacks context. The pack requires `SEAM-2` to publish `THR-02` before `SEAM-3` can start consuming runtime truth, and the current closeout explicitly says that handoff is still unpublished because the runtime path remains effectively single-backend and Codex-specific.

The blocker is not one bug. It is a four-part publication gap:

1. Runtime binding lookup is still `cli:codex`-only.
2. Shared request/auth schema is still `cli_codex`-only and not fully bounded.
3. Shell request construction still suppresses integrated auth for non-Codex backends.
4. Current tests prove the Codex path and explicit missing-binding behavior, but they do not prove a publishable multi-backend runtime handoff.

There is also governance drift in the current closeout/remediation state that should be cleaned up once implementation catches up.

## Why SEAM-3 is blocked

The pack says the next seam does not start until `SEAM-2` publishes `THR-02` and records closeout evidence in [threading.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threading.md:8) and [threading.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threading.md:14). The current `SEAM-2` closeout says the exact opposite of a ready handoff:

- `seam_exit_gate.status: blocked` and `promotion_readiness: blocked` in [seam-2-closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/seam-2-closeout.md:5)
- `gates.post_exec.landing: blocked` in [seam-2-closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/seam-2-closeout.md:18)
- `THR-02` remains unpublished because the integrated runtime still treats `cli:codex` as the only bound and authenticated integrated backend in [seam-2-closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/seam-2-closeout.md:27)

`SEAM-3` repeats that it remains blocked until `THR-02` publishes and a credible runtime proof target exists in [seam-3-closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/seam-3-closeout.md:35).

## Primary blockers

### 1. Runtime binding lookup is still single-backend

This is the core `S1` miss. `slice-1` requires runtime binding resolution to consume the selected backend id instead of treating `cli:codex` as the only integrated binding in [slice-1-binding-lookup-and-capability-gates.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/slice-1-binding-lookup-and-capability-gates.md:55).

Current code still hardcodes the binding table:

- `GatewayIntegratedAuthKind` only has `CliCodex` in [gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:109)
- `CLI_CODEX_BACKEND_BINDING` is the only binding in [gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:125)
- `resolve_gateway_backend_binding()` returns `Some` only for `DEFAULT_BACKEND` in [gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:148)

This matches the blocker already recorded in [seam-2-closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/seam-2-closeout.md:67).

Impact:

- Direct `THR-02` publication blocker
- Prevents `SEAM-2` from claiming it has landed adapter-driven runtime realization
- Makes `SEAM-3` parity work downstream of an unpublished runtime contract in practice, even if the docs already exist canonically

### 2. Runtime auth handoff is still Codex-only

The runtime handoff remains backend-specific. `resolve_integrated_auth_handoff()` only dispatches `GatewayIntegratedAuthKind::CliCodex` and only consumes `auth.cli_codex` in [gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:890).

This conflicts with the `S2.T1` requirement to widen runtime-owned request/auth shapes without reopening selection or policy ownership in [slice-2-request-auth-and-runtime-artifacts.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/slice-2-request-auth-and-runtime-artifacts.md:57).

This blocker is also explicitly called out in [seam-2-closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/seam-2-closeout.md:70).

Impact:

- Direct `THR-02` publication blocker
- Means runtime binding may accept a selected backend id upstream, but runtime auth semantics still collapse to one backend implementation

### 3. Shared request/auth schema is still `cli_codex`-specific

The shared schema remains too narrow for `THR-02` publication:

- `GatewayIntegratedAuthPayloadV1` contains `backend_id` plus optional `cli_codex` only in [lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs:836)

That directly conflicts with the `S2.T1` acceptance criteria in [slice-2-request-auth-and-runtime-artifacts.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/slice-2-request-auth-and-runtime-artifacts.md:40).

The closeout already records this as blocked in [seam-2-closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/seam-2-closeout.md:68).

Impact:

- Direct `THR-02` publication blocker
- Prevents the pack from honestly claiming the shared lifecycle request can carry bounded auth for more than one supported integrated backend

### 4. The outer lifecycle request is not fully schema-bounded

The seam review and slice plan both talk about a bounded lifecycle request at the protocol boundary in [review.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/review.md:31) and [slice-2-request-auth-and-runtime-artifacts.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/slice-2-request-auth-and-runtime-artifacts.md:38). But `GatewayLifecycleRequestV1` does not use `#[serde(deny_unknown_fields)]` in [lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/agent-api-types/src/lib.rs:842).

Impact:

- Publication blocker if `THR-02` is supposed to freeze the request surface, not just the semantics of the current fields
- Leaves drift room at the exact boundary the seam says it is stabilizing

### 5. Request preparation forwards structurally incomplete auth payloads

`prepare_gateway_runtime_request()` only checks whether `backend_id` is present and matches the selected backend; otherwise it clones the payload and forwards it in [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1455).

That means a payload can be structurally incomplete and still survive request preparation, which conflicts with `S2.T1`’s requirement that incomplete request-provided auth fail in the bounded invalid-request/invalid-integration bucket in [slice-2-request-auth-and-runtime-artifacts.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/slice-2-request-auth-and-runtime-artifacts.md:75).

Impact:

- Direct `THR-02` publication blocker
- Means request-validation semantics are still weaker than the seam plan says they need to be

### 6. Shell request construction still suppresses non-Codex integrated auth

Shell-side request building still enforces the single-backend model:

- `resolve_integrated_auth_payload()` returns `None` for any selected backend except `cli:codex` in [world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/builtins/world_gateway.rs:345)

That conflicts with `S2.T1`’s requirement that the shell handoff remain backend-aware without re-owning selection logic in [slice-2-request-auth-and-runtime-artifacts.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/slice-2-request-auth-and-runtime-artifacts.md:57).

The closeout already records this in [seam-2-closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/seam-2-closeout.md:69).

Impact:

- Direct `THR-02` publication blocker
- Means even if the runtime accepted generalized auth, the shell still would not send it for non-Codex backends

## What is already landed

Not everything is blocked. Several `SEAM-2` pieces are real and should be preserved while unblocking:

- Request preparation now preserves the selected backend id and rejects mismatched auth backend ids in [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1452)
- Missing-binding behavior is explicit and no longer silently falls back once the shell selected a backend in [service.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/service.rs:1493)
- Required-capability gating is pre-spawn and deterministic in [gateway_runtime.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/gateway_runtime.rs:942)
- The runtime path already has managed manifest/recovery/readiness/restart semantics for the current bound path, which is why `S3` is partially landed for Codex

These are meaningful partials, but they do not add up to a publishable `THR-02`.

## Test evidence: what it proves vs what it does not prove

### What current tests do prove

The runtime suite proves the current `cli:codex` lifecycle path across status/sync/restart, manifest recovery, timeout cleanup, and transient behavior in [gateway_runtime_parity.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/gateway_runtime_parity.rs:510).

It also proves explicit missing-binding behavior for non-Codex backends in [gateway_runtime_parity.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/gateway_runtime_parity.rs:530).

The shell suite proves:

- explicit unavailable/transient/policy/invalid-integration exit mapping in [world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/world_gateway.rs:603)
- Codex auth payload construction from file/env in [world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/world_gateway.rs:892)
- selected-backend continuity for non-Codex lifecycle requests in [world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/world_gateway.rs:1044)

### What current tests do not prove

They do not prove a publishable, generalized runtime handoff:

- no supported non-Codex backend binds, authenticates, syncs, reports status, restarts, and recovers end to end
- auth payload widening beyond `cli_codex` is not covered
- artifact-contract details beyond basic manifest recovery are not covered
- unsupported-backend lifecycle behavior is not covered across the full status/sync/restart matrix

Concrete evidence gaps:

- non-Codex runtime test fixtures explicitly strip auth in [gateway_runtime_parity.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/tests/gateway_runtime_parity.rs:66)
- shell tests only assert Codex auth payload details in [world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/world_gateway.rs:892)
- the generic-backend shell lifecycle test asserts `/integrated_auth == None` in [world_gateway.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/world_gateway.rs:1044), which locks in the current blocker behavior instead of proving the planned widening

## Governance drift

Two documentation inconsistencies showed up in the audit:

### 1. `seam-2-closeout.md` frontmatter disagrees with its body

The frontmatter says `open_remediations: []` in [seam-2-closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/seam-2-closeout.md:22), but later the same file says `REM-003` and `REM-004` are unresolved and carried forward in [seam-2-closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/seam-2-closeout.md:78).

### 2. `remediation-log.md` and `seam-2-closeout.md` disagree on blocker posture

The remediation log classifies `REM-003` and `REM-004` as deferred follow-ons with `blocked_targets: []` and says they are not pack blockers in [remediation-log.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/remediation-log.md:29). But `seam-2-closeout.md` uses the same unresolved items as justification for blocked landing and blocked promotion in [seam-2-closeout.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/governance/seam-2-closeout.md:63).

Interpretation:

- the implementation blockers are real
- the current governance vocabulary around whether they are “deferred follow-ons” or “seam-exit blockers” is not internally consistent

## Concrete checklist for unblocking SEAM-2

`THR-02` becomes publishable only when these facts are true:

- adapter lookup is no longer `cli:codex`-only
- runtime auth dispatch is no longer `cli_codex`-only
- shared request/auth schema is widened beyond one backend facet
- the lifecycle request is bounded at deserialization
- request preparation rejects incomplete request-provided auth
- shell request construction emits backend-aware integrated auth instead of suppressing it for non-Codex backends
- tests prove one supported non-Codex backend through the full lifecycle path
- closeout can honestly change from blocked to published and record `THR-02` as published, as required by [slice-99-seam-exit-gate.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/threaded-seams/seam-2-runtime-realization-and-artifacts/slice-99-seam-exit-gate.md:37)

## Recommended follow-on from this audit

If the next step is implementation, the clean execution order is:

1. Fix `S1` runtime binding generalization in `gateway_runtime.rs`
2. Fix `S2` shared schema and request validation in `agent-api-types` + `service.rs`
3. Fix shell request construction in `world_gateway.rs`
4. Expand runtime and shell tests so they prove a generalized handoff rather than Codex-only plus missing-binding behavior
5. Reconcile governance state once the code and tests justify publishing `THR-02`
