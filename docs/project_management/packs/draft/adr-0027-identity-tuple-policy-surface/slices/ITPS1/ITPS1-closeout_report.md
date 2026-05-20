# ITPS1 Closeout Report

## Status
- Recommendation: `ACCEPT`

## Final integrated commands run for `ITPS1-integ`
- `cargo test -p substrate-broker c1_itps1_policy_spec_locks_runtime_order_fail_early_rules_and_failure_buckets -- --nocapture`
- `cargo test -p substrate-broker c1_itps1_decision_register_locks_tuple_family_reuse_and_explain_surface_ownership -- --nocapture`
- `cargo test -p shell world_gateway_backend_allowlist_denial_happens_before_tuple_narrowing -- --nocapture`
- `cargo test -p shell world_gateway_tuple_narrowing_denies_unresolved_auth_authority -- --nocapture`
- `cargo fmt`
- `cargo test -p substrate-broker c1_itps1 -- --nocapture`
- `cargo test -p shell --test world_gateway -- --nocapture`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `make integ-checks`

## Spec-to-implementation drift resolved during integration
- Reconciled the code/test branch merge so `llm.allowed_backends` is enforced before tuple narrowing, while router, protocol, and provider constraints are still checked before auth-source resolution.
- Deferred `auth_authority` derivation and `llm.constraints.auth_authorities` enforcement until after integrated auth resolution, which preserves the ITPS1 fail-early ordering without letting unresolved auth fall through to transport/component-unavailable handling.
- Reapplied the ITPS1 coverage additions without carrying forward the stale planning-doc deletions present on the code/test side branches.

## Outcome
- Integrated runtime behavior satisfies the ITPS1 ordering and deny-taxonomy requirements.
- Broker docs/tests now lock the ITPS1 policy spec and decision register strings required by `AC-ITPS1-01` through `AC-ITPS1-08`.
- Shell integration coverage verifies backend-before-tuple gating, tuple-axis order, unresolved auth-authority denial, blocked env auth denial, and incomplete env auth invalid-integration behavior.

## Remaining follow-ups
- None for `ITPS1-integ`.
