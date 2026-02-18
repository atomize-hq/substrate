# Planning Workflow Overview (ADR → Planning Pack → Quality Gate → Triads)

This diagram shows the intended handoff flow:
- ADR creation (after brainstorming converges)
- Spec determination (ADR → required spec set + ownership map)
- Operator review via `## Executive Summary (Operator)` + drift guard
- Planning Pack creation (execution-ready specs/tasks/prompts)
- Platform parity plan (required when cross-platform)
- Third-party Planning Quality Gate (accept/flag)
- If flagged: planning-doc remediation (docs-only) and re-review
- Execution triads (code/test/integ)
- Cross-platform smoke via `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`
- Operational defaults:
  - Prefer smaller triads/slices (avoid “grab bag” slices).
  - Prefer `PLATFORM=behavior` for smoke (one dispatch runs only required behavior platforms).
  - Dispatch from the orchestration/task ref (not `main`/`testing`); workflows must be registered on `main` to be dispatchable.

```mermaid
flowchart TD
  A[Brainstorming session]
  B["ADR authoring agent reads: docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md"]
  C[ADR draft created]
  SM["Spec determination agent reads: docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md"]
  SMD[spec_manifest.md created]
  IM["Impact map agent reads: docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md"]
  IMD[impact_map.md created]
  CP["CI checkpoint planning agent reads: docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md"]
  CPD[ci_checkpoint_plan.md created]
  D{ADR accepted}
  D_NO[ADR not accepted]
  D_YES[ADR accepted]

  P["Planning agent reads: docs/project_management/standards/PLANNING_README.md"]
  PACK[Planning Pack created under docs/project_management/packs/active/**FEATURE_NAME**/ (legacy: docs/project_management/next/**FEATURE_NAME**/)]
  PP["Platform parity plan (cross-platform; required when meta.cross_platform=true)\n- meta.schema_version\n  - v2/v3: platform-fix tasks per slice\n  - v4: boundary-only platform-fix\n- v3+: meta.automation.enabled=true\n- v4: meta.checkpoint_boundaries (slice ids)\n- meta.behavior_platforms_required\n- meta.ci_parity_platforms_required (legacy: meta.platforms_required)\n- meta.wsl_required + meta.wsl_task_mode (if needed)\n- v2/v3 model: X-integ-core / X-integ-<platform> / X-integ (per slice)\n- v4 model: normal slices use X-integ; boundary slices use B-integ-core / B-integ-<platform> / B-integ\n- ci_checkpoint_plan.md (bounded CI checkpoints between groups of triads)"]

  Q["Quality gate reviewer reads: docs/project_management/system/prompts/planning/quality_gate_reviewer.md"]
  LINT["Run mechanical checks: docs/project_management/standards/PLANNING_LINT_CHECKLIST.md"]
  REPORT["Write: docs/project_management/packs/active/<feature>/quality_gate_report.md (legacy: docs/project_management/next/<feature>/quality_gate_report.md) using docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md"]
  GATE{RECOMMENDATION ACCEPT}
  REMEDIATE["Remediation agent reads: docs/project_management/system/prompts/planning/quality_gate_remediation.md\nFix Planning Pack docs only"]
  PREFLIGHT["Execution preflight gate (feature start)\n- docs/project_management/standards/EXECUTION_PREFLIGHT_GATE_STANDARD.md\n- execution_preflight_report.md"]
  GATE_YES["Execution triads start: docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md"]

  A --> B
  B --> C
  C --> SM --> SMD --> IM --> IMD --> CP --> CPD --> D
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
