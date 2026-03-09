# ADR Standard and Template

This file defines:
- the required structure and authoring rules for ADRs in this repo, and
- the contract for the canonical ADR template.

Canonical ADR template (single source of truth; do not duplicate it in this standard):
- `docs/project_management/system/templates/adr/ADR_TEMPLATE.md`

ADRs are used to record the **shape of a body of work** once initial discovery converges. ADRs are written **before** execution triads begin.

After drafting an ADR, you must derive the required spec set (and ownership map) before producing the Planning Pack:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
Then produce the impact map (touch set + cascading implications + cross-queue conflicts):
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`

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
- `docs/project_management/packs/<bucket>/<feature>/decision_register.md`

Bucket rule:
- Draft ADRs should reference `docs/project_management/packs/draft/<feature>/`.
- Accepted ADRs must reference `docs/project_management/packs/active/<feature>/`.

The ADR must link to that decision register and must not duplicate decision entries.

If the Planning Pack uses `contract.md` (see `docs/project_management/system/standards/shared/CONTRACT_SURFACE_STANDARD.md`), ADR/specs/playbooks must link to it and must not contradict it.

## Accepted ADR requirements (non-negotiable)

If an ADR status is `Accepted`, it must include:
- the exact feature directory path(s) under `docs/project_management/packs/active/<feature>/`, and
- the intended branch name(s) (e.g., `feat/<feature>`).

## Executive Summary (Operator) (required)

Every ADR must include an operator-facing summary section:
- Header must be exactly: `## Executive Summary (Operator)`
- Must contain an `ADR_BODY_SHA256: <64-hex>` line (drift guard)
- Must describe behavior changes using:
  - `Existing:` (what the operator experiences today)
  - `New:` (what the operator will experience after this ADR)
  - `Why:` (why this change exists / what it prevents/enables)
  - `Links:` (deep links into the ADR/specs/code/docs; prefer `path#L<line>` when possible)

Drift is enforced mechanically via:
- `make adr-check ADR=<adr.md>` (and `make adr-fix ADR=<adr.md>` to update the hash)

See also:
- `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md`

## Required ADR format

Every ADR must include all sections below. Every behavior statement must be singular and testable.

### Required header
- ADR ID and title
- Status: `Draft` or `Accepted`
- Date (UTC)
- Owner(s)
- Feature directory path under `docs/project_management/packs/<bucket>/<feature>/`
  - For Status=Draft: use `draft`
  - For Status=Accepted: use `active`
- Related documents (links to plan/specs/decision register/sequencing)

### Required sections

0) Executive Summary (Operator)
- Existing → New → Why bullets (operator-facing).
- Must include `ADR_BODY_SHA256` and deep links.

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
- Alignment to `docs/project_management/packs/sequencing.json`.
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

Preferred ADR registry location (canonical):
- `docs/project_management/adrs/`
  - Draft: `docs/project_management/adrs/draft/ADR-000X-<kebab-title>.md`
  - Queued/unimplemented: `docs/project_management/adrs/queued/ADR-000X-<kebab-title>.md`
  - Implemented: `docs/project_management/adrs/implemented/ADR-000X-<kebab-title>.md`
  - Superseded: `docs/project_management/adrs/superseded/ADR-000X-<kebab-title>.md`

Legacy locations (still supported):
- `docs/project_management/adrs/<bucket>/ADR-000X-<kebab-title>.md` (canonical ADR registry path).

If both a cross-cutting ADR and a feature-local ADR exist, the cross-cutting ADR is authoritative and the feature-local ADR must defer to it.

## ADR template (canonical)

Copy/paste the canonical template from:
- `docs/project_management/system/templates/adr/ADR_TEMPLATE.md`
