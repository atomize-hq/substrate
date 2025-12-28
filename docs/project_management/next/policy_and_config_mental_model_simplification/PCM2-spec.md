# PCM2 — Policy Mode + Routing Semantics (ADR-0003)

## Scope (authoritative)
Implement ADR-0003 runtime semantics for policy mode, command-level evaluation, and host/world selection rules.

### Policy mode (`disabled|observe|enforce`)
Source:
- `policy.mode` from effective config (see `PCM0`).

Semantics:
- `disabled`:
  - Substrate does not evaluate policy (no allow/deny decision computation).
  - Trace/logging still occurs.
- `observe`:
  - Substrate evaluates policy and records the decision.
  - Substrate does not block execution.
- `enforce`:
  - Substrate evaluates policy.
  - Substrate blocks and/or prompts according to policy.

### Command-level evaluation semantics (`observe|enforce`)
When `policy.mode` is `observe` or `enforce`, command evaluation implements:
- Deny check:
  - If the command matches any `cmd_denied` pattern, it is denied.
  - In `enforce`, Substrate blocks execution.
  - In `observe`, Substrate allows execution and records “would deny”.
- Allow check:
  - If the command matches any `cmd_allowed` pattern, it is allowed.
  - If neither denied nor allowed matches, it is treated as “unclassified” and is allowed in observe and enforce.
- Isolation check:
  - If the command matches any `cmd_isolated` pattern, it produces a “requires world” constraint.

### “Requires world” constraints (policy-derived)
When `policy.mode` is `observe` or `enforce`, these conditions produce a “requires world” constraint:
- `world_fs.require_world=true`
- `world_fs.mode=read_only`
- `world_fs.isolation=full`
- Any `cmd_isolated` match

Resolution rules:
- If a “requires world” constraint applies:
  - In `enforce`: Substrate runs the command in world; if world execution is not possible, Substrate fails closed (hard error).
  - In `observe`: Substrate records “would require world” and does not change world/host selection solely due to the requirement.

### Host vs world selection
World selection inputs (highest to lowest):
1. CLI: `--world` / `--no-world`
2. Env: `SUBSTRATE_WORLD=enabled|disabled`
3. Config: `world.enabled`

Rules:
- If `--no-world` is provided:
  - In `enforce`, if a “requires world” constraint applies, Substrate fails closed (hard error).
  - Otherwise, Substrate runs on host.
- If `--world` is provided, Substrate attempts world execution regardless of config/env.

World backend availability rule:
- If world execution is required (`--world` or policy-derived requirement in `enforce`) and the backend is unavailable, Substrate fails closed (hard error).
- If world execution is selected by config/env and not required by `enforce`, and the backend is unavailable, Substrate falls back to host execution and records the fallback reason.

### Approval “save to policy” write target selection (enforce mode)
When enforce mode requests interactive approval and the user selects “save this approval to policy file”:
- If `<workspace_root>` exists: write to `<workspace_root>/.substrate/policy.yaml` (create if missing).
- Else: write to `$SUBSTRATE_HOME/policy.yaml` (create if missing).

No other write targets are permitted.

## Non-scope (explicit)
- Policy/config schema and CLI (`PCM0`, `PCM1`).
- Env scripts and world enable home semantics (`PCM3`).

## Acceptance (implementation outcomes)
- `disabled|observe|enforce` semantics match ADR-0003 exactly.
- “requires world” constraints and fail-closed behavior in enforce mode match ADR-0003 exactly.
- Host/world selection precedence matches ADR-0003 exactly.
- “save to policy” writes select the correct on-disk target exactly.

