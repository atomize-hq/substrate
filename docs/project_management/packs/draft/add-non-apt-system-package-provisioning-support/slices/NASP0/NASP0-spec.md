# NASP0-spec — World-manager probe and provisioning support gate

## Behavior delta (single)
- Existing: ADR-0033 requires an in-world package-manager probe for `substrate world enable --provision-deps`, but the pack had no canonical slice spec for precedence between `/etc/os-release` and manager-presence checks, leaving shell, dispatch, and world-agent support-gate behavior unspecified.
- New: `NASP0` defines one deterministic provisioning probe and support gate: in-world `/etc/os-release` identity is authoritative, in-world `command -v pacman` confirms Arch-family support, unsupported or contradictory results fail closed with exit `4`, and host state is never consulted for manager selection.
- Why: Later slices need one stable manager-selection contract before pacman schema, provisioning routing, and runtime fail-early behavior can be implemented safely.

## Scope
- Define the exact `NASP0` probe inputs: in-world `/etc/os-release` `ID`, in-world `/etc/os-release` `ID_LIKE`, and in-world `command -v pacman`.
- Define normalization, family mapping, and contradiction handling for those inputs.
- Define the support gate that converts probe results plus backend capability into a supported `apt` path, a supported `pacman` path, or a deterministic exit `4`.
- Define the invariant that manager selection is derived inside the world and never from host PATH, host installer detection, or host package-manager state.
- Leave requirement derivation, pacman command construction, mixed-manager enabled-set enforcement, and runtime fail-early item scoping to later slices.

## Inputs (authoritative)
- Shared operator contract and exit-code meanings: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- Probe precedence decision: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md` (`DR-0002`)
- Required slice-owned surfaces and acceptance focus: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md`
- Cross-slice invariants and fail-closed posture: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/minimal_spec_draft.md`
- Implementation seam and touch boundary: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md`

## Behavior (authoritative)
### Slice boundary and produced outcome
- This slice owns only provisioning-time world-manager detection and provisioning eligibility gating.
- The probe outcome vocabulary is limited to:
  - detected manager `apt`
  - detected manager `pacman`
  - unsupported or contradictory world state, which exits `4`
- Later slices may consume the detected manager result, but they MUST NOT redefine the input set, precedence order, or fail-closed mismatch rule established here.

### In-world execution boundary
- The probe MUST execute only inside the world context used by `substrate world enable --provision-deps`.
- Linux host-native and Windows WSL support gates MUST NOT substitute host `/etc/os-release`, host PATH, host installer variables, or host package-manager state when provisioning is unsupported.
- macOS Lima guest provisioning MUST probe the guest world after backend enable succeeds and before any world OS package-manager command is attempted.
- `--dry-run` and non-dry-run MUST use the same in-world probe contract. `--dry-run` may skip later provisioning mutation, but it MUST NOT switch to a host-side probe.

### `/etc/os-release` normalization and family mapping
- The probe MUST read `/etc/os-release` inside the world and extract `ID` plus `ID_LIKE`.
- Normalization rules:
  - trim leading and trailing ASCII whitespace from both fields
  - lowercase all probe tokens before comparison
  - treat `ID` as one candidate token when non-empty
  - split `ID_LIKE` on ASCII whitespace and preserve token order
- Family resolution order:
  1. normalized `ID`
  2. each normalized `ID_LIKE` token in file order
- Supported family-token mapping:
  - `debian` or `ubuntu` => detected manager candidate `apt`
  - `arch` or `archlinux` => detected manager candidate `pacman`
- The first supported token found in that ordered search wins. If no supported token is present, the world is unsupported for this feature and the command exits `4`.

### Pacman confirmation and contradiction handling
- When `/etc/os-release` resolves to `pacman`, the probe MUST run in-world `command -v pacman`.
- `command -v pacman` success confirms the `pacman` manager result.
- If `/etc/os-release` resolves to `pacman` and `command -v pacman` fails, the command MUST exit `4` before any `apt`, `dpkg`, or `pacman` execution.
- The probe MUST NOT fall back from an Arch-family `/etc/os-release` result to `apt`.
- When `/etc/os-release` resolves to `apt`, a successful `command -v pacman` check is non-authoritative and MUST NOT switch selection away from `apt`.
- This slice does not redefine the established APT provisioning baseline; it only fixes the new tie-break rule that host state and `pacman` presence cannot override an `apt` result selected from in-world `/etc/os-release`.

### Support gate outcomes
- Linux host-native backend:
  - `substrate world enable --provision-deps` is unsupported and exits `4`.
  - The support gate MUST reject before any host OS package-manager command and MUST preserve the contract message that Substrate will not mutate the host OS.
- macOS Lima guest backend:
  - provisioning remains eligible only after the guest world probe returns detected manager `apt` or `pacman`
  - an Arch-family guest is eligible only when `command -v pacman` confirms support
  - an unmappable or contradictory guest world exits `4`
- Windows WSL backend:
  - `substrate world enable --provision-deps` is unsupported and exits `4`
  - the support gate MUST preserve the contract wording that the command is unsupported on Windows and will not mutate the Windows host OS
- This slice defines provisioning eligibility only. Mixed-manager enabled-set failure, requirement-set derivation, and package-manager command construction remain owned by later slices and the shared contract.

### Error-path invariants
- Every unsupported or contradictory probe outcome in this slice exits `4`.
- Every unsupported or contradictory probe outcome in this slice occurs before any world OS package-manager mutation.
- Error handling in this slice MUST remain manager-aware and fail-closed:
  - unsupported world OS family does not imply host fallback
  - unsupported backend does not imply host fallback
  - pacman absence on an Arch-family world does not imply APT fallback
- Later runtime and provisioning mismatch paths MUST continue to use the contract-owned exit `4` posture and remediation wording established by `contract.md`.

## Acceptance criteria
- AC-NASP0-01: On a supported guest backend whose in-world `/etc/os-release` resolves through `ID` or `ID_LIKE` to `arch` or `archlinux`, and whose in-world `command -v pacman` succeeds, `substrate world enable --provision-deps` recognizes the world as `pacman`-capable and leaves later provisioning slices a detected manager result of `pacman`.
- AC-NASP0-02: On a supported guest backend whose in-world `/etc/os-release` resolves through `ID` or `ID_LIKE` to `debian` or `ubuntu`, the probe selects `apt` from `/etc/os-release` and does not switch to `pacman` because of host PATH, host installer detection, or an in-world `pacman` binary alone.
- AC-NASP0-03: If in-world `/etc/os-release` is unreadable, missing, or contains no supported `ID` or `ID_LIKE` token, `substrate world enable --provision-deps` exits `4` before any `apt`, `dpkg`, or `pacman` command is executed.
- AC-NASP0-04: If in-world `/etc/os-release` resolves to an Arch-family token but in-world `command -v pacman` fails, `substrate world enable --provision-deps` exits `4`, reports a deterministic unsupported-or-mismatch path, and does not fall back to `apt`.
- AC-NASP0-05: On Linux host-native, `substrate world enable --provision-deps` exits `4` without consulting host package-manager state, and stderr preserves the contract phrase `Substrate will not mutate the host OS`.
- AC-NASP0-06: On Windows WSL, `substrate world enable --provision-deps` exits `4` without consulting host package-manager state, and stderr preserves the contract phrase `unsupported on Windows`.
- AC-NASP0-07: `substrate world enable --provision-deps --dry-run` and `substrate world enable --provision-deps` both use the same in-world probe inputs and precedence rules, and neither command is permitted to replace the probe with host-side routing inputs.

## Out of scope
- World-deps schema additions for `install.method=pacman` and `install.pacman` (`NASP1`).
- Requirement-set derivation, mixed-manager enabled-set enforcement, and pacman command execution (`NASP2`).
- Runtime fail-early wording for `substrate world deps current sync|install` once system-package items are in scope (`NASP3`).
- Platform smoke/manual evidence and cross-doc reconciliation work (`NASP4`).
