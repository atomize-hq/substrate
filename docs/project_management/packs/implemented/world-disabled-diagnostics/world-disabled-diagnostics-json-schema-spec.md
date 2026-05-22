# world-disabled-diagnostics — JSON schema spec (health + shim doctor)

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

This spec is authoritative for the **additive JSON output contract** introduced by ADR-0036 for:
- `substrate shim doctor --json`
- `substrate health --json`

Out of scope (authoritative elsewhere; this feature MUST NOT redefine):
- Effective-config precedence and `SUBSTRATE_OVERRIDE_*` semantics: `docs/reference/env/contract.md`
- Config key schema/meaning for `world.enabled`: `docs/CONFIGURATION.md`
- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Compatibility policy (explicit)

- Backward compatibility: additive-only; **no renames/removals** of existing fields.
- Forward compatibility: consumers MUST ignore unknown fields and unknown enum values.
- Deprecation policy: none introduced by this feature.

## Schema additions (authoritative)

### `substrate shim doctor --json`

#### `.world.status` (new)
- Type: string enum
- Presence:
  - If the `.world` object is present, `.world.status` MUST be present.
  - If `.world` is absent, `.world.status` is absent.
- Allowed values:
  - `healthy`
  - `needs_attention`
  - `disabled`
  - `unknown`
- Semantics:
  - `disabled`: effective config resolved `world.enabled=false`; world backend probing was skipped.
  - `healthy`: effective config resolved `world.enabled=true` and the world backend probe reported healthy.
  - `needs_attention`: effective config resolved `world.enabled=true` and the world backend probe reported unhealthy/unreachable (actionable error expected).
  - `unknown`: world backend status could not be determined (should be rare; indicates an internal/reporting failure).

Legacy fields (unchanged):
- `.world.ok` remains a boolean and MUST NOT be used alone to infer disabled/skipped status; `.world.status` is the canonical machine-readable classifier.

#### `.world_deps.status` (new)
- Type: string enum
- Presence:
  - If the `.world_deps` object is present, `.world_deps.status` MUST be present.
  - If `.world_deps` is absent, `.world_deps.status` is absent.
- Allowed values:
  - `ok`
  - `error`
  - `skipped_disabled`
  - `unknown`
- Semantics:
  - `skipped_disabled`: effective config resolved `world.enabled=false`; world-deps “applied” probing was skipped.
  - `ok`: snapshot was collected and applied probing succeeded (even if some enabled deps are missing/blocked).
  - `error`: snapshot unavailable or applied probing failed (error details are expected in existing legacy error fields).
  - `unknown`: status could not be determined (should be rare; indicates an internal/reporting failure).

Legacy fields (unchanged):
- `.world_deps.error` and `.world_deps.report.applied_error` remain the legacy error surfaces for world-deps snapshot collection / applied probing.
- `.world_deps.status` is the canonical machine-readable classifier; legacy error strings MUST NOT be used as the only “skipped because disabled” signal.

### `substrate health --json`

Health JSON is:
- top-level `shim`: the full `substrate shim doctor --json` payload
- top-level `summary`: a derived summary

Canonical status enums for health are therefore:
- `.shim.world.status`
- `.shim.world_deps.status`

## Emission and absence rules (authoritative)

### Disabled world (`world.enabled=false`)

When effective config resolves `world.enabled=false`:
- `substrate shim doctor --json`:
  - `.world.status` MUST be `disabled`.
  - `.world.details` MUST be omitted (no world backend probing for diagnostics).
  - `.world.error` MUST be omitted (disabled is non-error).
  - `.world_deps.status` MUST be `skipped_disabled`.
  - `.world_deps.report` MUST be omitted (no applied probing).
  - `.world_deps.error` MUST be omitted (skipped is non-error).
- `substrate health --json` summary:
  - `.summary.world_ok` MUST be `null`.
  - `.summary.world_error` MUST be omitted.
  - `.summary.world_deps_error` MUST be omitted.
  - `.summary.world_deps_missing` MUST be `[]`.
  - `.summary.world_deps_blocked` MUST be `[]`.
  - `.summary.failures` MUST NOT include world-backend/world-deps probe failures solely due to the disabled short-circuit.

### Enabled world (`world.enabled=true`)

When effective config resolves `world.enabled=true`:
- `.world.status` MUST NOT be `disabled`.
- `.world_deps.status` MUST NOT be `skipped_disabled`.
- `.world_deps.status` MUST be `error` when either:
  - `.world_deps.error` is present, or
  - `.world_deps.report.applied_error` is present.

## Examples (authoritative)

Examples below show only the fields relevant to this feature. Other fields in the payload are unchanged.

### `substrate shim doctor --json` — disabled world

```json
{
  "manifest": { "base": "/tmp/fixture/manager_hooks.yaml", "overlay": null, "overlay_exists": false },
  "path": {
    "shim_dir": "/tmp/fixture/.substrate/shims",
    "shim_dir_exists": true,
    "path_first_entry": "/tmp/fixture/.substrate/shims",
    "host_contains_shims": false,
    "shim_first_in_path": true,
    "bashenv_path": "/tmp/fixture/.substrate_bashenv",
    "bashenv_exists": false
  },
  "trace_log": "/tmp/fixture/.substrate/trace.jsonl",
  "skip_all_requested": false,
  "states": [],
  "hints": [],
  "world": { "status": "disabled", "ok": false, "platform": "linux" },
  "world_deps": { "status": "skipped_disabled", "source": "disabled" }
}
```

### `substrate shim doctor --json` — enabled world, backend needs attention

```json
{
  "manifest": { "base": "/tmp/fixture/manager_hooks.yaml", "overlay": null, "overlay_exists": false },
  "path": {
    "shim_dir": "/tmp/fixture/.substrate/shims",
    "shim_dir_exists": true,
    "path_first_entry": "/tmp/fixture/.substrate/shims",
    "host_contains_shims": false,
    "shim_first_in_path": true,
    "bashenv_path": "/tmp/fixture/.substrate_bashenv",
    "bashenv_exists": false
  },
  "trace_log": "/tmp/fixture/.substrate/trace.jsonl",
  "skip_all_requested": false,
  "states": [],
  "hints": [],
  "world": {
    "status": "needs_attention",
    "ok": false,
    "platform": "linux",
    "source": "command",
    "error": "failed to gather world doctor output: world-service socket probe failed"
  },
  "world_deps": {
    "status": "error",
    "error": "failed to collect world deps snapshot: world backend unavailable",
    "source": "command"
  }
}
```

### `substrate shim doctor --json` — enabled world, backend healthy

```json
{
  "manifest": { "base": "/tmp/fixture/manager_hooks.yaml", "overlay": null, "overlay_exists": false },
  "path": {
    "shim_dir": "/tmp/fixture/.substrate/shims",
    "shim_dir_exists": true,
    "path_first_entry": "/tmp/fixture/.substrate/shims",
    "host_contains_shims": false,
    "shim_first_in_path": true,
    "bashenv_path": "/tmp/fixture/.substrate_bashenv",
    "bashenv_exists": false
  },
  "trace_log": "/tmp/fixture/.substrate/trace.jsonl",
  "skip_all_requested": false,
  "states": [],
  "hints": [],
  "world": { "status": "healthy", "ok": true, "platform": "linux", "source": "command" },
  "world_deps": {
    "status": "ok",
    "report": {
      "schema_version": 1,
      "cwd": "/tmp/fixture",
      "inventory_packages": 0,
      "inventory_bundles": 0,
      "inventory_mode": "merged",
      "builtins": "enabled",
      "enabled": [],
      "applied": []
    },
    "source": "command"
  }
}
```

### `substrate health --json` — disabled world

```json
{
  "shim": {
    "manifest": { "base": "/tmp/fixture/manager_hooks.yaml", "overlay": null, "overlay_exists": false },
    "path": {
      "shim_dir": "/tmp/fixture/.substrate/shims",
      "shim_dir_exists": true,
      "path_first_entry": "/tmp/fixture/.substrate/shims",
      "host_contains_shims": false,
      "shim_first_in_path": true,
      "bashenv_path": "/tmp/fixture/.substrate_bashenv",
      "bashenv_exists": false
    },
    "trace_log": "/tmp/fixture/.substrate/trace.jsonl",
    "skip_all_requested": false,
    "states": [],
    "hints": [],
    "world": { "status": "disabled", "ok": false, "platform": "linux" },
    "world_deps": { "status": "skipped_disabled", "source": "disabled" }
  },
  "summary": {
    "ok": true,
    "missing_managers": [],
    "skip_manager_init": false,
    "world_ok": null,
    "world_deps_missing": [],
    "world_deps_blocked": []
  }
}
```

### `substrate health --json` — enabled world, backend needs attention

```json
{
  "shim": {
    "manifest": { "base": "/tmp/fixture/manager_hooks.yaml", "overlay": null, "overlay_exists": false },
    "path": {
      "shim_dir": "/tmp/fixture/.substrate/shims",
      "shim_dir_exists": true,
      "path_first_entry": "/tmp/fixture/.substrate/shims",
      "host_contains_shims": false,
      "shim_first_in_path": true,
      "bashenv_path": "/tmp/fixture/.substrate_bashenv",
      "bashenv_exists": false
    },
    "trace_log": "/tmp/fixture/.substrate/trace.jsonl",
    "skip_all_requested": false,
    "states": [],
    "hints": [],
    "world": {
      "status": "needs_attention",
      "ok": false,
      "platform": "linux",
      "source": "command",
      "error": "failed to gather world doctor output: world-service socket probe failed"
    },
    "world_deps": {
      "status": "error",
      "error": "failed to collect world deps snapshot: world backend unavailable",
      "source": "command"
    }
  },
  "summary": {
    "ok": false,
    "missing_managers": [],
    "skip_manager_init": false,
    "world_ok": false,
    "world_error": "failed to gather world doctor output: world-service socket probe failed",
    "world_deps_missing": [],
    "world_deps_blocked": [],
    "world_deps_error": "failed to collect world deps snapshot: world backend unavailable",
    "failures": [
      "world backend health check failed",
      "world backend error: failed to gather world doctor output: world-service socket probe failed",
      "world deps unavailable: failed to collect world deps snapshot: world backend unavailable"
    ]
  }
}
```

### `substrate health --json` — enabled world, backend healthy

```json
{
  "shim": {
    "manifest": { "base": "/tmp/fixture/manager_hooks.yaml", "overlay": null, "overlay_exists": false },
    "path": {
      "shim_dir": "/tmp/fixture/.substrate/shims",
      "shim_dir_exists": true,
      "path_first_entry": "/tmp/fixture/.substrate/shims",
      "host_contains_shims": false,
      "shim_first_in_path": true,
      "bashenv_path": "/tmp/fixture/.substrate_bashenv",
      "bashenv_exists": false
    },
    "trace_log": "/tmp/fixture/.substrate/trace.jsonl",
    "skip_all_requested": false,
    "states": [],
    "hints": [],
    "world": { "status": "healthy", "ok": true, "platform": "linux", "source": "command" },
    "world_deps": {
      "status": "ok",
      "report": {
        "schema_version": 1,
        "cwd": "/tmp/fixture",
        "inventory_packages": 0,
        "inventory_bundles": 0,
        "inventory_mode": "merged",
        "builtins": "enabled",
        "enabled": [],
        "applied": []
      },
      "source": "command"
    }
  },
  "summary": {
    "ok": true,
    "missing_managers": [],
    "skip_manager_init": false,
    "world_ok": true,
    "world_deps_missing": [],
    "world_deps_blocked": []
  }
}
```

## Error model (explicit)

- Effective-config resolution failures are not represented as JSON statuses; the command MUST fail and emit a user-facing error (see `contract.md` for exit-code mapping).
- JSON serialization failures are command execution failures (non-zero exit) and do not produce a partial payload.

## Security / redaction (explicit)

- The new status enums (`world.status`, `world_deps.status`) MUST NOT contain secrets.
- Disabled/skipped states MUST NOT embed config values, socket paths, or other potentially sensitive attribution details (owned by future attribution work).
