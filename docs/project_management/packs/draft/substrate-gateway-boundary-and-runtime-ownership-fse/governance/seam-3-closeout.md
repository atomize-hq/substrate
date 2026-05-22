---
seam_id: SEAM-3
status: landed
closeout_version: v1
seam_exit_gate:
  source_ref: ../threaded-seams/seam-3-typed-runtime-and-platform-parity/slice-99-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - ../seam-1-closeout.md
    - ../seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
    - THR-04
  stale_triggers:
    - revalidate `SEAM-4` if typed world-service lifecycle/status endpoint ownership or request/response shape changes
    - revalidate `SEAM-4` if the shell or shared-client consumption path changes
    - revalidate `SEAM-4` if `status --json` absence semantics or `client_wiring.*` meaning changes through runtime integration
    - revalidate `SEAM-4` if the allowed divergence list for Linux/macOS/Windows transport or bootstrap behavior changes
    - revalidate `SEAM-4` if the required parity evidence set changes
    - revalidate `SEAM-4` if provisioning scope is pulled back into the runtime parity contract
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-3 Typed runtime and platform parity

This closeout records the landed SEAM-3 exit state after the typed runtime boundary and parity evidence work completed and the seam-exit gate passed.

## Seam-exit gate record

- **Source artifact**: `../threaded-seams/seam-3-typed-runtime-and-platform-parity/slice-99-seam-exit-gate.md`
- **Landed evidence**:
  - `8c0bd439` - completed S1 typed runtime boundary across shared API types, shared client helpers, world-service gateway routes, and shell gateway consumption/tests
  - `4511b3a5` - completed S2 parity evidence updates in `docs/WORLD.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
  - `crates/transport-api-types/src/lib.rs`
  - `crates/transport-api-client/src/lib.rs`
  - `crates/world-service/src/handlers.rs`
  - `crates/world-service/src/service.rs`
  - `crates/shell/src/builtins/world_gateway.rs`
  - `crates/world-service/tests/gateway_runtime_parity.rs`
  - `crates/shell/tests/world_gateway.rs`
  - `docs/WORLD.md`
  - Validation commands:
    - `cargo test -p transport-api-client -- --nocapture`
    - `cargo test -p world-service --test gateway_runtime_parity -- --nocapture`
    - `cargo test -p shell --test world_gateway -- --nocapture`
  - Validation readback result:
    - `cargo test -p transport-api-client -- --nocapture` passed `13/13` tests plus `0` doc tests.
    - `cargo test -p world-service --test gateway_runtime_parity -- --nocapture` completed successfully and the `3/3` target-local route-shape tests passed; the runtime-dependent service cases self-skipped on this host after `WorldService::new()` reported Linux/VM-only support.
    - `cargo test -p shell --test world_gateway -- --nocapture` passed `8/8` tests.
- **Contracts published or changed**:
  - `C-04`
- **Threads published / advanced**:
  - `THR-04 published`
- **Review-surface delta**: none material; the landed runtime boundary and `docs/WORLD.md` parity evidence match the planned seam scope without widening into provisioning ownership or a second operator contract.
- **Planned-vs-landed delta**: none material; S1 and S2 landed the planned runtime ownership and parity evidence surfaces, and S99 only publishes the resulting governance state.
- **Downstream stale triggers raised**:
  - typed world-service lifecycle/status endpoint ownership or request/response shape changes
  - shell or shared-client consumption path changes
  - `status --json` absence semantics or `client_wiring.*` meaning changes through runtime integration
  - allowed divergence changes for Linux/macOS/Windows transport or bootstrap behavior
  - required parity evidence changes
  - provisioning scope is pulled back into the runtime parity contract
- **Remediation disposition**: `REM-001` resolved; the contract baseline, runtime implementation, and parity evidence all landed, and the targeted verification reruns passed on the current tree with the `world-service` host-local runtime cases explicitly self-skipping outside Linux/VM support.
- **Promotion blockers**: none; the seam-exit gate is passed and no open remediations remain.
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**: none
- **Carried-forward remediations**: none
