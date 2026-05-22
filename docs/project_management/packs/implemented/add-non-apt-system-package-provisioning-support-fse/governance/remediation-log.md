# Remediation Log - Add non-APT system-package provisioning support

## Open remediations

None.

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
related_artifact: crates/world-service/src/service.rs
severity: blocking
status: resolved
owner_seam: SEAM-4
blocked_targets:
  - seam: SEAM-4
    field: status
    value: decomposed
summary: adjacent staging and tracing work can stale the provisioning-wiring touch surface before seam-local planning reaches the shared world_enable and world-service execution paths.
required_fix: revalidate scripts/substrate/world-enable.sh and crates/world-service/src/service.rs against adjacent packs before decomposing SEAM-4.
resolution_evidence:
  - scripts/substrate/world-enable.sh revalidated during SEAM-4 promotion; helper posture still delegates provisioning follow-up without contradicting in-world manager routing.
  - crates/world-service/src/service.rs revalidated during SEAM-4 promotion; reserved world-deps request-profile handling remains in place for provisioning wrappers.
  - crates/shell/src/execution/routing/dispatch/world_ops.rs revalidated during SEAM-4 promotion; reserved profile handling still prevents host environment takeover of world-deps provisioning profiles.
  - governance/seam-3-closeout.md now publishes THR-03 and closes the upstream schema handoff required for SEAM-4 basis currentness.
```

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
status: resolved
owner_seam: SEAM-6
blocked_targets: []
summary: overlapping ADR and pack docs could present a second truth for the shared manager-aware provisioning contract until reconciliation landed.
required_fix: reconcile ADR-0033, the APT-pack contract, the bundles contract, and the world/deps docs so they defer to or restate the accepted C-01 contract in one consistent voice.
resolution_evidence:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/platform-parity-spec.md` now names the supported, unsupported, and manual-only lanes without reintroducing APT-only truth.
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh`, `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh`, and `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1` align the smoke assertions with the already-published manager-aware behavior.
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`, `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`, `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`, `docs/reference/world/deps/README.md`, and `docs/internals/world/deps.md` now defer to or restate the accepted `C-01` contract in one voice.
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
status: resolved
owner_seam: SEAM-6
blocked_targets: []
summary: Arch-family pacman-success evidence on macOS depended on a manual non-default Lima fixture and could drift if the fixture assumptions were not kept explicit.
required_fix: keep the Arch manual fixture contract explicit in the playbook and closeout evidence or raise a follow-on automation plan without broadening this pack.
resolution_evidence:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md` section 4 records the Arch-family Lima fixture assumptions explicitly, including the `ID=arch` / `ID_LIKE=arch`, `pacman`, `/usr/local/bin/substrate`, `/usr/local/bin/substrate-world-service`, and `/run/substrate.sock` checks.
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/platform-parity-spec.md` marks the Arch-family pacman-success path as manual-only evidence on macOS and keeps the default-guest smoke boundary explicit.
```
