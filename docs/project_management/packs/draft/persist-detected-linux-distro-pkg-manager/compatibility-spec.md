# persist-detected-linux-distro-pkg-manager — compatibility spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

This spec is authoritative for compatibility rules tied to the additive `host_state.platform.*` install-state expansion introduced by ADR-0032.

This spec covers:
- install-time writer compatibility for `scripts/substrate/install-substrate.sh`
- install-time writer compatibility for `scripts/substrate/dev-install-substrate.sh`
- uninstall-reader compatibility for `scripts/substrate/uninstall-substrate.sh`
- downstream guidance-reader fallback rules for persisted Linux platform metadata

Out of scope (authoritative elsewhere; this feature MUST NOT redefine):
- install success/failure and warning posture:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- package-manager detection semantics and `pkg_manager.source` vocabulary:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- `SUBSTRATE_HOME` path semantics:
  - `docs/reference/env/contract.md`

## Compatibility policy (explicit)

- Schema version lock:
  - `install_state.json` MUST remain `schema_version=1`
  - this feature MUST NOT bump the schema version
- Evolution mode:
  - additive-only under `host_state.platform`
  - no rename, removal, or semantic redefinition of `host_state.group`
  - no rename, removal, or semantic redefinition of `host_state.linger`
- Reader posture:
  - readers that do not know about `host_state.platform` MUST continue functioning by ignoring that subtree
  - readers that want platform metadata MUST fall back to runtime detection when the file is missing, unreadable, on the wrong schema version, or lacks the requested platform keys
- Writer posture:
  - a readable `schema_version=1` document MUST be merged
  - an unreadable or wrong-schema document MUST be reset to a fresh `schema_version=1` document

## Install-time writer compatibility matrix

### Case 1: install-state file is missing

Writer requirements:
- create `<resolved SUBSTRATE_HOME>/install_state.json`
- write `schema_version=1`
- write fresh `created_at` and `updated_at`
- write the baseline `host_state.group` and `host_state.linger` scaffolding required by the existing install-state contract
- write the current Linux `host_state.platform` payload

Compatibility result:
- existing uninstall cleanup keeps reading a `schema_version=1` document
- later guidance readers can consume persisted platform metadata immediately

### Case 2: install-state file is readable JSON with `schema_version=1`

Writer requirements:
- preserve `created_at`
- refresh `updated_at`
- preserve existing readable content outside `host_state.platform`
- preserve existing readable `host_state.group`
- preserve existing readable `host_state.linger`
- replace `host_state.platform` with the current run's payload

Compatibility result:
- uninstall cleanup keeps the same group/linger evidence it already depends on
- repeated installs refresh platform metadata without fragmenting the file contract

### Case 3: install-state file is readable JSON but `schema_version != 1`

Writer requirements:
- emit a warning
- ignore prior file contents for merge purposes
- write a fresh `schema_version=1` document with the current run's install-state content
- continue the successful install path without introducing a new non-zero exit

Compatibility result:
- the file returns to the only supported schema version
- prior incompatible content is discarded instead of being partially merged

### Case 4: install-state file exists but is unreadable or corrupt JSON

Writer requirements:
- emit a warning
- ignore prior file contents for merge purposes
- write a fresh `schema_version=1` document with the current run's install-state content
- continue the successful install path without introducing a new non-zero exit

Compatibility result:
- the file becomes readable again for subsequent installs and guidance readers
- previously unreadable content is discarded because it cannot be merged safely

### Case 5: metadata write cannot complete

Writer requirements:
- emit a warning
- continue the successful install path without introducing a new non-zero exit

Compatibility result:
- the feature remains fail-open
- consumers must rely on the existing fallback rule because persisted metadata is unavailable or stale

## Existing uninstall-reader compatibility

`scripts/substrate/uninstall-substrate.sh` remains a `schema_version=1` reader focused on `host_state.group` and `host_state.linger`.

The compatibility contract for uninstall is:
- the added `host_state.platform` subtree MUST NOT be required for uninstall cleanup
- the added `host_state.platform` subtree MUST NOT change how recorded group membership cleanup is interpreted
- the added `host_state.platform` subtree MUST NOT change how recorded linger cleanup is interpreted
- when uninstall reads:
  - missing install-state metadata
  - corrupt install-state metadata
  - install-state metadata with `schema_version != 1`
  it MUST fall back to manual cleanup guidance instead of failing closed

## Downstream guidance-reader compatibility

Any consumer that wants Linux platform guidance from persisted metadata MUST apply this rule set in order:

1. Read `<resolved SUBSTRATE_HOME>/install_state.json`.
2. If the file is missing, unreadable, or `schema_version != 1`, ignore the file and run runtime detection.
3. If `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` are present, prefer those persisted values.
4. If `host_state.platform.os_release.id` or `host_state.platform.os_release.id_like` is absent, treat the missing key as unavailable input and continue with the data that is present.
5. If the requested platform fields are absent, run runtime detection.

Consumers MUST NOT:
- fail closed because persisted platform metadata is absent
- treat omitted `os_release.*` keys as evidence of invalid data
- synthesize placeholder strings into the persisted file during a read path

## Compatibility examples (authoritative)

### Legacy schema-version-1 document before this feature

```json
{
  "schema_version": 1,
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2025-01-01T00:00:00Z",
  "host_state": {
    "group": {
      "name": "substrate",
      "members_added": [
        "legacy-user"
      ]
    },
    "linger": {
      "users": {}
    }
  }
}
```

### Same document after a readable merge by this feature

```json
{
  "schema_version": 1,
  "created_at": "2025-01-01T00:00:00Z",
  "updated_at": "2026-03-07T12:15:00Z",
  "host_state": {
    "group": {
      "name": "substrate",
      "members_added": [
        "legacy-user"
      ]
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

### Wrong-schema or corrupt input resets to fresh schema-version-1 content

Input examples that trigger reset:

```json
{
  "schema_version": 0,
  "host_state": {
    "group": {
      "members_added": [
        "legacy-user"
      ]
    }
  }
}
```

```text
{invalid
```

Required post-reset shape:

```json
{
  "schema_version": 1,
  "created_at": "2026-03-07T12:20:00Z",
  "updated_at": "2026-03-07T12:20:00Z",
  "host_state": {
    "group": {
      "name": "substrate",
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
