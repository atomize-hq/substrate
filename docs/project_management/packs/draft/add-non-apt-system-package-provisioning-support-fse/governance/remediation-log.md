# Remediation Log - Add non-APT system-package provisioning support

## Open remediations

```yaml
remediation_id: REM-001
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-6
related_slice: null
related_thread: THR-01
related_contract: C-01
related_artifact: docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md
severity: material
status: open
owner_seam: SEAM-6
blocked_targets: []
summary: overlapping ADR and pack docs can still present a second truth for the shared manager-aware provisioning contract until reconciliation lands.
required_fix: reconcile ADR-0033, the APT-pack contract, the bundles contract, and the world/deps docs so they defer to or restate the accepted C-01 contract in one consistent voice.
resolution_evidence: []
```

```yaml
remediation_id: REM-002
origin_phase: pre_exec
source_gate: review
related_seam: SEAM-6
related_slice: null
related_thread: null
related_contract: null
related_artifact: docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md
severity: follow_up
status: open
owner_seam: SEAM-6
blocked_targets: []
summary: Arch-family pacman-success evidence on macOS depends on a manual non-default Lima fixture and can drift if the fixture assumptions are not kept explicit.
required_fix: keep the Arch manual fixture contract explicit in the playbook and closeout evidence or raise a follow-on automation plan without broadening this pack.
resolution_evidence: []
```

Rules:

- Use canonical YAML blocks for remediation entries.
- Use seam ownership only. Do not emit `WS-*` owners.
- For `severity: blocking`, `blocked_targets` must not be empty.
- For `severity: material` or `follow_up`, use `blocked_targets: []` unless a concrete blocked transition also applies.

## Resolved remediations

- Move resolved items here using the same schema, set `status: resolved`, and populate `resolution_evidence`.

```yaml
remediation_id: REM-003
origin_phase: pre_exec
source_gate: revalidation
related_seam: SEAM-4
related_slice: null
related_thread: THR-04
related_contract: C-04
related_artifact: crates/world-agent/src/service.rs
severity: blocking
status: resolved
owner_seam: SEAM-4
blocked_targets:
  - seam: SEAM-4
    field: status
    value: decomposed
summary: adjacent staging and tracing work can stale the provisioning-wiring touch surface before seam-local planning reaches the shared world_enable and world-agent execution paths.
required_fix: revalidate scripts/substrate/world-enable.sh and crates/world-agent/src/service.rs against adjacent packs before decomposing SEAM-4.
resolution_evidence:
  - scripts/substrate/world-enable.sh revalidated during SEAM-4 promotion; helper posture still delegates provisioning follow-up without contradicting in-world manager routing.
  - crates/world-agent/src/service.rs revalidated during SEAM-4 promotion; reserved world-deps request-profile handling remains in place for provisioning wrappers.
  - crates/shell/src/execution/routing/dispatch/world_ops.rs revalidated during SEAM-4 promotion; reserved profile handling still prevents host environment takeover of world-deps provisioning profiles.
  - governance/seam-3-closeout.md now publishes THR-03 and closes the upstream schema handoff required for SEAM-4 basis currentness.
```
