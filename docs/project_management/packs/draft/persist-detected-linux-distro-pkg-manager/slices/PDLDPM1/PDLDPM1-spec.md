# PDLDPM1-spec — Guarantee production installer install-state writes

## Behavior delta (single)
- Existing: `scripts/substrate/install-substrate.sh` can skip `install_state.json` persistence when no `host_state.group` or `host_state.linger` event payload exists, and `--no-world` currently overlaps that skip path instead of preserving the Linux metadata contract.
- New: successful non-dry-run Linux runs through `scripts/substrate/install-substrate.sh` always create or update `<resolved SUBSTRATE_HOME>/install_state.json`, including `--no-world` runs, while classifying existing files per the compatibility spec and keeping metadata persistence fail-open.
- Why: ADR-0032 requires a deterministic post-install Linux metadata artifact on successful installs, even when the install produced no legacy event payload and even when world isolation was disabled.

## Scope
- `scripts/substrate/install-substrate.sh` only.
- Successful non-dry-run Linux install path, including `--no-world`.
- Existing-file classification and merge-versus-replace handling for `install_state.json`.
- Dry-run no-write behavior and fail-open persistence posture.

## Behavior (authoritative)
### Successful Linux write guarantee
- On Linux, a successful non-dry-run `scripts/substrate/install-substrate.sh` invocation MUST leave `<resolved SUBSTRATE_HOME>/install_state.json` present when the script exits `0`.
- The file-presence guarantee applies even when the install emitted no `host_state.group` or `host_state.linger` event payload.
- Passing `--no-world` MUST NOT suppress the same install-state write contract.

### Existing-file classification
- If `<resolved SUBSTRATE_HOME>/install_state.json` is missing, the installer MUST create a fresh `schema_version: 1` document for the current run.
- If the existing file parses as JSON with `schema_version: 1`, the installer MUST preserve compatible content per `compatibility-spec.md`, update `updated_at`, and replace only the current-run `host_state.platform` subtree plus any same-run `group` / `linger` changes.
- If the existing file is corrupt or has a missing or non-`1` `schema_version`, the installer MUST treat it as incompatible, emit a warning, and replace it with a fresh `schema_version: 1` document containing only current-run authoritative data.

### Dry-run and failure posture
- A dry-run invocation MUST NOT create, rewrite, truncate, or merge `<resolved SUBSTRATE_HOME>/install_state.json`.
- Persistence remains fail-open: a read, merge, or write failure during an otherwise successful install MUST NOT introduce a new non-zero exit code.
- On persistence failure, the installer MAY emit a warning to stderr, but it MUST NOT retry with broadened host inspection, write outside the resolved `SUBSTRATE_HOME`, or change the exit-code taxonomy.

## Acceptance criteria
- AC-PDLDPM1-01: A successful non-dry-run Linux execution of `scripts/substrate/install-substrate.sh` that emits no `host_state.group` or `host_state.linger` event payload still exits `0` and leaves `<resolved SUBSTRATE_HOME>/install_state.json` present with `schema_version: 1`.
- AC-PDLDPM1-02: The same successful Linux install invoked with `--no-world` still creates or updates `<resolved SUBSTRATE_HOME>/install_state.json` under the same schema and path contract as a world-enabled run.
- AC-PDLDPM1-03: A `--dry-run` execution of `scripts/substrate/install-substrate.sh` does not create a missing `install_state.json` and does not modify an existing file.
- AC-PDLDPM1-04: With a compatible existing `install_state.json`, the rewritten file preserves `created_at`, unknown top-level keys, and existing `host_state.group` / `host_state.linger` content that the same run did not change, while updating `updated_at` and replacing `host_state.platform` for the current run.
- AC-PDLDPM1-05: With a corrupt existing `install_state.json` or an existing file whose top-level `schema_version` is absent or not `1`, `scripts/substrate/install-substrate.sh` emits a warning, replaces the file with a fresh `schema_version: 1` document built from current-run data, and does not introduce a new non-zero exit solely because persistence needed replacement.

## Out of scope
- Porting the production installer write guarantee to `scripts/substrate/dev-install-substrate.sh`; that parity is owned by `PDLDPM3`.
- Adding new exit codes, new CLI flags, new environment variables, or a second metadata file.
- Changing macOS or Windows behavior beyond the explicit no-delta posture.
