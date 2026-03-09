# NASP2-spec — Provisioning routing and pacman command execution

## Behavior delta (single)
- Existing: the pack contract now defines a manager-aware `substrate world enable --provision-deps` surface, but there is no canonical slice that fixes how the effective enabled set becomes normalized pacman requirements, how mixed-manager system-package inputs fail closed, or how a supported pacman world constructs the exact in-world provisioning command.
- New: `NASP2` defines one deterministic provisioning-wiring path: derive APT and pacman requirement sets from the effective enabled set after upstream bundle expansion, normalize pacman requirements by exact-name de-duplication plus ascending byte-order sorting, use the dedicated provisioning request profile only for provisioning execution, and dispatch exactly `pacman -Sy --noconfirm --needed <packages...>` on supported pacman worlds.
- Why: later runtime fail-early and validation slices need one stable provisioning execution contract before they can lock operator remediation text and platform evidence without reopening command-shape or mismatch behavior.

## Scope
- Define the exact pacman requirement-derivation rule from the effective enabled world-deps set used by `substrate world enable --provision-deps`.
- Define the exact pacman package de-duplication and stable ordering rules used for dry-run output and command construction.
- Define the exact mixed-manager enabled-set failure rule when the enabled set contains both APT-backed and pacman-backed system-package items.
- Define the exact provisioning request-profile usage boundary for `substrate world enable --provision-deps`.
- Define the exact pacman command shape, no-op detection boundary, and non-partial-failure posture selected by `DR-0003`.
- Leave in-world manager probing, pacman inventory-schema validation, runtime fail-early remediation wording, and platform-validation evidence to other slice owners.

## Inputs (authoritative)
- Shared manager-aware provisioning contract, exit-code meanings, and request-profile invariants:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- Pacman execution and idempotency decision:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md` (`DR-0003`)
- World-manager probe and support gate consumed by this slice:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP0/NASP0-spec.md`
- Pacman inventory-schema and authored-list rules consumed by this slice:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP1/NASP1-spec.md`
- Required slice-owned surfaces and acceptance focus:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md`
- Upstream enabled-set, bundle expansion, and inventory replacement baseline reused by this slice:
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
- Implementation seam and touch boundary:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md`

## Behavior (authoritative)

### Slice boundary and produced outcome
- This slice owns only provisioning-time requirement derivation, request-profile routing, mismatch rejection, and pacman command execution for `substrate world enable --provision-deps`.
- The output of this slice is a deterministic provisioning plan with:
  - one normalized APT requirement set
  - one normalized pacman requirement set
  - one detected world manager result consumed from `NASP0`
  - one selected execution path or one deterministic exit-`4` mismatch path
- Later slices MAY reference the normalized requirement sets and the provisioning outcome, but they MUST NOT redefine the pacman normalization rule, the mixed-manager failure posture, the provisioning request-profile boundary, or the pacman command shape established here.

### Enabled-set input boundary
- Provisioning derives requirements from the effective enabled world-deps set for `cwd`, using the upstream inventory merge, enabled resolution, and bundle-expansion rules unchanged.
- V1 provisioning accepts no explicit item list and therefore does not reuse the runtime explicit-item scoping rules.
- Requirement derivation considers only system-package items in the effective enabled set:
  - APT-backed items contribute `install.apt` requirement entries to the normalized APT requirement set.
  - Pacman-backed items contribute `install.pacman` requirement entries to the normalized pacman requirement set.
  - `install.method=script` and `install.method=manual` items do not contribute to either system-package requirement set in this slice.
- This slice does not reinterpret authored pacman package names, does not add distro translation, and does not widen the enabled-set inputs beyond the resolved world-deps contract.

### Pacman requirement normalization
- Pacman requirement derivation starts from the authored `install.pacman` lists of every pacman-backed item in the effective enabled set after bundle expansion and inventory replacement.
- The normalized pacman requirement set is built by:
  1. collecting every authored `install.pacman[]` string from the in-scope pacman-backed items
  2. de-duplicating by exact package-name string equality
  3. sorting the surviving package names in ascending byte order
- Pacman normalization does not preserve authored cross-item ordering once normalization begins.
- Pacman normalization does not add version semantics, aliasing, virtual-package expansion, or manager-specific rewrite rules.
- The normalized pacman requirement order is authoritative for:
  - dry-run rendered package lists
  - verbose rendered package lists
  - pacman no-op detection probes
  - the eventual `pacman` command arguments

### Mixed-manager fail-closed rule
- If the normalized APT requirement set and the normalized pacman requirement set are both non-empty, provisioning MUST exit `4` before any `apt`, `dpkg`, or `pacman` command is executed.
- This mismatch path is determined after enabled-set resolution and requirement normalization, not from authored inventory files in isolation.
- The error path MUST:
  - name the detected world manager returned by the `NASP0` probe
  - state that the enabled set mixes incompatible system-package methods
  - instruct the operator to use a world image whose package manager matches the enabled inventory or to change the enabled inventory so only one system-package manager is in scope
- No partial provisioning is allowed. When this mismatch path triggers, Substrate executes neither the APT path nor the pacman path.

### Request-profile routing boundary
- Provisioning execution in this slice MUST use the dedicated request profile `world-deps-provision`.
- `SUBSTRATE_WORLD_REQUEST_PROFILE` is not an operator input for this slice and MUST NOT be required to activate provisioning behavior.
- The provisioning request profile is selected by the `substrate world enable --provision-deps` flow itself, not by host environment discovery and not by runtime `deps current sync|install` commands.
- Runtime `substrate world deps current sync|install` remains outside this slice and MUST NOT borrow the provisioning request profile.
- Dry-run MAY surface the selected provisioning request profile in verbose output, but dry-run MUST NOT require a different profile or a different routing path than non-dry-run execution.

### Pacman execution path
- This slice consumes the detected world manager from `NASP0` and dispatches the pacman execution path only when that detected world manager is `pacman`.
- If the detected world manager is `pacman`, provisioning uses only the normalized pacman requirement set and ignores the APT execution path.
- If the normalized pacman requirement set is empty, provisioning phase 2 is a contract-defined no-op and exits `0` without invoking `pacman`.
- Before mutation, pacman no-op detection MAY use read-only in-world pacman queries against the normalized pacman requirement set to determine whether every required package is already present.
- When provisioning is required, the exact in-world command shape is:

  ```text
  pacman -Sy --noconfirm --needed <packages...>
  ```

- The `<packages...>` arguments MUST appear in normalized pacman requirement order.
- This slice MUST NOT insert AUR helpers, alternate frontends, version pins, retries, lock-file deletion, or fallback from pacman to apt.
- If pacman returns non-zero, Substrate exits `4`, surfaces pacman stderr, performs no retry, and does not continue with any further system-package action.

### Dry-run and verbose rendering
- `substrate world enable --provision-deps --dry-run` MUST still:
  - perform the in-world manager probe from `NASP0`
  - derive the normalized APT and pacman requirement sets from the effective enabled set
  - enforce the mixed-manager fail-closed rule
  - report the detected world manager when the probe succeeds
- `--dry-run` MUST perform no mutation and MUST NOT execute `apt`, `dpkg`, or `pacman`.
- When the detected world manager is `pacman` and the normalized pacman requirement set is non-empty, `--dry-run` MUST render the normalized pacman requirement entries in stable order and the exact intended pacman command shape.
- When `--verbose` is present, output MUST include:
  - the detected world manager
  - the normalized APT and pacman requirement sets
  - the selected provisioning request profile value `world-deps-provision`

## Acceptance criteria
- AC-NASP2-01: If the effective enabled set contains pacman-backed packages only, the `NASP0` probe returns detected manager `pacman`, and the combined authored `install.pacman` entries include duplicates across multiple packages or bundles, `substrate world enable --provision-deps` derives one normalized pacman requirement set by exact-name de-duplication and ascending byte-order sorting before any pacman command is constructed.
- AC-NASP2-02: If the effective enabled set contains both APT-backed and pacman-backed system-package items, `substrate world enable --provision-deps` exits `4` before any `apt`, `dpkg`, or `pacman` command runs, names the detected world manager, and reports that the enabled set mixes incompatible system-package methods.
- AC-NASP2-03: If the detected world manager is `pacman` and the normalized pacman requirement set is non-empty, provisioning executes exactly `pacman -Sy --noconfirm --needed <packages...>` inside the world, with `<packages...>` rendered in normalized requirement order and with no AUR helper, retry, or lock-file intervention.
- AC-NASP2-04: If the detected world manager is `pacman` and every normalized pacman requirement is already present according to read-only in-world pacman queries, provisioning completes as a no-op with exit `0` and does not invoke a mutating pacman command.
- AC-NASP2-05: `substrate world enable --provision-deps --dry-run` uses the same enabled-set derivation, pacman normalization, detected-manager routing, and mixed-manager rejection rules as non-dry-run execution, but does not execute `apt`, `dpkg`, or `pacman`.
- AC-NASP2-06: Provisioning execution uses the dedicated request profile `world-deps-provision` without requiring the operator to set `SUBSTRATE_WORLD_REQUEST_PROFILE`, and runtime `substrate world deps current sync|install` does not reuse that provisioning profile.
- AC-NASP2-07: If the detected world manager is `pacman` and the pacman command returns non-zero, `substrate world enable --provision-deps` exits `4`, surfaces pacman stderr, performs no retry, and does not continue with any further system-package action or partial fallback.

## Out of scope
- In-world world-manager probing, `/etc/os-release` precedence, and support-gate behavior (`NASP0`).
- Pacman inventory schema, pacman-specific validation failures, and list/show rendering (`NASP1`).
- Runtime fail-early behavior, explicit-item scoping, and remediation wording for `substrate world deps current sync|install` (`NASP3`).
- Platform parity evidence, smoke/manual validation, and cross-doc reconciliation work (`NASP4`).
