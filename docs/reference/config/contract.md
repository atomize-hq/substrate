# Configuration Contract

This file is scaffolding: it will define the authoritative configuration file contract (paths, precedence, schema constraints).

Suggested sections:
- Global config: `$SUBSTRATE_HOME/config.yaml`
- Workspace config: `<workspace_root>/.substrate/workspace.yaml`
- Effective config precedence rules (including how CLI flags interact)
- Schema strictness and user error behavior

Existing related docs:
- `docs/CONFIGURATION.md`
- `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`
- `docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md`

