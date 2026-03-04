# Telemetry spec — Replay: world-disabled reason attribution (`replay_strategy`)

This document is authoritative for the **trace/telemetry schema contract** changes introduced by
ADR-0038 on the `event_type="replay_strategy"` records appended to `trace.jsonl`.

This document MUST NOT redefine:
- Disable attribution field names, enum values, and source object schema:
  - `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
- Operator-facing replay stderr wording (the `<REASON>` strings and line templates):
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md`
- Replay verbose gating / replay-only world toggles:
  - `docs/reference/env/contract.md`

## Invariants

- Additivity: changes are **additive-only** (new optional fields only); no renames; no breaking
  schema changes.
- Semantics: **no replay routing/selection behavior changes** are defined by this document.
- Redaction: new fields introduced by this feature MUST NOT leak:
  - absolute host paths (use tokenized display paths only), or
  - env var values beyond key names.
- Absence semantics: when a field is “not applicable”, it MUST be **omitted** (not `null`).

## Event in scope: `event_type="replay_strategy"`

`replay_strategy` records are appended for replay runs to capture the selected backend and any
fallback reasons. Producers live in:
- `crates/replay/src/replay/executor.rs` (world/agent/copy-diff backends)
- `crates/replay/src/replay/planner.rs` (direct fallback when isolation is unavailable)
- `crates/shell/src/execution/routing/replay.rs` (host-mode routing)

Joinability:
- `cmd_id` and `span_id` refer to the replayed command’s span id.

## Existing fields: inventory + meaning (stable)

The fields below exist today and are not redefined by this feature unless explicitly stated.

### Origin selection fields (existing; unchanged)

- `recorded_origin` (string): where the command originally executed (`host` or `world`).
- `target_origin` (string): the replay-selected origin (`host` or `world`) after replay-local
  overrides (CLI flags / `SUBSTRATE_REPLAY_USE_WORLD` / `--flip-world`).
- `origin_reason` (string, optional): human-readable reason for `target_origin` selection.
  - Examples today include: `--world flag`, `--no-world flag`, `SUBSTRATE_REPLAY_USE_WORLD=disabled`,
    `recorded origin (span)`, `--flip-world`.
  - This field MUST NOT be treated as the canonical carrier of `world.enabled` disable attribution.
- `origin_reason_code` (string, optional): stable machine code for `origin_reason`.
  - Values currently include: `flag_world`, `flag_no_world`, `env_disabled`, `recorded_origin`,
    `flip_world`.
  - This field MUST remain about replay-local origin selection (not `world.enabled` provenance).

### `origin_summary` (existing; meaning pinned here)

`origin_summary` (string, optional) is a human-readable string intended to mirror the replay verbose
origin summary line format.

When present, it MUST:
- match the replay origin summary line templates in `contract.md`, and
- when it includes a world-disable attribution `<REASON>`, use the same `<REASON>` string that would
  appear in the host-mode warning for the same invocation (per `contract.md`).

This feature relies on `origin_summary` as the trace-side “mirror” of the operator-facing origin
summary when the replay executes on host due to `world.enabled=false`.

## Additive schema: world-disable attribution fields (new; stable)

When replay executes on host due to `world.enabled=false` being effective (and the disable
provenance is known), `replay_strategy` records MUST include the following **additive** fields,
reusing ADR-0037 names/values verbatim:

### `world_disable_reason` (string enum; stable; reused verbatim)

Allowed values (from ADR-0037):
- `cli_flag`
- `override_env`
- `workspace_patch`
- `global_patch`
- `default`

### `world_disable_source` (object; stable keys; reused verbatim)

Schema (from ADR-0037):
- `key`: always `world.enabled`
- `layer`: matches `world_disable_reason`
- `flag` (optional): `--no-world` when `layer=cli_flag`
- `env` (optional): `SUBSTRATE_OVERRIDE_WORLD` when `layer=override_env`
- `path_display` (optional): one of:
  - `<workspace>/.substrate/workspace.yaml` when `layer=workspace_patch`
  - `$SUBSTRATE_HOME/config.yaml` when `layer=global_patch`
- `value_display`: always `false` (string) when present

## Emission conditions + absence semantics (authoritative)

### When to emit `world_disable_reason` / `world_disable_source`

`world_disable_reason` and `world_disable_source` MUST be emitted together on `replay_strategy`
records if and only if **all** of the following are true for the replay invocation:

1) Replay executes on `host`, and
2) The reason replay executes on `host` is that world isolation is disabled by the effective config
   (`world.enabled=false`) (per the “Attribution boundary” in `contract.md`), and
3) The disable provenance (ADR-0037 layer + source metadata) can be determined.

If (1) and (2) are true but (3) is false, both fields MUST be omitted (do not guess). In that case
the operator-facing `<REASON>` must fall back to the deterministic “source unknown” wording in
`contract.md`.

### When to omit (non-exhaustive)

The fields MUST be omitted when replay runs on host for reasons that are not “world disabled by
effective config”, including:
- Replay-only opt-outs (e.g. `SUBSTRATE_REPLAY_USE_WORLD=disabled`)
- Recorded-origin constraints (e.g. replay runs host because the span was recorded on host)
- Platform/backend limitations (e.g. isolation unavailable)

## Redaction rules (testable; additive fields only)

These rules apply to the new `world_disable_*` fields (and to any `origin_summary`/warning strings
that incorporate world-disable attribution).

- `world_disable_source.path_display` MUST be tokenized (no absolute host paths).
- `world_disable_source.env` MUST be the key name only (`SUBSTRATE_OVERRIDE_WORLD`), never a value.
- `world_disable_source.value_display` MUST be the literal string `false`.
- No additional keys may be added to `world_disable_source` by replay (schema is owned by ADR-0037).

## Example: replay host due to workspace config disablement

```json
{
  "event_type": "replay_strategy",
  "component": "replay",
  "strategy": "host",
  "recorded_origin": "world",
  "target_origin": "host",
  "origin_summary": "[replay] origin: world -> host (world isolation disabled by workspace config <workspace>/.substrate/workspace.yaml (world.enabled: false))",
  "world_disable_reason": "workspace_patch",
  "world_disable_source": {
    "key": "world.enabled",
    "layer": "workspace_patch",
    "path_display": "<workspace>/.substrate/workspace.yaml",
    "value_display": "false"
  }
}
```

