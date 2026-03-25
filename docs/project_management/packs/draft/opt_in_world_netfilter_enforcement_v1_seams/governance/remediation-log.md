# Remediation Log - Opt-in World Netfilter Enforcement

## Open remediations

```yaml
remediation_id: REM-001
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-1
related_slice: null
related_thread: THR-01
related_contract: C-01
related_artifact: crates/agent-api-types
severity: blocking
status: open
owner_seam: SEAM-1
blocked_targets:
  - seam: SEAM-1
    field: status
    value: decomposed
summary: Define hostname normalization rules for net_allowed
required_fix: Specify canonical hostname normalization (casefolding and IDNA posture) and add tests to prevent drift
resolution_evidence: []
```

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
    value: decomposed
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

## Resolved remediations

- Move resolved items here using the same schema, set `status: resolved`, and populate `resolution_evidence`.

