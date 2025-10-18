# Phase Handoff Prompt Template
Use this prompt after completing a phase to instruct the outgoing agent to craft a fully contextual handoff for the next session. Replace placeholders before delivering it to the agent who just finished their work.

```
You have just completed Phase <W/M/L/Final> of the Substrate transport parity spike.
Prepare a detailed handoff message for the next operator so they can resume with zero prior context.

Your handoff message must include the following sections:

1. Phase Recap
   - Summarize the objective of Phase <phase> and confirm its completion status.
   - Mention the exact branch and commit you ended on.

2. Execution Evidence Highlights
   - Enumerate the most critical commands run (reference the evidence log entry) and their outcomes.
   - Note where full output is stored (e.g., `docs/project_management/logs/<platform>_always_world.md#<anchor>`).

3. Wildcards / Non-Documented Findings
   - List any unexpected behaviors, manual tweaks, or environment quirks that are not already covered in `docs/SPIKE_TRANSPORT_PARITY_PLAN.md`.
   - Include remediation steps taken and anything still unresolved.

4. Required Reading For The Next Session
   - Provide an ordered list of files/directories the next operator must read completely before touching the code (minimum: updated source files, relevant docs under `docs/dev`, evidence log entry).
   - Highlight any diffs or config files that changed and need close inspection.

5. Guardrails To Honor
   - Restate the key guardrails from `docs/SPIKE_TRANSPORT_PARITY_PLAN.md` (status matrix updates, evidence logging requirements, prohibition on reordering tasks, etc.).
   - Call out any platform-specific constraints discovered during this phase.

6. Next Steps Checklist
   - Specify exactly which step number in `docs/SPIKE_TRANSPORT_PARITY_PLAN.md` the next operator should start at.
   - List outstanding TODOs, risks, or verification tasks that must happen first.
   - Confirm that the Phase Status Matrix was updated (provide cell details) and the repository state (pushed/stashed).

7. Contact / Follow-up
   - State who to contact (or note “self” if you will resume later).
   - Include any timing commitments or gating approvals still required.

Make sure the resulting handoff reads like an operations runbook entry: concise headers, bullet lists where useful, no missing context. Assume the receiver cannot infer anything that is not explicitly written.
```
