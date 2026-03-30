# PDLDPM1-spec - Make `install_state.json` writes reliable

## Behavior delta (single)
- Existing: `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh` only write `install_state.json` on Linux when the current run records host-state events and world provisioning stays enabled, so successful `--no-world` or no-event Linux installs can exit without a canonical metadata file.
- New: Every successful Linux producer flow in both installers creates or updates `<effective_prefix>/install_state.json` through a same-directory temp-file replacement flow, even when the current run recorded no `host_state.group` or `host_state.linger` deltas; hosted `--dry-run` and non-Linux flows remain no-write.
- Why: Later guidance consumers need one reliable post-install metadata file that does not depend on world provisioning or incidental group/linger side effects.

## Scope
- Define the exact write-trigger matrix for hosted install, hosted `--no-world`, dev install, and dev `--no-world`.
- Define the exact no-write branches for hosted `--dry-run` and non-Linux installs.
- Define the create/update, idempotency, and warning-only degradation rules for `install_state.json`.
- Define the temp-file path and single-replacement invariant used for successful writes.
- Keep payload field ownership with `PDLDPM0` and `install-state-schema-spec.md`.

## Inputs (authoritative)
- Operator-facing path, write matrix, atomicity, and failure-posture contract: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
- Schema, merge, and compatibility rules for the written document: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
- Accepted write-scope decision: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md` (`DR-0001`, `DR-0004`)
- Current implementation surfaces that this slice constrains:
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/dev-install-substrate.sh`

## Behavior (authoritative)
### Successful Linux write matrix
- A successful Linux run of `scripts/substrate/install-substrate.sh` without `--dry-run` MUST create or update `<effective_prefix>/install_state.json`.
- A successful Linux run of `scripts/substrate/install-substrate.sh --no-world` MUST create or update `<effective_prefix>/install_state.json`.
- A successful Linux run of `scripts/substrate/dev-install-substrate.sh` MUST create or update `<effective_prefix>/install_state.json`.
- A successful Linux run of `scripts/substrate/dev-install-substrate.sh --no-world` MUST create or update `<effective_prefix>/install_state.json`.
- Hosted installer `--dry-run` MUST NOT create or update the canonical file, a temp file, or a metadata-only parent directory.
- Successful macOS and Windows runs MUST NOT create or update `host_state.platform.*` metadata under this slice.
- This slice defines persistence only inside an otherwise successful Linux install. If another installer step already failed, this slice does not require the script to force a metadata write before exiting non-zero.

### Create/update semantics and schema-level idempotency
- If the canonical file is absent, a successful Linux producer flow MUST create `<effective_prefix>/install_state.json` with `schema_version = 1` even when the current run emitted no `host_state.group` or `host_state.linger` changes.
- If the canonical file exists and is readable, a successful Linux rewrite MUST preserve existing `host_state.group`, `host_state.linger`, and unknown fields according to `install-state-schema-spec.md` while refreshing the current `host_state.platform` payload.
- If the canonical file is unreadable, invalid JSON, or carries a non-`1` `schema_version`, the installer MUST emit warning-only diagnostics and seed the next successful write from a fresh `schema_version = 1` document rather than failing the install.
- Repeating the same successful Linux producer flow with unchanged upstream detection outputs and the same canonical path MUST remain idempotent at the schema level: exactly one canonical file exists at `<effective_prefix>/install_state.json`, `schema_version` remains `1`, the accepted `host_state.platform` nesting remains singular, and repeated runs do not introduce duplicate legacy membership entries or alternate metadata files.

### Temp-file replacement and failure posture
- The writer MUST render the next complete document into `<effective_prefix>/install_state.json.tmp`.
- The temp file MUST live in the same directory as `<effective_prefix>/install_state.json`.
- The replace step MUST occur only after the temp file contains a complete JSON document.
- The replace step MUST use a single file-replacement operation from `<effective_prefix>/install_state.json.tmp` to `<effective_prefix>/install_state.json`.
- In-place truncation of `install_state.json` is not allowed.
- If the temp-file write or the replace step fails during an otherwise successful Linux install, the installer MUST emit warning-only diagnostics, keep the install exit status successful, leave any preexisting canonical file unchanged, and remove the temp file when removal succeeds.

## Acceptance criteria
- AC-PDLDPM1-01: On Linux, successful hosted install, hosted `--no-world`, dev install, and dev `--no-world` each create or update `<effective_prefix>/install_state.json` even when the run recorded no `host_state.group` or `host_state.linger` deltas; persistence does not require `world_enabled=1` as a precondition.
- AC-PDLDPM1-02: Hosted installer `--dry-run` creates no `install_state.json`, no `install_state.json.tmp`, and no metadata-only parent directory; successful macOS and Windows runs do not create `host_state.platform.*` persistence for this ADR.
- AC-PDLDPM1-03: Starting from no canonical file, a successful Linux run creates a complete `schema_version = 1` document at `<effective_prefix>/install_state.json`; starting from a readable schema-valid file with existing `host_state.group`, `host_state.linger`, and unknown sibling keys, two consecutive successful Linux runs with unchanged upstream detection outputs leave one canonical file with `schema_version = 1`, preserved legacy and unknown data, and one accepted `host_state.platform` block rather than alternate metadata structures.
- AC-PDLDPM1-04: Starting from an unreadable, invalid-JSON, or `schema_version != 1` canonical file, a successful Linux run emits warning-only diagnostics and still finishes successfully; when the replacement write succeeds, the resulting canonical file is a complete `schema_version = 1` document at the accepted path.
- AC-PDLDPM1-05: If writing `<effective_prefix>/install_state.json.tmp` or replacing the canonical file fails during an otherwise successful Linux install, the installer emits a warning, preserves install success, leaves any preexisting canonical file unchanged, and removes the temp file when removal succeeds.
- AC-PDLDPM1-06: A successful update writes the next document to `<effective_prefix>/install_state.json.tmp` in the same directory as the canonical file and performs a single replace step only after the temp file contains a complete JSON document; the implementation never truncates `install_state.json` in place.

## Out of scope
- Defining the exact `host_state.platform.os_release.*` and `host_state.platform.pkg_manager.*` field payload remains owned by `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM0/PDLDPM0-spec.md` and `install-state-schema-spec.md`.
- Smoke-harness assertions and operator-facing validation evidence remain owned by `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/slices/PDLDPM2/PDLDPM2-spec.md`.
- Package-manager detection precedence, manager vocabulary, and `pkg_manager.source` ownership remain owned by `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`.
- Hosted uninstaller HOME-versus-prefix reconciliation remains a follow-up outside this slice.
