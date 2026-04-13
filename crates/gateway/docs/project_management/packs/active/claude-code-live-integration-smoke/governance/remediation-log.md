# Remediation Log - Claude Code Live Integration Smoke

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
