# Remediation Log - ChatGPT Codex OAuth Backend-API Responses

No remediations are opened during extraction by default. Risks and unknowns are captured in seam briefs and become remediations only when a pre-exec or post-exec gate raises a concrete issue.

## Open remediations

remediation_id: REM-001
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-2
related_slice: S00
related_thread: THR-15
related_contract: C-15
related_artifact: crates/gateway/docs/project_management/packs/active/chatgpt-codex-oauth-backend-api-responses/threaded-seams/seam-2-substrate-auth-handoff-and-account-id-provenance/review.md
severity: blocking
status: open
owner_seam: SEAM-2
blocked_targets:
  - seam: SEAM-2
    field: status
    value: exec-ready
summary: The owned auth-handoff contract baseline is not yet written into the canonical contract path, so SEAM-2 cannot become exec-ready.
required_fix: Author docs/contracts/chatgpt-codex-auth-handoff-contract.md and mirror the owner line, field precedence, and fallback rules into the seam-local planning artifacts.
resolution_evidence: []

When a remediation is opened, append it here using the canonical schema:

```text
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
