# Decision Register — Doctor scope split (host vs world)

This decision register supports `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md`.

## Format note

Each decision entry follows the required template in:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

### DR-0001 — CLI naming for host-scoped doctor

**Decision owner(s):** Substrate CLI maintainers  
**Date:** 2026-01-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md`, `docs/project_management/_archived/doctor_scopes/DS0-spec.md`

**Problem / Context**
- DS0 adds a host-scoped doctor surface and requires a single, unambiguous CLI name that is discoverable and consistent with existing nouns (`world`, `shim`, `health`).

**Option A — `substrate host doctor`**
- **Pros:** Uses an existing top-level noun pattern; separates host readiness from world readiness by command name; supports platform-specific output without overloading `world`.
- **Cons:** Adds a new top-level subcommand; requires docs updates for the command matrix.
- **Cascading implications:** Requires CLI routing changes and updates to any help text and docs that list doctor commands.
- **Risks:** Users search for `world doctor` and miss the host-only command on first try.
- **Unlocks:** A clean future split for `host` scoped commands beyond doctor (consistent UX).
- **Quick wins / low-hanging fruit:** Clear `--help` surface for a single-purpose host readiness check.

**Option B — `substrate world doctor --host`**
- **Pros:** Avoids adding a new top-level subcommand; keeps doctor invocation under the existing `world` namespace.
- **Cons:** Low discoverability; creates an overloaded mental model where “world doctor” includes host-only behavior; increases ambiguity in scripts and operator instructions.
- **Cascading implications:** Adds a new flag surface that must be documented and validated across platforms; increases risk of users running the wrong scope by default.
- **Risks:** Reintroduces the macOS confusion where “world doctor” is interpreted as host doctor.
- **Unlocks:** Keeps the command tree shallower.
- **Quick wins / low-hanging fruit:** Minimal CLI command tree changes.

**Recommendation**
- **Selected:** Option A — `substrate host doctor`
- **Rationale (crisp):** A dedicated `host` namespace removes scope ambiguity and matches existing Substrate noun patterns.

**Follow-up tasks (explicit)**
- `DS0-code`: implement `substrate host doctor` CLI routing and behavior.
- `DS0-test`: add CLI wiring + JSON contract tests for `substrate host doctor`.
- `DS0-integ-core`: validate the CLI surface and JSON output during integration.

### DR-0002 — What `substrate world doctor` reports by default

**Decision owner(s):** Substrate CLI maintainers  
**Date:** 2026-01-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md`, `docs/project_management/_archived/doctor_scopes/DS0-spec.md`

**Problem / Context**
- `substrate world doctor` must report world readiness, while operators still need host readiness context when the world is down.

**Option A — World-only output (agent-reported facts only)**
- **Pros:** Matches the command name strictly; keeps responsibility boundaries narrow; avoids duplication with `substrate host doctor`.
- **Cons:** Requires operators to run a second command for host readiness context; slows triage for unreachable-world cases.
- **Cascading implications:** More operator runbooks need multi-step instructions; more CI/smoke scripts need chained commands.
- **Risks:** Operators interpret “world doctor failed” without a clear next action.
- **Unlocks:** A cleaner long-term model where world doctor is exclusively agent-reported.
- **Quick wins / low-hanging fruit:** Simplifies the world doctor renderer when the world is reachable.

**Option B — Combined output with explicit `host` + `world` blocks**
- **Pros:** One command contains both the host readiness context and the agent-reported world readiness; preserves scope boundaries via explicit blocks; reduces false-green risk by coupling `ok` to both blocks.
- **Cons:** Slightly more verbose output; requires consistent envelope schema and internal consumer updates.
- **Cascading implications:** Requires updating internal consumers that parse doctor JSON to use scoped blocks.
- **Risks:** Poor labeling blurs “host-inferred” vs “agent-reported” facts.
- **Unlocks:** A stable envelope for future tooling and automation (`host` and `world` blocks become the interface).
- **Quick wins / low-hanging fruit:** Immediate operator UX improvement for “world down” triage.

**Recommendation**
- **Selected:** Option B — combined output with explicit `host` + `world` blocks
- **Rationale (crisp):** Explicit scope blocks keep contracts auditable while keeping operator triage single-command.

**Follow-up tasks (explicit)**
- `DS0-code`: implement world doctor envelope with `host` + `world` blocks and top-level `ok`.
- `DS0-test`: add tests that validate the envelope structure and `ok == host.ok && world.ok`.
- `DS0-integ-core`: confirm internal consumers are updated and no legacy parsers expect a flat schema.

### DR-0003 — How to obtain guest-kernel facts on macOS (Landlock, FS strategy probe)

**Decision owner(s):** Substrate world backend maintainers  
**Date:** 2026-01-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md`, `docs/project_management/_archived/doctor_scopes/DS0-spec.md`

**Problem / Context**
- On macOS, correctness/security facts for enforcement (Landlock ABI/support, world filesystem probe result) are properties of the Lima guest kernel and the world-agent service privileges.
- DS0 requires a reliable mechanism to surface those facts in `substrate world doctor` without requiring a guest-installed `substrate` binary.

**Option A — Run a guest-installed `substrate` CLI via Lima shell**
- **Pros:** Reuses existing Linux doctor logic; avoids adding a new agent endpoint route.
- **Cons:** Depends on guest package state and user permissions; produces misleading results if `substrate` is not installed in the guest or runs as an unprivileged guest user.
- **Cascading implications:** Adds operational complexity to provisioning and troubleshooting; increases variance across installs and environments.
- **Risks:** “Doctor” results drift based on guest filesystem state and PATH composition.
- **Unlocks:** A short-term path for macOS parity if the guest image always contains `substrate`.
- **Quick wins / low-hanging fruit:** Minimal new code in world-agent.

**Option B — Add a world-agent endpoint that returns a structured world doctor report**
- **Pros:** Measures the actual enforcement environment (guest kernel + agent privileges); removes dependency on guest-installed CLI; provides a stable API contract for host UI and automation.
- **Cons:** Introduces new API surface area (routing + schema); requires versioning and serde tests.
- **Cascading implications:** Requires adding types in `agent-api-types` and client support in `agent-api-client`.
- **Risks:** Schema drift if the endpoint is not treated as a stable contract.
- **Unlocks:** A single cross-platform mechanism for world readiness facts.
- **Quick wins / low-hanging fruit:** Enables macOS Landlock ABI reporting without additional guest setup.

**Recommendation**
- **Selected:** Option B — world-agent endpoint with a structured report
- **Rationale (crisp):** The agent has the correct vantage point for guest-kernel facts, and the endpoint is deterministic across installs.

**Follow-up tasks (explicit)**
- `DS0-code`: implement `GET /v1/doctor/world` in world-agent and the corresponding CLI client call.
- `DS0-test`: add schema round-trip tests for `WorldDoctorReportV1` and CLI parsing tests for the envelope.
- `DS0-integ-core`: validate macOS behavior (Landlock ABI/support comes from the guest kernel via the endpoint).

### DR-0004 — Orchestration branch naming

**Decision owner(s):** Substrate project maintainers  
**Date:** 2026-01-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/sequencing.json`, `docs/project_management/_archived/doctor_scopes/tasks.json`, `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md`

**Problem / Context**
- The planning pack declares an orchestration branch used by triad automation and CI dispatch commands; the branch name must match the sequencing entry and the integration workflow refs.

**Option A — `feat/doctor_scopes`**
- **Pros:** Matches the feature directory name; matches the default branch naming used by some planning tooling.
- **Cons:** Conflicts with the sprint branch naming already recorded in `docs/project_management/next/sequencing.json`.
- **Cascading implications:** CI dispatch commands that use `CI_WORKFLOW_REF` drift from sequencing; operators lose a single authoritative branch name.
- **Risks:** Mis-dispatching CI to the wrong ref or failing dispatch because the ref is missing.
- **Unlocks:** Directory-aligned naming for tools that default to underscore-separated feature IDs.
- **Quick wins / low-hanging fruit:** Minimal to type.

**Option B — `feat/doctor-scopes`**
- **Pros:** Matches `docs/project_management/next/sequencing.json` and ADR; aligns with existing hyphenated branch conventions; keeps CI dispatch refs consistent.
- **Cons:** Requires explicitly setting `meta.automation.orchestration_branch` in `tasks.json`.
- **Cascading implications:** All CI dispatch commands use the same ref string; feature directory and sequencing remain aligned.
- **Risks:** None beyond creating/pushing the branch before dispatch.
- **Unlocks:** Consistent repo-wide branch naming for sequencing-backed feature work.
- **Quick wins / low-hanging fruit:** Reduces preflight friction (one ref string reused everywhere).

**Recommendation**
- **Selected:** Option B — `feat/doctor-scopes`
- **Rationale (crisp):** Sequencing is the execution spine; the orchestration branch name matches it to avoid CI dispatch drift.

**Follow-up tasks (explicit)**
- `F0-exec-preflight`: confirm the ref exists on the remote before dispatch (`CI_WORKFLOW_REF="feat/doctor-scopes"`).
- `DS0-integ-core`: dispatch CI and smoke using `WORKFLOW_REF="feat/doctor-scopes"` and record run ids/URLs.
- `DS0-integ`: re-run CI and smoke using `WORKFLOW_REF="feat/doctor-scopes"` and record run ids/URLs in closeout.

### DR-0005 — World-doctor agent endpoint path

**Decision owner(s):** Substrate agent API maintainers  
**Date:** 2026-01-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md`, `docs/project_management/_archived/doctor_scopes/DS0-spec.md`

**Problem / Context**
- DS0 introduces an agent endpoint for world doctor. The path must be stable, descriptive, and extensible for future doctor endpoints.

**Option A — `GET /v1/doctor/world`**
- **Pros:** Groups doctor endpoints under a dedicated namespace; supports future additions (`/v1/doctor/host`) without one-off routes; is readable in logs and traces.
- **Cons:** Adds a new route subtree.
- **Cascading implications:** Router and client changes reference the new path; docs and tests must match exactly.
- **Risks:** None beyond route registration and versioning discipline.
- **Unlocks:** Clear API organization for “doctor” features.
- **Quick wins / low-hanging fruit:** Straightforward discoverability for operators and developers.

**Option B — `GET /v1/world_doctor`**
- **Pros:** Short path; minimal route nesting.
- **Cons:** Harder to extend to additional doctor endpoints without proliferating one-off routes; weak grouping in API surface.
- **Cascading implications:** Future endpoints require repeated ad-hoc naming decisions.
- **Risks:** API surface becomes inconsistent over time.
- **Unlocks:** Slightly shorter request URLs.
- **Quick wins / low-hanging fruit:** Minimal typing.

**Recommendation**
- **Selected:** Option A — `GET /v1/doctor/world`
- **Rationale (crisp):** A dedicated doctor namespace keeps the API extensible without one-off route naming.

**Follow-up tasks (explicit)**
- `DS0-code`: implement the world-agent route and handler at `/v1/doctor/world`, and call it from the CLI.
- `DS0-test`: add serde/schema tests and CLI tests that depend on this exact path.
- `DS0-integ-core`: validate the endpoint wiring through the CLI on behavior platforms.

### DR-0006 — Agent endpoint request shape

**Decision owner(s):** Substrate agent API maintainers  
**Date:** 2026-01-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/_archived/doctor_scopes/DS0-spec.md`

**Problem / Context**
- The agent endpoint must be callable by the CLI and smoke scripts with minimal interop risk and minimal input validation surface.

**Option A — `GET` with no request body**
- **Pros:** Lowest interoperability risk across clients and proxies; minimal schema and validation surface; simple smoke and manual commands.
- **Cons:** Per-request customization (timeouts/verbosity) cannot be expressed via payload.
- **Cascading implications:** Any future knobs require new query parameters or a new endpoint version.
- **Risks:** Future extension pressure leads to ad-hoc query flags without a versioned request model.
- **Unlocks:** Deterministic endpoint behavior with a single contract.
- **Quick wins / low-hanging fruit:** Simplifies initial implementation and test harnesses.

**Option B — `POST` with a structured request body**
- **Pros:** Clear extension point for per-request knobs; versioned request schema can evolve.
- **Cons:** Adds schema surface (input validation, compatibility); increases ambiguity risk if knobs are not exhaustively specified in specs.
- **Cascading implications:** Requires request models in `agent-api-types` and more test coverage for input combinations.
- **Risks:** Partial specification leads to divergent behavior across platforms and versions.
- **Unlocks:** Future configurable probes without endpoint proliferation.
- **Quick wins / low-hanging fruit:** None for DS0; overhead is immediate.

**Recommendation**
- **Selected:** Option A — `GET` with no request body
- **Rationale (crisp):** DS0 requires a fixed report; `GET` with no body keeps the contract small and deterministic.

**Follow-up tasks (explicit)**
- `DS0-code`: implement the endpoint as `GET` with no request body and fixed behavior.
- `DS0-test`: add schema tests that assume no request payload and fixed response fields.

### DR-0007 — JSON contract shape for doctor commands

**Decision owner(s):** Substrate CLI maintainers  
**Date:** 2026-01-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/_archived/doctor_scopes/DS0-spec.md`, `docs/project_management/_archived/doctor_scopes/manual_testing_playbook.md`, `docs/project_management/_archived/doctor_scopes/smoke/`

**Problem / Context**
- Existing doctor output mixes host- and world-derived facts across platforms.
- DS0 requires stable, auditable automation contracts that distinguish host readiness from world readiness.

**Option A — Keep a flat top-level JSON shape and add more keys**
- **Pros:** Smaller diff from existing output; fewer internal consumers need structural updates.
- **Cons:** Encourages scope mixing; increases risk of “false green” where host checks pass but world checks are absent or stale.
- **Cascading implications:** Future additions expand the flat namespace and make it harder to reason about provenance of fields.
- **Risks:** Consumers couple to incidental field presence and interpret host fields as world facts.
- **Unlocks:** Faster short-term adoption for existing scripts that parse flat keys.
- **Quick wins / low-hanging fruit:** Minimal implementation churn.

**Option B — Introduce an explicit schema version and scoped `host`/`world` blocks**
- **Pros:** Makes scope provenance explicit; supports strict contract tests; enables stable envelope semantics across platforms.
- **Cons:** Requires updating internal consumers that parse doctor JSON and related tests/fixtures.
- **Cascading implications:** Requires internal consumer updates in the same slice to avoid drift.
- **Risks:** Incomplete consumer updates break downstream commands until reconciled.
- **Unlocks:** A stable interface for future tooling and reporting pipelines.
- **Quick wins / low-hanging fruit:** Clear operator UX: one report contains both scopes with explicit boundaries.

**Recommendation**
- **Selected:** Option B — schema versioned envelope with `host` and `world` blocks
- **Rationale (crisp):** Scope separation in the JSON contract is required for auditability and to prevent false-green automation.

**Follow-up tasks (explicit)**
- `DS0-code`: implement the envelope schema and update internal consumers that parse doctor JSON.
- `DS0-test`: add schema and consumer tests/fixtures that validate the new scoped blocks.
- `DS0-integ-core`: confirm consumers parse the new schema and local gates remain green.

### DR-0008 — Exit code mapping for doctor commands

**Decision owner(s):** Substrate CLI maintainers  
**Date:** 2026-01-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`, `docs/project_management/_archived/doctor_scopes/DS0-spec.md`

**Problem / Context**
- Existing doctor behavior historically used exit code `2` for readiness failures on some platforms, which conflicts with the canonical taxonomy.
- DS0 adds new failure modes (world disabled, world unreachable, missing prerequisites) that require deterministic mapping for automation.

**Option A — Keep existing “doctor failures use exit 2” behavior**
- **Pros:** Minimizes short-term compatibility churn for any scripts that assume `2` for doctor failures.
- **Cons:** Conflicts with the canonical taxonomy where `2` is for usage/config errors; makes automation inconsistent across Substrate commands.
- **Cascading implications:** Future features either copy the inconsistency or introduce per-command exit code meanings.
- **Risks:** Operators misdiagnose usage errors vs readiness failures; CI pipelines treat readiness failures as misconfiguration.
- **Unlocks:** Compatibility with legacy assumptions in downstream tooling.
- **Quick wins / low-hanging fruit:** Minimal change to existing expectations.

**Option B — Align doctor exit codes to the canonical taxonomy**
- **Pros:** Consistent automation semantics across commands; aligns with the standards used by planning packs; supports unambiguous gating and scripting.
- **Cons:** Requires updating any tests/docs/scripts that assumed legacy doctor exit codes.
- **Cascading implications:** Specs, playbooks, smoke scripts, and tests must use the same mapping without per-platform drift.
- **Risks:** Partial updates create inconsistent behavior across platforms.
- **Unlocks:** A stable contract where exit codes map to “dependency unavailable” vs “missing prerequisites” vs “usage error”.
- **Quick wins / low-hanging fruit:** Immediate consistency with `EXIT_CODE_TAXONOMY.md`.

**Recommendation**
- **Selected:** Option B — align to the canonical taxonomy
- **Rationale (crisp):** DS0 is a contract change; aligning to the canonical taxonomy removes ambiguity and stabilizes automation.

**Follow-up tasks (explicit)**
- `DS0-code`: implement the DS0-spec exit code mapping, including the `3` vs `4` discriminator for “unreachable” vs “not provisioned”.
- `DS0-test`: add tests that enforce exit codes for disabled/unreachable/unsupported cases per DS0-spec.
- `DS0-integ-core`: validate exit codes during integration and capture evidence in the session log.

### DR-0009 — Doctor side effects (agent/VM start)

**Decision owner(s):** Substrate CLI maintainers  
**Date:** 2026-01-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/_archived/doctor_scopes/DS0-spec.md`

**Problem / Context**
- “Doctor” commands are safety diagnostics. Starting services or provisioning during doctor runs makes results non-deterministic and adds side effects that are not expected during validation and smoke.

**Option A — Doctor ensures the world backend is running (spawn/start)**
- **Pros:** Higher chance of returning success immediately after installation; one command can recover from “service stopped” states.
- **Cons:** Introduces side effects; complicates smoke scripts and makes repeated runs depend on prior state; mixes diagnosis with remediation.
- **Cascading implications:** Requires carefully audited state changes and logs; increases complexity around permissions and prompting.
- **Risks:** Doctor changes system state unexpectedly and hides underlying provisioning issues.
- **Unlocks:** A “self-healing” UX for some environments.
- **Quick wins / low-hanging fruit:** Fewer failures when the only issue is “service not started”.

**Option B — Doctor is passive (no provisioning, no spawning, no VM start)**
- **Pros:** Deterministic; safe; compatible with CI and smoke; results reflect the actual system state without mutation.
- **Cons:** Users must run explicit provisioning or start commands to remediate.
- **Cascading implications:** Operator runbooks reference separate provisioning flows.
- **Risks:** None beyond user friction in recovery workflows.
- **Unlocks:** Repeatable diagnostics that support gating and regression detection.
- **Quick wins / low-hanging fruit:** Simplifies implementation and makes failures reproducible.

**Recommendation**
- **Selected:** Option B — passive only
- **Rationale (crisp):** Deterministic, side-effect-free diagnostics are required for smoke and reliable operator triage.

**Follow-up tasks (explicit)**
- `DS0-code`: ensure both doctor commands avoid provisioning/spawning/VM start paths.
- `DS0-integ-core`: validate that running doctor does not start or provision the world backend as a side effect.

### DR-0010 — Behavior on Windows for doctor scopes

**Decision owner(s):** Substrate platform maintainers  
**Date:** 2026-01-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/_archived/doctor_scopes/DS0-spec.md`, `docs/project_management/_archived/doctor_scopes/manual_testing_playbook.md`

**Problem / Context**
- DS0 introduces new doctor scope commands. On Windows, the backend is not ready to report the required host/world readiness contracts without false-green risk.

**Option A — Keep placeholder “not implemented” output with `ok=true`**
- **Pros:** Avoids breaking any Windows workflows that assume exit code `0`.
- **Cons:** Creates false-green output; automation and operators treat Windows as ready when it is not; violates DS0 acceptance criteria.
- **Cascading implications:** Downstream tools accept unsupported state as successful; hard to detect regressions.
- **Risks:** Security posture degrades because “doctor green” does not reflect enforcement readiness.
- **Unlocks:** Compatibility with existing Windows CI expectations.
- **Quick wins / low-hanging fruit:** Minimal code change.

**Option B — Explicit unsupported contract (`ok=false`, status `unsupported`, exit code `4`)**
- **Pros:** No false-green output; automation can treat Windows as not supported/missing prerequisites; contract is explicit and testable.
- **Cons:** Any Windows scripts expecting exit `0` must be updated.
- **Cascading implications:** CI parity tasks and docs align to explicit unsupported semantics; manual playbook and smoke scripts validate the behavior.
- **Risks:** Minor compatibility churn for any existing scripts.
- **Unlocks:** Clean separation between “feature not supported” and “feature failing”.
- **Quick wins / low-hanging fruit:** A deterministic contract that stays stable until Windows support is implemented.

**Recommendation**
- **Selected:** Option B — explicit unsupported contract with exit `4`
- **Rationale (crisp):** Explicit unsupported semantics prevent false-green and are required for auditability.

**Follow-up tasks (explicit)**
- `DS0-code`: implement explicit unsupported behavior for Windows host/world doctor paths.
- `DS0-test`: add tests that enforce the Windows unsupported JSON contract and exit code.
- `DS0-integ-windows`: address CI parity failures and confirm Windows compile parity is green.

### DR-0011 — Behavior when world isolation is disabled

**Decision owner(s):** Substrate CLI maintainers  
**Date:** 2026-01-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/_archived/doctor_scopes/DS0-spec.md`

**Problem / Context**
- When world isolation is disabled by effective config, running probes against world-agent transport can produce misleading “agent down” output and introduces non-determinism for host-only installs.

**Option A — Probe transport even when world is disabled**
- **Pros:** Detects “disabled but still installed” states and can surface service issues even when world is not enabled.
- **Cons:** Produces confusing output on host-only installs; makes disabled state look like an error; adds unnecessary dependency checks.
- **Cascading implications:** Operator triage becomes ambiguous; scripts cannot distinguish “disabled by choice” from “agent unreachable”.
- **Risks:** Users interpret disabled state as a failure and attempt unnecessary remediation.
- **Unlocks:** Extra diagnostics for mixed installs.
- **Quick wins / low-hanging fruit:** Reuses existing probing paths.

**Option B — Short-circuit when world is disabled**
- **Pros:** Deterministic output; clear operator signal (`status=disabled`); avoids transport probing and related side effects.
- **Cons:** Does not report transport readiness while disabled.
- **Cascading implications:** Operator runbooks reference enabling world before probing transport readiness.
- **Risks:** None beyond reduced diagnostics in a disabled state.
- **Unlocks:** Stable automation semantics for `--no-world` and host-only installs.
- **Quick wins / low-hanging fruit:** Simple, consistent behavior across platforms.

**Recommendation**
- **Selected:** Option B — short-circuit when world is disabled
- **Rationale (crisp):** Disabled state is an intentional configuration and requires deterministic reporting without transport probes.

**Follow-up tasks (explicit)**
- `DS0-code`: implement the disabled short-circuit semantics in world doctor.
- `DS0-test`: add tests that enforce exit `4` and `world.status=="disabled"` for `--no-world`.
- `DS0-integ-core`: validate the disabled short-circuit during integration and manual playbook runs.

### DR-0012 — Timeout/retry policy for world-doctor agent request

**Decision owner(s):** Substrate CLI maintainers  
**Date:** 2026-01-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/_archived/doctor_scopes/DS0-spec.md`

**Problem / Context**
- World doctor is used in smoke and CI contexts where deterministic runtime is required. Retrying can hide failures and inflate time budgets.

**Option A — Retry with backoff**
- **Pros:** More resilient to transient jitter; improves success rate during unstable startup windows.
- **Cons:** Increases runtime variance; makes smoke scripts slower and less deterministic; failure reason is less crisp.
- **Cascading implications:** CI time budgets and operator guidance must account for multi-attempt behavior.
- **Risks:** Intermittent issues become harder to reproduce and debug.
- **Unlocks:** Better UX for users immediately after starting a service.
- **Quick wins / low-hanging fruit:** Better success rate in the presence of transient network/service flakiness.

**Option B — Single attempt with strict timeouts (no retries)**
- **Pros:** Deterministic; faster failure with clear error classification; supports reliable smoke and regression detection.
- **Cons:** Less tolerant of transient flakiness; users may re-run after provisioning.
- **Cascading implications:** Timeout values must be set explicitly and documented in implementation notes.
- **Risks:** A transient failure produces an exit `3` even if a second attempt would succeed.
- **Unlocks:** Predictable CI and operator triage.
- **Quick wins / low-hanging fruit:** Simple implementation with crisp failure semantics.

**Recommendation**
- **Selected:** Option B — single attempt with strict timeouts (no retries)
- **Rationale (crisp):** Determinism is required for smoke and automation; retries hide failures and inflate runtimes.

**Follow-up tasks (explicit)**
- `DS0-code`: implement a single-attempt agent request path with explicit timeouts and no retries.
- `DS0-test`: add a test that ensures the client does not retry on failure (one request per invocation).
