# PDLDPM3-spec — Keep the dev installer on the same install-state contract

## Behavior delta (single)
- Existing: `scripts/substrate/dev-install-substrate.sh` has its own install-state write path, so Linux platform metadata persistence can drift from the production installer's path, dry-run, `--no-world`, and compatibility semantics.
- New: `scripts/substrate/dev-install-substrate.sh` persists Linux install-state metadata using the same contract as `scripts/substrate/install-substrate.sh`: same file path, same `host_state.platform` schema, same dry-run no-write rule, same `--no-world` behavior, and the same compatibility-preserving merge or replacement posture.
- Why: both installer entrypoints already expose one operator-facing `install_state.json` surface, so they cannot be allowed to encode divergent meanings for that same file.

## Scope
- `scripts/substrate/dev-install-substrate.sh` only.
- Linux install-state persistence parity with the shared contract already defined for the production installer.
- Reuse of the same metadata-only implementation seam where practical, so shared-script churn stays scoped to persistence behavior.
- Explicit no-delta posture for macOS and Windows.

## Behavior (authoritative)
### Parity with the shared install-state contract
- On Linux, a successful non-dry-run `scripts/substrate/dev-install-substrate.sh` invocation MUST create or update `<resolved SUBSTRATE_HOME>/install_state.json` using the same schema and omission rules defined by `contract.md`, `install-state-schema-spec.md`, and `compatibility-spec.md`.
- The dev installer MUST write the same four feature-owned `host_state.platform.*` fields and MUST NOT introduce a dev-only key, alternate path, or alternate package-manager vocabulary.
- Passing `--no-world` to the dev installer MUST NOT suppress install-state persistence on Linux.

### Compatibility and dry-run parity
- A dev-installer dry run MUST NOT create, rewrite, truncate, or merge `<resolved SUBSTRATE_HOME>/install_state.json`.
- On a compatible existing file, the dev installer MUST preserve unknown top-level keys plus existing `host_state.group` / `host_state.linger` content outside the current run's allowed update boundary, while replacing the full `host_state.platform` subtree for the current run.
- On a corrupt or wrong-schema existing file, the dev installer MUST emit a warning and replace the file with a fresh `schema_version: 1` document containing only current-run authoritative data.

### Platform guard and failure posture
- macOS and Windows dev-installer flows MUST NOT gain new `host_state.platform` writes from this slice.
- Persistence remains fail-open for the dev installer: metadata read, merge, or write failures during an otherwise successful install MUST NOT introduce a new non-zero exit code.
- This slice MUST stay metadata-scoped and MUST NOT absorb unrelated helper-discovery, world-agent staging, or provisioning behavior changes in the dev installer.

## Acceptance criteria
- AC-PDLDPM3-01: A successful non-dry-run Linux execution of `scripts/substrate/dev-install-substrate.sh` creates or updates `<resolved SUBSTRATE_HOME>/install_state.json` with the same `host_state.platform` field set and omission semantics required by `install-state-schema-spec.md`, without introducing any dev-only JSON keys.
- AC-PDLDPM3-02: The same successful Linux dev-installer run invoked with `--no-world` still writes `<resolved SUBSTRATE_HOME>/install_state.json` under the same path and schema contract as a world-enabled dev install.
- AC-PDLDPM3-03: A `--dry-run` execution of `scripts/substrate/dev-install-substrate.sh` does not create a missing `install_state.json` and does not modify an existing file.
- AC-PDLDPM3-04: With a compatible existing `install_state.json`, the dev installer preserves unknown top-level keys and existing `host_state.group` / `host_state.linger` content that the same run did not change, while replacing `host_state.platform` for the current run; with a corrupt or wrong-schema file, it emits a warning and replaces the file with a fresh `schema_version: 1` document.
- AC-PDLDPM3-05: On macOS and Windows dev-installer flows, this slice introduces no new `host_state.platform` writes and no new metadata file, preserving the contract's explicit no-delta posture for those platforms.

## Out of scope
- Changing the dev installer's helper-discovery, world-agent staging, or provisioning semantics tracked by other draft packs.
- Adding new validation ownership in `tests/installers/install_smoke.sh`; validation planning remains outside this slice.
- Any production-installer-only write guarantee work already covered by `PDLDPM1`.
