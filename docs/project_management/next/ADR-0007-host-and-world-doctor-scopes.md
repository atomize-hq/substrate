# ADR-0007 — Host vs World Doctor Scopes

## Status
- Status: Accepted
- Date (UTC): 2026-01-07
- Owner(s): spenser

## Scope
- Feature directory: `docs/project_management/_archived/doctor_scopes/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Intended branch: `feat/doctor-scopes`
- Standards:
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`

## Related Docs
- Plan: `docs/project_management/_archived/doctor_scopes/plan.md`
- Decision Register: `docs/project_management/_archived/doctor_scopes/decision_register.md`
- Existing docs that MUST be updated as part of this ADR:
  - `docs/COMMANDS.md`
  - `docs/ISOLATION_SUPPORT_MATRIX.md` (currently marked incomplete)

## Executive Summary (Operator)

ADR_BODY_SHA256: 17f608b37d863a3ca196b75e164be79f4dce294cdc1e37f693984f8687aa926f
### Changes (operator-facing)
- Split doctor into host vs world scopes (without requiring a guest-installed `substrate` CLI)
  - Existing: `substrate world doctor` is host-oriented on macOS (Lima/transport/service checks) and kernel-oriented on Linux (overlay/nft/cgroup + Landlock probe). On macOS it cannot report guest-kernel facts like Landlock ABI/support without manually running commands inside the VM.
  - New: `substrate host doctor` reports host/transport readiness only. `substrate world doctor` reports world-agent / in-world readiness by querying the world-agent for a structured “world doctor” report; it includes guest-kernel facts (e.g., Landlock support/ABI) on macOS without depending on a guest-installed `substrate` binary.
  - Why: Operators need one canonical “is isolation actually enforceable right now?” answer. On macOS, that answer lives in the Lima guest kernel + world-agent, not on the host.
  - Links:
    - `crates/shell/src/execution/platform/macos.rs` (current macOS doctor is host/transport/service oriented)
    - `crates/shell/src/execution/platform/linux.rs` (current Linux doctor probes Landlock + kernel prereqs)
    - `crates/world-agent/src/service.rs` (agent capabilities + request handling)
    - `docs/WORLD.md` (current framing of `world doctor`)
    - `docs/COMMANDS.md` (command matrix; needs update)

## Problem / Context
- On macOS, Substrate world execution runs inside a Lima guest (Linux world-agent). Many correctness/security facts (Landlock ABI/support, mount namespace behavior, overlay viability in the guest) are properties of the guest kernel and the world-agent service privileges.
- Today’s `substrate world doctor` on macOS is necessarily host-centric (Lima installed, VM running, forwarding path, service/socket active), and does not report guest-kernel enforcement readiness (e.g. Landlock).
- Operators currently work around this by manually running commands inside the guest (e.g. `limactl shell substrate ...`), which is brittle and may require a guest-installed `substrate` binary (dev-installer quirk), and can produce misleading failures when run as an unprivileged guest user (socket permissions, mount EPERM).

## Goals
- Introduce a clear, user-facing split:
  - `substrate host doctor`: host/transport/service readiness only.
  - `substrate world doctor`: “in-world” readiness from the world-agent perspective (guest kernel + service privileges), without requiring a guest-installed `substrate` CLI.
- Make macOS `substrate world doctor` report guest-kernel facts that matter for enforcement (at minimum: Landlock support/ABI and the active FS strategy probe outcome).
- Keep Linux behavior strong and understandable: Linux should continue to surface Landlock support and overlay strategy diagnostics, but in a way that distinguishes host prerequisites from world-agent enforcement readiness.
- Preserve JSON output stability for automation by introducing new, explicitly-scoped JSON shapes rather than overloading ambiguous fields.

## Non-Goals
- Redesign the entire health stack (`substrate health`, `substrate shim doctor`, etc.). This ADR is scoped to doctor surfaces only.
- Require `substrate` to be installed inside the Lima guest. Guest probing must be done via the agent API contract.
- Expand Windows world doctor coverage beyond what the current platform backend can reliably report (Windows behavior must be explicitly guarded; parity work can follow).

## User Contract (Authoritative)

### CLI
- `substrate host doctor [--json]`
  - Purpose: Report host-side prerequisites and connectivity to the world backend.
  - Linux:
    - Reports host kernel/tooling prerequisites that affect world execution (e.g. overlay availability, nft presence, cgroup v2).
    - Reports agent socket activation status (e.g. `/run/substrate.sock`) and probe status when possible.
  - macOS:
    - Reports Lima installation and host virtualization support.
    - Reports VM running status and forwarding/agent socket reachability (capabilities probe).
    - MUST NOT attempt to infer guest-kernel primitives (Landlock ABI/support) except via explicit world-agent reported fields.
- `substrate world doctor [--json]`
  - Purpose: Report “in-world readiness” from the world-agent’s perspective (guest kernel + agent privileges).
  - Behavior:
    - MUST query the world-agent for a structured “world doctor” report over the active transport.
    - MUST NOT depend on `substrate` being installed in the guest.
    - MUST include the `host` doctor summary (host/transport readiness) and the `world` doctor summary (agent-reported readiness) as separate JSON blocks.
  - Output:
    - Text output is human-first: short PASS/WARN/FAIL lines grouped into `Host` and `World` sections.
    - JSON output includes separate `host` and `world` objects (stable field names), plus top-level `ok`.

Exit codes:
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- `0`: doctor report `ok=true` (all required checks passed)
- `1`: unexpected internal error (bug, panic, unhandled I/O)
- `2`: CLI usage/config error (invalid flags/args)
- `3`: required dependency unavailable (world enabled, transport prerequisites present, but world-agent unreachable)
- `4`: not supported / missing prerequisites (platform unsupported; world disabled; world not provisioned; or agent reachable but cannot enforce required primitives)

Discriminator for exit `3` vs `4` when world is enabled:
- Exit `4` when transport prerequisites are not satisfied (not provisioned):
  - Linux: agent socket path does not exist (`host.world_socket.socket_exists==false`)
  - macOS: Lima prerequisites are not satisfied (`host.lima.installed==false`, `host.lima.virtualization==false`, `host.lima.vm_status!="Running"`, or `host.lima.service_active==false`)
- Exit `3` when transport prerequisites above are satisfied but the world-agent request fails (connect/probe/HTTP failure).

### Config
- No new configuration keys are introduced by default.
- If any new environment variables are introduced (e.g. debug/override knobs for doctor scope, transport, or timeouts), they MUST be documented in:
  - `docs/internals/env/inventory.md`
  - `docs/reference/env/` (as applicable)

### Platform guarantees
- Linux: `substrate world doctor` MUST report Landlock support and the active filesystem strategy probe result based on the agent’s view of the world execution environment.
- macOS: `substrate world doctor` MUST surface guest-kernel enforcement facts (Landlock support/ABI at minimum) via agent reporting.
- Windows: `substrate host doctor` and `substrate world doctor` MUST be explicit about what is not implemented (no silent “ok”); any unsupported blocks must be clearly marked.

## Architecture Shape

### Components
- `crates/shell/src/execution/cli.rs`
  - Add a new top-level `host` subcommand (mirrors structure of `world`, `shim`, `health`).
  - Update the `world doctor` routing to include “world scope” behavior.
- `crates/shell/src/execution/platform/macos.rs`
  - Extract the existing macOS doctor into a “host doctor” implementation.
  - Add a client call to the world-agent “world doctor” endpoint and render it.
- `crates/shell/src/execution/platform/linux.rs`
  - Keep existing host-kernel probing behavior as the host doctor implementation.
  - Add the world-agent “world doctor” client call and render it for `substrate world doctor`.
- `crates/world-agent/src/service.rs` (+ API types)
  - Add a structured “world doctor” report endpoint that returns:
    - Landlock support/ABI (guest kernel)
    - Filesystem strategy probe status/reason (agent view)
    - Any other agent-side prerequisites required to enforce `world_fs` policy
- `crates/agent-api-types` / `crates/agent-api-client`
  - Add request/response models and client methods for the new endpoint.

### End-to-end flow
- Inputs:
  - Host platform (linux/macos/windows)
  - Active transport to world-agent (UDS on Linux; forwarded socket on macOS)
- Derived state:
  - Host readiness summary (`substrate host doctor`)
  - World readiness summary via agent endpoint (`substrate world doctor`)
- Actions:
  - Host doctor performs local/transport probes.
  - World doctor calls the agent endpoint and renders results.
- Outputs:
  - Text doctor output (PASS/WARN/FAIL grouped by scope)
  - JSON doctor output with stable `host`/`world` blocks and top-level `ok`

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/next/sequencing.json` → sprint `doctor_scopes` (already present).
- Prerequisite integration task IDs:
  - None; this work is an isolated CLI/API contract change. (It must still pass cross-platform smoke + CI.)

## Security / Safety Posture
- `substrate world doctor` MUST avoid “false green”:
  - If the world backend is unreachable, it must exit `3` with a clear hint to run `substrate host doctor` for transport/provisioning diagnostics.
  - If the agent is reachable but cannot enforce required primitives (e.g., Landlock unsupported in the guest), it must report `ok=false` with actionable fields.
- `substrate host doctor` is advisory and must not attempt to “fix” the system (no writes, no provisioning side effects).

## Validation Plan (Authoritative)

### Tests
- Add unit/integration coverage for:
  - CLI parsing: `substrate host doctor` and `substrate world doctor` wiring.
  - JSON contract: stable `host`/`world` blocks and top-level `ok`.
  - Agent endpoint schema round-trip via `agent-api-types`.

### Manual validation
- macOS:
  - `substrate host doctor --json` should continue to report Lima/transport/service status.
  - `substrate world doctor --json` should report Landlock support/ABI from the guest kernel (when available), without requiring `substrate` installed in the guest.
- Linux:
  - Both commands should function on a socket-activated service; `world doctor` should not require root when the operator has socket permissions.

### Smoke scripts
- Update or add a smoke step that asserts:
  - macOS world doctor JSON includes a guest-kernel Landlock field (supported/abi/reason).
  - Linux world doctor JSON includes the same Landlock field (host == world kernel).

## Rollout / Backwards Compatibility
- This change is greenfield by default:
  - The stable JSON contract is the new `schema_version` + scoped `host`/`world` blocks; legacy flat top-level keys are not part of the stable interface.
  - Internal consumers in this repo (e.g., health/shim snapshots) MUST be updated in the same change set.

## Decision Summary
- Decision register: `docs/project_management/_archived/doctor_scopes/decision_register.md`
  - DR-0001: CLI naming (`substrate host doctor` vs alternatives)
  - DR-0002: `substrate world doctor` scope (combined vs world-only output)
  - DR-0003: Guest probing mechanism (agent endpoint vs guest-installed CLI)
