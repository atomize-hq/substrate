# Integration Map — Policy + Config Precedence (ADR-0005)

## Scope
- Effective config precedence change in workspace context (workspace config overrides `SUBSTRATE_*` env vars).
- Test and smoke validation to lock the precedence contract.

## Non-Scope
- Policy precedence changes.
- Workspace discovery changes.
- Config schema changes.
- Env script generation and ownership changes.

## End-to-end data flow (inputs → derived state → actions)
- Inputs:
  - `cwd`
  - `<workspace_root>/.substrate/workspace.yaml` (workspace config)
  - `$SUBSTRATE_HOME/config.yaml` (global config; optional)
  - process environment (`SUBSTRATE_*`)
  - CLI flags (subset of config keys)
- Derived state:
  - `<workspace_root>` computed from `cwd` via walk-up marker discovery
  - effective config computed via precedence rules in ADR-0005
- Actions:
  - `substrate config show|set` read/write workspace config and render effective config
  - any command that consumes effective config uses the same resolver
- Outputs:
  - effective config (YAML/JSON) for workspace-scoped commands
  - exit codes per `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Component map (what changes where)
- Effective config resolver:
  - `crates/shell/src/execution/config_model.rs`
- CLI command surface that demonstrates the contract:
  - `crates/shell/src/execution/config_cmd.rs` (no contract change; used for tests)
- Consumers that inherit the new precedence:
  - `crates/shell/src/execution/settings/builder.rs`
  - `crates/shell/src/execution/invocation/plan.rs`
- Tests:
  - `crates/shell/tests/config_show.rs` (precedence assertions)
- Planning-pack validation:
  - `docs/project_management/next/policy_and_config_precedence/manual_testing_playbook.md`
  - `docs/project_management/next/policy_and_config_precedence/smoke/*`

## Composition with adjacent tracks
- Baseline config/policy model comes from ADR-0003:
  - `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`
- This change is a narrow correction that does not introduce new config keys and does not change workspace discovery.

## Sequencing alignment
- Sequencing spine entry: `docs/project_management/next/sequencing.json` → `policy_and_config_precedence`
- No prerequisite integration task IDs are required because ADR-0003 is already integrated; this sprint is a follow-up correction.

