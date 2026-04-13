# Remediation Log - Azure Foundry Provider Transport

No remediations are opened during extraction by default. Risks and unknowns are captured in seam briefs and become remediations only when a pre-exec or post-exec gate raises a concrete issue.

## Open remediations

None.

When a remediation is opened, append it here using the canonical schema:

```yaml
remediation_id: REM-<nnn>
origin_phase: pre_exec | post_exec
source_gate: review | contract | revalidation | landing | closeout
related_seam: SEAM-<n> | null
related_slice: null
related_thread: THR-<nn> | null
related_contract: C-<nn> | null
related_artifact: <repo-relative-path> | null
severity: blocking | material | follow_up
status: open | in_progress | resolved | accepted_risk | carried_forward
owner_seam: SEAM-<n>
blocked_targets:
  - seam: SEAM-<n>
    field: status | execution_horizon
    value: proposed | decomposed | exec-ready | in-flight | landed | closed | active | next | future
summary: <one-sentence machine-readable finding summary>
required_fix: <one-sentence explicit fix>
resolution_evidence: []
```

## Resolved remediations

Move resolved items here using the same schema with `status: resolved` and populated `resolution_evidence`.

```yaml
remediation_id: REM-001
origin_phase: post_exec
source_gate: closeout
related_seam: SEAM-2
related_slice: null
related_thread: THR-07
related_contract: C-08
related_artifact: docs/project_management/packs/active/azure-foundry-provider-transport/governance/seam-2-closeout.md
severity: blocking
status: resolved
owner_seam: SEAM-2
blocked_targets:
  - seam: SEAM-2
    field: status
    value: closed
summary: SEAM-2 could not publish THR-07 until redacted live Azure smoke evidence existed for both Kimi routes.
required_fix: Capture redacted live Azure `/v1/messages` evidence for both routes and publish THR-07.
resolution_evidence:
  - docs/project_management/packs/active/azure-foundry-provider-transport/threaded-seams/seam-2-azure-live-smoke-operator-readiness/evidence/manifest.json
  - docs/project_management/packs/active/azure-foundry-provider-transport/governance/seam-2-closeout.md
```
