# Contract — Replay: attribute why world isolation is disabled (verbose replay stderr)

This contract is authoritative for the **operator-facing replay stderr messaging** introduced by
`ADR-0038` when replay runs on host because world isolation is disabled by effective config
(`world.enabled=false`).

This contract MUST NOT redefine:
- Disable-attribution strings/tokens/precedence: `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
- Replay verbose gating + replay world toggles: `docs/reference/env/contract.md`
- Exit codes: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Invariants

- This feature is **attribution + messaging only**:
  - MUST NOT change replay routing/selection semantics (host vs world selection).
  - MUST NOT add new CLI flags or new environment variables.
- This feature introduces **no new exit codes** and makes **no exit-code behavior changes**.

## Surfaces in scope (replay verbose stderr)

This contract applies only to verbose replay stderr (enabled via `--replay-verbose` or
`SUBSTRATE_REPLAY_VERBOSE=1`; precedence and parsing are owned by `docs/reference/env/contract.md`).

The only replay stderr lines in scope for this feature are:
1) The replay **origin summary** line.
2) The replay **host-mode warning** line (when replay runs on host).

When verbose replay is **not** enabled, these lines MUST NOT be emitted (absence semantics).

## Line templates (authoritative)

### Origin summary

Replay verbose MUST emit exactly one origin summary line using one of these templates:

- Unchanged origin (no `<REASON>` is displayed):
  - `[replay] origin: <ORIGIN> (<ORIGIN_SOURCE>)`
- Origin changed or a `<REASON>` is displayed:
  - `[replay] origin: <RECORDED_ORIGIN> -> <EXECUTION_ORIGIN> (<REASON>)`

Where:
- `<ORIGIN>`, `<RECORDED_ORIGIN>`, `<EXECUTION_ORIGIN>` are `host` or `world`.
- `<ORIGIN_SOURCE>` is the existing source tag (e.g., `recorded`, `default`) and is unchanged by
  this feature.
- `<REASON>` MUST be a single parenthesized string and MUST be the same string that appears in the
  host-mode warning when that warning is emitted (see below).

### Host-mode warning

When replay executes on `host` and verbose replay is enabled, replay MUST emit a host-mode warning
line whenever the origin summary uses the “origin changed or a `<REASON>` is displayed” template (i.e., when a
parenthesized `<REASON>` is present).

`[replay] warn: running on host (<REASON>)`

Additional requirements:
- The `<REASON>` string MUST exactly match the `<REASON>` used by the origin summary line for the
  same replay invocation.
- Replay MUST NOT claim “world isolation disabled by …” unless the run is on host specifically due
  to `world.enabled=false` being effective (see “Attribution boundary”).

## World-disable attribution integration (ADR-0037)

### Attribution boundary (no misattribution)

The “world isolation disabled by …” attribution strings in this section are reserved for the case
where replay executes on host because world isolation is disabled by the **effective config**
(`world.enabled=false`).

Replay MUST NOT use these strings when host-mode replay is due to other causes, including:
- Replay-only opt-outs (for example `SUBSTRATE_REPLAY_USE_WORLD=disabled`), or
- Recorded-origin constraints (replay runs host because the span was recorded on host), or
- Platform limitations (for example, Windows/WSL falling back to direct execution because isolation
  is unavailable).

### Attribution selection rule (highest-precedence winner)

When replay executes on host due to `world.enabled=false`, the attribution MUST reflect the
**highest-precedence disable source** for `world.enabled` per ADR-0037:
1. CLI flags: `--world` / `--no-world` (when provided)
2. Workspace config patch: `<workspace>/.substrate/workspace.yaml` (when workspace exists and is enabled)
3. Override env: `SUBSTRATE_OVERRIDE_WORLD` (applies only when no workspace is enabled)
4. Global config patch: `$SUBSTRATE_HOME/config.yaml` (default: `~/.substrate/config.yaml`)
5. Default config

### Attribution strings (verbatim; no replay-local wording)

When the disable source is known, the `<REASON>` string used in both the origin summary and the
host-mode warning MUST be **exactly one** of the following verbatim strings from ADR-0037:

- `world isolation disabled by CLI flag --no-world`
- `world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled`
- `world isolation disabled by workspace config <workspace>/.substrate/workspace.yaml (world.enabled: false)`
- `world isolation disabled by global config $SUBSTRATE_HOME/config.yaml (world.enabled: false)`

### Fallback string (deterministic; non-misattributing)

If replay executes on host due to `world.enabled=false` but the disable provenance cannot be
determined, `<REASON>` MUST be exactly:

`world isolation disabled by effective config (source unknown)`

## Redaction rules (testable)

These rules apply to the strings introduced/modified by this feature (the `<REASON>` text above).

- Absolute host paths MUST NOT appear.
  - Only the stable display tokens `$SUBSTRATE_HOME/config.yaml` and
    `<workspace>/.substrate/workspace.yaml` are allowed in world-disable attribution strings.
- Env values MUST NOT be printed beyond key names.
  - The only allowed fixed env token in world-disable attribution strings is
    `SUBSTRATE_OVERRIDE_WORLD=disabled`.

## Exit codes

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This feature introduces no new exit codes and makes no exit-code behavior changes.

## Platform guarantees

- Linux/macOS/Windows: when the “host due to `world.enabled=false` effective config” case occurs,
  replay MUST use the same attribution selection rule, the same verbatim strings, and the same
  redaction guarantees.
