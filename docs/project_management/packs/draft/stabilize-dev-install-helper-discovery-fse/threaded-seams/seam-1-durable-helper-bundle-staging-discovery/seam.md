---
seam_id: SEAM-1
seam_slug: durable-helper-bundle-staging-discovery
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-1-durable-helper-bundle-staging-discovery.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads: []
  stale_triggers:
    - helper candidate order changes
    - fixed bundle path list changes
    - macOS release-root scope expands beyond helper discovery and dry-run proof
    - ADR-0035 changes shared install-script or helper-script surfaces
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S4
  status: pending
open_remediations:
  - REM-001
---
# SEAM-1 - Durable helper-bundle staging + discovery

## Seam Brief (Restated)

- **Goal / value**:
  - Make the dev install publish one durable helper/runtime bundle under `$SUBSTRATE_HOME` so `substrate world enable` keeps working after `cargo clean`, without changing the host launcher contract or loosening fail-closed behavior.
- **Type**: capability
- **Scope**
  - In:
    - Stage the fixed bundle surface under `$SUBSTRATE_HOME`, including the helper, install scripts, world-deps YAML, macOS Lima support files, and best-effort Linux guest binaries.
    - Preserve the exact helper lookup order `SUBSTRATE_WORLD_ENABLE_SCRIPT` -> prefix helper -> inferred version-dir helper.
    - Keep `substrate world enable --home` valid and `--prefix` invalid.
    - Make the managed-asset boundary concrete enough that downstream cleanup and conformance seams can consume it: repo-managed symlink assets for scripts/YAML/macOS support files and `bin/linux/*` when those paths still point at repo build outputs, plus manifest-tracked copied Linux guest binaries when cached from Lima.
  - Out:
    - Production layout changes under `$SUBSTRATE_HOME/versions/...`
    - New config keys, config precedence, or environment precedence
    - Dev-uninstall implementation and protected-path refusal behavior
    - Windows behavior enablement
    - Full macOS provisioning parity or ADR-0035 world-agent staging
- **Touch surface**:
  - `scripts/substrate/dev-install-substrate.sh`
  - `crates/shell/src/builtins/world_enable/runner/paths.rs`
  - `crates/shell/tests/world_enable.rs`
  - The staged bundle tree under `$SUBSTRATE_HOME/scripts/...` and `bin/linux/...`
- **Verification**:
  - Pre-exec verification must make the owned contracts concrete enough that implementation can stage one exact durable bundle, resolve helpers in one exact order, and keep cleanup ownership semantics narrow without waiting for post-exec publication.
  - The seam-local basis for execution is the current seam brief plus the current installer and helper-discovery surfaces; accepted or published contract artifacts remain seam-exit and closeout evidence.
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed: none
  - Required threads: none inbound; this seam is the producer for `THR-01` and `THR-02`
  - Stale triggers:
    - helper candidate order changes
    - fixed bundle path list changes
    - macOS release-root scope expands beyond helper discovery and dry-run proof
    - ADR-0035 changes shared install-script or helper-script surfaces
- **Threading constraints**
  - Upstream blockers:
    - no seam-local upstream closeout blocker exists
    - no current blocker remains after the 2026-03-30 ADR-0035 overlap revalidation; future shared-script changes still stale this basis
  - Downstream blocked seams:
    - `SEAM-2`
    - `SEAM-3`
  - Contracts produced:
    - `C-01`
    - `C-02`
    - `C-03`
  - Contracts consumed:
    - none inside this pack's contract registry; shared exit-code taxonomy and ADR guidance are basis references only

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S4` (`slice-4-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - Both downstream seams depend on closeout-backed truth for the fixed bundle surface, helper lookup order, and managed-asset classification. The seam-exit gate turns those from provisional planning assumptions into promotion-safe recorded facts.
- **Expected contracts to publish**:
  - `C-01`
  - `C-02`
  - `C-03`
- **Expected threads to publish / advance**:
  - `THR-01`: `defined` -> `published`
  - `THR-02`: `defined` -> `published`
- **Likely downstream stale triggers**:
  - any change to the fixed staged path list
  - any change to helper candidate order, helper-missing wording, or `world enable` flag surface
  - any change to which assets are repo-managed symlinks versus manifest-tracked copied binaries
  - any macOS scope expansion that turns helper discovery proof into a broader provisioning claim
- **Expected closeout evidence**:
  - landed staged bundle inventory for the fixed durable paths
  - landed managed-asset evidence for repo-managed symlinks and any manifest-tracked copied Linux guest binaries
  - landed helper-resolution proof showing prefix-helper precedence and `cargo clean` survival
  - explicit downstream stale-trigger record for `SEAM-2` and `SEAM-3`

## Slice index

- `S1` -> `slice-1-freeze-durable-bundle-contracts.md`
- `S2` -> `slice-2-dev-install-durable-bundle-staging.md`
- `S3` -> `slice-3-helper-discovery-and-fail-closed-validation.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
