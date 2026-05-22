# NASP3-spec — Runtime fail-early and manager-aware remediation

## Behavior delta (single)
- Existing: the pack contract now requires runtime `substrate world deps current sync|install` to stay read-only for system-package items, but there is no canonical slice that fixes how `apt` and `pacman` requirements are probed at runtime, how `current install <ITEM...>` avoids pulling in unrelated enabled items, or how remediation stays deterministic when missing requirements span one or both system-package managers.
- New: `NASP3` makes runtime `substrate world deps current sync|install` uniformly fail early for missing system-package requirements by reusing the normalized APT and pacman requirement sets, probing them read-only inside the selected world, scoping `current install <ITEM...>` to the explicit expanded item list only, and emitting one exact remediation command plus manager-aware missing-requirement output before any non-system-package mutation runs.
- Why: the manager-aware provisioning contract is only safe if runtime never mutates world OS packages, never lets enabled system-package items poison unrelated explicit installs, and tells the operator exactly how to move to the provisioning-time path without implying host mutation.

## Scope
- Runtime preflight and fail-early behavior for:
  - `substrate world deps current sync [--dry-run] [--verbose] [--all]`
  - `substrate world deps current install <ITEM...> [--dry-run] [--verbose]`
- Deterministic definitions for:
  - the runtime in-scope set for `sync`, `sync --all`, and `install <ITEM...>`
  - reuse of the normalized APT requirement set and normalized pacman requirement set defined by the shared contract plus earlier slices
  - read-only presence-probe rules for APT-backed and pacman-backed requirements
  - fail-early sequencing when one or both normalized system-package requirement sets are non-empty
  - remediation wording invariants, including mixed-manager runtime messaging
  - `--dry-run` and `--verbose` behavior when runtime exits `4`
- Leave provisioning-time manager probing, pacman command construction, inventory-schema validation, and cross-doc reconciliation to `NASP0`, `NASP1`, `NASP2`, and `NASP4`.

## Behavior (authoritative)

### Slice boundary and produced outcome
- This slice owns only runtime fail-early behavior for system-package items in `substrate world deps current sync|install`.
- The output of this slice is one deterministic runtime preflight outcome:
  - no system-package requirements in scope, so runtime proceeds with upstream non-system-package behavior
  - every in-scope system-package requirement is already satisfied, so runtime treats system-package items as satisfied and proceeds with upstream non-system-package behavior
  - one or more in-scope system-package requirements are unsatisfied, so runtime exits `4` before any non-system-package mutation
- Later slices MAY reference the runtime error posture established here, but they MUST NOT redefine the explicit-item scope rule, the read-only probe-only posture, or the remediation invariants fixed by this slice.

### Invariant: no runtime OS package-manager mutation
- Runtime `substrate world deps current sync|install` MUST NOT execute `apt`, `apt-get`, mutating `dpkg`, or mutating `pacman`.
- The only system-package-manager interactions permitted at runtime are read-only presence probes:
  - `dpkg-query` for the normalized APT requirement set
  - `pacman -Q` for the normalized pacman requirement set
- Runtime MUST NOT fall back from a failed read-only probe to provisioning, host mutation, or any alternate package-manager command.

### Runtime in-scope set selection
Source of truth for inventory merge, enabled resolution, and bundle expansion:
`docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`

Rules:
- `deps current sync` without `--all`:
  - the runtime in-scope set is the effective enabled world-deps set for `cwd`
- `deps current sync --all`:
  - the runtime in-scope set is every visible world-deps item from inventory
- `deps current install <ITEM...>`:
  - bundle expansion MUST occur for `<ITEM...>` per the upstream contract
  - the runtime in-scope set is the expanded explicit `<ITEM...>` only
  - the effective enabled set MUST NOT be added implicitly

### Requirement-set reuse and runtime derivation boundary
- This slice reuses the normalized APT requirement set defined by `contract.md` and the inherited APT version-resolution rules from `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md`.
- This slice reuses the normalized pacman requirement set defined by `contract.md` and `slices/NASP2/NASP2-spec.md`.
- Runtime derivation from the in-scope set therefore follows this order:
  1. select in-scope items whose resolved `install.method` is `apt` or `pacman`
  2. derive the normalized APT requirement set from the in-scope APT-backed items
  3. derive the normalized pacman requirement set from the in-scope pacman-backed items
- If normalized APT derivation hits a version-conflict error or if pacman-backed inventory violates the schema already fixed by `NASP1`, runtime MUST exit `2` before any read-only probe or non-system-package mutation runs.

### Read-only presence probes
- When the normalized APT requirement set is non-empty, Substrate MUST perform a read-only in-world `dpkg-query` probe for each normalized APT requirement entry.
- APT satisfaction rules:
  - unpinned requirement `name` is satisfied only when `dpkg-query` reports `install ok installed` for `name`
  - pinned requirement `name=version` is satisfied only when `dpkg-query` reports `install ok installed` and the installed version equals `version`
  - if `dpkg-query` cannot be executed inside the selected world, the APT requirement is unsatisfied
- When the normalized pacman requirement set is non-empty, Substrate MUST perform a read-only in-world `pacman -Q <name>` probe for each normalized pacman requirement entry.
- Pacman satisfaction rules:
  - requirement `name` is satisfied only when `pacman -Q <name>` returns success for that exact package name
  - if `pacman -Q` cannot be executed inside the selected world, the pacman requirement is unsatisfied
  - pacman runtime presence checks do not add version matching, alias expansion, or fallback to any other pacman query mode
- If world-service connectivity is required for either read-only probe and cannot be established, runtime MUST exit `3` with actionable stderr.

### Runtime fail-early sequencing
For `substrate world deps current sync|install`:
1. Derive the normalized APT requirement set and normalized pacman requirement set from the runtime in-scope set.
2. If both normalized system-package requirement sets are empty, proceed with upstream non-system-package behavior.
3. If one or both normalized system-package requirement sets are non-empty, perform the read-only presence probes for every derived requirement entry.
4. If any required system package is unsatisfied, exit `4`, emit remediation to stderr, and perform no non-system-package mutation.
5. If every derived system-package requirement is already satisfied, treat the system-package items as satisfied/no-op and proceed with upstream non-system-package behavior.
- Runtime does not reuse the provisioning mixed-manager rejection rule from `NASP2`.
- A runtime in-scope set MAY contain both APT-backed and pacman-backed items. Runtime probes both normalized requirement sets read-only, and it exits `4` only when one or more derived requirements are unsatisfied.

### Remediation output invariants
When runtime exits `4` because one or more system-package requirements are unsatisfied, stderr MUST:
- include the exact command

  ```text
  substrate world enable --provision-deps
  ```

- state that runtime system-package mutation is not supported
- render the unsatisfied requirement entries in stable order with manager-aware labeling:
  - APT missing entries render first in normalized APT order using `name` or `name=version`
  - pacman missing entries render second in normalized pacman order using `name`
- preserve backend-specific guidance from `contract.md`:
  - Linux host-native includes the exact phrase `Substrate will not mutate the host OS`
  - Windows includes the exact phrase `unsupported on Windows`
- if unsatisfied requirement entries span both APT and pacman, also state that the runtime in-scope set mixes incompatible system-package managers for provisioning and that the enabled inventory or selected world must be aligned before provisioning can succeed

### `--dry-run` and `--verbose`
- `--dry-run` MUST perform no mutation.
- `--dry-run` MUST still derive the normalized APT requirement set and normalized pacman requirement set, run the read-only presence probes, and enforce the same runtime fail-early rule as non-dry-run execution.
- When either normalized system-package requirement set is non-empty, `--dry-run` stdout MUST print the derived requirement entries in stable order:
  - normalized APT requirement entries first
  - normalized pacman requirement entries second
- When `--verbose` is present and runtime exits `4`, stderr MUST include:
  - the manager-aware missing-requirement rendering in stable order
  - which read-only probe family reported the unsatisfied entries

## Acceptance criteria
- AC-NASP3-01: `substrate world deps current sync|install` with an in-scope APT-backed or pacman-backed item whose required system package is unsatisfied exits `4`, emits stderr that includes `substrate world enable --provision-deps`, and does not execute `apt`, `apt-get`, mutating `dpkg`, or mutating `pacman`.
- AC-NASP3-02: `substrate world deps current install <ITEM...>` scopes fail-early only to the explicit expanded `<ITEM...>` set: if the effective enabled set contains system-package items but the explicit install target expands only to non-system-package items, the system-package fail-early posture does not trigger.
- AC-NASP3-03: `substrate world deps current sync --all` applies the runtime fail-early posture to visible APT-backed and pacman-backed items from inventory, not only to the effective enabled set for `cwd`.
- AC-NASP3-04: If the runtime in-scope set contains both APT-backed and pacman-backed items and at least one requirement is unsatisfied, runtime probes both normalized requirement sets read-only, exits `4` before any non-system-package mutation, and stderr reports the missing entries with manager-aware ordering plus a provisioning-alignment note.
- AC-NASP3-05: If every requirement in the normalized APT requirement set and normalized pacman requirement set is already satisfied, runtime treats the system-package items as satisfied/no-op and proceeds with the upstream non-system-package install behavior even when both system-package managers are represented in the runtime in-scope set.
- AC-NASP3-06: With `--dry-run`, `substrate world deps current sync|install` performs no mutation, still enforces the runtime fail-early rule, and prints the derived normalized APT and pacman requirement entries to stdout in stable manager-aware order whenever either system-package requirement set is non-empty.
- AC-NASP3-07: With `--verbose` and exit `4` due to unsatisfied system-package requirements, stderr includes the stable manager-aware missing-requirement rendering, and Linux host-native or Windows backends preserve the exact backend-specific guidance phrases required by `contract.md`.
- AC-NASP3-08: Error-path determinism remains fail-closed: schema or normalized-requirement derivation failures exit `2` before any read-only probe or non-system-package mutation, read-only probe connectivity failures exit `3` with actionable stderr, and missing `dpkg-query` or `pacman -Q` inside the selected world is treated as an unsatisfied requirement instead of triggering fallback mutation.

## Out of scope
- Provisioning-time world-manager detection and support gating (`NASP0`).
- Pacman inventory-schema validation and inventory-view rendering (`NASP1`).
- Provisioning-time mixed-manager rejection, request-profile routing, and pacman command execution (`NASP2`).
- Cross-doc reconciliation, platform parity evidence, and smoke/manual validation surfaces (`NASP4`).
