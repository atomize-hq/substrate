# Threading - Add non-APT system-package provisioning support

## Execution horizon summary

- **Active seam**: `SEAM-6`
- **Next seam**: none
- **Future seams**: none
- **Horizon inference**:
  - `SEAM-5` has landed, published `THR-05`, and moved out of the forward planning window.
  - `SEAM-6` is now the active terminal seam because `THR-01` through `THR-05` are landed or revalidated and the prior active seam handed off with a passed seam-exit gate.
  - No queued `next` seam remains in this pack because `SEAM-6` is the terminal conformance seam.
- **Governance-only lineage**:
  - `NASP-PWS-tasks_checkpoints` is intentionally represented as pack governance only. It is not a seam and does not own product behavior.

## Contract registry

- **Contract ID**: `C-01`
  - **Type**: UX affordance
  - **Owner seam**: `SEAM-1`
  - **Direct consumers**:
    - `SEAM-2`
    - `SEAM-3`
    - `SEAM-4`
    - `SEAM-5`
    - `SEAM-6`
  - **Derived consumers**:
    - operators
    - support/docs maintainers
    - overlapping APT and bundles contract packs
  - **Thread IDs**:
    - `THR-01`
  - **Definition**:
    - Shared manager-aware operator contract for `substrate world enable --provision-deps` and runtime `substrate world deps current sync|install`, including authority handoff, exit-code posture, safety invariants, request-profile boundaries, and v1 pacman scope.
  - **Versioning / compat**:
    - Additive manager-aware expansion over the older APT-only contract surfaces; no new config/env/protocol fields.

- **Contract ID**: `C-02`
  - **Type**: state
  - **Owner seam**: `SEAM-2`
  - **Direct consumers**:
    - `SEAM-4`
    - `SEAM-6`
  - **Derived consumers**:
    - provisioning logs and operator-visible backend diagnostics
  - **Thread IDs**:
    - `THR-02`
  - **Definition**:
    - Deterministic in-world world-manager probe and support-gate outcomes based on `/etc/os-release`, `ID_LIKE`, and in-world package-manager availability, with contradiction and unsupported handling.
  - **Versioning / compat**:
    - No host-detection fallback is allowed; only `apt`, `pacman`, or fail-closed unsupported outcomes are valid.

- **Contract ID**: `C-03`
  - **Type**: schema
  - **Owner seam**: `SEAM-3`
  - **Direct consumers**:
    - `SEAM-4`
    - `SEAM-5`
    - `SEAM-6`
  - **Derived consumers**:
    - world-deps inventory authors
    - inventory validation and list/show surfaces
  - **Thread IDs**:
    - `THR-03`
  - **Definition**:
    - Additive inventory/schema contract for `install.method=pacman`, `install.pacman`, mutual-exclusion rules, non-runnable v1 pacman constraints, and inventory-view rendering.
  - **Versioning / compat**:
    - Additive-only on `version: 1`; no translation layer, no schema-version bump, no remap into `apt`, `script`, or `manual`.

- **Contract ID**: `C-04`
  - **Type**: UX affordance
  - **Owner seam**: `SEAM-4`
  - **Direct consumers**:
    - `SEAM-5`
    - `SEAM-6`
  - **Derived consumers**:
    - provisioning operators
    - world-agent/request-profile maintainers
  - **Thread IDs**:
    - `THR-04`
  - **Definition**:
    - Provisioning-time requirement normalization, mixed-manager rejection, request-profile boundary, no-op detection, dry-run/verbose rendering, and exact pacman execution shape for `substrate world enable --provision-deps`.
  - **Versioning / compat**:
    - Must remain fail-closed and manager-specific; no fallback, no partial provisioning, no AUR-helper widening.

- **Contract ID**: `C-05`
  - **Type**: UX affordance
  - **Owner seam**: `SEAM-5`
  - **Direct consumers**:
    - `SEAM-6`
  - **Derived consumers**:
    - runtime operators
    - support/docs maintainers
  - **Thread IDs**:
    - `THR-05`
  - **Definition**:
    - Runtime no-system-package-mutation contract for `deps current sync|install`, including read-only probe families, explicit-item scope, manager-aware missing-requirement rendering, and deterministic remediation.
  - **Versioning / compat**:
    - Reuses upstream non-system-package behavior only after all in-scope system-package requirements are already satisfied.

## Thread registry

- **Thread ID**: `THR-01`
  - **Producer seam**: `SEAM-1`
  - **Consumer seam(s)**:
    - `SEAM-2`
    - `SEAM-3`
    - `SEAM-4`
    - `SEAM-5`
    - `SEAM-6`
  - **Carried contract IDs**:
    - `C-01`
  - **Purpose**:
    - Carry the single authoritative manager-aware operator contract and accepted decision set into every downstream seam.
  - **State**: revalidated
  - **Revalidation trigger**:
    - shared CLI/runtime wording, exit-code posture, request-profile posture, or authority-handoff targets change in ADR-0033, the APT pack, or the bundles contract
  - **Satisfied by**:
    - `SEAM-1` closeout publishing `C-01` plus the authoritative handoff/defer list, with downstream seam-local revalidation against the published pack-root contract and decision register
  - **Notes**:
    - This is the highest-load-bearing thread in the pack. It exists because the source planning pack explicitly split contract ownership from every later slice.

- **Thread ID**: `THR-02`
  - **Producer seam**: `SEAM-2`
  - **Consumer seam(s)**:
    - `SEAM-4`
    - `SEAM-6`
  - **Carried contract IDs**:
    - `C-02`
  - **Purpose**:
    - Carry deterministic in-world world-manager selection and support-gate outcomes into provisioning execution and platform validation.
  - **State**: revalidated
  - **Revalidation trigger**:
    - `/etc/os-release` tie-break rules, supported family mapping, or unsupported-backend posture changes
  - **Satisfied by**:
    - `SEAM-2` closeout publishing probe/support-gate evidence across supported and unsupported paths
  - **Notes**:
    - Runtime fail-early does not consume this thread directly because runtime system-package handling stays read-only and does not route through provisioning-time manager selection.

- **Thread ID**: `THR-03`
  - **Producer seam**: `SEAM-3`
  - **Consumer seam(s)**:
    - `SEAM-4`
    - `SEAM-5`
    - `SEAM-6`
  - **Carried contract IDs**:
    - `C-03`
  - **Purpose**:
    - Carry additive pacman schema truth and inventory-view obligations into provisioning, runtime fail-early handling, and validation surfaces.
  - **State**: revalidated
  - **Revalidation trigger**:
    - `install.method` vocabulary, `install.pacman` shape, invalid-state rules, or non-runnable pacman scope changes
  - **Satisfied by**:
    - `SEAM-3` closeout publishing the accepted schema/view contract and inventory implementation/test evidence
  - **Notes**:
    - This thread isolates authoring and validation churn from provisioning execution churn.

- **Thread ID**: `THR-04`
  - **Producer seam**: `SEAM-4`
  - **Consumer seam(s)**:
    - `SEAM-5`
    - `SEAM-6`
  - **Carried contract IDs**:
    - `C-04`
  - **Purpose**:
    - Carry provisioning-time normalization, mixed-manager rejection, request-profile routing, and exact pacman execution shape into runtime remediation and cross-platform validation.
  - **State**: revalidated
  - **Revalidation trigger**:
    - requirement normalization rules, pacman command shape, mixed-manager posture, or shared `world_enable` / `world-agent` touch surfaces change
  - **Satisfied by**:
    - `SEAM-4` closeout publishing provisioning-routing evidence and exact manager-aware dry-run/verbose rendering, with `SEAM-5` seam-local review revalidating the runtime fail-early plan against that published basis
  - **Notes**:
    - `REM-003` was resolved in `SEAM-4` closeout after the shared provisioning touch surface was revalidated.

- **Thread ID**: `THR-05`
  - **Producer seam**: `SEAM-5`
  - **Consumer seam(s)**:
    - `SEAM-6`
  - **Carried contract IDs**:
    - `C-05`
  - **Purpose**:
    - Carry runtime fail-early semantics, explicit-item scoping, and manager-aware remediation wording into validation evidence and doc reconciliation.
  - **State**: revalidated
  - **Revalidation trigger**:
    - runtime in-scope rules, read-only probe families, or backend-specific remediation wording change
  - **Satisfied by**:
    - `SEAM-5` closeout publishing runtime read-only probe evidence, explicit-item scoping evidence, and accepted remediation wording, with `SEAM-6` seam-local review revalidating the handoff for terminal conformance work
  - **Notes**:
    - This thread is what keeps runtime behavior from drifting back toward mutation-at-runtime semantics.

## Dependency graph

```mermaid
flowchart LR
  S1[SEAM-1] --> S2[SEAM-2]
  S1 --> S3[SEAM-3]
  S1 --> S4[SEAM-4]
  S1 --> S5[SEAM-5]
  S1 --> S6[SEAM-6]
  S2 --> S4
  S2 --> S6
  S3 --> S4
  S3 --> S5
  S3 --> S6
  S4 --> S5
  S4 --> S6
  S5 --> S6
```

## Critical path

1. `SEAM-1` has already published the manager-aware contract, decision register, and authority handoff on `THR-01`.
2. `SEAM-4` has now landed with a passed seam-exit gate and published `THR-04`, so the provisioning-routing handoff is a closeout-backed fact.
3. `SEAM-5` has now landed with a passed seam-exit gate and a published `THR-05` handoff.
4. `SEAM-6` is the terminal active seam because it now consumes the revalidated `THR-05` handoff and no queued `next` seam remains.

## Workstreams

- **Rolled into seams**
  - `NASP-PWS-contract` -> `SEAM-1`
  - `NASP-PWS-os_probe` -> `SEAM-2`
  - `NASP-PWS-schema_inventory` -> `SEAM-3`
  - `NASP-PWS-provisioning_wiring` -> `SEAM-4`
  - `NASP-PWS-runtime_fail_early` -> `SEAM-5`
  - `NASP-PWS-docs_validation` -> `SEAM-6`
- **Governance-only lineage**
  - `NASP-PWS-tasks_checkpoints`
    - owns the source pack's checkpoint cadence, kickoff prompts, `tasks.json`, `plan.md`, `session_log.md`, and `quality_gate_report.md`
    - remains represented here through closeout scaffolds, horizon policy, and pack-level governance rather than as a seam with product behavior

## Terminal boundary accounting

The pack does not introduce a separate closed-state vocabulary. At the SEAM-6 terminal boundary, the existing `revalidated` thread states are retained and recorded as consumed by the seam-exit closeout with no downstream carry.

- `THR-01`: `revalidated`; consumed by `SEAM-6`; no downstream carry.
- `THR-02`: `revalidated`; consumed by `SEAM-6`; no downstream carry.
- `THR-03`: `revalidated`; consumed by `SEAM-6`; no downstream carry.
- `THR-04`: `revalidated`; consumed by `SEAM-6`; no downstream carry.
- `THR-05`: `revalidated`; consumed by `SEAM-6`; no downstream carry.
