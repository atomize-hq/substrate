# Remediation Log - World Disabled Diagnostics

## Open remediations

- None at extraction time.
- Risks and unknowns are captured in seam briefs and become canonical remediation entries only after a gate or review produces a concrete finding.

Rules:

- Use canonical YAML blocks for remediation entries.
- Use seam ownership only. Do not emit `WS-*` owners.
- For `severity: blocking`, `blocked_targets` must not be empty.
- For `severity: material` or `follow_up`, use `blocked_targets: []` unless a concrete blocked transition also applies.

## Resolved remediations

- None.
