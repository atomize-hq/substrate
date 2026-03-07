# persist-detected-linux-distro-pkg-manager — install-state schema spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

This spec is authoritative for the additive `install_state.json` schema boundary introduced by ADR-0032 at:
- `<effective install prefix>/install_state.json`

This spec is not authoritative for:
- canonical path resolution, write-trigger branches, atomic replacement, or warning-only failure posture: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- package-manager vocabulary, `pkg_manager.source` vocabulary, or `/etc/os-release` parsing rules: `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- exit-code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Format and base document invariants

- Format: JSON
- Canonical file name: `install_state.json`
- Canonical top-level version field:
  - field name: `schema_version`
  - type: integer
  - required value: `1`
  - this ADR does not rename the field and does not bump the value

Existing v1 fields that remain unchanged by this ADR:
- `created_at`
- `updated_at`
- `host_state.group`
- `host_state.linger`

The examples in this spec include `created_at` and `updated_at` because those fields already exist in the v1 document. This ADR does not change their names or their timestamp role.

## Compatibility policy

- Compatibility is additive-only for pre-existing v1 fields.
- Writers MUST keep `schema_version = 1`.
- Writers MUST NOT rename or remove pre-existing v1 fields, including `created_at`, `updated_at`, `host_state.group`, and `host_state.linger`.
- Consumers that do not know `host_state.platform.*` MUST ignore those unknown keys.
- This ADR does not require a migration, backfill job, or second metadata file.
- The persisted representation for unavailable metadata is absence. Writers MUST NOT store placeholder strings such as `<unknown>`, empty arrays, or `null` for the four leaf fields owned by this ADR.

## New schema surface (authoritative)

### Container objects

- `host_state`
  - type: object
  - presence:
    - present when the document contains any `host_state.*` child
- `host_state.platform`
  - type: object
  - presence:
    - present when at least one child under `host_state.platform.*` is present after merge
    - absent when no child under `host_state.platform.*` remains after merge
- `host_state.platform.os_release`
  - type: object
  - presence:
    - present when `host_state.platform.os_release.id` or `host_state.platform.os_release.id_like` is present after merge
    - absent when both owned os-release leaf fields are absent and no preserved sibling keys remain
- `host_state.platform.pkg_manager`
  - type: object
  - presence:
    - present when `host_state.platform.pkg_manager.selected` or `host_state.platform.pkg_manager.source` is present after merge
    - absent when both owned package-manager leaf fields are absent and no preserved sibling keys remain

### Leaf fields

| Path | Type | Presence / absence | Storage rule |
| --- | --- | --- | --- |
| `host_state.platform.os_release.id` | string | Present only when the install run has a non-empty `ID` value to persist. Absent otherwise. | Stores the detected distro `ID` string. Writers MUST store one scalar string and MUST NOT persist `<unknown>`. |
| `host_state.platform.os_release.id_like` | string | Present only when the install run has a non-empty `ID_LIKE` value to persist. Absent otherwise. | Stores the detected `ID_LIKE` value as one string. Writers MUST NOT store an array, object, token list, or `<unknown>`. |
| `host_state.platform.pkg_manager.selected` | string | Present only when the install run has a selected-manager output to persist. Absent otherwise. | Stores the selected manager string emitted by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` without translation or remapping. |
| `host_state.platform.pkg_manager.source` | string | Present only when the install run has a source output to persist. Absent otherwise. | Stores the source string emitted by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md` without translation or remapping. |

## Merge and preservation rules (authoritative)

The post-write document for a successful Linux install MUST follow this merge policy:

1. Start from the existing readable `schema_version = 1` JSON document when one exists.
2. Preserve every key outside the four owned leaf paths:
   - `host_state.platform.os_release.id`
   - `host_state.platform.os_release.id_like`
   - `host_state.platform.pkg_manager.selected`
   - `host_state.platform.pkg_manager.source`
3. Preserve `host_state.group` exactly as it existed before this feature's platform-merge step, except for changes already owned by the pre-existing installer metadata flow.
4. Preserve `host_state.linger` exactly as it existed before this feature's platform-merge step, except for changes already owned by the pre-existing installer metadata flow.
5. For each owned leaf path, current-run availability controls final presence:
   - when the current install run provides a value, writers MUST set that value at the owned leaf path
   - when the current install run does not provide a value, writers MUST leave that owned leaf path absent in the post-write document
6. Writers MUST create only the containing objects required for the owned leaf fields that remain present after merge.
7. Writers MUST remove any owned empty container object that becomes empty after step 5, unless that object still contains preserved sibling keys:
   - `host_state.platform.os_release`
   - `host_state.platform.pkg_manager`
   - `host_state.platform`
8. Writers MUST preserve unknown sibling keys under `host_state`, `host_state.platform`, `host_state.platform.os_release`, and `host_state.platform.pkg_manager`.

Rule 5 defines the missing-`/etc/os-release` case:
- when `ID` and `ID_LIKE` are unavailable for the current run, `host_state.platform.os_release.id` and `host_state.platform.os_release.id_like` are absent in the post-write document
- when package-manager metadata remains available for the current run, `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` remain present

## Canonical JSON examples

Examples below are complete v1 documents. Field order is illustrative; consumers MUST treat the JSON object graph as unordered.

### Existing file upgraded with `host_state.platform.*`

```json
{
  "schema_version": 1,
  "created_at": "2026-03-07T12:00:00Z",
  "updated_at": "2026-03-07T12:05:00Z",
  "host_state": {
    "group": {
      "name": "substrate",
      "existed_before": false,
      "created_by_installer": true,
      "members_added": [
        "spenser"
      ]
    },
    "linger": {
      "users": {
        "spenser": {
          "state_at_install": "no",
          "enabled_by_substrate": false
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

### Linux install with missing `/etc/os-release`

```json
{
  "schema_version": 1,
  "created_at": "2026-03-07T12:00:00Z",
  "updated_at": "2026-03-07T12:10:00Z",
  "host_state": {
    "platform": {
      "pkg_manager": {
        "selected": "apt-get",
        "source": "path_probe"
      }
    }
  }
}
```

### Linux install with no group or linger deltas

```json
{
  "schema_version": 1,
  "created_at": "2026-03-07T12:00:00Z",
  "updated_at": "2026-03-07T12:15:00Z",
  "host_state": {
    "platform": {
      "os_release": {
        "id": "arch",
        "id_like": "arch"
      },
      "pkg_manager": {
        "selected": "pacman",
        "source": "os_release"
      }
    }
  }
}
```
