# warn-config-global-show-workspace-overrides — spec manifest

This file enumerates every contract surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0019-warn-config-global-show-when-workspace-config-overrides.md`

## Required spec documents (authoritative)

Spec templates:
- `docs/project_management/system/templates/spec/`

- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/spec_manifest.md` — spec selection + ownership map (this file)
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/impact_map.md` — touch set + cascading implications + cross-queue scan
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/ci_checkpoint_plan.md` — cross-platform CI cadence (automation + cross-platform)
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/plan.md` — execution runbook
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/tasks.json` — triad task graph + acceptance criteria
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/slices/C0/C0-spec.md` — slice behavior + acceptance criteria
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/contract.md` — CLI contract (stdout/stderr/exit codes) + path semantics
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/decision_register.md` — A/B decisions and selections
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/manual_testing_playbook.md` — manual validation (authoritative)
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/smoke/linux-smoke.sh` — Linux behavior smoke
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/smoke/macos-smoke.sh` — macOS behavior smoke
- `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/smoke/windows-smoke.ps1` — Windows behavior smoke

## Coverage matrix (surface → authoritative doc)

| Surface | Authoritative doc | What is explicitly defined |
| --- | --- | --- |
| CLI commands/flags/defaults | `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/contract.md` | stdout/stderr content rules, note template text, `--json` invariants |
| Config file paths/precedence/schema | `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/contract.md` | file paths, precedence reminder, workspace enablement definition reference |
| Exit code meanings | `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/contract.md` | success/unchanged failure posture; no new failures from workspace YAML |
| Environment variables | `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/contract.md` | `SUBSTRATE_HOME` path use for the global patch (no new env vars introduced) |
| Data schema / file format | `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/contract.md` | config patch serialization remains unchanged (YAML/JSON), workspace YAML parse failures are non-fatal |
| Platform parity | `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/contract.md` | note emits a platform-native path display for `<workspace_root>/.substrate/workspace.yaml` |
| Validation (manual) | `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/manual_testing_playbook.md` | deterministic cases and expected outputs |
| Validation (automation smoke) | `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/smoke/*` | smoke mirrors manual cases and asserts stdout is uncontaminated |
| Slice acceptance | `docs/project_management/packs/active/warn-config-global-show-workspace-overrides/slices/C0/C0-spec.md` | exact trigger conditions and invariants for the new note |

## Determinism checklist (must be satisfied before quality gate)

For the docs above, confirm they define:
- Inputs and precedence (global patch vs workspace patch).
- Defaults and absence semantics (empty workspace patch, outside workspace).
- Error model and failure posture (workspace patch invalid YAML is non-fatal).
- Output invariants (stderr-only note, stdout patch-only, JSON parseability).
- Platform guarantees (Linux/macOS/Windows behavior parity; path display form is platform-native).
