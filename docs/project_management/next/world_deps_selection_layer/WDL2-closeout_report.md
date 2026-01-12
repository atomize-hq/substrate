# Slice Closeout Gate Report — world_deps_selection_layer / WDL2

Date (UTC): 2026-01-12

Standards:
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`
- `docs/project_management/standards/EXECUTIVE_SUMMARY_STANDARD.md`

Feature directory:
- `docs/project_management/next/world_deps_selection_layer`

Slice spec:
- `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`

## Status

STATUS: **COMPLETED**

Rule:
- The `WDL2-integ` task must replace `NOT RUN` with `COMPLETED` and fill every section below.

## Behavior Delta (Existing → New → Why)

- Existing behavior: No `substrate world deps provision` command; tools requiring `install_class=system_packages` could not be explicitly provisioned by Substrate.
- New behavior: Adds `substrate world deps provision` to explicitly provision apt system packages for selected `system_packages` tools (or `--all`) on macOS (Lima guest) and Windows (WSL); Linux host backend fails with exit `4` and prints required packages + manual install guidance.
- Why: ADR-0002 / S2 require provisioning-time explicit system package installs and forbids implicit OS package mutation during `sync/install` (and forbids Linux host package mutation).
- Links:
  - Spec: `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`
  - CI behavior smoke: https://github.com/atomize-hq/substrate/actions/runs/20926130431
  - Integrated HEAD: `f4a05c40dbd2b3d225fce1e5132059eb956a2f75`

## Spec Parity (No Drift)

- Acceptance criteria satisfied: YES
- Spec changes during the slice recorded (with rationale): NONE

## Checks Run (Evidence)

- `cargo fmt`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- Relevant tests:
  - `cargo test -p substrate-shell --test world_deps`
  - `cargo test -p world-agent`
  - `cargo test -p world`
- `make integ-checks`: PASS

## Cross-Platform Smoke

Record run ids/URLs for required behavior platforms:
- Linux: https://github.com/atomize-hq/substrate/actions/runs/20926130431 (success)
- macOS: https://github.com/atomize-hq/substrate/actions/runs/20926130431 (success)
- Windows: https://github.com/atomize-hq/substrate/actions/runs/20926130431 (success)

## Smoke ↔ Manual Parity

- Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset): YES
- Smoke scripts validate exit codes and key output: YES

Notes:
- Behavior smoke passed for `linux`, `macos`, and `windows` in a single dispatched run: https://github.com/atomize-hq/substrate/actions/runs/20926130431
