# Execution Preflight Gate Report — warn-config-global-show-workspace-overrides

Date (UTC): 2026-01-30T00:01:01Z

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/next/warn-config-global-show-workspace-overrides`

## Recommendation

RECOMMENDATION: **ACCEPT** | **REVISE**

## Inputs Reviewed

- [ ] Planning quality gate is `ACCEPT` (`docs/project_management/next/warn-config-global-show-workspace-overrides/quality_gate_report.md`)
- [ ] ADR accepted and still matches intent
- [ ] Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts)
- [ ] Triad sizing is appropriate (each slice is one behavior delta; no “grab bag” slices)
- [ ] Required planning artifacts exist (when required by planning standards): `integration_map.md`, `manual_testing_playbook.md`
- [ ] Cross-platform plan is explicit (tasks.json meta: behavior + CI parity platforms, plus WSL mode if needed)

## 0) Slice Sizing (one behavior delta each)

- Slices reviewed:
- Any required splits before starting execution:

## 1) Cross-Platform Coverage (explicit and correct)

From `docs/project_management/next/warn-config-global-show-workspace-overrides/tasks.json` meta:
- Declared behavior platforms (smoke required): `linux, macos, windows`
- Declared CI parity platforms (parity required): `linux, macos, windows` (legacy alias: `platforms_required`)
- WSL required: `false`
- WSL task mode: `N/A` (when WSL required)

Notes:
- If WSL coverage is required, confirm `meta.wsl_required=true` and `meta.wsl_task_mode` is set correctly.
- If using the platform-fix integration model, confirm tasks exist per slice:
  - `X-integ-core`, optional `X-integ-<platform>` (CI parity platforms + optional WSL task when `wsl_task_mode="separate"`), and `X-integ` final.

## 2) Smoke Scripts Are Not “Toy” Checks

Smoke scripts must be a runnable, minimal version of how a careful human would validate the feature.

Manual playbook (when required):
- `docs/project_management/next/warn-config-global-show-workspace-overrides/manual_testing_playbook.md`

Smoke scripts to validate (only required for behavior platforms; parity-only platforms may be explicit no-ops):
- Linux smoke: `docs/project_management/next/warn-config-global-show-workspace-overrides/smoke/linux-smoke.sh`
- macOS smoke: `docs/project_management/next/warn-config-global-show-workspace-overrides/smoke/macos-smoke.sh`
- Windows smoke: `docs/project_management/next/warn-config-global-show-workspace-overrides/smoke/windows-smoke.ps1`

Parity notes (map smoke ↔ manual; include concrete assertions):
- Manual step(s):
  - Smoke command(s):
  - Expected output/assertion(s):

Gaps (must fix before execution begins):
- 

## 3) CI Dispatch Path Is Runnable (if applicable)

Integration task dispatch commands (copy verbatim from `tasks.json` integration checklists):
- CI compile parity:
- Feature Smoke dispatch:
- Advisory CI audit (recommended before any dispatch):
  - `scripts/ci-audit/ci_audit.sh --kind ci-testing --orch-branch "<orch-branch>" --ledger-path "docs/project_management/next/warn-config-global-show-workspace-overrides/logs/<slice>/ci-audit/ledger.jsonl"`
  - `scripts/ci-audit/ci_audit.sh --kind feature-smoke --orch-branch "<orch-branch>" --feature-dir "docs/project_management/next/warn-config-global-show-workspace-overrides" --ledger-path "docs/project_management/next/warn-config-global-show-workspace-overrides/logs/<slice>/ci-audit/ledger.jsonl"`
  - Evidence recorder (recommended after dispatch):
    - `scripts/ci-audit/ci_audit_record.sh --ledger-path "docs/project_management/next/warn-config-global-show-workspace-overrides/logs/<slice>/ci-audit/ledger.jsonl" --kind <ci-testing|feature-smoke> --orch-branch "<orch-branch>" --run-id "<id>" --tested-sha "<sha>" --feature-dir "docs/project_management/next/warn-config-global-show-workspace-overrides"`

Policy note:
- Docs/planning-only changes (anything under `docs/`) may skip all CI/smoke. `ci_audit.sh` should recommend `RECOMMEND=skip` with `DIFF_CLASS=docs_only`.

Runner readiness:
- Required self-hosted runners exist and are labeled correctly:

Run ids/URLs (if executed during preflight):
- CI compile parity:
- Linux smoke:
- macOS smoke:
- Windows smoke:
- WSL smoke:

## 4) Required Fixes Before Starting C0 (if any)

- 
