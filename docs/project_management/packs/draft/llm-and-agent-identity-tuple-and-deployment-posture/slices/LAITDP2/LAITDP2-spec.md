# LAITDP2-spec — platform rollout and validation lock

## Behavior delta (single)
- Existing: contract, policy, and telemetry surfaces are pinned, but the pack does not yet have a closing execution slice that fixes Linux, macOS, and Windows parity guarantees, the tuple-vocabulary rollout boundary against `backend_id`, and the validation evidence required to prove those guarantees.
- New: `LAITDP2` becomes the authoritative parity-and-rollout slice, defining one operator-visible tuple and posture meaning across all supported platforms, one compatibility end state for new docs and diagnostics, and one deterministic manual review procedure for owner-line and example stability.
- Why: shipping tuple vocabulary without parity and compatibility closure would leave platform guidance, diagnostics, and validation evidence inconsistent.

## Scope
- Lock Linux, macOS, and Windows parity guarantees for tuple semantics and placement-posture semantics.
- Lock the hidden-divergence boundary that treats Unix sockets, Lima forwarding, and WSL transport as implementation detail rather than new operator-facing semantics.
- Lock the compatibility rule that `backend_id` remains the adapter selector while tuple vocabulary carries semantic identity.
- Lock the historical-evidence rule for older overloaded backend wording.
- Lock the deterministic review procedure and stale-reference scans that prove the pack keeps one owner per surface.
- Leave routing evaluation, status placement, and trace placement semantics with the earlier slices that already own them.

## Inputs (authoritative)
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/platform-parity-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/compatibility-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/manual_testing_playbook.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md`
- `docs/contracts/substrate-gateway-runtime-parity.md`
- `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`

## Behavior (authoritative)

### Platform-parity boundary
- Linux, macOS, and Windows publish one operator-visible meaning for `client`, `router`, `provider`, `auth_authority`, `protocol`, `in_world`, `host_only`, and `host_to_world_bridge`.
- `router=direct_provider_path` requires `placement_posture.execution="host_only"` on every platform.
- `router=direct_provider_path` is invalid with `placement_posture.host_to_world_bridge=true` on every platform.
- `host_to_world_bridge` remains transport-only on every platform and does not alter in-world `net_allowed` governance.
- Windows WSL, macOS Lima forwarding, and Linux Unix-socket plumbing remain hidden transport detail only and never become second operator-facing contract surfaces.

### Compatibility-rollout boundary
- New operator-facing docs, planning specs, playbooks, examples, and diagnostics use tuple vocabulary for semantic identity.
- New operator-facing docs may mention `backend_id` only when they mean adapter selection, backend inventory identity, or allowlist selection.
- Additive tuple publication remains outside `client_wiring.*` and keeps ADR-0028 correlation keys unchanged.
- `direct_provider_path` remains routing authority only and never collapses into backend identity.
- Historical wording that predates ADR-0042 may remain visible only when the document is explicitly marked historical evidence or superseded context.

### Validation-evidence boundary
- The manual review procedure audits one owner for each surfaced area: tuple meanings, machine-readable schema ownership, policy and telemetry owner lines, and parity or compatibility closure.
- The manual review procedure includes the Claude Code example, the Codex plus `~/.codex/auth.json` example, and the pre-provider-selection omission example.
- The manual review procedure includes stale-reference searches for overloaded backend wording, bridge wording that implies a second control plane, status-schema drift, and active references that present archived wording as current ownership.
- Validation evidence remains bounded to the contract, schema, policy, telemetry, parity, compatibility, and manual-review surfaces named by this pack plus the cited external authorities.

## Acceptance criteria
- AC-LAITDP2-01: Linux, macOS, and Windows publish one operator-visible tuple and placement-posture meaning, while Unix-socket, Lima, and WSL transport differences stay hidden behind the parity boundary.
- AC-LAITDP2-02: The router and posture invariants from earlier slices remain unchanged across all three platforms, including the rule that bridge transport does not alter in-world `net_allowed` governance.
- AC-LAITDP2-03: New operator-facing docs and diagnostics use tuple vocabulary for semantic identity, while `backend_id` remains the `<kind>:<name>` adapter selector and never substitutes for tuple meaning.
- AC-LAITDP2-04: Additive tuple publication remains backward compatible with the existing status-schema and trace-owner boundaries by staying outside `client_wiring.*` and preserving ADR-0028 correlation keys unchanged.
- AC-LAITDP2-05: The manual review procedure defines the one-owner-per-surface audit plus the Claude Code, Codex, and pre-provider-selection review cases as required validation evidence for this feature.
- AC-LAITDP2-06: Active docs that overload `backend_id` into tuple semantics or treat `host_to_world_bridge` as a second router are compatibility defects, and older wording may remain only when it is explicitly marked historical or superseded.

## Out of scope
- Tuple field definitions and schema-shape ownership from `LAITDP0`.
- Routing-hint evaluation rules and observability placement ownership from `LAITDP1`.
- New CLI commands, new config keys, or new platform-specific execution semantics beyond the parity and compatibility guarantees named here.
