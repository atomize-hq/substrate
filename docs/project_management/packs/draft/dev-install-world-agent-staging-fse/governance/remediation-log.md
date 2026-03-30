# Remediation Log - dev-install-world-agent-staging

## Open remediations

```yaml
remediation_id: REM-001
origin_phase: pre_exec
source_gate: review
related_seam: SEAM-1
related_slice: null
related_thread: THR-02
related_contract: C-02
related_artifact: crates/shell/src/builtins/world_enable/runner.rs
severity: material
status: open
owner_seam: SEAM-1
blocked_targets: []
summary: Missing-artifact remediation can disappear from the visible CLI path if implementation drifts away from the runner boundary and relies on helper output that is hidden by default.
required_fix: Confirm during seam-local review that the required remediation block is emitted before helper launch in both dry-run and non-dry-run flows and remains visible without relying on helper logs.
resolution_evidence: []
```

```yaml
remediation_id: REM-002
origin_phase: pre_exec
source_gate: revalidation
related_seam: SEAM-2
related_slice: null
related_thread: THR-03
related_contract: C-04
related_artifact: scripts/substrate/install-substrate.sh
severity: material
status: open
owner_seam: SEAM-2
blocked_targets: []
summary: The source pack leaves ambiguity about whether scripts/substrate/install-substrate.sh is an actual changed surface or only a regression boundary guarded by installer smoke.
required_fix: Resolve the production-installer scope during SEAM-2 review and narrow the seam touch set or regression evidence explicitly before decomposition promotes.
resolution_evidence: []
```

```yaml
remediation_id: REM-003
origin_phase: pre_exec
source_gate: revalidation
related_seam: SEAM-3
related_slice: null
related_thread: THR-01
related_contract: C-01
related_artifact: platform-parity-spec.md
severity: follow_up
status: open
owner_seam: SEAM-3
blocked_targets: []
summary: Overlapping helper-discovery and provisioning work can stale the extracted parity basis before checkpoint evidence is trustworthy.
required_fix: Revalidate SEAM-1 and SEAM-2 closeouts against any overlapping packs that touch world-enable or dev-install surfaces before SEAM-3 promotes beyond future planning.
resolution_evidence: []
```

Rules:

- Use canonical YAML blocks for remediation entries.
- Use seam ownership only. Do not emit `WS-*` owners.
- For `severity: blocking`, `blocked_targets` must not be empty.
- For `severity: material` or `follow_up`, use `blocked_targets: []` unless a concrete blocked transition also applies.

## Resolved remediations

None yet. Move resolved items here using the same schema, set `status: resolved`, and populate `resolution_evidence`.
