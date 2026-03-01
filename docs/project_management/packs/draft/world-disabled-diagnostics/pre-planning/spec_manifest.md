# world-disabled-diagnostics — spec manifest (pre-planning)

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-disabled-diagnostics/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`

## Slice IDs (canonical)

ADR-0036 uses placeholder slice IDs (`C0`, `C1`, `C2`). This feature MUST use feature-derived slice IDs per:
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`

Canonical slice IDs selected for this feature:
- Slice prefix: `WDD` (derived from “World Disabled Diagnostics”)
- `WDD0` — effective `world.enabled` resolution inside diagnostics (ADR “C0”)
- `WDD1` — disabled-aware world + world-deps diagnostics classification (ADR “C1”)
- `WDD2` — contract + validation coverage (ADR “C2”)

## Required spec documents (authoritative)

Each entry lists:
- what surfaces it owns (authoritative), and
- what it links to (non-authoritative).

Spec templates:
- `docs/project_management/system/templates/spec/`

### Planning pack scaffolding (required)

- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md` — spec selection + ownership map (this file)
  - Owns (authoritative):
    - the required-doc set for this feature directory
    - the coverage matrix (surface → authoritative doc)
    - follow-ups required to remove ambiguity before quality gate
    - explicit “no implied surfaces” statements for every surface category that is NOT used by this feature (policy/protocol/telemetry/filesystem/etc.)
  - Must define:
    - an explicit “MUST NOT change” statement for each out-of-scope surface category listed in the coverage matrix
  - Links (non-authoritative):
    - `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`

- `docs/project_management/packs/draft/world-disabled-diagnostics/plan.md` — execution runbook + sequencing overview
  - Owns (authoritative):
    - slice ordering (`WDD0` then `WDD1` then `WDD2`)
    - task execution guardrails (triad workflow, worktree boundaries)
    - required validation evidence at slice completion (tests + manual + smoke)
  - Must define:
    - the orchestration branch (MUST match `tasks.json` `meta.automation.orchestration_branch`)
    - the canonical spec ownership map location for this pack: `pre-planning/spec_manifest.md`
    - the canonical impact map location for this pack: `pre-planning/impact_map.md`
    - the full slice list (`WDD0`, `WDD1`, `WDD2`) and each slice’s objective (single sentence each)
    - the validation evidence required at the end of each slice:
      - unit/integration test targets per ADR-0036
      - manual playbook completion
      - smoke scripts completion (Linux/macOS/Windows)
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/diagnostics-json-schema-spec.md`
    - slice specs under `docs/project_management/packs/draft/world-disabled-diagnostics/slices/`

- `docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json` — triad task graph + acceptance criteria
  - Owns (authoritative):
    - task IDs, dependency graph, and automation metadata (branch/worktree/prompt paths)
  - Must define:
    - `meta.checkpoint_boundaries` (required for schema v4 cross-platform automation packs): an array of slice IDs that are the last slice in each checkpoint group in `pre-planning/ci_checkpoint_plan.md`
    - triad tasks for each slice:
      - `WDD0-code`, `WDD0-test`, `WDD0-integ`
      - `WDD1-code`, `WDD1-test`, `WDD1-integ`
      - `WDD2-code`, `WDD2-test`, `WDD2-integ`
    - each task’s acceptance criteria MUST reference slice-spec AC IDs (no freeform acceptance)
    - kickoff prompt paths for every task (MUST exist on disk)
  - Links (non-authoritative):
    - slice specs under `docs/project_management/packs/draft/world-disabled-diagnostics/slices/`

- `docs/project_management/packs/draft/world-disabled-diagnostics/session_log.md` — planning + execution audit log
  - Owns (authoritative):
    - the append-only record of planning/execution events for this pack
  - Must define:
    - the file MUST be initialized from `docs/project_management/system/templates/planning_pack/session_log.md.tmpl`
    - every task start/end MUST be recorded with timestamp + task id

- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md` — touch set + cascading implications + cross-queue conflicts
  - Owns (authoritative):
    - the explicit touch set (create/edit/deprecate/delete) for this feature
    - the explicit list of cascading implications and contradiction risks
  - Must define:
    - the touch set MUST enumerate every repo file expected to be created/edited/deprecated/deleted by this feature
    - expected code touch points from ADR-0036 (at minimum):
      - `crates/shell/src/execution/config_model.rs`
      - `crates/shell/src/builtins/shim_doctor/report.rs`
      - `crates/shell/src/builtins/health.rs`

- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md` — CI cadence plan (schema v4 cross-platform automation packs)
  - Owns (authoritative):
    - the CI checkpoint grouping and required gates for cross-platform parity
  - Must define:
    - machine-readable plan per `docs/project_management/system/templates/planning_pack/ci_checkpoint_plan.md.tmpl`
    - checkpoint groups that cover `WDD0..WDD2`
    - tasks.json alignment rule: `meta.checkpoint_boundaries` MUST equal the last slice in each checkpoint group

### Contract surfaces (required)

- `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md` — user-facing contract surface (CLI/config/exit codes/platform)
  - Owns (authoritative):
    - CLI contract for:
      - `substrate shim doctor` (text + `--json`)
      - `substrate health` (text + `--json`)
    - config contract for effective `world.enabled` resolution as used by diagnostics
    - exit code semantics for `substrate shim doctor` / `substrate health`
    - platform guarantees (Linux/macOS/Windows)
  - Must define:
    - **Effective world-enabled resolution contract** (no implied defaults):
      - the complete precedence order for `world.enabled` as used by diagnostics:
        - CLI flags `--world` / `--no-world`
        - workspace config `<workspace_root>/.substrate/workspace.yaml` (when enabled)
        - env override `SUBSTRATE_OVERRIDE_WORLD` (when no enabled workspace exists)
        - global config `$SUBSTRATE_HOME/config.yaml` (default `~/.substrate/config.yaml`)
      - absence semantics for `world.enabled` (when unset in workspace/global config)
      - invalid-value semantics for `SUBSTRATE_OVERRIDE_WORLD`
    - **Diagnostics behavior contract**:
      - when effective `world.enabled=false`:
        - `substrate shim doctor` MUST emit a machine-detectable “world disabled” status and MUST NOT execute world-backend probes for diagnostics purposes
        - world-deps “applied” probing MUST be skipped and MUST be reported as an explicit skipped/disabled status (non-error)
        - diagnostics MUST NOT imply world health or world-deps “applied” state when disabled
      - when effective `world.enabled=true`:
        - diagnostics MUST continue to surface backend unavailability/errors as “needs attention” (no masking)
    - **Text output constraints** (deterministic and testable):
      - required stable phrases/substrings for the “world disabled” and “world-deps skipped” states, aligned across `substrate health` and `substrate shim doctor` (see DR-0003)
      - required minimal remediation guidance when world is disabled (e.g., “run `substrate world enable`”)
    - **Exit codes**:
      - taxonomy reference: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
      - explicit statement of “informational surface” exit semantics:
        - successful report generation MUST exit `0` regardless of `needs_attention` vs `disabled`
        - non-zero exits MUST be limited to command execution failures (usage/config/serialization)
    - **No new config keys** invariant:
      - this feature MUST NOT introduce new config keys (ADR-0036)
  - Links (non-authoritative):
    - `docs/CONFIGURATION.md`
    - `docs/reference/env/contract.md`
    - `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`

- `docs/project_management/packs/draft/world-disabled-diagnostics/decision_register.md` — decision register (required)
  - Owns (authoritative):
    - DR-0001: JSON field names + enum spellings for world/world-deps status
    - DR-0002: legacy error-field behavior for disabled/skip states
    - DR-0003: operator-facing copy standardization across `substrate health` / `substrate shim doctor`
  - Must define:
    - each DR as exactly two options (A/B) with one selection and explicit rationale
    - for each DR: the impacted surfaces and the spec(s) that must be updated to reflect the selection
    - DR-0001 MUST select the exact JSON field paths for the status fields (ADR default: `world.status`, `world_deps.status`)
  - Links (non-authoritative):
    - `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/diagnostics-json-schema-spec.md`

- `docs/project_management/packs/draft/world-disabled-diagnostics/diagnostics-json-schema-spec.md` — diagnostics JSON schema spec (health + shim doctor)
  - Owns (authoritative):
    - the complete JSON output schema for:
      - `substrate shim doctor --json`
      - `substrate health --json`
    - the stable status field(s) and enum spellings for world + world-deps states (as selected by DR-0001)
    - compatibility policy for additive fields (ADR-0036)
  - Must define:
    - schema for both commands’ JSON outputs (not only the new fields)
    - field names, types, required/optional, and absence semantics for every field relevant to world/world-deps diagnostics
    - additive-fields contract:
      - new status fields MUST be additive (no removal/rename of existing fields)
      - unknown fields handling requirements for consumers
    - the status field paths (exact; per DR-0001):
      - initial ADR field paths: `world.status` and `world_deps.status`
    - the disabled/skipped encoding:
      - the exact values used to represent:
        - “world disabled” (world status)
        - “world-deps skipped because world disabled” (world-deps status)
      - status enum contract (semantic states; spellings per DR-0001):
        - world status MUST include semantic states equivalent to: `healthy`, `needs_attention`, `disabled`, `unknown`
        - world-deps status MUST include semantic states equivalent to: `ok`, `error`, `skipped_disabled`
      - when world is disabled, the “skipped because disabled” signal MUST be a status value (not a generic error string)
      - the exact behavior of any legacy error fields when disabled/skipped (as selected by DR-0002)
    - examples (authoritative):
      - `world.enabled=false` example payloads for both commands
      - `world.enabled=true` with broken backend example payloads for both commands
  - Links (non-authoritative):
    - `docs/project_management/system/templates/spec/schema-spec.md.tmpl`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/decision_register.md`

### Slice specs (required)

Slice specs MUST use the canonical layout:
- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/<SLICE_ID>/<SLICE_ID>-spec.md`

- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD0/WDD0-spec.md` — effective `world.enabled` resolution inside diagnostics
  - Owns (authoritative):
    - the WDD0 behavior delta and acceptance criteria (implementation-level), without redefining the `contract.md` CLI/config/exit-code tables
  - Must define:
    - acceptance criteria (AC-WDD0-*) that prove:
      - `substrate shim doctor` consults effective config using the canonical precedence contract from `contract.md`
      - the resolved effective `world.enabled` is carried into the doctor report model as an explicit status input for later classification
    - explicit test expectations (unit-level) aligned to ADR-0036 Validation Plan
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`

- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD1/WDD1-spec.md` — disabled-aware world + world-deps diagnostics classification
  - Owns (authoritative):
    - the WDD1 behavior delta and acceptance criteria (implementation-level)
  - Must define:
    - acceptance criteria (AC-WDD1-*) that prove (when `world.enabled=false`):
      - world-backend probes are not executed for diagnostics (no world probe calls solely for diagnostics)
      - world-deps “applied” computation is skipped
      - text output includes the required “disabled/skipped” signals per `contract.md`
      - JSON output includes the required status fields/values per `diagnostics-json-schema-spec.md`
    - explicit non-regression acceptance criteria for `world.enabled=true` with broken backend (“needs attention” remains visible)
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/diagnostics-json-schema-spec.md`

- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD2/WDD2-spec.md` — contract + validation coverage
  - Owns (authoritative):
    - the WDD2 acceptance criteria that make the ADR’s validation plan fully concrete and runnable
  - Must define:
    - acceptance criteria (AC-WDD2-*) that require:
      - the integration test described by ADR-0036 is implemented and asserts the disabled/skipped status fields deterministically
      - `manual_testing_playbook.md` exists and is fully deterministic (no “…” steps; no ambiguous expected outputs)
      - smoke scripts exist for Linux/macOS/Windows and mirror the manual playbook assertions
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/world-disabled-diagnostics/manual_testing_playbook.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/`

### Validation artifacts (required)

- `docs/project_management/packs/draft/world-disabled-diagnostics/manual_testing_playbook.md` — manual validation (cross-platform)
  - Owns (authoritative):
    - the deterministic operator validation procedure for the world-disabled and world-enabled/broken-backend cases
  - Must define:
    - exact setup steps for `$SUBSTRATE_HOME/config.yaml` with `world.enabled: false`
    - the exact commands to run:
      - `substrate shim doctor` (text + `--json`)
      - `substrate health` (text + `--json`)
    - deterministic assertions for:
      - text output required phrases
      - JSON status fields/values
      - exit codes
    - deterministic steps to validate the non-regression path:
      - `world.enabled: true` with an intentionally broken backend yields “needs attention” with actionable error details
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/diagnostics-json-schema-spec.md`

- `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/linux-smoke.sh` — automated Linux smoke validation
- `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/macos-smoke.sh` — automated macOS smoke validation
- `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/windows-smoke.ps1` — automated Windows smoke validation
  - Owns (authoritative):
    - the smoke assertions and their pass/fail exit code contract for the platform
  - Must define:
    - deterministic checks that validate the WDD contract surfaces without human intervention
    - explicit skip/abort rules (if privilege is required, the script MUST state the required invocation mode and MUST fail with a deterministic exit code when not satisfied)
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`

## Coverage matrix (surface → authoritative doc)

Every surface that ADR-0036 touches MUST appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| CLI contract: `substrate shim doctor` (world-disabled behavior) | `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md` | required disabled/skipped semantics; “MUST NOT probe” rule; non-regression when enabled |
| CLI contract: `substrate health` (world-disabled behavior) | `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md` | disabled is non-failure; required remediation guidance; enabled+broken remains “needs attention” |
| Config precedence + paths for effective `world.enabled` | `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md` | precedence order; config file paths; workspace config gating; absence semantics |
| Env vars used for effective config (`SUBSTRATE_OVERRIDE_WORLD`, `SUBSTRATE_HOME`) | `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md` | name; type; allowed values; defaults; invalid-value semantics; precedence vs config/flags |
| Exit codes for `substrate shim doctor` / `substrate health` | `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md` | taxonomy mapping; “informational surface” exit contract; non-zero failure modes |
| JSON output schema: `substrate shim doctor --json` | `docs/project_management/packs/draft/world-disabled-diagnostics/diagnostics-json-schema-spec.md` | full schema; new status fields; disabled/skipped encoding; legacy error-field behavior |
| JSON output schema: `substrate health --json` | `docs/project_management/packs/draft/world-disabled-diagnostics/diagnostics-json-schema-spec.md` | full schema; new status fields; disabled/skipped encoding; legacy error-field behavior |
| JSON status fields (`world.status`, `world_deps.status`) | `docs/project_management/packs/draft/world-disabled-diagnostics/diagnostics-json-schema-spec.md` | exact field paths; status enum values; required/optional + absence semantics; “skipped” as status (not error string) |
| Compatibility policy for additive JSON fields | `docs/project_management/packs/draft/world-disabled-diagnostics/diagnostics-json-schema-spec.md` | additive-only rule; unknown fields handling; deprecation policy |
| Decision points (DR-0001..DR-0003) | `docs/project_management/packs/draft/world-disabled-diagnostics/decision_register.md` | exactly-two-options A/B; selection; impacted surfaces; required spec updates |
| Slice definitions + AC IDs | `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD*/WDD*-spec.md` | behavior delta per slice; acceptance criteria IDs and checks |
| Manual validation | `docs/project_management/packs/draft/world-disabled-diagnostics/manual_testing_playbook.md` | exact commands; expected exit codes; expected text/JSON assertions |
| Smoke validation (Linux/macOS/Windows) | `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/*` | automated commands; expected pass/fail; required privileges/dependencies |
| Touch set + cascading implications | `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md` | explicit touched files; cross-platform risks; contradiction scan (health/doctor vs other diagnostics) |
| CI cadence checkpoints | `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md` | checkpoint groups; gates; rationale; tasks.json alignment |
| Task graph (automation) | `docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json` | tasks; dependencies; kickoff prompt paths; branch/worktree metadata |
| Policy surface | `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md` | this feature MUST NOT add or modify policy broker schemas, policy decisions, or enforcement mode |
| New environment variables | `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md` | this feature MUST NOT introduce new `SUBSTRATE_*`, `SHIM_*`, or `WORLD_*` environment variables |
| Protocol surface | `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md` | this feature MUST NOT add or modify any host↔agent wire/API protocol surface; only conditional probe *invocation* changes are allowed |
| Telemetry/log schema fields | `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md` | this feature MUST NOT add or modify structured log schema fields, trace span fields, or redaction rules |
| Filesystem semantics | `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md` | this feature MUST NOT introduce new filesystem diff/protected-path/path-rewrite semantics |
| Non-diagnostics `substrate world deps ...` commands | `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md` | this feature MUST NOT change `substrate world deps ...` behavior (beyond the diagnostics-only “skip when disabled” contract defined in `contract.md`) |
| “Why disabled” attribution UX | `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md` | this feature MUST NOT add or change any user-facing attribution of which layer disabled the world (CLI/workspace/env/global); it only reports the effective disabled state |

## Determinism checklist (must be satisfied before quality gate)

For every required spec document listed above, it MUST explicitly define:
- Inputs (all) + precedence order (if multiple inputs exist)
- Defaults (all) + absence semantics (including “unset” and “invalid value” behavior)
- Data model (types/constraints) for every serialized boundary (including both JSON outputs)
- Error model (exit codes + JSON error fields, if any) and failure posture (degrade vs fail-closed)
- Ordering/atomicity rules for diagnostics classification (especially “skip probes when disabled”)
- Security/redaction invariants (explicitly “no changes” if not in scope)
- Platform guarantees (Linux/macOS/Windows) and required validation evidence per platform

## Follow-ups (required to remove ambiguity)

- ADR-0036 requires additive JSON fields + stable enums but leaves final naming as “to be decided”.
  - `decision_register.md` MUST complete DR-0001 (field names + enum spellings), and `diagnostics-json-schema-spec.md` MUST reflect the selection.
- ADR-0036 references “legacy error-field behavior” for disabled/skip states without defining the contract.
  - `decision_register.md` MUST complete DR-0002 and `contract.md` + `diagnostics-json-schema-spec.md` MUST reflect the selected behavior.
- ADR-0036 requires explicit operator-facing disabled/skipped messaging but allows “exact phrasing may vary”.
  - `decision_register.md` MUST complete DR-0003 by selecting deterministic copy constraints (stable phrases/substrings) and `contract.md` MUST encode them as testable requirements.
- `diagnostics-json-schema-spec.md` MUST define the full existing JSON shapes for both commands to ensure the change is strictly additive.
  - This requires inspecting current outputs/implementation so “existing fields” are not accidentally dropped from the schema spec.
- `contract.md` MUST define absence semantics for `world.enabled` (workspace/global) and invalid/unset semantics for `SUBSTRATE_OVERRIDE_WORLD`, sourced from existing contract/implementation (no implied defaults).
- `tasks.json` is schema v4 and cross-platform, but currently has no tasks and no `meta.checkpoint_boundaries`.
  - Planning MUST populate tasks and add `meta.checkpoint_boundaries` aligned to `pre-planning/ci_checkpoint_plan.md`.
- ADR-0036 “Sequencing entry” is `TBD`.
  - Planning MUST add the sequencing entry and reference it from `plan.md`.
