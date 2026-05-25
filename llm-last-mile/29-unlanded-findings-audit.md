# Slice 29 Unlanded Findings Audit

Status: final audit of the work landed after [28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md](28.5-explicit-control-only-session-recovery-and-host-rooted-world-start-alignment.md) and claimed under [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](29-shared-agent-dispatch-envelope-and-capability-override-contract.md).

## Scope

This document records what was reviewed, what appears correctly landed, what remains unlanded, and which gaps still belong to slice 29 versus later slices 30 and 31.

The review baseline was:

1. parent floor at commit `2bf50131` (`feat: finalize control-only session recovery`)
2. current branch head after the slice-29 landing commits
3. the current truth documents:
   - [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](./29-shared-agent-dispatch-envelope-and-capability-override-contract.md)
   - [30-public-world-scoped-agent-start-and-capability-flags.md](./30-public-world-scoped-agent-start-and-capability-flags.md)
   - [31-lazy-host-attach-for-host-rooted-world-start.md](./31-lazy-host-attach-for-host-rooted-world-start.md)

Primary runtime/code anchors reviewed:

- `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
- `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
- `crates/shell/src/execution/agent_runtime/state_store.rs`
- `crates/shell/src/execution/agent_runtime/validator.rs`
- `crates/shell/src/execution/agents_cmd.rs`
- `crates/shell/src/repl/async_repl.rs`

## High-Level Conclusion

Slice 29 is not invalidated, but it is not fully landed either.

The merged code appears to have landed the shared-dispatch-contract foundation successfully:

1. a new shared resolver exists in `dispatch_contract.rs`
2. human `start`, `reattach`, detached host `turn`, and `fork` now route through that resolver
3. the minimum 28.5 attach seam is still preserved
4. fail-closed error messaging and several targeted public-surface regressions are covered by tests

However, several parts of the slice-29 contract are still incomplete relative to the SOW:

1. the generalized persisted host attach contract is not yet the sole source of all persisted attach truth
2. inventory restriction-only policy overlays are not actually merged into the resolved contract
3. dispatch-time capability override families are named but not yet supported
4. caller parity is strong for human host flows and member-start selection, but not obviously complete for every orchestrator-controlled dispatch path

## What Appears Correctly Landed

### 1. Shared dispatch-resolution foundation exists

The repo now has a dedicated internal contract module:

- `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`

That module introduces:

1. `DispatchRequestEnvelope`
2. `DispatchCallerKind`
3. `DispatchBaselineKind`
4. `DispatchCapabilityOverrideSet`
5. `AttachLaunchKnobs`
6. `ResolvedLaunchContract`
7. explicit rejecting-layer / field-scoped resolution errors

This is the clearest evidence that 29 landed real shared-resolution infrastructure instead of only documentation.

### 2. Human host caller surfaces are wired onto the shared resolver

The following public host surfaces now consume the shared contract:

1. public `start`
   - `build_start_launch_plan(...)` in `crates/shell/src/execution/agents_cmd.rs`
2. public `reattach`
   - `build_attach_launch_plan(...)`
3. detached host `turn`
   - `build_resumed_turn_launch_plan(...)`
4. public `fork`
   - `allocate_fork_successor(...)`

These paths resolve from either:

1. inventory-backed baseline truth for new launch, or
2. persisted-attach-backed baseline truth for host continuity and successor allocation

That matches the core 29 direction.

### 3. The 28.5 persisted attach seam is preserved and extended

The attach seam remains durable and fail-closed through:

1. `HostAttachContract`
2. `OrchestrationSessionRecord.host_attach_contract`
3. `sync_host_attach_contract(...)`
4. `fork_successor_attach_contract(...)`

The public control/state-store logic still correctly refuses to proceed when required attach truth is missing or disallowed.

### 4. High-signal regression tests passed

The following targeted checks were run successfully during audit:

1. `cargo test -p shell dispatch_contract -- --nocapture`
2. `cargo test -p shell public_reattach_uses_persisted_attach_continuity_selector_for_resume_args -- --nocapture`
3. `cargo test -p shell public_turn_uses_persisted_attach_continuity_selector_when_recovering_detached_host_turns -- --nocapture`
4. `cargo test -p shell public_root_start_denials_name_field_layer_and_reason -- --nocapture`

These passing checks increase confidence that the slice-29 foundation is real and not merely aspirational.

## What Has Not Yet Been Landed

### 1. The generalized persisted host attach contract does not yet fully round-trip its own truth

This is the most important remaining gap.

Slice 29 says the persisted host attach contract should carry:

1. the resolved host-orchestrator launch contract
2. attach-relevant capability selections and restrictions
3. continuity selector state
4. enough exact launch truth to support both continuity attach and fresh attach

Current code only partially achieves that.

Evidence:

1. `HostAttachContract::from_manifest(...)` still writes defaulted capabilities/knobs rather than deriving them from a resolved shared contract
   - `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
2. `sync_host_attach_contract(...)` only refreshes `continuity_uaa_session_id`
   - it does not refresh capabilities, launch knobs, or other generalized attach truth
3. `resolve_persisted_host_attach_contract(...)` reconstructs capabilities with a hardcoded `AgentCapabilitiesV1` literal
   - it does not consume the persisted `HostAttachContract.capabilities` values
4. `build_attach_launch_plan(...)` and `allocate_fork_successor(...)` construct new `AttachLaunchKnobs` from caller intent and pass them in through the envelope
   - they do not replay attach knobs from the persisted contract as the primary source of truth

Practical meaning:

1. continuity state is persisted and reused correctly
2. baseline launch descriptor identity is persisted and reused correctly
3. but the generalized attach contract is not yet authoritative for all of the extra contract surface 29 claims to have landed

This is slice-29-owned work, not 30/31 work.

### 2. Inventory policy overlays are still validated but not merged into the resolved contract

Slice 29 says inventory owns restriction-only embedded policy overlays and that merge precedence must include inventory defaults, dispatch overrides, and policy restrictions.

Current code carries overlay data into `ProjectedInventoryEntryV1` but does not appear to apply it during contract resolution.

Evidence:

1. `ProjectedInventoryEntryV1` includes `policy_overlay`
   - `crates/shell/src/execution/agent_inventory.rs`
2. `resolve_inventory_projected_contract(...)` validates baseline candidate shape and checks `agents_allowed_backends`
   - but it never merges `projected.policy_overlay`
3. `ResolvedLaunchContract.effective_policy` is currently just `base_policy.clone()` on inventory-backed resolution
   - not a narrowed merged policy
4. `DispatchResolutionErrorKind::InvalidPolicyOverlay` exists but is not obviously exercised by the resolver path

Practical meaning:

1. the contract surface now has a place for merged policy truth
2. the merge itself is still incomplete

This is slice-29-owned work.

### 3. Dispatch-time capability override families are named but still entirely rejected

Slice 29 says the minimum supported override families should cover:

1. execution scope
2. explicit capability narrowing or selection
3. attach-relevant launch knobs needed later by 30 and 31
4. narrower policy overlays where permitted

The implementation currently names these categories, but does not yet support capability overrides on current caller surfaces.

Evidence:

1. `DispatchCapabilityOverrideSet` exists in `dispatch_contract.rs`
2. inventory-backed resolution rejects any non-empty `capability_overrides`
3. persisted-attach-backed resolution also rejects any non-empty `capability_overrides`
4. the rejection reason explicitly says these overrides are frozen but not supported by the current caller surfaces in slice-29 foundation

Practical meaning:

1. the shape is present
2. the fail-closed behavior is present
3. the actual minimum usable override contract is not yet fully landed

This is slice-29-owned work.

### 4. Caller parity is strong but not obviously complete for every orchestrator-controlled dispatch path

Slice 29 says:

1. human launches and orchestrator-controlled launches both resolve through the same code path
2. no second hidden contract should exist in REPL-only or toolbox-only code

What looks landed:

1. member-start selection now uses the shared resolver through `validate_member_selection(...)`
2. exact runtime selection helpers now route through the shared contract too

What remains unclear or incomplete:

1. `DispatchCallerKind::OrchestratorMemberTurn` exists but appears unused
2. retained world-member turn submission still appears to construct and send `MemberTurnSubmitRequestV1` directly rather than obviously resolving through `DispatchRequestEnvelope`

This is the least clear ownership gap:

1. it is not 30 work
2. it is not 31 work
3. it may be incomplete 29 work if the intended promise was full dispatch parity across all orchestrator-controlled dispatch paths
4. or it may be acceptable 29 foundation work if the intended promise was narrower and the remaining parity step was intentionally deferred without being written down clearly

For closeout purposes, this should be treated as an explicit open item rather than silently assumed complete.

## What Is Not Missing Because It Belongs To 30

The following are later-slice responsibilities and should not be counted as unlanded slice-29 work:

1. public `substrate agent start --scope world`
2. user-facing public scope selection on `agent start`
3. CLI capability flags for that new public world-start surface
4. creating a host-rooted durable session plus world worker without eager host attach as a public entrypoint

Those are explicitly owned by [30-public-world-scoped-agent-start-and-capability-flags.md](./30-public-world-scoped-agent-start-and-capability-flags.md).

## What Is Not Missing Because It Belongs To 31

The following are later-slice responsibilities and should not be counted as unlanded slice-29 work:

1. born-unattached host-rooted session behavior as an operator-visible steady state
2. fresh attach versus continuity attach mode selection at runtime
3. explicit lazy-attach triggering policy
4. attach-worker behavior for sessions born without a host execution client

Those are explicitly owned by [31-lazy-host-attach-for-host-rooted-world-start.md](./31-lazy-host-attach-for-host-rooted-world-start.md).

## Ownership Summary

### Clearly still owned by 29

1. finishing the generalized persisted host attach contract so its durable fields are actually authoritative end to end
2. merging restriction-only inventory policy overlays into resolved `effective_policy`
3. implementing the promised minimum usable override families, or explicitly narrowing the SOW/landed-truth claim to “foundation only”

### Probably owned by 29, but needs an explicit closeout decision

1. whether orchestrator member-turn dispatch must also resolve through the shared dispatch envelope to count as caller parity complete

### Owned by 30

1. public world-scoped `agent start`
2. public scope/capability CLI surface for that entrypoint

### Owned by 31

1. born-unattached session semantics
2. fresh attach realization
3. lazy-attach trigger semantics

## Recommended Closeout Position

The most accurate final statement today is:

1. slice 29 landed the shared dispatch-contract foundation correctly
2. slice 29 did not yet land every contract-completeness item its SOW claims
3. slice 30 and 31 should not be used to absorb the still-missing overlay/override/generalized-attach-truth work
4. a narrow slice-29 follow-on or explicit slice-29 closeout correction is still needed before the stack can honestly claim that 29 is fully complete

## Minimal Follow-On Checklist

If the goal is to finish slice 29 cleanly without broadening into 30/31:

1. make `HostAttachContract` the authoritative source for generalized attach capabilities and launch knobs during persisted-attach resolution
2. ensure session birth and later sync populate those fields from resolved contract truth rather than defaults
3. merge validated restriction-only inventory `policy_overlay` into the resolved `effective_policy`
4. either:
   - implement the minimum supported capability/overlay override families promised by the SOW, or
   - revise the landed-truth claim to say slice 29 landed only the override-contract foundation
5. make an explicit decision on whether orchestrator member-turn parity is required for 29 completeness

That closes the real slice-29 gaps without drifting into the public world-start or lazy-attach work reserved for 30 and 31.
