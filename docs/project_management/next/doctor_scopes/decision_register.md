# Decision Register — Doctor scope split (host vs world)

This decision register supports `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md`.

## DR-0001 — CLI naming for host-scoped doctor

Option A: `substrate host doctor`
- Pros: Clear scope label; aligns with other “scope nouns” (`world`, `shim`, `health`) without overloading `world`.
- Cons: Adds a new top-level subcommand.

Option B: `substrate world doctor --host` (or `substrate world doctor host`)
- Pros: Avoids a new top-level subcommand.
- Cons: Harder to discover; mixes two scopes under the `world` namespace; encourages “world doctor == host doctor” confusion on macOS.

Selected: Option A (`substrate host doctor`)

## DR-0002 — What `substrate world doctor` reports by default

Option A: World-only output (agent-reported “in-world” facts only)
- Pros: Strictly matches the “world doctor” name; avoids mixing responsibilities.
- Cons: When the world is unreachable, operators still need host diagnostics; forces additional steps and slows triage.

Option B: Combined output with explicit `host` + `world` blocks (world facts authoritative)
- Pros: One-stop diagnostics; makes the “scope split” explicit without hiding host readiness. Better operator ergonomics when world is down.
- Cons: Slightly more verbose output; must be careful to avoid conflating host-inferred and agent-reported facts.

Selected: Option B (combined `host` + `world` output; `substrate host doctor` provides host-only)

## DR-0003 — How to obtain guest-kernel facts on macOS (Landlock, mount capability, FS strategy probes)

Option A: Execute a guest-installed `substrate` CLI (e.g. `limactl shell substrate substrate world doctor --json`)
- Pros: Reuses existing Linux doctor implementation.
- Cons: Not reliable: dev installer may be the only reason `substrate` exists in the guest; guest user permissions can produce misleading failures (socket perms, mount EPERM).

Option B: Add a world-agent endpoint that reports “world doctor” facts from the service’s perspective
- Pros: Does not depend on a guest-installed CLI; measures the actual service/kernel used for enforcement; stable API for host UI.
- Cons: Requires new API surface and schema/versioning.

Selected: Option B (new world-agent endpoint)

## DR-0004 — Orchestration branch naming

Option A: `feat/doctor_scopes` (match feature directory name)
- Pros: Matches the default `make planning-new-feature FEATURE=doctor_scopes` branch naming.
- Cons: Conflicts with the already-established sprint branch naming in `docs/project_management/next/sequencing.json`.

Option B: `feat/doctor-scopes` (match sprint entry and ADR)
- Pros: Matches `docs/project_management/next/sequencing.json` and the ADR “Intended branch”; consistent with other hyphenated feature branches.
- Cons: Requires explicitly setting `meta.automation.orchestration_branch` (cannot rely on default naming).

Selected: Option B (`feat/doctor-scopes`)

## DR-0005 — World-doctor agent endpoint path

Option A: `GET /v1/doctor/world`
- Pros: Namespaces “doctor” endpoints cleanly; allows future additions like `/v1/doctor/host` without mixing unrelated endpoints.
- Cons: Adds a new route subtree.

Option B: `GET /v1/world_doctor`
- Pros: Short; mirrors existing “capabilities” style.
- Cons: Harder to extend without proliferating one-off endpoints; less discoverable grouping.

Selected: Option A (`GET /v1/doctor/world`)

## DR-0006 — Agent endpoint request shape

Option A: `GET` with no request body (fixed behavior)
- Pros: Simple interoperability across transports; easy to call from CLI and smoke scripts; no input validation surface.
- Cons: Cannot customize timeouts/verbosity via request payload.

Option B: `POST` with a structured request body (options, verbosity, probes)
- Pros: Extensible per-request; can add knobs without new endpoints.
- Cons: More schema surface area; increases ambiguity unless every knob is exhaustively specified.

Selected: Option A (no request body; fixed behavior)

## DR-0007 — JSON contract shape for doctor commands

Option A: Keep the current flat top-level JSON keys and add more keys for “world”
- Pros: Smaller diff to current code/tests; fewer consumers need updates.
- Cons: Encourages ambiguous mixing of host-derived and agent-derived facts; hard to make contracts auditable by scope.

Option B: Introduce an explicit schema version and `host`/`world` blocks; treat them as the only stable interface
- Pros: Makes scope boundaries explicit; reduces “false green” risk; enables strict, testable contracts per scope.
- Cons: Requires updating internal consumers/tests/docs that parse the existing flat shape.

Selected: Option B (`schema_version` + `host` + `world`; no stability guarantees for legacy flat keys)

## DR-0008 — Exit code mapping for doctor commands

Option A: Keep current doctor behavior (non-zero uses exit code `2` for readiness failures)
- Pros: Minimizes behavioral change in the short term.
- Cons: Conflicts with the canonical taxonomy where `2` is “usage/config”; increases operator confusion and breaks consistent automation.

Option B: Align to `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- Pros: Consistent automation semantics across Substrate commands and planning packs.
- Cons: Requires updating tests/docs that assumed “doctor failures are `2`”.

Selected: Option B (use canonical taxonomy; specify exact mappings in specs)

## DR-0009 — Doctor side effects (agent/VM start)

Option A: Doctor attempts to start/ensure the world backend (spawn agent, start VM) to improve success rate
- Pros: “One command fixes it” feel for some setups; fewer false negatives when services are merely stopped.
- Cons: Violates the “doctor is advisory” posture; introduces non-audited side effects; makes smoke scripts non-deterministic.

Option B: Doctor is passive (no provisioning; no spawning; no VM start); it only probes and reports
- Pros: Deterministic; safe; matches “doctor” expectations; avoids accidental state mutation.
- Cons: Users must run explicit provisioning commands/scripts to remediate.

Selected: Option B (passive only)

## DR-0010 — Behavior on Windows for doctor scopes

Option A: Keep the current placeholder behavior (`ok=true` + “not implemented” message)
- Pros: Keeps existing CI assumptions about exit code 0.
- Cons: Creates “false green”; violates the ADR requirement to be explicit about unsupported platforms.

Option B: Explicit “unsupported” report (`ok=false`) with exit code `4`
- Pros: No silent “ok”; automation can treat it as “not supported / prereq missing”.
- Cons: Any Windows scripts expecting `0` must be updated.

Selected: Option B (explicit unsupported + exit code `4`)

## DR-0011 — Behavior when world isolation is disabled (install/config/env)

Option A: Always probe sockets and report “backend unreachable” even when world is disabled
- Pros: Might catch “disabled but still installed” states.
- Cons: Misleading on host-only installs; contradicts the backlog goal to make host-only installs quiet and explicit.

Option B: Short-circuit when world is disabled: report `status=disabled` and do not probe the agent socket
- Pros: Clear operator signal; avoids misleading “agent down” messages when world is intentionally off; deterministic.
- Cons: Requires users to enable world before probing transport readiness.

Selected: Option B (short-circuit on disabled; no socket probing)

## DR-0012 — Timeout/retry policy for world-doctor agent request

Option A: Retry with backoff (agent-api-client default retry settings)
- Pros: Better resilience to transient startup/network jitter.
- Cons: Slower and less deterministic; complicates smoke scripts and CI time budgets.

Option B: Single attempt with strict timeouts (no retries)
- Pros: Fast and deterministic; failure modes are clear and reproducible.
- Cons: Less tolerant of transient flakiness; operators may need to re-run manually after provisioning.

Selected: Option B (single attempt; strict timeout)
