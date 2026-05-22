# Review Surfaces - gateway-backend-selection-runtime-integration

These diagrams are execution review surfaces for this pack. They capture the decisions and handoffs reviewers need to validate before the pack advances.
They do not, by themselves, satisfy seam-local pre-exec review.
`SEAM-2` now consumes its own seam-local `review.md`, and `SEAM-3` still requires one later.

## R1 - Selected-backend realization flow

```mermaid
flowchart LR
  CFG["effective config<br/>llm.routing.default_backend"] --> SEL["selected backend id"]
  POL["effective policy<br/>allowlists + auth-read gates"] --> GATE["selection and policy gate"]
  INV["backend inventory"] --> GATE
  SEL --> GATE
  GATE -->|"allowed + consistent"| BIND["integrated adapter binding"]
  GATE -->|"invalid / denied / unavailable"| FAIL["typed failure classification"]
  BIND --> RT["adapter-driven runtime realization"]
  RT --> CMD["status | sync | restart"]
```

Review focus:

- confirms the selected-backend source of truth and deny-by-default gate order
- confirms inventory identity and filename consistency checks happen before adapter dispatch
- confirms the landing evidence for those selection rules is carried by `world_gateway_missing_inventory_uses_exit_code_2_before_socket_dispatch`, `world_gateway_inventory_filename_id_mismatch_uses_exit_code_2`, and `world_gateway_allowlist_denial_uses_exit_code_5`
- confirms invalid selection, policy denial, and dependency unavailable remain distinct
- now serves as the upstream handoff that active `SEAM-2` must consume

## R2 - Auth handoff and managed artifact path

```mermaid
flowchart TB
  SRC["authorized auth source<br/>env or host credential file"] --> AUTH["integrated auth handoff"]
  AUTH --> WA["world-service gateway runtime manager"]
  WA --> CFG["rendered runtime config"]
  WA --> MAN["runtime manifest"]
  WA --> LOG["managed stdout/stderr logs"]
  CFG --> GW["substrate-gateway start"]
  MAN --> GW
  LOG --> OPS["operator diagnostics"]
```

Review focus:

- confirms env-primary, file-fallback-only auth precedence is preserved in implementation
- confirms the landing evidence for that rule is carried by `world_gateway_sync_builds_integrated_auth_payload_from_host_auth_file`, `world_gateway_status_prefers_allowed_env_auth_over_host_auth_file`, `world_gateway_status_builds_integrated_auth_payload_from_allowed_env_override`, `world_gateway_host_credential_policy_denials_use_exit_code_5`, `world_gateway_incomplete_env_override_uses_exit_code_2`, and `world_gateway_env_auth_blocked_by_policy_denies_without_file_fallback`
- confirms runtime realization stays inside the existing typed lifecycle boundary
- confirms managed config, manifest, and log artifacts stay part of the runtime-owned execution path
- now anchors `SEAM-2` implementation slices for adapter-driven launch, readiness, and restart behavior

## R3 - Future parity and rollout proof

```mermaid
flowchart LR
  BASE["cli:codex regression floor"] --> MATRIX["validation matrix"]
  ADD["first non-cli:codex integrated backend"] --> MATRIX
  MATRIX --> LNX["Linux proof"]
  MATRIX --> MAC["macOS proof"]
  MATRIX --> WIN["Windows proof"]
  MATRIX --> UNSUP["explicit unsupported-backend outcome"]
  MATRIX --> ROLL["rollout posture"]
```

Review focus:

- confirms parity and rollout are later proof obligations, not current blockers on `SEAM-1` or `SEAM-2`
- confirms Linux/macOS/Windows evidence must verify the landed runtime path rather than define it
- confirms first-additional-backend proof is deferred until a runtime implementation exists and a named backend is intentionally selected
