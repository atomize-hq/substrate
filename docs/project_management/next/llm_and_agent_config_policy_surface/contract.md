# llm_and_agent_config_policy_surface — contract

This document is the operator-facing contract summary for ADR-0027.

Authoritative inputs:
- ADR: `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md`
- Schema: `docs/project_management/next/llm_and_agent_config_policy_surface/SCHEMA.md`
- Decisions: `docs/project_management/next/llm_and_agent_config_policy_surface/decision_register.md`

## What changes
- Substrate’s LLM + agent features are governed via the existing layered config/policy patch files:
  - Config:
    - Global: `$SUBSTRATE_HOME/config.yaml` (default `~/.substrate/config.yaml`)
    - Workspace: `<workspace_root>/.substrate/workspace.yaml`
  - Policy:
    - Global: `$SUBSTRATE_HOME/policy.yaml` (default `~/.substrate/policy.yaml`)
    - Workspace: `<workspace_root>/.substrate/policy.yaml`
- Substrate gains an agent inventory directory model (one file per agent):
  - Global: `$SUBSTRATE_HOME/agents/<agent_id>.yaml` (default `~/.substrate/agents/<agent_id>.yaml`)
  - Workspace: `<workspace_root>/.substrate/agents/<agent_id>.yaml`
- New key families are introduced (new keys only; no new file families):
  - Config: `llm.*`, `agents.*`
  - Policy: `llm.*`, `agents.*`, `workflow.router.*` (router daemon indirect execution; Phase 8 additive)

## Non-negotiable invariants
- **Strict schema.** Unknown keys in config/policy patches are hard errors (exit code `2`).
- **Fail-closed by default.**
  - LLM operations are disabled unless:
    - config enables them (`llm.enabled=true`), AND
    - the selected backend id is allowed by policy (`llm.allowed_backends`).
  - Agent hub operations are disabled unless:
    - config enables them (`agents.enabled=true`), AND
    - the selected backend id(s) are allowed by policy (`agents.allowed_backends`).
- **World boundary is policy-owned.**
  - If `llm.fail_closed.routing=true`, any attempt to run LLM operations outside a world boundary must fail closed.
  - `llm.gateway.mode=host_only` is only permissible when effective policy has `llm.fail_closed.routing=false` (see schema + ADR).
- **Backend allowlists are deny-by-default.**
  - If `llm.allowed_backends=[]` or `agents.allowed_backends=[]`, routing must fail closed (no implicit backend selection).
- **Per-agent policy overlays can only tighten.**
  - Agent files MAY include a `policy_overlay`, but it MUST be restriction-only (cannot broaden beyond base policy).
- **No secrets in config/policy files.**
  - API keys/tokens must not be stored in Substrate YAML patches. Backends must use their own login state or environment variables defined by the backend contract.
  - If a backend adapter needs to read host credential material (e.g., a CLI’s existing login state) in order to deliver required auth fields to an in-world component over a Substrate-owned secret channel, that host credential read MUST be explicitly policy-gated (`agents.host_credentials.read.allowed_backends`).
- **Router indirect execution is fail-closed by default.**
  - Router-derived requests/actions (ADR-0029) MUST be explicitly policy-enabled via `workflow.router.enabled=true` and remain guarded by deny-by-default allowlists (rule ids, workflow ids, and target workspace ids).

## Precedence (summary)
- Config effective precedence is unchanged and applies per-key:
  1. CLI flags (world-related flags only)
  2. Workspace config patch (when inside an enabled workspace)
  3. `SUBSTRATE_OVERRIDE_*` environment overrides (ignored when in an enabled workspace)
  4. Global config patch
  5. Built-in defaults
- Policy effective precedence is unchanged and applies per-key:
  1. Workspace policy patch (when inside an enabled workspace)
  2. Global policy patch
  3. Built-in defaults

## Examples

### Minimal enable (LLM + agents)

Global config patch (`~/.substrate/config.yaml`):
```yaml
llm:
  enabled: true
  gateway:
    enabled: true
  routing:
    default_backend: "cli:codex"

agents:
  enabled: true
  defaults:
    execution:
      scope: world
    cli:
      mode: persistent
```

Global policy patch (`~/.substrate/policy.yaml`):
```yaml
llm:
  fail_closed:
    routing: true
  allowed_backends:
    - "cli:codex"
    - "cli:claude_code"

agents:
  allowed_backends:
    - "cli:codex"
    - "cli:claude_code"
  fail_closed:
    routing: true
```

Agent file (`~/.substrate/agents/codex.yaml`):
```yaml
version: 1
id: codex

config:
  kind: cli
  enabled: true
  execution:
    scope: world
  cli:
    binary: codex
    mode: persistent
  capabilities:
    llm: true
```

### CLI patch management (examples)
- Set config keys:
  - `substrate config global set llm.enabled=true llm.gateway.enabled=true llm.routing.default_backend=cli:codex`
  - `substrate config workspace set agents.enabled=true agents.defaults.execution.scope=world`
- Set policy keys:
  - `substrate policy global set llm.allowed_backends+=cli:codex`
  - `substrate policy workspace set agents.allowed_backends+=cli:codex`
