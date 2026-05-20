# AHCSITC3-spec — platform parity, compatibility, and validation closure

## Behavior delta (single)
- Existing: `platform-parity-spec.md`, `compatibility-spec.md`, and `manual_testing_playbook.md` close the parity, ADR-0025 supersession, and manual proof boundaries, but the pack does not yet have a final execution-ready slice that binds those closeout rules to one implementation and validation boundary.
- New: `AHCSITC3` becomes the authoritative closeout slice, fixing Linux, macOS, and Windows parity expectations, the ADR-0025 supersession boundary, the retained `substrate agents validate` compatibility leaf posture, and the validation evidence surfaces that confirm the feature is ready to ship.
- Why: the feature cannot close until parity, compatibility, and manual proof all refer to one deterministic successor story.

## Scope
- Lock Linux, macOS, and Windows operator-visible parity for `substrate agent list`, `substrate agent status`, `substrate agent doctor`, and the nested-record telemetry split those commands depend on.
- Lock the compatibility rule that ADR-0025 is historical evidence only and that existing `agents.allowed_backends` entries remain valid because `backend_id` keeps the derived adapter grammar.
- Lock the validation boundary on `manual_testing_playbook.md`, `docs/CONFIGURATION.md`, `docs/TRACE.md`, and the cross-platform shell tests named by the impact map.
- Lock the closeout file-owner boundary on `platform-parity-spec.md`, `compatibility-spec.md`, and validation surfaces without reopening earlier contract, protocol, or telemetry ownership.
- Keep the detailed command-surface contract in `AHCSITC0`, the session model in `AHCSITC1`, and the ordered fail-closed or telemetry publication mechanics in `AHCSITC2`.

## Inputs (authoritative)
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/platform-parity-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/compatibility-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/manual_testing_playbook.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/pre-planning/impact_map.md`

## Behavior (authoritative)

### Parity and compatibility closeout
- `AHCSITC3` owns the final execution unit that proves Linux, macOS, and Windows expose one operator-visible meaning for `backend_id`, `execution.scope`, `role`, `world_id`, and `world_generation`.
- `AHCSITC3` keeps `substrate agent list`, `substrate agent status`, and `substrate agent doctor` as the canonical successor namespace on every platform.
- `substrate agents validate` remains the retained compatibility leaf for inventory validation and does not expand into a second canonical namespace.
- ADR-0025 remains superseded historical evidence and no active validation, parity, or compatibility surface treats it as a live contract.

### Validation evidence boundary
- `manual_testing_playbook.md` carries the deterministic operator proof for list, status, doctor, nested-record visibility, and one-owner-per-surface review.
- `docs/CONFIGURATION.md` and `docs/TRACE.md` reflect the successor wording and the trace-visible tuple-compatible field rules already fixed by prior slices.
- Cross-platform tests and world warm or smoke evidence confirm the same fail-closed posture and the same pure-agent versus nested-record split on Linux, macOS, and Windows.

## Acceptance criteria
- AC-AHCSITC3-01: Linux, macOS, and Windows publish one operator-visible meaning for `backend_id`, `execution.scope`, `role`, `world_id`, and `world_generation` across list, status, doctor, and the related trace-visible surfaces.
- AC-AHCSITC3-02: Linux, macOS, and Windows preserve one operator-visible split between pure-agent records and nested gateway-backed records, including the rule that nested records omit `world_id` and `world_generation`.
- AC-AHCSITC3-03: `substrate agent list`, `substrate agent status`, and `substrate agent doctor` remain the canonical successor namespace on every platform, while `substrate agents validate` remains only the inventory-validation compatibility leaf.
- AC-AHCSITC3-04: Active parity, compatibility, validation, and rollout surfaces treat ADR-0025 as superseded historical evidence and do not reuse backend-id-centric role wording as a live contract.
- AC-AHCSITC3-05: Existing `agents.allowed_backends` entries remain valid without translation because `backend_id` keeps the derived `<kind>:<agent_id>` grammar throughout parity and compatibility closure.
- AC-AHCSITC3-06: Manual validation and cross-platform test surfaces named by the impact map cover list, status, doctor, nested-record visibility, and fail-closed world-boundary handling without reopening ownership already fixed by earlier slices.

## Out of scope
- Rewriting earlier contract wording for list, status, or doctor field order.
- Reopening capability-descriptor grammar, lifecycle transitions, or restart-handle object details.
- Reopening ordered deny taxonomy or top-level telemetry field placement mechanics.
