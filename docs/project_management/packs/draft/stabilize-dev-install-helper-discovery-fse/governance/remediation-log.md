# Remediation Log - stabilize-dev-install-helper-discovery

## Open remediations

```yaml
remediation_id: REM-001
origin_phase: pre_exec
source_gate: review
related_seam: SEAM-1
related_slice: null
related_thread: THR-02
related_contract: C-01
related_artifact: crates/shell/src/builtins/world_enable/runner/paths.rs
severity: material
status: open
owner_seam: SEAM-1
blocked_targets: []
summary: Helper-missing remediation text may still assume a production bundle and misdirect dev-install operators when all helper candidates are absent.
required_fix: Confirm or narrow the missing-helper message during seam-local review so the operator guidance matches staged-prefix reality.
resolution_evidence: []
```

```yaml
remediation_id: REM-002
origin_phase: pre_exec
source_gate: revalidation
related_seam: SEAM-3
related_slice: null
related_thread: THR-02
related_contract: C-02
related_artifact: manual_testing_playbook.md
severity: material
status: open
owner_seam: SEAM-3
blocked_targets: []
summary: macOS validation surfaces can overclaim provisioning parity because helper discovery correctness does not guarantee all release-root assets are staged.
required_fix: Keep playbook and parity evidence explicit that macOS scope is limited to helper discovery, validation, and managed cleanup unless additional release-root assets are intentionally added.
resolution_evidence: []
```

Rules:

- Use canonical YAML blocks for remediation entries.
- Use seam ownership only. Do not emit `WS-*` owners.
- For `severity: blocking`, `blocked_targets` must not be empty.
- For `severity: material` or `follow_up`, use `blocked_targets: []` unless a concrete blocked transition also applies.

## Resolved remediations

```yaml
remediation_id: REM-003
origin_phase: pre_exec
source_gate: revalidation
related_seam: SEAM-1
related_slice: null
related_thread: THR-01
related_contract: C-02
related_artifact: scripts/substrate/dev-install-substrate.sh
severity: follow_up
status: resolved
owner_seam: SEAM-1
blocked_targets: []
summary: ADR-0035 shares install-script and helper-script surfaces that can stale the extracted basis before SEAM-1 promotes.
required_fix: Revalidate SEAM-1 and downstream seam bases against any ADR-0035 changes touching shared script surfaces before promotion.
resolution_evidence:
  - 2026-03-30 pre-exec revalidation confirmed the current `scripts/substrate/dev-install-substrate.sh` runtime-bundle surface still matches the SEAM-1 owned contract.
  - 2026-03-30 pre-exec revalidation confirmed `crates/shell/src/builtins/world_enable/runner/paths.rs` still preserves the planned helper-order contract alongside the current `crates/shell/tests/world_enable.rs` coverage.
  - ADR-0035 remains a future stale trigger only if shared install-script or helper-script surfaces change again.
```
