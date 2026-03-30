# NASP1-spec — Pacman schema extension and inventory views

## Behavior delta (single)
- Existing: the effective world-deps inventory contract hard-codes `install.method` as `apt | script | manual`, so authored pacman-backed system-package items are rejected or rendered ambiguously and the pack has no canonical slice that defines pacman-specific schema failures or inventory-view output.
- New: `NASP1` extends the package schema to accept `install.method=pacman` plus `install.pacman`, constrains v1 pacman-backed packages to non-runnable prerequisites, and requires inventory list/show surfaces to expose pacman-backed definitions without collapsing them into another install method.
- Why: later provisioning and runtime slices need a stable, validated pacman inventory shape before manager-aware provisioning routing or fail-early behavior can be implemented safely.

## Scope
- Define the exact `install.method=pacman` schema and `install.pacman` field shape for package inventory items.
- Define the exact pacman-specific validation failures that produce schema/config exit `2`.
- Define the exact pacman-specific mutual-exclusion and absence rules for `install.apt`, `install.script`, `install.script_path`, and `install.manual_instructions`.
- Define the v1 scope restriction that pacman-backed packages are non-runnable prerequisites and therefore do not use entrypoints or wrappers.
- Define the required inventory-view updates for `substrate world deps ... list available` and `substrate world deps current show`.
- Leave world-manager probing, mixed-manager handling, pacman command construction, and runtime fail-early remediation to later slices.

## Inputs (authoritative)
- Shared manager-aware contract and exit-code meanings:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- Schema posture and runnable-scope decisions:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/decision_register.md` (`DR-0001`, `DR-0004`)
- Canonical pacman schema details for this pack:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md`
- Required slice-owned surfaces and acceptance focus:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md`
- Implementation seam and touch boundary:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md`
- Upstream inventory layout, merge, and list/show surface contract that remains authoritative for unchanged fields:
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`

## Behavior (authoritative)

### Slice boundary and produced outcome
- This slice owns only pacman inventory parsing, validation, and resolved-definition rendering.
- The output of this slice is a stable package-schema contract for pacman-backed system-package items plus deterministic list/show view obligations.
- Later slices MAY consume pacman-backed package definitions, but they MUST NOT redefine the schema, the pacman-specific invalid states, or the pacman list/show rendering established here.

### Package schema support
- `install.method` MUST accept `pacman` as an additional package-install method alongside `apt`, `script`, and `manual`.
- A pacman-backed package definition MUST satisfy every rule in:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md`
- `install.pacman` is the only pacman-specific requirement field added by this slice.
- `install.pacman` is an authored ordered list. This slice validates the authored shape but does not sort or de-duplicate it.
- This slice does not add a manager-agnostic translation layer and does not remap package names across distros.

### Pacman-specific validation and exit behavior
- Any invalid pacman-backed package definition is a schema/config error and MUST exit `2`.
- Invalid pacman-backed package definitions MUST fail before any world-agent call, provisioning probe, or system-package command execution.
- Invalid pacman-backed package definitions include:
  - missing or empty `install.pacman` when `install.method=pacman`
  - non-string, empty, or version-pinned `install.pacman[]` entries
  - `install.pacman` present when `install.method` is not `pacman`
  - non-empty `install.apt`, `install.script`, `install.script_path`, or `install.manual_instructions` on a pacman-backed package
  - `runnable: true`, non-empty `entrypoints`, or non-empty `wrappers` on a pacman-backed package

### V1 pacman package scope
- A pacman-backed package is a non-runnable prerequisite in v1.
- Pacman-backed packages therefore use:
  - `runnable: false`
  - empty `entrypoints`
  - empty `wrappers`
- This slice does not add pacman-specific runnable-wrapper generation.
- This slice does not add pacman-specific present semantics beyond the package-definition fields needed by later provisioning and runtime slices.

### Inventory-view rendering obligations
- `substrate world deps current list available [--json]`, `substrate world deps global list available [--json]`, and `substrate world deps workspace list available [--json]` MUST render pacman-backed packages with `method=pacman`.
- In JSON inventory-list output, pacman-backed package rows MUST expose:
  - `"kind": "package"`
  - `"method": "pacman"`
  - `"runnable": false`
  - `"entrypoints": []`
- `substrate world deps current show <item_name> --json` for a pacman-backed package MUST preserve the resolved definition after inventory merge replacement with:
  - `item.install.method = "pacman"`
  - `item.install.pacman = [...]` in authored order
- `substrate world deps current show <item_name>` without `--json` MUST retain `install.method: pacman` and the `install.pacman:` list in the resolved definition output.
- Inventory views MUST NOT collapse pacman-backed packages into `apt`, `script`, or `manual`, and they MUST NOT synthesize non-pacman install subfields for those packages.

### Inventory merge boundary
- Inventory merge and replacement remain owned by the upstream world-deps contract.
- If a closer inventory layer replaces a package with a pacman-backed definition, the pacman-backed definition becomes the resolved definition for `current show` and the list views.
- This slice does not introduce field-level merge for `install.pacman`; the closest winning package definition still replaces the entire package definition.

## Acceptance criteria
- AC-NASP1-01: A package definition with `install.method=pacman`, a non-empty `install.pacman` list of package-name strings, `runnable: false`, and no conflicting install fields is accepted as valid inventory input and is surfaced by `substrate world deps current list available --json` with `"method": "pacman"`.
- AC-NASP1-02: A package definition with `install.method=pacman` and either missing `install.pacman` or `install.pacman: []` fails inventory validation with exit `2` before any world-agent or provisioning execution is attempted.
- AC-NASP1-03: A package definition with `install.method=pacman` and any `install.pacman[]` entry that is non-string, empty after trimming, or contains `=` fails inventory validation with exit `2`.
- AC-NASP1-04: A package definition with `install.method=pacman` and any of `install.apt`, `install.script`, `install.script_path`, or `install.manual_instructions` present in violation of the schema fails inventory validation with exit `2`.
- AC-NASP1-05: A package definition with `install.method=pacman` and `runnable: true`, non-empty `entrypoints`, or non-empty `wrappers` fails inventory validation with exit `2`, enforcing the v1 non-runnable prerequisite scope.
- AC-NASP1-06: A package definition whose `install.method` is `apt`, `script`, or `manual` but that still includes `install.pacman` fails inventory validation with exit `2`.
- AC-NASP1-07: `substrate world deps current show <item_name> --json` for a pacman-backed package preserves the resolved package definition with `item.install.method = "pacman"` and `item.install.pacman` in authored order, and the non-JSON show output retains `install.method: pacman` plus the `install.pacman:` list.
- AC-NASP1-08: If a workspace inventory layer replaces a global package definition with a pacman-backed package definition of the same name, the merged inventory view used by `substrate world deps current list available` and `substrate world deps current show <item_name>` exposes only the pacman-backed resolved definition and does not retain the replaced method from the lower layer.

## Out of scope
- In-world manager detection and provisioning support gating (`NASP0`).
- Mixed-manager enabled-set failure, pacman requirement normalization, and pacman command execution (`NASP2`).
- Runtime fail-early behavior and remediation wording for pacman-backed items (`NASP3`).
- Cross-doc reconciliation, platform parity, and smoke/manual evidence (`NASP4`).
