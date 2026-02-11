# Execution Preflight Gate Report — world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment

Date (UTC): 2026-02-09T23:31:07Z
Reviewed commit: `854ca28b5fa7d60c816fd1c84e6ffb88556b49f8`

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment`

## Recommendation

RECOMMENDATION: **ACCEPT**

Commands run (verbatim) (all PASS):
- `make planning-validate FEATURE_DIR="docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment"`
- `make planning-lint FEATURE_DIR="docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment"`
- `bash -n docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/smoke/_core.sh`
- `bash -n docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/smoke/linux-smoke.sh`
- `bash -n docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/smoke/macos-smoke.sh`

## Inputs Reviewed

- [x] Planning quality gate is `ACCEPT` (`docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/quality_gate_report.md`)
- [x] ADR accepted and still matches intent (`docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`)
- [x] Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts)
- [x] Triad sizing is appropriate (each slice is one behavior delta; no “grab bag” slices)
- [x] Required planning artifacts exist (when required by planning standards): `impact_map.md`, `manual_testing_playbook.md`
- [x] Cross-platform plan is explicit (tasks.json meta: behavior + CI parity platforms, plus WSL mode if needed)

## 0) Slice Sizing (one behavior delta each)

- Slices reviewed:
  - `WFGADAXA0`: effective policy display output is V3-shaped (Appendix A.6) + no-backcompat rendering
  - `WFGADAXA1`: snapshot protocol lockstep (PolicySnapshotV3; schema_version=3 only)
  - `WFGADAXA2`: downstream surfaces + docs alignment to remove V2 operator-facing drift
- Any required splits before starting execution: `NONE`

## 1) Cross-Platform Coverage (explicit and correct)

From `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/tasks.json` meta:
- Declared behavior platforms (smoke required): `["linux"]`
- Declared CI parity platforms (parity required): `["linux","macos"]` (legacy alias: `platforms_required`)
- WSL required: `false`
- WSL task mode: `N/A` (WSL not required)

Notes:
- If WSL coverage is required, confirm `meta.wsl_required=true` and `meta.wsl_task_mode` is set correctly.
- If using the platform-fix integration model, confirm tasks exist per slice:
  - `X-integ-core`, optional `X-integ-<platform>` (CI parity platforms + optional WSL task when `wsl_task_mode="separate"`), and `X-integ` final.

## 2) Smoke Scripts Are Not “Toy” Checks

Smoke scripts must be a runnable, minimal version of how a careful human would validate the feature.

Manual playbook (when required):
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/manual_testing_playbook.md`

Smoke scripts to validate (only required for behavior platforms; parity-only platforms may be explicit no-ops):
- Linux smoke: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/smoke/linux-smoke.sh`
- macOS smoke: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/smoke/macos-smoke.sh`
  - Windows is not supported for this add-on pack; no Windows smoke.

Parity notes (map smoke ↔ manual; include concrete assertions):
- Manual step(s):
  - Manual “Case 1 — Effective policy display is V3-shaped (Appendix A.6)”
- Smoke command(s):
  - Linux: `bash docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/smoke/linux-smoke.sh`
  - macOS: `bash docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/smoke/macos-smoke.sh`
- Expected output/assertion(s):
  - Legacy V2 keys are rejected as config errors (`substrate policy set world_fs.{mode,isolation,require_world}=...` exits `2`).
  - `substrate policy show --json` includes:
    - `world_fs.host_visible == false`
    - `world_fs.{discover,read,write}.allow_list` is a non-empty array
    - `world_fs.{discover,read,write}.deny_list == []` (explicit empty array, not omitted)
    - No V2-shaped keys under `world_fs` (`mode|isolation|require_world|enforcement|read_allowlist|write_allowlist`)
  - `substrate policy show` (YAML) includes `deny_list: []` explicitly at least for discover/read/write (>= 3 occurrences).

Gaps (must fix before execution begins):
- `NONE`

## 3) CI Dispatch Path Is Runnable (if applicable)

Dispatch context:
- `ORCH_REF="feat/world-fs-granular-allow-deny-appendix-addon-v3-alignment"`
- `FEATURE_DIR="docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment"`
- `CHECKOUT_SHA="<sha>"` (set to the commit being tested)

Integration task dispatch commands (copy verbatim from `tasks.json` integration checklists):
- CI compile parity:
  - `make ci-compile-parity CI_WORKFLOW_REF="$ORCH_REF" CI_REMOTE=origin CI_CLEANUP=1 CI_CHECKOUT_REF="$CHECKOUT_SHA"`
- Feature Smoke dispatch:
  - `make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=linux SMOKE_SLICE_ID="WFGADAXA2" SMOKE_CHECKOUT_REF="$CHECKOUT_SHA" RUNNER_KIND=self-hosted WORKFLOW_REF="$ORCH_REF" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=0`
- Advisory CI audit (recommended before any dispatch):
  - `scripts/ci-audit/ci_audit.sh --ledger-path "$FEATURE_DIR/logs/WFGADAXA2/ci-audit/ledger.jsonl" --kind ci-testing --orch-branch "$ORCH_REF"`
  - `scripts/ci-audit/ci_audit.sh --ledger-path "$FEATURE_DIR/logs/WFGADAXA2/ci-audit/ledger.jsonl" --kind feature-smoke --orch-branch "$ORCH_REF" --feature-dir "$FEATURE_DIR"`
  - Evidence recorder (recommended after dispatch):
    - `scripts/ci-audit/ci_audit_record.sh --ledger-path "$FEATURE_DIR/logs/WFGADAXA2/ci-audit/ledger.jsonl" --kind <ci-testing|feature-smoke> --orch-branch "$ORCH_REF" --run-id "<id>" --tested-sha "<sha>" --feature-dir "$FEATURE_DIR"`

Policy note:
- Docs/planning-only changes (anything under `docs/`) may skip all CI/smoke **only when** the advisory audit outputs `DIFF_CLASS=docs_only` and `RECOMMEND=skip`.

Runner readiness:
- Required self-hosted runners exist and are labeled correctly:
  - Not validated during preflight (org/infra dependent).

Run ids/URLs (if executed during preflight):
- CI compile parity: `NOT EXECUTED (preflight only)`
- Linux smoke: `NOT EXECUTED (preflight only)`
- macOS smoke: `NOT EXECUTED (preflight only)`
- Windows smoke: `N/A` (Windows not in behavior platforms scope)
- WSL smoke: `N/A` (WSL not required)

## 4) Required Fixes Before Starting The First Slice (if any)

- `NONE`
