# Manual testing playbook — llm_and_agent_config_policy_surface

This playbook validates Phase 3 (ADR-0027) schema + precedence behavior for new `llm.*` and `agents.*` keys.

Prereqs:
- `substrate` is available in `PATH` (or set `SUBSTRATE_BIN=/path/to/substrate` for smoke scripts).

## 1) Basic schema acceptance (global)
1. `substrate config global init --force`
2. `substrate policy global init --force`
3. Set minimal config keys:
   - `substrate config global set llm.enabled=true llm.gateway.enabled=true llm.routing.default_backend=cli:codex`
   - `substrate config global set agents.enabled=true agents.defaults.execution.scope=world`
4. Set minimal policy keys:
   - `substrate policy global set llm.allowed_backends+=cli:codex`
   - `substrate policy global set agents.allowed_backends+=cli:codex`
5. Verify effective views include the keys:
   - `substrate config current show --explain`
   - `substrate policy current show --explain`
6. Verify unknown keys are rejected (exit code `2`):
   - `substrate config global set llm.unknown_key=true`
   - `substrate policy global set agents.unknown_key=true`

## 2) Workspace precedence (override global)
1. Create a temp workspace and init:
   - `mkdir -p /tmp/substrate-llm-policy-surface && cd /tmp/substrate-llm-policy-surface`
   - `substrate workspace init --force`
2. Seed global config/policy enables (as above).
3. Set workspace overrides:
   - `substrate config workspace set llm.routing.default_backend=cli:claude_code`
   - `substrate policy workspace set llm.allowed_backends+=cli:claude_code`
4. Verify `--explain` shows the override sources:
   - `substrate config current show --explain`
   - `substrate policy current show --explain`

## 3) Agent inventory strictness + restriction-only overlays

These steps validate the agent inventory directory surface and `policy_overlay` broadening rejection.

1. Create an isolated temp root:
   - `export TMP_ROOT="$(mktemp -d)"`
   - `export SUBSTRATE_HOME="$TMP_ROOT/substrate-home"`
   - `mkdir -p "$SUBSTRATE_HOME/agents"`
2. Initialize patches:
   - `substrate config global init --force`
   - `substrate policy global init --force`
3. Create a minimal valid agent file:
   - Create `$SUBSTRATE_HOME/agents/codex.yaml` with:
     - `id: codex` and a valid `config` section.
4. Validate:
   - `substrate agents validate` (exit `0`)
5. Unknown-key rejection (exit `2`):
   - Create `$SUBSTRATE_HOME/agents/bad_unknown.yaml` with an unknown top-level key.
   - `substrate agents validate`
6. Filename/id mismatch rejection (exit `2`):
   - Create `$SUBSTRATE_HOME/agents/mismatch.yaml` with YAML `id: other`.
   - `substrate agents validate`
7. Overlay broadening rejection (exit `2`):
   - Ensure effective base policy has `llm.secrets.env_allowed=[]` (default).
   - Create `$SUBSTRATE_HOME/agents/broaden.yaml` with:
     - `policy_overlay.llm.secrets.env_allowed=["OPENAI_API_KEY"]`
   - `substrate agents validate`

## Smoke scripts
- Linux: `docs/project_management/next/llm_and_agent_config_policy_surface/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/llm_and_agent_config_policy_surface/smoke/macos-smoke.sh`
