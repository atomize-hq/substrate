# Execution Preflight Gate Report — policy-patch-only-broker-effective-resolution

Date (UTC): 2026-01-17T04:39:32Z

Standard:
- `docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/next/policy-patch-only-broker-effective-resolution/`

## Recommendation

RECOMMENDATION: REVISE

## Inputs Reviewed

- Planning quality gate is `ACCEPT` (`docs/project_management/next/policy-patch-only-broker-effective-resolution/quality_gate_report.md`): VERIFIED
- ADR accepted and still matches intent (`docs/project_management/next/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`): NOT VERIFIED (Status: Draft)
- Planning Pack complete (`plan.md`, `tasks.json`, `session_log.md`, specs, kickoff prompts): VERIFIED
- Triad sizing is appropriate (each slice is one behavior delta; no “grab bag” slices): VERIFIED
- Required planning artifacts exist (`integration_map.md`, `manual_testing_playbook.md`): VERIFIED
- Cross-platform plan is explicit (tasks.json meta: behavior + CI parity platforms): VERIFIED

## 0) Slice Sizing (one behavior delta)

- Slices reviewed:
  - C0: broker canonical resolver + CLI delegation
  - C1: fail-closed execution paths + docs alignment
- Any required splits before starting execution:
- NONE

## 1) Cross-Platform Coverage (explicit and correct)

From `docs/project_management/next/policy-patch-only-broker-effective-resolution/tasks.json` meta:
- Declared behavior platforms (smoke required): `["linux","macos","windows"]`
- Declared CI parity platforms (parity required): `["linux","macos","windows"]`
- WSL required: `false`

## 2) Smoke Scripts Are Not “Toy” Checks

Manual playbook:
- `docs/project_management/next/policy-patch-only-broker-effective-resolution/manual_testing_playbook.md`

Smoke scripts:
- Linux smoke: `docs/project_management/next/policy-patch-only-broker-effective-resolution/smoke/linux-smoke.sh`
- macOS smoke: `docs/project_management/next/policy-patch-only-broker-effective-resolution/smoke/macos-smoke.sh`
- Windows smoke: `docs/project_management/next/policy-patch-only-broker-effective-resolution/smoke/windows-smoke.ps1`

Verified by inspection:
- The smoke scripts build `substrate`, set an isolated `SUBSTRATE_HOME`, create a scratch workspace, and validate real CLI behavior.
- C0 assertions:
  - Effective resolution includes workspace patch precedence over global (`world_fs_require_world=false`).
  - `.substrate/workspace.disabled` prevents the workspace patch from contributing (`world_fs_require_world=true` from global).
- C1 assertions:
  - Invalid YAML in a policy patch causes fail-closed behavior: exit `2` and the command is not executed.
- Manual playbook’s additional C1-required validation (shim + world-agent) is explicitly called out as not covered by smoke and is owned by `C1-integ-core`.

## 3) CI Dispatch Path Is Runnable (if applicable)

Integration task dispatch commands live in `docs/project_management/next/policy-patch-only-broker-effective-resolution/tasks.json` integration checklists.

Validated (dry-run only; not dispatched during this preflight):
- `make -n ci-compile-parity CI_WORKFLOW_REF="feat/policy-patch-only-broker-effective-resolution" CI_REMOTE=origin CI_CLEANUP=1` → exit `0`
- `make -n feature-smoke FEATURE_DIR="docs/project_management/next/policy-patch-only-broker-effective-resolution" PLATFORM=behavior SMOKE_SLICE_ID="C0" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/policy-patch-only-broker-effective-resolution" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1` → exit `0`

Runner labels (workflow contract; runner availability not validated in this preflight):
- Linux self-hosted: `[self-hosted, Linux, linux-host]` (`.github/workflows/feature-smoke.yml`)
- macOS self-hosted: `[self-hosted, macOS]` (`.github/workflows/feature-smoke.yml`)
- Windows self-hosted: `[self-hosted, Windows]` (`.github/workflows/feature-smoke.yml`)

Blocking issue:
- `git ls-remote --heads origin feat/policy-patch-only-broker-effective-resolution` returns no refs, so dispatching with `WORKFLOW_REF="feat/policy-patch-only-broker-effective-resolution"` will fail until the branch is pushed.

## 4) Required Fixes Before Starting C0 (if any)

- Approve ADR-0013 (update `docs/project_management/next/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md` status to `Approved`, or otherwise record acceptance) and re-run this preflight.
- Push the orchestration branch ref to `origin` so `WORKFLOW_REF="feat/policy-patch-only-broker-effective-resolution"` CI dispatch commands are runnable (e.g., `git push -u origin feat/policy-patch-only-broker-effective-resolution`), then re-run this preflight.
