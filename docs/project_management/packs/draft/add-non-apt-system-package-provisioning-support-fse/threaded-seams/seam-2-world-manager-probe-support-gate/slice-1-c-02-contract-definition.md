---
slice_id: S1
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: next
status: decomposed
plan_version: v1
basis:
  currentness: provisional
  basis_ref: seam.md#basis
  stale_triggers:
  - THR-01 changes probe precedence, supported families, or exit `4` posture
  - world_enable / world-agent shared-file changes alter where the probe runs
  - platform posture changes (Linux host-native / macOS Lima / Windows WSL) require a different
    supported-vs-unsupported gate outcome
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
contracts_produced:
  - C-02
contracts_consumed:
  - C-01
open_remediations: []
candidate_subslices: []
---
### S1 - Define `C-02` deterministic in-world manager probe + support gate

#### Goal

Make `C-02` explicit enough that downstream seams can treat manager selection as a single, deterministic, fail-closed truth:

- routing inputs are limited to `/etc/os-release` plus in-world package-manager availability probes (no host routing)
- the probe result selects exactly one of:
  - `apt`
  - `pacman`
  - unsupported (fail closed, exit `4`)

#### `C-02` — World-manager probe and support-gate contract (authoritative pre-exec text)

##### 1) Inputs (no host-derived routing)

The probe + gate MAY consult only:

- `/etc/os-release`:
  - `ID`
  - `ID_LIKE`
- in-world package-manager availability probe:
  - `command -v pacman` (present/absent)

The probe + gate MUST NOT consult:

- host PATH or host package-manager presence
- host installer detection results (ADR-0031 vocabulary is not a routing input here)
- any host-side `PKG_MANAGER` vocabulary

##### 2) Platform posture (supported vs unsupported lanes)

This pack’s v1 posture (from `../../review_surfaces.md#R4`) is:

- Linux host-native provisioning: unsupported (exit `4`, no host mutation)
- Windows WSL provisioning: unsupported (exit `4`, unsupported on Windows)
- macOS Lima guest provisioning: supported when the in-world probe selects one manager
  - default smoke uses an Ubuntu guest (Debian-family -> `apt`)
  - Arch-family pacman-success remains a manual evidence lane (still supported by contract; verified manually)

The support gate MUST be able to return “unsupported” even when `/etc/os-release` maps to a known family.

##### 3) `/etc/os-release` family mapping

Normalize `/etc/os-release` into exactly one family label:

- **Debian-family** (maps to manager `apt`)
- **Arch-family** (maps to manager `pacman`)
- **Unknown/ambiguous** (unsupported)

Mapping algorithm (deterministic):

1. Read `/etc/os-release`. If unreadable or missing `ID` and `ID_LIKE`, return unsupported (exit `4`).
2. Normalize both fields:
   - lower-case
   - treat `ID_LIKE` as whitespace-separated tokens (ignore quoting)
3. Compute match flags:
   - `is_debian_family` if `ID == debian` OR any `ID_LIKE` token equals `debian` OR `ubuntu`
   - `is_arch_family` if `ID == arch` OR any `ID_LIKE` token equals `arch` OR `archlinux`
4. If both flags are true, return unsupported (exit `4`) with reason `ambiguous_family_mapping`.
5. If neither flag is true, return unsupported (exit `4`) with reason `unmapped_family`.
6. Otherwise, the family is the single true flag.

##### 4) Package-manager presence check and contradiction rules

The probe MUST execute `command -v pacman` in-world and record `pacman_present: true|false`.

Contradiction rules (fail closed):

- If family is Arch-family and `pacman_present == false`:
  - return unsupported (exit `4`) with reason `arch_family_pacman_missing`
- If family is Debian-family:
  - `pacman_present` MUST NOT change the selected manager (still `apt`)
  - `pacman_present == true` is not a contradiction by itself (do not treat it as “Arch”)

##### 5) Outcomes

The support gate must choose exactly one outcome:

- **Supported: `apt`**
  - Preconditions: platform posture is supported AND family is Debian-family
  - Result: `manager=apt`, exit `0`
- **Supported: `pacman`**
  - Preconditions: platform posture is supported AND family is Arch-family AND `pacman_present=true`
  - Result: `manager=pacman`, exit `0`
- **Unsupported / fail closed**
  - Any platform-unsupported lane, unmapped family, ambiguous mapping, unreadable os-release, or contradiction
  - Result: no manager selected, exit `4`

##### 6) Dry-run behavior

Dry-run MUST still perform the in-world probe + gate decision and MUST NOT mutate system-package state.

##### 7) Exit codes and taxonomy alignment

- Canonical taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- `C-02` is responsible only for the fail-closed “unsupported/contradiction” posture:
  - use exit `4` for unsupported/contradiction/unmapped
  - use exit `3` when the world backend is unavailable and the probe cannot be executed in-world

#### Verification checklist (contract gate input)

- `C-02` uses only `/etc/os-release` + in-world `command -v pacman` inputs.
- `C-02` defines a deterministic family-mapping algorithm (including ambiguous/unmapped handling).
- `C-02` defines contradiction rules and is explicitly fail-closed.
- Platform posture outcomes (Linux host-native unsupported; macOS Lima supported; Windows WSL unsupported) are explicit.
- Exit-code posture matches `C-01` and references the canonical taxonomy.

