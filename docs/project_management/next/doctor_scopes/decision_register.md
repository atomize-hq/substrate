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

