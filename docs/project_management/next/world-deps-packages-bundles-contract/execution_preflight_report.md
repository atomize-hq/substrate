# Execution Preflight Gate Report — world-deps-packages-bundles-contract

Date (UTC): 2026-02-14T02:59:35Z

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/next/world-deps-packages-bundles-contract`

## Recommendation

RECOMMENDATION: **ACCEPT**

## Inputs Reviewed

- [x] Planning quality gate is `ACCEPT` (`docs/project_management/next/world-deps-packages-bundles-contract/quality_gate_report.md`)
- [x] ADR accepted and still matches intent (`docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md`)
- [x] Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts)
- [x] Triad sizing is appropriate (each slice is one behavior delta)
- [x] Required planning artifacts exist: `impact_map.md`, `manual_testing_playbook.md`
- [x] Cross-platform plan is explicit and matches parity spec (tasks.json meta: behavior + CI parity platforms + WSL mode)
- [x] Advisory CI audit + evidence ledger tooling exists and is referenced by integration prompts (`scripts/ci-audit/ci_audit.sh`, `scripts/ci-audit/ci_audit_record.sh`)

## 0) Slice Sizing (one behavior delta each)

- Slices reviewed: `WDP0, WDP1, WDP2, WDP3, WDP4, WDP5`
- Required splits before starting execution: none

## 1) Cross-Platform Coverage (explicit and correct)

From `docs/project_management/next/world-deps-packages-bundles-contract/tasks.json` meta:
- Declared behavior platforms (smoke required): `linux, macos`
- Declared CI parity platforms (parity required): `linux, macos`
- WSL required: `true` (bundled into Linux smoke via `RUN_WSL=1`)

## 2) Smoke Scripts Are Not “Toy” Checks

Manual playbook:
- `docs/project_management/next/world-deps-packages-bundles-contract/manual_testing_playbook.md`

Smoke scripts to validate:
- Linux smoke: `docs/project_management/next/world-deps-packages-bundles-contract/smoke/linux-smoke.sh`
- macOS smoke: `docs/project_management/next/world-deps-packages-bundles-contract/smoke/macos-smoke.sh`
- WSL smoke: run the Linux smoke inside WSL (CI dispatch via `RUN_WSL=1`)

Evidence (static review):
- The shared smoke core (`docs/project_management/next/world-deps-packages-bundles-contract/smoke/_core.sh`) runs real `substrate world deps ...` workflows, asserts exit codes, and validates key output (including backend-unavailable fail-closed exit `3`, legacy-path ignore via intentionally-invalid YAML fixtures, and `--json` shape stability via Python parsing).

## 3) CI Dispatch Path Is Runnable

- CI cadence is defined by:
  - `docs/project_management/next/world-deps-packages-bundles-contract/ci_checkpoint_plan.md`

Evidence (static review):
- Dispatch commands are concrete and map to repo Make targets:
  - `make ci-compile-parity ...` (cross-platform compile parity)
  - `make feature-smoke ...` (cross-platform feature smoke, behavior platforms only; WSL bundled via `RUN_WSL=1`)
- GitHub Actions workflows declare runner expectations:
  - Self-hosted smoke jobs require labels:
    - Linux: `[self-hosted, Linux, linux-host]`
    - macOS: `[self-hosted, macOS]`
    - WSL: `[self-hosted, Linux, wsl]`

## 4) Required Fixes Before Starting The First Slice

- None.
