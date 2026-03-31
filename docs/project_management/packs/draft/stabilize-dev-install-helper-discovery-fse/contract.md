# stabilize-dev-install-helper-discovery — contract surface

This file is the single authoritative contract for the SEAM-1 staging and discovery boundary in `stabilize-dev-install-helper-discovery`.

Decision inputs:
- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/threaded-seams/seam-1-durable-helper-bundle-staging-discovery/slice-1-freeze-durable-bundle-contracts.md`
- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/threaded-seams/seam-1-durable-helper-bundle-staging-discovery/seam.md`
- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/threading.md`
- `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/seam_map.md`

Validation references:
- `scripts/substrate/dev-install-substrate.sh`
- `crates/shell/src/builtins/world_enable/runner/paths.rs`
- `crates/shell/tests/world_enable.rs`

## Authority + scope

- Canonical planning-pack path for this feature: `docs/project_management/packs/draft/stabilize-dev-install-helper-discovery-fse/`
- SEAM-1 owns the first publishable contracts for the durable helper-bundle staging and discovery surface.
- In scope:
  - one exact staged bundle surface under `$SUBSTRATE_HOME`
  - helper discovery precedence for `substrate world enable`
  - managed-asset eligibility rules for repo-managed symlinks versus manifest-tracked copied Linux guest binaries
  - fail-closed helper discovery when no candidate exists
  - the `--home` / `--prefix` CLI posture for `substrate world enable`
- Out of scope:
  - dev-uninstall implementation and protected-path refusal behavior
  - helper-missing remediation wording changes
  - runtime code changes
  - Windows behavior enablement
  - full macOS provisioning parity

## `C-02` - Fixed durable runtime bundle surface

The durable staged surface under `$SUBSTRATE_HOME` includes exactly:

- `scripts/substrate/world-enable.sh`
- `scripts/substrate/install-substrate.sh`
- `scripts/substrate/world-deps.yaml`
- `scripts/mac/lima-warm.sh`
- `scripts/mac/lima/substrate.yaml`
- `scripts/mac/lima/substrate-dev.yaml`
- best-effort Linux guest binaries under `bin/linux/`

Contract notes:
- Additive or subtractive path-list changes are outside this seam and stale `THR-01` plus `THR-02`.
- The host launcher at `$SUBSTRATE_HOME/bin/substrate` is not part of the durable bundle contract and remains unchanged.
- The bundle inventory is a publication boundary for downstream seams; later seams consume the landed surface instead of inferring it from script internals.

## `C-03` - Managed-asset eligibility

- Repo-owned script, YAML, and macOS support assets in the durable bundle stage as repo-managed symlinks.
- Linux guest binaries under `bin/linux/` are dev-managed only when they remain repo-managed symlinks into local build outputs or when copied from Lima and recorded in `.dev-install-managed/mac-linux-binaries.txt`.
- No other asset class is considered dev-managed by this seam.
- The contract does not authorize recursive cleanup or ownership inference beyond symlink target provenance plus manifest membership.

## `C-01` - Helper discovery and CLI contract

`substrate world enable` resolves helpers in this exact order:

1. `SUBSTRATE_WORLD_ENABLE_SCRIPT`
2. `$SUBSTRATE_HOME/scripts/substrate/world-enable.sh`
3. `<inferred version dir>/scripts/substrate/world-enable.sh`

Contract notes:
- If none of those candidates exists, the command exits fail-closed.
- `--home` remains valid for `substrate world enable`.
- `--prefix` remains invalid for `substrate world enable`.
- Any helper-order, flag-surface, or helper-missing guidance change stales `THR-02`.

## Downstream contract consumers

- `SEAM-2` consumes `C-02` and `C-03` for cleanup and managed-asset reasoning.
- `SEAM-3` consumes `C-01`, `C-02`, `C-03`, and `C-04` for conformance and drift-guard planning.

## `C-04` - Protected-path refusal and preserved-path reporting

- Protected-path conflicts preserve user-managed regular files and non-repo-managed symlinks.
- Cleanup reports the preserved path deterministically.
- Protected-path cleanup refusal uses exit class `5` rather than destructive cleanup.
- No other asset class is considered eligible for protected-path deletion authority by this seam.
- Any change to refusal classification, preserved-path messaging, or cleanup eligibility forces conformance revalidation.

## Verification summary

| Check | Planned location | Pass condition |
| --- | --- | --- |
| Durable bundle inventory | `scripts/substrate/dev-install-substrate.sh` | one fixed path list is staged under `$SUBSTRATE_HOME` without widening scope |
| Managed-asset rules | `scripts/substrate/dev-install-substrate.sh` and later closeout evidence | symlinked assets and manifest-tracked copied binaries are distinguishable without ambiguity |
| Helper order | `crates/shell/src/builtins/world_enable/runner/paths.rs` | prefix helper wins over inferred version-dir when env override is absent |
| Flag and fail-closed behavior | `crates/shell/tests/world_enable.rs` | `--home` remains valid, `--prefix` remains invalid, and missing helpers still fail closed |
| Protected-path refusal | `scripts/substrate/dev-uninstall-substrate.sh` and later closeout evidence | preserved-path reporting stays deterministic and exits with class `5` |

Contract-readiness for SEAM-1 is documentary: the publication artifact is complete when the contract boundaries above are explicit enough for downstream implementation and closeout without reopening ownership or lookup-order questions.
