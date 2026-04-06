# BEDPM2-spec - Preserve wrapper exits and propagate the operator contract

## Behavior delta (single)
- Existing: the quick-install wrapper path can hide feature-specific failure classes, and the operator-facing installation and env-contract docs do not yet guarantee reuse of the selected precedence, warning, remediation, and alternate os-release semantics from this pack.
- New: `scripts/substrate/install.sh` preserves the feature-specific upstream exit classes `0`, `2`, `3`, and `4`, while `docs/INSTALLATION.md` and `docs/reference/env/contract.md` reuse the accepted precedence chain, warning posture, remediation posture, and `SUBSTRATE_INSTALL_OS_RELEASE_PATH` semantics from `contract.md` with no vocabulary drift.
- Why: the documented wrapper entrypoint and the operator docs must expose the same contract the direct installer implements or the feature remains internally consistent only.

## Scope
- Constrain wrapper exit-status pass-through for the feature path in `scripts/substrate/install.sh`.
- Constrain operator-document propagation in `docs/INSTALLATION.md`.
- Constrain env-contract propagation in `docs/reference/env/contract.md`.
- Keep distro detection, override evaluation, and hermetic validation ownership outside this slice.

## Inputs (authoritative)
- Operator-facing wrapper, precedence, warning, remediation, and env-var contract: `docs/project_management/packs/implemented/best-effort-distro-package-manager/contract.md`
- Accepted wrapper and smoke-topology decisions: `docs/project_management/packs/implemented/best-effort-distro-package-manager/decision_register.md` (`DR-0003`, `DR-0004`, `DR-0005`)
- Slice boundary and required assertions: `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
- Implementation and documentation surfaces constrained by this slice:
  - `scripts/substrate/install.sh`
  - `docs/INSTALLATION.md`
  - `docs/reference/env/contract.md`

## Behavior (authoritative)
### Wrapper exit-status pass-through
- `scripts/substrate/install.sh` passes installer arguments through to `scripts/substrate/install-substrate.sh` for this feature path.
- For the explicit contract branches introduced by this pack, the wrapper preserves upstream exit codes `0`, `2`, `3`, and `4`.
- The wrapper does not collapse those feature-specific exits to `1`.
- This slice links to the exit-code meanings in `contract.md`; it does not create a second exit-code table.

### Operator-doc propagation in `docs/INSTALLATION.md`
- `docs/INSTALLATION.md` reuses the exact precedence chain, supported manager vocabulary, `pkg_manager.source` vocabulary, stable decision-line wording, multi-manager warning posture, and remediation posture defined in `contract.md`.
- The Linux quick-install narrative, offline-install narrative, and installer options table document `--pkg-manager`, `PKG_MANAGER`, the stable decision line, and wrapper exit-status pass-through for this feature.
- `docs/INSTALLATION.md` keeps macOS and Windows as explicit no-change platforms for ADR-0031.

### Env-contract propagation in `docs/reference/env/contract.md`
- `docs/reference/env/contract.md` documents `PKG_MANAGER` as a Linux hosted-installer override with the contract-owned precedence, allowed values, invalid-value posture, and missing-binary posture.
- `docs/reference/env/contract.md` documents `SUBSTRATE_INSTALL_OS_RELEASE_PATH` with the contract-owned absolute-path validation rule, unreadable-path semantics, no-fallback-to-`/etc/os-release` rule when set, and Linux-only scope.
- This slice reuses the exact hook names and behavior from `contract.md`; it does not introduce extra installer-local env variables or alternate vocabulary.

## Acceptance criteria
- AC-BEDPM2-01: For the feature-specific branches introduced by ADR-0031, `scripts/substrate/install.sh` preserves upstream exit classes `0`, `2`, `3`, and `4` from `scripts/substrate/install-substrate.sh` and does not collapse any of those branches to exit `1`.
- AC-BEDPM2-02: `docs/INSTALLATION.md` documents the Linux precedence chain `--pkg-manager -> PKG_MANAGER -> os_release -> path_probe`, the stable decision line, the fixed multi-manager warning posture, and the feature-specific remediation posture without restating a conflicting contract.
- AC-BEDPM2-03: `docs/INSTALLATION.md` documents wrapper parity for the feature path by stating that `scripts/substrate/install.sh` preserves upstream feature exits `0`, `2`, `3`, and `4`, while keeping macOS and Windows as explicit no-change platforms for ADR-0031.
- AC-BEDPM2-04: `docs/reference/env/contract.md` documents `PKG_MANAGER` with Linux-only scope, the exact allowed manager spellings, precedence below `--pkg-manager`, exit `2` for invalid values, and exit `3` when a valid explicit selection is absent from `PATH`.
- AC-BEDPM2-05: `docs/reference/env/contract.md` documents `SUBSTRATE_INSTALL_OS_RELEASE_PATH` with the exact hook name, absolute-path requirement, readable-regular-file requirement, `<unknown>` degradation on invalid or unreadable alternate paths, and the rule that setting the hook suppresses fallback reads from `/etc/os-release`.

## Out of scope
- Safe os-release parsing, distro-family mapping, `<unknown>` sentinel ownership, and stable decision-line timing remain owned by `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`.
- Explicit override precedence, ordered `PATH` probing, multi-manager warnings, and exits `2`, `3`, or `4` remain owned by `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`.
- Hermetic repo-test assertions, the thin Linux smoke wrapper, and manual evidence capture remain owned by `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM3/BEDPM3-spec.md`.
