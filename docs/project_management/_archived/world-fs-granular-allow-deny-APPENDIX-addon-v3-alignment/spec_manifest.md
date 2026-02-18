# world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment — spec manifest

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment`
- ADR(s):
  - `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
  - Appendix authoritative inputs:
    - `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/contract.md`
    - `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md`
    - `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/PROTOCOL.md`
    - `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/ENV.md`

## Required spec documents (authoritative)

List the exact spec documents that must exist under `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/`.

Each entry must include:
- path
- what surfaces it owns (authoritative)
- what it links to (non-authoritative)

Spec templates:
- `docs/project_management/standards/templates/spec/`

- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/spec_manifest.md` — spec selection + ownership map (this file)
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/impact_map.md` — touch set + cascading implications + cross-queue conflicts
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/plan.md` — execution runbook + sequencing overview
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/tasks.json` — triad task graph + acceptance criteria
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/contract.md` — add-on contract deltas + drift closure definition
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/decision_register.md` — explicit decisions where implementation ambiguity exists
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA0-spec.md` — effective policy display output contract enforcement (Appendix A.6)
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA1-spec.md` — snapshot schema/protocol migration to PolicySnapshotV3
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA2-spec.md` — downstream surface + docs alignment (doctor/health/trace/env docs)

## Coverage matrix (surface → authoritative doc)

Every surface that the ADR touches must appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| Effective policy display (`substrate policy show`) | `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/contract.md` (§1.3) | output shape + explicit empty deny lists |
| Policy patch schema (V3) + legacy-key rejection | `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md` (§1.2) | invalid legacy keys; invariants; defaults |
| Snapshot schema (PolicySnapshotV3) | `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md` (§2) | `schema_version=3`; unknown fields rejected; canonicalization |
| Snapshot transport (HTTP + WS) | `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/PROTOCOL.md` | request shapes; error shapes; no compat lockstep |
| Env var contract (exported state) | `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/ENV.md` | `SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING`; deletion of `SUBSTRATE_WORLD_REQUIRE_WORLD` |
| Exit codes | `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/contract.md` (§2) | exit `2` for actionable policy/config errors |
| Operator docs (canonical references) | `docs/CONFIGURATION.md`, `docs/WORLD.md`, `docs/reference/env/contract.md` | must not present V2 keys as operator-facing |

## Add-on ownership (implementation alignment)
This add-on pack owns:
- the engineering plan to make the code match the Appendix authoritative docs above, and
- tests/gates that enforce the Appendix output/protocol contracts.

## Determinism checklist (must be satisfied before quality gate)

For every selected spec document, confirm it explicitly defines:
- Inputs (all) + precedence order (if multiple inputs exist)
- Defaults (all) + absence semantics
- Data model (types/constraints) for every serialized boundary
- Error model (exit codes, error messages where applicable) and failure posture
- Ordering/atomicity/concurrency rules (if any)
- Security/redaction invariants (if any)
- Platform guarantees (Linux/macOS/Windows/WSL as applicable)
