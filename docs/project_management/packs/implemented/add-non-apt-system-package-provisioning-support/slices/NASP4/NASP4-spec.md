# NASP4-spec — Validation evidence and contract reconciliation

## Behavior delta (single)
- Existing: the pack contract and slice specs define manager-aware behavior, but the validation surfaces and reconciliation targets are not captured in one canonical slice, so Linux, macOS, Windows, and upstream doc updates can drift independently.
- New: `NASP4` fixes one authoritative validation and reconciliation lane: `platform-parity-spec.md` defines the cross-platform guarantees, `manual_testing_playbook.md` defines the required operator rerun evidence, the three smoke scripts define the platform-specific assertion sets, and this slice names the exact upstream docs that later implementation work must reconcile to the manager-aware contract.
- Why: the pack is execution-ready only when validation evidence and doc-update targets are pinned with the same determinism as the runtime and provisioning contracts.

## Scope
- Define the exact doc-update targets that later execution work must reconcile to the accepted manager-aware contract.
- Define the exact platform parity posture for Linux host-native, macOS Lima guest, and Windows WSL.
- Define the exact manual validation cases required before slice closeout.
- Define the exact smoke assertion sets for `smoke/linux-smoke.sh`, `smoke/macos-smoke.sh`, and `smoke/windows-smoke.ps1`.
- Keep provisioning-time probe semantics, schema validation, pacman command construction, and runtime fail-early behavior owned by `NASP0`, `NASP1`, `NASP2`, and `NASP3`.

## Inputs (authoritative)
- Shared manager-aware contract:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- Selected decisions for probe precedence, pacman execution shape, and v1 pacman scope:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md`
- Canonical schema and validation boundaries:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md`
- Earlier slice contracts:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP0/NASP0-spec.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP1/NASP1-spec.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP2/NASP2-spec.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP3/NASP3-spec.md`
- Required-doc ownership and touch boundary:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md`

## Behavior (authoritative)
### Validation ownership and produced outcome
- This slice owns validation evidence and reconciliation targets only.
- The output of this slice is:
  - one platform parity specification
  - one manual testing playbook
  - three platform smoke scripts
  - one exact reconciliation target list for later implementation and doc updates
- Later execution work MUST treat those artifacts as the single validation authority for this feature.

### Reconciliation target set
- Later implementation and documentation work for this pack MUST reconcile the shared manager-aware contract across exactly these upstream docs:
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
  - `docs/reference/world/deps/README.md`
  - `docs/internals/world/deps.md`
- Reconciliation is complete only when those docs no longer present a second truth for:
  - the shared `substrate world enable --provision-deps` entrypoint
  - runtime prohibition on `apt` and `pacman`
  - the exact remediation command string
  - the manager-aware support matrix

### Platform validation contract
- Linux host-native validation proves fail-closed behavior:
  - provisioning exits `4`
  - stderr preserves `Substrate will not mutate the host OS`
  - runtime fail-early remains deterministic for pacman-backed items
- macOS validation splits into two exact lanes:
  - `smoke/macos-smoke.sh` validates the supported guest-backend path that the repo-default Ubuntu-based Lima fixture can exercise deterministically: world-backend health, mixed-manager provisioning rejection, runtime fail-early ordering, and explicit-item scope
  - `manual_testing_playbook.md` validates the Arch-family pacman-success path against an Arch-family Lima VM named `substrate`
- Windows validation proves fail-closed behavior:
  - provisioning exits `4`
  - stderr preserves `unsupported on Windows`
  - runtime fail-early remains deterministic for pacman-backed items

### Manual evidence contract
- The manual playbook MUST define exactly four behavior cases:
  - Arch-family provisioning success on macOS against an Arch-family `substrate` Lima VM
  - unsupported provisioning on Linux host-native or Windows
  - manager mismatch or mixed-manager rejection on macOS
  - runtime fail-early plus explicit-item scope for pacman-backed items
- Every manual case MUST specify:
  - fixture setup
  - exact command lines
  - expected exit codes
  - required stdout or stderr substrings

### Smoke evidence contract
- `smoke/linux-smoke.sh` is authoritative for Linux host-native validation.
- `smoke/macos-smoke.sh` is authoritative for macOS validation against the repo-default Lima guest.
- `smoke/windows-smoke.ps1` is authoritative for Windows validation.
- The smoke scripts MUST not introduce a second behavior contract. They execute the command paths already fixed by `contract.md` and earlier slice specs.

## Acceptance criteria
- AC-NASP4-01: `platform-parity-spec.md` defines one exact Linux/macOS/Windows support matrix that matches `contract.md`, including Linux and Windows fail-closed provisioning and the macOS split between automated default-guest smoke coverage and manual Arch-family success evidence.
- AC-NASP4-02: `manual_testing_playbook.md` defines exact setup, commands, expected exit codes, and required stdout or stderr substrings for Arch-family provisioning success, unsupported provisioning, manager mismatch or mixed-manager rejection, and runtime fail-early behavior.
- AC-NASP4-03: `smoke/linux-smoke.sh` validates Linux host-native exit-`4` provisioning, pacman-backed runtime fail-early output, and explicit-item scoping without mutating the host OS.
- AC-NASP4-04: `smoke/macos-smoke.sh` validates a healthy Lima guest backend, exit-`4` mixed-manager provisioning rejection, manager-aware runtime fail-early ordering, and explicit-item scoping on the repo-default macOS guest fixture.
- AC-NASP4-05: `smoke/windows-smoke.ps1` validates Windows exit-`4` provisioning, pacman-backed runtime fail-early output, and explicit-item scoping while preserving the `unsupported on Windows` guidance.
- AC-NASP4-06: This slice names the exact reconciliation targets `ADR-0033-routing-weasel.md`, `world-deps-apt-provisioning/contract.md`, `world-deps-packages-bundles-contract/contract.md`, `docs/reference/world/deps/README.md`, and `docs/internals/world/deps.md`, so later execution work has one fixed doc-update set instead of ad hoc propagation.

## Out of scope
- Provisioning-time `/etc/os-release` probe inputs and tie-break rules remain owned by `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP0/NASP0-spec.md`.
- Pacman inventory schema and pacman-specific invalid states remain owned by `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP1/NASP1-spec.md`.
- Mixed-manager provisioning rejection, request-profile routing, and pacman command execution remain owned by `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP2/NASP2-spec.md`.
- Runtime fail-early derivation, read-only probes, remediation wording, and explicit-item scoping remain owned by `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP3/NASP3-spec.md`.
