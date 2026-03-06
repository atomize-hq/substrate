# persist-detected-linux-distro-pkg-manager — install-state schema spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

This spec is authoritative for the additive JSON payload introduced by ADR-0032 at:
- `<resolved SUBSTRATE_HOME>/install_state.json`
- object path: `host_state.platform`

Out of scope:
- `SUBSTRATE_HOME` resolution and default path semantics: `docs/reference/env/contract.md`
- package-manager detection algorithm, selected-manager vocabulary, and `pkg_manager.source` meanings: `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- `/etc/os-release` parsing and normalization rules that produce persisted `os_release.*` values: `docs/project_management/packs/draft/best-effort-distro-package-manager/decision_register.md`
- file-level compatibility handling for corrupt JSON or wrong-schema files: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/compatibility-spec.md`

## Schema policy

- The feature is additive-only.
- The feature extends the existing `install_state.json` document and MUST NOT introduce a second metadata file.
- The feature adds exactly one new subtree: `host_state.platform`.
- The feature MUST preserve compatible pre-existing `host_state.group` and `host_state.linger` content exactly as defined by `compatibility-spec.md`.
- The feature MUST NOT persist any new platform keys outside this list:
  - `host_state.platform.os_release.id`
  - `host_state.platform.os_release.id_like`
  - `host_state.platform.pkg_manager.selected`
  - `host_state.platform.pkg_manager.source`

## Object model

### `host_state.platform`

- Type: object
- Presence:
  - MUST be present on successful non-dry-run Linux installs when at least one persisted platform field is available from the dependency-owned detection pipeline.
  - MUST be omitted on macOS and Windows.
  - MUST be omitted on Linux only when all four feature-owned fields are unavailable for the current write.

### `host_state.platform.os_release`

- Type: object
- Presence:
  - MUST be present when either `host_state.platform.os_release.id` or `host_state.platform.os_release.id_like` is present.
  - MUST be omitted when both `os_release` child fields are unavailable.

#### `host_state.platform.os_release.id`

- Type: string
- Value source:
  - The normalized Linux `ID` value produced by `best-effort-distro-package-manager`.
- Presence:
  - MUST be present when the dependency-owned parser produced a usable `ID` value.
  - MUST be omitted when `ID` was unavailable, unreadable, or normalized to the dependency-owned unknown sentinel.
- Forbidden encodings:
  - MUST NOT be `null`.
  - MUST NOT be an empty string.
  - MUST NOT store sentinel strings such as `<unknown>`.

#### `host_state.platform.os_release.id_like`

- Type: string
- Value source:
  - The normalized Linux `ID_LIKE` value produced by `best-effort-distro-package-manager`.
- Presence:
  - MUST be present when the dependency-owned parser produced a usable `ID_LIKE` value.
  - MUST be omitted when `ID_LIKE` was unavailable, unreadable, or normalized to the dependency-owned unknown sentinel.
- Forbidden encodings:
  - MUST NOT be `null`.
  - MUST NOT be an empty string.
  - MUST NOT store sentinel strings such as `<unknown>`.

### `host_state.platform.pkg_manager`

- Type: object
- Presence:
  - MUST be present when either `host_state.platform.pkg_manager.selected` or `host_state.platform.pkg_manager.source` is present.
  - MUST be omitted when both `pkg_manager` child fields are unavailable.

#### `host_state.platform.pkg_manager.selected`

- Type: string
- Value source:
  - The final package-manager name selected by `best-effort-distro-package-manager`.
- Allowed values:
  - The exact supported package-manager set owned by the dependency contract.
- Presence:
  - MUST be present whenever the upstream detection contract selected a supported package manager for the install run.
- Forbidden encodings:
  - MUST NOT be `null`.
  - MUST NOT be an empty string.
  - MUST NOT restate fallback reasoning inside the value.

#### `host_state.platform.pkg_manager.source`

- Type: string enum
- Value source:
  - The exact source enum emitted by `best-effort-distro-package-manager`.
- Allowed values:
  - `flag`
  - `env`
  - `os_release`
  - `path_probe`
- Presence:
  - MUST be present whenever `host_state.platform.pkg_manager.selected` is present.
- Forbidden encodings:
  - MUST NOT be `null`.
  - MUST NOT be an empty string.
  - MUST NOT use feature-local spellings or aliases.

## Omission rules

- Unavailable platform values MUST be omitted key-by-key.
- A parent object MUST be omitted when all of its feature-owned children are omitted.
- The writer MUST NOT serialize unavailable values as `null`, empty strings, `<unknown>`, `unknown`, `n/a`, or any other sentinel.
- Missing or unreadable `/etc/os-release` data MUST NOT block persistence of `pkg_manager.selected` and `pkg_manager.source`.
- Presence of `pkg_manager.*` without `os_release.*` is valid and MUST be treated as a complete schema-conforming payload.

## Preservation and merge boundary

- On a compatible existing file, the writer MUST preserve:
  - top-level keys other than the feature-owned `host_state.platform` subtree,
  - existing `host_state.group`,
  - existing `host_state.linger`.
- The writer MUST replace the full `host_state.platform` subtree atomically for the current run rather than merging stale child keys forward.
- The writer MUST NOT infer or synthesize `group` or `linger` values from platform detection inputs.

## Examples

### Compatible Linux payload with readable `/etc/os-release`

```json
{
  "schema_version": 1,
  "created_at": "2026-03-06T00:00:00Z",
  "updated_at": "2026-03-06T00:00:00Z",
  "host_state": {
    "group": {
      "name": "substrate",
      "existed_before": false,
      "created_by_installer": true,
      "members_added": ["spenser"]
    },
    "linger": {
      "users": {
        "spenser": {
          "state_at_install": "no",
          "enabled_by_substrate": true
        }
      }
    },
    "platform": {
      "os_release": {
        "id": "ubuntu",
        "id_like": "debian"
      },
      "pkg_manager": {
        "selected": "apt-get",
        "source": "os_release"
      }
    }
  }
}
```

### Compatible Linux payload with unreadable `/etc/os-release`

```json
{
  "schema_version": 1,
  "created_at": "2026-03-06T00:00:00Z",
  "updated_at": "2026-03-06T00:00:00Z",
  "host_state": {
    "group": {
      "name": "substrate",
      "existed_before": true,
      "members_added": []
    },
    "linger": {
      "users": {}
    },
    "platform": {
      "pkg_manager": {
        "selected": "apt-get",
        "source": "path_probe"
      }
    }
  }
}
```

## Invariants

- `host_state.platform` is Linux-only persisted state.
- The schema MUST remain compatible with `schema_version=1`.
- Older uninstall consumers that read only `host_state.group` and `host_state.linger` MUST continue to function without needing to understand `host_state.platform`.
