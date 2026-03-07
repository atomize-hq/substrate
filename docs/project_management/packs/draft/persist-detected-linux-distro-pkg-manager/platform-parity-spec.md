# persist-detected-linux-distro-pkg-manager — platform parity spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope
- This spec is authoritative for platform guarantees and permitted divergences for ADR-0032 in `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/`.
- This spec locks the validation boundary between Linux behavior smoke and macOS/Windows no-delta evidence.
- This spec does not redefine the install-state schema or installer write contract; those remain authoritative in `contract.md`, `install-state-schema-spec.md`, and `compatibility-spec.md`.

## Required platforms
- Behavior platforms (smoke required): `linux`
- CI parity platforms (parity required): `linux`, `macos`, `windows`
- WSL required: `false`
- WSL task mode: `bundled`

## Guarantees (explicit)
- Cross-platform invariants:
  - The persisted metadata path for this feature remains `<resolved SUBSTRATE_HOME>/install_state.json`.
  - `schema_version` remains `1`.
  - This feature introduces no new CLI flags, config files, environment variables, or success-path exit codes.
  - `install_state.json` keeps one meaning across `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh`.
- Permitted divergence:
  - Linux is the only behavior-delta platform. Successful non-dry-run Linux installs through the two in-scope Unix installers persist `host_state.platform.*` according to `contract.md` and `install-state-schema-spec.md`.
  - macOS gains no new `host_state.platform.*` write contract from ADR-0032 and gains no new file-presence guarantee from this feature.
  - Windows gains no new `host_state.platform.*` write contract from ADR-0032 and gains no new file-presence guarantee from this feature.

## Known platform hazards (explicit)
- Hazard: `scripts/substrate/install-substrate.sh` is a shared Unix entry point. Linux-only metadata changes must stay behind Linux guards so macOS behavior does not drift.
  - Mitigation: require Linux smoke plus `make ci-compile-parity CI_WORKFLOW_REF="feat/persist-detected-linux-distro-pkg-manager"`.
- Hazard: `scripts/substrate/dev-install-substrate.sh` writes the same `install_state.json` path as the production installer. Divergent Linux payload rules would split one on-disk contract.
  - Mitigation: require the `dev` scenario in `tests/installers/install_smoke.sh` to assert the same `schema_version=1` and `host_state.platform.*` meaning.
- Hazard: operator docs can overstate the Linux metadata change as a cross-platform behavior change.
  - Mitigation: closeout docs must state Linux-only behavior delta and explicit macOS/Windows no-delta notes.

## Validation evidence (explicit)
- Smoke scripts required:
  - `bash tests/installers/install_state_smoke.sh --scenario metadata`
  - `bash tests/installers/install_state_smoke.sh --scenario cleanup`
  - `bash tests/installers/install_state_smoke.sh --scenario missing`
  - `bash tests/installers/install_smoke.sh --scenario prod-no-world`
  - `bash tests/installers/install_smoke.sh --scenario dev`
- CI parity gates required:
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/persist-detected-linux-distro-pkg-manager"`
- Manual playbook sections required:
  - `docs/INSTALLATION.md` `### Installer Metadata & Cleanup`
  - `docs/INSTALLATION.md` `### macOS (arm64)`
  - `docs/INSTALLATION.md` `### Windows Host (PowerShell)`

## Acceptance criteria (testable)
- Linux: `platform-parity-spec.md`, `plan.md`, and `slices/PDLDPM2/PDLDPM2-spec.md` all identify Linux as the only behavior-smoke platform, with production metadata validation stabilized before dev-installer parity, and they point to the exact Linux smoke commands listed above.
- macOS: the pack states that ADR-0032 adds no new `host_state.platform.*` write contract on macOS, and validation evidence for macOS is limited to explicit no-delta documentation plus compile parity.
- Windows: the pack states that ADR-0032 adds no new `host_state.platform.*` write contract on Windows, and validation evidence for Windows is limited to explicit no-delta documentation plus compile parity.
- Cross-platform: `make ci-compile-parity CI_WORKFLOW_REF="feat/persist-detected-linux-distro-pkg-manager"` remains the required cross-platform gate after the Linux installer-smoke changes land.
