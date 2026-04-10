# persist-macos-host-os-install-state — spec manifest (pre-planning)

This file enumerates every contract, schema, filesystem, compatibility, validation, and no-change surface touched by ADR-0039 and assigns each surface to exactly one authoritative document.

Authoring standards:
- `docs/project_management/system/fse/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`
- `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-macos-host-os-install-state/`
- ADRs:
  - `docs/project_management/adrs/draft/ADR-0039-capturing-koala.md`
- External authoritative inputs that remain outside this pack:
  - `docs/project_management/packs/implemented/persist-detected-linux-distro-pkg-manager/contract.md`
    - authoritative for the existing Linux producer contract for `install_state.json`, including the canonical path rule and the Linux-only `host_state.platform.*` surface that ADR-0039 preserves
  - `docs/project_management/packs/implemented/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
    - authoritative for the existing Linux `host_state.platform.*` schema and its merge/preservation boundary
  - `docs/INSTALLATION.md`
    - existing operator doc that already documents `install_state.json` and needs reconciliation after promotion
  - `scripts/substrate/install-substrate.sh`
    - current hosted installer producer implementation
  - `scripts/substrate/dev-install-substrate.sh`
    - current dev-installer producer implementation that already shares the Linux metadata writer pattern
  - `scripts/substrate/uninstall-substrate.sh`
    - current hosted uninstaller reader of `install_state.json`
  - `scripts/substrate/dev-uninstall-substrate.sh`
    - current dev-uninstaller reader of `install_state.json`
  - `tests/installers/install_state_smoke.sh`
    - existing cross-installer install-state smoke harness
  - `tests/mac/installer_parity_fixture.sh`
    - existing macOS installer parity fixture and cleanup-guidance harness

## Required documents by role

### Pre-planning artifacts produced in this lane

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md`
  - Role: pre-planning
  - Owns:
    - the exact required-doc set for ADR-0039
    - the surface-to-doc ownership map
    - the explicit list of selected and unselected doc classes
  - Must define:
    - every feature-local doc selected for this body of work
    - every reused external authority that ADR-0039 depends on without redefining
    - every follow-up needed to remove remaining ADR ambiguity before downstream planning continues

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md`
  - Role: pre-planning
  - Owns:
    - the exact touched implementation and doc reconciliation set
    - the external harness and operator-doc edits required by ADR-0039
  - Must define:
    - the touched producer scripts
    - the touched uninstaller-reader scripts
    - the touched validation harnesses
    - the touched operator docs, including `docs/INSTALLATION.md`

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/minimal_spec_draft.md`
  - Role: pre-planning
  - Owns:
    - the first-pass pack-local summary of frozen invariants before downstream specs are authored
  - Must define:
    - the single canonical metadata path rule
    - the single accepted additive-file rule
    - the pack-local list of downstream docs and their intended authority boundaries

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/workstream_triage.md`
  - Role: pre-planning
  - Owns:
    - advisory sequencing guidance for downstream doc authoring and decomposition
  - Must define:
    - the doc-authoring order that keeps schema, filesystem semantics, parity, and validation surfaces coherent
    - the highest-risk seams in the shared `install_state.json` writer and reader boundary

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/alignment_report.md`
  - Role: pre-planning
  - Owns:
    - the wrapper-compiled alignment status for this pack
  - Must define:
    - every blocking contradiction or drift item discovered during pre-planning
    - the pack-local closeout gate for moving into downstream planning

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/ci_checkpoint_plan.md`
  - Role: pre-planning
  - Owns:
    - the checkpoint cadence for this cross-platform shared-file change
  - Must define:
    - the checkpoint boundary after contract, schema, filesystem, parity, and compatibility docs are all authored
    - the validation gates that verify macOS producer behavior plus Linux and Windows no-change boundaries
  - Selection rationale:
    - ADR-0039 changes a shared persisted file used across macOS, Linux, and uninstall readers.
    - Validation crosses hosted install, hosted `--no-world`, and existing reader compatibility boundaries.

### Topic-specific specs required by the ADR

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md`
  - Role: topic-specific spec
  - Owns:
    - the operator-facing contract for ADR-0039
    - the canonical metadata path rule
    - the user-visible warning and exit-code posture
    - the future-consumer read-precedence rule
  - Must define:
    - the exact statement that ADR-0039 introduces no new commands and no new flags
    - the exact installer entrypoints covered by the selected producer contract
    - the canonical file rule:
      - on-disk path: `<effective_prefix>/install_state.json`
      - default-prefix alias: `~/.substrate/install_state.json`
    - the exact rule that ADR-0039 does not introduce a second metadata file
    - the exact exit-code posture:
      - exit-code taxonomy reference
      - metadata read, parse, collection, or write failures do not flip an otherwise successful install into failure
    - the exact warning-only diagnostic posture visible to operators
    - the exact future-consumer rule:
      - prefer persisted `host_state.os.*` values when present
      - tolerate missing or partial values
      - fall back to runtime detection when persisted values are absent or unreadable
    - the explicit statement that ADR-0039 introduces no new environment variable and no new structured telemetry field

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md`
  - Role: topic-specific spec
  - Owns:
    - the `host_state.os.*` schema introduced by ADR-0039
    - the exact merge boundary between the new macOS block and the preexisting Linux and cleanup fields
  - Must define:
    - the exact field paths, types, and allowed values for:
      - `host_state.os`
      - `host_state.os.family`
      - `host_state.os.product_version`
      - `host_state.os.build_version`
      - `host_state.os.arch`
    - the exact rule that `host_state.os.family` is `"macos"`
    - the exact source mapping for:
      - `sw_vers -productVersion`
      - `sw_vers -buildVersion`
      - `uname -m`
    - the exact field-level absence semantics when one or more source commands fail
    - the exact merge rule that preserves:
      - `host_state.group`
      - `host_state.linger`
      - `host_state.platform.*`
      - unknown top-level and unknown `host_state` sibling keys
    - canonical JSON examples for:
      - a fresh macOS install
      - an existing file upgraded with the macOS block
      - a successful macOS install with partial OS-detail capture

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md`
  - Role: topic-specific spec
  - Owns:
    - the file-write algorithm and failure-handling semantics for ADR-0039
  - Must define:
    - the exact write trigger for a successful macOS producer flow
    - the exact same-directory temp-file path rule for `install_state.json.tmp`
    - the exact replace rule that occurs only after a complete JSON document exists at the temp path
    - the explicit ban on in-place truncation of the canonical file
    - the exact recovery rule on parse failure:
      - emit a warning
      - seed from a fresh `schema_version = 1` document
    - the exact recovery rule on temp-file write or replace failure:
      - emit a warning
      - preserve the prior canonical file when one existed
      - remove the temp file when removal succeeds
    - the exact directory-creation and protected-path boundaries for metadata writes

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md`
  - Role: topic-specific spec
  - Owns:
    - the platform matrix for producer and reader behavior under ADR-0039
    - the validation evidence contract for macOS plus Linux and Windows no-change boundaries
  - Must define:
    - the exact macOS guarantee for the selected producer entrypoints
    - the exact Linux no-change guarantee for `host_state.platform.*`
    - the exact Windows no-change guarantee for this ADR
    - the exact uninstaller no-change boundary:
      - cleanup behavior remains Linux-only
      - new macOS `host_state.os.*` data is diagnostic-only and does not create a new cleanup action
    - the exact automated-evidence mapping to existing external harnesses
    - the exact manual-evidence mapping to `manual_testing_playbook.md`

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md`
  - Role: topic-specific spec
  - Owns:
    - the additive compatibility policy for ADR-0039
    - the reader-tolerance contract for existing uninstallers and future consumers
  - Must define:
    - the explicit additive-only rule for the new macOS block
    - the explicit rule that existing readers ignore unknown keys and survive the new `host_state.os.*` subtree
    - the explicit no-migration and no-backfill rule beyond normal successful macOS install writes
    - the explicit rule that ADR-0039 does not redefine or rename Linux `host_state.platform.*` fields
    - the compatibility relationship between ADR-0039 and the implemented Linux persistence pack

### Downstream FSE planning/decomposition artifacts that must exist later

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/decision_register.md`
  - Role: downstream planning artifact
  - Owns:
    - the remaining A/B decisions that ADR-0039 leaves unresolved
  - Must define:
    - DR-0001: macOS producer scope
      - option A: hosted installer only
      - option B: hosted installer plus dev installer
      - one selected option
    - DR-0002: partial-capture serialization rule
      - option A: write `host_state.os` with all successfully collected leaves and omit failed leaves
      - option B: require a full leaf set before writing the block
      - one selected option
    - DR-0003: automated validation split
      - option A: `tests/installers/install_state_smoke.sh` is the primary automated evidence path
      - option B: `tests/mac/installer_parity_fixture.sh` is the primary automated evidence path
      - one selected option plus the rule for any secondary evidence path

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/manual_testing_playbook.md`
  - Role: downstream planning artifact
  - Owns:
    - the deterministic manual validation procedure for ADR-0039
  - Must define:
    - the macOS hosted install case
    - the macOS hosted `--no-world` case
    - the exact file-inspection assertions for `install_state.json`
    - the exact warning-only behavior check for OS-detail collection failure
    - the exact no-change check for Linux-only uninstall cleanup behavior

### Candidate downstream decomposition docs likely needed later

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/persist-host-state-os-schema-and-merge.md`
  - Role: candidate decomposition doc
  - Intended ownership:
    - slice-local acceptance criteria for `host_state.os.*` payload shape and merge preservation

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/macos-writer-flow-and-warning-only-degradation.md`
  - Role: candidate decomposition doc
  - Intended ownership:
    - slice-local acceptance criteria for producer scope, temp-file replacement, and failure degradation

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/slices/validation-and-doc-reconciliation.md`
  - Role: candidate decomposition doc
  - Intended ownership:
    - slice-local acceptance criteria for harness updates, `docs/INSTALLATION.md` reconciliation, and manual evidence capture

## Coverage matrix (surface → authoritative doc)

Every surface touched by ADR-0039 appears here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| No new CLI commands and no new flags | `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md` | exact no-new-CLI statement plus the installer entrypoints constrained by ADR-0039 |
| Canonical metadata file path and default-prefix alias | `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md` | `<effective_prefix>/install_state.json`, `~/.substrate/install_state.json`, and the no-second-file rule |
| Exit-code posture for metadata collection, parse, and write failures | `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md` | taxonomy reference plus the rule that these failures remain warning-only on otherwise successful installs |
| Warning-only operator diagnostics | `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md` | exact warning posture and the rule that diagnostics remain non-fatal |
| Future-consumer read precedence for `host_state.os.*` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/contract.md` | prefer persisted values, tolerate missing or partial values, fall back to runtime detection |
| `host_state.os` object presence rule | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | object shape, requiredness, and relation to successful macOS installs |
| `host_state.os.family` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | path, type, and fixed value `"macos"` |
| `host_state.os.product_version` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | path, type, source command, and absence semantics |
| `host_state.os.build_version` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | path, type, source command, and absence semantics |
| `host_state.os.arch` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | path, type, source command, and absence semantics |
| Preservation of `host_state.group` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | exact merge-preservation rule for the preexisting cleanup metadata subtree |
| Preservation of `host_state.linger` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | exact merge-preservation rule for the preexisting cleanup metadata subtree |
| Preservation of `host_state.platform.*` | `docs/project_management/packs/implemented/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | existing Linux field names, types, and semantics that ADR-0039 preserves without redefining |
| Preservation of unknown keys | `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md` | additive-only compatibility rule and reader tolerance of untouched unknown keys |
| `schema_version` remaining integer `1` | `docs/project_management/packs/draft/persist-macos-host-os-install-state/install-state-schema-spec.md` | field name, type, and fixed value |
| Additive-only compatibility and no migration | `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md` | no rename, no removal, no migration, no backfill beyond normal successful writes |
| Existing uninstaller tolerance of the new `host_state.os.*` subtree | `docs/project_management/packs/draft/persist-macos-host-os-install-state/compatibility-spec.md` | exact reader-tolerance contract for hosted and dev uninstallers |
| Successful macOS producer-scope decision across installer entrypoints | `docs/project_management/packs/draft/persist-macos-host-os-install-state/decision_register.md` | one selected producer-scope rule |
| Partial-capture serialization decision | `docs/project_management/packs/draft/persist-macos-host-os-install-state/decision_register.md` | one selected rule for block creation when a source command fails |
| Automated validation split across existing harnesses | `docs/project_management/packs/draft/persist-macos-host-os-install-state/decision_register.md` | one selected primary automated evidence path and the rule for any secondary path |
| Same-directory temp-file path and replace sequence | `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md` | temp path, complete-document requirement, and single replace step |
| Parse-failure recovery to a fresh document | `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md` | warning emission plus seeding from a fresh `schema_version = 1` document |
| Failed temp-file write or replace recovery | `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md` | preserve prior canonical file, remove temp file when removal succeeds, maintain warning-only posture |
| Protected-path and directory-creation boundary for metadata writes | `docs/project_management/packs/draft/persist-macos-host-os-install-state/filesystem-semantics-spec.md` | write boundary remains inside the effective install prefix and does not expand to unrelated paths |
| macOS producer guarantee | `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md` | exact supported producer branches and exact no-change exclusions |
| Linux no-change guarantee | `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md` | Linux `host_state.platform.*` semantics remain externally owned and unchanged |
| Windows no-change guarantee | `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md` | no new Windows producer behavior under ADR-0039 |
| Uninstaller cleanup remains Linux-only | `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md` | exact no-change statement for cleanup actions on macOS and Windows |
| Manual validation procedure | `docs/project_management/packs/draft/persist-macos-host-os-install-state/manual_testing_playbook.md` | preconditions, commands, expected file content, expected warnings, and no-change checks |
| External automated validation harness targets | `docs/project_management/packs/draft/persist-macos-host-os-install-state/platform-parity-spec.md` | exact harness list, exact required assertions, and expected evidence per harness |
| External touch set and operator-doc reconciliation set | `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md` | exact implementation files, test files, and docs touched by the feature |
| Required-doc set and ownership map | `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md` | complete selected-doc list, coverage matrix, explicit unselected doc classes, and follow-ups |
| No new environment-variable surface | `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md` | explicit statement that ADR-0039 introduces no new env-var contract |
| No new wire or API protocol surface | `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md` | explicit statement that ADR-0039 does not add a host↔world or host↔agent protocol |
| No new policy surface | `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md` | explicit statement that ADR-0039 does not add or change policy evaluation inputs |
| No new structured telemetry or trace field | `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md` | explicit statement that ADR-0039 changes warning diagnostics only, not structured telemetry schema |

## Explicitly unselected doc classes

No feature-local doc is selected for these classes:

- Protocol spec
  - ADR-0039 does not define a new stable HTTP, WebSocket, CLI RPC, or IPC surface.

- Env-vars spec
  - ADR-0039 introduces no new environment variable and does not redefine existing environment-variable semantics.

- Policy spec
  - ADR-0039 does not add a broker rule, approval rule, or enforcement-mode change.

- Telemetry spec
  - ADR-0039 adds warning-only diagnostics but does not add a structured log field, trace field, or redaction rule.

- Feature-local smoke scripts
  - ADR-0039 extends existing external harnesses rather than introducing a new feature-local smoke script under this pack.

## Determinism checklist

Before quality gate, the selected docs must define:

- one canonical metadata path and one default-prefix alias
- one exact producer-scope rule across the macOS installer entrypoints
- one exact `host_state.os.*` field set with field-level absence semantics
- one exact additive-compatibility rule for preserved Linux fields, preserved cleanup fields, and preserved unknown keys
- one exact temp-file and atomic-replace rule
- one exact warning-only failure posture for collection, parse, and write errors
- one exact platform matrix for macOS, Linux, and Windows
- one exact mapping from required validation evidence to the existing automated harnesses and the manual playbook

## Follow-ups

1. Producer-scope wording remains unresolved in the ADR body
   - Issue: ADR-0039 names `scripts/substrate/install-substrate.sh` and manual validation for hosted install plus hosted `--no-world`, but the repo already has `scripts/substrate/dev-install-substrate.sh` as a second install-state producer.
   - Required action: author `decision_register.md` DR-0001 and then reconcile `contract.md`, `filesystem-semantics-spec.md`, and `platform-parity-spec.md` to the selected producer scope.

2. Partial-capture semantics remain unresolved
   - Issue: ADR-0039 requires `host_state.os` after successful macOS installs and also states that future consumers tolerate missing or partial values. The exact block-creation rule for failed `sw_vers` or `uname` commands is not locked.
   - Required action: author `decision_register.md` DR-0002 and then codify the selected rule in `install-state-schema-spec.md`.

3. Automated validation ownership remains unresolved
   - Issue: ADR-0039 requires macOS parity or smoke coverage, and the repo already has both `tests/installers/install_state_smoke.sh` and `tests/mac/installer_parity_fixture.sh`.
   - Required action: author `decision_register.md` DR-0003 and then map the selected primary harness plus any secondary evidence path in `platform-parity-spec.md` and `impact_map.md`.

4. Operator docs contain live drift
   - Issue: `docs/INSTALLATION.md` currently states that macOS does not write `install_state.json`, which contradicts ADR-0039.
   - Required action: record `docs/INSTALLATION.md` in `impact_map.md` and reconcile that wording after promotion so the operator doc matches the new contract.
