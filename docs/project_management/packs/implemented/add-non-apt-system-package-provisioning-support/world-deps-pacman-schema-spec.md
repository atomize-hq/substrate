# add-non-apt-system-package-provisioning-support — world deps pacman schema spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for the additive world-deps inventory-schema extension introduced by ADR-0033 for `install.method=pacman`.
- This spec defines:
  - the exact `install.method` enum extension,
  - the exact `install.pacman` field shape,
  - pacman-specific package constraints for v1,
  - the mutual-exclusion and absence rules for pacman-backed packages, and
  - the inventory-view obligations for resolved pacman-backed package definitions.

Out of scope (authoritative elsewhere; this feature MUST NOT redefine):
- Inventory directories, filename-to-name matching, bundle schema, inventory merge/replacement, enabled-set resolution, and platform filtering:
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
- Provisioning-time manager selection, mixed-manager failure posture, and pacman command execution:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP0/NASP0-spec.md`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP2/NASP2-spec.md`
- Runtime fail-early behavior and remediation wording:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP3/NASP3-spec.md`

## Compatibility policy (explicit)

- Backward compatibility: additive-only. This feature extends the package-install method vocabulary but does not rename or remove existing package or bundle fields.
- Schema-version policy: package files remain on `version: 1`; this feature does not introduce a new inventory schema version.
- Layout policy: package files remain `deps/packages/<dep_name>.yaml`; bundle files remain `deps/bundles/<dep_name>.yaml`.
- Merge policy: closest-scope full replacement by package name remains unchanged. This spec changes only the winning package definition when that definition uses `install.method=pacman`.
- View policy: inventory views MUST expose pacman-backed packages as pacman-backed packages. They MUST NOT remap pacman definitions into `apt`, `script`, or `manual`.

## Schema extension (authoritative)

### `install.method`

- Type: string enum
- Allowed values after this feature:
  - `apt`
  - `pacman`
  - `script`
  - `manual`

### `install.pacman`

- Type: ordered YAML list of strings
- Required: yes iff `install.method=pacman`
- Required: no otherwise
- Entry rules:
  - every entry MUST be a string scalar
  - every entry MUST remain non-empty after trimming leading and trailing ASCII whitespace
  - every entry MUST name one pacman package only
  - every entry MUST NOT contain `=` because version pinning is unsupported in v1
- Ordering rules:
  - authored list order is preserved in the stored package definition and in resolved inventory views
  - this schema does not sort or de-duplicate `install.pacman`
  - later provisioning-time normalization owns de-duplication and ordering of the derived pacman requirement set

### Pacman-backed package constraints (v1)

- A package with `install.method=pacman` is a non-runnable system-package prerequisite in v1.
- A package with `install.method=pacman` MUST set `runnable: false`.
- A package with `install.method=pacman` MUST omit `entrypoints` or set `entrypoints: []`.
- A package with `install.method=pacman` MUST omit `wrappers` or set `wrappers: []`.
- `description`, `platforms`, and `probe` remain governed by the upstream world-deps contract.
- If `probe.command` is present on a pacman-backed package, it does not widen runnable-wrapper behavior, does not redefine pacman present semantics, and does not change provisioning-time requirement derivation from `install.pacman`.

### Mutual-exclusion and absence rules

- If `install.method=pacman`:
  - `install.pacman` MUST be present and non-empty
  - `install.apt` MUST be omitted or `[]`
  - `install.script` MUST be absent
  - `install.script_path` MUST be absent
  - `install.manual_instructions` MUST be absent
- If `install.method=apt`:
  - `install.pacman` MUST be omitted or `[]`
- If `install.method=script`:
  - `install.pacman` MUST be omitted or `[]`
- If `install.method=manual`:
  - `install.pacman` MUST be omitted or `[]`

## Composition with the upstream contract (authoritative)

- Every package field not named in this spec keeps the meaning already defined by:
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
- This spec does not change bundle definitions.
- This spec does not change enabled-set evaluation, bundle expansion, or workspace/global inventory replacement rules.
- This spec does not define a distro-translation layer. Pacman-backed package names remain author-authored manager-specific strings.

## Inventory-view obligations (authoritative)

- `substrate world deps current list available [--json]`, `substrate world deps global list available [--json]`, and `substrate world deps workspace list available [--json]` MUST render pacman-backed packages with `method=pacman`.
- In JSON inventory-list views, pacman-backed package rows MUST expose `"method": "pacman"`.
- Pacman-backed package rows in inventory-list views MUST expose `runnable=false` and empty `entrypoints`.
- `substrate world deps current show <item_name> --json` MUST preserve the resolved pacman-backed package definition as:
  - `item.install.method = "pacman"`
  - `item.install.pacman = [...]` in authored order after inventory merge replacement
- `substrate world deps current show <item_name>` without `--json` MUST retain `install.method: pacman` and the `install.pacman:` list in the resolved definition output.
- Inventory views MUST NOT synthesize `install.apt`, `install.script`, `install.script_path`, or `install.manual_instructions` for pacman-backed packages.

## Invalid states (authoritative)

- `install.method=pacman` with `install.pacman` omitted
- `install.method=pacman` with `install.pacman: []`
- `install.method=pacman` with any `install.pacman[]` entry that is not a string
- `install.method=pacman` with any `install.pacman[]` entry that is empty after ASCII-whitespace trimming
- `install.method=pacman` with any `install.pacman[]` entry containing `=`
- `install.method=pacman` with non-empty `install.apt`
- `install.method=pacman` with `install.script` present
- `install.method=pacman` with `install.script_path` present
- `install.method=pacman` with `install.manual_instructions` present
- `install.method=pacman` with `runnable: true`
- `install.method=pacman` with non-empty `entrypoints`
- `install.method=pacman` with non-empty `wrappers`
- `install.pacman` present with `install.method` equal to `apt`, `script`, or `manual`

## Examples (authoritative)

### Valid pacman-backed prerequisite package

```yaml
version: 1
name: ripgrep-prereq
description: Ripgrep is required inside Arch-family worlds.
runnable: false
platforms:
  - linux
install:
  method: pacman
  pacman:
    - ripgrep
```

### Valid pacman-backed package with multiple requirements

```yaml
version: 1
name: build-essential-arch
description: Arch-family build prerequisites for native crates.
runnable: false
install:
  method: pacman
  pacman:
    - base-devel
    - pkgconf
```

### Invalid pacman-backed package: missing `install.pacman`

```yaml
version: 1
name: bad-pacman-missing-list
runnable: false
install:
  method: pacman
```

Invalid because `install.method=pacman` requires a non-empty `install.pacman` list.

### Invalid pacman-backed package: version pin attempt

```yaml
version: 1
name: bad-pacman-version-pin
runnable: false
install:
  method: pacman
  pacman:
    - ripgrep=14.1.0
```

Invalid because `install.pacman[]` entries MUST name one package only and MUST NOT contain `=`.

### Invalid pacman-backed package: runnable wrapper surface

```yaml
version: 1
name: bad-pacman-runnable
runnable: true
entrypoints:
  - rg
install:
  method: pacman
  pacman:
    - ripgrep
```

Invalid because pacman-backed packages are non-runnable prerequisites in v1 and therefore MUST NOT use runnable entrypoints or wrappers.
