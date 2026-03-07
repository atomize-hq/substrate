# PDLDPM1-spec — Reliable Linux install-state creation and replacement

## Behavior delta (single)
- Existing: successful Linux installs do not yet have a slice-traced contract that pins exactly when `install_state.json` is created or updated, how hosted and dev `--no-world` branches behave, how `--dry-run` suppresses writes, or how updates avoid partial-file replacement risk.
- New: successful non-dry-run Linux runs for both installer scripts perform one best-effort persistence step against the canonical install-state path, create the file when absent, replace it through a same-directory temp file when present, and degrade to a warning without changing installer success behavior when persistence fails.
- Why: persisted Linux distro and package-manager metadata is only useful if the file-creation and update semantics are deterministic across hosted and dev installers without making `--dry-run` stateful or turning metadata persistence into a release blocker.

## Scope
- Trace the contract-defined write matrix for `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh`.
- Define the success-path trigger for Linux persistence, including hosted install, hosted `--no-world`, dev install, and dev `--no-world`.
- Define the no-write boundary for `--dry-run` and non-Linux branches.
- Define canonical-path usage, file creation when `install_state.json` is absent, and same-directory temp-file replacement when it already exists.
- Define the idempotence rule for repeated successful Linux runs and the warning-only failure posture for read, merge, write, and replace failures.

## Inputs (authoritative)
- Installer-facing path rule, write matrix, atomic update rule, and failure posture:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- Install-state schema version and merge boundary for the persisted payload:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- Accepted branch-scope decision for hosted install, hosted `--no-world`, dev install, dev `--no-world`, and `--dry-run`:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`

## Behavior (authoritative)

### Linux write-trigger matrix
- On Linux, `scripts/substrate/install-substrate.sh` MUST perform one persistence step against the canonical install-state path after install work reaches its success path.
- On Linux, `scripts/substrate/install-substrate.sh --no-world` MUST perform the same persistence step. `--no-world` changes world provisioning only; it does not suppress metadata persistence.
- On Linux, `scripts/substrate/dev-install-substrate.sh` MUST perform one persistence step against the canonical install-state path after install work reaches its success path.
- On Linux, `scripts/substrate/dev-install-substrate.sh --no-world` MUST perform the same persistence step and MUST follow the same producer contract as the hosted installer.
- Any installer branch with `--dry-run` MUST NOT read for merge, create, update, temp-write, rename, or replace the canonical install-state path.
- Any non-Linux installer branch MUST NOT create new `host_state.platform.*` metadata for this feature and MUST NOT invent a macOS-specific or Windows-specific substitute schema.

### Canonical path and file-presence rule
- The canonical install-state path for this slice is `<effective install prefix>/install_state.json`.
- If `--prefix` is set, installers in scope MUST read, create, update, and replace only `<prefix>/install_state.json`.
- If `--prefix` is not set, installers in scope MUST read, create, update, and replace only `~/.substrate/install_state.json`.
- When `--prefix` points outside `~/.substrate`, installers in scope MUST NOT also read, create, update, or replace `~/.substrate/install_state.json`.
- When the canonical file does not exist on a successful non-dry-run Linux branch, installers in scope MUST create `install_state.json` at the canonical path as a `schema_version = 1` document.
- The persisted payload written by that document remains limited to the `host_state.platform.*` boundary defined by `install-state-schema-spec.md` and traced in `PDLDPM0`.

### Update mechanics and idempotence
- When the canonical install-state file already exists, installers in scope MUST write a complete replacement JSON document to a temporary file in the same directory as the canonical file and then replace the canonical file with that temporary file.
- Installers in scope MUST NOT truncate and rewrite the canonical file in place.
- Installers in scope MUST NOT leave a partial JSON document at the canonical file path after a failed update attempt.
- The persistence step MUST be idempotent at the persisted `host_state.platform.*` boundary. Re-running the same successful non-dry-run Linux install against the same detected inputs MUST converge on the same persisted platform values at the canonical path.

### Warning-only failure posture
- Persistence is best-effort and warning-only.
- If detection inputs needed for `host_state.platform.*` are partially unavailable, installers in scope MUST continue the install and persist only the subset that remains available under the schema rules.
- Missing `/etc/os-release` MUST NOT fail the install. When package-manager metadata remains available, installers in scope MUST persist the package-manager fields and leave the unavailable os-release fields absent.
- If reading the existing install-state file fails, merging the new metadata fails, writing the temporary file fails, or replacing the canonical file fails, installers in scope MUST emit a warning and continue with the existing installer success exit behavior.
- A persistence warning MUST identify the canonical target file and MUST state that install execution continues.
- This feature introduces no new failure exit code. Persistence failure MUST NOT convert an otherwise successful install into a failure exit.

### Cross-slice boundary
- `PDLDPM0` owns the exact `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*` field contract plus the merge semantics inside the JSON document.
- This slice owns only when persistence is attempted, which file path is authoritative, how that file is created or replaced, and how persistence failure degrades without changing installer success semantics.

## Acceptance criteria
- AC-PDLDPM1-01: A successful hosted Linux install performs one persistence step against the canonical install-state path after install work succeeds; with `--prefix /tmp/substrate-root`, installers read, create, update, and replace only `/tmp/substrate-root/install_state.json` and do not also touch `~/.substrate/install_state.json`.
- AC-PDLDPM1-02: A successful hosted Linux install with `--no-world` performs the same persistence step against the canonical install-state path after install work succeeds, and `--no-world` does not suppress metadata persistence.
- AC-PDLDPM1-03: A successful Linux dev install, including dev `--no-world`, performs one persistence step against the canonical install-state path after install work succeeds and follows the same producer contract as the hosted installer instead of a second branch-specific persistence rule.
- AC-PDLDPM1-04: Any installer branch with `--dry-run` performs no read-for-merge, create, update, temp-write, rename, or replace operation against the canonical install-state path; any non-Linux installer branch creates no new `host_state.platform.*` metadata and no substitute schema for this feature.
- AC-PDLDPM1-05: When the canonical install-state file already exists, installers update it by writing a complete replacement JSON document to a temporary file in the same directory and then replacing the canonical file; they do not truncate and rewrite the canonical file in place and do not leave a partial JSON document at the canonical path after a failed update attempt.
- AC-PDLDPM1-06: When the canonical install-state file does not exist on a successful non-dry-run Linux branch, installers create `install_state.json` at the canonical path as a `schema_version = 1` document and keep the persisted payload within the `host_state.platform.*` boundary defined by `install-state-schema-spec.md` and `PDLDPM0`.
- AC-PDLDPM1-07: Re-running the same successful non-dry-run Linux install against the same detected inputs converges on the same persisted `host_state.platform.*` values at the canonical install-state path rather than creating an alternate file or diverging manager or distro metadata.
- AC-PDLDPM1-08: If detection inputs are partially unavailable, or reading, merging, temp-writing, or replacing the canonical install-state file fails, installers emit a warning that names the canonical target file and states that install execution continues; missing `/etc/os-release` leaves only the unavailable os-release fields absent while package-manager fields remain eligible for persistence, and persistence failure introduces no new failure exit code.

## Out of scope
- Defining the exact `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*` leaf-field schema, merge policy, or absence semantics inside the JSON document.
- Changing package-manager detection vocabulary, `pkg_manager.source` vocabulary, or `/etc/os-release` parsing behavior owned by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`.
- Defining smoke-harness assertions, operator-doc wording, or downstream consumer behavior that reads the persisted metadata.
