---
slice_id: S00
seam_id: SEAM-3
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: inherited
    contract: passed
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-04
contracts_produced:
  - C-04
contracts_consumed:
  - C-01
  - C-02
  - C-03
open_remediations:
  - REM-001
---
### S00 - Runtime parity contract definition

- **User/system value**: downstream runtime and docs work inherit one explicit typed lifecycle/status contract instead of improvising around backend-specific behavior.
- **Scope (in/out)**:
  - In:
    - define the owned boundary for the typed world-agent lifecycle/status contract
    - define the allowed divergence list and the evidence expectations for Linux, macOS, and Windows
    - name the publication surfaces for the feature-local parity spec and the durable runtime/parity contract
  - Out:
    - world-agent handler implementation
    - shell/client adoption work
    - seam-exit evidence and downstream docs lock-in
- **Acceptance criteria**:
  - `C-04` names the typed lifecycle/status ownership boundary and its canonical contract refs
  - the feature-local parity spec and durable runtime/parity contract exist and align
  - later slices do not need to reopen which surface owns runtime/parity truth
  - the owner artifact includes a concrete execution checklist with named code and test loci
- **Dependencies**:
  - `C-01`
  - `C-02`
  - `C-03`
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
- **Verification**:
  - the contract-definition bundle must map directly to `platform-parity-spec.md`, `docs/contracts/substrate-gateway-runtime-parity.md`, seam-local review, and later runtime/parity implementation surfaces
- **Rollout/safety**:
  - keep provisioning out of scope and keep raw exec probing out of the operator contract
- **Review surface refs**: `review.md` R1 and R2

#### C-04 contract rules (concrete)

These are the binding contract statements this seam must implement and later publish in closeout.

1. **Authority boundary**: lifecycle and status operations behind `substrate world gateway sync`, `status`, and `restart` must consume a typed world/backend runtime surface instead of shell-side raw exec probing or backend-private heuristics.
2. **Ownership split inside the runtime path**:
   - `crates/shell/src/builtins/world_gateway.rs` owns operator rendering and exit classification only.
   - `crates/world-agent`, `crates/agent-api-types`, and `crates/agent-api-client` own the typed runtime transport boundary.
   - `substrate-gateway` internals remain out of scope for this contract except through the stable lifecycle/status boundary they present to Substrate.
3. **Inherited upstream contracts remain authoritative**:
   - `C-01` still owns the command family and exit taxonomy.
   - `C-02` still owns the `status --json` envelope and `client_wiring.*` rules.
   - `C-03` still owns fail-closed placement, secret-delivery, and non-trust policy behavior.
   - `C-04` may consume those contracts but must not widen or silently redefine them.
4. **Platform parity guarantee**: Linux, macOS, and Windows expose one operator-facing lifecycle/status meaning even though the underlying world transport differs by platform.
5. **Allowed divergence list**:
   - Linux may use direct Unix socket transport to `/run/substrate.sock`.
   - macOS may use Lima-backed forwarding to reach the guest `world-agent`.
   - Windows may use the WSL-backed forwarder path, including named-pipe or TCP bridge transport as needed.
   - Provisioning helpers, doctor commands, and transport-remediation hints may differ by platform, but those differences must not create a second operator contract.
6. **Out-of-scope guardrail**: provisioning and world warm-flow ownership remain outside this seam, and runtime parity language must not absorb them.

#### Verification plan

- **Doc surfaces to publish and keep aligned**:
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/platform-parity-spec.md`
  - `docs/contracts/substrate-gateway-runtime-parity.md`
- **Code surfaces that must land against this contract**:
  - `crates/world-agent/src/lib.rs`
  - `crates/agent-api-types/src/lib.rs`
  - `crates/agent-api-client/src/lib.rs`
  - `crates/shell/src/builtins/world_gateway.rs`
- **Tests to add or extend**:
  - extend `crates/shell/tests/world_gateway.rs` with `world_gateway_status_uses_typed_runtime_contract`
  - extend `crates/shell/tests/world_gateway.rs` with `world_gateway_status_json_preserves_client_wiring_absence_rules`
  - extend `crates/shell/tests/world_gateway.rs` with `world_gateway_sync_and_restart_follow_typed_lifecycle_contract`
  - add `crates/world-agent/tests/gateway_runtime_parity.rs` with `gateway_runtime_status_route_matches_socket_activation_transport`
  - add `crates/world-agent/tests/gateway_runtime_parity.rs` with `gateway_runtime_restart_route_preserves_component_unavailable_classification`
- **Edge cases that later slices must cover**:
  - required gateway/world component unavailable still produces the operator-owned exit `4` posture across `sync`, `status`, `status --json`, and `restart`
  - `status --json` omits `client_wiring` rather than emitting placeholders when the runtime cannot publish wiring
  - transport failure on Linux, macOS, or Windows must not trigger a raw exec probe fallback that bypasses the typed runtime surface
  - platform-specific doctor or forwarding detail must not change the operator-visible lifecycle/status meaning
- **Pass/fail conditions**:
  - pass when the typed runtime boundary is explicit in code, the shell consumes it without reconstructing state locally, and the named tests prove parity and absence semantics
  - fail if shell logic widens `status --json`, introduces raw probing, or allows platform-specific transport details to change the operator-facing contract

#### S00.T1 - Publish the owned runtime/parity baseline

- **Outcome**: `C-04` is concrete in both the feature-local parity spec and the durable contract mirror.
- **Inputs/outputs**:
  - Inputs: `../../threading.md`, `../../governance/seam-1-closeout.md`, `../../governance/seam-2-closeout.md`, `docs/WORLD.md`, `docs/INSTALLATION.md`, and the current shell/world-agent transport surfaces
  - Outputs: aligned `platform-parity-spec.md` and `docs/contracts/substrate-gateway-runtime-parity.md`
- **Thread/contract refs**: `THR-04`, `C-04`
- **Acceptance criteria**:
  - the typed runtime authority boundary is explicit
  - the allowed divergence list is explicit
  - the out-of-scope provisioning boundary is explicit

#### S00.T2 - Freeze the owner execution checklist

- **Outcome**: later slices can implement the runtime boundary without reopening contract questions.
- **Inputs/outputs**:
  - Inputs: the published `C-01`, `C-02`, and `C-03` contracts plus the current shell/world-agent code surface
  - Outputs: named code loci, test loci, edge cases, and pass/fail conditions for S1 and S2
- **Thread/contract refs**: `THR-04`, `C-04`
- **Acceptance criteria**:
  - every required code surface and test surface is named
  - the unavailable posture, `client_wiring` omission rule, and no-probing guardrail are testable without new contract decisions

Checklist:
- Implement: N/A in this slice
- Test: N/A in this slice
- Validate: read back the parity spec, durable contract mirror, and test plan against the published `C-01`/`C-02`/`C-03` surfaces
- Cleanup: none
