# best-effort-distro-package-manager — spec manifest (pre-planning)

This file enumerates every contract, path, validation, and decision surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`
- Supporting references that inform this manifest but are not owned by this pack:
  - `docs/project_management/intake/adrs/detecting_badger_adr_intake.md`
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
  - `scripts/substrate/install-substrate.sh`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`

## Slice IDs (canonical)

ADR-0031 uses placeholder slice IDs (`C0`, `C1`, `C2`). This feature MUST use feature-derived slice IDs per:
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`

Canonical slice IDs selected for this feature:
- Slice prefix: `BEDPM` (derived from `best-effort-distro-package-manager`)
- `BEDPM0` — best-effort distro detection, mapping, and stable installer reporting
- `BEDPM1` — explicit override precedence, `PATH` fallback, and fail-closed error posture
- `BEDPM2` — wrapper exit-status pass-through and operator/env-doc propagation
- `BEDPM3` — hermetic Linux validation for precedence, mapping, warning, remediation, and wrapper pass-through

## Required spec documents (authoritative)

This ADR requires one user-facing contract doc, one decision register, one impact map, one feature-local CI checkpoint plan, one execution plan, one manual validation playbook, one Linux smoke script, one session log, one quality gate report, and four canonical slice specs.

No separate protocol, schema, telemetry, filesystem-semantics, platform-parity, or compatibility doc is selected.
- This ADR introduces no wire or IPC contract.
- This ADR introduces no stable serialized file format or additive JSON schema.
- Installer environment-variable surfaces stay small and installer-local; `contract.md` owns them.
- This ADR introduces no structured trace field and no structured log schema field.
- The only filesystem read contract is `/etc/os-release`; `contract.md` owns that path and its absence semantics.
- The behavior delta is Linux-only; `contract.md` owns the explicit no-change contract for macOS and Windows.
- ADR-0031 does not require a migration, deprecation window, or staged rollout.
- ADR-0031 lift data says `cross_platform=false`; `pre-planning/ci_checkpoint_plan.md` is required planning scaffolding for checkpoint cadence only and does not create a second operator contract.

### Planning pack scaffolding (required)

- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md` (this file)
  - Owns (authoritative):
    - the exact required-doc set for this feature directory
    - the surface-to-doc ownership map
    - the follow-ups required to remove ADR ambiguity before quality gate
    - the canonical slice IDs and canonical slice spec paths
  - Must define:
    - a surface-complete coverage matrix with exactly one owner per ADR-touched surface
    - the explicit statement that unselected doc classes stay unselected for ADR-0031
    - the Linux-only planning posture for this feature
  - Links (non-authoritative):
    - `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md`
  - Owns (authoritative):
    - the exact create/edit touch set for this feature
    - the cascading implications and cross-pack conflicts
    - the exact operator-doc reconciliation set
  - Must define:
    - the exact touched implementation paths for `BEDPM0`, `BEDPM1`, `BEDPM2`, and `BEDPM3`, including:
      - `scripts/substrate/install-substrate.sh`
      - one exact hermetic test path under `tests/installers/`
      - every operator-doc path that must change for the new flag, output line, and remediation flow
    - the explicit no-change boundary for:
      - `scripts/substrate/dev-install-substrate.sh`
      - runtime crates
      - world-deps behavior
    - the downstream contract dependency with `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
    - slice specs under `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/ci_checkpoint_plan.md`
  - Owns (authoritative):
    - the checkpoint grouping for the accepted slice order
    - the gate cadence that tasks/checkpoint wiring must mirror
  - Must define:
    - the single checkpoint boundary at the accepted last slice `BEDPM3`
    - the exact gate list that must run before the checkpoint is marked complete
    - the canonical relationship between checkpoint groups and `tasks.json` checkpoint metadata
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md`
  - Owns (authoritative):
    - the execution order for `BEDPM0`, `BEDPM1`, `BEDPM2`, and `BEDPM3`
    - the required validation commands and evidence expectations
  - Must define:
    - the orchestration branch `feat/best-effort-distro-package-manager`
    - the canonical locations for this pack’s pre-planning artifacts:
      - `pre-planning/spec_manifest.md`
      - `pre-planning/impact_map.md`
      - `pre-planning/ci_checkpoint_plan.md`
    - the exact slice order `BEDPM0` → `BEDPM1` → `BEDPM2` → `BEDPM3`
    - the exact validation commands for the hermetic installer harness and the feature-local Linux smoke script
    - the exact rule that this pack uses Linux-only validation artifacts; macOS and Windows remain explicit no-change platforms for ADR-0031
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json` (already exists)
  - Owns (authoritative):
    - the triad task graph and automation metadata for this pack
  - Must define:
    - the orchestration branch `feat/best-effort-distro-package-manager`
    - a Linux-only task model for this pack:
      - `meta.cross_platform` = `false`
      - `meta.behavior_platforms_required` MUST be `["linux"]`
      - `meta.checkpoint_boundaries` MUST match `pre-planning/ci_checkpoint_plan.md` and end at `BEDPM3`
      - any CI parity platform metadata MUST match `pre-planning/ci_checkpoint_plan.md`
    - triad task IDs and dependencies for:
      - `BEDPM0-code`, `BEDPM0-test`, `BEDPM0-integ`
      - `BEDPM1-code`, `BEDPM1-test`, `BEDPM1-integ`
      - `BEDPM2-code`, `BEDPM2-test`, `BEDPM2-integ`
      - `BEDPM3-code`, `BEDPM3-test`, `BEDPM3-integ`
    - references to the canonical slice spec paths under `slices/BEDPM*/`
    - acceptance-criteria traceability to `AC-BEDPM*` IDs
    - kickoff prompt paths for every task
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md`
    - slice specs under `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/session_log.md`
  - Owns (authoritative):
    - the append-only planning and execution log for this pack
  - Must define:
    - initialization from `docs/project_management/system/templates/planning_pack/session_log.md.tmpl`
    - the rule that every task start and end is recorded with timestamp and task id
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/quality_gate_report.md`
  - Owns (authoritative):
    - the planning quality-gate outcome for starting execution triads
  - Must define:
    - initialization from `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`
    - the rule that triads MUST NOT start unless the recommendation is `ACCEPT`
    - evidence that `make planning-lint FEATURE_DIR="docs/project_management/packs/draft/best-effort-distro-package-manager"` passed
  - Links (non-authoritative):
    - every required artifact referenced by the recommendation

### Feature contract + decisions (required)

- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - Owns (authoritative):
    - the Linux installer contract introduced or changed by ADR-0031
    - the operator-visible output, failure, and precedence rules for package-manager selection
    - the Linux-only path semantics and platform guarantees for this feature
  - Must define:
    - the exact installer entrypoint: `scripts/substrate/install-substrate.sh`
    - the exact new flag:
      - `--pkg-manager <apt-get|dnf|yum|pacman|zypper>`
    - the exact legacy override env var:
      - `PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>`
    - the exact selection precedence:
      - `--pkg-manager`
      - `PKG_MANAGER`
      - `/etc/os-release` mapping
      - deterministic `PATH` probe
    - the exact supported package-manager spellings and the exact `pkg_manager.source` vocabulary:
      - `flag`
      - `env`
      - `os_release`
      - `path_probe`
    - the exact `/etc/os-release` contract:
      - Linux-only read path
      - keys read: `ID`, `ID_LIKE`
      - safe parsing rule
      - exact `<unknown>` sentinel behavior when the file is missing or unreadable
      - exact rule that detection performs no network call
    - the exact distro-family mapping table for:
      - Debian/Ubuntu
      - Fedora/RHEL
      - Arch
      - SUSE
    - the exact fallback `PATH` probe order and the exact ambiguity-warning rule when more than one supported manager is present
    - the exact stderr decision line template:
      - `Detected distro: <id> (like: <id_like>), using package manager: <pkg_manager> (source: <flag|env|os_release|path_probe>)`
    - the exact exit-code mapping for all feature-relevant outcomes:
      - `0`
      - `2`
      - `3`
      - `4`
      - explicit taxonomy reference for `1` and `5`
    - the exact fail-closed rules for:
      - invalid `--pkg-manager`
      - invalid `PKG_MANAGER`
      - forced manager missing from `PATH`
      - no supported manager selected
    - the exact remediation-content requirements for unsupported or unavailable package-manager failures
    - the exact wrapper rule:
      - `scripts/substrate/install.sh` preserves upstream feature exit codes `0`, `2`, `3`, and `4`
    - the explicit no-change statements for:
      - no new config file
      - no persistent config key
      - no change to the prerequisite command set
      - no change to per-manager package-name mapping tables
      - no change to macOS behavior
      - no change to Windows behavior
    - the exact alternate os-release env-var contract:
      - `SUBSTRATE_INSTALL_OS_RELEASE_PATH`
      - precedence and path-validation rules
      - invalid-path absence semantics
      - Linux-only scope
  - Links (non-authoritative):
    - `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
  - Owns (authoritative):
    - the A/B decisions required to remove ambiguity from ADR-0031
  - Must define:
    - DR-0001 — `/etc/os-release` parser approach
      - option A and option B with one selected option
    - DR-0002 — multi-manager `PATH` probe ambiguity posture and fixed fallback order
      - option A: warn and pick one deterministic manager
      - option B: fail when more than one supported manager is present
      - one selected option
    - DR-0003 — hermetic testability hook for alternate os-release input
      - option A: no production-visible hook; use test-only harness mechanics
      - option B: expose one exact installer env var for alternate os-release input
      - one selected option
    - DR-0004 — wrapper exit-status posture for `scripts/substrate/install.sh`
      - option A: collapse direct-installer failures to wrapper exit `1`
      - option B: preserve upstream feature exit codes `0`, `2`, `3`, and `4`
      - one selected option
    - DR-0005 — feature-local smoke topology
      - option A: `smoke/linux-smoke.sh` is a thin wrapper over `tests/installers/pkg_manager_detection_smoke.sh`
      - option B: `smoke/linux-smoke.sh` owns an independent assertion suite
      - one selected option
  - Must define:
    - exactly two options (A/B) per decision
    - exactly one selected option per decision
    - the exact impacted contract surfaces for each selected option
    - the exact downstream docs that inherit the selected output vocabulary
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`

### Slice specs (required)

Slice specs MUST use the canonical layout:
- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/<SLICE_ID>/<SLICE_ID>-spec.md`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`
  - Owns (authoritative):
    - the `BEDPM0` slice scope and acceptance criteria for distro detection, mapping, and stable reporting
  - Must define:
    - the exact implementation boundary for reading `/etc/os-release`, deriving `distro_id` and `distro_id_like`, selecting a mapped manager, and rendering the decision line
    - the exact acceptance criteria that prove:
      - mapped Debian-family input selects `apt-get`
      - mapped Arch-family input selects `pacman`
      - Fedora/RHEL-family input prefers `dnf` and falls back to `yum` when `dnf` is unavailable
      - unreadable or missing `/etc/os-release` renders `<unknown>` fields and does not block fallback
      - the decision line appears exactly once before prerequisite installation begins
    - the contract-link rule: this slice spec links to `contract.md` and does not redefine the operator-facing contract
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`
  - Owns (authoritative):
    - the `BEDPM1` slice scope and acceptance criteria for override precedence, deterministic fallback, and actionable failure posture
  - Must define:
    - the exact implementation boundary for `--pkg-manager`, `PKG_MANAGER`, availability checks, fixed fallback order, and ambiguity warning behavior
    - the exact acceptance criteria that prove:
      - `--pkg-manager` wins over `PKG_MANAGER`
      - `PKG_MANAGER` wins over os-release mapping and `PATH` probing
      - invalid override values exit with `2`
      - forced manager missing from `PATH` exits with `3`
      - no supported manager selected exits with `4`
      - the multi-manager `PATH` branch follows the single order and warning posture selected by `decision_register.md`
    - the contract-link rule: this slice spec links to `contract.md` and does not redefine CLI, env-var, or exit-code tables
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md`
  - Owns (authoritative):
    - the `BEDPM2` slice scope and acceptance criteria for wrapper exit-status pass-through and operator/env-doc propagation
  - Must define:
    - the exact implementation boundary for `scripts/substrate/install.sh` exit-status pass-through and propagation of the selected precedence, warning, remediation, and alternate os-release contract into `docs/INSTALLATION.md` and `docs/reference/env/contract.md`
    - the exact acceptance criteria that prove:
      - `scripts/substrate/install.sh` preserves upstream feature-specific exit classes `0`, `2`, `3`, and `4`
      - `docs/INSTALLATION.md` reuses the selected override-precedence, warning, and remediation posture without drift
      - `docs/reference/env/contract.md` reuses the selected `PKG_MANAGER` and `SUBSTRATE_INSTALL_OS_RELEASE_PATH` contract without drift
    - the contract-link rule: this slice spec links to `contract.md` and `decision_register.md` and does not redefine the operator-facing contract
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM3/BEDPM3-spec.md`
  - Owns (authoritative):
    - the `BEDPM3` slice scope and acceptance criteria for hermetic validation, thin smoke-wrapper alignment, and evidence capture
  - Must define:
    - one exact hermetic test path under `tests/installers/`
    - the exact harness controls:
      - fake `PATH` with stub package-manager binaries
      - fake os-release input source
      - deterministic capture of selected manager and source
    - the exact acceptance criteria that prove:
      - precedence order `flag → env → os_release → path_probe`
      - emitted `pkg_manager.source` values match `contract.md`
      - the missing-os-release branch still reaches the fallback path
      - failure branches emit the expected exit-code class and required remediation elements
      - the wrapper path preserves the same feature-specific exit classes as the direct installer path
    - the exact rule that `smoke/linux-smoke.sh` remains a thin wrapper over the selected repo test and that any local container smoke remains non-gating and outside the required feature-local smoke contract
    - the contract-link rule: this slice spec links to `contract.md` and `decision_register.md` and does not redefine the operator-facing contract
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`

### Validation artifacts (required)

- `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`
  - Owns (authoritative):
    - the deterministic operator validation workflow for ADR-0031
  - Must define:
    - exact commands, expected key stderr text, and expected exit codes for:
      - default Debian-family selection
      - default Arch-family selection
      - forced override via `--pkg-manager`
      - legacy override via `PKG_MANAGER`
      - failure path with actionable remediation
    - the exact command that runs `smoke/linux-smoke.sh`
    - the explicit statement that macOS and Windows have no behavior delta under this ADR
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh`

- `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh`
  - Owns (authoritative):
    - the non-interactive Linux smoke validation steps for this feature
  - Must define:
    - the exact commands it runs
    - the exact pass/fail assertions for the Linux detection and override contract
    - the exact relation between the smoke script and the hermetic `tests/installers/` harness selected by `BEDPM3-spec.md`
    - the smoke script’s own exit-code contract
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`

## Coverage matrix (surface → authoritative doc)

Every surface touched by ADR-0031 must appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| Canonical feature directory and canonical slice IDs (`BEDPM0`, `BEDPM1`, `BEDPM2`, `BEDPM3`) | `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md` | exact directory path; exact slice ids; exact slice spec paths; explicit rejection of `C0/C1/C2` in planning artifacts |
| Installer entrypoint `scripts/substrate/install-substrate.sh` | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | script scope, Linux-only behavior delta, explicit no-change surfaces |
| CLI flag `--pkg-manager <apt-get|dnf|yum|pacman|zypper>` | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | allowed values, precedence, validation, availability checks, exit-code mapping |
| Legacy env override `PKG_MANAGER` | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | allowed values, precedence, validation, failure posture |
| Alternate os-release env var `SUBSTRATE_INSTALL_OS_RELEASE_PATH` | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact name, absolute-path validation, precedence against `/etc/os-release`, invalid-path semantics, Linux-only scope |
| `/etc/os-release` read semantics | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact path, keys read, safe parsing rule, missing-file behavior, `<unknown>` sentinel |
| Distro-family mapping table | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact families, exact match rules, exact preferred manager per family |
| Emitted selected-manager vocabulary | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact strings `apt-get|dnf|yum|pacman|zypper`; exact conditions that emit each |
| Emitted `pkg_manager.source` vocabulary | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact strings `flag|env|os_release|path_probe`; exact conditions that emit each |
| Fixed fallback `PATH` probe order | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact ordered probe list; selection rule when several supported managers exist |
| Multi-manager `PATH` ambiguity policy | `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` | warn-vs-fail selection, rationale, and exact downstream docs constrained by the selection |
| Wrapper exit-status preservation selection | `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` | wrapper pass-through vs collapse selection, rationale, and impacted downstream docs |
| Stderr decision line format | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact string template, exact stream, exact placement relative to prerequisite installation |
| Override-failure remediation content | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | required guidance elements for invalid override, missing manager binary, and no-supported-manager failure |
| Exit-code meanings for this feature (`0`, `2`, `3`, `4`) | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | taxonomy reference, per-code meaning, and explicit no override for unrelated taxonomy slots |
| `/etc/os-release` parser decision | `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` | exactly two parser options, one selection, and the contract sections constrained by the selection |
| Hermetic alternate-os-release input posture | `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` | exactly two options, one selection, and the exact rule for whether a production-visible env var exists |
| Feature-local smoke topology selection | `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` | thin-wrapper vs independent-smoke selection, rationale, and exact validation docs constrained by the selection |
| Explicit no-change surfaces | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | no new config file, no persistent config key, no detection network call, no macOS change, no Windows change, no package-map-table change |
| Exact implementation touch set and out-of-scope paths | `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md` | touched repo paths, untouched repo paths, operator-doc updates, downstream dependency boundaries |
| CI cadence checkpoints | `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/ci_checkpoint_plan.md` | checkpoint boundary at `BEDPM3`; gate list; tasks.json checkpoint alignment |
| `BEDPM0` acceptance criteria | `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md` | detection/mapping/reporting assertions and evidence commands |
| `BEDPM1` acceptance criteria | `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md` | precedence/failure assertions and evidence commands |
| `BEDPM2` acceptance criteria | `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md` | wrapper exit-status pass-through and operator/env-doc propagation assertions |
| Exact hermetic test harness path and `BEDPM3` validation assertions | `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM3/BEDPM3-spec.md` | exact `tests/installers/` path, stub inputs, assertions, thin-wrapper alignment, and non-gating local-container rule |
| Manual operator validation | `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md` | exact commands, expected stderr text, expected exit codes |
| Automated Linux smoke validation | `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh` | exact smoke commands, assertions, and script exit-code contract |
| Task graph and automation metadata | `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json` | exact task ids, dependencies, references, kickoff prompts, Linux-only task model |
| Slice order and validation evidence requirements | `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md` | exact sequence, exact validation commands, exact artifact expectations |

## Determinism checklist (per document; must be satisfied before quality gate)

- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - MUST define one exact precedence order and one exact emitted vocabulary for selected manager and source.
  - MUST define one exact `/etc/os-release` parsing and absence contract, including `<unknown>` behavior.
  - MUST define one exact fallback `PATH` probe order and one exact ambiguity-warning or ambiguity-failure contract.
  - MUST define one exact stderr decision line template and placement rule.
  - MUST define one exact exit-code mapping for invalid overrides, unavailable forced managers, and unsupported selection failures.
  - MUST define one exact Linux-only scope statement and one exact no-change statement for macOS and Windows.
  - MUST define `SUBSTRATE_INSTALL_OS_RELEASE_PATH` with exact name, precedence, path-validation, invalid-path semantics, and Linux-only scope.
  - MUST define the wrapper pass-through rule for `scripts/substrate/install.sh`.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
  - MUST include DR-0001, DR-0002, DR-0003, DR-0004, and DR-0005.
  - Each decision MUST contain exactly two options and exactly one selected option.
  - Each selected option MUST link to the contract or slice sections that implement it.
  - DR-0003 MUST state one exact result for hermetic fake os-release input: `SUBSTRATE_INSTALL_OS_RELEASE_PATH`.
  - DR-0004 MUST state one exact wrapper result for feature exit codes `0`, `2`, `3`, and `4`.
  - DR-0005 MUST state that `smoke/linux-smoke.sh` is either a thin wrapper or an independent contract and must pick exactly one result.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`
  - MUST scope the slice to detection, mapping, and reporting only.
  - MUST define runnable acceptance criteria that prove Debian-family, Arch-family, Fedora/RHEL-family, and missing-os-release branches.
  - MUST include at least one acceptance criterion that proves the decision line appears exactly once before package installation begins.
  - MUST link to `contract.md` instead of restating CLI or exit-code tables.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`
  - MUST scope the slice to override precedence, deterministic fallback, and failure posture.
  - MUST define runnable acceptance criteria that prove `--pkg-manager` overrides `PKG_MANAGER`, `PKG_MANAGER` overrides autodetection, invalid values exit with `2`, missing forced manager exits with `3`, and no-supported-manager exits with `4`.
  - MUST include one acceptance criterion for the selected multi-manager `PATH` posture from `decision_register.md`.
  - MUST link to `contract.md` instead of restating CLI or exit-code tables.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md`
  - MUST scope the slice to wrapper exit-status pass-through and operator/env-doc propagation only.
  - MUST define runnable acceptance criteria that prove `scripts/substrate/install.sh` preserves upstream `0`, `2`, `3`, and `4` outcomes for this feature path.
  - MUST require `docs/INSTALLATION.md` and `docs/reference/env/contract.md` to reuse the selected precedence, warning, remediation, and alternate os-release contract without drift.
  - MUST link to `contract.md` instead of restating CLI, env-var, or exit-code tables.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM3/BEDPM3-spec.md`
  - MUST define one exact hermetic test harness path under `tests/installers/`.
  - MUST define one exact fake-input mechanism for os-release content and one exact fake-input mechanism for supported-manager binaries.
  - MUST require assertions for precedence, source vocabulary, missing-os-release fallback, wrapper pass-through, and required remediation content.
  - MUST state that `smoke/linux-smoke.sh` remains a thin wrapper over the selected repo test and that any local container smoke stays outside the required feature-local smoke contract.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`
  - MUST include exact commands and expected key stderr text for default Debian-family, default Arch-family, forced override, legacy env override, and no-supported-manager failure.
  - MUST reference `smoke/linux-smoke.sh`.
  - MUST state that macOS and Windows are explicit no-change platforms for ADR-0031.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh`
  - MUST define exact non-interactive Linux smoke steps and exact assertions.
  - MUST define its own pass/fail exit-code contract.
  - MUST reference the hermetic `tests/installers/` harness selected by `BEDPM3-spec.md` or explicitly inline the same assertions without drift.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md`
  - MUST enumerate the exact implementation touch set, the exact operator-doc touch set, and the explicit no-change boundary for dev-install and runtime crates.
  - MUST call out the downstream dependency on `persist-detected-linux-distro-pkg-manager`.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/plan.md`
  - MUST define the exact slice order `BEDPM0` → `BEDPM1` → `BEDPM2` → `BEDPM3`.
  - MUST define the exact validation commands and the exact evidence expected from the manual playbook and Linux smoke script.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/ci_checkpoint_plan.md`
  - MUST define the single checkpoint boundary at `BEDPM3`.
  - MUST define the exact gates and their pass criteria.
  - MUST define the exact `tasks.json` checkpoint metadata that must match it.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json`
  - MUST define the full triad task graph for `BEDPM0`, `BEDPM1`, `BEDPM2`, and `BEDPM3`.
  - MUST align to the Linux-only validation posture selected by this manifest.
  - MUST ensure each task’s acceptance criteria point at the relevant slice-spec `AC-BEDPM*` ids and do not duplicate conflicting contract text.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/session_log.md`
  - MUST define the append-only START/END task logging rule with timestamp and task id.

- `docs/project_management/packs/draft/best-effort-distro-package-manager/quality_gate_report.md`
  - MUST record the planning-lint evidence and one exact gate recommendation.
  - MUST block execution triads until the recommendation is `ACCEPT`.

## Follow-ups

- ADR path reconciliation: `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md` still points related docs at `docs/project_management/packs/draft/detecting-badger/`. Planning artifacts for this feature MUST use `docs/project_management/packs/draft/best-effort-distro-package-manager/`.
- Slice ID reconciliation: ADR-0031 uses generic `C0/C1/C2`. Planning artifacts for this feature MUST use the accepted slice order `BEDPM0/BEDPM1/BEDPM2/BEDPM3` and MUST include the ADR-to-slice mapping.
- CI checkpoint-plan drift: `pre-planning/ci_checkpoint_plan.md` currently predates the accepted `BEDPM3` end-of-checkpoint boundary. Planning MUST update that file and matching `tasks.json` checkpoint metadata before quality gate.
- Tasks metadata drift: `docs/project_management/packs/draft/best-effort-distro-package-manager/tasks.json` currently declares `meta.cross_platform=true` with Linux/macOS/Windows platform arrays. That conflicts with ADR-0031 lift data (`cross_platform=false`) and this manifest’s Linux-only doc set. Planning MUST reconcile `tasks.json` to the Linux-only posture before quality gate.
