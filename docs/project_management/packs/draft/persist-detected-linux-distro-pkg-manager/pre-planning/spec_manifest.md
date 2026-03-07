# persist-detected-linux-distro-pkg-manager — spec manifest (pre-planning)

This file enumerates every contract surface touched by ADR-0032 and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`

## External authoritative inputs (this feature MUST NOT redefine these surfaces)
- `SUBSTRATE_HOME` environment-variable meaning, default path resolution, and global-home semantics:
  - `docs/reference/env/contract.md`
- Default exit-code taxonomy:
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Linux installer package-manager detection algorithm, selected-manager value set, and `pkg_manager.source` enum semantics:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`

## Slice IDs (canonical)

ADR-0032 uses placeholder slice IDs (`C0`, `C1`, `C2`). This feature MUST use feature-derived slice IDs per:
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`

Canonical slice IDs selected for this feature:
- Slice prefix: `PDLDPM` (derived from `persist-detected-linux-distro-pkg-manager`)
- `PDLDPM0` — capture distro/package-manager metadata into the install-state schema
- `PDLDPM1` — guarantee Linux install-state persistence on successful installs without host-state events
- `PDLDPM3` — extend dev-installer parity for the shared install-state contract
- `PDLDPM2` — extend Linux installer smoke coverage for the persisted metadata contract

## Required spec documents (authoritative)

Each entry below MUST exist under `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/` and MUST be treated as authoritative for the listed surfaces.

Spec templates:
- `docs/project_management/system/templates/planning_pack/`
- `docs/project_management/system/templates/spec/`

### Planning pack docs

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`
  - Owns (authoritative):
    - required-doc selection for this feature directory
    - surface-to-doc ownership map
    - follow-ups required to remove ADR ambiguity before quality gate
  - Must define (deterministic items):
    - the complete required-doc set listed in this file
    - the canonical slice IDs `PDLDPM0`, `PDLDPM1`, `PDLDPM3`, `PDLDPM2`
    - every ADR-0032 surface category, including surfaces owned by external dependency docs

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
  - Owns (authoritative):
    - touch set, cascading implications, and cross-pack conflicts for `PDLDPM0`, `PDLDPM1`, `PDLDPM3`, and `PDLDPM2`
  - Must define (deterministic items):
    - explicit edit allowlists by path for installer scripts, uninstall scripts, tests, and operator docs
    - exact external-doc update targets required to keep non-authoritative docs aligned with the selected contract
    - dependency notes for `best-effort-distro-package-manager`

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`
  - Owns (authoritative):
    - execution runbook and slice sequencing for `PDLDPM0`, `PDLDPM1`, `PDLDPM3`, and `PDLDPM2`
  - Must define (deterministic items):
    - one explicit slice order
    - exact validation commands for Linux installer smoke coverage
    - explicit statement that ADR-0032 introduces no macOS or Windows behavior delta
    - explicit list of any operator-doc updates that are required for closeout

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json` (already exists)
  - Owns (authoritative):
    - triad task graph, automation metadata, and slice-to-task traceability
  - Must define (deterministic items):
    - triad tasks for `PDLDPM0`, `PDLDPM1`, `PDLDPM3`, and `PDLDPM2`
    - references to `slices/PDLDPM0/PDLDPM0-spec.md`, `slices/PDLDPM1/PDLDPM1-spec.md`, `slices/PDLDPM3/PDLDPM3-spec.md`, and `slices/PDLDPM2/PDLDPM2-spec.md`
    - orchestration branch `feat/persist-detected-linux-distro-pkg-manager`

### Feature contract, schema, compatibility, and platform docs

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
  - Owns (authoritative):
    - the operator-facing contract for writing and reading persisted Linux install metadata
    - file path semantics for the resolved install-state file
    - failure posture, path invariants, and exit-code posture for metadata persistence
  - Must define (deterministic items):
    - the resolved file path as `<resolved SUBSTRATE_HOME>/install_state.json`
    - Linux-only write guarantee on successful installs
    - explicit no-change contract for macOS and Windows
    - explicit rule for `--no-world` Linux installs
    - explicit rule for dry-run installs
    - exact no-fail posture when metadata read, merge, or write steps fail
    - exact read contract for downstream guidance consumers: prefer persisted metadata when available; fall back to runtime detection when missing or unreadable
    - path invariant: metadata writes occur only under the resolved `SUBSTRATE_HOME`
    - exit-code rule: metadata persistence does not introduce a new non-zero success-path exit

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
  - Owns (authoritative):
    - the additive JSON schema for the persisted install-state file content introduced by ADR-0032
  - Must define (deterministic items):
    - exact JSON object path `host_state.platform`
    - field names, types, and optionality for:
      - `host_state.platform.os_release.id`
      - `host_state.platform.os_release.id_like`
      - `host_state.platform.pkg_manager.selected`
      - `host_state.platform.pkg_manager.source`
    - canonical omission rules when any platform field is unavailable
    - exact rule for preserving existing `host_state.group` and `host_state.linger` objects
    - sample payloads for:
      - Linux install with `os_release.*` and `pkg_manager.*`
      - Linux install without readable `/etc/os-release`

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md`
  - Owns (authoritative):
    - backward-compatibility rules for extending `install_state.json`
  - Must define (deterministic items):
    - `schema_version` remains `1`
    - older uninstall flows ignore unknown `host_state.platform.*` keys
    - merge/upgrade behavior when an existing file is missing fields, corrupt, or on the wrong schema version
    - additive-only evolution rule for this feature’s schema additions
    - explicit end condition: compat remains in force until a separate ADR changes `install_state.json` schema policy

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/platform-parity-spec.md`
  - Owns (authoritative):
    - per-platform guarantees and permitted divergences for ADR-0032
  - Must define (deterministic items):
    - Linux guarantee: successful installs persist the install-state contract selected by `contract.md` and `install-state-schema-spec.md`
    - macOS guarantee: ADR-0032 introduces no new `host_state.platform.*` write contract
    - Windows guarantee: ADR-0032 introduces no new `host_state.platform.*` write contract
    - required validation evidence: Linux coverage is required; macOS and Windows require explicit no-delta validation notes

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`
  - Owns (authoritative):
    - A/B decisions required to remove ADR-0032 ambiguity before implementation
  - Must define (deterministic items):
    - DR-0001: exact write semantics when the installer succeeds but platform metadata inputs are partially unavailable
    - DR-0002: exact dry-run semantics for install-state persistence
    - DR-0003: exact implementation scope for installer entrypoints (`install-substrate.sh` only vs both install scripts)
    - for each DR: exactly two options (A/B), one selection, and the exact docs that selection updates

### Slice specs

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md`
  - Owns (authoritative):
    - acceptance criteria for persisting the additive `host_state.platform.*` schema
  - Must define (deterministic items):
    - the exact JSON keys written by this slice
    - acceptance criteria for storing the installer-selected package manager without redefining the detection algorithm
    - acceptance criteria for storing `os_release.id` and `os_release.id_like`
    - acceptance criteria for omission when `/etc/os-release` fields are unavailable

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`
  - Owns (authoritative):
    - acceptance criteria for reliable install-state writes on successful Linux installs
  - Must define (deterministic items):
    - file-creation/update acceptance when no group or linger events occurred
    - idempotent merge acceptance criteria for existing install-state files
    - failure-posture acceptance criteria when metadata persistence fails
    - acceptance criteria for the selected `--no-world` and dry-run behaviors

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM3/PDLDPM3-spec.md`
  - Owns (authoritative):
    - acceptance criteria for dev-installer parity with the shared install-state contract
  - Must define (deterministic items):
    - exact acceptance criteria for persisting the selected install-state contract through `scripts/substrate/dev-install-substrate.sh`
    - exact acceptance criteria for keeping one `install_state.json` meaning across the production and dev installers
    - exact validation target path that owns dev-installer parity assertions

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`
  - Owns (authoritative):
    - acceptance criteria for automated validation of ADR-0032
  - Must define (deterministic items):
    - exact Linux smoke assertions for the persisted platform keys
    - exact Linux smoke assertions for schema-version preservation and host-state preservation
    - exact Linux smoke assertions for the selected fallback behavior when `/etc/os-release` is unreadable
    - exact validation target path for the smoke coverage that owns these assertions

## Coverage matrix (surface → authoritative doc)

Every ADR-0032 surface MUST appear exactly once in this matrix.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| `SUBSTRATE_HOME` meaning and default home resolution | `docs/reference/env/contract.md` | exact env-var semantics and default home path when unset |
| Resolved install-state file path `<resolved SUBSTRATE_HOME>/install_state.json` | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | path contract and write boundary |
| Linux-only successful-install write guarantee | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | file existence/update guarantee after successful Linux install |
| `--no-world` Linux install persistence behavior | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | one exact rule for whether platform metadata is still written |
| Dry-run persistence behavior | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | one exact rule for whether dry-run writes or skips install-state metadata |
| Metadata-persistence failure posture | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | warnings vs hard-failure rule and exit-code posture |
| Downstream read contract for guidance consumers | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | prefer persisted metadata when available; fall back to runtime detection when missing or unreadable |
| JSON path `host_state.platform` | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | object nesting and presence rules |
| JSON field `host_state.platform.os_release.id` | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | type, optionality, and serialization rules |
| JSON field `host_state.platform.os_release.id_like` | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | type, optionality, and serialization rules |
| JSON field `host_state.platform.pkg_manager.selected` | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | type, optionality, and serialization rules |
| JSON field `host_state.platform.pkg_manager.source` | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | type, optionality, and serialization rules |
| Stored value rules for `pkg_manager.selected` | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact allowed manager names and selection semantics |
| Stored value rules for `pkg_manager.source` | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact enum members and meaning of each member |
| Detection algorithm that produces the selected package manager | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | precedence order and fallback rules for Linux installer detection |
| Detection rules that produce `os_release.id` and `os_release.id_like` input values | `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md` | exact parsing and normalization rules for `/etc/os-release` inputs |
| Existing `host_state.group` preservation during writes | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md` | merge rule and no-regression requirement |
| Existing `host_state.linger` preservation during writes | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md` | merge rule and no-regression requirement |
| `schema_version` invariance (`1`) | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md` | explicit no-bump rule |
| Behavior when the existing install-state file is corrupt or on the wrong schema version | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md` | exact reset/merge/replace policy |
| Backward compatibility for older uninstall consumers | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md` | unknown-key tolerance and no-break guarantee |
| Linux/macOS/Windows divergence contract | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/platform-parity-spec.md` | Linux behavior delta and explicit no-delta rules for macOS/Windows |
| Required A/B decisions for ADR-0032 ambiguity removal | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` | DR-0001, DR-0002, DR-0003 options and selections |
| Slice acceptance: schema persistence | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md` | observable outcomes for additive platform fields |
| Slice acceptance: reliable file creation/update | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md` | observable outcomes for no-event installs and merge behavior |
| Slice acceptance: dev-installer parity | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM3/PDLDPM3-spec.md` | observable outcomes for keeping one install-state contract across both installers |
| Slice acceptance: smoke validation | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md` | observable outcomes for Linux smoke coverage |
| Execution sequencing and validation commands | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md` | slice order and required validation commands |
| Task graph and automation metadata | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json` | triad tasks, deps, references, and orchestration branch |
| Exact touched paths and external-doc reconciliation | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md` | edit allowlists and non-authoritative doc update targets |

## Determinism checklist (must be satisfied before quality gate)

Every selected doc MUST define its inputs, defaults, absence semantics, error posture, compatibility rules, and platform guarantees for the surfaces it owns.

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`

MUST define:
- the resolved install-state path and the write boundary under `SUBSTRATE_HOME`
- the exact successful-install write contract for Linux
- the exact no-change contract for macOS and Windows
- one exact rule for `--no-world`
- one exact rule for dry-run
- one exact rule for metadata-write failure posture
- one exact rule for downstream read precedence
- exit-code posture aligned to `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`

MUST define:
- the exact object path `host_state.platform`
- exact field names, types, and omission rules for `os_release.id`, `os_release.id_like`, `pkg_manager.selected`, and `pkg_manager.source`
- exact sample payloads for readable and unreadable `/etc/os-release` cases
- exact preservation rule for `host_state.group` and `host_state.linger` content in emitted JSON

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md`

MUST define:
- `schema_version` remains `1`
- exact behavior for pre-existing files with missing fields, corrupt JSON, or mismatched schema versions
- exact unknown-key tolerance rule for older uninstall consumers
- additive-only evolution rule for this schema extension

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/platform-parity-spec.md`

MUST define:
- Linux as the only platform with a behavior delta
- macOS and Windows as explicit no-delta platforms for this feature
- required validation evidence for each platform statement

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`

MUST define:
- DR-0001: exact persistence behavior when platform metadata inputs are incomplete
- DR-0002: exact dry-run persistence behavior
- DR-0003: exact installer-entrypoint scope
- exactly two options (A/B) and one selection for each DR

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md`

MUST define:
- the exact keys that this slice writes
- the exact acceptance criteria for recording installer-selected manager metadata
- the exact acceptance criteria for recording `/etc/os-release` metadata without redefining detection semantics

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`

MUST define:
- the exact acceptance criteria for file creation when no group/linger events occurred
- the exact acceptance criteria for idempotent updates and merge preservation
- the exact acceptance criteria for the selected `--no-world` and dry-run rules

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM3/PDLDPM3-spec.md`

MUST define:
- the exact acceptance criteria for dev-installer parity with the selected install-state contract
- the exact acceptance criteria for keeping one `install_state.json` meaning across both installer entrypoints
- the exact validation target path that owns dev-installer parity assertions

### `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`

MUST define:
- the exact smoke assertions for persisted platform keys
- the exact smoke assertions for compatibility invariants
- the exact smoke target path that owns these assertions

## Follow-ups

1. ADR feature-directory paths conflict with the dispatcher-selected feature directory.
   - Conflict: ADR-0032 points at `docs/project_management/packs/draft/stashing-ferret/`, while this run is authoritative for `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`.
   - Required fix: `impact_map.md` and `plan.md` MUST use `persist-detected-linux-distro-pkg-manager`, and the ADR MUST be reconciled in a separate allowed change before quality gate.

2. Dry-run persistence semantics are not defined by ADR-0032.
   - Gap: ADR-0032 defines successful-install persistence but does not state whether `--dry-run` writes `install_state.json`.
   - Required fix: `decision_register.md` DR-0002 and `contract.md` MUST select and state one exact dry-run rule.

3. Installer-entrypoint scope is not pinned.
   - Gap: the ADR names `scripts/substrate/install-substrate.sh`, while repository code and docs already expose the same metadata surface through `scripts/substrate/dev-install-substrate.sh`.
   - Required fix: `decision_register.md` DR-0003, `impact_map.md`, and `plan.md` MUST select one exact scope and list the touched paths that follow from that scope.

4. Dependency-owned detection semantics must stay external.
   - Gap: ADR-0032 stores `pkg_manager.selected` and `pkg_manager.source`, but the detection algorithm and enum semantics belong to `best-effort-distro-package-manager`.
   - Required fix: `contract.md` and `install-state-schema-spec.md` MUST link to the external detection docs and MUST NOT restate or override the detection algorithm.
