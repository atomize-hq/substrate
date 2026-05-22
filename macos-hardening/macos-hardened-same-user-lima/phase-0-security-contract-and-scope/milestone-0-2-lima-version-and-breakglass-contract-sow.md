# Milestone 0.2: Lima Version and Breakglass Contract SOW

Status: Draft  
Last updated: 2026-05-19

## Purpose / outcome

Freeze the environment assumptions and operator escape hatches for hardened same-user Lima. The milestone outcome is a decision-ready contract that says which Lima/runtime capabilities are required for the supported mode, which manual workflows are retained only for breakglass, and which Substrate-owned commands must replace direct guest administration in normal operation.

## Why this milestone exists

Current macOS behavior still depends on tooling and workflows that are only
partly under Substrate's control, but some of the intended control plane is now
already landed.

- `scripts/mac/lima/substrate.yaml` assumes `vmType: "vz"`, guest systemd, and
  a mount model that still exposes all of `$HOME` read-only.
- `scripts/mac/lima-warm.sh`, `scripts/mac/lima-doctor.sh`, and
  `scripts/mac/smoke.sh` still rely heavily on direct `limactl shell`
  execution even though canonical CLI doctors already exist.
- `crates/world-mac-lima/src/forwarding.rs` already routes to the guest UDS
  endpoint `/run/substrate.sock` and intentionally skips SSH TCP fallback;
  remaining transport drift is stale `7788` references plus doctor/probe
  fallback use of host TCP `17788`.
- `crates/shell/src/builtins/world_gateway.rs`,
  `docs/contracts/substrate-gateway-operator-contract.md`, and
  `docs/contracts/substrate-gateway-status-schema.md` already define a gateway
  lifecycle/status contract, including `substrate world gateway sync|status|restart`
  and status JSON.
- `docs/cross-platform/mac_world_setup.md` still teaches direct guest build,
  install, service enablement, and raw guest probing as standard setup.

Hardening cannot succeed if the supported environment and breakglass boundary remain implicit.

## In-scope

- Define the minimum supported Lima and host capability contract for hardened mode.
- Define the accepted transport capability assumptions for the supported mode.
- Define which direct guest operations are breakglass-only.
- Define the replacement rule that normal lifecycle and diagnostics must flow through Substrate-owned commands and scripts.
- Define evidence expectations for future implementation phases that touch provisioning, docs, and transport.

## Out-of-scope

- Implementing transport unification, mount reduction, or CLI replacements.
- Selecting a different virtualization stack.
- Defining full production multi-user support on macOS.

## Architectural approach

This milestone should produce four concrete decisions.

1. Supported environment contract
   - Freeze the minimum macOS and Lima capability set required for hardened mode.
   - The contract must account for features already assumed by the repo:
     - `vmType: "vz"` in `scripts/mac/lima/substrate.yaml`
     - guest systemd units and socket activation
     - stable `limactl` SSH config layout used by `crates/world-mac-lima/src/forwarding.rs`
   - The contract must say whether `vsock-proxy` is required, optional, or
     unsupported for the hardened default.
   - The contract must explicitly include the already-landed managed gateway
     runtime artifact location `/run/substrate/substrate-gateway-runtime/` as a
     supported guest runtime surface.
2. Transport contract
   - Freeze one canonical forwarding story for supported mode.
   - State that the canonical guest service endpoint is `/run/substrate.sock`
     and that current runtime forwarding is UDS-backed.
   - State that SSH TCP fallback is intentionally unsupported in the default
     story.
   - Eliminate stale `7788` references and classify host TCP `17788` use as a
     compatibility/breakglass probe rather than the supported contract.
   - State that guest raw TCP listeners such as
     `SUBSTRATE_AGENT_TCP_PORT=61337` are not part of the hardened default and
     require explicit breakglass treatment if kept at all.
3. Breakglass contract
   - Reclassify direct `limactl shell`, direct guest `systemctl`, direct guest
     binary install, direct guest socket curls, and host-side
     `SUBSTRATE_WORLD_SOCKET` override use as breakglass or advanced/test-only
     workflows.
   - Require those workflows to be documented as emergency recovery or deep debugging only.
4. Substrate-owned control-plane contract
   - Normal operators should use Substrate entry points for doctor, gateway
     lifecycle/status, and validation flows now.
   - Warm/provision repair remains transitional, but later phases must cut over
     around the existing doctors and gateway commands instead of bypassing them.
   - Future docs and scripts must avoid telling users to repair normal operation by manually managing the guest first.

## Dependencies / sequencing

- Depends on milestone 0.1 target-mode decisions.
- Draws evidence from:
  - `scripts/mac/lima/substrate.yaml`
  - `scripts/mac/lima-warm.sh`
  - `scripts/mac/lima-doctor.sh`
  - `scripts/mac/smoke.sh`
  - `crates/world-mac-lima/src/forwarding.rs`
  - `crates/world-mac-lima/src/transport.rs`
  - `docs/WORLD.md`
  - `docs/cross-platform/mac_world_setup.md`
- Must complete before later phases can safely change operator docs or remove permissive transport fallbacks.

## Concrete repo surfaces and file pointers

- Lima capability and mount assumptions:
  - `scripts/mac/lima/substrate.yaml`
- Provisioning and health workflows:
  - `scripts/mac/lima-warm.sh`
  - `scripts/mac/lima-doctor.sh`
  - `scripts/mac/smoke.sh`
- Forwarding and endpoint mismatches:
  - `crates/world-mac-lima/src/forwarding.rs`
  - `crates/world-mac-lima/src/transport.rs`
  - `crates/world-mac-lima/src/lib.rs`
  - `crates/shell/src/execution/platform/macos.rs`
  - `crates/shell/src/builtins/world_gateway.rs`
- Gateway lifecycle/status contract already landed:
  - `docs/contracts/substrate-gateway-operator-contract.md`
  - `docs/contracts/substrate-gateway-status-schema.md`
  - `crates/world-service/src/gateway_runtime.rs`
- Operator guidance that must be reclassified:
  - `docs/WORLD.md`
  - `docs/cross-platform/mac_world_setup.md`

## Deliverables

- A version-floor decision section that later implementation can encode in docs, doctor checks, and release criteria.
- A breakglass matrix listing:
  - command or workflow
  - why it is breakglass
  - supported replacement path
  - evidence that later phases must provide before demoting manual steps
- A transport baseline statement covering default forwarding, guest TCP posture,
  endpoint consistency requirements, and the difference between the supported
  UDS-backed path versus retained host TCP fallback probes.

## Acceptance criteria

- The milestone defines the minimum supported environment in terms of required capabilities, not vague "recent Lima" wording.
- The milestone decides whether `vsock-proxy` is optional fallback infrastructure or a prerequisite for hardened mode.
- The milestone states that the hardened default does not expose a default guest
  TCP listener and that any retained TCP path is explicit breakglass or
  compatibility-only, not the supported Lima contract.
- The milestone identifies direct guest administration flows that must move out of normal setup and troubleshooting guidance.
- The milestone defines a replacement expectation that future Substrate-owned commands must cover:
  - doctor and status
  - gateway lifecycle/status
  - warm/provision
  - service restart
  - validation/smoke evidence
- The milestone names the remaining transport consistency gaps
  (`17788` compatibility probing and stale `7788` references) as explicit later
  implementation requirements, not incidental cleanup.
- The milestone classifies host-side `SUBSTRATE_WORLD_SOCKET` override use as
  advanced/test/breakglass rather than the default supported Lima path.

## Validation / evidence plan

- Review `scripts/mac/lima/substrate.yaml` and confirm the version contract
  accounts for every capability the profile already assumes.
- Review `crates/world-mac-lima/src/forwarding.rs`,
  `crates/world-mac-lima/src/transport.rs`,
  `crates/world-mac-lima/src/lib.rs`, and
  `crates/shell/src/execution/platform/macos.rs` and confirm the transport
  baseline captures the current UDS-backed story, stale `7788` references, and
  host TCP `17788` compatibility probing.
- Review `crates/shell/src/builtins/world_gateway.rs`,
  `crates/world-service/src/gateway_runtime.rs`, and
  `scripts/mac/smoke.sh` and confirm the SOW acknowledges the already-landed
  gateway lifecycle/status contract and smoke coverage.
- Review `docs/cross-platform/mac_world_setup.md` and `docs/WORLD.md` and
  enumerate every direct guest command or host-side override that should be
  labeled breakglass in later docs work.
- Require future implementation-phase plans to cite this milestone when changing:
  - default transport selection
  - TCP listener posture
  - mount exposure
  - operator troubleshooting guidance

## Risks / open questions

- If the hardened mode requires a narrower Lima feature set than current contributors use, the support matrix may tighten before the replacement workflows are fully built.
- Some manual `limactl` escape hatches may remain necessary longer than desired while Substrate-owned repair commands are still missing.
- The exact version floor still needs a concrete selection during milestone execution; until then, this SOW names the decision and evidence required but not the final version number.
