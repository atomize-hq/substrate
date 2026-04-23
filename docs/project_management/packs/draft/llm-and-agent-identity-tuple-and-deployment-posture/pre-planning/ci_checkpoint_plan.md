# llm-and-agent-identity-tuple-and-deployment-posture — CI checkpoint plan

This file defines when cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/minimal_spec_draft.md`

## Operator rules
- This plan is authoritative for checkpoint cadence during pre-planning.
- If slice ids, platform scope, or checkpoint boundaries change, update this plan first.
- When full planning writes `tasks.json`, set `meta.checkpoint_boundaries` to `["LAITDP1", "LAITDP2"]`.
- Before work that depends on widened tuple publication moves forward, run document validation against the selected pre-planning artifacts and the authored docs attached to `LAITDP0` and `LAITDP1`.
- At every checkpoint, run micro-lint and ambiguity scans across every authored markdown file that lands inside the checkpoint boundary.

## Applicability
- Checkpoint planning applies to this feature because the authoritative inputs lock Linux, macOS, and Windows parity for tuple and placement-posture semantics and reuse security-sensitive routing, status, and trace surfaces.
- Later verification falls into three layers:
  - compile parity on shared gateway runtime, shell, and trace publication surfaces
  - targeted feature smoke on gateway status publication and unavailable-shape behavior
  - deeper CI and manual parity review when platform rollout and validation evidence land

## Machine-readable plan (linted)

```json
{
  "version": 1,
  "defaults": {
    "min_triads_per_checkpoint": 1,
    "max_triads_per_checkpoint": 4
  },
  "checkpoints": [
    {
      "id": "CP1",
      "task_id": "CP1-ci-checkpoint",
      "slices": ["LAITDP0", "LAITDP1"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "targeted"
      },
      "rationale": "Lock tuple vocabulary, placement-posture semantics, routing-hint behavior, and additive status and trace publication before platform-rollout work starts. This checkpoint stabilizes the shared contract, policy, status, and trace surfaces."
    },
    {
      "id": "CP2",
      "task_id": "CP2-ci-checkpoint",
      "slices": ["LAITDP2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "deeper"
      },
      "rationale": "Run the parity and rollout checkpoint after platform-rollout and validation evidence land. This checkpoint closes the compatibility and manual-validation surfaces for Linux, macOS, and Windows."
    }
  ]
}
```

## Human-readable rationale

### CP1 (`LAITDP1`) — contract, schema, policy, and observability stabilization
- Slices: `LAITDP0`, `LAITDP1`
- Code-grounded boundary: `minimal_spec_draft.md` assigns `LAITDP0` to operator wording and schema boundaries and assigns `LAITDP1` to routing-hint policy, additive status publication, redaction, and trace vocabulary alignment.
- Stabilized surfaces:
  - operator-visible tuple vocabulary
  - placement-posture vocabulary
  - routing-hint evaluation semantics
  - additive status-field placement outside `client_wiring.*`
  - additive trace publication
  - the rule that `backend_id` stays distinct from tuple meaning
- Risk reduced:
  - field-family drift between status and trace
  - backend-id overload drift
  - tuple-field redaction regressions
  - consumer confusion around correlation keys or placement posture

### CP2 (`LAITDP2`) — platform rollout and validation closure
- Slices: `LAITDP2`
- Code-grounded boundary: `minimal_spec_draft.md` assigns `LAITDP2` to platform parity, compatibility proof, terminology rollout, validation evidence, and bridge transport invariants across Linux, macOS, and Windows.
- Stabilized surfaces:
  - Linux, macOS, and Windows parity guarantees for tuple and placement-posture semantics
  - the bridge transport-only invariant
  - the compatibility posture for retiring overloaded backend terminology
  - the manual validation evidence that proves one owner per surface
- Risk reduced:
  - platform-specific semantic drift
  - hidden transport divergence leaking into the public contract
  - compatibility regressions in operator messaging
  - rollout evidence gaps at feature closeout

## Follow-ups

- Freeze the exact top-level status and diagnostics field family that publishes tuple and placement-posture metadata outside `client_wiring.*`.
- Freeze the exact trace field family and placement that augment ADR-0028 correlation keys.
- Confirm whether downstream runtime planning touches WSL-specific bridging, path-translation, or parity surfaces.
