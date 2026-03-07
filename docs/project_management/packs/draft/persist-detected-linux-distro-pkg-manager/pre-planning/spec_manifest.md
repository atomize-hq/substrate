# persist-detected-linux-distro-pkg-manager — spec manifest (pre-planning)

This file enumerates every contract, schema, path, compatibility, and validation surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
- Upstream contracts and touched references that inform this manifest but are not owned by this pack:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` — authoritative for Linux installer distro/package-manager detection, selected manager vocabulary, and `pkg_manager.source` vocabulary
  - `docs/INSTALLATION.md` — touched operator doc that already documents `install_state.json`
  - `scripts/substrate/install-substrate.sh` — current hosted-installer implementation
  - `scripts/substrate/dev-install-substrate.sh` — current dev-installer implementation
  - `tests/installers/install_state_smoke.sh` — current installer smoke harness

## Slice IDs (canonical)

Slice IDs MUST be feature-derived and stable per:
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`

Canonical slice IDs selected for this feature:
- Slice prefix: `PDLDPM` (derived from `persist-detected-linux-distro-pkg-manager`)
- `PDLDPM0` — persist `host_state.platform.*` metadata fields
- `PDLDPM1` — make `install_state.json` reliably present on successful Linux installs
- `PDLDPM2` — lock smoke coverage and validation evidence for persisted metadata

## Required spec documents (authoritative)

This ADR requires one user-facing contract doc, one schema/compatibility doc, one decision register, one impact map, one execution plan, and three canonical slice specs.

No separate protocol, policy, telemetry, env-vars, filesystem-semantics, platform-parity, or standalone compatibility doc is selected.
- This ADR introduces no wire or IPC contract.
- This ADR introduces no new policy rule.
- This ADR introduces no new log or trace field.
- This ADR introduces no new environment variable.
- Path semantics and platform guarantees are owned by `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`.
- JSON schema and additive compatibility are owned by `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`.

### Planning pack scaffolding (required)

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md` (this file)
  - Owns (authoritative):
    - the exact required-doc set for this feature directory
    - the surface-to-doc ownership map
    - the follow-ups required to remove ADR ambiguity before quality gate
  - Must define:
    - a surface-complete coverage matrix with exactly one owner per surface
    - the canonical slice IDs and canonical slice spec paths
    - the explicit statement that unselected doc classes stay unselected for this ADR
  - Links (non-authoritative):
    - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
  - Owns (authoritative):
    - the exact create/edit touch set for this feature
    - the cascading implications and cross-pack conflicts
  - Must define:
    - the exact touched implementation paths for the selected slices, including the hosted installer path, the smoke harness path, and every operator doc path that must change
    - the dependency boundary with `docs/project_management/packs/draft/best-effort-distro-package-manager/`
    - whether dev-installer and uninstaller paths are in scope edits or explicit follow-up surfaces
    - the exact operator-doc reconciliation set for `docs/INSTALLATION.md`
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`
  - Owns (authoritative):
    - the execution order for `PDLDPM0`, `PDLDPM1`, and `PDLDPM2`
    - the required validation commands and evidence expectations
  - Must define:
    - the slice order `PDLDPM0` → `PDLDPM1` → `PDLDPM2`
    - the exact validation commands for installer smoke coverage and any targeted test commands
    - the exact rule for manual validation artifacts for this pack:
      - no manual playbook is required for this ADR because smoke coverage is the required validation artifact
    - the dependency on the detection contract owned by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`
    - `tests/installers/install_state_smoke.sh`

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json` (already exists)
  - Owns (authoritative):
    - the triad task graph and automation metadata for this pack
  - Must define:
    - triad task IDs and dependencies for:
      - `PDLDPM0-code`, `PDLDPM0-test`, `PDLDPM0-integ`
      - `PDLDPM1-code`, `PDLDPM1-test`, `PDLDPM1-integ`
      - `PDLDPM2-code`, `PDLDPM2-test`, `PDLDPM2-integ`
    - references to the canonical slice spec paths under `slices/PDLDPM*/`
    - the orchestration branch `feat/persist-detected-linux-distro-pkg-manager`
    - acceptance-criteria traceability to the slice specs
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`

### Feature contract, schema, and decisions (required)

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
  - Owns (authoritative):
    - the user-facing installer contract for this feature
    - config/path semantics for the metadata file location
    - platform guarantees, failure posture, and read semantics for later consumers
  - Must define:
    - the exact CLI delta: this ADR introduces no new commands and no new flags
    - the exact exit-code delta: installer exit-code behavior does not change for successful runs, and metadata-persistence failures do not introduce a new exit-code path
    - the canonical metadata path rule:
      - the installer write path is derived from the effective install prefix
      - the post-install user-facing form of that same path is `$SUBSTRATE_HOME/install_state.json`
      - the default prefix is `~/.substrate`
    - the Linux-only behavior contract:
      - successful Linux installs create or update `install_state.json`
      - macOS and Windows do not gain new `host_state.platform.*` metadata writes from this ADR
    - the exact best-effort failure posture:
      - missing `/etc/os-release` does not fail install
      - metadata read or write failure does not flip an otherwise successful install into failure
      - metadata write is skipped during dry-run
    - the future-consumer read contract:
      - consumers prefer persisted metadata for guidance strings
      - consumers fall back to runtime detection when persisted metadata is missing or unreadable
    - the protected-path invariant:
      - this ADR writes only under the effective Substrate home
    - the explicit statement that this ADR introduces no new environment variable and no new log or trace field
  - Links (non-authoritative):
    - `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
  - Owns (authoritative):
    - the `install_state.json` schema boundary touched by this ADR
    - the additive compatibility rules for the new metadata fields
  - Must define:
    - the exact top-level version invariant:
      - field name: `schema_version`
      - value: `1`
      - this ADR does not change that value
    - the exact existing-structure preservation rule for:
      - `host_state.group`
      - `host_state.linger`
    - the exact new field paths, types, and absence semantics:
      - `host_state.platform.os_release.id`
      - `host_state.platform.os_release.id_like`
      - `host_state.platform.pkg_manager.selected`
      - `host_state.platform.pkg_manager.source`
    - the exact storage rules for those fields:
      - `os_release.id` stores the detected `ID` string when available
      - `os_release.id_like` stores the detected `ID_LIKE` raw string when available
      - `pkg_manager.selected` stores the selected manager string emitted by the detection contract
      - `pkg_manager.source` stores the source string emitted by the detection contract
    - the additive compatibility policy:
      - existing consumers that ignore unknown keys remain compatible
      - this ADR does not rename or remove existing fields
      - this ADR does not require a migration or backfill outside normal successful-install writes
    - the canonical JSON examples for:
      - an existing file upgraded with the new `host_state.platform.*` block
      - a Linux install with missing `/etc/os-release`
      - a Linux install with no group or linger deltas
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`
  - Owns (authoritative):
    - the A/B decisions required to remove ambiguity from ADR-0032
  - Must define:
    - DR-0001: persistence location contract
      - option A: extend `install_state.json`
      - option B: write a separate metadata file
      - one selected option
    - DR-0002: field naming and nesting contract under `host_state.platform.*`
      - option A and option B with one selected option
    - DR-0003: selected-manager and source-vocabulary ownership
      - option A: duplicate the vocabulary locally
      - option B: treat `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` as the single source of truth and copy its emitted strings verbatim
      - one selected option
    - DR-0004: successful-install write-trigger scope
      - option A and option B with one selected option for hosted install, hosted `--no-world`, dev install, and dev `--no-world`
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`

### Slice specs (required)

Slice specs MUST use the canonical layout:
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/<SLICE_ID>/<SLICE_ID>-spec.md`

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md`
  - Owns (authoritative):
    - the `PDLDPM0` slice scope and acceptance criteria for persisting the new metadata fields
  - Must define:
    - the exact implementation boundary for storing `host_state.platform.*`
    - the exact inputs it consumes from the detection contract and from `/etc/os-release`
    - the exact acceptance criteria that prove:
      - the four new field paths are written under the correct JSON nesting
      - `pkg_manager.selected` and `pkg_manager.source` are copied from the detection contract without local re-derivation
      - missing `/etc/os-release` does not block persistence of package-manager metadata when that metadata exists
      - existing `host_state.group` and `host_state.linger` data remain intact after the write
    - the contract-link rule: this slice spec links to `contract.md` and `install-state-schema-spec.md` and does not redefine the operator-facing contract
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
    - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`
  - Owns (authoritative):
    - the `PDLDPM1` slice scope and acceptance criteria for reliable file creation and update semantics
  - Must define:
    - the exact success branches that trigger a write or update
    - the exact no-write branches, including dry-run and non-Linux installs
    - the exact acceptance criteria that prove:
      - `install_state.json` is created on successful Linux installs even when there are no group or linger deltas
      - repeated successful installs are idempotent at the schema level
      - metadata write failure degrades with warning-only behavior instead of flipping install success
      - the selected write-trigger rule from `decision_register.md` is enforced for hosted install, hosted `--no-world`, dev install, and dev `--no-world`
    - the exact file-replacement rule used by the implementation, including any temp-file path and replace step
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`

- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`
  - Owns (authoritative):
    - the `PDLDPM2` slice scope and acceptance criteria for smoke coverage and validation evidence
  - Must define:
    - the exact required assertions in `tests/installers/install_state_smoke.sh`
    - the exact acceptance criteria that prove:
      - successful Linux install writes `install_state.json`
      - the new `host_state.platform.*` keys exist when the required inputs exist
      - missing `/etc/os-release` does not fail install and still records the package-manager metadata that remains available
      - older consumers remain compatible because the file stays on `schema_version = 1`
    - the exact validation evidence required by `plan.md`
    - the explicit statement that this slice does not redefine the contract or schema surfaces owned by `contract.md` and `install-state-schema-spec.md`
  - Links (non-authoritative):
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
    - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
    - `tests/installers/install_state_smoke.sh`

## Coverage matrix (surface → authoritative doc)

Every surface that ADR-0032 touches appears here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| CLI delta: no new commands or flags | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | exact “no new CLI surface” statement |
| Exit-code delta: no new success or failure exit-code class for metadata persistence | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | taxonomy reference and the no-change rule |
| Metadata file path resolution (`--prefix`, default `~/.substrate`, user-facing `$SUBSTRATE_HOME`) | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | one canonical path rule and precedence |
| Linux-only write guarantee | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | exact Linux behavior and exact no-change statement for macOS and Windows |
| Successful-install write trigger and dry-run no-write rule | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | exact write and no-write branches |
| Metadata read/write failure posture | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | warning-only degrade behavior and non-failing install rule |
| Future-consumer read precedence | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | prefer persisted metadata, fall back to runtime detection |
| Protected-path invariant | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md` | write boundary stays under the effective Substrate home |
| `install_state.json` top-level version field | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | field name, fixed value, and no-version-bump rule |
| Existing `host_state.group` and `host_state.linger` preservation | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | merge and preservation rules |
| `host_state.platform.os_release.id` | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | path, type, source, and absence semantics |
| `host_state.platform.os_release.id_like` | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | path, type, raw-string rule, and absence semantics |
| `host_state.platform.pkg_manager.selected` persisted field | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | path, type, and copy-through storage rule |
| `host_state.platform.pkg_manager.source` persisted field | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | path, type, and copy-through storage rule |
| Additive compatibility and unknown-key tolerance | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md` | additive-only rule, old-consumer compatibility, no field removal |
| Selected manager vocabulary and selection algorithm | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact manager strings and exact selection semantics |
| `pkg_manager.source` vocabulary and emission rules | `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` | exact source strings and exact source-selection semantics |
| Installer-scope boundary for metadata persistence | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` | one selected scope rule across hosted installer and dev installer |
| Slice acceptance for field persistence | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md` | implementation scope and acceptance criteria |
| Slice acceptance for reliable file creation/update | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md` | implementation scope and acceptance criteria |
| Smoke and validation acceptance | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md` | exact assertions and evidence requirements |
| Slice sequencing and validation commands | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md` | exact slice order and exact validation commands |
| Task graph and automation metadata | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json` | exact task IDs, dependencies, and references |
| Touch set, operator-doc reconciliation, and cross-pack conflicts | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md` | explicit touched paths, dependency notes, and conflict handling |
| Required-doc set and ownership map | `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md` | complete doc list, ownership map, and follow-ups |

## Determinism checklist (must be satisfied before quality gate)

For every selected spec document, confirm it explicitly defines:
- Inputs and precedence order when multiple inputs exist
- Defaults and absence semantics
- The data model and constraints for every serialized boundary
- The error model and failure posture
- Ordering, idempotency, and atomic-write rules when writes occur
- Security and path invariants
- Platform guarantees for Linux, macOS, and Windows

## Follow-ups

1. ADR feature-path drift exists
   - Issue: `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md` still points at `docs/project_management/packs/draft/stashing-ferret/`, while orchestration resolved this feature to `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`.
   - Required fix: update ADR-0032 Scope and Related Docs to the resolved feature directory and align every selected spec path with that directory.

2. Prefix naming and `$SUBSTRATE_HOME` naming need one canonical rule
   - Issue: ADR-0032 names `$SUBSTRATE_HOME/install_state.json`, while the hosted installer currently writes to the effective `--prefix` path with default `~/.substrate`.
   - Required fix: `contract.md` must define one canonical equivalence rule, and `impact_map.md` must include every touched doc or script that still uses the alternate naming.

3. Installer scope is not pinned
   - Issue: `docs/INSTALLATION.md` states both installers record host-state details, while ADR-0032 architecture names only `scripts/substrate/install-substrate.sh`.
   - Required fix: `decision_register.md` and `contract.md` must select one installer-scope rule, and `impact_map.md` must reflect the resulting touch set.

4. Successful-install branches need exact write and no-write semantics
   - Issue: ADR-0032 says “successful Linux install” but does not enumerate hosted `--no-world`, dev-install `--no-world`, or `--dry-run`.
   - Required fix: `decision_register.md`, `contract.md`, and `slices/PDLDPM1/PDLDPM1-spec.md` must define the exact rule for each branch.

5. Operator-doc schema naming drift already exists
   - Issue: `docs/INSTALLATION.md` documents “Schema version = 1”, while ADR-0032 and the installer code use `schema_version = 1`.
   - Required fix: `install-state-schema-spec.md` must declare the authoritative field name, and `impact_map.md` must include the operator-doc reconciliation path.

6. Uninstaller path compatibility needs explicit review
   - Issue: install docs describe `<prefix>/install_state.json`, while `scripts/substrate/uninstall-substrate.sh` still reads `HOME/.substrate/install_state.json`.
   - Required fix: `impact_map.md` must record whether uninstaller path handling is an untouched follow-up surface or an in-scope dependency of this feature.
