# ADR-000X — <Title>

## Status
- Status: Draft | Accepted
- Date (UTC): <YYYY-MM-DD>
- Owner(s): <names/roles>

## Scope
- Feature directory: `docs/project_management/packs/<bucket>/<feature>/`
  - For Status=Draft: use `docs/project_management/packs/draft/<feature>/`
  - For Status=Accepted: use `docs/project_management/packs/active/<feature>/`
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Related Docs (links only)

List expected paths for traceability. Do not create or update these files as part of ADR authoring; they may not exist yet (especially for Status: Draft).
- Plan: `docs/project_management/packs/<bucket>/<feature>/plan.md`
- Tasks: `docs/project_management/packs/<bucket>/<feature>/tasks.json`
- Spec manifest: `docs/project_management/packs/<bucket>/<feature>/spec_manifest.md`
- Specs: <list spec paths>
- Contract (if present): `docs/project_management/packs/<bucket>/<feature>/contract.md`
- Decision Register: `docs/project_management/packs/<bucket>/<feature>/decision_register.md` (if required)
- Impact Map: `docs/project_management/packs/<bucket>/<feature>/impact_map.md` (if required)
- Manual Playbook: `docs/project_management/packs/<bucket>/<feature>/manual_testing_playbook.md` (if required)

## Executive Summary (Operator)

ADR_BODY_SHA256: <run `make adr-fix ADR=<this-file>` after drafting>

### Changes (operator-facing)
- <short change title>
  - Existing: <existing behavior>
  - New: <new behavior>
  - Why: <why this change>
  - Links:
    - `<path>#L<line>` (best-effort; update line links when ADR changes)

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
  - Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md` (unless explicitly overridden here)
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
- Sequencing entry: `docs/project_management/packs/sequencing.json` → `<sprint id>`
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
- <manual playbook references / steps>

### Manual playbook (if required)
- Manual playbook: `docs/project_management/packs/<bucket>/<feature>/manual_testing_playbook.md`

### Smoke scripts (if required)
- Linux: `docs/project_management/packs/<bucket>/<feature>/smoke/linux-smoke.sh`
- macOS: `docs/project_management/packs/<bucket>/<feature>/smoke/macos-smoke.sh`
- Windows: `docs/project_management/packs/<bucket>/<feature>/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- <policy>

## Decision Summary
- Decision Register entries (if applicable):
  - `docs/project_management/packs/<bucket>/<feature>/decision_register.md`:
    - DR-0001, DR-0002, …
- Options (required; at least two):
  - A) <option>
  - B) <option>
- Selection:
  - Chosen: A | B
  - Rationale: <why>
