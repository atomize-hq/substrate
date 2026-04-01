---
seam_id: SEAM-2
seam_slug: json-health-disable-attribution
status: decomposed
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-2-json-health-disable-attribution.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - SEAM-1 manual macOS or Windows parity proof changes exact message bodies, precedence truth, fallback posture, or tokenized source rendering
    - doctor JSON envelopes stop exposing a stable top-level placement adjacent to world_enabled and ok
    - shim doctor or health refactors stop carrying disable-source provenance through the disabled path
    - queued JSON envelope or provisioning work changes the payload root shape or omission rules
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
open_remediations: []
---
# SEAM-2 - JSON + health disable attribution (threaded decomposition)

## Seam Brief (Restated)

- **Goal / value**: expose the already-landed disable-attribution truth from `SEAM-1` as stable top-level JSON fields and matching health output so operators and automation do not need to scrape text or guess why world isolation is off.
- **Type**: integration (consumer of `C-01` and `C-02`, producer of `C-03`)
- **Scope**
  - In:
    - additive top-level `world_disable_reason` and `world_disable_source` on host doctor JSON, world doctor JSON, and `substrate health --json`
    - health text parity with the exact `C-01` message bodies from `SEAM-1`
    - shim-doctor and health plumbing needed so `--no-world` keeps `cli_flag` attribution all the way through the disabled path
    - Linux, macOS, and Windows parity for field names, enum values, and redaction posture where the payload surface exists
  - Out:
    - any change to `world.enabled` resolution, precedence, or enablement behavior
    - renaming or removing existing JSON fields
    - replay warnings, provisioning guidance, or unrelated world-status messaging
    - post-exec publication work for downstream external consumers
- **Touch surface (expected)**:
  - `crates/shell/src/execution/config_model.rs`
  - `crates/shell/src/execution/platform/mod.rs`
  - `crates/shell/src/execution/platform/linux.rs`
  - `crates/shell/src/execution/platform/macos.rs`
  - `crates/shell/src/execution/platform/windows.rs`
  - `crates/shell/src/builtins/shim_doctor/report.rs`
  - `crates/shell/src/builtins/health.rs`
  - `crates/shell/tests/doctor_scopes_ds0.rs`
  - `crates/shell/tests/shim_health.rs`
  - `crates/shell/tests/shim_doctor.rs`
- **Verification**:
  - This seam **consumes** `C-01` and `C-02` exactly as published in `../../governance/seam-1-closeout.md`; it does not redefine message bodies, winner precedence, fallback posture, or tokenization rules.
  - This seam **produces** `C-03`; readiness means the field placement, object shape, omit rules, enum vocabulary, and health-parity behavior are concrete enough to implement without guessing.
  - Verification later depends on doctor JSON envelope tests, health JSON tests, disabled-mode human-output parity checks, and focused `--no-world` preservation checks through shim-doctor and health plumbing.
- **Basis posture**:
  - Currentness: `current` because `../../threading.md` and `../../governance/seam-1-closeout.md` already publish the upstream handoff this seam consumes.
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
  - Required threads:
    - `THR-01`
    - `THR-02`
  - Stale triggers:
    - listed in frontmatter
- **Threading constraints**
  - Upstream blockers:
    - none for seam-local planning; the pack already records the resequencing that allows `SEAM-2` to advance while `REM-001` remains scoped to `SEAM-1` promotion readiness
    - any later `SEAM-1` proof drift that changes `C-01` or `C-02` forces revalidation before `SEAM-2` lands
  - Downstream blocked seams:
    - no additional internal seams are extracted in this pack; future JSON-envelope and provisioning consumers must wait for `C-03` closeout-backed publication
  - Contracts produced:
    - `C-03`
  - Contracts consumed:
    - `C-01`
    - `C-02`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S4` (`slice-4-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: downstream consumers need closeout-backed proof that top-level JSON placement, health text parity, and disabled-path attribution are stable across the supported payload surfaces.
- **Expected contracts to publish**:
  - `C-03`
- **Expected threads to publish / advance**:
  - `THR-01` toward downstream publication with the structured schema layered on top of the already-published message truth
  - `THR-02` toward publication once health text and nested disabled-path parity are proven
- **Likely downstream stale triggers**:
  - top-level JSON placement changes
  - enum vocabulary or redaction posture changes
  - any future health/shim refactor that changes disabled-path attribution or omits the exact `C-01` text
- **Expected closeout evidence**:
  - doctor JSON tests proving emit/omit rules and top-level field placement
  - health JSON tests proving the same structured contract
  - human-mode health output evidence proving exact text parity
  - explicit accounting for any unsupported or fixture-backed platform differences that remain non-blocking

## Slice index

- `S1` -> `slice-1-contract-definition-json-health-disable-attribution.md`
- `S2` -> `slice-2-doctor-json-top-level-schema-and-tests.md`
- `S3` -> `slice-3-health-parity-and-disabled-path-plumbing.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
