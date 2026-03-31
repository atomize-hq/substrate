# world-disabled-diagnostics — spec manifest (pre-planning)

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-disabled-diagnostics/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`

External authoritative inputs (this feature MUST NOT redefine these surfaces):
- Effective config precedence and env-var parsing rules (including “workspace enabled ⇒ ignore `SUBSTRATE_OVERRIDE_*`”):
  - `docs/reference/env/contract.md` (`SUBSTRATE_OVERRIDE_WORLD`, `SUBSTRATE_HOME`, and the effective-config precedence stack)
- Config key schema and meaning for `world.enabled`:
  - `docs/CONFIGURATION.md`
- Exit code taxonomy:
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Slice IDs (canonical)

ADR-0036 uses placeholder slice IDs (`C0`, `C1`, `C2`). This feature MUST use feature-derived slice IDs per:
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`

Canonical slice IDs selected for this feature:
- Slice prefix: `WDD` (derived from “World Disabled Diagnostics”)
- `WDD0` — disabled-aware diagnostics for `substrate health` + `substrate shim doctor` (ADR-0036 C0–C2)

## Required spec documents (authoritative)

Each entry is feature-local (must live under `docs/project_management/packs/draft/world-disabled-diagnostics/`) and must be treated as authoritative for the surfaces listed.

Spec templates:
- `docs/project_management/system/templates/planning_pack/`
- `docs/project_management/system/templates/spec/`

### Planning pack scaffolding (required)

- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/spec_manifest.md`
  - Owns (authoritative):
    - required-doc selection for this feature directory
    - surface → authoritative-doc ownership map (coverage matrix)
    - follow-ups required to remove ADR ambiguity before quality gate
  - Must define (deterministic items):
    - an explicit “no implied surfaces” statement for every surface category that is not used by ADR-0036
    - the slice ID set for this feature (feature-derived; no `C0/C1/...`)
  - Links to (non-authoritative):
    - `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`
    - `docs/reference/env/contract.md`
    - `docs/CONFIGURATION.md`
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/impact_map.md`
  - Owns (authoritative):
    - touch set + cascading implications + cross-queue conflicts for slice `WDD0`
  - Must define (deterministic items):
    - explicit create/edit touch allowlists by path for `WDD0-{code,test,integ}`
    - explicit list of operator-doc update targets (by path) if any docs outside this feature pack must be updated
  - Links to (non-authoritative):
    - all feature-local docs listed in this section

- `docs/project_management/packs/draft/world-disabled-diagnostics/pre-planning/ci_checkpoint_plan.md`
  - Owns (authoritative):
    - checkpoint grouping + CI gates for this pack (schema v4 cross-platform automation packs)
  - Must define (deterministic items):
    - checkpoint groups and which slice(s) end each checkpoint group (must include `WDD0`)
    - alignment rule: `tasks.json` MUST add `meta.checkpoint_boundaries` and it MUST match this plan
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json`

- `docs/project_management/packs/draft/world-disabled-diagnostics/plan.md`
  - Owns (authoritative):
    - execution runbook + sequencing overview (including required validation evidence)
  - Must define (deterministic items):
    - orchestration branch name (MUST match `tasks.json` `meta.automation.orchestration_branch`)
    - canonical locations for pre-planning artifacts for this pack:
      - `pre-planning/spec_manifest.md`
      - `pre-planning/impact_map.md`
      - `pre-planning/ci_checkpoint_plan.md`
    - slice ordering (single-slice pack: `WDD0`)
    - required validation commands per ADR-0036 (unit + integration targets) and required platforms (Linux/macOS/Windows)
    - required completion criteria for manual playbook and smoke scripts (by exact path)
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`
    - slice specs under `docs/project_management/packs/draft/world-disabled-diagnostics/slices/`

- `docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json` (already exists)
  - Owns (authoritative):
    - task IDs, dependency graph, and automation metadata (branch/worktree/prompt paths)
  - Must define (deterministic items):
    - `meta.checkpoint_boundaries` (required for schema v4 cross-platform automation packs)
    - triad tasks for `WDD0`:
      - `WDD0-code`, `WDD0-test`, `WDD0-integ`
    - each task’s acceptance criteria MUST reference `AC-WDD0-*` IDs from `slices/WDD0/WDD0-spec.md`
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD0/WDD0-spec.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/plan.md`

- `docs/project_management/packs/draft/world-disabled-diagnostics/session_log.md`
  - Owns (authoritative):
    - append-only planning/execution log for this pack
  - Must define (deterministic items):
    - initialization from `docs/project_management/system/templates/planning_pack/session_log.md.tmpl`
    - “every task start/end must be logged with timestamp + task id” rule
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/world-disabled-diagnostics/tasks.json`

- `docs/project_management/packs/draft/world-disabled-diagnostics/quality_gate_report.md`
  - Owns (authoritative):
    - planning quality gate outcome for starting triads
  - Must define (deterministic items):
    - initialization from `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`
    - rule: execution triads MUST NOT start unless `RECOMMENDATION: ACCEPT` is present
  - Links to (non-authoritative):
    - every required artifact referenced by `RECOMMENDATION`

### Feature contract + decisions (required by ADR-0036)

- `docs/project_management/packs/draft/world-disabled-diagnostics/decision_register.md`
  - Owns (authoritative):
    - DR-0001 — JSON field names + enum spellings for world/world-deps statuses (across both `substrate health --json` and `substrate shim doctor --json`)
    - DR-0002 — legacy error-field behavior for disabled/skip states (ensure disabled/skip is carried via explicit status enums)
    - DR-0003 — operator-facing copy standardization for disabled/skip across `substrate health` and `substrate shim doctor`
  - Must define (deterministic items):
    - exactly two options (A/B) per DR and exactly one selection per DR
    - the exact surfaces impacted by each DR (which spec docs must change)
  - Links to (non-authoritative):
    - `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`

- `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`
  - Owns (authoritative):
    - operator-facing contract changes introduced by ADR-0036 for:
      - `substrate shim doctor` (text output; behavior when world disabled vs enabled)
      - `substrate health` (text output; classification when world disabled vs enabled-but-broken)
    - exit-code meanings for these diagnostics commands (taxonomy-aligned; no overrides unless explicitly declared)
    - platform guarantees (Linux/macOS/Windows)
  - Must define (deterministic items):
    - in-scope commands:
      - `substrate shim doctor`
      - `substrate shim doctor --json`
      - `substrate health`
      - `substrate health --json`
    - world-disabled behavior (singular; testable) for both commands:
      - when effective `world.enabled=false`, status MUST be reported as disabled/skipped (non-error)
      - when effective `world.enabled=true` and backend is broken/unreachable, status MUST be reported as attention required/needs attention with actionable error detail
    - operator-facing copy requirements selected by DR-0003 (singular and testable) for:
      - world backend disabled
      - world-deps skipped due to world disabled
      - required remediation guidance element(s), including `substrate world enable`
    - explicit statement: this feature introduces no new config keys and no new environment variables
    - exit codes:
      - taxonomy reference: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
      - explicit exit-code mapping for `substrate health` and `substrate shim doctor`, including whether “attention required” affects exit status (must be singular and testable)
    - platform guarantees:
      - Linux/macOS/Windows MUST expose the same disabled/skip semantics in text output
  - Links to (non-authoritative):
    - `docs/reference/env/contract.md` (effective config precedence; `SUBSTRATE_OVERRIDE_WORLD` ignore rule)
    - `docs/CONFIGURATION.md` (config key semantics)
    - `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/decision_register.md`

- `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`
  - Owns (authoritative):
    - additive JSON output contract changes for:
      - `substrate shim doctor --json`
      - `substrate health --json`
  - Must define (deterministic items):
    - compatibility policy: additive-only; MUST NOT rename/remove existing fields
    - exact field paths and enum spellings selected by DR-0001 for:
      - world backend status field (ADR baseline name: `world.status`) with semantic values for: healthy, needs-attention, disabled, unknown
      - world-deps status field (ADR baseline name: `world_deps.status`) with semantic values for: ok, error, skipped-disabled
    - emission rules and absence semantics:
      - when effective `world.enabled=false`, the DR-0001-selected world status field MUST equal the DR-0001-selected disabled enum value and the DR-0001-selected world-deps status field MUST equal the DR-0001-selected skipped-disabled enum value
      - when effective `world.enabled=true`, disabled/skip values MUST NOT be emitted
      - “unknown” handling rule(s) (single deterministic fallback)
    - legacy error fields:
      - exact behavior for any existing error fields when status is disabled/skipped (per DR-0002)
    - at least one example payload for:
      - disabled world (`world.enabled=false`)
      - enabled world with broken backend (needs attention)
      - enabled world healthy
  - Links to (non-authoritative):
    - `docs/project_management/system/templates/spec/schema-spec.md.tmpl`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/decision_register.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`

### Validation artifacts (required by ADR-0036)

- `docs/project_management/packs/draft/world-disabled-diagnostics/manual_testing_playbook.md`
  - Owns (authoritative):
    - deterministic manual validation steps for ADR-0036 across supported platforms
  - Must define (deterministic items):
    - preconditions for each platform (including how to set `world.enabled: false` via config patch)
    - exact commands to run:
      - `substrate shim doctor`
      - `substrate shim doctor --json`
      - `substrate health`
      - `substrate health --json`
    - expected key observations for each command (text + JSON), including:
      - disabled/skipped statuses when `world.enabled=false`
      - needs-attention behavior when `world.enabled=true` and backend is intentionally broken
    - expected exit codes for each command path (aligned to `contract.md`)
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`

- Smoke scripts (feature-local; cross-platform):
  - `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/macos-smoke.sh`
  - `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/windows-smoke.ps1`
  - Own (authoritative):
    - automated validation steps per platform and pass/fail expectations aligned to `manual_testing_playbook.md`
  - Must define (deterministic items):
    - exact commands executed and exact assertions performed for disabled vs enabled-but-broken cases
    - exit-code expectations for the smoke scripts themselves
  - Link to (non-authoritative):
    - `docs/project_management/packs/draft/world-disabled-diagnostics/manual_testing_playbook.md`

### Slice specs (required)

Slice specs MUST use the canonical layout:
- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/<SLICE_ID>/<SLICE_ID>-spec.md`

- `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD0/WDD0-spec.md`
  - Owns (authoritative):
    - vertical slice behavior + acceptance criteria for ADR-0036 C0–C2 (implementation-level)
  - Must define (deterministic items):
    - acceptance criteria (AC-WDD0-*) that prove:
      - effective `world.enabled` resolution is consulted by diagnostics (not hard-coded)
      - when `world.enabled=false`, diagnostics do not execute world-backend probes and still return a report with disabled/skipped statuses
      - when `world.enabled=true` and backend is broken/unreachable, diagnostics surface needs-attention behavior (no masking)
      - JSON outputs contain the new status fields and enum values as specified by `world-disabled-diagnostics-json-schema-spec.md`
      - text outputs match the disabled/skip copy requirements in `contract.md`
    - explicit non-goals/out-of-scope list matching ADR-0036 non-goals/out-of-scope
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`
    - `docs/project_management/packs/draft/world-disabled-diagnostics/decision_register.md`

## Coverage matrix (surface → authoritative doc)

Every surface touched by ADR-0036 must appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| CLI commands in scope (`substrate health`, `substrate shim doctor`) | `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md` | command names; which output surfaces are in scope (text + `--json`); success/no-op semantics; platform scope |
| Effective-config precedence for `world.enabled` (including “workspace enabled ⇒ ignore `SUBSTRATE_OVERRIDE_WORLD`”) | `docs/reference/env/contract.md` | precedence ordering; ignore rule; default config path when `SUBSTRATE_HOME` unset |
| Env var `SUBSTRATE_OVERRIDE_WORLD` | `docs/reference/env/contract.md` | allowed values; parsing rules; precedence; explicit statement that it can yield effective `world.enabled=false` |
| Env var `SUBSTRATE_HOME` | `docs/reference/env/contract.md` | default when unset; interaction with `$SUBSTRATE_HOME/config.yaml` lookup |
| Config key `world.enabled` (YAML) | `docs/CONFIGURATION.md` | schema type; meaning; default posture |
| World-disabled text output (both commands) | `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md` | copy contract selected by DR-0003 (singular + testable); non-error posture; remediation element(s) |
| World-enabled-but-broken text output classification | `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md` | needs-attention classification rule and required error-detail elements |
| Exit codes for diagnostics commands | `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md` | taxonomy reference + explicit statement about exit `0` vs non-zero command failures |
| “Skip probes” behavior when world disabled | `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD0/WDD0-spec.md` | operational definition of “probe” (world backend snapshot collection; world-deps applied probing); how tests assert no probing occurs |
| JSON output additions (`substrate health --json`, `substrate shim doctor --json`) | `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md` | exact field names, enum spellings, emission rules, absence semantics, examples, additive compatibility |
| Legacy error-field behavior in JSON when disabled/skip applies | `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md` | explicit rule per DR-0002 (must not encode skip purely as an error string) |
| A/B decisions for JSON naming and copy standardization | `docs/project_management/packs/draft/world-disabled-diagnostics/decision_register.md` | DR-0001/DR-0002/DR-0003 options + one explicit selection each |
| Manual validation | `docs/project_management/packs/draft/world-disabled-diagnostics/manual_testing_playbook.md` | deterministic preconditions, exact commands, expected key output lines, expected JSON assertions, expected exit codes |
| Smoke validation (Linux/macOS/Windows) | `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/*` | deterministic smoke commands and assertions aligned to manual playbook |
| Platform guarantees | `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md` | explicit Linux/macOS/Windows parity statement for disabled/skip semantics |

Explicit “MUST NOT change” statements for out-of-scope surface categories (ADR-0036):
- Protocol surfaces: MUST NOT change (no new host↔agent RPC, endpoints, or IPC framing changes).
- Telemetry/log schema: MUST NOT change (no new trace span fields; no new log schema fields).
- Policy evaluation/enforcement: MUST NOT change.
- Filesystem semantics/protected paths: MUST NOT change.
- Config keys and env vars: MUST NOT introduce new keys or variables (only consume existing `world.enabled`, `SUBSTRATE_OVERRIDE_WORLD`, `SUBSTRATE_HOME`).

## Determinism checklist (must be satisfied before quality gate)

Each required document must explicitly define the items below (no implied behavior).

### `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`
- CLI surface (text):
  - exact statements for world-disabled behavior for both commands
  - exact statements for enabled-but-broken behavior (no masking)
  - the deterministic operator-facing copy requirements selected by DR-0003
- Exit codes:
  - taxonomy reference: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
  - explicit “informational surface” exit semantics for `health` and `shim doctor` (or explicit override + rationale)
- Config/env statements (no redefinition):
  - explicit statement that precedence/parsing is owned by `docs/reference/env/contract.md`
  - explicit statement that config key semantics are owned by `docs/CONFIGURATION.md`
- Platform guarantees:
  - explicit parity statement across Linux/macOS/Windows

### `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`
- Exact schema additions (additive-only) for both commands’ JSON outputs:
  - final field names + enum spellings (from DR-0001)
  - emission rules and explicit absence semantics (disabled vs enabled)
  - legacy error-field behavior (from DR-0002)
  - examples for disabled/healthy/needs-attention cases

### `docs/project_management/packs/draft/world-disabled-diagnostics/slices/WDD0/WDD0-spec.md`
- Acceptance criteria:
  - `AC-WDD0-*` list covering text + JSON + “skip probes” invariants
  - explicit negative assertions:
    - when `world.enabled=false`, diagnostics MUST NOT run world backend snapshot collection or world-deps applied probing
    - when `world.enabled=true`, failures MUST remain visible (no masking)
- Scope hygiene:
  - explicit non-goals matching ADR-0036
  - explicit statement: no new config keys, no new env vars

### `docs/project_management/packs/draft/world-disabled-diagnostics/decision_register.md`
- DR-0001: JSON field names + enum spellings (A/B; one selection)
- DR-0002: legacy error-field behavior when disabled/skip applies (A/B; one selection)
- DR-0003: operator-facing copy standardization (A/B; one selection)

### `docs/project_management/packs/draft/world-disabled-diagnostics/manual_testing_playbook.md`
- Deterministic manual cases for:
  - disabled world (`world.enabled=false`)
  - enabled world with intentionally broken backend
- Exact expected outputs (key lines and JSON assertions) and expected exit codes

### `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/*`
- Cross-platform smoke assertions aligned to manual playbook:
  - disabled world shows disabled/skip statuses (non-error)
  - enabled-but-broken shows needs-attention (error details present)

## Follow-ups

Record missing/ambiguous ADR intent here (do not patch ADRs from this step).

1) ADR-0036 leaves the health JSON contract underspecified
   - Issue: ADR-0036 states `substrate health --json` MUST include explicit stable status enums, but only enumerates field paths for `substrate shim doctor --json`.
   - Required fix: in DR-0001 + `world-disabled-diagnostics-json-schema-spec.md`, define one deterministic contract for `substrate health --json` (field paths + enum spellings) and ensure it is aligned with `substrate shim doctor --json`.

2) Operator-facing copy is explicitly non-deterministic in ADR-0036
   - Issue: ADR-0036 includes “exact phrasing may vary” for disabled messaging; tests and operator expectations need a deterministic contract.
   - Required fix: DR-0003 + `contract.md` MUST choose either:
     - exact line templates (recommended for stable UX), or
     - exact required substrings + ordering rules (if full templates are not desired).

3) Legacy JSON error-field behavior for disabled/skip is not defined
   - Issue: ADR-0036 requires the disabled/skip signal be a status enum (not a generic error string) but does not define whether existing error fields are omitted, nulled, or still populated.
   - Required fix: DR-0002 + `world-disabled-diagnostics-json-schema-spec.md` MUST define one deterministic behavior for legacy error fields when the DR-0001-selected world status / world-deps status fields carry the disabled/skipped-disabled enum values.

4) Exit code semantics are marked as an ADR assumption
   - Issue: ADR-0036 assumes `substrate health` and `substrate shim doctor` exit `0` when a report is successfully generated, regardless of “needs attention” classification.
   - Required fix: `contract.md` MUST codify the actual exit-code mapping with singular, testable rules. If the current implementation differs from ADR-0036’s assumption, planning MUST record the discrepancy and correct ADR-0036 (or explicitly supersede it) before quality gate.

5) “Skip probes” needs a testable operational definition
   - Issue: ADR-0036 requires diagnostics to skip world-backend probes and world-deps “applied” probing when disabled, but does not define the exact boundary of “probe”.
   - Required fix: `slices/WDD0/WDD0-spec.md` MUST define which operations are forbidden when disabled (e.g., world-agent socket calls, snapshot collection, world-deps applied computation) and how unit/integration tests prove the short-circuit.
