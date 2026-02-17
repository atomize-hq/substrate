# Final alignment pass prompt template (multi-track)

```md
You are performing a final “zero-misalignment” pass across the queued planning docs for:
- <list directories>

Do not write production code. Fix docs only.
Enforce the Planning Rubric (lint-like rules). Output must contain 0 unresolved misalignments.

Deliverables:
- docs/project_management/next/final_alignment_report.md
- missing manual_testing_playbook.md files for any triads that affect UX/provisioning
- updated tasks.json dependencies and/or sequencing.json as required (final, no placeholders)
```
