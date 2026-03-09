# PDLDPM2-spec - Lock installer smoke coverage and operator evidence

## Behavior delta (single)
- Existing: `tests/installers/install_state_smoke.sh` proves current `install_state.json` creation and upgrade behavior for `host_state.group` and `host_state.linger`, plus uninstall cleanup fallback, but it does not yet lock no-event Linux writes, persisted `host_state.platform.*` assertions, missing `/etc/os-release` degradation, or the operator-facing wording that explains the new metadata.
- New: This slice requires Linux installer smoke coverage and operator-facing evidence that prove successful Linux installs create or update `<effective_prefix>/install_state.json`, persist the additive `host_state.platform.*` block under `schema_version = 1`, preserve older-file compatibility, and keep `docs/INSTALLATION.md` aligned with that contract.
- Why: The pack needs one end-of-feature evidence surface that catches drift between installer behavior, persisted JSON, and operator documentation.

## Scope
- Define the exact Linux smoke assertions that `tests/installers/install_state_smoke.sh` must enforce for no-event success, persisted platform fields, missing `/etc/os-release` degradation, and additive compatibility.
- Define the exact operator-facing statements that `docs/INSTALLATION.md` must carry for the canonical metadata path, shared hosted-plus-dev producer scope, and accepted persisted field names.
- Define the validation evidence that `plan.md` must require for this slice.
- Keep installer write semantics and schema ownership with `contract.md`, `install-state-schema-spec.md`, `PDLDPM0`, and `PDLDPM1`.

## Inputs (authoritative)
- Operator-facing producer contract and failure posture: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- Schema paths, additive compatibility, and canonical examples: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- Reliable write semantics and no-write branches: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`
- Smoke and doc touch-set expectations: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
- Required `PDLDPM2` scope and evidence surfaces: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`
- Existing smoke harness surface: `tests/installers/install_state_smoke.sh`
- Existing operator doc surface: `docs/INSTALLATION.md`

## Behavior (authoritative)
### Slice boundary and authority
- This slice owns smoke acceptance and documentation evidence only.
- This slice MUST link to `contract.md` and `install-state-schema-spec.md` and MUST NOT redefine canonical file-path resolution, temp-file replacement semantics, supported manager vocabulary, or the exact JSON field contract.
- The persisted field meanings and merge rules remain owned by `install-state-schema-spec.md` and `PDLDPM0`; the successful-install write matrix and dry-run or no-write behavior remain owned by `contract.md` and `PDLDPM1`.

### Required smoke assertions in `tests/installers/install_state_smoke.sh`
- The harness MUST continue to run hermetically on Linux by stubbing privileged and systemd commands under a temp root; the test contract remains no host mutation.
- A no-event Linux success branch MUST start from an absent canonical file plus a host state where the installer has no new `host_state.group` or `host_state.linger` delta to record, run a successful install, and assert that `<effective_prefix>/install_state.json` now exists with `schema_version = 1`. This branch MUST treat absence of freshly emitted `host_state.group` or `host_state.linger` entries as allowed; file creation is the required invariant.
- A persisted-platform branch MUST assert that a successful Linux install writes exactly `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source`, using the concrete detector outputs supplied to the harness and no alternate nesting.
- A missing-`/etc/os-release` branch MUST simulate the selected os-release file being missing or unreadable, assert the installer still exits successfully, and assert that `host_state.platform.os_release.id` and `host_state.platform.os_release.id_like` persist the literal `<unknown>` sentinel while `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` remain present from the surviving detection inputs.
- An additive-compatibility branch MUST start from a readable `schema_version = 1` file that already contains `host_state.group`, `host_state.linger`, and at least one unknown key, rerun a successful Linux install, and assert that the rewrite preserves those legacy and unknown values while refreshing only the accepted `host_state.platform.*` block.
- The smoke contract for this slice is Linux behavior only. Cross-platform parity for this feature remains compile and documentation parity rather than new runtime assertions on macOS or Windows.

### Validation evidence requirements
- `plan.md` MUST require the smoke command set that covers all four branches above. If `bash tests/installers/install_state_smoke.sh --scenario metadata` covers no-event success, persisted-platform assertions, missing `/etc/os-release`, and additive compatibility, that one command is sufficient. If the harness keeps those branches in separate scenarios, `plan.md` MUST list each scenario command explicitly instead of relying on an opaque `all` aggregate.
- The recorded evidence for this slice MUST be the smoke-command exit status plus assertion output from the harness; this slice does not require a separate manual playbook.
- The same validation evidence MUST show that the harness writes and reads `<effective_prefix>/install_state.json`, not a second metadata file or a HOME-only alias with independent semantics.

### Operator-facing evidence in `docs/INSTALLATION.md`
- `docs/INSTALLATION.md` MUST state that `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` share one Linux metadata-producer contract.
- The doc MUST describe `<effective_prefix>/install_state.json` as the on-disk canonical path and `$SUBSTRATE_HOME/install_state.json` as the operator-facing alias when the default prefix is used.
- The doc MUST state that successful Linux installs, including successful `--no-world` runs, create or update that file, while hosted installer `--dry-run` remains a no-write branch.
- The doc MUST use the field name `schema_version = 1` and MUST name the additive persisted fields `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source`.
- The doc MUST describe metadata persistence as additive and warning-only: missing or unreadable metadata inputs do not fail an otherwise successful install, and this feature does not redefine package-manager selection semantics owned by `best-effort-distro-package-manager`.

## Acceptance criteria
- AC-PDLDPM2-01: `tests/installers/install_state_smoke.sh` contains a Linux no-event success assertion that starts from no canonical file, records no new `host_state.group` or `host_state.linger` delta, exits successfully, and proves `<effective_prefix>/install_state.json` is still created with `schema_version = 1`.
- AC-PDLDPM2-02: The smoke harness asserts that a successful Linux install persists exactly `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source` under the accepted nesting, with values matching the harness-supplied detector outputs and no alternate field layout.
- AC-PDLDPM2-03: The smoke harness includes a missing-`/etc/os-release` branch that exits successfully and proves the resulting file keeps `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` while persisting literal `<unknown>` values for `host_state.platform.os_release.id` and `host_state.platform.os_release.id_like`.
- AC-PDLDPM2-04: The smoke harness includes an additive-compatibility branch that begins from a readable `schema_version = 1` file with existing `host_state.group`, `host_state.linger`, and unknown keys, reruns a successful Linux install, and proves the rewritten file stays on `schema_version = 1` while preserving those legacy and unknown values.
- AC-PDLDPM2-05: `plan.md` requires the exact smoke command set that covers no-event success, persisted-platform assertions, missing `/etc/os-release`, and additive compatibility, and the required recorded evidence for this slice is the smoke harness output rather than a separate manual validation playbook.
- AC-PDLDPM2-06: `docs/INSTALLATION.md` states that both installers write `<effective_prefix>/install_state.json` on successful Linux installs, including successful `--no-world` runs, uses `schema_version = 1`, names the four `host_state.platform.*` fields, and explains `$SUBSTRATE_HOME/install_state.json` as the operator-facing alias for the default-prefix path.

## Out of scope
- Redefining the canonical metadata path, the successful-install write matrix, temp-file replacement, or dry-run and no-write behavior remains owned by `contract.md` and `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM1/PDLDPM1-spec.md`.
- Redefining the `host_state.platform.*` field paths, merge rules, `<unknown>` sentinel semantics, or supported `pkg_manager.source` vocabulary remains owned by `install-state-schema-spec.md`, `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md`, and `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`.
- Uninstaller `--cleanup-state` behavior and its existing smoke assertions remain outside this slice unless a later planning decision explicitly expands the installer-metadata evidence boundary.
