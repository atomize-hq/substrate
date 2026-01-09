# Planning Workflow Overview (ADR → Planning Pack → Quality Gate → Triads)

This diagram shows the intended handoff flow:
- ADR creation (after brainstorming converges)
- Operator review via `## Executive Summary (Operator)` + drift guard
- Planning Pack creation (execution-ready specs/tasks/prompts)
- Platform parity plan (required when cross-platform)
- Third-party Planning Quality Gate (accept/flag)
- If flagged: planning-doc remediation (docs-only) and re-review
- Execution triads (code/test/integ)
- Cross-platform smoke via `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`
- Operational defaults:
  - Prefer smaller triads/slices (avoid “grab bag” slices).
  - Prefer stable GitHub Actions dispatch refs (`testing`/`main`) + `PLATFORM=behavior` for smoke.

```mermaid
flowchart TD
  A[Brainstorming session]
  B["ADR authoring agent reads: docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md"]
  C[ADR draft created]
  D{ADR accepted}
  D_NO[ADR not accepted]
  D_YES[ADR accepted]

  P["Planning agent reads: docs/project_management/standards/PLANNING_README.md"]
  PACK[Planning Pack created under docs/project_management/next/**FEATURE_NAME**/]
  PP["Platform parity plan (schema v2/v3)\n- meta.schema_version\n- v2: cross-platform model fields\n- v3: meta.automation.enabled=true\n- meta.behavior_platforms_required\n- meta.ci_parity_platforms_required (legacy: meta.platforms_required)\n- meta.wsl_required + meta.wsl_task_mode (if needed)\n- X-integ-core / X-integ-<platform> / X-integ"]

  Q["Quality gate reviewer reads: docs/project_management/standards/PLANNING_QUALITY_GATE_PROMPT.md"]
  LINT["Run mechanical checks: docs/project_management/standards/PLANNING_LINT_CHECKLIST.md"]
  REPORT["Write: docs/project_management/next/<feature>/quality_gate_report.md using docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md"]
  GATE{RECOMMENDATION ACCEPT}
  REMEDIATE["Remediation agent reads: docs/project_management/standards/PLANNING_QUALITY_GATE_REMEDIATION_PROMPT.md\nFix Planning Pack docs only"]
  PREFLIGHT["Execution preflight gate (feature start)\n- docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md\n- execution_preflight_report.md"]
  GATE_YES["Execution triads start: docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md"]

  A --> B
  B --> C
  C --> D
  D --> D_NO
  D --> D_YES
  D_NO --> B

  D_YES --> P
  P --> PACK
  PACK --> PP
  PP --> Q
  Q --> LINT
  LINT --> REPORT
  REPORT --> GATE
  GATE -- "NO" --> REMEDIATE
  GATE -- "YES" --> PREFLIGHT --> GATE_YES
  REMEDIATE --> P
```
