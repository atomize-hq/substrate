---
slice_id: S1
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
  - THR-01 changes pacman v1 scope or authority boundaries for schema work
  - upstream bundles contract changes merge or enabled-view semantics relied on by pacman-backed inventory
  - inventory validation rules drift toward runnable or translated pacman behavior
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
contracts_produced:
  - C-03
contracts_consumed:
  - C-01
open_remediations: []
candidate_subslices: []
---
### S1 - Define `C-03` additive pacman schema and inventory-view contract

#### Goal

Make `C-03` explicit enough that downstream seams can treat pacman-backed inventory as one additive schema truth without inventing a translation layer or widening runnable behavior.

#### `C-03` - Pacman schema and inventory-view contract (authoritative pre-exec text)

##### 1) Additive schema posture

- World-deps package files remain on `version: 1`.
- `install.method` MAY now be `pacman` in addition to the existing supported methods.
- Pacman-backed packages MUST provide `install.pacman` and MUST NOT rely on any translation layer or abstract system-package mapping.

##### 2) `install.pacman` field shape

- `install.pacman` MUST be a non-empty ordered list of package-name strings.
- Author order in `install.pacman` is preserved in stored definitions and rendered inventory views.
- The schema layer MUST NOT normalize, de-duplicate, or sort `install.pacman`; that belongs to later provisioning normalization.

##### 3) Mutual exclusion rules

- When `install.method=pacman`, package definitions MUST reject:
  - `install.apt`
  - `install.script`
  - `install.script_embedded`
  - `install.manual_instructions`
- Invalid combinations remain taxonomy-aligned exit `2` behavior in downstream consumers.

##### 4) v1 pacman scope constraints

- Pacman-backed packages remain provisioning-only prerequisites in v1.
- Pacman-backed packages remain non-runnable in v1:
  - no wrapper generation
  - no runnable entrypoint widening
  - no new pacman-specific present semantics
- This seam does not add built-in pacman catalog entries; it only extends authored inventory support.

##### 5) Inventory view obligations

- Inventory JSON, YAML, and list/show views MUST preserve `install.method=pacman` explicitly.
- Pacman-backed items MUST NOT be collapsed into `apt`, `script`, or `manual` renderings.
- Rendered views MUST preserve authored `install.pacman` order.

#### Verification checklist (contract gate input)

- `C-03` stays additive on `version: 1`.
- `install.method=pacman` plus `install.pacman` are explicit and concrete.
- Mutual-exclusion rules are explicit.
- Pacman-backed items remain non-runnable prerequisites in v1.
- Inventory views preserve `pacman` as a first-class method and preserve authored order.
