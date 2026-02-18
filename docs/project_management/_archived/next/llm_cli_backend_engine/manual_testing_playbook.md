# Manual testing playbook — llm_cli_backend_engine

Placeholder manual playbook for ADR-0024. Will be expanded once the CLI engine implementation exists.

Intended checks (v1):
- Deny-by-default: `llm.allowed_backends=[]` blocks routing to `cli:*`.
- Allowlist behavior: allowlisted `cli:<agent_id>` routes; non-allowlisted denies.
- Fail-closed routing: with `llm.fail_closed.routing=true`, requests fail when world is unavailable.
- Agent inventory mapping: `cli:<agent_id>` resolves to `~/.substrate/agents/<agent_id>.yaml` (workspace overrides global).
- CLI session mode honors `config.cli.mode` (persistent vs per-request) and records attribution in trace.

