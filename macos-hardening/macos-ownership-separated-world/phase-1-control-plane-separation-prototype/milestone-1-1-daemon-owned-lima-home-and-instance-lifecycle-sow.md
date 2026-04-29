# Milestone 1.1: Daemon-Owned Lima Home and Instance Lifecycle SOW

Status: Proposed

## Purpose / Outcome

Prototype host-side ownership separation by moving Lima state and lifecycle under a Substrate-owned daemon or service account. The outcome is a running macOS world whose instance directory, credentials, and lifecycle commands are no longer routine developer-owned assets.

## Why This Milestone Exists

The current repo provisions and manages the macOS world as the developer user. `scripts/mac/lima-warm.sh` creates and configures the VM directly, and `MacLimaBackend` assumes the CLI can inspect and start the instance directly through `limactl`. Phase 1 must first break that ownership pattern before transport changes mean anything.

## In-Scope

- Create a daemon-owned state root for Lima control-plane artifacts.
- Move or recreate `LIMA_HOME` under daemon ownership.
- Move VM create/start/stop/provision operations behind daemon IPC.
- Update the macOS backend path so routine CLI execution no longer directly invokes `limactl`.
- Update install and uninstall planning so they manage daemon-owned state rather than same-user instance state.

## Out-of-Scope

- Final private forwarding path or broker endpoint implementation.
- Final breakglass UX.
- Full in-place migration for every existing development machine.

## Architectural Approach

Prototype the smallest credible ownership split:

- the daemon owns the state root and launches `limactl`
- the daemon provisions or delegates provisioning of the guest image and guest binaries
- the CLI asks the daemon for lifecycle operations and health summaries
- any remaining direct `limactl` usage is restricted to breakglass tooling

The prototype may choose to recreate the VM under daemon ownership instead of attempting in-place adoption of a user-owned `~/.lima/substrate` tree.

## Dependencies / Sequencing

- Requires phase 0 milestone 0.1 and milestone 0.2.
- Requires the same-user hardening track to have completed:
  - phase 1 milestone 1.1 transport contract unification
  - phase 1 milestone 1.2 policy application parity
  - phase 1 milestone 1.3 doctor/smoke readiness parity
  - phase 3 milestone 3.2 breakglass reclassification and doc cutover
- Must land before milestone 1.2 because the broker/transport work depends on daemon-owned VM state and credentials.
- Should identify any state-root abstractions needed before installer changes are attempted.

## Concrete Repo Surfaces and File Pointers

- `crates/world-mac-lima/src/lib.rs`
  Replace direct `ensure_vm_running`, `wait_for_agent`, and guest socket probing with daemon-mediated lifecycle requests.
- `crates/world-mac-lima/src/vm.rs`
  Current `limactl` helpers to either move behind daemon internals or stop being called by routine CLI code.
- `crates/world-mac-lima/src/limactl.rs`
  Current `limactl` discovery path that will no longer be a routine CLI requirement.
- `scripts/mac/lima-warm.sh`
  Extract or rehome provisioning logic so it can run under daemon ownership.
- `scripts/mac/lima/substrate.yaml`
  Evaluate any assumptions that the instance is created in a user-owned Lima home.
- `scripts/substrate/install-substrate.sh`
  Add daemon installation/bootstrap flow instead of directly calling user-owned warm logic as the normal macOS path.
- `scripts/substrate/uninstall-substrate.sh`
  Replace direct `limactl stop/delete substrate` assumptions with daemon-owned teardown.

## Deliverables

- Prototype design note for the daemon-owned state root and `LIMA_HOME` location.
- Implementation-ready lifecycle RPC list covering create/start/stop/status/provision.
- Prototype closeout note for the stable macOS control-plane registration
  mechanism required by Phase 2 installer, upgrade, and migration work.
- A repo change map showing which current shell/backend code paths become daemon clients.
- A migration strategy note for existing user-owned instances:
  - recreate
  - adopt with conversion
  - or explicitly unsupported for the prototype

## Acceptance Criteria

- The milestone defines a host state location owned by the daemon or dedicated service account, not the developer user.
- Routine CLI execution no longer requires direct `limactl` access in the intended prototype path.
- The milestone names the code and script seams where daemon lifecycle calls replace current direct lifecycle logic.
- The milestone states how install and uninstall flows treat daemon-owned instance state.

## Validation / Evidence Plan

- Demonstrate that the prototype design has no routine dependency on `~/.lima/substrate` owned by the developer user.
- Walk the current `scripts/mac/lima-warm.sh` flow and mark which steps move behind daemon control.
- Walk the current `MacLimaBackend` startup path and show where direct lifecycle checks become daemon calls.
- Confirm that a normal developer session cannot create or start the routine VM without going through Substrate.

## Risks / Open Questions

- Lima may have undocumented assumptions about user-session state that complicate daemon ownership.
- Installer UX may need explicit privileged setup that is acceptable for the prototype but not for long-term product UX.
- Recreating instances may be simpler than migration, but it raises workspace continuity and debugging questions.
