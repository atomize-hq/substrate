# WCU5 Spec — Docs + smoke + manual playbook parity (includes ADR-0012 Phase A/B validation)

References:
- `docs/project_management/_archived/workspace-config-policy-unification/plan.md`
- `docs/project_management/_archived/workspace-config-policy-unification/manual_testing_playbook.md`
- `docs/project_management/_archived/workspace-config-policy-unification/smoke/linux-smoke.sh`
- `docs/project_management/_archived/workspace-config-policy-unification/smoke/macos-smoke.sh`
- `docs/project_management/_archived/workspace-config-policy-unification/smoke/windows-smoke.ps1`
- `docs/project_management/_archived/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`

## Scope
- Ensure the feature-local manual testing playbook and all feature smoke scripts remain in parity.
- Ensure the automated smoke journey validates ADR-0012 Phase A/B behaviors for:
  - `world.deps.enabled` effective merge semantics, and
  - multi-source provenance via `config current show --explain`.

## Non-goals
- Adding new feature behavior beyond validating the existing contract.

## Parity contract (authoritative)
The smoke scripts MUST implement a minimal viable subset of the manual playbook that:
- uses a scratch workspace directory and a scratch `SUBSTRATE_HOME`,
- performs config editor mutations at both global and workspace scopes for `world.deps.enabled`,
- performs config editor mutations at both global and workspace scopes for:
  - `world.deps.inventory_mode` (`merged|workspace_only`, `merge_strategy=replace`)
  - `world.deps.builtins` (`enabled|disabled`, `merge_strategy=replace`)
- validates:
  - concat+dedupe ordered-set behavior (including a deliberate duplicate across scopes),
  - `--explain` includes `merge_strategy` and multi-source provenance,
  - `--explain` includes `merge_strategy=replace` and exactly one contributing source layer for the enum keys,
  - workspace disabled marker causes workspace contribution to be ignored (and provenance excludes `workspace_patch`),
  - strict invalid-enum value behavior (exit `2`, no writes; patch bytes unchanged),
  - determinism/idempotence: re-running the `current show` + `--explain` commands without changes yields identical outputs.

## Validation requirements (authoritative)
- WCU5 tests (if any) must cover smoke assertions at unit/integration level where appropriate.
- WCU5 integration must:
  - dispatch feature smoke for behavior platforms via `make feature-smoke`, and
  - record run ids/URLs and parity notes in `WCU5-closeout_report.md`.
