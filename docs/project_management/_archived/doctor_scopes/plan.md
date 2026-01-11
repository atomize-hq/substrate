# doctor_scopes — plan

## Scope
- Feature directory: `docs/project_management/_archived/doctor_scopes/`
- Orchestration branch: `feat/doctor-scopes`
- ADR: `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md`

## Goal
- Produce a clear operator-facing split between:
  - `substrate host doctor`: host/transport readiness only (no guest-kernel inference).
  - `substrate world doctor`: combined `host` + `world` report, where `world` facts come from a world-agent endpoint (guest kernel + agent privileges).

## Guardrails (non-negotiable)
- Specs are the single source of truth.
- Planning Pack docs are edited only on the orchestration branch.
- Do not edit planning docs inside the worktree.
- Doctor commands are passive diagnostics only (no provisioning, no agent spawning, no VM start).

## Platforms
- Behavior platforms required (smoke required): `linux`, `macos`
- CI parity platforms required (compile parity required): `linux`, `macos`, `windows`
- WSL required: `false`

## Execution gates
- Planning quality gate: `docs/project_management/_archived/doctor_scopes/quality_gate_report.md` (required before triads begin)
- Execution preflight gate: `docs/project_management/_archived/doctor_scopes/execution_preflight_report.md` (required because `execution_gates=true`)
- Slice closeout gate: `docs/project_management/_archived/doctor_scopes/DS0-closeout_report.md` (required as part of `DS0-integ`)

## Triads
- DS0: split doctor into host vs world scopes (code/test/integ)

## Smoke
- Linux: `docs/project_management/_archived/doctor_scopes/smoke/linux-smoke.sh`
- macOS: `docs/project_management/_archived/doctor_scopes/smoke/macos-smoke.sh`
- Windows: `docs/project_management/_archived/doctor_scopes/smoke/windows-smoke.ps1` (CI parity only; smoke must be a no-op)
