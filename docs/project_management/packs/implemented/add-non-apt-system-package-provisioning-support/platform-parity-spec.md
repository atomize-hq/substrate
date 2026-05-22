# add-non-apt-system-package-provisioning-support — platform parity spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for cross-platform guarantees and validation evidence for the manager-aware provisioning contract introduced by ADR-0033.

## Required platforms

- Behavior platforms (smoke required): `linux`, `macos`, `windows`
- CI parity platforms (parity required): `linux`, `macos`, `windows`
- WSL required: `false`

## Guarantees (explicit)

What is identical across platforms:

- Runtime `substrate world deps current sync|install` never invokes mutating `apt`, `dpkg`, or `pacman`.
- Runtime fail-early remediation always includes the exact command `substrate world enable --provision-deps`.
- Pacman-backed package names render in normalized stable order in dry-run output.
- Explicit-item runtime installs do not add unrelated enabled system-package items implicitly.
- Provisioning ignores `SUBSTRATE_WORLD_REQUEST_PROFILE` and keeps the provisioning profile value `world-deps-provision` internal to the Substrate flow.

## Platform/backend matrix (explicit)

### Linux host-native backend

- Provisioning posture:
  - `substrate world enable --provision-deps` is unsupported.
  - The command exits `4` before any host OS package-manager command runs.
  - Stderr includes `Substrate will not mutate the host OS`.
- Runtime posture:
  - Pacman-backed and APT-backed items fail early with exit `4` when required system packages are missing.
  - Runtime output remains manager-aware and preserves the exact remediation command.
- Required evidence:
  - `smoke/linux-smoke.sh`
  - the Linux sections of `manual_testing_playbook.md`

### macOS Lima guest backend

- Repo-default backend posture:
  - `scripts/mac/lima/substrate.yaml` provisions an Ubuntu 24.04 guest, so the repo-default `substrate` Lima VM proves the supported guest-backend routing path and the manager-aware mismatch and runtime semantics.
  - `smoke/macos-smoke.sh` is authoritative for:
    - world-backend health
    - mixed-manager provisioning rejection
    - runtime fail-early ordering across APT-backed and pacman-backed items
    - explicit-item scoping
- Arch-family success posture:
  - The required pacman-success evidence is manual-only in this pack and is recorded separately from `smoke/macos-smoke.sh`.
  - The manual fixture is an Arch-family Lima VM named `substrate` whose guest exposes:
    - `/etc/os-release` with `ID=arch` or `ID_LIKE=arch`
    - `pacman` on `PATH`
    - `/usr/local/bin/substrate`
    - `/usr/local/bin/substrate-world-service`
    - `/run/substrate.sock`
  - `manual_testing_playbook.md` is authoritative for that Arch-family success case.

### Windows WSL backend

- Provisioning posture:
  - `substrate world enable --provision-deps` is unsupported.
  - The command exits `4`.
  - Stderr includes `unsupported on Windows`.
  - The command does not mutate the Windows host OS.
- Runtime posture:
  - Pacman-backed and APT-backed items fail early with exit `4` when required system packages are missing.
  - Runtime remediation preserves the exact command string and Windows guidance.
- Required evidence:
  - `smoke/windows-smoke.ps1`
  - the Windows sections of `manual_testing_playbook.md`

## Permitted divergences (explicit)

- macOS default-guest smoke does not prove the pacman-success path because the repo-default Lima profile is Ubuntu-based.
- The required Arch-family success path on macOS is therefore manual evidence only.
- Linux and Windows validate unsupported provisioning directly; macOS validates supported guest-backend behavior plus mixed-manager rejection on the default guest and validates pacman-success separately through the manual Arch fixture.

## Validation evidence (explicit)

- Smoke scripts required:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1`
- Manual playbook required:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`
- Doc reconciliation targets required before slice closeout:
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
  - `docs/reference/world/deps/README.md`
  - `docs/internals/world/deps.md`

## Acceptance criteria

- Linux and Windows:
  - provisioning exits `4` with the required platform-specific guidance before any host OS package-manager mutation
  - runtime dry-run output remains manager-aware for pacman-backed fixtures
  - explicit-item runtime installs remain scoped to the explicit item set
- macOS:
  - `smoke/macos-smoke.sh` proves the supported guest-backend path is healthy and that mixed-manager provisioning rejection plus runtime fail-early ordering match the contract on the repo-default Lima guest
  - `manual_testing_playbook.md` proves the Arch-family pacman-success path against an Arch-family `substrate` VM fixture
