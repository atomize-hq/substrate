---
seam_id: SEAM-1
seam_slug: doctor-text-disable-attribution
status: decomposed
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-1-doctor-text-disable-attribution.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - effective-config precedence for world.enabled changes
    - exact doctor disable-attribution message bodies change
    - disabled-status UX work changes doctor framing or status semantics
    - platform doctor renderers bypass tokenized path or env redaction
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S4
  status: pending
open_remediations: []
---
# SEAM-1 - Doctor text disable attribution (threaded decomposition)

## Seam Brief (Restated)

- **Goal / value**: make `substrate host doctor` and `substrate world doctor` state the real disable source immediately so operators stop debugging the wrong layer.
- **Type**: capability (producer seam for `C-01` and `C-02`)
- **Scope**
  - In:
    - doctor text output for host and world doctor
    - shared provenance-backed disable-attribution mapping from effective `world.enabled` winner to exact message body (`C-01`)
    - precedence truth, fallback posture, and redaction invariants (`C-02`)
    - Linux/macOS parity for host doctor and Linux/macOS/Windows parity for world doctor
  - Out:
    - additive JSON fields (`SEAM-2`)
    - health text or health JSON (`SEAM-2`)
    - replay-warning reuse, provisioning guidance, or exit-code changes
- **Touch surface (expected)**:
  - `crates/shell/src/execution/config_model.rs` (`resolve_effective_config_with_explain`)
  - `crates/shell/src/execution/platform/mod.rs` (doctor entrypoints)
  - `crates/shell/src/execution/platform/linux.rs`
  - `crates/shell/src/execution/platform/macos.rs`
  - `crates/shell/src/execution/platform/windows.rs`
  - new shared helper (location determined in `S2`)
  - new/updated tests (locations determined in `S2`/`S3`)
- **Verification**:
  - Produces contracts `C-01` (exact doctor message bodies) and `C-02` (winner mapping + fallback + redaction invariants).
  - Evidence lives in: winner-to-message mapping tests, enabled-case omission tests, manual CLI/env/workspace/global/default scenarios, and parity checks on platforms where the doctor surface exists.
- **Basis posture**:
  - Currentness: `current` (the effective-config precedence + explain sources already exist in `config_model.rs`; this seam binds to that truth rather than redefining it)
  - Upstream closeouts assumed: none (first critical-path seam)
  - Required threads: `THR-01`, `THR-02`
  - Stale triggers: listed in frontmatter
- **Threading constraints**
  - Upstream blockers: effective-config explain provenance must be available at doctor entrypoints
  - Downstream blocked seams: `SEAM-2` must not promote until `THR-01` / `THR-02` truth is publishable from closeout-backed evidence
  - Contracts produced: `C-01`, `C-02`
  - Contracts consumed: none

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S4` (`slice-4-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: `SEAM-2` must bind to landed attribution truth (not inferred intent) before it can safely expand the surface area (health + JSON).
- **Expected contracts to publish**: `C-01`, `C-02`
- **Expected threads to publish / advance**:
  - `THR-01`: toward `published` (closeout-backed `C-01`/`C-02`)
  - `THR-02`: toward `published` (message-body parity protection)
- **Likely downstream stale triggers**:
  - any change to the `world.enabled` winner precedence
  - any change to exact message bodies or their mapping inputs
  - any change to tokenized display-path or safe env token rules
- **Expected closeout evidence**:
  - unit/integration tests covering all disable sources + enabled omission
  - cross-platform parity evidence for doctor output surfaces
  - confirmation that `source unknown` is used whenever provenance is unsafe or missing

## Slice index

- `S1` -> `slice-1-contract-definition-disable-attribution.md`
- `S2` -> `slice-2-shared-helper-and-winner-mapping-tests.md`
- `S3` -> `slice-3-wire-doctor-output-and-parity-evidence.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`

