---
slice_id: S1
seam_id: SEAM-1
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers: []
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
contracts_produced: []
contracts_consumed:
  - C-01
open_remediations: []
---
### S1 - Command family and status entrypoint

- **User/system value**: operators get one stable command family and one stable status entrypoint, so downstream seams stop inheriting archived command order or ad hoc status discovery rules.
- **Scope (in/out)**:
  - In:
    - wire the `substrate world gateway sync|status|restart` command family into the CLI/parser and builtin surfaces
    - make `status` the stable operator entrypoint for availability and wiring discovery
    - define the absent-state posture for human-readable and machine-readable status surfaces
    - add regression coverage that rejects archived alternate command ordering
  - Out:
    - field-level `status --json` schema decisions
    - fail-closed policy evaluation rules and trust-boundary tables
- **Acceptance criteria**:
  - one CLI surface owns `sync|status|restart`
  - `status` remains the operator entrypoint and `status --json` remains the machine-readable authority
  - absent-state behavior is explicit and regression-tested
  - archived `status gateway` or `sync gateway` ordering does not survive in current CLI/docs/tests
- **Dependencies**:
  - `C-01`
  - `THR-01`
  - `crates/shell/src/execution/cli.rs`
  - `crates/shell/src/builtins/mod.rs`
  - `crates/shell/src/builtins/world_gateway.rs`
  - `crates/shell/tests/world_gateway.rs`
- **Verification**:
  - targeted CLI/parser and builtin tests
  - readback diff against `contract.md` and ADR-0040 user contract
- **Rollout/safety**: keep the command family singular and do not let runtime-private probing become the operator contract.
- **Review surface refs**: `../../review_surfaces.md` R1

#### S1.T1 - Wire the singular gateway command family into the shell surfaces

- **Outcome**: the CLI exposes one gateway lifecycle/status surface and does not depend on archived command ordering or hidden aliases.
- **Inputs/outputs**:
  - Inputs: `C-01`, CLI parser surfaces, builtin registration, planned `world_gateway` builtin file
  - Outputs: one command family and one status entrypoint wired through current shell surfaces
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - prefer one builtin entry surface over multiple partially overlapping command handlers
  - keep machine-readable status authority explicit even if text output is abbreviated
- **Acceptance criteria**:
  - the shell surfaces do not publish an alternate command order
  - `sync`, `status`, and `restart` remain grouped as one operator boundary
- **Test notes**:
  - add parser/help or builtin dispatch coverage near `crates/shell/tests/world_gateway.rs`
- **Risk/rollback notes**:
  - avoid introducing compatibility aliases that keep two operator contracts alive

Checklist:
- Implement: CLI and builtin wiring
- Test: command-family regression coverage
- Validate: compare the surfaced commands to `C-01`
- Cleanup: remove stale command-order wording from touched shell surfaces

#### S1.T2 - Lock the status entrypoint and absent-state behavior

- **Outcome**: operator status discovery stays singular and downstream seams can safely inherit one absent-state posture.
- **Inputs/outputs**:
  - Inputs: `C-01`, ADR-0040 status wording, future `status --json` ownership boundary
  - Outputs: explicit status entrypoint behavior and absent-state regression coverage
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - keep human-readable status and machine-readable status synchronized at the contract boundary, not by duplicating schema detail here
- **Acceptance criteria**:
  - absent-state text and `--json` posture do not contradict each other
  - downstream seams inherit one status entrypoint for schema and policy work
- **Test notes**:
  - add readback checks for absent-state messaging and `--json` authority
- **Risk/rollback notes**:
  - do not let operator text imply field-level JSON guarantees that belong to `SEAM-2`

Checklist:
- Implement: status entrypoint wording and behavior
- Test: absent-state and status-entrypoint coverage
- Validate: compare wording to ADR-0040
- Cleanup: remove stale status-entrypoint phrasing from touched surfaces
