---
slice_id: S2
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
  - inventory validation rules change around supported methods or mutual exclusion
  - inventory view surfaces change and can erase `pacman` or reorder `install.pacman`
  - downstream seams assume provisioning normalization belongs in schema validation
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-03
contracts_produced: []
contracts_consumed:
  - C-01
  - C-03
open_remediations: []
candidate_subslices: []
---
### S2 - Implement pacman inventory validation and view rendering

- **User/system value**:
  - Inventory authors and downstream seams get one deterministic pacman schema implementation and one consistent pacman rendering story across validation and list/show surfaces.
- **Scope (in/out)**:
  - In:
    - extend schema validation to accept `install.method=pacman`
    - validate `install.pacman` shape and mutual-exclusion rules
    - keep pacman-backed packages non-runnable in v1
    - surface pacman-backed items correctly in list/show JSON and YAML views
  - Out:
    - probe/support-gate behavior
    - provisioning-time normalization or pacman execution
    - runtime read-only probes and remediation wording
- **Acceptance criteria**:
  - Pacman-backed inventory remains additive on `version: 1`.
  - Invalid pacman schema shapes exit `2` with deterministic validation behavior.
  - Pacman-backed items render as `pacman` in list/show surfaces without losing authored package order.
  - Pacman-backed packages remain non-runnable prerequisites in v1.
- **Dependencies**:
  - `SEAM-1` publishes `C-01` and advances `THR-01` to a stable shared contract.
- **Verification**:
  - Unit and integration tests for inventory validation on valid and invalid pacman-backed package shapes.
  - Inventory view tests that assert pacman-backed items remain visible as `pacman` and preserve authored `install.pacman` order.
- **Rollout/safety**:
  - Keep the seam additive and validation-driven; do not widen into provisioning execution or runtime mutation.
- **Review surface refs**:
  - `../../review_surfaces.md#R1`
  - `../../review_surfaces.md#R2`
  - `../../review_surfaces.md#R3`

#### S2.T1 - Extend schema validation for pacman-backed items

- **Outcome**:
  - Inventory validation accepts valid pacman-backed items and rejects invalid mutual-exclusion or non-runnable violations.
- **Inputs/outputs**:
  - Inputs: package YAML definitions, `version: 1`, shared contract `C-01`
  - Outputs: accepted additive schema truth for pacman-backed authored inventory
- **Thread/contract refs**:
  - Consumes `C-01`
  - Produces `C-03` behavior
- **Implementation notes**:
  - Keep method vocabulary and mutual-exclusion rules in `inventory.rs` or its existing validation helpers.
- **Acceptance criteria**:
  - Valid pacman-backed items parse on `version: 1`; invalid combinations remain exit `2`.
- **Test notes**:
  - Cover empty `install.pacman`, missing method/list coupling, and runnable pacman violations.

#### S2.T2 - Render pacman-backed inventory in list/show surfaces

- **Outcome**:
  - Inventory views preserve `pacman` as a first-class method and keep authored `install.pacman` order visible.
- **Inputs/outputs**:
  - Inputs: validated pacman-backed inventory definitions
  - Outputs: deterministic JSON, YAML, and list/show rendering
- **Thread/contract refs**:
  - Inbound: `THR-01` / `C-01`
  - Outbound: `THR-03` / `C-03`
- **Implementation notes**:
  - Revalidate `world_deps_inventory_validation_wdp0.rs` and `world_deps_inventory_views.rs` as the primary test surfaces.
- **Acceptance criteria**:
  - View output never collapses `pacman` into another method and never reorders `install.pacman` entries.
- **Test notes**:
  - Add view assertions for list/show text plus structured JSON/YAML output where relevant.
