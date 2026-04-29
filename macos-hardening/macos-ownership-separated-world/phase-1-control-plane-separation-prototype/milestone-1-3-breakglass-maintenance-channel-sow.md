# Milestone 1.3: Breakglass Maintenance Channel SOW

Status: Proposed

## Purpose / Outcome

Define and prototype the minimum maintenance path needed to recover, inspect, and repair the ownership-separated macOS world without making those operations routine runtime capabilities. The outcome is an explicit breakglass channel with gating, audit expectations, and repo-defined operator workflows.

## Why This Milestone Exists

The current repo treats direct guest access as normal operations: `limactl shell`, guest `systemctl`, and direct `curl --unix-socket /run/substrate.sock` are written into docs and scripts. Once routine runtime access is brokered, the project still needs a way to diagnose failures and repair the environment. This milestone exists to make that path explicit and narrow instead of letting old same-user workflows linger informally.

## In-Scope

- Define the allowed breakglass operations for lifecycle repair, guest inspection, and agent recovery.
- Define how breakglass access is invoked, who can invoke it, and what evidence it must emit.
- Define how current direct maintenance instructions in docs and scripts are reclassified.
- Prototype an auditable breakglass path consistent with phase 0 contracts and phase 1 runtime changes.

## Out-of-Scope

- Rich operator UI for breakglass flows.
- Long-term enterprise policy integration.
- Reopening routine runtime transport decisions.

## Architectural Approach

Breakglass should be treated as a maintenance mode, not as an alternate routine API. The prototype should:

- require explicit operator action to enter breakglass mode
- keep breakglass paths separate from normal CLI execution paths
- emit traceable local evidence when breakglass is used
- prefer daemon-mediated maintenance commands where possible
- reserve direct `limactl shell`, direct SSH, guest `systemctl`, and guest `curl` for the smallest necessary set of recovery operations

The milestone should also define how breakglass exits and how operators return the system to normal broker-only operation.

## Dependencies / Sequencing

- Requires the same-user hardening track prerequisites named by the parent
  phase:
  - phase 1 milestone 1.1 transport contract unification
  - phase 1 milestone 1.2 policy application parity
  - phase 1 milestone 1.3 doctor/smoke readiness parity
  - phase 3 milestone 3.2 breakglass reclassification and doc cutover
- Requires milestone 1.1 and 1.2 so breakglass is designed relative to the new normal path, not the old one.
- Depends on phase 0 definitions of routine versus breakglass authority.
- Should inform the follow-on doc rewrite for macOS setup and operations guides.

## Concrete Repo Surfaces and File Pointers

- `docs/WORLD.md`
  Current operator narrative that still presents direct guest access as normal; future updates must mark these as breakglass only.
- `docs/cross-platform/mac_world_setup.md`
  Current setup and troubleshooting instructions using direct `limactl shell`, guest `systemctl`, and direct socket `curl`.
- `scripts/mac/lima-warm.sh`
  Current repair and reprovision logic that may be retained only as a breakglass or daemon-internal tool.
- `scripts/mac/lima-doctor.sh`
  Current direct diagnostic model that may need a breakglass classification or a broker-aware split.
- `scripts/substrate/install-substrate.sh`
  Current install path that needs a documented recovery story when daemon-owned provisioning fails.
- `scripts/substrate/uninstall-substrate.sh`
  Current teardown path that needs a safe operator recovery path for partial or failed ownership-separated installs.

## Deliverables

- A breakglass operation list covering:
  - VM inspection
  - service status
  - agent socket probing
  - forced reprovision
  - full teardown and rebuild
- Entry and exit rules for breakglass mode.
- An evidence checklist for breakglass sessions.
- A mapping from current docs/scripts to future breakglass-only usage or retirement.

## Acceptance Criteria

- The milestone clearly separates routine runtime operations from maintenance-only operations.
- Direct `limactl shell`, direct SSH, guest `systemctl`, and guest `curl` are documented as breakglass only.
- The milestone defines how breakglass usage is made visible in logs, traces, or operator evidence.
- The milestone defines how to return a system from breakglass to normal broker-only operation.

## Validation / Evidence Plan

- Review every currently documented direct guest maintenance command and classify it as:
  - retired
  - daemon-mediated
  - or breakglass-only
- Confirm the prototype still has a recoverability story when broker setup fails.
- Confirm breakglass is not required for normal doctor, health, or execution flows.
- Produce an operator evidence checklist that a future implementation PR can satisfy.

## Risks / Open Questions

- If breakglass is too convenient, teams will keep using it as the real control plane; the prototype must avoid that.
- Some failure scenarios may still require more direct guest access than desired until daemon recovery flows mature.
- The repo may need temporary dual documentation during migration from same-user Lima to ownership-separated operation.
