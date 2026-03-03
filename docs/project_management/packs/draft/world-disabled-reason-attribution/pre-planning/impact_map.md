# world-disabled-reason-attribution — impact map (pre-planning)

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-disabled-reason-attribution/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md`
- Spec manifest:
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/world-disabled-reason-attribution/"` (strict packs only).

### Create
- `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/decision_register.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/plan.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA0/WDRA0-spec.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md`

### Edit
- `crates/replay/src/replay/executor.rs`
- `crates/replay/src/replay/mod.rs`
- `crates/shell/src/execution/routing/replay.rs`
- `crates/shell/tests/replay_world.rs`
- `docs/REPLAY.md`
- `docs/TRACE.md`
- `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json`
- `docs/project_management/packs/sequencing.json`

### Deprecate
- None

### Delete
- None

## Cascading implications (behavior/UX)

For each externally visible change, list:
- direct impact (what the operator experiences),
- cascading impact (what else must change to stay coherent),
- contradiction risks (what existing semantics would conflict).

### CLI / UX
- Change: Replay verbose stderr attribution for host-mode due to world-disablement is aligned with ADR-0037 (doctor/health wording + precedence).
  - Direct impact:
    - Replay output no longer misattributes host-only replay to `--no-world` when the effective disable source is persisted config or `SUBSTRATE_OVERRIDE_WORLD=disabled`.
    - Operators get an actionable “what to flip” indicator directly in replay output.
  - Cascading impact:
    - `contract.md` must define the exact replay stderr surfaces in scope (origin summary + host-mode warning) and where the ADR-0037 attribution string appears.
    - `slices/WDRA0/WDRA0-spec.md` must explicitly define “host due to world-disablement” vs “host due to recorded origin” vs “host due to replay opt-out” so replay does not mislead.
    - Integration tests must cover each disable source (CLI, override env, workspace config, global config), including the “workspace disables env override” rule.
  - Contradiction risks:
    - Replay’s existing host-mode reasons (e.g., `SUBSTRATE_REPLAY_USE_WORLD=disabled`, `--flip-world`) are not the same thing as “world is disabled by effective config”; conflating them would produce misleading attribution.
    - Windows/WSL replay may run host because isolation is unavailable; that must not be reported as “disabled by config/env/flag”.

- Change: `replay_strategy.origin_summary` continues to mirror the replay origin stderr line format, but now includes correct disable-attribution when applicable.
  - Direct impact:
    - Trace consumers can join replay stderr diagnostics with `replay_strategy` entries and see consistent messaging.
  - Cascading impact:
    - `telemetry-spec.md` must define the stability/meaning of `origin_summary` alongside any new disable-attribution fields.
    - `docs/TRACE.md` example payload and field notes must be updated to match the shipped schema/wording.
  - Contradiction risks:
    - Any scripts that parse replay stderr or `replay_strategy` text fields may need adjustment; contract should steer automation to structured fields (below) where possible.

### Config / env vars / paths
- Change: Replay computes disable attribution via the ADR-0037 contract (tokens/enums/strings + precedence), including the “env override ignored when workspace enabled” rule.
  - Direct impact:
    - Disable attribution is correct across the precedence stack and does not point operators at the wrong knob.
  - Cascading impact:
    - ADR-0038’s precedence list must be reconciled with ADR-0037 and `docs/reference/env/contract.md` (ADR-0037 is authoritative).
    - Tokenized display paths must be used consistently in replay output and telemetry (`<workspace>/.substrate/workspace.yaml`, `$SUBSTRATE_HOME/config.yaml`), never absolute host paths.
  - Contradiction risks:
    - ADR-0003 (queued) proposes `SUBSTRATE_WORLD=enabled|disabled` as an input; current implementation and operator contract treat `SUBSTRATE_WORLD` as exported state and `SUBSTRATE_OVERRIDE_WORLD` as the input. This feature must implement against the current contract to avoid drift.

- Change: Redaction constraints apply to both stderr and telemetry.
  - Direct impact:
    - Replay messaging and trace events remain safe to share for support/debugging without leaking absolute paths or arbitrary env values.
  - Cascading impact:
    - Tests must assert redaction invariants (no absolute paths; no env values beyond fixed tokens).
    - `telemetry-spec.md` must define per-field redaction rules (including tokenized paths and env key-only constraints).
  - Contradiction risks:
    - Mixing real filesystem paths (already present elsewhere in `replay_strategy`, e.g., `agent_socket`) with tokenized config provenance paths must be deliberate and documented to avoid confusing consumers.

### Policy / isolation / security posture
- Change: Additive `replay_strategy` disable-attribution fields are emitted only when replay runs host due to world-disablement.
  - Direct impact:
    - Trace consumers can reliably determine *why* world was disabled (flag vs env vs workspace/global config) without scraping stderr strings.
  - Cascading impact:
    - `telemetry-spec.md` must pin field placement and explicit absence semantics:
      - fields MUST be omitted when not applicable (e.g., replay is host because the recorded origin is host, or because replay opted out via `SUBSTRATE_REPLAY_USE_WORLD`).
    - Integration tests must assert conditional emission and additivity (existing fields remain stable).
  - Contradiction risks:
    - DR-0002 must prevent schema drift: overloading `origin_reason(_code)` as the sole carrier of world-disable attribution would be ambiguous and risks breaking existing semantics/consumers.

- Change: Replay routing/selection semantics are unchanged; only attribution/messaging and additive telemetry are introduced.
  - Direct impact:
    - No behavior change to host/world selection, timeouts, or backend strategy; operators see only corrected attribution on existing surfaces.
  - Cascading impact:
    - `WDRA0-spec.md` must include explicit acceptance criteria that assert “no routing/selection change” to guard against accidental behavioral drift when adding attribution computation.
  - Contradiction risks:
    - If attribution computation requires additional effective-config resolution in replay routing, failures must degrade to a generic non-misattributing message (per ADR-0037/0038 invariants) and must not cause replay to fail.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
  - Overlap surfaces:
    - attribution enums/strings/precedence contract (`world_disable_reason`, `world_disable_source`, tokenized paths)
    - shared code helper expected to be reused by replay
  - Conflict: no (hard dependency)
  - Resolution (explicit):
    - ADR-0037 must land first; this feature reuses ADR-0037’s helper and MUST NOT redefine attribution semantics.

- ADR: `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`
  - Overlap surfaces:
    - “world enabled/disabled” operator semantics and effective-config resolution (`world.enabled=false`)
  - Conflict: no (different UX surfaces), but terminology must remain coherent
  - Resolution (explicit):
    - WDD owns “disabled/skipped” diagnostic statuses; ADR-0037 owns “why disabled” attribution; this feature reuses ADR-0037 attribution wording inside replay without expanding WDD scope.

- ADR: `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`
  - Overlap surfaces:
    - world-selection terminology and environment-variable semantics presented to operators/tooling
  - Conflict: yes (env input model differs from current contract)
  - Resolution (explicit):
    - Implement attribution against the current effective-config resolver + env contract (`SUBSTRATE_OVERRIDE_WORLD`, workspace/global patches). Treat ADR-0003 alignment as a later migration once ADR-0003 is implemented.

- ADR: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - Overlap surfaces:
    - `replay_strategy` trace schema and joinability expectations; `docs/TRACE.md` guidance
  - Conflict: no (additive-only fields), but shared schema surface
  - Resolution (explicit):
    - Keep `replay_strategy` changes strictly additive (new optional fields only), and update trace docs/examples to include the new fields without renaming existing keys.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/`
  - Overlap surfaces:
    - shared attribution helper (`crates/shell/src/execution/world_disable_attribution.rs`) and its unit tests
  - Conflict: no (dependency)
  - Resolution (explicit):
    - Require ADR-0037 integration before executing this pack; reuse helper API and extend tests only additively.

- Planning Pack: `docs/project_management/packs/draft/world-disabled-diagnostics/`
  - Overlap surfaces:
    - effective-config `world.enabled=false` semantics and operator terminology for “disabled by choice”
  - Conflict: yes (UX consistency risk; potential shared doc touch in world-disabled guidance)
  - Resolution (explicit):
    - Non-overlap boundary: WDD defines disabled/skipped statuses; this feature defines attribution in replay surfaces only. Ensure the replay attribution strings remain those from ADR-0037 and do not introduce new “disabled” terminology that contradicts WDD.

- Planning Pack: `docs/project_management/packs/active/world_process_exec_tracing_parity/`
  - Overlap surfaces:
    - trace schema + documentation in `docs/TRACE.md` (including `replay_strategy` examples)
  - Conflict: yes (shared doc surface)
  - Resolution (explicit):
    - Sequence doc edits to avoid churn: update `docs/TRACE.md` once the telemetry field placement is finalized in `telemetry-spec.md`; keep changes additive and compatible with existing example JSON.

## Follow-ups (explicit)

- Decision Register entries required:
  - DR-0001 — Shared helper reuse vs replay-local duplication (ADR-0038 selection is A; record rationale + test strategy).
  - DR-0002 — Replay trace contract: emit explicit `world_disable_reason` / `world_disable_source` on `replay_strategy` vs overloading `origin_reason(_code)` (must remain additive with explicit absence semantics).
- Spec updates required (if any):
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json` — populate WDRA0 triad tasks (code/test/integ) with acceptance criteria IDs from `slices/WDRA0/WDRA0-spec.md`.
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md` — lock replay stderr line templates, gating/absence semantics, and redaction rules; link ADR-0037 strings without redefining them.
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md` — lock `replay_strategy` field placement and redaction rules; define additivity + absence semantics.
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA0/WDRA0-spec.md` — define AC matrix (CLI/env/workspace/global winners + redaction + “no behavior change” assertions) and the boundary between world-disablement vs replay-only opt-out.
  - `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md` — reconcile precedence text with ADR-0037 + env contract (workspace overrides env); update Related Docs paths to point at the canonical pre-planning artifacts.
  - `docs/project_management/packs/sequencing.json` — add the sequencing entry and gate this work on ADR-0037 integration landing first.
