# Remediation Log - Persist detected Linux distro + pkg manager

## Open remediations

```yaml
remediation_id: REM-001
origin_phase: pre_exec
source_gate: review
related_seam: SEAM-1
related_slice: null
related_thread: THR-01
related_contract: C-02
related_artifact: docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md
severity: blocking
status: open
owner_seam: SEAM-1
blocked_targets:
  - seam: SEAM-1
    field: status
    value: decomposed
summary: ADR-0032 still points at a stale feature directory and leaves dual-authority source paths unresolved for the persistence contract.
required_fix: reconcile ADR-0032 and its related-doc references to the resolved feature directory or record a single authoritative override before seam-local decomposition begins.
resolution_evidence: []
```

```yaml
remediation_id: REM-002
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-3
related_slice: null
related_thread: THR-03
related_contract: C-06
related_artifact: docs/INSTALLATION.md
severity: material
status: open
owner_seam: SEAM-3
blocked_targets: []
summary: operator installation wording still drifts from the accepted canonical path field naming and shared producer contract.
required_fix: update docs/INSTALLATION.md to use the canonical metadata path story schema_version one wording and the four accepted host_state.platform field names.
resolution_evidence: []
```

```yaml
remediation_id: REM-003
origin_phase: pre_exec
source_gate: review
related_seam: SEAM-2
related_slice: null
related_thread: null
related_contract: C-02
related_artifact: scripts/substrate/uninstall-substrate.sh
severity: follow_up
status: open
owner_seam: SEAM-2
blocked_targets: []
summary: hosted uninstaller cleanup still reads a HOME-derived path that is not yet reconciled with the effective-prefix producer semantics tracked by this pack.
required_fix: carry a follow-on cleanup-reader alignment task or pack so the out-of-scope uninstaller path mismatch remains visible without broadening this producer feature.
resolution_evidence: []
```

Rules:

- Use canonical YAML blocks for remediation entries.
- Use seam ownership only. Do not emit `WS-*` owners.
- For `severity: blocking`, `blocked_targets` must not be empty.
- For `severity: material` or `follow_up`, use `blocked_targets: []` unless a concrete blocked transition also applies.

## Resolved remediations

- Move resolved items here using the same schema, set `status: resolved`, and populate `resolution_evidence`.
