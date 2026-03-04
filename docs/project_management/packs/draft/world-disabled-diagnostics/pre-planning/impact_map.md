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
- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/decision_register.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/manual_testing_playbook.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/plan.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/quality_gate_report.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/session_log.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD0/WDD0-spec.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/macos-smoke.sh`
- `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/windows-smoke.ps1`
- `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`

### Edit
- `crates/shell/src/builtins/health.rs`
- `crates/shell/src/builtins/shim_doctor/output.rs`
- `crates/shell/src/builtins/shim_doctor/report.rs`
- `crates/shell/tests/shim_doctor.rs`
- `crates/shell/tests/shim_health.rs`
- `docs/USAGE.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json`
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
- Change: `substrate shim doctor` treats effective `world.enabled=false` as **World backend: disabled** and MUST NOT execute world probes for diagnostics.
  - Direct impact:
    - Operators running host-only by choice no longer see “needs attention” or probe-derived errors for the world backend.
    - “Disabled” is clearly distinguished from “broken/unreachable”.
  - Cascading impact:
    - Text rendering must branch on the new status enum rather than `world.ok` alone (avoid “healthy” implying a real backend check occurred).
    - Unit + integration coverage must prove “no probes when disabled” (at minimum: no `substrate world doctor --json` subprocess and no world-deps applied probing).
  - Contradiction risks:
    - If disabled is represented as “unknown” or “ok=true”, operators/tooling will misread the state; the status enum must be authoritative.
    - “Disabled” must not mask genuine failures when `world.enabled=true` (skip must be gated strictly on effective config, not just missing sockets).

- Change: `substrate health` treats effective `world.enabled=false` as a non-failure state and reports **World deps: skipped (world disabled)** (non-error).
  - Direct impact:
    - Health summary no longer goes “attention required” solely due to world-disabled probing failures.
    - Remediation hints point to enabling/provisioning (e.g., `substrate world enable`) rather than world-deps sync/apply when world is disabled.
  - Cascading impact:
    - Health summary failure aggregation must not treat “skipped_disabled” as an error (avoid populating `failures` with “world deps unavailable” when disabled).
    - `docs/USAGE.md` examples and field explanations must be updated so operator docs match shipped behavior.
  - Contradiction risks:
    - World-deps provisioning packs (ADR-0030/ADR-0033) evolve “next steps” guidance; WDD must ensure guidance remains method-aware when enabled, and fully suppressed/redirected when disabled.

- Change: JSON outputs gain explicit, additive status enums for world + world-deps (both commands).
  - Direct impact:
    - Tooling can detect disabled/skipped without parsing error strings.
  - Cascading impact:
    - `world-disabled-diagnostics-json-schema-spec.md` must lock field placement, enum spellings, emission rules, and absence semantics (including the health JSON surface, which ADR-0036 leaves underspecified).
    - Tests must assert both the disabled/skipped emissions and the “enabled-but-broken remains needs-attention” case.
  - Contradiction risks:
    - `json-mode` pack may wrap/reshape outputs; it must preserve these fields inside any envelope without renames/removals.
    - `make-doctor-health-output-explain-why` (ADR-0037) adds disable-attribution fields; avoid collisions and keep “disabled” vs “why disabled” orthogonal.

### Config / env vars / paths
- Change: Diagnostics consult the Substrate effective-config resolver for `world.enabled` (CLI flags > workspace patch > override env (when no enabled workspace) > global patch > defaults).
  - Direct impact:
    - `SUBSTRATE_OVERRIDE_WORLD=disabled` and persisted config disablement become first-class inputs to health/doctor behavior (instead of relying only on `--no-world`).
  - Cascading impact:
    - Tests must cover the workspace rule (“workspace enabled ⇒ ignore `SUBSTRATE_OVERRIDE_WORLD`”) so disabled detection matches the operator contract.
    - Contract/specs must pin behavior when effective config resolution fails (invalid YAML, unreadable config) so diagnostics do not silently misclassify disabled vs unknown.
  - Contradiction risks:
    - ADR-0003 (queued) may further tighten config/env semantics; WDD must align to the current authoritative env/config contracts and avoid duplicating precedence rules in feature-local docs.

### Policy / isolation / security posture
- Change: Disabled-mode diagnostics degrade by skipping probes; enabled-mode diagnostics remain fail-visible.
  - Direct impact:
    - Avoids unnecessary backend socket traffic and reduces false-negative “broken setup” signals when host-only is intentional.
  - Cascading impact:
    - “Skip probes” must be defined operationally in `slices/WDD0/WDD0-spec.md` (what counts as a probe; what is forbidden; how to assert in tests).
  - Contradiction risks:
    - If the skip path still performs partial probing (e.g., world-deps applied computation), it will reintroduce the same misleading failures this ADR is meant to remove.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
  - Overlap surfaces:
    - `substrate health` / `substrate shim doctor` text + JSON output surfaces
    - effective-config `world.enabled` resolution plumbing
  - Conflict: yes (shared files + overlapping JSON evolution)
  - Resolution (explicit):
    - Sequencing boundary: implement WDD first (disabled/skipped statuses + probe short-circuit), then layer attribution (“why disabled”) in ADR-0037 without changing the WDD semantics.
    - Contract boundary: WDD owns **status** (`disabled` / `skipped_disabled`); ADR-0037 owns **attribution** (flag/env/workspace/global). Both must be present and must not contradict.

- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
  - Overlap surfaces:
    - `crates/shell/src/builtins/health.rs` world-deps availability/messaging and “next steps” guidance
  - Conflict: yes (shared operator guidance surface)
  - Resolution (explicit):
    - WDD must fully short-circuit world-deps probing when disabled and MUST NOT suggest provisioning/sync actions in that mode; provisioning ADRs remain authoritative for enabled-mode remediation guidance.

- ADR: `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
  - Overlap surfaces:
    - `crates/shell/src/builtins/health.rs` messaging as provisioning methods expand beyond APT
  - Conflict: yes (shared file + remediation branching)
  - Resolution (explicit):
    - Ensure health output branches first on effective `world.enabled` (disabled ⇒ skipped), then on provisioning method semantics only when enabled.

- ADR: `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
  - Overlap surfaces:
    - “install with `--no-world`, enable later” workflows (world disabled by design)
  - Conflict: no
  - Resolution (explicit):
    - WDD’s “disabled” diagnostics becomes the expected baseline after `--no-world` installs, reducing confusion without changing enable/provisioning behavior.

- ADR: `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md`
  - Overlap surfaces:
    - shared operator terminology around “world disabled by choice” and effective-config resolution
  - Conflict: no
  - Resolution (explicit):
    - Keep terminology coherent: replay attribution work should reuse ADR-0037 wording; WDD provides the baseline status enums and “skip probes” posture for diagnostics only.

- ADR: `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`
  - Overlap surfaces:
    - effective-config model and operator-facing terminology around config/env precedence
  - Conflict: yes (future tightening may require retesting diagnostics against evolved resolver behavior)
  - Resolution (explicit):
    - Implement WDD against the current authoritative resolver + env contract; treat any ADR-0003-driven precedence/terminology shifts as a later alignment pass once ADR-0003 is implemented.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/`
  - Overlap surfaces:
    - `crates/shell/src/builtins/health.rs`
    - `crates/shell/src/builtins/shim_doctor/report.rs`
    - JSON evolution when world is disabled
  - Conflict: yes (shared files)
  - Resolution (explicit):
    - Sequence WDD first; attribution pack must integrate without renaming/removing WDD’s status fields and must keep disabled/skipped semantics unchanged.

- Planning Pack: `docs/project_management/packs/draft/json-mode/`
  - Overlap surfaces:
    - JSON contract stability for `substrate health --json` and `substrate shim doctor --json`
  - Conflict: yes (schema reshaping risk)
  - Resolution (explicit):
    - Non-overlap boundary: WDD adds additive fields to the existing payload objects; json-mode must preserve them inside any envelope and must not rename/remove them without an explicit compat posture.

- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `crates/shell/src/builtins/health.rs` world-deps remediation guidance
  - Conflict: yes (shared file + messaging evolution)
  - Resolution (explicit):
    - WDD defines disabled/skipped short-circuit and the absence of “world-deps next steps” when disabled; provisioning packs define enabled-mode remediation messaging and must keep the disabled short-circuit intact.

- Planning Pack: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
  - Overlap surfaces:
    - `crates/shell/src/builtins/health.rs` remediation messaging as provisioning methods grow
  - Conflict: yes
  - Resolution (explicit):
    - Ensure provisioning packs branch on WDD’s disabled/skipped status and do not print provisioning guidance in disabled mode.

- Planning Pack: `docs/project_management/packs/draft/dev-install-world-agent-staging/`
  - Overlap surfaces:
    - “disabled by design” workflows created by `--no-world` dev installs
  - Conflict: no
  - Resolution (explicit):
    - Keep copy consistent: avoid implying the backend is broken when it is disabled; WDD ensures diagnostics reflect that baseline.

## Follow-ups (explicit)

- Decision Register entries required:
  - DR-0001 — JSON field paths + enum spellings for world/world-deps status (including the health JSON surface).
  - DR-0002 — Legacy error-field behavior when disabled/skipped applies (must not encode skip purely as an error string).
  - DR-0003 — Operator-facing copy standardization for disabled/skipped across `substrate health` and `substrate shim doctor` (deterministic + testable).
- Spec updates required (if any):
  - `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md` — define checkpoint boundaries and ensure `tasks.json` `meta.checkpoint_boundaries` matches.
  - `docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json` — populate `WDD0-code`/`WDD0-test`/`WDD0-integ` with AC references and automation metadata.
  - `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md` — lock additive field placement, enums, emission/absence rules, and examples for disabled/healthy/needs-attention cases.
  - `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md` — lock deterministic copy, exit-code mapping, and cross-platform parity statements.
  - `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD0/WDD0-spec.md` — define the operational “skip probes” boundary and AC matrix (disabled vs enabled-but-broken).
  - `docs/project_management/packs/sequencing.json` — add the sequencing entry and dependency edges (at minimum: WDD before ADR-0037 attribution work that touches the same surfaces).

