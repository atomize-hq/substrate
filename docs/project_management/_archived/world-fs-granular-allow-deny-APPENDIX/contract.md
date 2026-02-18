# Contract — World FS Allow/Deny Appendix (V3) (Authoritative)

This document is authoritative for Appendix A + B of:
- `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

## 1) Operator-facing policy keys (V3)
- Full schema and validation rules: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md`

Intent-driven keys (high level):
- `world_fs.host_visible=true|false`
- `world_fs.fail_closed.routing=true|false`
- `world_fs.deny_enforcement=strict|prefer_strict|weak`
- `world_fs.caged_required=true|false`
- `world_fs.write.enabled=true|false`

## 1.2) Operator-facing config keys (Appendix B)
Config key:
- `repl.exit_cwd=entered|last_world` (YAML: `repl.exit_cwd`)
  - Default: `entered`
  - `entered`: the exit target is `entered_cwd`
  - `last_world`: the exit target is the last observed `world_cwd`, with deterministic fallback to `entered_cwd` when the target cannot be applied safely (see section 4)

Existing config keys referenced by Appendix B compatibility constraints:
- `world.caged=true|false`
- `world.anchor_mode=project|follow-cwd|custom`

## 1.3) Output contract: effective policy display (`substrate policy show`) (Appendix A.6)
When the effective policy for the current cwd has `world_fs.host_visible=false`, `substrate policy show` MUST render:
- `world_fs.discover`, `world_fs.read`, and `world_fs.write` in the effective policy output,
- under each, both `allow_list` and `deny_list`,
- and when a `deny_list` is empty, it MUST render the empty list explicitly (`deny_list: []` in YAML; `"deny_list":[]` in `--json` output).

When `discover` is defaulted from `read`, the effective output MUST still show `discover` explicitly with its effective allow/deny lists.

## 2) Deterministic failure taxonomy (host)

### 2.1 Policy/config hard errors (exit 2)
Hard errors MUST occur before command execution and before starting the REPL, including:
- invalid schema keys or invalid key combinations
- invalid patterns
- `world_fs.fail_closed.routing=true` combined with effective world disable
- `world_fs.write.enabled=false` combined with `world_fs.fail_closed.routing=false`
- `world_fs.caged_required=true` combined with effective config `world.caged=false`
- `world_fs.caged_required=true` combined with `world.anchor_mode=follow-cwd`
- `repl.exit_cwd` value outside `entered|last_world`

### 2.2 Runtime routing fail-closed (exit 3 or 4)
When `world_fs.fail_closed.routing=true` and the world is enabled:
- Exit `3` when routing fails due to required dependency unavailability.
- Exit `4` when routing fails due to missing prerequisites or unsupported capability.

### 2.3 Routing fallback warning contract (Appendix B.2.1)
If routing to the world fails at runtime and Substrate falls back to host execution while:
- `world_fs.host_visible=false` was requested, and
- `world_fs.fail_closed.routing=false` allows fallback,
Substrate MUST emit a high-signal warning to stderr.

Warning content requirements (message format is otherwise implementation-defined):
- The warning MUST be printed to stderr.
- The warning MUST contain these substrings:
  - `world routing failed; falling back to host`
  - `world_fs.host_visible=false was requested`
  - `world_fs.fail_closed.routing=false allows fallback`

## 3) Caging contract (REPL and command execution)
When the effective policy for the entered cwd scope has `world_fs.caged_required=true`:
- Substrate MUST enforce caging for interactive REPL sessions.
- Substrate MUST reject uncaged mode requests (exit `2`).
- Cage root derivation MUST follow Appendix B:
  - `cage_root = workspace_root` when entered cwd is inside a workspace.
  - `cage_root = entered_cwd` when entered cwd is not inside a workspace.

## 4) REPL exit transparency + `repl.exit_cwd`
On REPL exit, if `world_cwd != entered_cwd`, Substrate MUST print a note line:
- `substrate: note: returning to host cwd: <path>`

Rules (zero ambiguity):
- `<path>` MUST be the computed host exit target after applying `repl.exit_cwd` rules:
  - If `repl.exit_cwd=entered`: `<path> = entered_cwd`.
  - If `repl.exit_cwd=last_world`:
    - If the last observed `world_cwd` is representable as a host path and the directory exists on the host at exit:
      - `<path> = <last_world_cwd>`
    - Else:
      - `<path> = entered_cwd`
      - Substrate MUST print an additional note line explaining the fallback and the reason.
- The note line above is a stable, machine-parseable hook for shell integration wrappers:
  - A wrapper that wants to apply `repl.exit_cwd=last_world` parses the `<path>` suffix and applies `cd <path>` (or equivalent) after `substrate` exits.
  - Without such a wrapper, the host shell cwd remains unchanged; the note line remains the observable contract.

## 5) No backwards compatibility
- Legacy policy keys MUST be rejected as invalid config (exit `2`).
- Legacy snapshot schema versions MUST be rejected by world-agent (protocol error).
