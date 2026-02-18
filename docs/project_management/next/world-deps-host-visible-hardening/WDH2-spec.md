# WDH2 — Exec-time guardrails for host-mounted binaries in host-visible worlds

## Goal
Prevent explicit execution of host-mounted toolchain binaries inside the world when `world_fs.host_visible=true`, unless explicitly allowed by override inputs.

## Inputs (authoritative)
- `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` (host_visible)
- `docs/project_management/next/world-deps-host-visible-hardening/decision_register.md` (DR-0005, DR-0007)

## Contract

### Default posture
When running with `--world` and `world_fs.host_visible=true`, Substrate MUST deny execution of binaries whose resolved executable path indicates a host toolchain location (denylist of path substrings), unless explicitly allowed by an override.

### Default denylist (required)
Execution is denied (exit `5`) when the resolved executable path contains any of the following substrings:
- `/.nvm/`
- `/.config/nvm/`
- `/.pyenv/`
- `/.cargo/bin/`
- `/.local/bin/`
- `/.bun/bin/`

The guard MUST apply to:
- PATH-resolved commands (defense in depth), and
- explicit absolute paths (the primary bypass it is intended to close).

### Overrides (required)
Support explicit override-input env vars (ADR-0006 taxonomy):
- `SUBSTRATE_OVERRIDE_WORLD_EXEC_GUARD=0|1`
  - default: `1` when `world_fs.host_visible=true`
- `SUBSTRATE_OVERRIDE_WORLD_EXEC_GUARD_DENY_CONTAINS="<substr1>,<substr2>,..."`
  - when set, replaces the default denylist entirely (comma-separated substrings)

Backlog (policy/config):
- Add a persistent policy/config lever for the denylist (global + workspace patchable) so operators do not need env vars. Track: `docs/BACKLOG.md` (“P1 – Policy/config lever for world exec-guard denylist (host-visible hardening)”).

### Error behavior
When execution is denied:
- exit code MUST be `5`
- stderr MUST include an actionable remediation: “enable a world-deps package instead” and/or “adjust `SUBSTRATE_OVERRIDE_WORLD_EXEC_GUARD_DENY_CONTAINS` / disable the guard explicitly if you accept the risk”.

## Exit codes
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- 5: host-binary execution denied by safety/policy

## Acceptance criteria
- With `world_fs.host_visible=true` and default guard settings, running an explicit host toolchain path inside the world fails with exit `5`.
- The denial does not depend on PATH (i.e., it triggers on explicit paths too).
- Setting `world.env.inherit_from_host=true` does not bypass the guard.

## Out of scope
- Perfect classification for every possible mount topology on every backend (start with Linux + Lima and expand).
