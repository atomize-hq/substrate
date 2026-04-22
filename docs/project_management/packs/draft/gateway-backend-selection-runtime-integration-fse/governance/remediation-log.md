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
- remediation_id: REM-003
  origin_phase: exec
  source_gate: implementation
  related_seam: SEAM-2
  related_slice: S01
  related_thread: THR-02
  related_contract: C-03
  related_artifact: docs/contracts/substrate-gateway-backend-adapter-protocol.md
  severity: medium
  status: deferred
  owner_seam: SEAM-2
  blocked_targets: []
  summary: adapter lookup, capability gating, and missing-binding handling are implementation work under the already-published protocol contract
  required_fix: land runtime behavior and tests that implement the published adapter-resolution and capability-gating rules for more than `cli:codex`
  resolution_evidence: []
```

```yaml
- remediation_id: REM-004
  origin_phase: exec
  source_gate: implementation
  related_seam: SEAM-2
  related_slice: S01
  related_thread: THR-02
  related_contract: C-04
  related_artifact: docs/contracts/substrate-gateway-backend-adapter-schema.md
  severity: medium
  status: deferred
  owner_seam: SEAM-2
  blocked_targets: []
  summary: shared payload and artifact surfaces may need schema hardening to support more than the current `cli_codex` integrated auth path, but that is a `SEAM-2` implementation concern rather than a pack-level contract blocker
  required_fix: widen or normalize the needed shared types only as required to land multi-backend integrated runtime behavior
  resolution_evidence: []
```

```yaml
- remediation_id: REM-005
  origin_phase: post_exec
  source_gate: revalidation
  related_seam: SEAM-3
  related_slice: S01
  related_thread: THR-03
  related_contract: C-05
  related_artifact: docs/contracts/substrate-gateway-runtime-parity.md
  severity: medium
  status: deferred
  owner_seam: SEAM-3
  blocked_targets: []
  summary: the first supported non-`cli:codex` integrated backend baseline is a later validation and rollout decision, not a blocker on the current implementation pack
  required_fix: once a named additional backend is intentionally selected, add parity evidence and rollout proof across Linux/macOS/Windows
  resolution_evidence: []
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
  related_artifact: docs/contracts/substrate-gateway-policy-evaluation.md
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
  related_artifact: docs/contracts/substrate-gateway-backend-adapter-selection.md
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
  related_artifact: docs/contracts/substrate-gateway-policy-evaluation.md
  severity: none
  status: retired
  owner_seam: SEAM-2
  blocked_targets: []
  summary: auth-source precedence is already fixed while carrier choice is explicitly deferred by the policy contract, so choosing env-only, file-only, or a stronger secret-channel carrier is not a current pack blocker
  required_fix: none inside the current execution target
  resolution_evidence:
    - docs/contracts/substrate-gateway-policy-evaluation.md states that auth-source precedence governs handoff content while carrier choice remains separate and current env delivery remains compatible
```
