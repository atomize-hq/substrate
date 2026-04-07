---
slice_id: S00
seam_id: SEAM-1
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
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced:
  - C-01
  - C-02
contracts_consumed: []
open_remediations: []
---
### S00 - Routing and tracing contract definition

- **User/system value**: execution work proceeds from one owned contract bundle instead of ad-hoc replay heuristics and ambiguous tracing-validation language.
- **Scope (in/out)**:
  - In: define `C-01` and `C-02` rules, boundaries, and verification checklists tightly enough that the producer seam can later pass its contract gate.
  - Out: final publication evidence and closeout accounting, which belong to `S99`.
- **Acceptance criteria**:
  - `C-01` names the canonical four replay-routing outcomes, their inputs, and the producer-side test matrix.
  - `C-02` names the execution-mode behavior matrix, the safe trace omission rule, and what WPEP Case B must assert.
  - Both contracts include explicit target files/tests and pass-fail conditions.
- **Dependencies**:
  - `threading.md` contract registry for `C-01`, `C-02`, and `THR-01`
  - `crates/shell/src/execution/policy_snapshot.rs`
  - `crates/replay/src/replay/executor.rs`
  - `crates/shell/src/execution/manager.rs`
  - `crates/shell/src/scripts/bash_preexec.rs`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md`
- **Verification**:
  - Contract rules below must map directly to executable tests or deterministic validation probes in later slices.
- **Rollout/safety**: no new user-facing flags or config keys; preserve canonical trace safety while removing routing drift.
- **Review surface refs**: `../../review_surfaces.md` R2 and R3

#### C-01 contract rules

1. **Authority**: replay request construction must consume the same canonical world-network routing contract already owned by `crates/shell/src/execution/policy_snapshot.rs`, not replay-local heuristics.
2. **Four-case matrix**:
   - gate off plus restrictive `net_allowed` => `policy_snapshot.net_allowed` is canonicalized, `world_network.isolate_network=false`, `world_network.allowed_domains=[]`
   - gate on plus allow-all `["*"]` => `policy_snapshot.net_allowed=["*"]`, `world_network.isolate_network=false`, `world_network.allowed_domains=[]`
   - gate on plus deny-all `[]` => `policy_snapshot.net_allowed=[]`, `world_network.isolate_network=true`, `world_network.allowed_domains=[]`
   - gate on plus restrictive allowlist => `policy_snapshot.net_allowed` is canonicalized, `world_network.isolate_network=true`, `world_network.allowed_domains` equals the canonical domains
3. **Parity requirement**: both local replay and agent-backed replay must derive `policy_snapshot.net_allowed` and `world_network` from the same contract surface, so the four outcomes above stay identical across replay backends.
4. **Verification checklist**:
   - `crates/shell/src/execution/policy_snapshot.rs` unit coverage remains the authority for canonicalization and empty-allowlist handling
   - `crates/replay/src/replay/executor.rs` request-construction tests or readbacks must prove replay does not hardcode `net_allowed: []` or `world_network: None`
   - `crates/shell/tests/world_request_net_allowed_snapshot.rs` and `crates/shell/tests/repl_world_first_routing_v1.rs` remain the named replay/shell parity references for the four cases
   - docs in `docs/REPLAY.md` must name the same four cases without adding a replay-only branch

#### C-02 contract rules

1. **Authority**: builtin-routing and preexec-derived tracing are separate sources of `builtin_command` records.
   - builtin-routing `builtin_command` records come from `crates/shell/src/execution/routing/builtin/utility.rs` + `builtin/shim_actions.rs`
   - preexec-derived `builtin_command` records come from child-shell wiring that passes `SUBSTRATE_ENABLE_PREEXEC`
   - canonical trace handling for `builtin_command` bodies is owned by `crates/shell/src/execution/routing/telemetry.rs`
2. **Safe trace posture**: canonical trace must continue omitting raw builtin or preexec command bodies; every canonical `builtin_command` record keeps `command_omitted: true`.
3. **Behavior matrix**: wrap/script/interactive behavior by platform is summarized below.

| Platform | Wrap | Script | Interactive |
| --- | --- | --- | --- |
| Linux | `builtin_command` may come from builtin routing; `SUBSTRATE_ENABLE_PREEXEC` is not propagated by the wrap path; shell completion summarizes `process_events_status` / `process_events_reason` and must preserve any landed Linux-backed `world_process_*` assertions from the active WPEP tests | `SUBSTRATE_ENABLE_PREEXEC` is honored only when `configure_child_shell_env(...)` is invoked with `enable_preexec=true` for a bash child-shell case; builtin-routing `builtin_command` remains distinct from any preexec-derived emission | `SUBSTRATE_ENABLE_PREEXEC` is removed; `builtin_command` may still come from builtin routing; shell completion summarizes/degrades process telemetry, and Linux-backed tests assert `world_process_*` only where the runtime already lands it |
| macOS | same omission invariant and builtin-routing split as Linux; do not assume Linux-only `world_process_*` assertions unless the macOS-backed tests document them | same child-shell/preexec rule as Linux, but keep process-telemetry wording conservative unless backed by the active macOS tests | same as Linux for `builtin_command` source separation and preexec removal; process telemetry should be described only as summarized/degraded unless a macOS-backed assertion exists |
| Windows | builtin-routing `builtin_command` may occur; `SUBSTRATE_ENABLE_PREEXEC` is not part of the canonical trace contract here | treat preexec as not asserted unless a Windows callsite/test explicitly proves otherwise | process telemetry is summarized as `unavailable` / `not_supported_platform`; do not assert `world_process_*` emission |

4. **Case B requirement**: the active WPEP pack's Case B must assert the chosen matrix directly, not a proxy signal that can drift from runtime behavior.
5. **Verification checklist**:
   - `crates/shell/src/execution/routing/builtin/utility.rs`, `crates/shell/src/execution/routing/builtin/shim_actions.rs`, and `crates/shell/src/execution/routing/telemetry.rs` are the named evidence surfaces for builtin-routing versus omission behavior
   - `crates/shell/src/execution/manager.rs`, `crates/shell/src/execution/routing/dispatch/exec.rs`, `crates/shell/src/execution/invocation/runtime.rs`, and `crates/shell/src/execution/pty/io/runner.rs` are the named evidence surfaces for `SUBSTRATE_ENABLE_PREEXEC` propagation or removal
   - `crates/shell/tests/world_process_exec_tracing_parity_wpep1.rs`, `crates/shell/tests/world_process_exec_tracing_parity_wpep2.rs`, `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md`, and `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/_core.sh` are the named evidence surfaces for the partially landed process-telemetry contract
   - `docs/TRACE.md` and `docs/internals/env/inventory.md` must continue to preserve the safe-by-default omission rule

#### S00.T1 - Record the concrete routing contract for `C-01`

- **Outcome**: later implementation slices can wire replay onto the canonical helper without re-deciding empty-allowlist or allow-all behavior.
- **Inputs/outputs**:
  - Inputs: `threading.md`, `crates/shell/src/execution/policy_snapshot.rs`, `docs/REPLAY.md`
  - Outputs: locked four-case rules, helper expectations, and named test locations for replay parity
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - prefer a minimal shared helper or shared request-construction path over duplicating shell logic inside replay
  - keep local and agent-backed replay on the same contract surface
- **Acceptance criteria**:
  - every routing case has one expected `isolate_network` / `allowed_domains` outcome and one concrete verification surface
- **Test notes**:
  - `crates/shell/src/execution/policy_snapshot.rs` unit coverage remains authoritative for the matrix
  - `crates/shell/tests/world_request_net_allowed_snapshot.rs` and `crates/shell/tests/repl_world_first_routing_v1.rs` are the named replay/shell parity readbacks
  - replay-focused tests should live near `crates/replay/src/replay/executor.rs`
- **Risk/rollback notes**:
  - if helper extraction broadens surface area too far, preserve the contract and reduce the extraction boundary, rather than reintroducing replay-local heuristics

Checklist:
- Implement: N/A in this slice
- Test: N/A in this slice
- Validate: cross-check all four cases against the existing shell helper tests
- Cleanup: none

#### S00.T2 - Record the concrete tracing behavior matrix for `C-02`

- **Outcome**: later slices can change runtime and validation assets in one publication unit without guessing what Case B or operator docs mean.
- **Inputs/outputs**:
  - Inputs: `threading.md`, `crates/shell/src/execution/manager.rs`, `crates/shell/src/scripts/bash_preexec.rs`, WPEP playbook/smoke assets, `docs/TRACE.md`
  - Outputs: explicit mode-by-platform matrix, WPEP Case B assertion target, and named verification surfaces
- **Thread/contract refs**: `THR-01`, `C-02`
- **Implementation notes**:
  - treat `builtin_command` omission as non-negotiable
  - keep the matrix narrow to the execution surfaces in scope for this pack
- **Acceptance criteria**:
  - every mode/platform cell answers the three contract questions (`world_process_*`, `builtin_command`, `SUBSTRATE_ENABLE_PREEXEC`)
  - Case B expectation is stated plainly enough to test and document
- **Test notes**:
  - manual playbook + smoke assets are authoritative validation surfaces
  - `crates/shell/tests/world_process_exec_tracing_parity_wpep1.rs` and `crates/shell/tests/world_process_exec_tracing_parity_wpep2.rs` are the named proof points for the partially landed Linux-backed process-telemetry behavior
  - any runtime assertions should be added where manager/dispatch behavior is already tested
- **Risk/rollback notes**:
  - if the matrix reveals a broader preexec product decision, keep the matrix explicit here and defer new public controls to a later seam

Checklist:
- Implement: N/A in this slice
- Test: N/A in this slice
- Validate: cross-check against current WPEP playbook and trace omission rules
- Cleanup: none
