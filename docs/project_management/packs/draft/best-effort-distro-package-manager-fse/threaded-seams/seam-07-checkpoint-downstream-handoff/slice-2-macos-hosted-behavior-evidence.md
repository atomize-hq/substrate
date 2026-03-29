---
slice_id: S2
seam_id: SEAM-07
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - macOS Lima-backed behavior-evidence expectations change
    - compile parity or CI quick requirements change
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-06
contracts_produced:
  - C-11
contracts_consumed:
  - C-10
open_remediations: []
candidate_subslices: []
---
### S2 - macOS-hosted behavior evidence

- **User/system value**: macOS checkpoint evidence proves real Lima-backed Linux installer behavior instead of stopping at parity-only signals.
- **Scope (in/out)**:
  - In: the macOS-hosted behavior-evidence record that depends on the Lima-backed Linux smoke path
  - Out: new macOS-native package-manager-selection logic or non-checkpoint validation work
- **Acceptance criteria**:
  - hosted evidence names the Lima-backed Linux path explicitly
  - hosted evidence stays tied to the checkpoint boundary
  - no artifact implies native macOS package-manager-selection behavior
- **Dependencies**:
  - `S1`
  - `../../governance/seam-06-closeout.md`
- **Verification**:
  - review proves hosted evidence is behavior coverage, not compile-only parity

## Realized macOS-hosted evidence

- `SEAM-06` already published the authoritative hosted-behavior path for this feature: `scripts/mac/smoke.sh --bedpm-installer-conformance` runs `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh` through the Lima-backed Linux guest path rather than asserting native macOS package-manager-selection behavior.
- The published operator evidence still points at that same behavior path:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md` records the macOS-hosted Lima verification command and states that its output must match the authoritative Linux smoke behavior.
  - `docs/WORLD.md` states that hosted installer behavior coverage on macOS flows through the Lima-backed Linux guest/world-agent path and explicitly avoids claiming native macOS package-manager selection.
- The current CP1 compile-parity run `23711447102` (`https://github.com/atomize-hq/substrate/actions/runs/23711447102`) includes a successful `macos-14` lane, but this slice treats that result as parity evidence only, not as the hosted behavior proof required by `C-11`.
- The current CP1 checkpoint record therefore ties macOS-hosted behavior evidence to the already-published `SEAM-06` conformance surface, while using the new CP1 runs only to show that the checkpoint still includes macOS parity coverage and has not widened the feature into native macOS behavior scope.
- Quick CI failure in run `23711510594` does not change this boundary: it blocks a clean checkpoint outcome, but it does not convert compile parity into hosted behavior evidence or weaken the Lima-backed Linux path requirement.
