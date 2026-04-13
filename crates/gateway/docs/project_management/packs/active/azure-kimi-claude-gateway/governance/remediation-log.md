# Remediation Log - Azure Kimi Claude Gateway

No remediations are opened during extraction by default. Risks and unknowns are captured in seam briefs and become remediations only when a pre-exec or post-exec gate raises a concrete issue.

## Open remediations

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
origin_phase: pre_exec
source_gate: revalidation
related_seam: SEAM-2
related_slice: null
related_thread: THR-01
related_contract: C-01
related_artifact: docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-1-closeout.md
severity: blocking
status: resolved
owner_seam: SEAM-2
blocked_targets:
  - seam: SEAM-2
    field: execution_horizon
    value: active
summary: SEAM-2 horizon promotion was unblocked by recording that THR-01 is already published and that Azure hidden-tool revalidation is owned by SEAM-2 active work instead of SEAM-1 promotion-readiness.
required_fix: Keep the Azure hidden-tool validation gap explicit as SEAM-2 normalization work and preserve the published C-01 handoff from SEAM-1 without treating the carried gap as a foundation-horizon blocker.
resolution_evidence:
  - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-1-closeout.md
  - docs/project_management/packs/active/azure-kimi-claude-gateway/seam-2-azure-kimi-normalization.md
  - docs/project_management/packs/active/azure-kimi-claude-gateway/threading.md
  - docs/project_management/packs/active/azure-kimi-claude-gateway/README.md
  - docs/project_management/packs/active/azure-kimi-claude-gateway/scope_brief.md
  - docs/project_management/packs/active/azure-kimi-claude-gateway/seam_map.md
```

```yaml
remediation_id: REM-002
origin_phase: post_exec
source_gate: closeout
related_seam: SEAM-2
related_slice: S4
related_thread: THR-02
related_contract: C-02
related_artifact: docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-2-closeout.md
severity: blocking
status: resolved
owner_seam: SEAM-2
blocked_targets:
  - seam: SEAM-2
    field: status
    value: closed
  - seam: SEAM-3
    field: execution_horizon
    value: active
summary: SEAM-2 could not publish THR-02 while streamed hidden-marker-only Azure responses bypassed the landed C-02 normalization path and left downstream seams dependent on raw provider reasoning deltas.
required_fix: Extend the SEAM-2 streaming provider boundary so hidden-marker-only `reasoning_content` deltas normalize into the same C-02 `tool_intent` and `action` path as non-stream hidden markers and explicit `tool_calls`, then rerun seam-2 closeout against streamed regression evidence before retrying SEAM-3 horizon promotion.
resolution_evidence:
  - gateway/src/providers/openai.rs
  - gateway/tests/fixtures/azure_kimi/hidden-markers-k2-thinking-stream.json
  - docs/project_management/packs/active/azure-kimi-claude-gateway/threaded-seams/seam-2-azure-kimi-normalization/evidence/cases/hidden-markers-k2-thinking-stream/raw-response.json
  - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-2-closeout.md
  - docs/project_management/packs/active/azure-kimi-claude-gateway/threading.md
```
