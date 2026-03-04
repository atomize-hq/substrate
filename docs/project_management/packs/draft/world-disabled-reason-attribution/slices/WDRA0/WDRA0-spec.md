# WDRA0-spec — Replay: attribute why world isolation is disabled (`world.enabled=false`)

## Behavior delta (single)
- Existing: when replay executes on `host`, verbose replay stderr can be generic or misleading about *why* isolation is disabled (often implying `--no-world` even when the effective disable source is config or `SUBSTRATE_OVERRIDE_WORLD=disabled`).
- New: when replay executes on `host` **because** `world.enabled=false` is effective, replay verbose stderr (origin summary + host warning) and `event_type="replay_strategy"` telemetry attribute the **highest-precedence** disable source using the ADR-0037 wording/fields, without changing replay routing/selection semantics.

## Scope
- Replay verbose stderr (gated by `--replay-verbose` / `SUBSTRATE_REPLAY_VERBOSE=1`):
  - origin summary line
  - host-mode warning line (only when the origin summary includes a parenthesized `<REASON>`)
- Trace/telemetry (`trace.jsonl`):
  - `event_type="replay_strategy"` entries: add optional `world_disable_reason` / `world_disable_source` and ensure `origin_summary` mirrors the operator-facing origin summary format when it includes world-disable attribution.

## Inputs (authoritative)
- Operator stderr contract (line templates, `<REASON>` strings, redaction rules, attribution boundary):
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md`
- Telemetry/trace contract (`replay_strategy` field inventory + additive `world_disable_*` schema + absence semantics):
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md`
- Disable-attribution tokens/enums/precedence (MUST be reused verbatim; no replay-local taxonomy):
  - `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
- Replay verbose gating and replay world-toggle precedence (MUST NOT be changed):
  - `docs/reference/env/contract.md`

## Behavior (authoritative)

### Definitions
- **Effective config**: resolved `world.enabled` value for the replay invocation using the existing effective-config resolver and precedence rules (ADR-0037).
- **Replay target origin**: the origin selected by replay-local rules (recorded origin + `--flip-world` + replay toggles), as already recorded in `replay_strategy.target_origin` and explained by `origin_reason(_code)` (telemetry-spec).
- **Host due to world-disablement** (this slice’s attribution boundary):
  - replay executes on `host`, AND
  - effective config has `world.enabled=false`, AND
  - the host execution is attributable to `world.enabled=false` being effective (not to replay-only opt-outs, recorded-origin constraints, or platform limitations), as defined below.

### Attribution boundary (no misattribution)
Replay MUST treat host-mode as **host due to world-disablement** only in the following cases:

1) **Flag-driven disablement**: `--no-world` was provided (effective `world.enabled=false` is attributable to the CLI flag), OR
2) **Config/env-driven disablement prevents a world replay**:
   - the replay target origin is `world`, AND
   - effective config has `world.enabled=false` (from workspace patch, override env, global patch, or default), so isolation is disabled for this run.

Replay MUST NOT use the “world isolation disabled by …” attribution strings (contract) when replay executes on `host` for other causes, including:
- replay-only opt-outs (`SUBSTRATE_REPLAY_USE_WORLD=disabled`),
- recorded-origin constraints (span recorded on host and replay target origin is host), or
- platform/backend limitations (isolation unavailable).

### Disable-attribution selection rule (highest-precedence winner)
When replay is **host due to world-disablement**, the disable attribution MUST reflect the *highest-precedence* effective winner for `world.enabled` per ADR-0037:
1. CLI flags: `--world` / `--no-world` (when provided)
2. Workspace config patch: `<workspace>/.substrate/workspace.yaml` (when workspace exists and is enabled)
3. Override env: `SUBSTRATE_OVERRIDE_WORLD` (applies only when no workspace is enabled)
4. Global config patch: `$SUBSTRATE_HOME/config.yaml` (default: `~/.substrate/config.yaml`)
5. Default config

### Operator-facing replay stderr (verbose only)
All operator-facing replay stderr behavior in scope is defined by `contract.md`; this slice binds it to the replay implementation seams.

Requirements this slice MUST satisfy:
- Gating: origin summary + host-mode warning MUST be emitted only when verbose replay is enabled (env contract + contract.md absence semantics).
- Templates: origin summary and host warning MUST follow the exact line templates in `contract.md`.
- Reason matching: when the host-mode warning line is emitted, its parenthesized `<REASON>` MUST exactly match the `<REASON>` in the origin summary for the same invocation (`contract.md`).
- World-disable attribution strings: when replay is host due to world-disablement and disable provenance is known, the `<REASON>` MUST be the exact verbatim string from `contract.md` (sourced from ADR-0037); when provenance is unknown, `<REASON>` MUST be the deterministic fallback from `contract.md`.
- Redaction: the redaction rules in `contract.md` MUST hold for any `<REASON>` string introduced/modified by this slice.

### Replay strategy telemetry (`event_type="replay_strategy"`)
All `replay_strategy` telemetry behavior in scope is defined by `telemetry-spec.md`; this slice binds it to the replay producers.

Requirements this slice MUST satisfy:
- Additivity: `world_disable_reason` / `world_disable_source` are additive-only, optional fields (no renames; no breaking changes).
- Emission conditions:
  - When replay is host due to world-disablement and provenance is known, `world_disable_reason` and `world_disable_source` MUST be emitted together.
  - When replay is host due to world-disablement but provenance is unknown, both fields MUST be omitted (do not guess).
  - When replay is host for other causes, both fields MUST be omitted.
- Absence semantics: omit fields when not applicable (do not emit `null`).
- Redaction:
  - `world_disable_source.path_display` MUST be tokenized (`<workspace>/.substrate/workspace.yaml` or `$SUBSTRATE_HOME/config.yaml` only).
  - `world_disable_source.env` MUST be the key name only (`SUBSTRATE_OVERRIDE_WORLD`).
  - `world_disable_source.value_display` MUST be the literal string `"false"` when present.
- `origin_summary`:
  - When the operator-facing origin summary includes a world-disable attribution `<REASON>`, the `replay_strategy.origin_summary` field MUST include the same `<REASON>` string that would appear in the host-mode warning for the same invocation (per `telemetry-spec.md` + `contract.md`).

## Acceptance criteria
- AC-WDRA0-01: With `--replay-verbose` and `--no-world`, replay emits:
  - an origin summary line using the reason-bearing template, and
  - a host-mode warning line,
  and both lines contain the exact verbatim `<REASON>`: `world isolation disabled by CLI flag --no-world`.
- AC-WDRA0-02: Outside an enabled workspace, with `SUBSTRATE_OVERRIDE_WORLD=disabled` (and no `--no-world`), when replay would otherwise target `world`, replay executes on `host` and the verbose origin summary + host warning `<REASON>` is exactly: `world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled`.
- AC-WDRA0-03: Inside an enabled workspace with `<workspace>/.substrate/workspace.yaml` setting `world.enabled: false`, when replay would otherwise target `world`, replay executes on `host` and the verbose origin summary + host warning `<REASON>` is exactly: `world isolation disabled by workspace config <workspace>/.substrate/workspace.yaml (world.enabled: false)`.
- AC-WDRA0-04: Outside an enabled workspace, with `$SUBSTRATE_HOME/config.yaml` setting `world.enabled: false`, when replay would otherwise target `world`, replay executes on `host` and the verbose origin summary + host warning `<REASON>` is exactly: `world isolation disabled by global config $SUBSTRATE_HOME/config.yaml (world.enabled: false)`.
- AC-WDRA0-05: Precedence: inside an enabled workspace with `world.enabled: false`, even if `SUBSTRATE_OVERRIDE_WORLD=disabled` is also set, the attribution winner is `workspace_patch` (not `override_env`) in both stderr `<REASON>` selection and `world_disable_reason`.
- AC-WDRA0-06: Boundary: with replay-only opt-out `SUBSTRATE_REPLAY_USE_WORLD=disabled`, replay verbose stderr MUST NOT use any “world isolation disabled by …” attribution strings, and `replay_strategy` MUST omit `world_disable_reason` / `world_disable_source`.
- AC-WDRA0-07: Redaction: in all cases where a world-disable attribution `<REASON>` is emitted and/or `world_disable_source` is emitted:
  - no absolute host paths appear (only `$SUBSTRATE_HOME/config.yaml` / `<workspace>/.substrate/workspace.yaml` tokens), and
  - no env var values are leaked beyond the fixed allowed token `SUBSTRATE_OVERRIDE_WORLD=disabled` and key-only `SUBSTRATE_OVERRIDE_WORLD`.
- AC-WDRA0-08: Telemetry emission: when replay is host due to world-disablement and provenance is known, the latest `event_type="replay_strategy"` entry contains:
  - `world_disable_reason` with the expected enum value (`cli_flag|override_env|workspace_patch|global_patch|default`), and
  - `world_disable_source` with only ADR-0037 keys and semantics,
  and when not applicable both fields are omitted (not `null`).
- AC-WDRA0-09: Reason matching: whenever the host-mode warning line is emitted, its parenthesized `<REASON>` exactly matches the `<REASON>` in the origin summary for the same invocation (contract invariant).
- AC-WDRA0-10: Fallback posture: if replay is host due to world-disablement but disable provenance cannot be determined, the verbose stderr `<REASON>` is exactly `world isolation disabled by effective config (source unknown)` and `world_disable_reason` / `world_disable_source` are omitted.

## Out of scope
- Any replay routing/selection semantic changes (host vs world selection rules), including replay toggle precedence (`--world` > `--no-world` > `SUBSTRATE_REPLAY_USE_WORLD`) and recorded-origin handling.
- New CLI flags, new environment variables, new config keys, or exit-code changes.
- Attribution for non-config causes of host-mode replay (platform limitations, replay-only opt-outs) beyond the boundary rules above.
