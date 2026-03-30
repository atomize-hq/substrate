---
seam_id: SEAM-5
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-5-runtime-fail-early-remediation/slice-3-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
  - seam-1-closeout.md
  - seam-3-closeout.md
  - seam-4-closeout.md
  required_threads:
  - THR-01
  - THR-03
  - THR-04
  - THR-05
  stale_triggers:
  - runtime docs/tests drift back toward mutation-at-runtime semantics
  - runtime scope rules, read-only probe families, or remediation wording change before downstream revalidation
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-5 Runtime fail-early and remediation

This closeout records the landed `C-05` runtime fail-early and remediation work published by `SEAM-5`.

## Seam-exit gate record

- **Source artifact**: [`slice-3-seam-exit-gate.md`](../threaded-seams/seam-5-runtime-fail-early-remediation/slice-3-seam-exit-gate.md)
- **Landed evidence**:
  - Published contract artifact at [`../contract.md`](../contract.md)
  - Runtime fail-early implementation in [`crates/shell/src/builtins/world_deps/surfaces.rs`](/home/spenser/__Active_code/substrate/crates/shell/src/builtins/world_deps/surfaces.rs)
  - Regression coverage in [`crates/shell/tests/world_deps_current_dry_run_wdp3.rs`](/home/spenser/__Active_code/substrate/crates/shell/tests/world_deps_current_dry_run_wdp3.rs), [`crates/shell/tests/world_deps_apt_fail_early_wdap1.rs`](/home/spenser/__Active_code/substrate/crates/shell/tests/world_deps_apt_fail_early_wdap1.rs), and [`crates/shell/tests/world_deps_apt_install_wdp5.rs`](/home/spenser/__Active_code/substrate/crates/shell/tests/world_deps_apt_install_wdp5.rs)
  - Runtime-doc reconciliation in [`docs/reference/world/deps/README.md`](/home/spenser/__Active_code/substrate/docs/reference/world/deps/README.md) and [`docs/internals/world/deps.md`](/home/spenser/__Active_code/substrate/docs/internals/world/deps.md)
- **Contracts published or changed**: `C-05`
- **Threads published / advanced**: `THR-05` -> `published`
- **Runtime safety record**:
  - `substrate world deps current sync|install` remains read-only for system-package managers, uses only read-only `dpkg-query` / `pacman -Q` probes, and does not invoke `apt`, `apt-get`, `dpkg`, or `pacman` mutating commands
  - `deps current install <ITEM...>` scopes fail-early checks only to the explicit expanded item set
  - missing APT or pacman requirements exit `4` before non-system-package mutation
  - remediation stays exact and points back to `substrate world enable --provision-deps`
  - dry-run and verbose output stay deterministic and stable for normalized requirement rendering
- **Review-surface delta**:
  - `review_surfaces.md` still describes the pack-level workflow, but `SEAM-5` now has closeout-backed runtime evidence instead of a scaffold-only claim
  - `C-05` is now explicit runtime truth for read-only probes, explicit-item scope, manager-aware missing-requirement rendering, and deterministic remediation
  - the runtime branch of the workflow diagram is the concrete downstream surface `SEAM-6` must revalidate
- **Planned-vs-landed delta**:
  - The planned runtime fail-early contract landed as published `C-05` and the implemented runtime surface
  - Runtime system-package handling stayed probe-only and fail-closed
  - explicit-item scoping remained bounded to the requested items rather than the broader enabled set
  - manager-aware remediation stayed exact and continued to point back to provisioning
- **Downstream stale triggers raised**:
  - any change to runtime in-scope rules, read-only probe families, or remediation wording must revalidate `SEAM-6`
  - any doc drift that reintroduces runtime mutation semantics or obscures `C-05` must also revalidate `SEAM-6`
- **Remediation disposition**:
  - `SEAM-5` owns no open blocking remediations at closeout
  - `REM-001` and `REM-002` remain downstream context owned by `SEAM-6`
- **Promotion blockers**: none
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**:
  - `REM-001` and `REM-002` remain downstream context owned by `SEAM-6`
