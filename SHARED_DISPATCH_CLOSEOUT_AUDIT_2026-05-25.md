# Shared Dispatch Closeout Audit

Date: 2026-05-25

Scope audited:

- [PLAN.md](/home/azureuser/__Active_Code/atomize-hq/substrate/PLAN.md)
- [ORCH_PLAN.md](/home/azureuser/__Active_Code/atomize-hq/substrate/ORCH_PLAN.md)
- [llm-last-mile/29.5-shared-dispatch-contract-closeout-and-parity-hardening.md](/home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/29.5-shared-dispatch-contract-closeout-and-parity-hardening.md)
- [llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md](/home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md)

## Verdict

The 29.5 closeout should be treated as partially landed, not fully complete.

What appears landed correctly:

- inventory-backed dispatch now merges `policy_overlay` into resolved `effective_policy`
- bounded capability narrowing is implemented for the approved family
- public persisted-attach control flows are wired through the shared resolver
- successor attach truth is copied forward with continuity cleared
- docs for 29, 30, and 31 were updated to describe the narrowed 29.5 floor

What blocks calling the slice complete:

- REPL host-orchestrator cold start still bypasses the shared dispatch contract and persists manifest-default attach truth instead of resolved-contract truth
- persisted attach resolution still does not fully treat persisted attach knobs as authoritative baseline truth
- fallback behavior can still recreate permissive/default attach truth in paths that the plan said should be durable and authoritative

## Primary Findings

### Finding 1: REPL host cold start still bypasses the shared contract

Severity: High

This is the main reason the slice should not yet be considered complete.

The public/start side persists durable attach truth from `ResolvedLaunchContract`:

- [crates/shell/src/execution/agents_cmd.rs:1159](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:1159)

That path rewrites `session.host_attach_contract` from:

- `HostAttachContract::from_resolved_contract(...)`

The REPL host-orchestrator cold-start path does not do that. It still:

1. resolves only to a `RuntimeSelectionDescriptor`
2. constructs the manifest directly
3. constructs `OrchestrationSessionRecord::new(...)`
4. relies on `HostAttachContract::from_manifest(...)`

Key references:

- REPL bootstrap object only carries `RuntimeSelectionDescriptor`:
  - [crates/shell/src/repl/async_repl.rs:1746](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:1746)
- REPL host bootstrap resolves via inventory selection + runtime realizability, not shared contract resolution:
  - [crates/shell/src/repl/async_repl.rs:2266](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2266)
  - [crates/shell/src/repl/async_repl.rs:2291](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2291)
- REPL host startup persists a new orchestration session directly from the manifest:
  - [crates/shell/src/repl/async_repl.rs:2204](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2204)
- `OrchestrationSessionRecord::new(...)` still seeds durable attach truth from `from_manifest(...)`:
  - [crates/shell/src/execution/agent_runtime/orchestration_session.rs:349](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:349)
- `HostAttachContract::from_manifest(...)` still hardcodes default attach capabilities and no persisted policy snapshot:
  - [crates/shell/src/execution/agent_runtime/orchestration_session.rs:152](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:152)
  - [crates/shell/src/execution/agent_runtime/orchestration_session.rs:177](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:177)
  - [crates/shell/src/execution/agent_runtime/orchestration_session.rs:182](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:182)

Why this matters:

- the 29.5 plan and SOW require one truthful contract floor across human and orchestrator-controlled launches
- this path still gives REPL/orchestrator cold starts a different durable attach truth shape than the public/start path
- the most visible drift is in attach-relevant capabilities and persisted `effective_policy`

Practical consequence:

- equivalent human and REPL cold starts are not yet guaranteed to persist equivalent host attach truth
- that invalidates the parity acceptance claim in 29.5

### Finding 2: persisted attach resolution only partially trusts persisted attach knobs

Severity: Medium

The resolver now reuses persisted capabilities and persisted policy snapshot, which is good. But attach-knob authority is still partial.

Key reference:

- [crates/shell/src/execution/agent_runtime/dispatch_contract.rs:314](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:314)

What it currently does:

- trusts persisted `requested_execution_scope`
- trusts persisted attach-relevant capabilities
- trusts persisted policy snapshot if present
- takes `host_execution_client_start` from the caller envelope
- takes `attach_mode_preference` from the caller envelope

Exact lines:

- persisted policy snapshot load:
  - [crates/shell/src/execution/agent_runtime/dispatch_contract.rs:390](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:390)
- reconstructed attach knobs:
  - [crates/shell/src/execution/agent_runtime/dispatch_contract.rs:452](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:452)

Why this matters:

- the plan text says persisted attach resolution should trust persisted `attach_launch_knobs` baseline and then apply attach-mode request checks
- the landed code instead treats two of those knobs as caller-supplied runtime inputs rather than durable baseline truth

This is not necessarily a wrong runtime design, but it does not match the contract language that says persisted attach knobs are authoritative baseline truth.

### Finding 3: missing attach state can still regain permissive/default truth

Severity: Medium

The closeout docs and plan describe birth-time durable attach truth as non-negotiable. There are still fallback paths that recreate weaker/default truth.

Key references:

- if a session has no host attach contract, `sync_host_attach_contract(...)` recreates one from manifest defaults:
  - [crates/shell/src/execution/agent_runtime/orchestration_session.rs:388](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:388)
- if persisted policy is absent, persisted attach resolution falls back to `Policy::default()`:
  - [crates/shell/src/execution/agent_runtime/dispatch_contract.rs:390](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:390)
  - [crates/shell/src/execution/agent_runtime/dispatch_contract.rs:403](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:403)

Why this matters:

- 29.5 explicitly called out the need to stop regaining permissive defaults from ambient/runtime state
- these fallback branches mean the durable truth is still not fully fail-closed in every path

## Items That Look Correct

### Inventory `policy_overlay` merge

This appears to be landed correctly.

Key references:

- inventory resolution applies overlay into `effective_policy`:
  - [crates/shell/src/execution/agent_runtime/dispatch_contract.rs:475](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:475)
  - [crates/shell/src/execution/agent_runtime/dispatch_contract.rs:653](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:653)
- overlay validation remains restriction-only:
  - [crates/shell/src/execution/agent_inventory.rs:616](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:616)

### Bounded capability narrowing

This also appears landed correctly for the explicitly approved family.

Key references:

- unsupported families fail closed:
  - [crates/shell/src/execution/agent_runtime/dispatch_contract.rs:663](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:663)
- supported family only narrows from `true` to `false`:
  - [crates/shell/src/execution/agent_runtime/dispatch_contract.rs:760](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:760)

### Retained member follow-up parity subset

This looks substantially aligned with the 29.5 intent.

Key references:

- retained parity helper:
  - [crates/shell/src/repl/async_repl.rs:4186](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4186)
- typed follow-up dispatch uses parity subset instead of live descriptor drift:
  - [crates/shell/src/repl/async_repl.rs:4677](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4677)

## Test and Validation Notes

Commands run during audit:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 c3_world_restart_invalidates_stale_member_generation_before_publish -- --nocapture
```

Observed results:

- `agent_public_control_surface_v1`: passed
- `agent_successor_contract_ahcsitc0`: passed
- `repl_world_first_routing_v1`: initially showed one failure when large suites were run in parallel, but the failing case passed when rerun alone

Important environment note:

- later extra filtered `cargo test -p shell ...` attempts hit local environment failures including `No space left on device` and linker/tempdir failures inside `target/debug`
- those environment failures should not be interpreted as product regressions, but they did limit additional verification passes

## Recommended Remediation Focus

If a fresh session is planning the fix, the highest-value remediation target is:

1. route REPL host-orchestrator cold start through the same shared dispatch contract used by the public/start path
2. carry a `ResolvedLaunchContract` or equivalent resolved-contract truth into REPL host session birth
3. persist `HostAttachContract` from `HostAttachContract::from_resolved_contract(...)` in that path
4. remove or quarantine fallback `from_manifest(...)` reconstruction for steady-state birth paths
5. then decide whether persisted attach knobs are truly durable baseline truth or caller-supplied attach-mode inputs, and align code plus docs one way or the other

## Suggested Questions For The Next Session

1. Should REPL host cold start produce a `ResolvedLaunchContract` directly, or should it wrap existing selection logic with the shared resolver before runtime materialization?
2. Is `attach_mode_preference` supposed to be persisted baseline truth, caller intent at attach time, or a split between baseline plus caller narrowing?
3. Should `effective_policy` be mandatory in persisted `HostAttachContract`, with missing policy treated as corruption instead of defaulting?
4. Can `HostAttachContract::from_manifest(...)` be restricted to compatibility/recovery-only usage so steady-state launch paths cannot silently bypass resolved-contract truth?

## Bottom Line

The slice is close, but it is not yet honest enough to call fully complete.

The largest remaining issue is not in the public control path. It is in the REPL/orchestrator cold-start path, which still persists durable attach truth from manifest-era defaults instead of from the shared resolved contract that 29.5 says should now be authoritative everywhere.
