# BEDPM0-spec - Lock distro detection, mapping, and decision-line reporting

## Behavior delta (single)
- Existing: `scripts/substrate/install-substrate.sh` picks a supported package manager from host state with no stable Linux distro-detection contract, no fixed `<unknown>` sentinel for missing os-release data, and no single operator-visible decision line that names the selected manager source.
- New: `scripts/substrate/install-substrate.sh` reads the selected os-release input safely, derives normalized `distro_id` and `distro_id_like`, applies the accepted distro-family mapping table, and emits the stable stderr decision line exactly once before prerequisite installation when this slice produces a concrete manager selection.
- Why: downstream slices and docs need one stable detection vocabulary and one stable reporting point before override handling, wrapper parity, and validation evidence can stay aligned.

## Scope
- Constrain Linux hosted-installer behavior for reading os-release input, normalizing `ID` and `ID_LIKE`, applying distro-family mapping, and rendering the stable decision line.
- Keep the missing-data sentinel fixed at `<unknown>` for unreadable, missing, or invalid alternate os-release inputs.
- Keep explicit override precedence, invalid override failures, no-supported-manager exit handling, wrapper pass-through, and downstream documentation propagation outside this slice.

## Inputs (authoritative)
- Operator-facing detection, mapping, `<unknown>`, and decision-line contract: `docs/project_management/packs/implemented/best-effort-distro-package-manager/contract.md`
- Accepted parser and mapping decisions: `docs/project_management/packs/implemented/best-effort-distro-package-manager/decision_register.md` (`DR-0001`, `DR-0002`, `DR-0003`)
- Slice boundary and required assertions: `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
- Implementation surfaces constrained by this slice:
  - `scripts/substrate/install-substrate.sh`

## Behavior (authoritative)
### Detection input and normalization
- This slice inherits the exact os-release parsing and alternate-path rules from `contract.md` and uses them without restating the parser table.
- The installer reads only the selected os-release input path, extracts normalized lowercase `distro_id` and `distro_id_like`, and uses the literal `<unknown>` sentinel when the selected input is unavailable or yields no accepted value for that field.
- Detection remains Linux-only, local-only, and free of network calls.
- Detection reads data only; it does not own override parsing, wrapper behavior, or remediation text.

### Distro-family mapping stage
- The mapping stage evaluates the normalized values against the contract-owned family table for Debian or Ubuntu, Fedora or RHEL, Arch, and SUSE families.
- A family match selects a manager only when the contract-owned manager for that family is present in `PATH`.
- Fedora or RHEL family selection prefers `dnf`; when `dnf` is absent and `yum` is present, the stage selects `yum`.
- When the family table produces a concrete selection, the installer records `pkg_manager.source=os_release`.
- When no family entry produces an available manager, control falls through to the later `PATH` probe stage owned by `BEDPM1`.

### Stable decision-line reporting
- When this slice produces a concrete manager selection, the installer emits the contract-owned decision-line template to stderr exactly once.
- The decision line uses the normalized `distro_id`, normalized `distro_id_like`, selected manager spelling, and `pkg_manager.source=os_release`.
- The decision line appears after manager selection is complete and before any package-manager install command begins.
- This slice does not redefine decision-line suppression for failure exits owned by `BEDPM1`.

## Acceptance criteria
- AC-BEDPM0-01: With a readable alternate os-release input whose normalized `ID` or `ID_LIKE` maps to the Debian or Ubuntu family and with `apt-get` present in `PATH`, `scripts/substrate/install-substrate.sh` selects `apt-get`, records `pkg_manager.source=os_release`, and emits the stable decision line with the normalized distro fields.
- AC-BEDPM0-02: With a readable alternate os-release input whose normalized `ID` or `ID_LIKE` maps to the Arch family and with `pacman` present in `PATH`, the installer selects `pacman`, records `pkg_manager.source=os_release`, and emits the stable decision line with the normalized distro fields.
- AC-BEDPM0-03: With a readable alternate os-release input whose normalized `ID` or `ID_LIKE` maps to the Fedora or RHEL family, the installer selects `dnf` when `dnf` is present in `PATH`; when `dnf` is absent and `yum` is present, the same branch selects `yum`, and both outcomes keep `pkg_manager.source=os_release`.
- AC-BEDPM0-04: When the selected os-release input is missing, unreadable, or an invalid alternate path chosen by `SUBSTRATE_INSTALL_OS_RELEASE_PATH`, the installer renders `distro_id=<unknown>` and `distro_id_like=<unknown>`, does not read `/etc/os-release` as a second fallback, and continues to the later selection stages instead of failing in this slice.
- AC-BEDPM0-05: For every concrete manager selection produced by this slice, the stable decision line appears on stderr exactly once after selection completes and before prerequisite installation begins; the implementation does not emit a second decision line for the same run.

## Out of scope
- Parsing `--pkg-manager`, honoring `PKG_MANAGER`, fixed `PATH` probe order, multi-manager warnings, and exits `2`, `3`, or `4` remain owned by `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`.
- Wrapper exit-status pass-through in `scripts/substrate/install.sh` and propagation into `docs/INSTALLATION.md` and `docs/reference/env/contract.md` remain owned by `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md`.
- Hermetic repo-test assertions, the thin Linux smoke wrapper, and manual evidence capture remain owned by `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM3/BEDPM3-spec.md`.
