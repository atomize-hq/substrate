---
slice_id: S2
seam_id: SEAM-07
slice_kind: delivery
execution_horizon: active
status: landed
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
    landing: passed
    closeout: passed
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

## Realized macOS-hosted checkpoint evidence

- Hosted behavior evidence for `SEAM-07` remains inherited from `../../governance/seam-06-closeout.md`, which already published the authoritative `scripts/mac/smoke.sh --bedpm-installer-conformance` path as the Lima-backed Linux guest verification entrypoint for the BEDPM installer contract.
- The current checkpoint does not reinterpret compile parity as hosted behavior evidence. Instead, it ties the existing hosted-evidence path into the realized `CP1` record alongside the live checkpoint runs captured in `S1`.
- Repository evidence remains explicit and unchanged:
  - `scripts/mac/smoke.sh --bedpm-installer-conformance` is defined as "Run the BEDPM Linux installer smoke through the Lima-backed guest path"
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md` records the macOS-hosted Lima verification flow and states that it runs `smoke/linux-smoke.sh` through the Lima-backed Linux guest path
  - `docs/WORLD.md` states that hosted installer behavior coverage on macOS flows through the Lima-backed Linux guest/world-service path and does not define native macOS package-manager selection
- Current checkpoint references that bind the inherited hosted evidence into `CP1`:
  - compile parity run `23711447102` (`https://github.com/atomize-hq/substrate/actions/runs/23711447102`) passed on `macos-14`, so the checkpoint record includes current macOS parity coverage without claiming native behavior coverage
  - quick CI run `23711510594` (`https://github.com/atomize-hq/substrate/actions/runs/23711510594`) failed on Linux shell lint, not on the hosted-evidence posture, so the blocker is checkpoint cleanliness rather than macOS-hosted ambiguity
  - Linux feature-smoke run `23711646303` (`https://github.com/atomize-hq/substrate/actions/runs/23711646303`) passed for `BEDPM3`, preserving the authoritative Linux harness that the macOS-hosted path is defined to reuse
- This slice therefore lands by making the hosted evidence explicit, inherited, and checkpoint-scoped: `SEAM-07` consumes the published `SEAM-06` hosted-verification truth and ties it to the realized `CP1` record without inventing a second macOS behavior authority.
