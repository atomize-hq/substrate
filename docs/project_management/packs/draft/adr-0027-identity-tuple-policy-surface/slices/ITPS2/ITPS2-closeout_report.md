# ITPS2 Closeout Report

## Status
- Recommendation: `ACCEPT`

## Integration summary
- Integrated the substantive `ITPS2-test` branch commit (`20603b9b`, cherry-picked from `60c626eb0a2364842919998c4864b5e3b94cb3c0`) onto `adr-0027-identity-tuple-policy-surface-itps2-integ`.
- The integrated slice stays within the existing ADR-0027 policy surface and adds coverage that locks the ITPS2 telemetry and compatibility seam without introducing a second tuple root, config root, or trace-only schema.

## Commands run
- `cargo test -p shell policy_patch_parses_tuple_axes_under_existing_llm_constraints_root -- --nocapture`
- `cargo test -p shell policy_patch_rejects_second_tuple_policy_roots_and_unknown_tuple_axes -- --nocapture`
- `cargo test -p shell --test world_gateway -- --nocapture`
- `cargo fmt --all`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `make integ-checks`

## Spec reconciliation
- `AC-ITPS2-01` and `AC-ITPS2-02`: verified by the existing gateway status contract coverage plus the integrated allow-path status test, which keeps `identity_tuple` and `placement_posture` top-level and out of `client_wiring.*`.
- `AC-ITPS2-03`: verified by the integrated allow-path status test, which asserts deny-only fields are omitted on allow publication, while the existing deny-path gateway tests continue to require the policy failure detail.
- `AC-ITPS2-04`: verified by the existing gateway lifecycle request assertions that keep `backend_id` as the adapter/correlation selector while tuple-axis routing remains separate.
- `AC-ITPS2-05`: verified by the integrated empty-or-absent constraint coverage, which preserves backend selection and operator-visible workflow when all tuple-axis lists are absent or empty.
- `AC-ITPS2-06` and `AC-ITPS2-07`: verified by the integrated policy patch parsing/rejection tests, which accept tuple axes only under `llm.constraints.*` and reject second tuple-policy roots plus unknown future-axis keys.

## Drift resolved during integration
- Avoided merging planning-doc churn from the task branches; only the substantive Rust test changes were integrated into this worktree.
- Confirmed the current runtime behavior already satisfies the ITPS2 seam once the missing parser and gateway regression coverage is present.

## Remaining follow-ups
- None for `ITPS2-integ`.
