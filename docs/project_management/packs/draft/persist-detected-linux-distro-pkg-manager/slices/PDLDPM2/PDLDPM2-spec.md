# PDLDPM2-spec — Linux smoke coverage and operator evidence for persisted platform metadata

## Behavior delta (single)
- Existing: `tests/installers/install_state_smoke.sh` and `docs/INSTALLATION.md` cover install-state group and linger metadata, but they do not yet lock the Linux evidence for no-event file creation, `host_state.platform.*` persistence, missing `/etc/os-release` degradation, or additive `schema_version = 1` compatibility.
- New: the metadata smoke scenario and the installation guide become the required evidence surfaces for this pack, asserting canonical-file writes on successful Linux installs, exact `host_state.platform.*` presence or absence, missing-`/etc/os-release` degradation, and operator-visible path and write-matrix wording.
- Why: the pack is only complete when the persistence contract is proven by Linux smoke coverage and described consistently for operators without reopening the contract or schema boundaries.

## Scope
- Define the exact Linux assertions that `tests/installers/install_state_smoke.sh --scenario metadata` must enforce for this pack.
- Define the operator-facing reconciliation required in `docs/INSTALLATION.md` for install-state path wording, field naming, and persistence-branch behavior.
- Define the validation artifact that `plan.md` must carry for this slice.
- Keep payload shape, write semantics, and vocabulary authority delegated to the contract and schema docs that already own those surfaces.

## Inputs (authoritative)
- Installer-facing path rule, write matrix, warning-only failure posture, and future-consumer read precedence:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- Exact `install_state.json` field paths, absence semantics, and additive merge rules:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- Upstream authority for Linux distro parsing, package-manager selection, and `pkg_manager.source` values:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- Required touched surfaces and slice ownership for smoke coverage plus operator evidence:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/ci_checkpoint_plan.md`

## Behavior (authoritative)

### Linux metadata smoke assertions
- `tests/installers/install_state_smoke.sh --scenario metadata` MUST remain the Linux behavior-smoke entrypoint for this slice.
- The `metadata` scenario MUST include a successful hosted Linux install case where no `host_state.group` delta and no `host_state.linger` delta are produced for the run. That case MUST still create `<prefix>/install_state.json` and MUST assert:
  - `schema_version = 1`
  - `host_state.platform.os_release.id = ubuntu`
  - `host_state.platform.os_release.id_like = debian`
  - `host_state.platform.pkg_manager.selected = apt-get`
  - `host_state.platform.pkg_manager.source = os_release`
- The no-event hosted Linux case MUST derive those values from an os-release fixture that maps to the Debian or Ubuntu family plus a controlled `PATH` that contains `apt-get`.
- The `metadata` scenario MUST include a successful hosted Linux `--no-world` case that writes the same canonical file under `<prefix>/install_state.json` and proves that `--no-world` does not suppress metadata persistence.
- The `metadata` scenario MUST include a missing-`/etc/os-release` case by setting `SUBSTRATE_INSTALL_OS_RELEASE_PATH` to a non-existent path and presenting a controlled `PATH` that contains `apt-get`. That case MUST assert:
  - installer exit remains success
  - `<prefix>/install_state.json` exists
  - `host_state.platform.os_release.id` is absent
  - `host_state.platform.os_release.id_like` is absent
  - `host_state.platform.pkg_manager.selected = apt-get`
  - `host_state.platform.pkg_manager.source = path_probe`
- The `metadata` scenario MUST include an additive-compatibility merge case that seeds `<prefix>/install_state.json` with an existing `schema_version = 1` document containing `host_state.group`, `host_state.linger`, and at least one unknown sibling key under the preserved object graph. After a successful Linux persistence run, that case MUST assert:
  - `schema_version` remains `1`
  - pre-existing `host_state.group` content remains intact
  - pre-existing `host_state.linger` content remains intact
  - the unknown sibling key remains intact
  - the owned `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*` fields are present with current-run values
- The metadata scenario MUST validate additive v1 compatibility, not a schema bump. A passing run for this slice is preservation plus additive `host_state.platform.*`, not replacement with a new schema version.

### Operator evidence in `docs/INSTALLATION.md`
- The `### Installer Metadata & Cleanup` section MUST state that `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` share one metadata producer contract.
- That section MUST identify the canonical metadata file as `<effective install prefix>/install_state.json`, and it MUST state that operators may refer to that same path as `$SUBSTRATE_HOME/install_state.json`.
- That section MUST use the exact field name `schema_version = 1`.
- That section MUST name the four additive platform fields exactly:
  - `host_state.platform.os_release.id`
  - `host_state.platform.os_release.id_like`
  - `host_state.platform.pkg_manager.selected`
  - `host_state.platform.pkg_manager.source`
- That section MUST state the accepted Linux write matrix in prose:
  - successful Linux install persists metadata
  - successful Linux install with `--no-world` persists metadata
  - any installer run with `--dry-run` performs no metadata write
- That section MUST state the missing-`/etc/os-release` degradation rule in prose: absent os-release values leave only `host_state.platform.os_release.*` absent, while package-manager metadata remains eligible for persistence when selection succeeds.
- `docs/INSTALLATION.md` MUST summarize the behavior without duplicating package-manager vocabulary tables or schema tables that are owned by the pack contract docs.

### Validation evidence and plan handoff
- `plan.md` MUST list `bash tests/installers/install_state_smoke.sh --scenario metadata` as the required Linux validation command for this slice.
- The required validation artifact for this slice is the smoke-harness pass result from that command; no separate manual playbook belongs to this slice.
- This slice traces evidence only. It MUST NOT redefine field paths, merge semantics, source vocabulary, temp-file replacement, or exit-code policy that are already owned by `contract.md`, `install-state-schema-spec.md`, and the upstream detection contract.

## Acceptance criteria
- AC-PDLDPM2-01: `tests/installers/install_state_smoke.sh --scenario metadata` includes a successful hosted Linux no-event case where no group or linger delta occurs, yet `<prefix>/install_state.json` is created and records `schema_version = 1`, `host_state.platform.os_release.id = ubuntu`, `host_state.platform.os_release.id_like = debian`, `host_state.platform.pkg_manager.selected = apt-get`, and `host_state.platform.pkg_manager.source = os_release`.
- AC-PDLDPM2-02: `tests/installers/install_state_smoke.sh --scenario metadata` includes a successful hosted Linux `--no-world` case that writes `<prefix>/install_state.json` and proves the persisted metadata contract remains active when world provisioning is skipped.
- AC-PDLDPM2-03: `tests/installers/install_state_smoke.sh --scenario metadata` includes a missing-`/etc/os-release` case that exits successfully, leaves `host_state.platform.os_release.id` and `host_state.platform.os_release.id_like` absent, and still records `host_state.platform.pkg_manager.selected = apt-get` with `host_state.platform.pkg_manager.source = path_probe`.
- AC-PDLDPM2-04: `tests/installers/install_state_smoke.sh --scenario metadata` includes an additive-compatibility merge case that starts from a `schema_version = 1` file, preserves pre-existing `host_state.group`, preserves pre-existing `host_state.linger`, preserves at least one unknown sibling key, and adds or updates only the owned `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*` fields.
- AC-PDLDPM2-05: `docs/INSTALLATION.md` `### Installer Metadata & Cleanup` states that both installers share one metadata contract, uses the canonical path wording `<effective install prefix>/install_state.json` plus the operator-facing alias `$SUBSTRATE_HOME/install_state.json`, and uses the exact field name `schema_version = 1`.
- AC-PDLDPM2-06: `docs/INSTALLATION.md` `### Installer Metadata & Cleanup` names `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source`, states that successful Linux `--no-world` installs still persist metadata, and states that `--dry-run` performs no metadata write.
- AC-PDLDPM2-07: `docs/INSTALLATION.md` `### Installer Metadata & Cleanup` states the missing-`/etc/os-release` degradation rule in prose without duplicating package-manager vocabulary tables or schema tables, and `plan.md` lists `bash tests/installers/install_state_smoke.sh --scenario metadata` as the required Linux validation command with the smoke pass result as the required evidence artifact.

## Out of scope
- Defining new `install_state.json` field paths, merge rules, or package-manager vocabulary values.
- Defining temp-file replacement mechanics, idempotent write implementation details, or warning text details for installer failures.
- Adding new smoke harness entrypoints outside `tests/installers/install_state_smoke.sh` or expanding this slice into uninstaller cleanup behavior.
