# Remediation Log - substrate-gateway-boundary-and-runtime-ownership

This pack is extracted at seam-brief depth. No seam-local pre-exec review, landing review, or closeout review has run yet, so no remediations are opened at extraction time.

Future remediation entries must use the canonical fields from the extractor governance model:

- `remediation_id`
- `origin_phase`
- `source_gate`
- `related_seam`
- `related_slice`
- `related_thread`
- `related_contract`
- `related_artifact`
- `severity`
- `status`
- `owner_seam`
- `blocked_targets`
- `summary`
- `required_fix`
- `resolution_evidence`

Additional remediation rules for this pack:

- remediation ownership must use real seam IDs only
- blocking remediations must use structured `blocked_targets`
- future entries must distinguish pre-exec versus post-exec origin phase
- cross-seam issues must still pick one owning seam

## Open remediations

- None.

## Resolved remediations

```yaml
- remediation_id: REM-001
  origin_phase: pre_exec
  source_gate: contract
  related_seam: SEAM-3
  related_slice: S00
  related_thread: THR-04
  related_contract: C-04
  related_artifact: docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md
  severity: material
  status: resolved
  owner_seam: SEAM-3
  blocked_targets: []
  summary: typed runtime and platform parity now has a concrete owned contract baseline, and the SEAM-3 runtime and publication surfaces have landed against it
  required_fix: land the SEAM-3 owner execution surfaces attached to `threaded-seams/seam-3-typed-runtime-and-platform-parity/slice-00-runtime-parity-contract-definition.md`, `slice-1-typed-lifecycle-status-api-boundary.md`, and `slice-2-shell-consumption-and-platform-parity-evidence.md` so `C-04` and `THR-04` publish from runtime evidence without widening the contract
  resolution_evidence:
    - `8c0bd439` landed the S1 typed runtime boundary across `crates/agent-api-types/src/lib.rs`, `crates/agent-api-client/src/lib.rs`, `crates/world-agent/src/handlers.rs`, `crates/world-agent/src/service.rs`, `crates/shell/src/builtins/world_gateway.rs`, and the corresponding runtime parity and shell gateway tests
    - `4511b3a5` landed the S2 parity evidence update in `docs/WORLD.md` without widening the operator contract
    - `docs/contracts/substrate-gateway-runtime-parity.md` remains the durable canonical `C-04` contract without planning IDs
    - `cargo test -p agent-api-client -- --nocapture` passed `13/13` tests plus `0` doc tests on the current tree
    - `cargo test -p world-agent --test gateway_runtime_parity -- --nocapture` completed successfully on the current tree; the `3/3` target-local route-shape tests passed and the runtime-dependent service cases self-skipped on this host after `WorldAgentService::new()` reported Linux/VM-only support
    - `cargo test -p shell --test world_gateway -- --nocapture` passed `8/8` tests on the current tree
```
