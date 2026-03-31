---
seam_id: SEAM-1
seam_slug: standard-version-dir-preflight-deterministic-remediation
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-1-standard-version-dir-preflight-deterministic-remediation.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - accepted staged path set or sufficiency rule changes
    - standard version-dir derivation changes
    - helper-output suppression or visible remediation path changes
    - world.enabled ordering or --home precedence changes
    - overlapping helper-discovery or provisioning work lands first on shared world-enable surfaces
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
# SEAM-1 - Standard version-dir preflight + deterministic remediation

## Seam Brief (Restated)

- **Goal / value**: Make `substrate world enable` behave deterministically in the standard version-dir flow by either finding an accepted staged `world-agent` artifact or failing early with one operator-visible remediation before helper launch, privileged work, or state mutation.
- **Type**: capability (producer seam)
- **Scope**
  - In:
    - Standard version-dir preflight when `SUBSTRATE_WORLD_ENABLE_SCRIPT` is unset
    - Accepted staged path derivation + fixed search order + “either path is sufficient” rule
    - Deterministic missing-artifact classification (exit `3`) with one remediation block
    - `--dry-run` parity with non-dry-run preflight; no writes when missing or dry-run
    - `world.enabled` ordering and `--home` precedence on the success path
    - Explicit override carve-out preservation (`SUBSTRATE_WORLD_ENABLE_SCRIPT`)
  - Out:
    - Linux dev-install staging changes (owned by `SEAM-2`)
    - Building `world-agent` inside `substrate world enable`
    - Broad helper-discovery durability work (e.g., `cargo clean` resilience)
- **Touch surface**:
  - `crates/shell/src/builtins/world_enable/runner.rs`
  - `crates/shell/src/builtins/world_enable/runner/paths.rs`
  - `crates/shell/tests/world_enable.rs`
  - `scripts/substrate/world-enable.sh` (preserved helper boundary)
  - `$SUBSTRATE_HOME/config.yaml` + manager env exports ordering on success
- **Verification**:
  - Produces contracts `C-01`, `C-02`, `C-03` that must be concrete enough for downstream seams to revalidate against `governance/seam-1-closeout.md` after landing.
  - Evidence lives in: unit/integration tests for runner behavior, plus closeout-backed runtime evidence for stderr remediation + early exit ordering.
- **Basis posture**:
  - Currentness: `current` (preflight derivation binds to existing `resolve_version_dir()` behavior in `runner/paths.rs`)
  - Upstream closeouts assumed: none (first producer seam in pack critical path)
  - Required threads: `THR-01`, `THR-02`
  - Stale triggers: listed in frontmatter
- **Threading constraints**
  - Upstream blockers: none
  - Downstream blocked seams: `SEAM-2` and `SEAM-3` cannot promote without `THR-01` / `THR-02` publication evidence
  - Contracts produced: `C-01`, `C-02`, `C-03`
  - Contracts consumed: none

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S4` (`slice-4-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: This seam publishes the first runtime-facing contract boundary for the enable-later workflow; downstream seams must be able to bind to landed truth (not inferred intent) before promotion.
- **Expected contracts to publish**: `C-01`, `C-02`, `C-03`
- **Expected threads to publish / advance**: `THR-01`, `THR-02`
- **Likely downstream stale triggers**:
  - Any change to standard version-dir derivation or accepted staged path set
  - Any change to missing-artifact remediation content, visibility, or exit-code taxonomy
  - Any new step inserted before preflight (helper launch, provisioning, config writes)
- **Expected closeout evidence**:
  - Captured landed rule statements (paths + ordering) in `governance/seam-1-closeout.md`
  - Test names + stdout/stderr snippets sufficient for downstream revalidation

## Slice index

- `S1` -> `slice-1-contract-definition-runtime-preflight.md`
- `S2` -> `slice-2-preflight-and-dry-run-parity.md`
- `S3` -> `slice-3-remediation-and-state-ordering.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
