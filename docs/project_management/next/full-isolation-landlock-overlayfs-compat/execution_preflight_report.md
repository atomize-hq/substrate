# Execution Preflight Gate Report — full-isolation-landlock-overlayfs-compat

Date (UTC): 2026-01-20T01:52:53Z

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/next/full-isolation-landlock-overlayfs-compat`

## Recommendation

RECOMMENDATION: REVISE

## Inputs Reviewed

- [ ] Planning quality gate is `ACCEPT` (`docs/project_management/next/full-isolation-landlock-overlayfs-compat/quality_gate_report.md`)
- [ ] ADR accepted and still matches intent (`docs/project_management/next/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`)
- [ ] Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts)
- [ ] Triad sizing is one behavior delta (no mixed independent deltas)
- [ ] Required planning artifacts exist: `decision_register.md`, `integration_map.md`, `manual_testing_playbook.md`, `smoke/*`
- [ ] Cross-platform plan is explicit (`tasks.json` meta: behavior + CI parity platforms)

## 0) Slice Sizing (one behavior delta each)

- Slices reviewed:
  - C0 (`docs/project_management/next/full-isolation-landlock-overlayfs-compat/C0-spec.md`)
- Any required splits before starting execution:
  - None

## 1) Cross-Platform Coverage (explicit and correct)

From `docs/project_management/next/full-isolation-landlock-overlayfs-compat/tasks.json` meta:
- Declared behavior platforms (smoke required): `["linux"]`
- Declared CI parity platforms (parity required): `["linux","macos","windows"]`
- WSL required: `false`
- WSL task mode: N/A

Notes:
- If WSL coverage is required, confirm `meta.wsl_required=true` and `meta.wsl_task_mode` is set correctly.
- If using the platform-fix integration model, confirm tasks exist per slice:
  - `X-integ-core`, optional `X-integ-<platform>` (CI parity platforms + optional WSL task when `wsl_task_mode="separate"`), and `X-integ` final.

## 2) Smoke Scripts Are Not “Toy” Checks

Smoke scripts must be a runnable, minimal version of how a careful human would validate the feature.

Manual playbook (when required):
- `docs/project_management/next/full-isolation-landlock-overlayfs-compat/manual_testing_playbook.md`

Smoke scripts to validate (only required for behavior platforms; parity-only platforms may be explicit no-ops):
- Linux smoke: `docs/project_management/next/full-isolation-landlock-overlayfs-compat/smoke/linux-smoke.sh`
- macOS smoke: `docs/project_management/next/full-isolation-landlock-overlayfs-compat/smoke/macos-smoke.sh`
- Windows smoke: `docs/project_management/next/full-isolation-landlock-overlayfs-compat/smoke/windows-smoke.ps1`

Parity notes (map smoke ↔ manual; include concrete assertions):
- Manual step(s):
  - Run `bash docs/project_management/next/full-isolation-landlock-overlayfs-compat/smoke/linux-smoke.sh`.
- Smoke command(s):
  - `bash smoke/linux-smoke.sh`
- Expected output/assertion(s):
  - Exit `0`.
  - Output contains `OK: allowlisted write succeeded` and `OK: denied write remained denied`.

Gaps (must fix before execution begins):
- Complete the Planning Quality Gate report and set `RECOMMENDATION: ACCEPT`.

## 3) CI Dispatch Path Is Runnable (if applicable)

Integration task dispatch commands (copy verbatim from `tasks.json` integration checklists):
- CI compile parity:
- `make ci-compile-parity CI_WORKFLOW_REF="feat/full-isolation-landlock-overlayfs-compat" CI_REMOTE=origin CI_CLEANUP=1`
- Feature Smoke dispatch:
- `make feature-smoke FEATURE_DIR="docs/project_management/next/full-isolation-landlock-overlayfs-compat" PLATFORM=behavior SMOKE_SLICE_ID="C0" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/full-isolation-landlock-overlayfs-compat" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

Runner readiness:
- Required self-hosted runners exist and are labeled correctly:
  - Linux runner (`[self-hosted, Linux, linux-host]`)

Run ids/URLs (if executed during preflight):
- CI compile parity:
- Not executed
- Linux smoke:
- Not executed
- macOS smoke:
- Not required (Linux-only behavior platform)
- Windows smoke:
- Not required (Linux-only behavior platform)
- WSL smoke:
- Not required

## 4) Required Fixes Before Starting C0 (if any)

- Complete `docs/project_management/next/full-isolation-landlock-overlayfs-compat/quality_gate_report.md` with `RECOMMENDATION: ACCEPT`.
- Re-run `make planning-lint FEATURE_DIR="docs/project_management/next/full-isolation-landlock-overlayfs-compat"` and record the command + exit code in the quality gate report.
- Update this report to `RECOMMENDATION: ACCEPT` once the above are complete.
