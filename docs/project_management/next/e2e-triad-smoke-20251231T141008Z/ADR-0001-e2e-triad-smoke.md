# ADR-0001: E2E triad automation smoke

## Executive Summary (Operator)

ADR_BODY_SHA256: 0c7a6772991ea84da3dcc092d3f84b03aba1846c054e060f01c81b24b52e3c78

ADR_BODY_SHA256: placeholder

- Existing: No single scripted end-to-end run proves planning + triad automation + CI smoke wiring works together.
- New: Add a temporary Planning Pack + triad execution run that exercises worktrees, Codex headless launch, CI smoke dispatch, and final FF merge-back.
- Why: Catch workflow/automation bugs early with a deterministic, repeatable smoke scenario.

## Decision
- Use an automation-enabled Planning Pack (tasks.json schema v3 + meta.automation.enabled=true).
- Use the cross-platform integration model (integ-core + platform-fix + final aggregator).

## Notes
- This feature is intended for workflow validation and can be removed after debugging.
