# Remediation Log - gateway-backend-selection-runtime-integration

This pack is now execution-focused. Remediations exist only to record the minimum remaining alignment work needed to keep implementation deterministic and to explicitly defer non-blocking follow-ons.

Future remediation entries must use the canonical fields from the extractor governance model:

- `remediation_id`
- `origin_phase`
- `source_gate`
- `related_seam`
- `related_slice`
- `related_thread`
- `related_contract`
- `related_artifact`
- `severity`
- `status`
- `owner_seam`
- `blocked_targets`
- `summary`
- `required_fix`
- `resolution_evidence`

## Open remediations

```yaml
[]
```

## Deferred follow-ons (not pack blockers)

```yaml
[]
```

## Resolved remediations

```yaml
- remediation_id: REM-001
  origin_phase: pre_exec
  source_gate: contract
  related_seam: SEAM-1
  related_slice: S00
  related_thread: THR-01
  related_contract: C-02
  related_artifact: docs/contracts/gateway/policy-evaluation.md
  severity: medium
  status: resolved
  owner_seam: SEAM-1
  blocked_targets: []
  summary: the canonical policy-evaluation contract already pins env-primary, file-fallback-only auth precedence; the remaining work was consumer and supporting-doc alignment behind that published rule
  required_fix: none inside the current execution target
  resolution_evidence:
    - `ca799c1c` landed env-wins and env-blocked-no-fallback auth tests in the shell gateway path
    - `f13e82e9` downgraded stale support-doc references to subordinate material and aligned closeout evidence targets
    - `cargo test -p shell --test world_gateway -- --nocapture`
    - `cargo test -p shell --test agents_validate -- --nocapture`
    - future ADR-0046 support docs remain subordinate implementation notes under the canonical `docs/contracts/` policy truth

- remediation_id: REM-002
  origin_phase: pre_exec
  source_gate: contract
  related_seam: SEAM-1
  related_slice: S00
  related_thread: THR-01
  related_contract: C-01
  related_artifact: docs/contracts/gateway/backend-adapter-selection.md
  severity: medium
  status: resolved
  owner_seam: SEAM-1
  blocked_targets: []
  summary: the selection contract already fixes stable backend ids, one-file-per-backend posture, and filename/id consistency, and the landed shell evidence now proves that implementation alignment
  required_fix: none inside the current execution target
  resolution_evidence:
    - `c12b8fd3` landed the inventory-backed selection gate and shared inventory validation in shell code and tests
    - `f13e82e9` aligned closeout evidence targets and downgraded stale support-doc references to subordinate material
    - `cargo test -p shell --test world_gateway -- --nocapture`
    - `cargo test -p shell --test agents_validate -- --nocapture`
    - any later ADR-0046 support docs remain subordinate to canonical `docs/contracts/` truth rather than expanding the selection surface

- remediation_id: REM-003
  origin_phase: exec
  source_gate: implementation
  related_seam: SEAM-2
  related_slice: S01
  related_thread: THR-02
  related_contract: C-03
  related_artifact: docs/contracts/gateway/backend-adapter-protocol.md
  severity: medium
  status: resolved
  owner_seam: SEAM-2
  blocked_targets: []
  summary: adapter lookup, capability gating, and missing-binding handling are implementation work under the already-published protocol contract
  required_fix: none inside the current execution target
  resolution_evidence:
    - `crates/world-service/src/gateway_runtime.rs` now binds both `cli:codex` and `api:openai` through an explicit runtime registry with binding-driven config render and auth injection
    - `crates/world-service/tests/gateway_runtime_parity.rs` now proves `api:openai` through unavailable-before-sync, sync/status/idempotent, restart, manifest recovery, and explicit no-fallback behavior
    - `cargo test -p world-service --lib -- --nocapture`
    - `limactl shell substrate -- bash -lc 'cd /Users/spensermcconnell/__Active_Code/atomize-hq/substrate && CARGO_TARGET_DIR=/tmp/substrate-target cargo test -p world-service --test gateway_runtime_parity -- --nocapture'`

- remediation_id: REM-004
  origin_phase: exec
  source_gate: implementation
  related_seam: SEAM-2
  related_slice: S01
  related_thread: THR-02
  related_contract: C-04
  related_artifact: docs/contracts/gateway/backend-adapter-schema.md
  severity: medium
  status: resolved
  owner_seam: SEAM-2
  blocked_targets: []
  summary: shared payload and artifact surfaces needed schema hardening to support more than the current `cli_codex` integrated auth path
  required_fix: none inside the current execution target
  resolution_evidence:
    - `crates/transport-api-types/src/lib.rs` now hardens `GatewayLifecycleRequestV1` with `deny_unknown_fields` and adds the closed backend-neutral `api_env` auth facet beside `cli_codex`
    - `crates/world-service/src/service.rs` now validates request-provided auth at the shared boundary before runtime execution
    - `crates/shell/src/builtins/world_gateway.rs` now emits backend-aware integrated auth from resolved inventory instead of suppressing all non-`cli:codex` auth handoff
    - `cargo test -p transport-api-types -- --nocapture`
    - `cargo test -p shell --test world_gateway -- --nocapture`

- remediation_id: REM-005
  origin_phase: post_exec
  source_gate: revalidation
  related_seam: SEAM-3
  related_slice: S01
  related_thread: THR-03
  related_contract: C-05
  related_artifact: docs/contracts/gateway/runtime-parity.md
  severity: medium
  status: resolved
  owner_seam: SEAM-3
  blocked_targets: []
  summary: the first supported non-`cli:codex` integrated backend baseline is now fixed as `api:openai`, with landed parity and rollout evidence across Linux/macOS/Windows
  required_fix: none inside the current implementation pack
  resolution_evidence:
    - `0b372f6e` made the supported backend matrix explicit in the runtime and shell parity suites
    - `cdf5db5d` landed platform parity, manual testing, and smoke evidence under the `-fse` pack
    - `3b8a988a` landed the compatibility publication surface bounded to `cli:codex`, `api:openai`, and explicit unsupported-backend handling
    - `4822e70a` published the seam-exit gate and `THR-03` closeout record
    - `cargo test -p shell --test world_gateway -- --nocapture`
    - `SUBSTRATE_BIN=target/debug/substrate bash docs/project_management/packs/draft/gateway-backend-selection-runtime-integration-fse/smoke/macos-smoke.sh`
```

## Retired remediations

```yaml
- remediation_id: REM-006
  origin_phase: pre_exec
  source_gate: contract
  related_seam: SEAM-2
  related_slice: S00
  related_thread: THR-02
  related_contract: C-04
  related_artifact: docs/contracts/gateway/policy-evaluation.md
  severity: none
  status: retired
  owner_seam: SEAM-2
  blocked_targets: []
  summary: auth-source precedence is already fixed while carrier choice is explicitly deferred by the policy contract, so choosing env-only, file-only, or a stronger secret-channel carrier is not a current pack blocker
  required_fix: none inside the current execution target
  resolution_evidence:
    - docs/contracts/gateway/policy-evaluation.md states that auth-source precedence governs handoff content while carrier choice remains separate and current env delivery remains compatible
```
