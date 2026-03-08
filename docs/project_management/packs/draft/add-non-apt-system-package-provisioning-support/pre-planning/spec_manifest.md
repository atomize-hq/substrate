# add-non-apt-system-package-provisioning-support — spec manifest (pre-planning)

This file enumerates every contract, schema, validation, and decision surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`

External authoritative inputs (this pack MUST NOT redefine these surfaces):
- Exit code taxonomy:
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Agent API execute protocol baseline and request `profile` field baseline:
  - `docs/WORLD.md`
- Environment-variable registry baseline:
  - `docs/CONFIGURATION.md`
- World-deps inventory/enabled merge order, bundle expansion, and non-system-package package branches:
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
- APT-specific provisioning decisions that ADR-0033 reuses unless this pack explicitly supersedes them:
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md`

Existing docs that this pack MUST reconcile and MUST NOT leave as competing authorities for the same surface:
- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`
- `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
- `docs/internals/world/deps.md`
- `docs/reference/world/deps/README.md`
- `docs/COMMANDS.md`

## Slice IDs (canonical)

ADR-0033 uses placeholder slice IDs (`C0`, `C1`, `C2`). This feature MUST use feature-derived slice IDs per:
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`

Canonical slice IDs selected for this feature:
- Slice prefix: `NASPP` (derived from `add-non-apt-system-package-provisioning-support`)
- `NASPP0` — world OS package-manager probe
- `NASPP1` — pacman provisioning path for world-deps system packages
- `NASPP2` — runtime fail-early extension, validation, and operator-doc reconciliation

Accepted full-planning slice order:
- `NASPP0` → `NASPP1` → `NASPP2`

## Required spec documents (authoritative)

This ADR requires one local consolidated contract doc, one local schema spec, one local decision register, one impact map, one feature-local CI checkpoint plan, one execution plan, one manual validation playbook, three platform smoke scripts, one session log, one quality gate report, and three canonical slice specs.

No separate protocol, env-vars, policy, telemetry, filesystem-semantics, platform-parity, or standalone compatibility doc is selected.
- This feature reuses the existing Agent API `/v1/execute` request shape and existing `profile` field; it does not add a new endpoint, message shape, or request field.
- This feature introduces no new `SUBSTRATE_*`, `SHIM_*`, or `WORLD_*` environment variable; the existing `SUBSTRATE_WORLD_REQUEST_PROFILE` baseline remains external.
- This feature introduces no policy broker schema change, approval-cache change, or enforcement-mode change.
- This feature introduces no new structured log field, trace-span field, or redaction rule.
- This feature introduces no new overlay, path-rewrite, or filesystem-diff contract; `contract.md` owns only the guest-only mutation and no-host-mutation invariants for this feature.
- This feature-local contract plus the smoke artifacts are sufficient for platform/backends divergence because every divergence in ADR-0033 is confined to CLI behavior, exit codes, remediation text, and validation evidence.
- Schema compatibility for `install.method=pacman` is part of `world-deps-system-package-schema-spec.md`; no separate compatibility doc is required.

### Planning pack scaffolding (required)

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md` (this file)
  - Owns (authoritative):
    - the exact required-doc set for this feature directory
    - the surface-to-doc ownership map
    - the canonical slice IDs and canonical slice-spec paths
    - the explicit statement that unselected doc classes stay unselected for ADR-0033
    - the follow-ups required before quality gate
  - Must define:
    - a surface-complete coverage matrix with exactly one owner per ADR-touched surface
    - the exact ADR-to-slice mapping from `C0/C1/C2` to `NASPP0/NASPP1/NASPP2`
    - the explicit rule that this pack owns the new pacman-specific and manager-aware system-package surfaces
  - Links (non-authoritative):
    - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md`
  - Owns (authoritative):
    - the exact create/edit touch set for this feature
    - the exact cross-pack and operator-doc reconciliation set
    - the cascading implications and cross-queue conflicts
  - Must define:
    - the exact touched implementation paths for `NASPP0`, `NASPP1`, and `NASPP2`, including:
      - `crates/shell/src/builtins/world_deps/inventory.rs`
      - `crates/shell/src/builtins/world_deps/surfaces.rs`
      - `crates/shell/src/builtins/world_enable/`
      - `crates/world-agent/src/service.rs`
    - the exact test paths that must change for schema validation, pacman command construction, world-agent request-profile usage, and runtime fail-early behavior
    - the exact doc paths and headings that must be updated so the final state has one authority for system-package provisioning and runtime remediation
    - the exact reconciliation rule for:
      - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
      - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
      - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`
      - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-system-package-schema-spec.md`
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`
    - slice specs under `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/`

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/ci_checkpoint_plan.md`
  - Owns (authoritative):
    - the checkpoint grouping for the accepted slice order
    - the gate cadence that `tasks.json` checkpoint wiring must mirror
  - Must define:
    - checkpoint boundary `CP1` ending at `NASPP1`
    - checkpoint boundary `CP2` ending at `NASPP2`
    - the exact cross-platform gate list required at each checkpoint
    - the exact relationship between checkpoint groups and `tasks.json` `meta.checkpoint_boundaries`
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json`

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/plan.md`
  - Owns (authoritative):
    - the execution order for `NASPP0`, `NASPP1`, and `NASPP2`
    - the required validation commands and evidence expectations
  - Must define:
    - the orchestration branch `feat/add-non-apt-system-package-provisioning-support`
    - the canonical locations for this pack’s pre-planning artifacts:
      - `pre-planning/spec_manifest.md`
      - `pre-planning/impact_map.md`
      - `pre-planning/ci_checkpoint_plan.md`
    - the exact slice order `NASPP0` → `NASPP1` → `NASPP2`
    - the exact rule that this pack is cross-platform at the contract level and requires Linux, macOS, and Windows validation artifacts
    - the exact validation command set for:
      - targeted Rust tests for schema validation and provisioning/runtime behavior
      - `make planning-micro-lint`
      - `smoke/linux-smoke.sh`
      - `smoke/macos-smoke.sh`
      - `smoke/windows-smoke.ps1`
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json`
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-system-package-schema-spec.md`
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json` (already exists)
  - Owns (authoritative):
    - the triad task graph and automation metadata for this pack
  - Must define:
    - `meta.cross_platform = true`
    - `meta.behavior_platforms_required = ["linux", "macos", "windows"]`
    - `meta.ci_parity_platforms_required = ["linux", "macos", "windows"]`
    - `meta.checkpoint_boundaries = ["NASPP1", "NASPP2"]`
    - the orchestration branch `feat/add-non-apt-system-package-provisioning-support`
    - triad task IDs and dependencies for:
      - `NASPP0-code`, `NASPP0-test`, `NASPP0-integ`
      - `NASPP1-code`, `NASPP1-test`, `NASPP1-integ`
      - `NASPP2-code`, `NASPP2-test`, `NASPP2-integ`
    - CI checkpoint task wiring for `CP1-ci-checkpoint` and `CP2-ci-checkpoint`
    - references to the canonical slice-spec paths under `slices/NASPP*/`
    - acceptance-criteria traceability to `AC-NASPP*` IDs
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/plan.md`
    - slice specs under `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/`

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/session_log.md`
  - Owns (authoritative):
    - the append-only planning and execution log for this pack
  - Must define:
    - initialization from `docs/project_management/system/templates/planning_pack/session_log.md.tmpl`
    - the rule that every task start and end is recorded with timestamp and task id
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json`

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/quality_gate_report.md`
  - Owns (authoritative):
    - the planning quality-gate outcome required before execution triads begin
  - Must define:
    - initialization from `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`
    - the rule that triads MUST NOT start unless the recommendation is `ACCEPT`
    - evidence that planning lint and the required micro-lint checks passed
  - Links (non-authoritative):
    - every required artifact referenced by the recommendation

### Feature contract, schema, and decisions (required)

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
  - Owns (authoritative):
    - the operator-facing system-package provisioning contract after ADR-0033
    - the operator-facing runtime fail-early contract for `install.method=apt|pacman`
    - the platform/backend guarantees, exit codes, and remediation invariants for this feature
  - Must define:
    - the exact commands in scope:
      - `substrate world enable --provision-deps [--dry-run] [--verbose]`
      - `substrate world deps current sync [--dry-run] [--verbose] [--all]`
      - `substrate world deps current install <ITEM...> [--dry-run] [--verbose]`
    - the exact in-scope rule for runtime commands when system-package items are present
    - the exact operator-visible meaning of “system-package-backed item” for this feature:
      - `install.method=apt`
      - `install.method=pacman`
    - the exact provisioning mismatch posture when the enabled system-package methods do not match the detected world OS package manager
    - the exact `--dry-run` and `--verbose` behavior for provisioning and runtime fail-early branches
    - the exact remediation-content requirements, including the exact command string:
      - `substrate world enable --provision-deps`
    - the exact exit-code mapping for:
      - `0`
      - `2`
      - `3`
      - `4`
      - `5`
      - explicit taxonomy reference for `1`
    - the exact platform/backend matrix for:
      - Linux host-native
      - macOS Lima guest backend
      - Windows WSL backend
    - the exact no-host-mutation and guest-only-mutation invariants
    - the exact relationship to the existing provisioning request profile:
      - provisioning reuses `world-deps-provision`
      - operators MUST NOT need to set `SUBSTRATE_WORLD_REQUEST_PROFILE`
      - provisioning MUST NOT invent a second profile value
    - the exact statement that this feature introduces no new config key and no new environment variable
  - Links (non-authoritative):
    - `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-system-package-schema-spec.md`
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-system-package-schema-spec.md`
  - Owns (authoritative):
    - the system-package branch of the world-deps package schema after ADR-0033
    - schema compatibility and validation rules for `install.method=pacman`
  - Must define:
    - the exact allowed system-package `install.method` values owned by this feature:
      - `apt`
      - `pacman`
    - the exact `install.apt[]` entry shape that remains valid for system-package items
    - the exact `install.pacman[]` shape:
      - ordered list
      - non-empty string elements
      - version pinning is not supported in v1
    - the exact field presence and absence rules:
      - `install.pacman` is required iff `install.method=pacman`
      - `install.apt` is required iff `install.method=apt`
      - `install.pacman` is forbidden when `install.method` is not `pacman`
      - `install.apt` is forbidden when `install.method` is not `apt`
    - the exact validation and canonicalization rules for:
      - duplicate pacman package names
      - empty strings
      - unsupported extra fields inside system-package branches
    - the exact schema compatibility rule:
      - `install.method=pacman` is an additive extension
      - older tooling without pacman support fails with exit `2`
      - this feature does not change bundle schema, inventory directory layout, wrapper schema, script schema, or manual schema
    - exact YAML examples for:
      - valid `apt`
      - valid `pacman`
      - invalid `pacman`
  - Links (non-authoritative):
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`
  - Owns (authoritative):
    - the A/B decisions required to remove ADR-0033 ambiguity
  - Must define:
    - DR-0001 — schema approach
      - option A: explicit `install.method=pacman`
      - option B: abstract `install.method=system_packages`
      - one selected option
    - DR-0002 — world OS package-manager probe strategy
      - option A and option B with one selected option
    - DR-0003 — pacman invocation and idempotency strategy
      - option A and option B with one selected option
    - DR-0004 — mismatch policy when enabled system-package methods do not match the detected world OS manager
      - option A and option B with one selected option
  - Must define:
    - exactly two options (`A` and `B`) per decision
    - exactly one selected option per decision
    - the exact downstream docs and surfaces constrained by each selected option
    - the exact relationship to the reused APT-specific decisions in:
      - `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md`
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
    - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-system-package-schema-spec.md`
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md`

### Validation artifacts (required)

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`
  - Owns (authoritative):
    - deterministic manual validation setup, commands, expected key output, and expected exit codes
  - Must define:
    - exact setup for an Arch-family world image or guest environment
    - exact setup for a Debian/Ubuntu-family world image or guest environment
    - manual validation for:
      - pacman provisioning success on an Arch-family world
      - mismatch failure when enabled `apt|pacman` methods do not match the detected world OS manager
      - runtime fail-early behavior for `substrate world deps current sync|install`
      - Linux host-native no-host-mutation posture
      - Windows posture selected by `contract.md`

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh`
  - Owns (authoritative):
    - automated Linux validation for the contract selected by `contract.md`
  - Must define:
    - the exact assertions for Linux host-native unsupported or supported behavior
    - the exact exit-code and remediation-text assertions

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh`
  - Owns (authoritative):
    - automated macOS validation for manager-aware provisioning and runtime fail-early behavior
  - Must define:
    - the exact assertions for Arch-family pacman provisioning
    - the exact assertions for mismatch failure and runtime remediation

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1`
  - Owns (authoritative):
    - automated Windows validation for the Windows posture selected by `contract.md`
  - Must define:
    - the exact assertions for Windows unsupported behavior or Windows WSL provisioning behavior
    - the exact exit-code and remediation-text assertions

### Slice specs (required)

Slice specs MUST use the canonical layout:
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/<SLICE_ID>/<SLICE_ID>-spec.md`

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASPP0/NASPP0-spec.md`
  - Owns (authoritative):
    - the `NASPP0` slice scope and acceptance criteria for world OS package-manager probing
  - Must define:
    - the exact in-world probe inputs:
      - `/etc/os-release`
      - `ID`
      - `ID_LIKE`
      - manager presence checks
    - the exact rule that the probe MUST run inside the world and MUST NOT consult the host `PATH`
    - the exact precedence rule when `/etc/os-release` evidence and manager presence checks disagree
    - the exact derived-manager vocabulary emitted to downstream provisioning logic
    - the exact unsupported or unknown-manager outcome
    - the exact acceptance criteria that prove:
      - Arch-family worlds derive `pacman`
      - Debian/Ubuntu-family worlds derive `apt`
      - unknown-manager worlds fail closed
    - the contract-link rule: this slice spec links to `contract.md` and does not redefine operator-facing exit-code tables

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASPP1/NASPP1-spec.md`
  - Owns (authoritative):
    - the `NASPP1` slice scope and acceptance criteria for pacman provisioning
  - Must define:
    - the exact pacman-backed item selection rule from the effective enabled world-deps set
    - the exact pacman requirement normalization rule:
      - ordering
      - de-duplication
      - empty-list handling
    - the exact pacman no-op detection or presence-probe rule selected by `decision_register.md`
    - the exact pacman command construction:
      - command name
      - flags
      - argument ordering
      - non-interactive posture
    - the exact request-profile usage:
      - `world-deps-provision`
      - ignore `SUBSTRATE_WORLD_REQUEST_PROFILE`
    - the exact backend guard rails and exit-code mapping for unsupported provisioning attempts
    - the exact partial-failure rule:
      - no partial provision when mismatch or validation failure occurs
    - the contract-link rule: this slice spec links to `contract.md`, `world-deps-system-package-schema-spec.md`, and `decision_register.md`

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASPP2/NASPP2-spec.md`
  - Owns (authoritative):
    - the `NASPP2` slice scope and acceptance criteria for runtime fail-early extension, validation, and doc reconciliation
  - Must define:
    - the exact runtime fail-early behavior for `install.method=pacman`
    - the exact rule for mixed in-scope `apt|pacman` items during runtime `sync`, `sync --all`, and `install <ITEM...>`
    - the exact remediation output content and stream placement for runtime failures
    - the exact doc reconciliation targets by path and heading for:
      - operator docs
      - internal docs
      - existing ADR-0030 planning-pack docs that would otherwise compete
    - the exact validation evidence that must be captured before the slice is complete
    - the contract-link rule: this slice spec links to `contract.md` and `world-deps-system-package-schema-spec.md` and does not restate schema tables

## Coverage matrix (surface → authoritative doc)

Every surface touched by ADR-0033 must appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| Canonical feature directory and canonical slice IDs (`NASPP0`, `NASPP1`, `NASPP2`) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md` | exact directory path; exact slice ids; exact ADR-to-slice mapping; exact slice-spec paths |
| Checkpoint cadence (`CP1` ends at `NASPP1`; `CP2` ends at `NASPP2`) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/ci_checkpoint_plan.md` | exact checkpoint boundaries; exact gates; exact relationship to `tasks.json` |
| Consolidated operator-facing system-package provisioning contract after ADR-0033 | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | exact statement that this doc owns the post-ADR-0033 provisioning and runtime system-package contract; exact defer or reconciliation rule for older docs |
| CLI provisioning entrypoint `substrate world enable --provision-deps [--dry-run] [--verbose]` | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | flags; no-op semantics; supported and unsupported outcomes; remediation invariants; exact exit codes |
| Runtime fail-early contract for `substrate world deps current sync`, `sync --all`, and `install <ITEM...>` when system-package items are in scope | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | in-scope rules; fail-early rules; no-op rules; exact remediation command; exact streams and exit codes |
| Exit-code meanings (`0`, `2`, `3`, `4`, `5`) for provisioning and runtime system-package branches | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | exact mapping to taxonomy; exact command-level meaning; exact reserved meaning for `5` |
| Platform/backend support matrix (Linux host-native, macOS Lima, Windows WSL) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | exact supported and unsupported posture; exact remediation requirements; exact no-host-mutation statements |
| Guest-only mutation and no-host-mutation invariants | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | exact guard rails; exact protected-path posture; exact statement that runtime never mutates system packages |
| Existing environment variable `SUBSTRATE_WORLD_REQUEST_PROFILE` | `docs/CONFIGURATION.md` | name; default; baseline meaning; exact statement that it is an advanced/testing variable |
| Existing Agent API `/v1/execute` request `profile` field baseline | `docs/WORLD.md` | request shape; transport notes; exact baseline existence of the `profile` field |
| World-deps inventory merge order, enabled resolution, bundle expansion, and non-system-package methods (`script`, `manual`) | `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` | inventory directories; merge order; enabled resolution; bundle expansion; unchanged non-system-package branches |
| System-package package schema branch (`install.method=apt|pacman`, `install.apt[]`, `install.pacman[]`) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-system-package-schema-spec.md` | exact field names; types; presence rules; validation rules; canonicalization |
| Schema compatibility and forward or backward handling for `install.method=pacman` | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-system-package-schema-spec.md` | exact additive-extension rule; exact old-tool failure posture; exact no-migration statement |
| APT-specific requirement normalization, probe-only satisfied detection, and reuse of `world-deps-provision` | `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md` | exact reused APT decisions that ADR-0033 inherits unless explicitly superseded |
| World OS package-manager probe inputs (`/etc/os-release`, `ID`, `ID_LIKE`, manager presence checks) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASPP0/NASPP0-spec.md` | exact inputs; exact in-world execution requirement; exact absence semantics |
| Probe precedence and derived-manager vocabulary | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASPP0/NASPP0-spec.md` | exact precedence when signals disagree; exact emitted vocabulary; exact unsupported result |
| Pacman requirement normalization, no-op detection, and command construction | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASPP1/NASPP1-spec.md` | exact ordering; de-dup; presence-probe or no-op rules; exact pacman command and flags |
| Pacman invocation strategy and idempotency choice | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md` | exact selected option; exact downstream docs constrained by the selection |
| Mixed-method mismatch policy when enabled system-package methods do not match the detected world OS manager | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md` | exact fail or partial-provision choice; exact downstream docs constrained by the selection |
| Mismatch remediation output and operator guidance | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | exact text requirements; exact command string; exact stream placement; exact exit-code mapping |
| Exact cross-pack and doc reconciliation targets | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md` | exact file paths and headings; exact rule for removing competing authorities |
| Triad task graph, automation metadata, and checkpoint wiring | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json` | exact task ids; dependencies; checkpoint tasks; canonical spec references; `meta.checkpoint_boundaries` |
| Manual validation | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md` | exact setup; exact commands; exact expected output; exact exit codes |
| Smoke validation | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/*` | exact automated assertions per platform; exact pass or fail outcomes |
| New wire or IPC protocol surfaces | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md` | this feature MUST NOT add a new endpoint, new request field, new response field, or new streaming frame |
| New environment variables | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md` | this feature MUST NOT introduce a new `SUBSTRATE_*`, `SHIM_*`, or `WORLD_*` variable |
| Policy surface | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md` | this feature MUST NOT add or modify policy broker schemas, approval-cache behavior, or enforcement mode |
| Telemetry and structured-log fields | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md` | this feature MUST NOT add or modify trace-span fields, structured log fields, or redaction behavior |
| Filesystem-diff, overlay, or path-rewrite semantics | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md` | this feature MUST NOT introduce a new filesystem diff contract, overlay contract, or path-rewrite rule |

## Determinism checklist (must be satisfied before quality gate)

For every selected spec document, confirm it explicitly defines:
- inputs and precedence order when multiple inputs exist
- defaults and absence semantics
- exact data-model constraints for every serialized or parsed boundary
- exact error model, exit codes, and fail-closed posture
- exact ordering, idempotency, and no-partial-apply rules
- exact security and safety invariants
- exact platform/backend guarantees
- exact reconciliation targets when an older doc already describes the same surface

## Follow-ups

1. Competing provisioning-contract ownership must be collapsed to one authority
   - Issue: `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` already describes `substrate world enable --provision-deps` and runtime system-package posture for `apt`. ADR-0033 extends the same CLI surface with manager-aware behavior.
   - Required fix: `pre-planning/impact_map.md` and `contract.md` MUST define one exact reconciliation rule:
     - either this pack’s `contract.md` becomes the single authority and the ADR-0030 contract becomes a defer-only doc, or
     - the ADR-0030 contract remains the single authority and this pack’s docs defer to it.
   - The final state MUST leave exactly one authoritative contract doc for the shared CLI surface.

2. System-package schema ownership must be collapsed to one authority
   - Issue: `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` currently owns the package schema and lists `install.method=apt|script|manual`, while ADR-0033 adds `install.method=pacman`.
   - Required fix: `world-deps-system-package-schema-spec.md` and `pre-planning/impact_map.md` MUST define one exact end-state ownership rule so system-package schema text exists in exactly one authoritative doc set.

3. Windows posture is still ambiguous in ADR-0033
   - Issue: ADR-0033 says Windows WSL provisioning is an assumption, not a final contract.
   - Required fix: `contract.md`, `manual_testing_playbook.md`, and `smoke/windows-smoke.ps1` MUST choose exactly one Windows posture:
     - unsupported with exit `4`, or
     - supported inside WSL with explicit guard rails.

4. Probe disagreement semantics are not fully specified in ADR-0033
   - Issue: ADR-0033 requires using both `/etc/os-release` and manager presence checks, but it does not define the exact precedence rule when they disagree.
   - Required fix: `decision_register.md` and `slices/NASPP0/NASPP0-spec.md` MUST define:
     - the exact precedence rule
     - the exact derived-manager vocabulary
     - the exact fail-closed outcome for unknown or conflicting signals

5. `install.pacman[]` canonicalization is underspecified
   - Issue: ADR-0033 states that `install.pacman` is an ordered list of package names, but it does not define duplicate handling, empty-string rejection, or normalization.
   - Required fix: `world-deps-system-package-schema-spec.md` and `slices/NASPP1/NASPP1-spec.md` MUST define:
     - duplicate-package handling
     - empty-string rejection
     - stable ordering rules after normalization

6. Mixed enabled `apt` and `pacman` requirement sets need one exact outcome
   - Issue: ADR-0033 requires failure when enabled system-package methods do not match the detected world OS package manager, but it does not define the exact dry-run output and non-dry-run output for mixed method sets.
   - Required fix: `decision_register.md` and `contract.md` MUST define one exact rule for:
     - `--dry-run`
     - `--verbose`
     - non-dry-run provisioning
     - runtime `sync`
     - runtime `install <ITEM...>`

7. Reconciliation targets must be enumerated by exact path and heading
   - Issue: ADR-0033 requires manager-aware operator guidance, but it does not list the exact headings that must stop describing runtime APT-only behavior or APT-only remediation.
   - Required fix: `pre-planning/impact_map.md` and `slices/NASPP2/NASPP2-spec.md` MUST enumerate the exact files and headings to update, including the ADR-0030 planning-pack docs, operator docs, and internal docs.
