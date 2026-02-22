# Execution Preflight Gate Report — world-sync

Date (UTC): 2026-02-11T18:54:27Z

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/packs/active/world-sync`

## Recommendation

RECOMMENDATION: **ACCEPT**

Reason:
- Planning Pack inputs are coherent, cross-platform coverage is explicit and consistent across plan/contract/tasks, smoke scripts exercise real workflows with assertions, and CI dispatch/audit tooling is present and referenced by integration kickoff prompts.

## Inputs Reviewed

- [x] Planning quality gate is `ACCEPT` (`docs/project_management/packs/active/world-sync/quality_gate_report.md`)
- [x] ADR(s) and executive summaries reviewed; accepted ADRs match this pack (note: `ADR-0016` is `Draft` and treated as non-blocking context)
- [x] Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts)
- [x] Triad sizing is appropriate (each slice is one behavior delta; no “grab bag” slices)
- [x] Required planning artifacts exist: `impact_map.md`, `manual_testing_playbook.md`
- [x] Cross-platform plan is explicit (tasks.json meta: behavior + CI parity platforms)

## 0) Slice Sizing (one behavior delta each)

- Slices reviewed:
  - `WS0`: workspace CLI surfaces + gating + `sync --dry-run` baseline (no mutations)
  - `WS1`: non-PTY pending diff discovery + dry-run reporting (no mutations)
  - `WS2`: non-PTY world→host apply + safety rails + clear/ack
  - `WS3`: non-PTY auto-sync trigger behavior
  - `WS4`: PTY pending diff discovery + reporting
  - `WS5`: direction expansion (`from_host|both`) + host→world pre-sync + PTY apply
  - `WS6`: internal checkpoint (`workspace checkpoint`)
  - `WS7`: internal rollback (`workspace rollback`)
- Any required splits before starting execution:
  - None.

## 1) Cross-Platform Coverage (explicit and correct)

From `docs/project_management/packs/active/world-sync/tasks.json` meta:
- Declared behavior platforms (smoke required): `["linux", "macos"]`
- Declared CI parity platforms (parity required): `["linux", "macos"]` (legacy alias: `platforms_required`)

Notes:
- Schema v4+ platform-fix model (boundary-only):
  - Normal slices have only `X-integ` (single merge task).
  - Checkpoint-boundary slices (`WS2`, `WS5`, `WS7`) have `X-integ-core`, `X-integ-linux` / `X-integ-macos`, and `X-integ` final.

## 2) Smoke Scripts Are Not “Toy” Checks

Smoke scripts must be a runnable, minimal version of how a careful human would validate the feature.

Manual playbook (when required):
- `docs/project_management/packs/active/world-sync/manual_testing_playbook.md`

Smoke scripts to validate (only required for behavior platforms; parity-only platforms may be explicit no-ops):
- Linux smoke: `docs/project_management/packs/active/world-sync/smoke/linux-smoke.sh`
- macOS smoke: `docs/project_management/packs/active/world-sync/smoke/macos-smoke.sh`

Parity notes (map smoke ↔ manual; include concrete assertions):
- Manual step(s):
  - WS2: create world change → `workspace sync --verbose` includes the touched path and apply makes the host file exist
  - WS7: rollback safety rail refusal without `--force` (exit `5`), then forced rollback succeeds (exit `0`) and removes the mutated file
- Smoke command(s):
  - `bash smoke/linux-smoke.sh` / `bash smoke/macos-smoke.sh` (branches on `SUBSTRATE_SMOKE_SLICE_ID`)
- Expected output/assertion(s):
  - Exit `0` when the slice’s required behaviors are satisfied for the current platform.
  - `workspace sync --verbose` output contains the pending path being applied (e.g., `hello-from-world.txt`, `hello-both.txt`).
  - Rollback refusal path returns exit `5` and the forced path returns exit `0`.

Gaps (must fix before execution begins):
- None.

## 3) CI Dispatch Path Is Runnable (if applicable)

Integration task dispatch commands (copy verbatim from `tasks.json` integration checklists):
- CI compile parity:
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/world-sync" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`
- Feature Smoke dispatch:
  - `make feature-smoke FEATURE_DIR="docs/project_management/packs/active/world-sync" PLATFORM=behavior SMOKE_SLICE_ID="WS2" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-sync" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`
- Advisory CI audit (recommended before any dispatch):
  - Set `SLICE_ID` to the checkpoint boundary slice: `WS2`, `WS5`, or `WS7`.
  - `LEDGER_PATH="docs/project_management/packs/active/world-sync/logs/$SLICE_ID/ci-audit/ledger.jsonl"`
  - Set `CHECKOUT_SHA` to the exact commit you dispatch (must match `CI_CHECKOUT_REF` / `SMOKE_CHECKOUT_REF` when provided):
    - `CHECKOUT_SHA="$(git rev-parse HEAD)"`
  - `scripts/ci-audit/ci_audit.sh --kind ci-testing --orch-branch "feat/world-sync" --ledger-path "$LEDGER_PATH"`
  - `scripts/ci-audit/ci_audit.sh --kind feature-smoke --orch-branch "feat/world-sync" --feature-dir "docs/project_management/packs/active/world-sync" --ledger-path "$LEDGER_PATH"`
  - Evidence recorder (recommended after dispatch; `RUN_ID` is the numeric id in the Actions run URL, e.g. `.../actions/runs/123456789`):
    - `scripts/ci-audit/ci_audit_record.sh --ledger-path "$LEDGER_PATH" --kind ci-testing --orch-branch "feat/world-sync" --run-id "$RUN_ID" --tested-sha "$CHECKOUT_SHA" --feature-dir "docs/project_management/packs/active/world-sync"`
    - `scripts/ci-audit/ci_audit_record.sh --ledger-path "$LEDGER_PATH" --kind feature-smoke --orch-branch "feat/world-sync" --run-id "$RUN_ID" --tested-sha "$CHECKOUT_SHA" --feature-dir "docs/project_management/packs/active/world-sync"`

Policy note:
- Docs/planning-only changes (anything under `docs/`) may skip all CI/smoke **only when** the advisory audit outputs `DIFF_CLASS=docs_only` and `RECOMMEND=skip`.

Runner readiness:
- Required self-hosted runners exist and are labeled correctly:
  - `Feature Smoke (self-hosted linux)`: `runs-on: [self-hosted, Linux, linux-host]`
  - `Feature Smoke (self-hosted macos)`: `runs-on: [self-hosted, macOS]`

Run ids/URLs (if executed during preflight):
- CI compile parity:
- Linux smoke:
- macOS smoke:

## 4) Required Fixes Before Starting The First Slice (if any)

- None.
