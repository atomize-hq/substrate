# Remediation Log - Substrate gateway backend adapter contract

## Open remediations

```yaml
remediation_id: REM-001
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-1
related_slice: null
related_thread: THR-01
related_contract: C-02
related_artifact: docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/workstream_triage.md
severity: blocking
status: resolved
owner_seam: SEAM-1
blocked_targets: []
summary: v1 gateway status publication remains limited to the existing `status` plus `client_wiring.*` schema, and any future additive adapter-visible field family must be introduced by the status-schema owner before runtime models widen.
required_fix: keep `docs/contracts/substrate-gateway-status-schema.md` as the owner for the machine-readable status field list and require an explicit schema-owner update before any additive adapter-visible status field family ships.
resolution_evidence:
  - docs/contracts/substrate-gateway-status-schema.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/contract.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/policy-spec.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threaded-seams/seam-1-adapter-selection-boundary/slice-00-c-01-c-02-contract-definition.md
```

```yaml
remediation_id: REM-002
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-2
related_slice: null
related_thread: THR-02
related_contract: C-04
related_artifact: docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md
severity: material
status: carried_forward
owner_seam: SEAM-2
blocked_targets: []
summary: the adopted Unified Agent API subset is now pinned to a bounded cross-backend execution baseline, but landing and closeout still need to keep the capability set, extension-key subset, session-handle facet, and bounded adapter error detail aligned with that baseline.
required_fix: keep `docs/contracts/substrate-gateway-backend-adapter-schema.md` and `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md` aligned as implementation lands, and verify the unified-agent-api capability, session-handle, selector-validation, cancellation, and runtime-rejection suites still match the adopted subset before closeout publishes `THR-02`.
resolution_evidence:
  - docs/contracts/substrate-gateway-backend-adapter-schema.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-schema-spec.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threaded-seams/seam-2-adapter-protocol-and-schema/slice-00-c-03-c-04-contract-definition.md
  - /Users/spensermcconnell/atomize-hq/unified-agent-api/docs/specs/unified-agent-api/capability-matrix.md
  - /Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/codex/tests/capabilities.rs
  - /Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/claude_code/tests/capabilities.rs
  - /Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/codex/tests/session_handle.rs
  - /Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/claude_code/tests/session_handle.rs
  - /Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/session_selectors.rs
```

```yaml
remediation_id: REM-003
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-2
related_slice: null
related_thread: THR-02
related_contract: C-03
related_artifact: docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md
severity: material
status: carried_forward
owner_seam: SEAM-2
blocked_targets: []
summary: the local-to-external owner line is now pinned: gateway-local adapter translation stops at bounded gateway-local event and completion shapes, while ADR-0017 and ADR-0028 remain the external owners of structured-event envelope and canonical trace semantics.
required_fix: keep `docs/contracts/substrate-gateway-backend-adapter-protocol.md` and `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md` aligned as implementation lands, and verify the unified-agent-api harness ownership tests plus the standalone gateway normalized-event surfaces still respect that owner line before closeout publishes `THR-02`.
resolution_evidence:
  - docs/contracts/substrate-gateway-backend-adapter-protocol.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/gateway-backend-adapter-protocol-spec.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threaded-seams/seam-2-adapter-protocol-and-schema/slice-00-c-03-c-04-contract-definition.md
  - /Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/codex/tests/backend_contract.rs
  - /Users/spensermcconnell/atomize-hq/unified-agent-api/crates/agent_api/src/backends/claude_code/tests/backend_contract.rs
  - /Users/spensermcconnell/__Active_Code/kimi-claude-adapter/gateway/src/structured_events.rs
  - docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md
  - docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md
```

```yaml
remediation_id: REM-004
origin_phase: pre_exec
source_gate: revalidation
related_seam: SEAM-3
related_slice: null
related_thread: null
related_contract: null
related_artifact: docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md
severity: blocking
status: resolved
owner_seam: SEAM-3
blocked_targets: []
summary: ADR-0040 is now explicitly confirmed as prerequisite boundary evidence for SEAM-3 rather than as a direct touch surface, so parity and compatibility work can execute against the existing runtime-ownership owner line without reopening the ADR text.
required_fix: keep ADR-0040 as evidence-only basis for runtime ownership and reopen it only if downstream parity or compatibility proof discovers a concrete ownership drift that the current owner line no longer explains.
resolution_evidence:
  - docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md
  - docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md
  - docs/contracts/substrate-gateway-runtime-parity.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threaded-seams/seam-3-parity-and-validation/review.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threaded-seams/seam-3-parity-and-validation/seam.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threaded-seams/seam-3-parity-and-validation/slice-2-compatibility-proof-and-adr-0040-decision.md
```

```yaml
remediation_id: REM-005
origin_phase: pre_exec
source_gate: review
related_seam: SEAM-1
related_slice: null
related_thread: null
related_contract: null
related_artifact: docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md
severity: material
status: resolved
owner_seam: SEAM-1
blocked_targets: []
summary: ADR-0041 now points at the implemented `packs/implemented/llm_and_agent_config_policy_surface/*` paths, resolving the authority-path drift that previously confused downstream planning.
required_fix: replaced the stale `packs/active/...` references with the current `packs/implemented/...` paths and recorded the exit-gate evidence.
resolution_evidence:
  - docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/governance/seam-1-closeout.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threading.md
```

```yaml
remediation_id: REM-006
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-1
related_slice: S00
related_thread: THR-01
related_contract: C-01
related_artifact: docs/contracts/substrate-gateway-backend-adapter-selection.md
severity: blocking
status: resolved
owner_seam: SEAM-1
blocked_targets: []
summary: the canonical backend-selection baseline now exists and the seam-local execution checklist points downstream work at that durable contract instead of at planning-pack-only prose.
required_fix: keep `docs/contracts/substrate-gateway-backend-adapter-selection.md` aligned with the pack-local `contract.md` and `policy-spec.md` as the implementation checklist executes.
resolution_evidence:
  - docs/contracts/substrate-gateway-backend-adapter-selection.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/contract.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/policy-spec.md
  - docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/threaded-seams/seam-1-adapter-selection-boundary/slice-00-c-01-c-02-contract-definition.md
```

## Resolved remediations

- `REM-001`
- `REM-004`
- `REM-005`
- `REM-006`
