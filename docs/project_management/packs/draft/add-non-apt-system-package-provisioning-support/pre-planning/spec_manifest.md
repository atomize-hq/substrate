# add-non-apt-system-package-provisioning-support — spec manifest (pre-planning)

This file enumerates every contract, schema, platform, validation, and no-change surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
- External authoritative inputs that inform this pack but are not owned by it:
  - `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
    - authoritative for the baseline provisioning-time APT posture and the runtime “no OS package manager mutation” direction that ADR-0033 extends
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
    - authoritative for the existing world-deps inventory/enabled model, including `install.method=apt`, `install.apt`, and effective enabled-set resolution
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - authoritative default exit-code meanings
  - `docs/WORLD.md`
    - authoritative for existing `/v1/execute` and `/v1/stream` protocol baselines and request `profile` field existence
  - `docs/CONFIGURATION.md`
    - authoritative for the existing `SUBSTRATE_WORLD_REQUEST_PROFILE` environment-variable registry entry
  - `docs/project_management/intake/adrs/routing_weasel_adr_intake.md`
    - supporting discovery reference only

## Slice IDs (canonical)

ADR-0033 uses placeholder slice IDs (`C0`, `C1`, `C2`). This feature MUST use feature-derived slice IDs per:
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`

Canonical slice IDs selected for this feature:
- Slice prefix: `NASP` (derived from `non-apt-system-package-provisioning`)
- `NASP0` — world OS package-manager probe and provisioning support gate
- `NASP1` — pacman schema extension and inventory-view updates
- `NASP2` — provisioning routing, request-profile use, and pacman command execution
- `NASP3` — runtime fail-early behavior, explicit-item scoping, and remediation wording
- `NASP4` — contract reconciliation, platform parity, and manual/smoke validation evidence

## Required spec documents (authoritative)

This ADR requires one user-facing contract doc, one schema spec, one platform-parity spec, one decision register, one impact map, one CI checkpoint plan, one execution plan, one manual validation playbook, three platform smoke scripts, and five canonical slice specs.

No separate protocol, env-vars, telemetry, filesystem-semantics, or compatibility doc is selected.
- ADR-0033 reuses the existing world-agent execute/stream protocol and existing request `profile` field; it does not require a new endpoint or request/response field.
- ADR-0033 introduces no new `SUBSTRATE_*`, `SHIM_*`, or `WORLD_*` environment variable.
- ADR-0033 does not require new structured log fields or trace span fields.
- Filesystem/path invariants remain limited to `/etc/os-release` probe behavior, “no host OS mutation,” and provisioning mutation boundaries inside the world OS; those surfaces are owned by `contract.md`, `platform-parity-spec.md`, and the slice specs.
- ADR-0033 does not require a migration, deprecation window, or staged compatibility policy.

### Planning pack scaffolding (required)

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md`
  - Owns (authoritative):
    - the exact required-doc set for this feature directory
    - the surface-to-doc ownership map
    - the canonical slice IDs and canonical slice spec paths
    - the explicit no-new-protocol, no-new-env-var, and no-new-telemetry boundaries for this ADR
  - Must define:
    - a complete coverage matrix with exactly one owner per surface
    - the selected and unselected doc classes for ADR-0033
    - any follow-ups required to remove ADR ambiguity before quality gate

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md`
  - Owns (authoritative):
    - the exact create/edit touch set for this feature
    - the cascading implications and cross-pack conflicts
    - the exact operator-doc reconciliation set
  - Must define:
    - every implementation path expected to change across shell, world enable, world deps, tests, and docs
    - the exact contract-reconciliation edits required for ADR-0030 and the implemented world-deps contract docs
    - the exact validation/doc paths that must change for manager-aware remediation

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/ci_checkpoint_plan.md`
  - Owns (authoritative):
    - checkpoint grouping for the accepted slice order `NASP0`, `NASP1`, `NASP2`, `NASP3`, and `NASP4`
    - the CI gate cadence that `tasks.json` must mirror
  - Must define:
    - the accepted checkpoint boundary or boundaries for this pack
    - the exact gate list required before a checkpoint can close
    - the mapping from checkpoint groups to `tasks.json` metadata

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/plan.md`
  - Owns (authoritative):
    - the execution order for `NASP0`, `NASP1`, `NASP2`, `NASP3`, and `NASP4`
    - the required validation commands and evidence expectations
  - Must define:
    - the orchestration branch `feat/add-non-apt-system-package-provisioning-support`
    - the exact slice order `NASP0` → `NASP1` → `NASP2` → `NASP3` → `NASP4`
    - the dependency boundary with ADR-0030
    - the exact validation commands for shell tests, manual validation, and smoke scripts

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json` (already exists)
  - Owns (authoritative):
    - the triad task graph and automation metadata for this pack
  - Must define:
    - task IDs and dependencies for:
      - `NASP0-code`, `NASP0-test`, `NASP0-integ`
      - `NASP1-code`, `NASP1-test`, `NASP1-integ`
      - `NASP2-code`, `NASP2-test`, `NASP2-integ`
      - `NASP3-code`, `NASP3-test`, `NASP3-integ`
      - `NASP4-code`, `NASP4-test`, `NASP4-integ`
    - references to the canonical slice spec paths under `slices/NASP*/`
    - acceptance-criteria traceability to `AC-NASP*` IDs
    - behavior-platform and CI-parity metadata consistent with `linux`, `macos`, and `windows`
    - checkpoint metadata consistent with `pre-planning/ci_checkpoint_plan.md`

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/session_log.md`
  - Owns (authoritative):
    - the append-only planning and execution log for this pack
  - Must define:
    - initialization from the planning-pack session-log template
    - the rule that every task start and end is recorded with timestamp and task ID

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/quality_gate_report.md`
  - Owns (authoritative):
    - the planning quality-gate outcome for starting execution triads
  - Must define:
    - initialization from the planning gate template
    - the rule that triads MUST NOT start unless the recommendation is `ACCEPT`
    - evidence that the required planning validation passed

### Feature contract, schema, parity, and decisions (required)

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
  - Owns (authoritative):
    - the operator-facing contract introduced or changed by ADR-0033
    - the shared `--provision-deps` CLI behavior as extended to non-APT system packages
    - runtime remediation, exit-code meaning, and host-no-mutation guarantees
  - Must define:
    - the exact provisioning entrypoint:
      - `substrate world enable --provision-deps [--dry-run] [--verbose]`
    - the exact runtime commands whose behavior is constrained:
      - `substrate world deps current sync`
      - `substrate world deps current install`
    - the exact success/no-op/error semantics for:
      - supported Arch-family provisioning
      - unsupported backend posture
      - unsupported world OS / manager mismatch posture
      - runtime fail-early posture for system-package items
    - the exact exit-code mapping for `0`, `2`, `3`, `4`, and `5`
    - the exact “no host OS mutation” guarantee and the exact relationship to guest-world mutation
    - the exact `--dry-run` and `--verbose` guarantees, including minimum required output elements
    - the exact remediation message invariants, including the required command string `substrate world enable --provision-deps`
    - the exact rule for how runtime scope is evaluated when `current install` is invoked with explicit item arguments
    - the exact relationship between the operator-facing workflow and `SUBSTRATE_WORLD_REQUEST_PROFILE`
    - the explicit statement that this ADR introduces no new config key and no new environment variable

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md`
  - Owns (authoritative):
    - the stable inventory-schema extension introduced by ADR-0033
  - Must define:
    - the exact enum extension for `install.method`:
      - `apt | pacman | script | manual`
    - the exact `install.pacman` schema:
      - required iff `install.method=pacman`
      - ordered list of non-empty package-name strings
      - version pinning unsupported in v1
    - the exact mutual-exclusion and absence rules for `install.apt`, `install.pacman`, `install.script_path`, `install.script`, and `install.manual_instructions`
    - the exact validation failures that make an item invalid under ADR-0033
    - canonical YAML examples for valid and invalid pacman-backed packages
    - the exact rule for how this schema extension composes with the upstream world-deps inventory contract without redefining unrelated fields

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/platform-parity-spec.md`
  - Owns (authoritative):
    - the per-platform and per-backend guarantees for ADR-0033
    - the required validation evidence per platform
  - Must define:
    - required behavior platforms and CI parity platforms
    - the exact current Linux posture for host-native backends
    - the exact current macOS posture for Lima-backed worlds
    - the exact current Windows posture for WSL-backed worlds
    - permitted divergences in remediation text when platform-specific guidance is required
    - the exact smoke/manual evidence expected for supported and unsupported platform paths

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`
  - Owns (authoritative):
    - the A/B decisions required to remove ADR-0033 ambiguity before execution
  - Must define:
    - DR-0001 — schema posture
      - option A: explicit `install.method=pacman`
      - option B: abstract manager-agnostic system-packages method
      - one selected option
    - DR-0002 — world OS probe contract
      - exactly two options for the probe source-of-truth and mismatch/tie-break handling
      - one selected option
    - DR-0003 — pacman invocation and idempotency strategy
      - exactly two options for pacman command construction and repeat-run semantics
      - one selected option
    - the exact downstream docs and surfaces constrained by each selected decision

### Validation artifacts (required)

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`
  - Owns (authoritative):
    - deterministic manual validation setup, commands, expected output, and exit-code expectations
  - Must define:
    - a supported Arch-family provisioning case
    - an unsupported-backend or unsupported-platform case
    - a manager-mismatch case
    - a runtime fail-early case for system-package items

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh`
  - Owns (authoritative):
    - Linux smoke validation for the current Linux contract selected by `platform-parity-spec.md`
  - Must define:
    - exact commands and assertions
    - the expected exit codes and required substrings for unsupported or supported Linux behavior

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh`
  - Owns (authoritative):
    - macOS smoke validation for the current macOS contract selected by `platform-parity-spec.md`
  - Must define:
    - exact commands and assertions
    - the expected exit codes and required substrings for supported or unsupported macOS behavior

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1`
  - Owns (authoritative):
    - Windows smoke validation for the current Windows contract selected by `platform-parity-spec.md`
  - Must define:
    - exact commands and assertions
    - the expected exit codes and required substrings for supported or unsupported Windows behavior

### Slice specs (required)

Slice specs MUST use the canonical layout:
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/<SLICE_ID>/<SLICE_ID>-spec.md`

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP0/NASP0-spec.md`
  - Owns (authoritative):
    - the `NASP0` slice scope and acceptance criteria for world OS detection and support gating
  - Must define:
    - the exact probe inputs:
      - `/etc/os-release` `ID`
      - `/etc/os-release` `ID_LIKE`
      - `command -v pacman`
    - the exact normalization, absence, and mismatch rules for those inputs
    - the exact rule that probe selection is in-world only and never host-PATH based
    - the exact acceptance criteria proving:
      - Arch-family worlds with `pacman` available are recognized as pacman-capable
      - non-Arch or unsupported worlds are rejected deterministically
      - the selected runtime/provisioning mismatch error path uses the contract-defined exit code and remediation wording

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP1/NASP1-spec.md`
  - Owns (authoritative):
    - the `NASP1` slice scope and acceptance criteria for schema validation and inventory-view updates
  - Must define:
    - the exact implementation boundary for `install.method=pacman` inventory support
    - the exact validation failures that make pacman-backed inventory items invalid
    - the exact mutual exclusion and absence rules for `install.apt`, `install.pacman`, `install.script*`, and `install.manual_instructions`
    - the exact valid and invalid YAML examples for pacman-backed items
    - the exact list/show JSON and YAML view expectations for `install.method=pacman`
    - the exact acceptance criteria proving:
      - invalid pacman schema shapes fail with the contract-defined config/schema exit code
      - pacman-backed items render with the intended method and field names across inventory views

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP2/NASP2-spec.md`
  - Owns (authoritative):
    - the `NASP2` slice scope and acceptance criteria for provisioning routing and pacman command execution
  - Must define:
    - the exact pacman requirement-derivation rule from the effective enabled set
    - the exact pacman package de-duplication and stable ordering rules
    - the exact request-profile usage boundaries for provisioning
    - the exact pacman command-construction and idempotency contract selected by `decision_register.md`
    - the exact acceptance criteria proving:
      - supported provisioning constructs the intended in-world pacman command set
      - mismatched manager requirements fail deterministically without partial mutation

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP3/NASP3-spec.md`
  - Owns (authoritative):
    - the `NASP3` slice scope and acceptance criteria for runtime fail-early behavior and explicit-item scoping
  - Must define:
    - the exact runtime fail-early rule for `current sync` and `current install`
    - the exact explicit-item scope rule for `current install <ITEM...>`
    - the exact remediation wording invariants for runtime failure
    - the exact acceptance criteria proving:
      - runtime paths never invoke OS package managers for `apt` or `pacman`
      - remediation text is manager-aware and does not imply host mutation

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP4/NASP4-spec.md`
  - Owns (authoritative):
    - the `NASP4` slice scope and acceptance criteria for doc reconciliation, platform parity, and validation evidence
  - Must define:
    - the exact doc-update targets that must be reconciled to the feature contract
    - the exact platform parity posture to validate across Linux, macOS, and Windows
    - the exact manual and smoke evidence required before slice closeout
    - the exact acceptance criteria proving:
      - upstream docs and contracts are updated to leave exactly one authoritative truth
      - platform parity and validation evidence match the accepted manager-aware contract

## Coverage matrix (surface → authoritative doc)

Every surface touched by ADR-0033 appears here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| CLI provisioning entrypoint: `substrate world enable --provision-deps [--dry-run] [--verbose]` | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | flags, defaults, success/no-op semantics, unsupported semantics, examples |
| Provisioning `--dry-run` and `--verbose` output contract | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | exact no-mutation rule, minimum required output elements, stream expectations |
| Runtime invariant: `substrate world deps current sync|install` MUST NOT invoke `apt` or `pacman` | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | prohibited side effects, command coverage, operator-visible rationale |
| Runtime scope rule for `current install` with explicit item args | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | one deterministic rule for enabled-set vs explicit-item evaluation |
| Runtime remediation message invariants | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | exact command string, manager-aware wording, no-host-mutation wording |
| Exit-code meanings for provisioning and runtime (`0`, `2`, `3`, `4`, `5`) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | taxonomy mapping and per-command meaning |
| Host-no-mutation guarantee and provisioning mutation boundary | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | exact host-vs-world mutation rule and failure posture |
| Relationship between operator workflow and `SUBSTRATE_WORLD_REQUEST_PROFILE` | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md` | exact statement that operator-facing provisioning does not rely on manual env-var use |
| `install.method` enum extension to include `pacman` | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md` | exact allowed values and schema delta |
| `install.pacman` field | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md` | type, ordering, absence semantics, non-empty-string rule |
| Pacman version-pinning posture | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md` | explicit v1 prohibition and validation rule |
| Mutual exclusion between `install.apt`, `install.pacman`, `install.script*`, and `install.manual_instructions` | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md` | exact allowed combinations and invalid combinations |
| Invalid pacman schema shapes | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md` | exact invalid cases and links to contract-defined exit behavior |
| Pacman requirement derivation from the effective enabled set | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP2/NASP2-spec.md` | filter rule, bundle expansion boundary, enabled-set inputs |
| Pacman package de-duplication and stable ordering | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP2/NASP2-spec.md` | one deterministic de-dup rule and one deterministic ordering rule |
| Mixed-manager enabled-set posture | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP2/NASP2-spec.md` | exact mismatch/failure rule when the enabled set contains incompatible system-package methods |
| World OS probe inputs (`ID`, `ID_LIKE`, `command -v pacman`) | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP0/NASP0-spec.md` | exact probe inputs, normalization, and absence semantics |
| World OS probe tie-break and mismatch rules | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP0/NASP0-spec.md` | one deterministic source-of-truth rule and one deterministic conflict rule |
| In-world-only manager selection invariant | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP0/NASP0-spec.md` | explicit prohibition on host-PATH-based selection |
| Pacman invocation and idempotency contract | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP2/NASP2-spec.md` | exact command shape, flags, repeat-run behavior, and non-partial-failure posture |
| Platform/backend support matrix | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/platform-parity-spec.md` | exact Linux/macOS/Windows guarantees and permitted divergences |
| Linux validation contract | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh` | exact assertions and exit-code expectations for Linux |
| macOS validation contract | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh` | exact assertions and exit-code expectations for macOS |
| Windows validation contract | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1` | exact assertions and exit-code expectations for Windows |
| Manual validation | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md` | preconditions, commands, expected output, expected exit codes |
| Shared `--provision-deps` contract reconciliation across affected docs | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP4/NASP4-spec.md` | exact doc targets and acceptance criteria for single-source-of-truth reconciliation |
| Slice acceptance for world OS probe and support gate | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP0/NASP0-spec.md` | scope and acceptance criteria IDs |
| Slice acceptance for schema validation and inventory views | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP1/NASP1-spec.md` | scope and acceptance criteria IDs |
| Slice acceptance for provisioning wiring | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP2/NASP2-spec.md` | scope and acceptance criteria IDs |
| Slice acceptance for runtime fail-early | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP3/NASP3-spec.md` | scope and acceptance criteria IDs |
| Slice acceptance for validation evidence and doc reconciliation | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP4/NASP4-spec.md` | scope and acceptance criteria IDs |
| Decision A/B selections required by ADR-0033 | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md` | exactly two options and one selection for DR-0001, DR-0002, and DR-0003 |
| Existing world-agent execute/stream protocol baseline | `docs/WORLD.md` | existing endpoint/request semantics and request `profile` field existence |
| Existing env-var surface `SUBSTRATE_WORLD_REQUEST_PROFILE` | `docs/CONFIGURATION.md` | name, meaning, default, and advanced/testing scope |
| Existing world-deps inventory/enabled baseline (`install.method=apt`, `install.apt`, enabled-set resolution) | `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` | existing schema and enabled-resolution rules reused by ADR-0033 |
| Baseline provisioning-time APT posture and runtime fail-early direction | `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md` | the APT baseline that ADR-0033 extends and must reconcile with |
| No new environment variables | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md` | explicit statement that ADR-0033 introduces no new `SUBSTRATE_*`, `SHIM_*`, or `WORLD_*` variable |
| No new structured log or trace fields | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md` | explicit statement that ADR-0033 introduces no telemetry-schema delta |
| Slice sequencing and validation commands | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/plan.md` | exact slice order and exact validation command list |
| Task graph and automation metadata | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json` | exact task IDs, dependencies, references, and automation metadata |
| Checkpoint boundaries and gate cadence | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/ci_checkpoint_plan.md` | exact checkpoint layout and required gates |
| Touch set, cross-pack conflicts, and operator-doc reconciliation | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md` | exact touched paths and explicit conflict handling |
| Required-doc set and ownership map | `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md` | complete doc list, ownership map, and follow-ups |

## Determinism checklist (must be satisfied before quality gate)

For every selected spec document, confirm it explicitly defines:
- inputs and precedence order when multiple inputs exist
- defaults and absence semantics
- the data model and constraints for every stable serialized boundary
- the error model and failure posture
- ordering, idempotency, and partial-failure rules when provisioning occurs
- security and host/world mutation invariants
- platform guarantees for Linux, macOS, and Windows

## Follow-ups

1. Shared contract ownership is still split across existing docs
   - Issue: ADR-0033 extends the same `substrate world enable --provision-deps` surface introduced by ADR-0030, while the implemented world-deps contract docs still describe runtime APT behavior that ADR-0030/0033 reject.
   - Required fix: in `contract.md`, `slices/NASP4/NASP4-spec.md`, and `pre-planning/impact_map.md`, define the exact single-source-of-truth reconciliation plan so there is one authoritative contract for provisioning-time system packages and runtime fail-early behavior.

2. ADR-0033 still points at stale spec paths
   - Issue: ADR-0033 “Related Docs” references a flat `spec_manifest.md` path and an assumed `specs/world_deps_pacman_provisioning.md` file, but this pack’s canonical output is `pre-planning/spec_manifest.md` plus canonical slice specs under `slices/NASP*/`.
   - Required fix: after promotion, update ADR-0033 to the canonical pre-planning and slice-spec paths.

3. Windows posture is still assumption-only
   - Issue: ADR-0033 says Windows behavior is an assumption, but this pack requires an exact cross-platform contract.
   - Required fix: in `platform-parity-spec.md`, `contract.md`, and `smoke/windows-smoke.ps1`, select one deterministic Windows posture and define the exact validation expectation.

4. Mixed-manager behavior is not pinned yet
   - Issue: ADR-0033 requires mismatch failure when system-package methods do not match the detected world OS package manager, but it does not define the exact rule for enabled sets that contain both APT and pacman items.
   - Required fix: in `slices/NASP2/NASP2-spec.md` and `contract.md`, define one deterministic mixed-manager rule, including whether any partial provisioning is forbidden and how the error is surfaced.

5. Runtime `current install <ITEM...>` scope remains ambiguous
   - Issue: ADR-0033 talks about the effective enabled set, but the CLI also supports explicit-item install targeting.
   - Required fix: in `contract.md` and `slices/NASP3/NASP3-spec.md`, choose one deterministic scope rule for runtime fail-early behavior and require tests to enforce it.

6. Probe tie-break behavior is still implied
   - Issue: ADR-0033 requires both `/etc/os-release` and `command -v pacman`, but it does not specify which signal wins when they disagree.
   - Required fix: in `decision_register.md` DR-0002 and `slices/NASP0/NASP0-spec.md`, define one exact precedence/tie-break rule and the exact failure behavior for contradictory probe results.

7. Pacman invocation details are not pinned yet
   - Issue: ADR-0033 requires provisioning via `pacman` but does not define the exact command flags, update/install sequencing, lock-handling posture, or dry-run rendering.
   - Required fix: in `decision_register.md` DR-0003 and `slices/NASP2/NASP2-spec.md`, define one exact pacman invocation contract and its idempotency guarantees.

8. Built-in inventory strategy is still open
   - Issue: the intake asks whether built-in inventory items gain pacman variants now or whether pacman support is user-defined inventory only in v1.
   - Required fix: in `pre-planning/impact_map.md` and `slices/NASP1/NASP1-spec.md`, define the exact v1 touch boundary so planning does not imply built-in inventory expansion unless it is explicitly selected.

9. Validation substrate for real Arch-family success is not enumerated by exact path
   - Issue: ADR-0033 requires an Arch-family provisioning success case, but it does not identify the exact manual fixture, guest image, or automated harness path that will provide that evidence.
   - Required fix: in `manual_testing_playbook.md`, `platform-parity-spec.md`, and `pre-planning/impact_map.md`, name the exact evidence path for the supported success case and the exact fallback if that evidence cannot be automated in smoke.
