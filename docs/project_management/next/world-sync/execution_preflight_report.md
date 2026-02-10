# Execution Preflight Gate Report — world-sync

Date (UTC): 2026-02-10T18:38:23Z

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/next/world-sync`

## Recommendation

RECOMMENDATION: **REVISE**

Reason:
- Preflight gate has not been executed yet. This file is an execution-time report and must be completed by task `F0-exec-preflight` before any triads begin.

## Inputs Reviewed

- [ ] Planning quality gate is `ACCEPT` (`docs/project_management/next/world-sync/quality_gate_report.md`)
- [ ] ADR accepted and still matches intent
- [ ] Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts)
- [ ] Triad sizing is appropriate (each slice is one behavior delta; no “grab bag” slices)
- [ ] Required planning artifacts exist (when required by planning standards): `impact_map.md`, `manual_testing_playbook.md`
- [ ] Cross-platform plan is explicit (tasks.json meta: behavior + CI parity platforms, plus WSL mode if needed)

## 0) Slice Sizing (one behavior delta each)

- Slices reviewed:
- Any required splits before starting execution:

## 1) Cross-Platform Coverage (explicit and correct)

From `docs/project_management/next/world-sync/tasks.json` meta:
- Declared behavior platforms (smoke required): `["linux", "macos", "windows"]`
- Declared CI parity platforms (parity required): `["linux", "macos", "windows"]` (legacy alias: `platforms_required`)
- WSL required: `false`
- WSL task mode: `N/A`

Notes:
- If WSL coverage is required, confirm `meta.wsl_required=true` and `meta.wsl_task_mode` is set correctly.
- Schema v4+ platform-fix model (boundary-only):
  - Normal slices have only `X-integ` (single merge task).
  - Checkpoint-boundary slices (`WS2`, `WS5`, `WS7`) have `X-integ-core`, `X-integ-linux` / `X-integ-macos` / `X-integ-windows`, and `X-integ` final.

## 2) Smoke Scripts Are Not “Toy” Checks

Smoke scripts must be a runnable, minimal version of how a careful human would validate the feature.

Manual playbook (when required):
- `docs/project_management/next/world-sync/manual_testing_playbook.md`

Smoke scripts to validate (only required for behavior platforms; parity-only platforms may be explicit no-ops):
- Linux smoke: `docs/project_management/next/world-sync/smoke/linux-smoke.sh`
- macOS smoke: `docs/project_management/next/world-sync/smoke/macos-smoke.sh`
- Windows smoke: `docs/project_management/next/world-sync/smoke/windows-smoke.ps1`

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
  - Set `SLICE_ID` to the checkpoint boundary slice: `WS2`, `WS5`, or `WS7`.
  - `LEDGER_PATH="docs/project_management/next/world-sync/logs/$SLICE_ID/ci-audit/ledger.jsonl"`
  - Set `CHECKOUT_SHA` to the exact commit you dispatch (must match `CI_CHECKOUT_REF` / `SMOKE_CHECKOUT_REF` when provided):
    - `CHECKOUT_SHA="$(git rev-parse HEAD)"`
  - `scripts/ci-audit/ci_audit.sh --kind ci-testing --orch-branch "feat/world-sync" --ledger-path "$LEDGER_PATH"`
  - `scripts/ci-audit/ci_audit.sh --kind feature-smoke --orch-branch "feat/world-sync" --feature-dir "docs/project_management/next/world-sync" --ledger-path "$LEDGER_PATH"`
  - Evidence recorder (recommended after dispatch; `RUN_ID` is the numeric id in the Actions run URL, e.g. `.../actions/runs/123456789`):
    - `scripts/ci-audit/ci_audit_record.sh --ledger-path "$LEDGER_PATH" --kind ci-testing --orch-branch "feat/world-sync" --run-id "$RUN_ID" --tested-sha "$CHECKOUT_SHA" --feature-dir "docs/project_management/next/world-sync"`
    - `scripts/ci-audit/ci_audit_record.sh --ledger-path "$LEDGER_PATH" --kind feature-smoke --orch-branch "feat/world-sync" --run-id "$RUN_ID" --tested-sha "$CHECKOUT_SHA" --feature-dir "docs/project_management/next/world-sync"`

Policy note:
- Docs/planning-only changes (anything under `docs/`) may skip all CI/smoke **only when** the advisory audit outputs `DIFF_CLASS=docs_only` and `RECOMMEND=skip`.

Runner readiness:
- Required self-hosted runners exist and are labeled correctly:

Run ids/URLs (if executed during preflight):
- CI compile parity:
- Linux smoke:
- macOS smoke:
- Windows smoke:
- WSL smoke:

## 4) Required Fixes Before Starting The First Slice (if any)

- 
