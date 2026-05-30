# CLOSEOUT-31.25: Prompt-Free Fresh Attach Remediation

Source spec: [SPEC-31_25.md](./SPEC-31_25.md)  
Source plan: [PLAN-31_25.md](./PLAN-31_25.md)  
Source tasks: [TASKS-31_25.md](./TASKS-31_25.md)  
Historical gap context: [CLOSEOUT-31-packet-4.md](./CLOSEOUT-31-packet-4.md)  
Date: `2026-05-30`

## Status

Slice 31.25 is complete on 2026-05-30.

This corrective closeout is the authoritative shipped-behavior record for the remediation. The historical Packet 4 closeout remains useful as gap context, but its planner/bootstrap-only prompt-free fresh attach claim is superseded by this fail-closed contract.

## Restored Contract

1. Control-only attach is continuity-only in this slice; missing continuity truth does not authorize a fresh control-only launch.
2. Automatic attach planning remains rooted in persisted `HostAttachContract` truth, but fresh-needed outcomes fail closed before backend launch.
3. Prompt-free fresh attach is not a shipped runtime behavior for Codex or Claude in this slice.
4. Any future sanctioned non-prompt fresh attach mechanism remains deferred to a separate reviewed slice.

## Corrective Scope

1. Regression coverage now pins fail-closed startup and fail-closed auto-attach planning instead of treating fresh attach without continuity as valid behavior.
2. Slice 31.25 spec, plan, and tasks now point at the fail-closed regressions that exist in the tree and carry the corrected standalone remediation truth.
3. No slice-31 planning artifact rewrite was required for this corrective record; slice 31.25 stands on its own as the authoritative remediation set.

## Validation

The following commands passed on 2026-05-30:

1. `cargo test -p shell --lib hidden_owner_helper_attach_startup_fails_closed_without_continuity -- --nocapture`
2. `cargo test -p shell --lib auto_attach_launch_plan_fails_closed_when_continuity_is_unavailable_and_fresh_would_be_required -- --nocapture`
3. `cargo fmt --all -- --check`
4. `cargo clippy --workspace --all-targets -- -D warnings`
5. `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
6. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
7. `cargo test --workspace -- --nocapture`

## Deferred Scope

1. This remediation does not sanction a new non-prompt fresh attach primitive.
2. This remediation does not reopen slice 31 product direction beyond restoring fail-closed truth.
3. Any future fresh attach proposal must land as a separate reviewed slice with an explicit shared contract.
