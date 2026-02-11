# Decision Register — world-fs-granular-allow-deny-appendix

This document records A/B decisions for Appendix A + B implementation planning.

Authoring standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

### DR-AX-0001 — Replace isolation enum with intent boolean `host_visible`

**Decision owner(s):** Substrate maintainers  
**Date:** 2026-02-06  
**Status:** Accepted  
**Related docs:**
- `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` (Appendix A)
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/contract.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/WFGADAX0-spec.md`

**Problem / Context**
- Operators need an intent-driven policy key that communicates “host paths are nameable” vs “host paths are not nameable” without translating an implementation encoding (`isolation=workspace|full`).
- Appendix A is a breaking schema update; this decision selects the V3 key that encodes the former `workspace|full` meaning.

**Option A — Keep `world_fs.isolation=workspace|full` (V2)**
- **Pros:** Preserves existing key name and mental model for current operators.
- **Cons:** Requires operators to map “workspace/full” to the intended outcome; carries forward implementation-leaky language.
- **Cascading implications:** Appendix A remains a partial rename; schema + docs keep the enum surface and its defaults.
- **Risks:** Operators misunderstand `workspace|full` as a platform knob instead of a host-path-visibility guarantee.
- **Unlocks:** Minimal key churn for existing profiles.
- **Quick wins / low-hanging fruit:** Reduced renaming work inside broker/shell models.

**Option B — Replace with `world_fs.host_visible=true|false` (V3)**
- **Pros:** Encodes operator intent directly (host path visibility); supports a schema where each key carries one behavior axis.
- **Cons:** Breaking rename; requires updating schema parsing, snapshot shape, and documentation.
- **Cascading implications:** Requires replaced-key hard errors and a new snapshot schema version (`PolicySnapshotV3`).
- **Risks:** Rollout requires policy updates; any missed references create deterministic hard errors.
- **Unlocks:** Allows routing (`fail_closed.routing`) and caging (`caged_required`) to remain orthogonal to host visibility.
- **Quick wins / low-hanging fruit:** Cleaner defaults and validation rules in the V3 schema.

**Recommendation**
- **Selected:** Option B — Replace with `world_fs.host_visible=true|false` (V3)
- **Rationale (crisp):** Appendix A is intent-driven; the V3 schema encodes host visibility explicitly and avoids retaining implementation-leaky enums.

**Follow-up tasks (explicit)**
- Implement V3 patch parsing, defaults, and replaced-key hard errors: `WFGADAX0-code`, `WFGADAX0-test`, `WFGADAX0-integ`.
- Update operator-facing contract surfaces to use `host_visible`: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/contract.md`.

### DR-AX-0002 — Routing fail-closed placement

**Decision owner(s):** Substrate maintainers  
**Date:** 2026-02-06  
**Status:** Accepted  
**Related docs:**
- `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` (Appendix B.2)
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/contract.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/ENV.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/WFGADAX1-spec.md`

**Problem / Context**
- Appendix B introduces routing fail-closed semantics that are independent of host visibility (`host_visible`) and policy enforcement posture.
- The policy surface must express “fail closed when routing fails” without reusing a legacy key that conflates meaning.

**Option A — Keep legacy `world_fs.require_world=true|false`**
- **Pros:** Preserves an existing boolean surface and implementation shape.
- **Cons:** Conflates routing fallback with other “require world” semantics and conflicts with Appendix A’s intent-driven model.
- **Cascading implications:** Exported state and telemetry remain tied to `require_world`; renamed semantics remain implicit.
- **Risks:** Operators misinterpret the meaning under V3 and assume it implies host visibility or deny enforcement.
- **Unlocks:** Avoids adding a new nested key in the schema.
- **Quick wins / low-hanging fruit:** Fewer rename edits across crates.

**Option B — Introduce `world_fs.fail_closed.routing=true|false`**
- **Pros:** Encodes routing fallback semantics precisely; orthogonal to host visibility and caging.
- **Cons:** Requires env var rename and explicit failure taxonomy wiring in routing code paths.
- **Cascading implications:** Requires exported state var `SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING=1|0` and deletion of `SUBSTRATE_WORLD_REQUIRE_WORLD`.
- **Risks:** Incorrect failure mapping can produce silent host fallback or wrong exit codes; requires explicit test coverage.
- **Unlocks:** Deterministic policy/config incompatibility behavior when world is disabled by operator intent.
- **Quick wins / low-hanging fruit:** Clear operator-facing contract for fail-closed routing.

**Recommendation**
- **Selected:** Option B — Introduce `world_fs.fail_closed.routing=true|false`
- **Rationale (crisp):** Appendix B defines routing fail-closed as an explicit operator intent; a dedicated key preserves orthogonality and enables deterministic failure taxonomy.

**Follow-up tasks (explicit)**
- Implement routing fail-closed behavior and exported state env var rename: `WFGADAX1-code`, `WFGADAX1-test`, `WFGADAX1-integ-core`.
- Validate runtime failure mapping and cross-platform compile parity at CP1 boundary: `CP1-ci-checkpoint`, `WFGADAX1-integ-linux`, `WFGADAX1-integ-macos`, `WFGADAX1-integ-windows`, `WFGADAX1-integ`.

### DR-AX-0003 — Deny enforcement posture naming

**Decision owner(s):** Substrate maintainers  
**Date:** 2026-02-06  
**Status:** Accepted  
**Related docs:**
- `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` (Appendix A)
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/SECURITY.md`

**Problem / Context**
- Appendix A replaces the V2 enforcement naming (`strict|best_effort`) and introduces a third posture that encodes preference for strict prerequisites without failing.
- The key name and enum values must encode operator intent and match the security contract in Appendix A/B.

**Option A — Keep `strict|best_effort` naming**
- **Pros:** Matches baseline V2 naming and existing code paths.
- **Cons:** Does not encode preference semantics; `best_effort` is ambiguous between “prefer strict” and “weak”.
- **Cascading implications:** Appendix A cannot express preference fallback without additional keys or rules.
- **Risks:** Operators interpret `best_effort` as “safe enough” when it is not a boundary under adversarial workloads.
- **Unlocks:** Reduces changes in model enum names.
- **Quick wins / low-hanging fruit:** Minimal documentation churn.

**Option B — Use `strict|prefer_strict|weak`**
- **Pros:** Encodes three distinct intents: strict boundary, strict when available, and non-boundary weak mode.
- **Cons:** Larger surface; requires explicit documentation and validation wiring.
- **Cascading implications:** Schema + security doc must define posture meaning and failure taxonomy when strict prerequisites are missing.
- **Risks:** If `weak` is selected unintentionally, operator expectations can be violated; mitigated by explicit enum naming.
- **Unlocks:** Aligns the schema surface to Appendix A/B semantics without extra keys.
- **Quick wins / low-hanging fruit:** Clear operator choice; predictable downgrade behavior only in `prefer_strict`.

**Recommendation**
- **Selected:** Option B — Use `strict|prefer_strict|weak`
- **Rationale (crisp):** Appendix A requires a preference posture distinct from weak mode; the three-value enum captures the exact intent set without additional keys.

**Follow-up tasks (explicit)**
- Implement V3 schema parsing and validation for `deny_enforcement`: `WFGADAX0-code`, `WFGADAX0-test`, `WFGADAX0-integ`.
- Validate security semantics and docs alignment: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/SECURITY.md`.

### DR-AX-0004 — Caging requirement is a policy key

**Decision owner(s):** Substrate maintainers  
**Date:** 2026-02-06  
**Status:** Accepted  
**Related docs:**
- `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` (Appendix B.3)
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/contract.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/WFGADAX2-spec.md`

**Problem / Context**
- Appendix B introduces caging as a policy-level safety boundary for specific scopes; config-only caging cannot represent mandatory “must be caged” constraints.
- The enforcement posture must be deterministic and fail before execution when policy requires caging and config disables it.

**Option A — Keep caging config-only (`world.caged=true|false`)**
- **Pros:** Avoids adding a policy boundary key.
- **Cons:** Cannot express “mandatory” caging for a policy scope; a config toggle can weaken a policy boundary.
- **Cascading implications:** Policy cannot require caging; Appendix B compatibility rules become non-expressible.
- **Risks:** Users run uncaged in a scope that intends caging as a safety boundary.
- **Unlocks:** No new policy key.
- **Quick wins / low-hanging fruit:** No new schema wiring.

**Option B — Introduce `world_fs.caged_required=true|false`**
- **Pros:** Encodes a mandatory safety boundary at the policy layer; enables deterministic compatibility checks.
- **Cons:** Requires explicit policy/config compatibility enforcement (hard error when config requests uncaged).
- **Cascading implications:** REPL start and command execution paths must validate: `world.caged=true` and `world.anchor_mode!=follow-cwd` when caging is required.
- **Risks:** Incorrect enforcement creates a silent weakening of a safety boundary; mitigated with explicit tests and manual playbook coverage.
- **Unlocks:** Enables workspace-scoped policy boundaries tied to `entered_cwd`.
- **Quick wins / low-hanging fruit:** Clear operator intent; consistent failure taxonomy (exit 2 for incompatibility).

**Recommendation**
- **Selected:** Option B — Introduce `world_fs.caged_required=true|false`
- **Rationale (crisp):** Appendix B defines caging as a policy boundary; placing it in policy prevents config from weakening the boundary and enables deterministic compatibility enforcement.

**Follow-up tasks (explicit)**
- Implement caging-required compatibility enforcement and cage-root derivation: `WFGADAX2-code`, `WFGADAX2-test`, `WFGADAX2-integ`.
- Ensure the V3 schema includes `caged_required`: `WFGADAX0-code`, `WFGADAX0-test`, `WFGADAX0-integ`.
- Validate with manual playbook cases (caging required): `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/manual_testing_playbook.md`.

### DR-AX-0005 — REPL exit behavior includes explicit transparency note

**Decision owner(s):** Substrate maintainers  
**Date:** 2026-02-06  
**Status:** Accepted  
**Related docs:**
- `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` (Appendix B.4)
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/contract.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/WFGADAX3-spec.md`

**Problem / Context**
- Appendix B defines an operator-facing transparency requirement: when the REPL changes the in-world cwd relative to the entered host cwd, the exit behavior must be observable and deterministic.
- The contract must remain safe when `repl.exit_cwd=last_world` cannot be applied (non-representable path or missing directory).

**Option A — Silent exit (no note)**
- **Pros:** Minimal output surface.
- **Cons:** Hides a user-visible cwd drift between `entered_cwd` and the last world cwd; exit behavior is not auditable.
- **Cascading implications:** Shell integration cannot safely apply a cwd change because there is no stable output contract to consume.
- **Risks:** Users assume their host cwd matches the last world cwd and run follow-on commands in the wrong directory.
- **Unlocks:** Avoids introducing a note format contract.
- **Quick wins / low-hanging fruit:** No new string surface.

**Option B — Exit note when `world_cwd != entered_cwd`**
- **Pros:** Deterministic transparency; provides a stable hook for shell integration wrappers.
- **Cons:** Adds an output surface that must remain stable.
- **Cascading implications:** Contract must specify the note line format and when it is printed, plus fallback-note behavior for `repl.exit_cwd=last_world`.
- **Risks:** Output drift breaks wrappers; mitigated by an explicit contract and targeted tests.
- **Unlocks:** Enables `repl.exit_cwd=last_world` to be validated without requiring an active shell integration.
- **Quick wins / low-hanging fruit:** Clear operator UX with minimal complexity.

**Recommendation**
- **Selected:** Option B — Exit note when `world_cwd != entered_cwd`
- **Rationale (crisp):** A stable note line makes exit behavior observable and supports `repl.exit_cwd` workflows without hidden cwd drift.

**Follow-up tasks (explicit)**
- Implement exit note and `repl.exit_cwd` behavior (including fallback-note behavior): `WFGADAX3-code`, `WFGADAX3-test`, `WFGADAX3-integ-core`, `WFGADAX3-integ`.
- Validate manual cases (exit note and `repl.exit_cwd=last_world`): `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX/manual_testing_playbook.md`.
