# persist-detected-linux-distro-pkg-manager — contract surface

This file is the single authoritative contract for the installer-facing behavior introduced by ADR-0032: persist detected Linux distro and package-manager metadata into the install-state file used by Substrate installers.

Decision inputs:
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`

This contract MUST NOT redefine:
- selected package-manager vocabulary or `pkg_manager.source` vocabulary owned by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- JSON field paths, types, examples, or merge rules owned by `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- exit-code taxonomy owned by `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Invariants

- This feature is persistence-only. It MUST NOT add new commands, new flags, new environment variables, or new exit-code classes.
- The canonical persisted file for this feature is `install_state.json`. This feature MUST NOT introduce a second metadata file for distro or package-manager state.
- All writes introduced by this feature MUST stay under the effective install prefix. The feature MUST NOT create or update a second copy under another prefix.
- Persisted package-manager metadata MUST be copied verbatim from the upstream detection contract. This feature MUST NOT remap manager strings or source strings.
- Hosted installer and dev installer are one producer contract for this feature. They MUST follow the same path rule, write matrix, and failure posture.

## Commands in scope

- `scripts/substrate/install-substrate.sh`
- `scripts/substrate/dev-install-substrate.sh`

No new CLI surface is introduced. The existing `--prefix`, `--no-world`, and `--dry-run` behaviors remain authoritative; this contract only defines how those existing branches affect metadata persistence.

## Canonical path rule

Definitions:
- Effective install prefix: the resolved installer prefix. If `--prefix` is set, the effective install prefix is that exact path. If `--prefix` is not set, the effective install prefix is `~/.substrate`.
- Effective Substrate home: for the installer flows in scope, this is the same directory as the effective install prefix.
- Canonical install-state path: `<effective install prefix>/install_state.json`

Required rule:
- `$SUBSTRATE_HOME/install_state.json` and `<prefix>/install_state.json` refer to the same canonical file for this feature: `<effective install prefix>/install_state.json`.
- If `--prefix` points outside `~/.substrate`, installers in scope MUST read, create, update, and replace only the file under that explicit prefix.
- Installers in scope MUST NOT also read, create, or update `~/.substrate/install_state.json` when the effective install prefix is another directory.

## Persisted metadata boundary

This feature persists only the Linux host metadata introduced by ADR-0032:
- `host_state.platform.os_release.id`
- `host_state.platform.os_release.id_like`
- `host_state.platform.pkg_manager.selected`
- `host_state.platform.pkg_manager.source`

Required interaction rules:
- `host_state.platform.pkg_manager.selected` and `host_state.platform.pkg_manager.source` MUST be copied verbatim from the outputs defined by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`.
- `/etc/os-release` values MUST be persisted only under the `host_state.platform.os_release.*` paths defined by the schema spec.
- This feature MUST preserve pre-existing install-state content outside the paths listed above, including `host_state.group` and `host_state.linger`.

## Write matrix

The persistence branch is defined by platform, installer mode, and dry-run state.

| Branch | Persistence result |
| --- | --- |
| Hosted install on Linux | MUST perform one persistence step against the canonical install-state path after install work succeeds. |
| Hosted install on Linux with `--no-world` | MUST perform the same persistence step. `--no-world` changes world provisioning only; it does not suppress metadata persistence. |
| Dev install on Linux | MUST perform one persistence step against the canonical install-state path after install work succeeds. |
| Dev install on Linux with `--no-world` | MUST perform the same persistence step. `--no-world` does not suppress metadata persistence. |
| Any installer branch with `--dry-run` | MUST NOT read for merge, create, update, temp-write, rename, or replace the canonical install-state file. |
| Any non-Linux install branch | MUST NOT create new `host_state.platform.*` metadata for this feature and MUST NOT invent a macOS-specific or Windows-specific substitute schema. |

Additional required rules:
- A successful Linux install means the installer reached its success path for package installation and setup. Metadata persistence runs after that success point and does not define installer success on its own.
- The persistence step MUST be idempotent. Re-running the same successful Linux install against the same detected inputs MUST converge on the same persisted `host_state.platform.*` values.

## Atomic update rule

When the canonical install-state file already exists:
- installers in scope MUST update it by writing a complete replacement JSON document to a temporary file in the same directory as the canonical file and then replacing the canonical file with that temporary file
- installers in scope MUST NOT truncate and rewrite the canonical file in place
- installers in scope MUST NOT leave a partial JSON document at the canonical file path after a failed update attempt

When the canonical install-state file does not exist:
- installers in scope MUST create it at the canonical path using the same schema version already in use for install state

## Failure posture

Persistence is best-effort and warning-only.

Required rules:
- If detection inputs needed for `host_state.platform.*` are unavailable, installers in scope MUST continue the install and persist the subset of metadata that remains available under the schema rules.
- Missing `/etc/os-release` MUST NOT fail the install. When package-manager metadata is still available, installers in scope MUST persist the package-manager fields and leave the unavailable os-release fields absent.
- If reading the existing install-state file fails, merging the new metadata fails, writing the temporary file fails, or replacing the canonical file fails, installers in scope MUST emit a warning and continue with the existing installer success exit behavior.
- A persistence warning MUST identify the canonical target file and MUST state that install execution continues.
- This feature introduces no new failure exit code. Persistence failure MUST NOT convert an otherwise successful install into a failure exit.

## Future-consumer read precedence

This pack establishes the read rule that downstream consumers MUST use when they need Linux distro or package-manager metadata from install state:
- Prefer the persisted values at the canonical install-state path first.
- If the canonical file is missing, unreadable, or the relevant fields are absent, fall back to runtime detection.
- Consumers MUST NOT treat absence of the persisted metadata as proof that the host is non-Linux.
- Consumers MUST NOT search alternate install-state paths when the effective install prefix is known.

## Exit codes

- Taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Overrides: none

This feature introduces no new success code and no new failure code.

## Logging and operator-visible output

- This feature introduces no new structured telemetry field or trace schema field.
- The machine-readable contract added by this feature is the persisted content in `install_state.json`.
- Operator-visible signaling for persistence degradation is limited to the warning behavior in the Failure posture section.

## Platform guarantees

- Linux: hosted install and dev install MUST follow the canonical path rule, write matrix, atomic update rule, and failure posture in this contract.
- macOS: this feature MUST NOT add Linux distro or package-manager persistence behavior.
- Windows: this feature MUST NOT add Linux distro or package-manager persistence behavior.

## Out-of-scope boundary

- This feature MUST NOT change hosted uninstaller path resolution.
- Any change to `scripts/substrate/uninstall-substrate.sh` path handling remains a separate follow-up outside this contract.
