# world-sync — spec manifest

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/active/world-sync`
- ADR(s) / upstream contracts (inputs; not owned by this feature pack):
  - `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
  - `docs/project_management/adrs/implemented/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`
  - `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`
  - `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
  - `docs/project_management/future/INTERNAL_GIT.md` (PRD input; not an ADR)

## Required spec documents (authoritative)

List the exact spec documents that must exist under `docs/project_management/packs/active/world-sync/`.

Each entry must include:
- path
- what surfaces it owns (authoritative)
- what it links to (non-authoritative)

Spec templates:
- `docs/project_management/standards/templates/spec/`

- `docs/project_management/packs/active/world-sync/spec_manifest.md` — spec selection + ownership map (this file)
- `docs/project_management/packs/active/world-sync/impact_map.md` — touch set + cascading implications + cross-queue conflicts
- `docs/project_management/packs/active/world-sync/plan.md` — execution runbook + sequencing overview
- `docs/project_management/packs/active/world-sync/tasks.json` — triad task graph + acceptance criteria
- `docs/project_management/packs/active/world-sync/contract.md` — CLI/config/exit codes/paths contract (single authoritative contract file)
- `docs/project_management/packs/active/world-sync/filesystem-semantics-spec.md` — diff/path filtering, protected paths, size guards, atomicity rules
- `docs/project_management/packs/active/world-sync/platform-parity-spec.md` — per-platform support/unsupported contract + validation evidence requirements
- `docs/project_management/packs/active/world-sync/internal-git-spec.md` — internal git directory contract, commit/tag naming, checkpoint/rollback semantics
- Slice specs (authoritative per-slice behavior/acceptance; no overlapping contracts):
  - `docs/project_management/packs/active/world-sync/WS0-spec.md`
  - `docs/project_management/packs/active/world-sync/WS1-spec.md`
  - `docs/project_management/packs/active/world-sync/WS2-spec.md`
  - `docs/project_management/packs/active/world-sync/WS3-spec.md`
  - `docs/project_management/packs/active/world-sync/WS4-spec.md`
  - `docs/project_management/packs/active/world-sync/WS5-spec.md`
  - `docs/project_management/packs/active/world-sync/WS6-spec.md`
  - `docs/project_management/packs/active/world-sync/WS7-spec.md`

## Coverage matrix (surface → authoritative doc)

Every surface that the ADR touches must appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| CLI commands/flags/defaults | `docs/project_management/packs/active/world-sync/contract.md` | names, defaults, examples, exit codes |
| Config file paths/precedence/keys | `docs/project_management/packs/active/world-sync/contract.md` | file paths, precedence, key set + defaults |
| Exit code meanings | `docs/project_management/packs/active/world-sync/contract.md` | mapping to taxonomy, per-command meanings |
| Internal git directory + tag/commit contract | `docs/project_management/packs/active/world-sync/internal-git-spec.md` | paths, init rules, commit format, tag ids, restore rules |
| Filesystem semantics (diff interpretation + filtering) | `docs/project_management/packs/active/world-sync/filesystem-semantics-spec.md` | protected paths, exclude matching, size guards, atomicity |
| Platform parity / supported backends | `docs/project_management/packs/active/world-sync/platform-parity-spec.md` | supported/unsupported by platform, required tests, smoke expectations |
| Slice-level behavior deltas | `docs/project_management/packs/active/world-sync/WS*-spec.md` | per-slice scope, exact behavior, errors, acceptance |

## Determinism checklist (must be satisfied before quality gate)

For every selected spec document, confirm it explicitly defines:
- Inputs (all) + precedence order (if multiple inputs exist)
- Defaults (all) + absence semantics
- Data model (types/constraints) for every serialized boundary
- Error model (exit codes, error messages where applicable) and failure posture
- Ordering/atomicity/concurrency rules (if any)
- Security/redaction invariants (if any)
- Platform guarantees (Linux/macOS as applicable)
