# Planning Workflow Overview (ADR → Planning Pack → Quality Gate → Triads)

This diagram shows the intended handoff flow:
- ADR creation (after brainstorming converges)
- Planning Pack creation (execution-ready specs/tasks/prompts)
- Third-party Planning Quality Gate (accept/flag)
- Execution triads (code/test/integ)

```mermaid
flowchart TD
  A[Brainstorming / Noodling Session] --> B[ADR Authoring Agent<br/>Read: docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md];
  B --> C[ADR Draft Created];
  C --> D{ADR Ready?};
  D -->|No| E[Revise ADR (repeat)];
  E --> C;
  D -->|Yes| F[ADR Accepted];

  F --> G[Planning Agent<br/>Read: docs/project_management/standards/PLANNING_README.md];
  G --> H[Planning Pack Created<br/>docs/project_management/next/&lt;feature&gt;/<br/>plan.md, tasks.json, session_log.md, specs, kickoff_prompts/,<br/>+ decision_register/integration_map/manual_playbook/smoke/ if required];

  H --> I[Quality Gate Reviewer (3rd party)<br/>Read: docs/project_management/standards/PLANNING_QUALITY_GATE_PROMPT.md];
  I --> J[Run Mechanical Checks<br/>docs/project_management/standards/PLANNING_LINT_CHECKLIST.md];
  J --> K[Write Gate Report<br/>docs/project_management/next/&lt;feature&gt;/quality_gate_report.md<br/>(template: docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md)];

  K --> L{RECOMMENDATION: ACCEPT?};
  L -->|No| M[Fix Planning Pack (repeat)];
  M --> H;
  L -->|Yes| N[Execution Triads Start<br/>Read: docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md];
```
