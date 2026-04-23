# llm-and-agent-identity-tuple-and-deployment-posture — CI checkpoint plan

This file defines where later multi-platform verification is expected to happen for this feature.

Standard:
- `docs/project_management/system/fse/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/minimal_spec_draft.md`

## Operator rules
- This plan is authoritative for checkpoint intent during pre-planning.
- If reality diverges from this plan, update this plan first.
- This document remains advisory until downstream FSE planning or decomposition turns the checkpoint cadence into concrete execution behavior.
- Before downstream seam planning starts, run document validation across the selected pre-planning artifacts and the authored docs attached to `LAITDP-01` and `LAITDP-02`.
- At the first checkpoint, run the external-alignment gate against `docs/contracts/substrate-gateway-status-schema.md` and `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md` before any downstream plan depends on widened tuple and posture publication.
- At every checkpoint, run micro-lint and ambiguity scans across every authored markdown file that lands inside the checkpoint boundary.

## Applicability
- Checkpoint planning applies to this feature because the authoritative inputs lock Linux, macOS, and Windows parity for tuple and placement-posture semantics and reuse security-sensitive routing, status, and trace surfaces.
- Later verification falls into three layers:
  - compile parity on shared gateway runtime, shell, and trace publication surfaces,
  - targeted feature smoke on gateway status publication and unavailable-shape behavior,
  - deeper CI and manual parity review when platform rollout and validation evidence land.

## Machine-readable plan

```json
{
  "version": 1,
  "defaults": {
    "min_draft_seams_per_checkpoint": 2,
    "max_draft_seams_per_checkpoint": 6
  },
  "platform_scope": {
    "behavior_platforms_required": ["linux", "macos", "windows"],
    "ci_parity_platforms_required": ["linux", "macos", "windows"],
    "wsl_status": "follow_up_required"
  },
  "checkpoints": [
    {
      "checkpoint_id": "CP1",
      "draft_seam_ids": ["LAITDP-01", "LAITDP-02"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "targeted"
      },
      "doc_gates": [
        "document_validation",
        "status_schema_external_alignment",
        "trace_vocabulary_external_alignment",
        "micro_lint",
        "ambiguity_scan"
      ],
      "rationale": "Lock tuple vocabulary, placement-posture semantics, routing-hint behavior, and additive status and trace publication before platform rollout work starts. This boundary stabilizes shared gateway contract, policy, status, and trace surfaces and creates the required document-validation and external-alignment gate before downstream seam planning consumes the authored specs."
    },
    {
      "checkpoint_id": "CP2",
      "draft_seam_ids": ["LAITDP-03"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "deeper"
      },
      "doc_gates": [
        "manual_validation_playbook_review",
        "micro_lint",
        "ambiguity_scan"
      ],
      "rationale": "Run the heavy parity checkpoint after platform rollout and validation evidence lands. This single-seam checkpoint is justified because LAITDP-03 owns Linux, macOS, and Windows parity proof, compatibility proof, terminology rollout, and the manual validation evidence that closes the feature."
    }
  ]
}
```

## Human-readable rationale

### CP1 — Contract and observability stabilization
- Draft seams: `LAITDP-01`, `LAITDP-02`
- Code-grounded boundary: `minimal_spec_draft.md` assigns seam 1 to operator wording and schema boundaries and assigns seam 2 to routing-hint policy, additive status publication, redaction, and trace vocabulary alignment. `impact_map.md` ties those seams to `crates/shell/src/builtins/world_gateway.rs`, `crates/world-agent/src/gateway_runtime.rs`, `crates/trace/src/span.rs`, `docs/contracts/substrate-gateway-status-schema.md`, and `docs/TRACE.md`.
- Stabilized surfaces: operator-visible tuple vocabulary, placement-posture vocabulary, routing-hint evaluation semantics, additive status-field placement outside `client_wiring.*`, additive trace publication, and the rule that `backend_id` stays distinct from tuple meaning.
- Intended gates:
  - Document validation before downstream seam planning starts.
  - External alignment review for status-schema widening and ADR-0028 trace-vocabulary widening.
  - Compile parity on the shared gateway runtime, shell, and trace publication paths.
  - Targeted feature smoke on gateway status publication and unavailable-shape behavior once downstream implementation exists.
  - Targeted CI testing on the shared gateway runtime, shell, and trace publication paths.
  - Micro-lint and ambiguity scans on the authored docs inside this checkpoint boundary.
- Risk reduced: field-family drift between status and trace, backend-id overload drift, tuple-field redaction regressions, and consumer confusion around correlation keys or placement posture.
- Remaining downstream confirmation: the exact top-level status and diagnostics field family for tuple publication, the exact trace field family and placement, and the exact absence semantics for `provider` and `auth_authority` before provider selection completes.

### CP2 — Platform rollout and validation
- Draft seams: `LAITDP-03`
- Code-grounded boundary: `minimal_spec_draft.md` assigns seam 3 to platform parity, compatibility proof, terminology rollout, validation evidence, and bridge transport invariants across Linux, macOS, and Windows. `spec_manifest.md` assigns the acceptance boundary for Linux, macOS, and Windows parity review to `platform-parity-spec.md`, `compatibility-spec.md`, and `manual_testing_playbook.md`.
- Stabilized surfaces: Linux, macOS, and Windows parity guarantees for tuple and placement-posture semantics, the bridge transport-only invariant, the compatibility posture for retiring overloaded backend terminology, and the manual validation evidence that proves one owner per surface.
- Intended gates:
  - Compile parity across the shared gateway runtime and validation surfaces that expose platform parity.
  - Feature smoke across Linux, macOS, and Windows on gateway status publication and terminology-sensitive operator flows.
  - Deeper CI testing and release-safe parity review across Linux, macOS, and Windows after rollout evidence lands.
  - Manual validation playbook review for tuple vocabulary, placement posture, compatibility proof, and one-owner-per-surface proof.
  - Micro-lint and ambiguity scans on the authored docs inside this checkpoint boundary.
- Risk reduced: platform-specific semantic drift, hidden transport divergence leaking into the public contract, compatibility regressions in operator messaging, and rollout evidence gaps at feature closeout.
- Remaining downstream confirmation: the exact verification cadence once downstream planning locks the touched runtime and validation surfaces and the exact WSL stance after downstream runtime review confirms whether any WSL-specific surface enters scope.

## Follow-ups

- Replace the draft seam IDs in this plan with the final downstream seam identifiers once seam planning locks them.
- Confirm the exact platform scope and verification cadence after downstream planning locks the touched runtime, status, trace, and validation surfaces.
- Convert this checkpoint intent into concrete execution wiring only in the downstream subsystem that owns execution.
- Freeze the exact top-level status and diagnostics field family that publishes tuple and placement-posture metadata outside `client_wiring.*`.
- Freeze the exact trace field family and placement that augment ADR-0028 correlation keys.
- Confirm whether downstream runtime planning touches WSL-specific bridging, path-translation, or parity surfaces. Add WSL as a CI parity environment only when that review finds a concrete surface.
