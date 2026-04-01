---
slice_id: S2
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - ADR-0037 winning-layer precedence changes for world.enabled=false
    - tokenized display rules change for workspace or global config paths
    - allowlisted env token display rules change for SUBSTRATE_OVERRIDE_WORLD
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
contracts_produced:
  - C-01
  - C-02
contracts_consumed: []
open_remediations: []
candidate_subslices: []
---
### S2 - Deterministic precedence + redaction tests

- **User/system value**: The attribution contract is enforced by deterministic tests that prevent precedence drift and prevent redaction leaks before any replay wiring lands.
- **Scope (in/out)**:
  - In: add regression coverage for layer mapping, precedence assumptions, and redaction invariants (including fallback).
  - Out: replay copy/telemetry assertions (`SEAM-2`) and conformance lock-in (`SEAM-3`).
- **Acceptance criteria**:
  - Tests cover every layer in the `C-01` layer vocabulary plus the unsafe-proof fallback behavior.
  - Tests include negative assertions that:
    - no absolute workspace/global config path strings appear
    - no raw env values appear (only the allowlisted token `SUBSTRATE_OVERRIDE_WORLD` may appear)
  - Tests are deterministic and do not depend on the host filesystem beyond tempdir fixtures.
- **Dependencies**: `S1` contract shape is concrete.
- **Verification**:
  - Unit tests can target `crates/shell/src/execution/config_model.rs` helper mapping directly.
  - Optional integration tests may live in `crates/shell/tests/replay_world.rs` to protect the replay-adjacent wiring surface once `SEAM-2` begins.
- **Rollout/safety**:
  - Tests-only slice; safe to land early.
- **Review surface refs**:
  - `../../threading.md`
  - `../../review_surfaces.md`

#### S2.T1 - Winner-to-layer mapping coverage

- **Outcome**: each provenance layer produces the expected `C-01` structured source and triggers `source_unknown` when proof is unsafe.
- **Inputs/outputs**:
  - Input: mocked `ConfigExplainKey.sources`
  - Output: structured attribution result
- **Thread/contract refs**: `THR-01` / `C-01`, `THR-02` / `C-02`
- **Acceptance criteria**:
  - `cli_flag`, `override_env`, `workspace_patch`, `global_patch`, `default` map as specified.
  - missing/empty/multi-source/unknown-layer cases map to `source_unknown`.

#### S2.T2 - Redaction invariants coverage

- **Outcome**: negative assertions prevent raw path/env leakage.
- **Acceptance criteria**:
  - raw source paths in provenance are ignored (tokenized displays only).
  - raw env values are never emitted.
