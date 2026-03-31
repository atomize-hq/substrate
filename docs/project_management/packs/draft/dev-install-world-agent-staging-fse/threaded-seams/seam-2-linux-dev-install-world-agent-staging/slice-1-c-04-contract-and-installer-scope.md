---
slice_id: S1
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - accepted staged path set or sufficiency rule changes
    - selected-profile mapping changes
    - scripts/substrate/install-substrate.sh moves from reference-only posture into an actual touched surface
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
  - C-04
contracts_consumed:
  - C-01
  - C-03
open_remediations: []
candidate_subslices: []
---
### S1 - Define `C-04` staging contract and freeze installer scope

- **User/system value**: Downstream seams can rely on one exact Linux staging contract without inheriting runtime-message work from `SEAM-1` or accidental production-installer scope.
- **Scope (in/out)**:
  - In:
    - concrete `C-04` statements for staged paths, selected-profile mapping, refresh semantics, disabled-world invariants, and no-provisioning posture
    - explicit resolution of the production-installer scope question from `REM-002`
  - Out:
    - runtime preflight or remediation wording
    - any owned behavior change in `scripts/substrate/install-substrate.sh`
- **Acceptance criteria**:
  - `C-04` is stated as executable rules that downstream seams can revalidate without guessing.
  - `scripts/substrate/install-substrate.sh` is explicitly recorded as a regression-only reference surface, not an owned touch surface.
  - verification names concrete surfaces for installer smoke and staged-link evidence.

#### `C-04` - Linux `--no-world` staging contract (authoritative pre-exec text)

- On Linux, `scripts/substrate/dev-install-substrate.sh --no-world --profile <debug|release>` stages executable links at:
  1. `target/bin/world-agent`
  2. `target/bin/linux/world-agent`
- Both staged links must point at `target/<profile>/world-agent` for the selected profile.
- Either staged link location must satisfy the runtime preflight contract published by `SEAM-1`; when both exist, the root `target/bin/world-agent` path remains the runtime-preferred path.
- Re-running the script refreshes both staged links with `ln -sfn`; stale debug/release links do not persist across reruns.
- The `--no-world` path keeps `world.enabled: false`, skips provisioning, and performs no systemd mutation.
- `substrate world enable --profile` does not retarget the staged bridge; staging ownership remains with dev-install.
- `scripts/substrate/install-substrate.sh` remains a regression-only reference surface unless a later seam explicitly widens scope.

#### Verification checklist

- selected-profile mapping is explicit for both `debug` and `release`
- both staged paths are owned and refreshed
- `world.enabled: false` and no-provisioning semantics are explicit
- installer smoke remains the production-installer regression boundary

Checklist:
- Implement: N/A (contract-definition slice)
- Test: N/A (contract-definition slice)
- Validate: confirm `REM-002` is resolved in review and remediation log
- Cleanup: none
