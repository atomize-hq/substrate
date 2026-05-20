# ITPS3-spec — validation closure, checkpoint alignment, and promotion packaging

## Behavior delta (single)
- Existing: the pack has contract, schema, runtime-policy, telemetry, and compatibility surfaces, but it does not yet have one authoritative closing slice that binds manual validation, final CI checkpoint alignment, and promotion packaging into a deterministic end-of-pack seam.
- New: `ITPS3` becomes the authoritative validation-and-promotion slice, fixing the cross-platform review matrix, the one-final-checkpoint rule after `ITPS3`, the one-owner-per-surface validation boundary, and the promotion target into the implemented ADR-0027 pack.
- Why: the feature cannot exit full planning cleanly unless validation evidence, checkpoint posture, and closeout packaging are locked as one final slice.

## Scope
- Lock the manual validation closure for Linux, macOS, and Windows against the authored tuple-policy docs and current CLI or test surfaces.
- Lock the final CI checkpoint alignment that places one `CP1` boundary after `ITPS3`.
- Lock the promotion target and closeout boundary into the implemented ADR-0027 pack.
- Lock the one-owner-per-surface validation rule across contract, schema, policy, telemetry, compatibility, and playbook docs.
- Leave telemetry-field placement, additive rollout invariants, and `backend_id` compatibility ownership with `ITPS2`.
- Leave `tasks.json`, `plan.md`, `session_log.md`, and kickoff prompt writing to the single-writer checkpoints lane.

## Inputs (authoritative)
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/manual_testing_playbook.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/telemetry-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/compatibility-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/policy-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/workstream_triage.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- `crates/broker/src/tests.rs`
- `crates/shell/src/builtins/world_gateway.rs`
- `crates/shell/tests/world_gateway.rs`

## Behavior (authoritative)

### Validation closure seam
- `ITPS3` closes the manual review matrix for the authoritative `substrate policy current show --explain` surface, schema-invalid tuple-policy input, tuple-aware gateway status publication, and the router, provider, protocol, and auth-authority deny families.
- Validation closure covers Linux, macOS, and Windows as one operator contract, even when the execution substrate differs across world backends.
- Validation examples that mention paths such as `~/.codex/auth.json` remain validation-only examples and do not become new Substrate-owned path contracts.
- Validation closure includes one-owner-per-surface review across `contract.md`, `tuple-policy-schema-spec.md`, `policy-spec.md`, `telemetry-spec.md`, `compatibility-spec.md`, and `manual_testing_playbook.md`.
- Validation closure includes stale-reference review for overloaded `backend_id` wording, config-versus-policy inspection drift, and any wording that implies telemetry owns tuple semantics.

### Checkpoint alignment seam
- `ITPS3` is the final slice before the only pre-planned CI checkpoint boundary for this feature.
- The accepted slice order remains `ITPS0`, `ITPS1`, `ITPS2`, `ITPS3`.
- `CP1-ci-checkpoint` is aligned to run after `ITPS3` and to validate the completed broker, shell, trace, and operator-contract seam together.
- `meta.checkpoint_boundaries = ["ITPS3"]` remains the required final-task wiring target once the single-writer planning lane materializes `tasks.json`.
- `ITPS3` does not create a second checkpoint or split the existing cross-platform gate into platform-local promotion paths.

### Promotion packaging seam
- Promotion closes into the implemented ADR-0027 pack by extending its contract and schema surfaces with the tuple-axis policy additions locked by this feature.
- Promotion packaging keeps ADR-0042 as the owner of tuple semantics and ADR-0028 as the owner of the base trace envelope.
- Promotion packaging requires the implemented ADR-0027 pack to absorb the authoritative policy inspection surface, tuple-axis schema tables, additive rollout wording, and validation-ready operator contract text.
- Promotion packaging does not move telemetry-field ownership out of `ITPS2`, does not move runtime-ordering ownership out of `ITPS1`, and does not create a second implemented pack for the same policy family.

## Acceptance criteria
- AC-ITPS3-01: `ITPS3` states that manual validation closes the authoritative policy inspection surface, schema-invalid input behavior, tuple-aware status publication, and router, provider, protocol, plus auth-authority deny cases.
- AC-ITPS3-02: `ITPS3` states that Linux, macOS, and Windows are validated as one operator contract and that example auth-file paths remain validation-only examples rather than new product path contracts.
- AC-ITPS3-03: `ITPS3` states that validation includes a one-owner-per-surface review across contract, schema, policy, telemetry, compatibility, and playbook docs plus stale-reference scans for overloaded `backend_id` or config-view wording.
- AC-ITPS3-04: `ITPS3` states that the accepted slice order is `ITPS0`, `ITPS1`, `ITPS2`, `ITPS3` and that the only planned CI checkpoint boundary runs after `ITPS3`.
- AC-ITPS3-05: `ITPS3` states that `CP1-ci-checkpoint` and `meta.checkpoint_boundaries = [\"ITPS3\"]` are the required final-task wiring targets for the single-writer checkpoints lane.
- AC-ITPS3-06: `ITPS3` states that promotion closes into the implemented ADR-0027 pack by extending existing contract and schema surfaces rather than creating a second implemented policy pack.
- AC-ITPS3-07: `ITPS3` states that telemetry-field ownership remains with `ITPS2` and runtime-ordering ownership remains with `ITPS1` during promotion packaging.

## Out of scope
- Tuple-aware status and trace field placement, additive rollout invariants, and `backend_id` compatibility ownership from `ITPS2`.
- Runtime fail-early ordering, deny taxonomy, and explain-surface ownership from `ITPS1`.
- `tasks.json`, `plan.md`, `session_log.md`, and kickoff prompt edits owned by `ITPS-PWS-tasks_checkpoints`.
