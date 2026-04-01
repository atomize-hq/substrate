# make-doctor-health-output-explain-why — spec manifest (pre-planning)

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`

## Slice IDs (canonical)

ADR-0037 uses placeholder slice IDs (`C0`, `C1`). This feature MUST use feature-derived slice IDs per:
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`

Canonical slice IDs selected for this feature:
- Slice prefix: `DHO` (derived from “Doctor/Health Output”)
- `DHO0` — doctor text output: correct disable attribution (ADR “C0”)
- `DHO1` — doctor/health JSON additions + health attribution surface (ADR “C1”)

## Required spec documents (authoritative)

Each entry lists:
- what surfaces it owns (authoritative), and
- what it links to (non-authoritative).

Spec templates:
- `docs/project_management/system/templates/spec/`

### Planning pack scaffolding (required)

- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/spec_manifest.md` — spec selection + ownership map (this file)
  - Owns (authoritative):
    - the required-doc set for this feature directory
    - the coverage matrix (surface → authoritative doc)
    - follow-ups required to remove ambiguity before quality gate
    - explicit “no implied surfaces” constraints for every surface category that is NOT used by this feature
  - Must define:
    - the exact required-doc set under `docs/project_management/packs/draft/make-doctor-health-output-explain-why/`
    - a coverage matrix that assigns every ADR-touched surface to exactly one authoritative doc
    - a follow-ups list that is sufficient to remove all ADR ambiguity before quality gate
    - an explicit “MUST NOT change” statement for each out-of-scope surface category listed in the coverage matrix
  - Links (non-authoritative):
    - `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`

- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/plan.md` — execution runbook + sequencing overview
  - Owns (authoritative):
    - slice ordering (`DHO0` then `DHO1`)
    - task execution guardrails (triad workflow, worktree boundaries)
    - required validation evidence at slice completion (tests + manual + smoke)
  - Must define:
    - the orchestration branch (MUST match `tasks.json` `meta.automation.orchestration_branch`)
    - the canonical spec ownership map location for this pack: `pre-planning/spec_manifest.md`
    - the canonical impact map location for this pack: `pre-planning/impact_map.md`
    - the canonical CI checkpoint plan location for this pack: `pre-planning/ci_checkpoint_plan.md`
    - the full slice list (`DHO0`, `DHO1`) and each slice’s objective (single sentence each)
    - the validation evidence required at the end of each slice:
      - unit test targets and integration test targets per ADR-0037
      - manual playbook completion
      - smoke scripts completion (Linux/macOS/Windows)
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/tasks.json`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md`
    - slice specs under `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/`

- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/tasks.json` — triad task graph + acceptance criteria
  - Owns (authoritative):
    - task IDs, dependency graph, and automation metadata (branch/worktree/prompt paths)
  - Must define:
    - `meta.checkpoint_boundaries` (required for schema v4 cross-platform automation packs): an array of slice IDs that are the last slice in each checkpoint group in `pre-planning/ci_checkpoint_plan.md`
    - triad tasks for each slice:
      - `DHO0-code`, `DHO0-test`, `DHO0-integ`
      - `DHO1-code`, `DHO1-test`, `DHO1-integ`
    - each task’s acceptance criteria MUST reference slice-spec AC IDs (no freeform acceptance)
    - kickoff prompt paths for every task (MUST exist on disk)
  - Links (non-authoritative):
    - slice specs under `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/`

- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/session_log.md` — planning + execution audit log
  - Owns (authoritative):
    - the append-only record of planning/execution events for this pack
  - Must define:
    - the file MUST be initialized from `docs/project_management/system/templates/planning_pack/session_log.md.tmpl`
    - every task start/end MUST be recorded with timestamp + task id
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/tasks.json`

- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/impact_map.md` — touch set + cascading implications + cross-queue conflicts
  - Owns (authoritative):
    - the explicit touch set (create/edit/deprecate/delete) for this feature
    - the explicit list of cascading implications and contradiction risks
  - Must define:
    - the touch set MUST enumerate every repo file expected to be created/edited/deprecated/deleted by this feature
    - the touch set MUST enumerate the cross-platform touch implications implied by ADR-0037 (Linux/macOS/Windows)
    - the touch set MUST enumerate the exact JSON output surfaces to be modified (commands + `--json` output objects)
    - the touch set MUST enumerate the exact config surfaces referenced by the new attribution behavior:
      - `<workspace>/.substrate/workspace.yaml`
      - `$SUBSTRATE_HOME/config.yaml`
      - `SUBSTRATE_HOME`
      - `SUBSTRATE_OVERRIDE_WORLD`
      - `world.enabled`
  - Links (non-authoritative):
    - `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md`

- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/ci_checkpoint_plan.md` — cross-platform CI cadence (automation + cross-platform)
  - Owns (authoritative):
    - the checkpoint grouping for `DHO0..DHO1` across Linux/macOS/Windows
    - the required CI/smoke gates per checkpoint group
  - Must define:
    - checkpoint groups (each group is a contiguous slice range ending at a slice in `tasks.json` `meta.checkpoint_boundaries`)
    - the exact gate list per checkpoint group (compile parity, CI testing, feature smoke) and the pass criteria
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/tasks.json`

### Contract + schemas (required)

- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md` — authoritative user-facing contract (doctor/health attribution)
  - Owns (authoritative):
    - CLI contract for the attribution behavior and output text constraints for:
      - `substrate host doctor`
      - `substrate world doctor`
      - `substrate health`
    - config precedence contract as consumed by attribution (no behavior changes; attribution MUST match effective winner):
      - CLI flags `--world` / `--no-world`
      - workspace patch `<workspace>/.substrate/workspace.yaml`
      - override env `SUBSTRATE_OVERRIDE_WORLD` (legacy gating behavior)
      - global patch `$SUBSTRATE_HOME/config.yaml` (default `~/.substrate/config.yaml`)
      - default config
    - config schema subset required by this feature:
      - `world.enabled` (boolean) meaning and absence semantics
    - output redaction invariants for both text and JSON surfaces:
      - env var printing MUST NOT leak raw env values beyond the var name (explicit allow: the fixed token `SUBSTRATE_OVERRIDE_WORLD=disabled`)
      - config paths MUST use tokenized display paths (`$SUBSTRATE_HOME/config.yaml`, `<workspace>/.substrate/workspace.yaml`) rather than absolute host paths
    - the fallback attribution behavior when the disable source cannot be determined without misattribution
    - exit codes for the commands above (taxonomy reference; no overrides)
    - platform parity guarantees and allowed divergences (Linux/macOS/Windows)
  - Must define:
    - the exact attribution line templates for each disable source (CLI flag, override env, workspace patch, global patch)
    - the exact trigger for emitting attribution (effective `world.enabled=false`) and the exact trigger for omitting attribution (effective `world.enabled=true`)
    - the deterministic rule that selects the disable source (MUST match the effective winner used for `world.enabled`)
    - the exact fallback message (string) and the exact trigger condition(s) that use it
    - the redaction rules as testable output constraints for both text and JSON:
      - env values are never printed (except the fixed allowlisted token)
      - absolute host paths are never printed; only tokenized display paths are printed
    - `SUBSTRATE_HOME`: meaning, default when unset, and interaction with the global config patch path and display token
    - explicit “no behavior change” statements for:
      - world enable/disable precedence
      - exit code meanings (taxonomy reference only; no overrides)
  - Links (non-authoritative):
    - `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/decision_register.md`

- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md` — doctor/health JSON output additions (additive)
  - Owns (authoritative):
    - the JSON schema additions for:
      - `substrate host doctor --json`
      - `substrate world doctor --json`
      - `substrate health --json`
    - additive compatibility policy and “unknown fields” handling rules for these JSON outputs
  - Must define:
    - whether the existing “world enabled” field in these JSON objects is named `world_enabled`, `world.enabled`, or another spelling (no implied naming)
    - the precise condition for emitting the new fields (MUST be based on the effective world-enabled value)
    - `world_disable_reason`:
      - type, allowed enum values, and when each value applies
      - absence semantics when world is enabled
    - `world_disable_source`:
      - object schema (keys, types, required/optional, conditional presence rules)
      - required constant-value rules and their sources in `contract.md` (flag/env/path_display/key display tokens)
      - `value_display` type and constant value rule
    - at least one example payload for each disable source (`cli_flag`, `override_env`, `workspace_patch`, `global_patch`, `default`)
    - explicit statement that the change is additive-only and must not rename/remove existing fields
  - Links (non-authoritative):
    - `docs/project_management/system/templates/spec/schema-spec.md.tmpl`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/decision_register.md`

- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/decision_register.md` — A/B decisions and selections
  - Owns (authoritative):
    - DR-0001 — Attribution implementation strategy:
      - A) provenance-based (config explain)
      - B) heuristic-based (must still match effective precedence)
    - DR-0002 — JSON contract:
      - field names, enum values, and redaction strategy for display paths and env/flag rendering
  - Must define:
    - exactly two options (A/B) per DR with explicit tradeoffs
    - exactly one selection per DR
    - the exact impacted contract surfaces per selection (which spec docs must change)
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md`

### Slice specs (required)

Slice specs MUST use the canonical layout:
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/<SLICE_ID>/<SLICE_ID>-spec.md`

- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO0/DHO0-spec.md` — doctor text output attribution
  - Owns (authoritative):
    - the DHO0 behavior delta and acceptance criteria (implementation-level), without redefining the `contract.md` CLI/config/exit-code contract
  - Must define:
    - acceptance criteria (AC-DHO0-*) that prove, for `substrate host doctor` and `substrate world doctor` (text):
      - attribution is omitted when world is enabled
      - attribution is present when world is disabled and matches the effective disable source
      - the attribution strings meet the exact wording requirements in `contract.md`
      - the “source unknown” fallback string from `contract.md` is emitted only when the contract-defined disable source cannot be determined without misattribution
    - explicit platform parity expectations for Linux/macOS/Windows text outputs
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/decision_register.md`

- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO1/DHO1-spec.md` — JSON additions + health attribution surface
  - Owns (authoritative):
    - the DHO1 behavior delta and acceptance criteria (implementation-level)
  - Must define:
    - acceptance criteria (AC-DHO1-*) that prove:
      - doctor JSON outputs include the new fields when world is disabled and omit them when enabled
      - health text output displays the same attribution string as doctor text output when world is disabled
      - health JSON output includes the new fields when world is disabled and omits them when enabled
      - JSON field names, enum values, and object structure match `doctor-health-output-attribution-schema-spec.md`
    - explicit platform parity expectations for Linux/macOS/Windows health outputs (text + JSON)
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/decision_register.md`

### Validation artifacts (required)

- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/manual_testing_playbook.md` — manual validation (cross-platform)
  - Owns (authoritative):
    - the deterministic operator validation procedure for attribution correctness (text + JSON) on Linux/macOS/Windows
  - Must define:
    - exact setup steps to exercise each disable source deterministically:
      - CLI `--no-world`
      - env `SUBSTRATE_OVERRIDE_WORLD=disabled` (including workspace-gating behavior, if applicable)
      - workspace config `<workspace>/.substrate/workspace.yaml` with `world.enabled: false`
      - global config `$SUBSTRATE_HOME/config.yaml` with `world.enabled: false`
    - the exact commands to run (text + `--json`) and the deterministic assertions for:
      - required attribution line substrings in text output
      - required JSON fields/values when disabled
      - absence of new JSON fields when enabled
      - exit code expectations (taxonomy reference; no overrides)
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md`
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md`

- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/smoke/linux-smoke.sh` — automated Linux smoke validation
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/smoke/macos-smoke.sh` — automated macOS smoke validation
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/smoke/windows-smoke.ps1` — automated Windows smoke validation
  - Owns (authoritative):
    - the non-interactive smoke assertions and their pass/fail exit code contract for the platform
  - Must define:
    - deterministic checks that validate the attribution surfaces in `contract.md` for both text and JSON outputs
    - explicit skip/abort rules (if a dependency is required, the script MUST state it and MUST fail with a deterministic exit code when not satisfied)
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/manual_testing_playbook.md`

## Coverage matrix (surface → authoritative doc)

Every surface that ADR-0037 touches MUST appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| CLI contract: `substrate host doctor` (text attribution) | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md` | exact attribution line templates; emit/omit rules; redaction rules |
| CLI contract: `substrate world doctor` (text attribution) | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md` | exact attribution line templates; emit/omit rules; redaction rules |
| CLI contract: `substrate health` (text attribution) | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md` | exact attribution line templates; emit/omit rules; parity with doctor attribution copy |
| CLI flags referenced by attribution (`--world`, `--no-world`, `--json`) | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md` | precedence statement (no behavior change); exact printed flag token(s) |
| Config precedence for effective `world.enabled` | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md` | total ordering + workspace gating rule + “attribution matches effective winner” invariant |
| Config file paths + stable display-path tokens | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md` | token strings (`$SUBSTRATE_HOME/config.yaml`, `<workspace>/.substrate/workspace.yaml`); absolute-path suppression rule |
| Env var: `SUBSTRATE_HOME` | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md` | meaning, default when unset, and interaction with `$SUBSTRATE_HOME/config.yaml` display |
| Config key schema: `world.enabled` | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md` | type, meaning, and absence semantics |
| Env override surface (`SUBSTRATE_OVERRIDE_WORLD`) | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md` | allowed value(s) referenced by outputs; workspace gating rule; redaction rules |
| Fallback attribution message (“source unknown”) | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md` | exact string; exact trigger condition(s); must not misattribute |
| Exit codes for doctor/health commands | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md` | taxonomy reference (`docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`); explicit “no override/no change” statement |
| Platform parity (Linux/macOS/Windows) | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md` | command availability per platform; parity guarantees; any allowed divergences must be enumerated |
| JSON schema: `substrate host doctor --json` additions | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md` | field names, types, optionality, emit/omit rules, examples |
| JSON schema: `substrate world doctor --json` additions | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md` | field names, types, optionality, emit/omit rules, examples |
| JSON schema: `substrate health --json` additions | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md` | field names, types, optionality, emit/omit rules, examples |
| JSON enum: `world_disable_reason` | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md` | exact allowed values and mapping rules for each disable source (including `default`) |
| JSON object: `world_disable_source` | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md` | keys, conditional presence rules, and constant-value constraints |
| JSON compatibility policy | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md` | additive-only rule; unknown fields handling; deprecation policy |
| Decision points (DR-0001..DR-0002) | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/decision_register.md` | exactly-two-options A/B; selection; impacted surfaces; required spec updates |
| Slice definitions + AC IDs | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO*/DHO*-spec.md` | one behavior delta per slice; acceptance criteria IDs and checks |
| Manual validation | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/manual_testing_playbook.md` | exact commands; expected exit codes; expected text/JSON assertions |
| Smoke validation (Linux/macOS/Windows) | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/smoke/*` | automated commands; deterministic pass/fail; dependency/skip rules |
| Touch set + cascading implications | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/impact_map.md` | explicit touched files; cross-platform risks; contradiction scan |
| CI cadence checkpoints | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/ci_checkpoint_plan.md` | checkpoint groups; gates; tasks.json alignment |
| Task graph (automation) | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/tasks.json` | tasks; dependencies; kickoff prompt paths; branch/worktree metadata |
| Policy surface | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/spec_manifest.md` | this feature MUST NOT add or modify policy broker schemas, policy decisions, or enforcement mode |
| New environment variables | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/spec_manifest.md` | this feature MUST NOT introduce new `SUBSTRATE_*`, `SHIM_*`, or `WORLD_*` environment variables |
| Protocol surface | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/spec_manifest.md` | this feature MUST NOT add or modify any host↔agent wire/API protocol surface |
| Telemetry/log schema fields | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/spec_manifest.md` | this feature MUST NOT add or modify structured log schema fields, trace span fields, or redaction helper behavior |
| Filesystem semantics | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/spec_manifest.md` | this feature MUST NOT introduce new filesystem diff/protected-path/path-rewrite semantics |
| World enable/disable behavior | `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/spec_manifest.md` | this feature MUST NOT change when/why world isolation is enabled/disabled; attribution/messaging only |

## Determinism checklist (must be satisfied before quality gate)

For every required spec document listed above, it MUST explicitly define:
- Inputs (all) + precedence order (if multiple inputs exist)
- Defaults (all) + absence semantics (including “unset” and “invalid value” behavior)
- Data model (types/constraints) for every serialized boundary (including all `--json` outputs changed by this feature)
- Error model (exit codes + JSON error fields, if any) and failure posture (unchanged for this feature unless explicitly specified)
- Security/redaction invariants (explicitly “no changes” where out of scope)
- Platform guarantees (Linux/macOS/Windows) and required validation evidence per platform

## Follow-ups (required to remove ambiguity)

- ADR-0037 is internally inconsistent about DR-0001:
  - The “Recommendation” section recommends Option A (provenance), but “Decision Summary” selects Option B (heuristic).
  - `decision_register.md` MUST resolve DR-0001 with one explicit selection, and `contract.md` MUST state the determinism requirement: attribution MUST match the effective winner for `world.enabled`.
- ADR-0037 precedence wording is internally inconsistent across sections:
  - “Config” precedence lists workspace before env, and also states env applies only when no workspace exists.
  - Option B prose describes CLI → env → workspace → global without stating the workspace gating rule.
  - `contract.md` MUST define one deterministic precedence model (sourced from existing config resolution) and the attribution logic MUST follow it.
- ADR-0037 introduces `world_disable_reason` value `default` but does not define when it occurs.
  - `doctor-health-output-attribution-schema-spec.md` MUST define the trigger condition for `default`, or `decision_register.md` MUST decide to remove/rename it and update the ADR-derived contract accordingly.
- ADR-0037 uses `world_enabled=false` in multiple places but does not define the existing JSON field naming for doctor/health outputs.
  - `doctor-health-output-attribution-schema-spec.md` MUST inventory the current JSON objects and explicitly define the naming of the “world enabled” field for each command.
- Validation artifact mismatch risk:
  - ADR-0037 says smoke scripts are not required, but `tasks.json` `meta.behavior_platforms_required` includes Linux/macOS/Windows.
  - Planning MUST either (1) author the smoke scripts listed in this manifest, or (2) change the tasks meta/platform requirements in a way that is consistent with triad standards (no implied gates).
- Slice ID reconciliation:
  - ADR-0037 uses generic slice IDs (`C0`, `C1`); planning artifacts MUST use `DHO0`, `DHO1` and MUST include a mapping (this manifest provides the mapping).
