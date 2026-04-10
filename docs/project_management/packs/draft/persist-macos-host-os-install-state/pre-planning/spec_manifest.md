# persist-macos-host-os-install-state — spec manifest (pre-planning)

This file enumerates every contract, schema, path, compatibility, platform, and validation surface touched by ADR-0039 and assigns each surface to exactly one authoritative document.

Authoring standards:
- `docs/project_management/system/fse/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`
- `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-macos-host-os-install-state/`
- ADRs:
  - `docs/project_management/adrs/draft/ADR-0039-capturing-koala.md`
- External authoritative docs and touched references that remain outside this pack:
  - `docs/project_management/packs/implemented/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` — authoritative for Linux `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*` field semantics
  - `docs/project_management/packs/implemented/persist-detected-linux-distro-pkg-manager/contract.md` — authoritative for the existing Linux installer metadata contract that this ADR preserves rather than redefining
  - `docs/INSTALLATION.md` — touched operator doc that currently states macOS does not write `install_state.json`
  - `scripts/substrate/install-substrate.sh` — current hosted installer implementation reference
  - `scripts/substrate/dev-install-substrate.sh` — current dev installer implementation reference
  - `scripts/substrate/uninstall-substrate.sh` — current hosted uninstall implementation reference
  - `scripts/substrate/dev-uninstall-substrate.sh` — current dev uninstall implementation reference
  - `tests/installers/install_state_smoke.sh` — current cross-installer metadata smoke harness
  - `tests/mac/installer_parity_fixture.sh` — current macOS installer parity harness

## Required documents under the feature directory

### Pre-planning artifacts produced in this lane

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md`
  - Ownership scope:
    - exact required-doc selection for this feature
    - one-owner-per-surface coverage matrix
    - explicit list of unselected doc classes for this ADR
  - Phase:
    - pre-planning

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md`
  - Ownership scope:
    - exact touch set for installer scripts, test harnesses, and operator docs
    - cross-pack implications for Linux persistence docs and macOS operator docs
  - Phase:
    - pre-planning

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/minimal_spec_draft.md`
  - Ownership scope:
    - initial contract skeleton for the selected downstream specs
    - cross-doc term normalization for `effective_prefix`, `install_state.json`, and `host_state.os`
  - Phase:
    - pre-planning

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/workstream_triage.md`
  - Ownership scope:
    - downstream workstream split and sequencing guidance
    - recommended seam boundaries for contract/schema work versus validation/doc reconciliation
  - Phase:
    - pre-planning

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/alignment_report.md`
  - Ownership scope:
    - cross-doc contradiction check
    - hard gates that block downstream planning promotion
  - Phase:
    - pre-planning

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/ci_checkpoint_plan.md`
  - Ownership scope:
    - checkpoint cadence for a cross-platform validation surface
    - exact validation gates required before downstream execution
  - Phase:
    - pre-planning

### Topic-specific specs required by ADR-0039

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md`
  - Ownership scope:
    - installer-facing contract for no-new-command, no-new-flag, no-new-env-var, path alias, and exit-code posture
    - operator-facing statement of when macOS installs create or update `install_state.json`
  - Phase:
    - downstream spec authoring

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md`
  - Ownership scope:
    - full `install_state.json` schema boundary touched by this ADR
    - exact `host_state.os.*` field definitions and persisted-data allowlist
  - Phase:
    - downstream spec authoring

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md`
  - Ownership scope:
    - on-disk write path resolution
    - temp-file naming, same-directory placement, replace flow, cleanup, and no-write branches
  - Phase:
    - downstream spec authoring

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md`
  - Ownership scope:
    - macOS, Linux, and Windows behavior guarantees
    - installer-scope matrix and validation topology across hosted, hosted `--no-world`, and any included dev-install paths
  - Phase:
    - downstream spec authoring

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md`
  - Ownership scope:
    - additive-only compatibility policy
    - existing-file recovery rules and future-consumer tolerance rules
  - Phase:
    - downstream spec authoring

### Downstream planning and validation artifacts that must exist later

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/manual_testing_playbook.md`
  - Ownership scope:
    - deterministic manual validation procedure for macOS hosted install and macOS hosted `--no-world`
    - manual checks for file presence, JSON content, warning-only degradation, and operator-doc reconciliation evidence
  - Phase:
    - downstream planning and validation

### Candidate downstream decomposition docs

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/threaded-seams/seam-1-install-state-surface-lock.md`
  - Intended ownership:
    - contract, schema, filesystem, and compatibility surfaces for `install_state.json`

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/threaded-seams/seam-2-macos-validation-and-doc-alignment.md`
  - Intended ownership:
    - parity evidence, smoke-harness split, macOS fixture coverage, and operator-doc reconciliation

## Coverage matrix (surface → authoritative doc)

| Surface | Authoritative doc | What that doc must define |
| --- | --- | --- |
| No new CLI commands and no new CLI flags for this ADR | `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md` | exact statement that the installer command surface remains unchanged |
| No new feature-local environment variable surface | `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md` | exact statement that this ADR adds no env vars and reuses existing installer inputs only |
| Effective install prefix inputs and the operator-facing canonical location for `install_state.json` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md` | `--prefix` interaction, default `~/.substrate` alias, and the canonical operator-visible path rule |
| Exit-code semantics for successful installs with metadata degradation | `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md` | taxonomy reference plus the exact rule that metadata collection or persistence failure does not change an otherwise successful install into failure |
| On-disk canonical metadata path and temp-path derivation | `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md` | `<effective_prefix>/install_state.json`, `<effective_prefix>/install_state.json.tmp`, parent-directory behavior, and path invariants |
| Same-directory temp-file write and atomic replace flow | `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md` | create, write-complete, replace, and no in-place truncation rule |
| Warning-only file failure posture | `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md` | read failure, parse failure, temp-file write failure, replace failure, cleanup after failed temp write, and installer continuation rule |
| Dry-run no-write branch | `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md` | exact rule for file creation, temp-file creation, and parent-directory creation during dry-run |
| `install_state.json` top-level schema touched by this ADR | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | `schema_version`, `created_at`, `updated_at`, `host_state`, canonical types, and presence rules |
| New macOS object `host_state.os` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | object type, required-on-success rule, and exact leaf-field absence semantics |
| `host_state.os.family` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | type `string` and exact value `macos` |
| `host_state.os.product_version` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | source `sw_vers -productVersion`, type, canonical stored value, and absence rule |
| `host_state.os.build_version` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | source `sw_vers -buildVersion`, type, canonical stored value, and absence rule |
| `host_state.os.arch` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | source `uname -m`, type, canonical stored value, and absence rule |
| Persisted-data allowlist and explicit sensitive-data exclusions | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | exact statement that hostnames, serial numbers, and broad system profiler output are outside the persisted schema |
| Preservation of existing `host_state.group` JSON values | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | preserve-as-read rule without redefining the nested group schema |
| Preservation of existing `host_state.linger` JSON values | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | preserve-as-read rule without redefining the nested linger schema |
| Existing Linux `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*` field semantics | `docs/project_management/packs/implemented/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | Linux field names, Linux field meanings, and Linux vocabulary ownership |
| Preservation of any pre-existing Linux `host_state.platform.*` block during a macOS rewrite | `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md` | preserve-as-read rule and explicit no-mutation boundary for Linux metadata during macOS writes |
| Unknown-key preservation | `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md` | additive-only merge rule for unknown top-level and nested keys |
| Existing-file recovery when the prior document is unreadable or carries an unsupported schema version | `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md` | warning-only rebuild rule, fresh `schema_version = 1` seed rule, and no migration requirement |
| `schema_version = 1` compatibility posture | `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md` | no bump, no rename, no field removal, and no backfill requirement |
| Future-consumer read contract for `host_state.os.*` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md` | prefer persisted values when present, tolerate missing or partial values, and fall back to runtime detection |
| macOS success branches that create or update `install_state.json` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md` | exact macOS branch matrix for hosted install and hosted `--no-world` |
| Linux no-change guarantee for this ADR | `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md` | explicit statement that Linux persistence semantics remain externally owned and unchanged |
| Windows no-change guarantee for this ADR | `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md` | explicit statement that Windows host install metadata behavior remains unchanged |
| Dev-install inclusion or exclusion for macOS install-state persistence | `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md` | exact decision for `scripts/substrate/dev-install-substrate.sh` scope and required evidence |
| Automated validation topology | `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md` | exact split between `tests/installers/install_state_smoke.sh` and `tests/mac/installer_parity_fixture.sh` |
| Manual validation procedure | `docs/project_management/packs/draft/persist-macos-host-os-install-state/manual_testing_playbook.md` | exact setup, execution steps, expected JSON assertions, and expected warning-only outcomes |
| Existing operator-doc reconciliation set | `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md` | exact docs that restate the current macOS no-write posture and require update |
| Exact implementation and test touch set | `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md` | installer files, uninstall files reviewed for no-change boundaries, and test harness files updated or explicitly left unchanged |

## Deterministic items required per selected document

### Pre-planning artifacts

- `pre-planning/spec_manifest.md`
  - Must list every required doc under this feature directory.
  - Must map every touched surface to exactly one authoritative doc.
  - Must list every unselected doc class with a direct reason.

- `pre-planning/impact_map.md`
  - Must enumerate each create and edit target for installer scripts, test harnesses, and operator docs.
  - Must record the external Linux schema docs that remain authoritative.
  - Must record the uninstall-path review as a no-change verification surface or a bounded follow-up.

- `pre-planning/minimal_spec_draft.md`
  - Must provide one normalized vocabulary set for `effective_prefix`, `install_state.json`, `host_state.os`, `schema_version`, `created_at`, and `updated_at`.
  - Must provide initial section skeletons for the selected downstream specs.

- `pre-planning/workstream_triage.md`
  - Must group work into one contract-and-schema workstream and one validation-and-doc-alignment workstream.
  - Must define the sequencing rule that contract ownership locks before validation docs finalize.

- `pre-planning/alignment_report.md`
  - Must verify one-owner-per-surface coverage across ADR-0039 and every selected doc.
  - Must report any contradiction that blocks downstream authoring.

- `pre-planning/ci_checkpoint_plan.md`
  - Must define at least one checkpoint that validates macOS behavior and cross-platform no-change guarantees.
  - Must state the exact evidence gates for the selected validation harnesses.

### Topic-specific specs

- `contract.md`
  - Must define the no-new-command, no-new-flag, and no-new-env-var contract.
  - Must define the operator-facing location rule for `install_state.json`.
  - Must define the exit-code posture for warning-only metadata failures.

- `install-state-schema-spec.md`
  - Must define `schema_version`, `created_at`, and `updated_at`.
  - Must define the full `host_state.os` object, each leaf field, and canonical JSON examples.
  - Must define the persisted-data allowlist and the prohibited data families.
  - Must define preservation rules for `host_state.group` and `host_state.linger` values already present in the file.

- `filesystem-semantics-spec.md`
  - Must define the canonical path, temp path, parent-directory behavior, and replace flow.
  - Must define dry-run behavior and warning-only failure handling.
  - Must define the cleanup rule for a failed temp-file write.

- `platform-parity-spec.md`
  - Must define the exact macOS behavior delta.
  - Must define Linux and Windows no-change guarantees.
  - Must define whether dev install is included or excluded and the evidence required for that decision.
  - Must define the automated validation split across the existing harnesses.

- `compatibility-spec.md`
  - Must define additive-only compatibility, unknown-key preservation, and no migration.
  - Must define recovery behavior for unreadable existing files and unsupported schema versions.
  - Must define future-consumer tolerance for missing or partial `host_state.os.*` data and runtime fallback.
  - Must define the preserve-as-read rule for any pre-existing Linux `host_state.platform.*` block during macOS writes.

### Downstream planning and validation artifacts

- `manual_testing_playbook.md`
  - Must define the exact hosted macOS manual run.
  - Must define the exact hosted macOS `--no-world` manual run.
  - Must define expected JSON assertions for `host_state.os.family`, `product_version`, `build_version`, and `arch`.
  - Must define the expected warning-only result when OS-detail collection or metadata persistence fails.

## Explicitly unselected doc classes

No feature-local doc is selected for these classes:
- Protocol spec
  - ADR-0039 defines no wire, RPC, HTTP, WebSocket, or IPC contract.
- Policy spec
  - ADR-0039 defines no broker, approval, or policy-evaluation rule.
- Telemetry spec
  - ADR-0039 defines no new stable log field and no new trace field.
- Standalone env-vars spec
  - This ADR introduces no new env vars, and `contract.md` owns the small existing-input statement.
- Decision register
  - ADR-0039 already records the storage-file decision and the macOS field-family decision. Remaining gaps are scope and absence clarifications that belong in the selected specs.

## Determinism checklist

Before downstream planning starts, the selected docs must define:
- one canonical operator-facing path rule for `install_state.json`
- one canonical on-disk path and temp-path rule
- one full schema for every touched serialized field, including `created_at` and `updated_at`
- one exact absence rule for partial `host_state.os.*` capture failures
- one exact compatibility rule for unknown keys and pre-existing Linux metadata
- one exact platform matrix for macOS behavior and Linux and Windows no-change guarantees
- one exact validation topology across the existing automated harnesses and manual validation

## Follow-ups

- `platform-parity-spec.md` must decide whether `scripts/substrate/dev-install-substrate.sh` participates in the macOS write contract or remains outside this pack.
- `install-state-schema-spec.md` must define the exact stored shape when `sw_vers -productVersion`, `sw_vers -buildVersion`, or `uname -m` fails during an otherwise successful macOS install.
- `impact_map.md` must include every operator doc that states macOS does not write `install_state.json`, starting with `docs/INSTALLATION.md`.
- `platform-parity-spec.md` and `manual_testing_playbook.md` must lock one exact validation split between `tests/installers/install_state_smoke.sh` and `tests/mac/installer_parity_fixture.sh`.
- `impact_map.md` must record whether hosted and dev uninstall surfaces require doc updates or an explicit no-change note after macOS successful installs start leaving `install_state.json` behind.
