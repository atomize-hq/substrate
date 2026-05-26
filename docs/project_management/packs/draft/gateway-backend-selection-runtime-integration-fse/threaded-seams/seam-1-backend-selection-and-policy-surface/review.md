---
seam_id: SEAM-1
review_phase: pre_exec
execution_horizon: active
basis_ref: seam.md#basis
---
# Review Bundle - SEAM-1 Backend selection and policy surface

This artifact feeds `gates.pre_exec.review`.
`../../review_surfaces.md` is pack orientation only.

## Falsification questions

- Can the gateway lifecycle still authorize or choose a backend from gateway-local persistence, admin state, or any surface outside the existing ADR-0027 config and policy inputs?
- Can shell validation still skip the published inventory and selection rules badly enough that `SEAM-2` would have to invent runtime-facing behavior locally?
- Can env material and host credential files both remain allowed without one explicit precedence rule, causing the current `cli:codex` branch to become accidental contract truth?

## R1 - Selected-backend gate that must land

```mermaid
flowchart LR
  CFG["existing config<br/>llm.routing.default_backend"] --> SEL["resolve selected backend id"]
  INV["published inventory roots<br/>and filename/id rules"] --> SEL
  SEL --> VALID["validate grammar + inventory identity"]
  POLICY["effective policy<br/>llm.allowed_backends"] --> GATE["deny-by-default allowlist gate"]
  VALID --> GATE
  GATE -->|"allowed"| OUT["allowed backend id for SEAM-2"]
  GATE -->|"denied"| DENY["policy-denial outcome"]
  VALID -->|"malformed / unknown / mismatch"| INVALID["invalid-integration outcome"]
```

## R2 - Auth precedence and trusted-input boundary that must land

```mermaid
flowchart TB
  ENV["allowlisted env material"] --> PRECEDENCE["explicit precedence rule"]
  FILE["allowlisted host credential file"] --> PRECEDENCE
  PRECEDENCE --> AUTH["Substrate-owned auth handoff input"]
  POLICY["llm.gateway.mode + fail_closed + host credential read gates"] --> AUTH
  AUTH --> RT["runtime realization boundary (SEAM-2)"]
  GWLOCAL["gateway-local config / persistence / admin"] -.not trusted.-> AUTH
```

## Likely mismatch hotspots

- `docs/contracts/gateway/backend-adapter-selection.md` already publishes inventory roots, filename/id invariants, and selection order, and the landed shell evidence for that rule is now split across `world_gateway_missing_inventory_uses_exit_code_2_before_socket_dispatch`, `world_gateway_inventory_filename_id_mismatch_uses_exit_code_2`, and `world_gateway_allowlist_denial_uses_exit_code_5`.
- `docs/contracts/gateway/policy-evaluation.md` already publishes env-primary precedence, and the landed shell evidence for that rule is now split across `world_gateway_sync_builds_integrated_auth_payload_from_host_auth_file`, `world_gateway_status_prefers_allowed_env_auth_over_host_auth_file`, `world_gateway_status_builds_integrated_auth_payload_from_allowed_env_override`, `world_gateway_host_credential_policy_denials_use_exit_code_5`, and `world_gateway_incomplete_env_override_uses_exit_code_2`.
- `crates/shell/src/builtins/world_gateway.rs` now proves inventory-backed shell validation at the boundary, and any remaining runtime-owned cases are limited to adapter binding, capability, or availability questions outside this seam.
- Any future ADR-0046 support docs under `docs/project_management/packs/draft/gateway-backend-selection-runtime-integration/` must remain subordinate implementation notes, not current canonical surfaces.

## Pre-exec findings

- The review gate passes. The selected-backend and auth-boundary diagrams still expose falsifiable product-facing flows, and the out-of-scope line against tuple/status widening remains explicit.
- The contract gate passes. Canonical `C-01` and `C-02` already publish the selection, inventory, precedence, and fail-closed rules this seam needs.
- `REM-001` and `REM-002` remain only as seam-exit follow-through: the canonical contracts already publish the rules; the remaining work is shell adoption, supporting-doc alignment, and landed evidence capture before closeout can publish `THR-01`.
- Revalidation passes against current repo evidence:
  - `crates/shell/src/builtins/world_gateway.rs` keeps invalid integration, policy denial, transient runtime failure, and component unavailability distinct at the shell boundary.
  - `crates/shell/src/builtins/world_gateway.rs` still enforces fail-closed posture for disabled or host-only gateway lifecycle use before dispatch.
  - `crates/shell/src/builtins/world_gateway.rs` prefers allowlisted env auth material when an access token is present and falls back to the allowlisted host credential file only when env auth is absent; partial env material still fails as invalid integration.
  - `crates/shell/tests/world_gateway.rs` gives the seam concrete drift-guard evidence through `world_gateway_missing_inventory_uses_exit_code_2_before_socket_dispatch`, `world_gateway_inventory_filename_id_mismatch_uses_exit_code_2`, `world_gateway_allowlist_denial_uses_exit_code_5`, `world_gateway_sync_builds_integrated_auth_payload_from_host_auth_file`, `world_gateway_status_prefers_allowed_env_auth_over_host_auth_file`, `world_gateway_status_builds_integrated_auth_payload_from_allowed_env_override`, `world_gateway_host_credential_policy_denials_use_exit_code_5`, `world_gateway_incomplete_env_override_uses_exit_code_2`, and `world_gateway_env_auth_blocked_by_policy_denies_without_file_fallback`.
- No blocking pre-exec remediations remain open against the `decomposed -> exec-ready` transition, so the seam is ready to execute even though seam-exit publication work is still pending.
- No new pre-exec remediation is opened by this review refresh. The missing work is implementation and evidence, not fresh contract publication.
- The likely failure mode is downstream runtime work inheriting too much shell-owned validation from the current `cli:codex` path.

## Pre-exec gate disposition

- **Review gate**: passed
- **Contract gate**: passed
- **Revalidation gate**: passed
- **Revalidation evidence**:
  - the latest shell gateway implementation still matches the documented selection boundary and failure buckets before execution starts
  - no external upstream closeout or contract publication changed this seam's basis outside the planned stale triggers
- **Opened remediations**:
  - none
- **Carried seam-exit follow-through**:
  - `REM-001`
  - `REM-002`

## Planned seam-exit gate focus

- **What must be true before downstream promotion is legal**:
  - shell behavior and shell tests demonstrably adopt published `C-01` and `C-02`
  - `THR-01` is recorded as `published` in `../../governance/seam-1-closeout.md`
  - any review-surface delta from the planned selection/policy flow is captured as a stale trigger for `SEAM-2` and `SEAM-3`
- **Which outbound contracts/threads matter most**:
  - `C-01`
  - `C-02`
  - `THR-01`
- **Which review-surface deltas would force downstream revalidation**:
  - changes to shell-owned selection or inventory validation behavior
  - changes to auth precedence or host-fallback behavior
  - changes to invalid-integration versus policy-denial versus runtime-unavailable classification
