# ADR Standard and Template

This file defines:
- the required structure and authoring rules for ADRs in this repo, and
- a copy/paste template for creating new ADRs.

ADRs are used to record the **shape of a body of work** once initial discovery converges. ADRs are written **before** execution triads begin.

## ADR scope

An ADR must:
- define the user-facing contract (CLI/config/exit codes/path semantics) at the correct level of detail,
- define non-goals and boundaries to prevent scope creep,
- define the architecture shape and where code changes will land (by crate/module),
- define sequencing/dependencies with adjacent work,
- define required validation artifacts (manual playbook + smoke scripts where applicable).

An ADR must not:
- be a substitute for the Decision Register (fine-grained A/B decisions live in `decision_register.md`),
- contain open questions or optional contracts,
- require migrations/backwards compatibility unless explicitly mandated (greenfield default).

## Relationship to Decision Register

Use this rule:
- ADR: **one** document that describes the body of work and the end-to-end contract.
- Decision Register: **all** architectural decisions recorded as exactly two options (A/B) with explicit tradeoffs and one selection.

If the work has more than one meaningful decision, the Planning Pack must include:
- `docs/project_management/next/<feature>/decision_register.md`

The ADR must link to that decision register and must not duplicate decision entries.

## Required ADR format

Every ADR must include all sections below. Every behavior statement must be singular and testable.

### Required header
- ADR ID and title
- Status: `Draft` or `Accepted`
- Date (UTC)
- Owner(s)
- Feature directory path under `docs/project_management/next/<feature>/`
- Related documents (links to plan/specs/decision register/sequencing)

### Required sections

1) Problem / Context
- Why this body of work exists now.
- What breaks or is blocked without it.

2) Goals (explicit)
- A finite list of concrete outcomes.

3) Non-Goals (explicit)
- A finite list of explicitly out-of-scope items.

4) User Contract (authoritative)
- CLI commands, flags, default behavior, and error/exit-code taxonomy.
- Config files: filenames, locations, precedence, and schema constraints.
- Platform notes: required parity guarantees and platform-specific guards.

5) Architecture Shape
- Components affected (crates/modules/scripts) and responsibilities.
- End-to-end flow: inputs → derived state → actions → outputs.

6) Sequencing / Dependencies
- Alignment to `docs/project_management/next/sequencing.json`.
- Dependencies on other sprints/triads (must reference integration task IDs).

7) Security / Safety Posture
- Fail-closed vs degrade behavior, and what triggers each.
- Protected paths and invariants (if FS operations exist).
- Observability requirements (trace/log fields if applicable).

8) Validation Plan (authoritative)
- Unit/integration test expectations (high-level).
- Manual playbook requirement and scope.
- Smoke scripts requirement (platform-specific scripts in the feature directory).

9) Rollout / Backwards Compatibility
- Default: greenfield breaking is allowed.
- If backwards compatibility is required, explicitly state the exact compat policy and the end condition for removing compat.

10) Decision Summary
- Link to `decision_register.md` entries used by this ADR.
- If no decision register exists, the ADR must explain why the body of work has no architectural decisions beyond trivial implementation details.

## File naming and placement

- ADRs for upcoming work live under `docs/project_management/next/`:
  - `docs/project_management/next/ADR-000X-<kebab-title>.md` (cross-cutting ADR), and/or
  - `docs/project_management/next/<feature>/ADR-000X-<kebab-title>.md` (feature-local ADR).
- If both exist, the cross-cutting ADR is authoritative and the feature-local ADR must defer to it.

## ADR template (copy/paste)

```md
# ADR-000X — <Title>

## Status
- Status: Draft | Accepted
- Date (UTC): <YYYY-MM-DD>
- Owner(s): <names/roles>

## Scope
- Feature directory: `docs/project_management/next/<feature>/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`

## Related Docs
- Plan: `docs/project_management/next/<feature>/plan.md`
- Tasks: `docs/project_management/next/<feature>/tasks.json`
- Specs: <list spec paths>
- Decision Register: `docs/project_management/next/<feature>/decision_register.md` (if required)
- Integration Map: `docs/project_management/next/<feature>/integration_map.md` (if required)
- Manual Playbook: `docs/project_management/next/<feature>/manual_testing_playbook.md` (if required)

## Problem / Context
- <why this work exists now>

## Goals
- <explicit goal>
- <explicit goal>

## Non-Goals
- <explicit non-goal>
- <explicit non-goal>

## User Contract (Authoritative)

### CLI
- Commands:
  - `<command>`: <behavior, defaults, exit codes>
- Exit codes:
  - `0`: <meaning>
  - `2`: <meaning>
  - `3`: <meaning>
  - `4`: <meaning>
  - `5`: <meaning>

### Config
- Files and locations (precedence):
  1. `<path A>`: <meaning>
  2. `<path B>`: <meaning>
- Schema:
  - <constraints>

### Platform guarantees
- Linux: <guarantees and guards>
- macOS: <guarantees and guards>
- Windows: <guarantees and guards>

## Architecture Shape
- Components:
  - `<crate/module>`: <responsibility>
  - `<script>`: <responsibility>
- End-to-end flow:
  - Inputs: <list>
  - Derived state: <list>
  - Actions: <list>
  - Outputs: <list>

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/next/sequencing.json` → `<sprint id>`
- Prerequisite integration task IDs:
  - `<X-integ>` before `<Y-code>`

## Security / Safety Posture
- Fail-closed rules:
  - <explicit>
- Protected paths/invariants:
  - <explicit>

## Validation Plan (Authoritative)

### Tests
- Unit tests: <where and what>
- Integration tests: <where and what>

### Manual validation
- Manual playbook: `docs/project_management/next/<feature>/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/next/<feature>/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/<feature>/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/<feature>/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none

## Decision Summary
- Decision Register entries:
  - `docs/project_management/next/<feature>/decision_register.md`:
    - DR-0001, DR-0002, …
```

