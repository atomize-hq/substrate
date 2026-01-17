# policy-patch-only-broker-effective-resolution — plan

## Scope
- Feature directory: `docs/project_management/next/policy-patch-only-broker-effective-resolution/`
- Orchestration branch: `feat/policy-patch-only-broker-effective-resolution`
- ADR: `docs/project_management/next/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`

## Goal
- Make `policy.yaml` patch-only everywhere and make the broker the canonical resolver for the effective policy used by shell, shim, and world-agent execution paths.

## Non-Goals
- Backwards compatibility or migrations for any legacy policy formats or legacy discovery locations.
- Supporting any full-policy document on disk as a first-class contract.

## Platform scope
- Cross-platform parity is required: Linux/macOS/Windows.
- WSL coverage is not required for this feature.

## Guardrails (non-negotiable)
- Specs are the single source of truth.
- Planning Pack docs are edited only on the orchestration branch.
- Do not edit planning docs inside the worktree.
- Execution triads do not begin until:
  - `docs/project_management/next/policy-patch-only-broker-effective-resolution/quality_gate_report.md` contains `RECOMMENDATION: ACCEPT`
  - `F0-exec-preflight` is completed (when `meta.execution_gates=true`).

## Triads
- C0: Broker canonical resolver + CLI delegation (code/test/integ).
- C1: Fail-closed enforcement across execution paths + docs alignment (code/test/integ).
