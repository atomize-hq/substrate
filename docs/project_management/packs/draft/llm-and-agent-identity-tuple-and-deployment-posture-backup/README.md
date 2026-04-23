# llm-and-agent-identity-tuple-and-deployment-posture

Source:
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`

This pack is the semantic planning companion to ADR-0042. It exists to make the identity tuple and
deployment-posture model consumable by downstream work without forcing later ADRs to restate tuple
meanings locally.

Start here:
- `plan.md`
- `spec_manifest.md`
- `contract.md`

Supporting docs:
- `impact_map.md`
- `decision_register.md`
- `manual_testing_playbook.md`

Pack posture:
- planning-only
- no new config or policy keys
- no code or test slices in this pack
- ADR-0043 consumes tuple semantics from this pack and only owns additive `llm.constraints.*`
- ADR-0044 consumes tuple semantics from this pack and only owns agent-hub successor behavior
