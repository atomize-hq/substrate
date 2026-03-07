# persist-detected-linux-distro-pkg-manager — install state schema spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for the additive `install_state.json` schema introduced by `persist-detected-linux-distro-pkg-manager`.
- This spec defines:
  - the canonical persisted field paths for detected Linux distro and package manager metadata,
  - the compatibility rules for extending existing `install_state.json` files, and
  - the merge rules that preserve existing install-state data while writing the new metadata.

Out of scope (authoritative elsewhere; this feature MUST NOT redefine):
- Package manager detection precedence, supported manager vocabulary, and source vocabulary:
  - `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- Installer write path selection and atomic temp-file replace behavior:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- Legacy `host_state.group` and `host_state.linger` sub-schema:
  - existing install-state producers remain authoritative for those JSON values

## Format

- Format: JSON
- Canonical file name: `install_state.json`
- Canonical location:
  - `<effective_prefix>/install_state.json`
  - `$SUBSTRATE_HOME/install_state.json` is the operator-facing alias when the effective prefix is the default user-scoped install root
- Top-level `schema_version` remains fixed at integer `1`

## Compatibility policy (explicit)

- Backward compatibility: additive-only. This feature MUST NOT rename or remove any existing `install_state.json` fields.
- Forward compatibility: consumers MUST ignore unknown fields.
- Legacy-file compatibility: an existing `install_state.json` that lacks `host_state.platform` remains valid input for upgrade writes.
- Schema-version policy: this feature MUST NOT bump `schema_version`; upgraded files remain on `schema_version = 1`.
- Preservation policy: writers MUST preserve any existing `host_state.group` and `host_state.linger` JSON values verbatim.

## Schema additions and preserved fields (authoritative)

### Top-level invariants

- `schema_version`
  - Type: integer
  - Required: yes
  - Allowed values:
    - `1`
- `host_state`
  - Type: object
  - Required: yes when any host metadata is persisted

### Preserved legacy fields

- `host_state.group`
  - Type: existing value
  - Required: no
  - Semantics:
    - This spec does not redefine the JSON shape.
    - If the field exists before a rewrite, the writer MUST preserve the exact JSON value.
- `host_state.linger`
  - Type: existing value
  - Required: no
  - Semantics:
    - This spec does not redefine the JSON shape.
    - If the field exists before a rewrite, the writer MUST preserve the exact JSON value.

### `host_state.platform` (new)

- `host_state.platform`
  - Type: object
  - Required: yes for Linux installer flows that persist detected distro and package manager metadata
  - Required: no for non-Linux flows or dry-run flows that do not persist install state
  - Constraints:
    - If present, it MUST contain both `os_release` and `pkg_manager`.
- `host_state.platform.os_release`
  - Type: object
  - Required: yes when `host_state.platform` is present
- `host_state.platform.os_release.id`
  - Type: string
  - Required: yes when `host_state.platform.os_release` is present
  - Semantics:
    - MUST copy the detector's normalized distro identifier verbatim.
    - MUST persist the literal string `<unknown>` when the detector emitted `<unknown>` for `/etc/os-release` `ID`.
- `host_state.platform.os_release.id_like`
  - Type: string
  - Required: yes when `host_state.platform.os_release` is present
  - Semantics:
    - MUST copy the detector's normalized distro-family string verbatim.
    - MUST persist the literal string `<unknown>` when the detector emitted `<unknown>` for `/etc/os-release` `ID_LIKE`.
- `host_state.platform.pkg_manager`
  - Type: object
  - Required: yes when `host_state.platform` is present
- `host_state.platform.pkg_manager.selected`
  - Type: string enum
  - Required: yes when `host_state.platform.pkg_manager` is present
  - Allowed values:
    - `apt-get`
    - `dnf`
    - `yum`
    - `pacman`
    - `zypper`
  - Semantics:
    - MUST copy the selected package manager emitted by `best-effort-distro-package-manager`.
    - MUST NOT be re-derived by the persistence writer.
- `host_state.platform.pkg_manager.source`
  - Type: string enum
  - Required: yes when `host_state.platform.pkg_manager` is present
  - Allowed values:
    - `flag`
    - `env`
    - `os_release`
    - `path_probe`
  - Semantics:
    - MUST copy the detection-contract source value verbatim.
    - MUST use the same vocabulary as `best-effort-distro-package-manager`.

## Emission and merge rules (authoritative)

- Writers MUST create `install_state.json` with `schema_version = 1` when the file does not already exist and the Linux install flow persists metadata.
- Writers MUST read the existing `install_state.json` before rewriting it.
- Writers MUST preserve existing unknown fields unless another authoritative spec for that field says otherwise.
- Writers MUST preserve the existing JSON values at `host_state.group` and `host_state.linger` exactly as they were read.
- Writers MUST write `host_state.platform.os_release.id` and `host_state.platform.os_release.id_like` exactly as produced by distro detection.
- Writers MUST write the literal `<unknown>` sentinel when distro detection emitted `<unknown>` for missing or unreadable `/etc/os-release` data.
- Writers MUST write `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` exactly as produced by package-manager detection.
- Writers MUST update the file through the contract-defined temp-file replace flow so the resulting file is a complete JSON document after each successful write.
- Writers MUST NOT remove `host_state.group` or `host_state.linger` solely because the current install run did not emit new values for those fields.

## Invalid states (authoritative)

- `schema_version != 1`
- `host_state.platform` present without `host_state.platform.os_release`
- `host_state.platform` present without `host_state.platform.pkg_manager`
- `host_state.platform.pkg_manager.selected` outside the allowed enum
- `host_state.platform.pkg_manager.source` outside the allowed enum
- `host_state.platform.os_release.id` or `host_state.platform.os_release.id_like` omitted when `host_state.platform.os_release` is present

## Examples (authoritative)

Examples below show the fields controlled by this feature. If an existing file already contains `host_state.group` or `host_state.linger`, those JSON values remain unchanged by the upgrade write.

### Fresh Linux install with distro-based selection

```json
{
  "schema_version": 1,
  "host_state": {
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

### Existing file upgraded with operator override

```json
{
  "schema_version": 1,
  "host_state": {
    "platform": {
      "os_release": {
        "id": "fedora",
        "id_like": "fedora"
      },
      "pkg_manager": {
        "selected": "dnf",
        "source": "flag"
      }
    }
  }
}
```

### Linux install with missing `/etc/os-release`

```json
{
  "schema_version": 1,
  "host_state": {
    "platform": {
      "os_release": {
        "id": "<unknown>",
        "id_like": "<unknown>"
      },
      "pkg_manager": {
        "selected": "apt-get",
        "source": "path_probe"
      }
    }
  }
}
```
