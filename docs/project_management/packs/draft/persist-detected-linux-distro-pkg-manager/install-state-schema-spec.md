# persist-detected-linux-distro-pkg-manager — install-state schema spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

This spec is authoritative for the additive on-disk schema introduced by ADR-0032 in:
- `<resolved SUBSTRATE_HOME>/install_state.json`

This spec applies to successful Linux installs performed by:
- `scripts/substrate/install-substrate.sh`
- `scripts/substrate/dev-install-substrate.sh`

This feature extends the existing `schema_version=1` install-state document. It does not create a second metadata file and it does not change the schema version.

Out of scope (authoritative elsewhere; this feature MUST NOT redefine):
- `SUBSTRATE_HOME` meaning and default path resolution:
  - `docs/reference/env/contract.md`
- Linux package-manager detection, normalized os-release parsing, supported manager names, and `pkg_manager.source` enum semantics:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- Dry-run and exit-code posture:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`

## Schema invariants (authoritative)

- Root `schema_version` MUST remain the integer `1`.
- The file path MUST remain `<resolved SUBSTRATE_HOME>/install_state.json`.
- This feature adds exactly one new subtree:
  - `host_state.platform`
- This feature MUST NOT add any sibling to `host_state.platform` under `host_state`.
- This feature MUST NOT add any persisted platform field outside `host_state.platform`.
- The persisted platform payload is limited to:
  - `host_state.platform.os_release.id`
  - `host_state.platform.os_release.id_like`
  - `host_state.platform.pkg_manager.selected`
  - `host_state.platform.pkg_manager.source`

## Required document shape

When a successful non-dry-run Linux install writes install-state metadata, the resulting JSON document MUST satisfy all of the following:

- Top level:
  - `schema_version`: integer `1`
  - `created_at`: string
  - `updated_at`: string
  - `host_state`: object
- `host_state.group` MUST remain present with:
  - `name`: string, value `substrate`
  - `members_added`: array of strings, sorted ascending, empty when no users were added during readable-schema merge or fresh creation
- `host_state.linger` MUST remain present with:
  - `users`: object, empty when no linger observations were recorded
- `host_state.platform` MUST be present and MUST contain:
  - `pkg_manager`: object
  - `os_release`: object only when at least one `os_release.*` field is available for persistence

## Field contract (authoritative)

### `host_state.platform.pkg_manager.selected`

- Type: string
- Presence: required whenever `host_state.platform` is present
- Value source: MUST exactly match the final selected manager chosen by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- Allowed values: the exact supported package-manager set owned by the external contract

### `host_state.platform.pkg_manager.source`

- Type: string
- Presence: required whenever `host_state.platform` is present
- Value source: MUST exactly match the final selection source emitted by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- Allowed values: the exact `source` enum owned by the external contract

### `host_state.platform.os_release.id`

- Type: string
- Presence: present only when the normalized `ID` value is available from the external detection contract
- Omission rule: MUST be omitted when the normalized `ID` value is unavailable
- Stored value: MUST use the normalized string produced by the external detection contract; the stored value MUST NOT be `<unknown>`, `null`, or any other sentinel

### `host_state.platform.os_release.id_like`

- Type: string
- Presence: present only when the normalized `ID_LIKE` value is available from the external detection contract
- Omission rule: MUST be omitted when the normalized `ID_LIKE` value is unavailable
- Stored value: MUST use the normalized string produced by the external detection contract; the stored value MUST NOT be `<unknown>`, `null`, or any other sentinel

### `host_state.platform.os_release`

- Type: object
- Presence:
  - MUST be present when `host_state.platform.os_release.id` is present
  - MUST be present when `host_state.platform.os_release.id_like` is present
  - MUST be omitted when both `host_state.platform.os_release.id` and `host_state.platform.os_release.id_like` are omitted
- Allowed keys:
  - `id`
  - `id_like`
- Forbidden payloads:
  - empty object `{}` when both child keys are absent
  - placeholder strings for missing values

## Merge and preservation rules

When the existing install-state file is readable JSON with `schema_version=1`, the writer for this feature MUST:

- preserve `created_at`
- refresh `updated_at`
- preserve every readable field outside `host_state.platform`
- preserve the full readable contents of:
  - `host_state.group`
  - `host_state.linger`
- replace the full `host_state.platform` subtree with the current run's platform payload

This feature does not own `host_state.group` or `host_state.linger`. It only requires that those subtrees survive readable `schema_version=1` merges unchanged unless another feature in the same install invocation updates them.

## Examples (authoritative)

Examples below show only the fields relevant to this feature and the existing install-state baseline.

### Fresh successful Linux install with no recorded group or linger change

```json
{
  "schema_version": 1,
  "created_at": "2026-03-07T12:00:00Z",
  "updated_at": "2026-03-07T12:00:00Z",
  "host_state": {
    "group": {
      "name": "substrate",
      "members_added": []
    },
    "linger": {
      "users": {}
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

### Successful Linux install with partial os-release input

In this example, normalized `ID` is available and normalized `ID_LIKE` is unavailable. The missing field is omitted.

```json
{
  "schema_version": 1,
  "created_at": "2026-03-07T12:00:00Z",
  "updated_at": "2026-03-07T12:05:00Z",
  "host_state": {
    "group": {
      "name": "substrate",
      "members_added": []
    },
    "linger": {
      "users": {}
    },
    "platform": {
      "os_release": {
        "id": "arch"
      },
      "pkg_manager": {
        "selected": "pacman",
        "source": "os_release"
      }
    }
  }
}
```

### Readable schema-version-1 merge that preserves prior host-state content

```json
{
  "schema_version": 1,
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2026-03-07T12:10:00Z",
  "host_state": {
    "group": {
      "name": "substrate",
      "existed_before": false,
      "created_by_installer": true,
      "members_added": [
        "alice"
      ]
    },
    "linger": {
      "users": {
        "alice": {
          "enabled_by_substrate": true,
          "state_at_install": "no"
        }
      }
    },
    "platform": {
      "pkg_manager": {
        "selected": "dnf",
        "source": "path_probe"
      }
    }
  }
}
```
