# Remediation Log - Opt-in World Netfilter Enforcement

## Open remediations

```yaml
remediation_id: REM-002
origin_phase: pre_exec
source_gate: revalidation
related_seam: SEAM-2
related_slice: null
related_thread: THR-02
related_contract: C-02
related_artifact: crates/world
severity: blocking
status: open
owner_seam: SEAM-2
blocked_targets:
  - seam: SEAM-2
    field: status
    value: exec-ready
summary: Enumerate and harden all process spawn paths for cgroup attach under isolate_network
required_fix: Inventory all execution paths and ensure each attaches to the world cgroup or fails when isolate_network=true
resolution_evidence: []
```

```yaml
remediation_id: REM-003
origin_phase: pre_exec
source_gate: review
related_seam: SEAM-3
related_slice: null
related_thread: THR-03
related_contract: C-04
related_artifact: docs/reference/config/world.md
severity: material
status: open
owner_seam: SEAM-3
blocked_targets: []
summary: Clarify operator workflow for enabling netfilter safely
required_fix: Document the three-way gate alignment (world.net.filter, WORLD_NETFILTER_ENABLE, policy net_allowed) with examples
resolution_evidence: []
```

```yaml
remediation_id: REM-004
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-1
related_slice: S2
related_thread: THR-03
related_contract: C-04
related_artifact: docs/project_management/packs/draft/opt_in_world_netfilter_enforcement_v1_seams/threaded-seams/seam-1-snapshot-v3-net-allowlist-plumbing/slice-2-host-snapshot-and-worldspec-plumbing.md
severity: blocking
status: open
owner_seam: SEAM-3
blocked_targets:
  - seam: SEAM-1
    field: status
    value: exec-ready
summary: SEAM-1 still consumes unpublished C-04 and THR-03 from future SEAM-3, so the active seam basis is only provisional
required_fix: Publish C-04 and THR-03 from SEAM-3 or explicitly resequence/redistribute the config-gating work so SEAM-1 no longer depends on future-seam output before promotion
resolution_evidence: []
```

## Resolved remediations

- Move resolved items here using the same schema, set `status: resolved`, and populate `resolution_evidence`.

```yaml
remediation_id: REM-001
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-1
related_slice: S1
related_thread: THR-01
related_contract: C-01
related_artifact: crates/agent-api-types
severity: blocking
status: resolved
owner_seam: SEAM-1
blocked_targets:
  - seam: SEAM-1
    field: status
    value: exec-ready
summary: Decide and record `net_allowed` hostname normalization rules (casefolding + IDNA posture)
required_fix: Document the canonical normalization posture (ASCII casefolding, trailing-dot handling, IDNA posture) and the verification plan
resolution_evidence:
  - "Contract decision recorded: threaded-seams/seam-1-snapshot-v3-net-allowlist-plumbing/slice-1-publish-net-allowed-contract.md (S1.T1)"
  - "Seam gate disposition updated: threaded-seams/seam-1-snapshot-v3-net-allowlist-plumbing/seam.md + review.md"
```
