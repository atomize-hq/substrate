# world-disabled-reason-attribution — contract surface

This file is the single place that holds the user-visible replay contract for ADR-0038.

Decision inputs:
- `docs/project_management/packs/draft/world-disabled-reason-attribution/decision_register.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/platform-parity-spec.md`

## CLI

### Commands in scope
- `substrate --replay <span_id>`
- `substrate --replay <span_id> --replay-verbose`

### Definitions
- Replay opt-out: host replay selected by replay-local toggles.
- Effective disable attribution: a human-visible explanation for `world.enabled=false` using ADR-0037 precedence and redaction rules.
- Reason fragment: the text inside the existing replay origin and host-warning parentheses.

### Selection invariants
- This feature does not change replay selection precedence:
  - `--world`
  - `--no-world`
  - `SUBSTRATE_REPLAY_USE_WORLD`
  - recorded origin plus `--flip-world`
- This feature does not change replay backend selection, replay timeout behavior, or replay exit codes.

### Stable replay-local reason fragments (unchanged)
- `--world flag`
- `--no-world flag`
- `SUBSTRATE_REPLAY_USE_WORLD=disabled`
- `--flip-world`
- `recorded origin (span)`
- `recorded origin (replay_context)`
- `default origin`

### Stable effective-disable reason fragments
- `world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled`
- `world isolation disabled by config <workspace>/.substrate/workspace.yaml (world.enabled: false)`
- `world isolation disabled by config $SUBSTRATE_HOME/config.yaml (world.enabled: false)`
- `world isolation disabled by effective config (source unknown)`

### Replay surfaces in scope

When replay uses host execution and the host path is explained by effective world disablement, replay uses the fragments above in these existing surfaces:
- Origin summary:
  - direction-changing case: `[replay] origin: <from> -> <to> (<reason>)`
  - recorded-host case: `[replay] origin: host (recorded; <reason>)`
- Host warning:
  - `[replay] warn: running on host (<reason>)`

Rules:
- Replay-local opt-outs keep their current fragments.
- Effective-disable attribution does not rewrite replay-local opt-out fragments.
- Replay does not emit additional replay lines outside the origin summary and the existing host warning.

## Config

Authoritative precedence remains external:
- `docs/reference/env/contract.md`
- `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`

Effective-disable attribution reuses this winning-layer order for `world.enabled=false`:
1. replay CLI opt-out `--no-world` keeps the stable replay-local fragment `--no-world flag`
2. `SUBSTRATE_OVERRIDE_WORLD=disabled` when no workspace exists
3. `<workspace>/.substrate/workspace.yaml` with `world.enabled: false`
4. `$SUBSTRATE_HOME/config.yaml` with `world.enabled: false`
5. fallback `world isolation disabled by effective config (source unknown)` when provenance cannot be trusted

Replay-local env `SUBSTRATE_REPLAY_USE_WORLD` remains separate from effective `world.enabled` attribution.

## Redaction contract
- Replay stderr and telemetry do not print absolute config paths for effective-disable attribution.
- Replay stderr and telemetry do not print raw env values beyond the fixed allowlisted tokens:
  - `SUBSTRATE_OVERRIDE_WORLD=disabled`
  - `SUBSTRATE_REPLAY_USE_WORLD=disabled`

## Exit codes
- No new exit codes.
- Replay command exit behavior remains unchanged by this feature.

## Platform guarantees
- Linux, macOS, and Windows use the same reason fragments, the same tokenized path displays, and the same telemetry field names for effective-disable attribution.
- Backend-specific transport or socket details outside the reason fragment are allowed to differ by platform.
