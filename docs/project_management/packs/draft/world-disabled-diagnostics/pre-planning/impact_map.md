# world-disabled-diagnostics — impact map (pre-planning)

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-disabled-diagnostics/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`
- Spec manifest:
  - `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/world-disabled-diagnostics/"` (strict packs only).

### Create
- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/plan.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/session_log.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/decision_register.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/diagnostics-json-schema-spec.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD0/WDD0-spec.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD0/kickoff_prompts/WDD0-code.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD0/kickoff_prompts/WDD0-test.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD0/kickoff_prompts/WDD0-integ.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD1/WDD1-spec.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD1/kickoff_prompts/WDD1-code.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD1/kickoff_prompts/WDD1-test.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD1/kickoff_prompts/WDD1-integ.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD2/WDD2-spec.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD2/kickoff_prompts/WDD2-code.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD2/kickoff_prompts/WDD2-test.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD2/kickoff_prompts/WDD2-integ.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/manual_testing_playbook.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/macos-smoke.sh`
- `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/windows-smoke.ps1`

### Edit
- `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json`
- `docs/project_management/packs/sequencing.json`
- `crates/shell/src/execution/config_model.rs`
- `crates/shell/src/builtins/shim_doctor/report.rs`
- `crates/shell/src/builtins/shim_doctor/output.rs`
- `crates/shell/src/builtins/world_deps/mod.rs`
- `crates/shell/src/builtins/health.rs`
- `crates/shell/tests/common.rs`
- `crates/shell/tests/shim_doctor.rs`
- `crates/shell/tests/shim_health.rs`
- `docs/USAGE.md`
- `docs/CONFIGURATION.md`
- `docs/COMMANDS.md`
- `docs/INSTALLATION.md`
- `docs/cross-platform/wsl_world_troubleshooting.md`

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
- Change: When effective `world.enabled=false`, `substrate health` reports **World backend: disabled** and **World deps: skipped (world disabled)**, and MUST NOT report “attention required” solely due to world-disabled diagnostics.
  - Direct impact:
    - Operators running host-only by choice (e.g., persisted `world.enabled: false` or `SUBSTRATE_OVERRIDE_WORLD=disabled`) stop seeing false-negative “world backend: needs attention” / “world deps unavailable” signals.
  - Cascading impact:
    - `substrate health` text output must add a deterministic disabled state and minimal remediation guidance (e.g., “run `substrate world enable`”), aligned with `substrate shim doctor`.
    - `substrate health --json` must become strictly additive: new status fields + enums are added while existing fields remain present (schema spec + tests required).
  - Contradiction risks:
    - Other commands (not in scope here) may still emit “world disabled … (--no-world)” misattribution or hard errors when disabled (e.g., `substrate world deps current list applied`); avoid implying those commands will become quiet in this pack.

- Change: When effective `world.enabled=false`, `substrate shim doctor` MUST NOT execute world-backend probes for diagnostics purposes (including world-deps “applied” computation) and must report explicit disabled/skipped states in both text and JSON.
  - Direct impact:
    - `substrate shim doctor` becomes a reliable “host-only by choice” diagnostic surface instead of appearing broken when the world is intentionally disabled.
  - Cascading impact:
    - Shim doctor’s text output must render a disabled world backend state (not “needs attention”) and must report world-deps as skipped-disabled (non-error), with copy aligned to `substrate health`.
    - Shim doctor’s JSON model must carry stable, machine-detectable status enums for world + world-deps (field names/spellings decided by DR-0001).
  - Contradiction risks:
    - Shim doctor currently consumes `~/.substrate/health/{world_doctor.json,world_deps.json}` fixtures before falling back to live probes; contract/specs must make explicit whether fixtures are ignored or incorporated when world is disabled to avoid “stale health” implication.

- Change: Additive JSON status fields + stable enums for world + world-deps states in `substrate shim doctor --json` and `substrate health --json`.
  - Direct impact:
    - Automation can distinguish `disabled` vs `needs_attention` vs `unknown` without brittle string matching on error text.
  - Cascading impact:
    - `diagnostics-json-schema-spec.md` must inventory the full existing JSON shapes to keep the change strictly additive.
    - Tests and docs that reference `summary.world_ok` / `summary.world_deps_error` must be updated (or explicitly documented) for the disabled/skipped states.
  - Contradiction risks:
    - `substrate world doctor --json` already contains `world.status` with a different vocabulary (`disabled|not_provisioned|unreachable|ok|…`); if this pack introduces `world.status` at the shim/health layer, naming collisions and semantic drift are likely unless DR-0001 selects unambiguous field paths/enum spellings.

### Config / env vars / paths
- Change: Diagnostics resolve effective `world.enabled` using the same precedence contract as normal invocation planning (CLI flags → workspace patch → `SUBSTRATE_OVERRIDE_WORLD` when no enabled workspace → global config).
  - Direct impact:
    - World-disabled diagnostics apply consistently whether the disablement comes from CLI flags, persisted config, or override env vars (without adding new knobs).
  - Cascading impact:
    - Shim doctor must thread CLI `--world/--no-world` into any diagnostic sub-invocations (or avoid those sub-invocations) so the diagnostic view does not diverge from the effective-config resolver.
    - Integration tests must cover the persisted-config case (`$SUBSTRATE_HOME/config.yaml` with `world.enabled: false`) per ADR-0036.
  - Contradiction risks:
    - Repo docs and queued ADRs contain inconsistent notions of “world selection env vars” (e.g., `SUBSTRATE_WORLD` vs `SUBSTRATE_OVERRIDE_WORLD`); this pack must not fork precedence semantics inside diagnostics.

### Policy / isolation / security posture
- Change: When `world.enabled=false`, diagnostics degrade by skipping probes, but MUST NOT imply world health or world-deps “applied” state; when `world.enabled=true`, diagnostics MUST continue to surface backend failures (no masking).
  - Direct impact:
    - Reduced operator confusion without weakening failure visibility when world isolation is expected to be available.
  - Cascading impact:
    - JSON outputs must encode “skipped because disabled” as an explicit status value (not a generic error string), and legacy error fields must have deterministic behavior when disabled/skipped (DR-0002).
  - Contradiction risks:
    - Other in-flight packs that modify `substrate health` “next steps” guidance (world-deps provisioning work) can accidentally suggest world-backed actions even when the world is disabled unless they branch on the new disabled/skipped status fields.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - persisted `world.enabled: false` setups (`--no-world` installs) becoming more common
    - operator expectation that “world disabled” is not “broken world”
  - Conflict: no
  - Resolution (explicit):
    - Treat this pack as complementary: DIWAS improves “enable later” readiness; WDD quiets diagnostics while disabled.

- ADR: `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
  - Overlap surfaces:
    - doctor/health messaging when `world.enabled=false`
    - config provenance plumbing for `world.enabled`
  - Conflict: no (this pack explicitly forbids “why disabled” attribution)
  - Resolution (explicit):
    - Sequence boundary: land WDD disabled/skipped status fields first; later attribution work must layer on top without changing the disabled/skip semantics or reintroducing “(--no-world)” misattribution.

- ADR: `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md`
  - Overlap surfaces:
    - shared “world disabled” semantics as an input to replay warnings (via ADR-0037)
  - Conflict: no
  - Resolution (explicit):
    - Non-overlap boundary: replay attribution work is blocked on ADR-0037 and should reuse (not redefine) WDD’s disabled/skipped semantic states.

- ADR: `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`
  - Overlap surfaces:
    - world selection semantics and environment-variable terminology used by operators/tooling
  - Conflict: yes (queued ADR references `SUBSTRATE_WORLD=enabled|disabled` as an input; current implementation/contract uses `SUBSTRATE_OVERRIDE_WORLD` and treats `SUBSTRATE_WORLD` as exported state)
  - Resolution (explicit):
    - Option A: Implement WDD using the current effective-config resolver (`crates/shell/src/execution/config_model.rs`) and operator contract (`docs/reference/env/contract.md`) with `SUBSTRATE_OVERRIDE_WORLD` (selected; matches code + ADR-0036).
    - Option B: Preemptively align WDD to ADR-0003’s `SUBSTRATE_WORLD` input model (not selected; expands scope and contradicts current contract docs).

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/dev-install-world-agent-staging/`
  - Overlap surfaces:
    - operator workflows that intentionally keep `world.enabled: false` until later enablement
  - Conflict: no
  - Resolution (explicit):
    - No sequencing lock required; ensure docs and remediation text remain consistent (“disabled by choice” vs “backend broken”).

- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `crates/shell/src/builtins/health.rs` operator “next steps” guidance for world-deps
    - shared doc update targets (`docs/USAGE.md`, `docs/CONFIGURATION.md`, `docs/cross-platform/wsl_world_troubleshooting.md`)
  - Conflict: yes
  - Resolution (explicit):
    - Sequence boundary: land WDD first so health/doctor can reliably short-circuit when the world is disabled; provisioning packs must branch remediation guidance on the disabled/skipped status fields and MUST NOT suggest provisioning/sync actions when `world.enabled=false`.

- Planning Pack: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
  - Overlap surfaces:
    - `crates/shell/src/builtins/health.rs` operator messaging and method-aware next steps
    - shared operator-doc updates for world-deps workflows
  - Conflict: yes
  - Resolution (explicit):
    - Same as APT provisioning pack: WDD defines disabled short-circuit posture; provisioning packs remain authoritative only when the world is enabled.

- Planning Pack: `docs/project_management/packs/draft/json-mode/`
  - Overlap surfaces:
    - JSON output contract for `substrate health --json` and `substrate shim doctor --json` (field naming, stability, documentation)
  - Conflict: yes (json-mode introduces a cross-command envelope/schema layer)
  - Resolution (explicit):
    - Non-overlap boundary: WDD adds status fields to the existing payload model; json-mode must preserve them inside any envelope and must not rename/remove them without an explicit compat posture. DR-0001 should avoid ambiguous field names that would collide with json-mode envelope keys.

- Planning Pack (archived reference): `docs/project_management/_archived/substrate-isolated-shell/`
  - Overlap surfaces:
    - historical shim-doctor/health fixture harness and world-disabled messaging expectations
  - Conflict: no (archived)
  - Resolution (explicit):
    - Treat as historical context only; do not resurrect archived “world disabled” semantics that contradict ADR-0036 or this pack’s contract/specs.

## Follow-ups (explicit)

- Decision Register entries required:
  - DR-0001 — Decide JSON field paths + enum spellings for world/world-deps status fields (avoid collisions with existing `world_doctor` status vocabulary).
  - DR-0002 — Decide legacy `error`/`ok` field behavior when disabled/skipped (strictly additive; no ambiguous “ok=false” that reads as failure when disabled).
  - DR-0003 — Decide deterministic copy constraints (stable phrases/substrings) for disabled/skipped messaging across `substrate health` and `substrate shim doctor`.
- Spec updates required (if any):
  - `docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json` — populate schema v4 triad tasks + kickoff prompt paths; add `meta.checkpoint_boundaries`.
  - `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md` — define checkpoint groups covering `WDD0..WDD2`; ensure alignment with `tasks.json`.
  - `docs/project_management/packs/sequencing.json` — add the WDD sequencing entry referenced by ADR-0036 and `plan.md`.
  - `docs/project_management/packs/draft/world-disabled-diagnostics/diagnostics-json-schema-spec.md` — inventory and lock the full existing JSON shapes for both commands before specifying additive status fields.
  - `docs/USAGE.md` — document the disabled/skipped states and the preferred machine-detectable status fields (keep `summary.world_ok` semantics coherent when disabled).

