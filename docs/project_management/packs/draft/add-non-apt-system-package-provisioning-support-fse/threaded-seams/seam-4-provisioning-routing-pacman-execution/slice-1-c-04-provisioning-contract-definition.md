---
slice_id: S1
seam_id: SEAM-4
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
  - C-02 changes probe/support-gate outcomes
  - C-03 changes pacman schema semantics or derived requirement assumptions
  - shared-file changes in world_enable or world-agent invalidate the provisioning execution basis
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
  - THR-02
  - THR-03
  - THR-04
contracts_produced:
  - C-04
contracts_consumed:
  - C-01
  - C-02
  - C-03
open_remediations: []
candidate_subslices: []
---
### S1 - Define `C-04` provisioning routing and pacman execution contract

#### Goal

Make `C-04` explicit enough that downstream seams can treat provisioning-time normalization, mixed-manager rejection, request-profile routing, and exact pacman execution as one deterministic contract.

#### `C-04` - Provisioning routing and pacman execution contract (authoritative pre-exec text)

##### 1) Requirement-set derivation

- Provisioning derives one normalized APT requirement set and one normalized pacman requirement set from the effective enabled world-deps set.
- Pacman requirement normalization de-duplicates by exact package name and sorts in ascending byte order.
- Schema/view ordering from `C-03` is preserved as input truth; provisioning normalization owns the execution-time ordering.

##### 2) Mixed-manager rule

- If both normalized requirement sets are non-empty, provisioning exits `4` before any package-manager command runs.
- No partial provisioning is allowed.
- Error output must identify the mismatch and point back to the enabled inventory / world-manager expectation.

##### 3) Request-profile boundary

- The provisioning execution path uses the internal `world-deps-provision` request profile only.
- `SUBSTRATE_WORLD_REQUEST_PROFILE` is not an operator control surface for this seam.
- Host environment must not override the provisioning routing boundary.

##### 4) Pacman execution shape

- Pacman provisioning uses the exact command shape:
  - `pacman -Sy --noconfirm --needed <packages...>`
- Package arguments follow normalized pacman requirement order.
- No AUR helpers, retries, lock-file intervention, or fallback to `apt` are allowed.

##### 5) No-op, dry-run, and verbose behavior

- Dry-run performs the probe and requirement derivation but does not mutate the world.
- Dry-run and verbose output render the detected world manager and normalized requirement sets in stable order.
- If the normalized requirement set for the detected manager is empty, provisioning is a contract-defined no-op.

#### Verification checklist (contract gate input)

- Requirement derivation and normalization are explicit.
- Mixed-manager rejection is explicit and fail-closed.
- Request-profile routing stays internal.
- Pacman command shape and failure posture are explicit.
- Dry-run / verbose rendering and no-op behavior are concrete enough for implementation.
