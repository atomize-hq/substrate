# Milestone 0.2: Lima Version and Breakglass Contract SOW

Status: Draft  
Last updated: 2026-04-28

## Purpose / outcome

Freeze the environment assumptions and operator escape hatches for hardened same-user Lima. The milestone outcome is a decision-ready contract that says which Lima/runtime capabilities are required for the supported mode, which manual workflows are retained only for breakglass, and which Substrate-owned commands must replace direct guest administration in normal operation.

## Why this milestone exists

Current macOS behavior still depends on tooling and workflows that are only partly under Substrate's control.

- `scripts/mac/lima/substrate.yaml` assumes `vmType: "vz"`, guest systemd, and a mount model that still exposes all of `$HOME` read-only.
- `scripts/mac/lima-warm.sh`, `scripts/mac/lima-doctor.sh`, and `scripts/mac/smoke.sh` still rely heavily on direct `limactl shell` execution.
- `crates/world-mac-lima/src/forwarding.rs` and `crates/world-mac-lima/src/transport.rs` encode transport assumptions that are not yet one hardened story.
- `docs/cross-platform/mac_world_setup.md` teaches direct guest build, install, service enablement, and raw guest probing as standard setup.

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
   - The contract must say whether `vsock-proxy` is required, optional, or unsupported for the hardened default.
2. Transport contract
   - Freeze one canonical forwarding story for supported mode.
   - Eliminate ambiguity between `17788` and `7788` as an implementation requirement for later phases.
   - State that guest raw TCP listeners such as `SUBSTRATE_AGENT_TCP_PORT=61337` are not part of the hardened default and require explicit breakglass treatment if kept at all.
3. Breakglass contract
   - Reclassify direct `limactl shell`, direct guest `systemctl`, direct guest binary install, and direct guest socket curls as breakglass workflows.
   - Require those workflows to be documented as emergency recovery or deep debugging only.
4. Substrate-owned control-plane contract
   - Normal operators should use Substrate entry points for warm, doctor, status, restart, and validation flows.
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
- A transport baseline statement covering default forwarding, guest TCP posture, and endpoint consistency requirements.

## Acceptance criteria

- The milestone defines the minimum supported environment in terms of required capabilities, not vague "recent Lima" wording.
- The milestone decides whether `vsock-proxy` is optional fallback infrastructure or a prerequisite for hardened mode.
- The milestone states that the hardened default does not expose a default guest TCP listener and that any retained TCP path is explicit breakglass.
- The milestone identifies direct guest administration flows that must move out of normal setup and troubleshooting guidance.
- The milestone defines a replacement expectation that future Substrate-owned commands must cover:
  - warm/provision
  - doctor and status
  - service restart
  - validation/smoke evidence
- The milestone names the transport consistency gap (`17788` vs `7788`) as an explicit later implementation requirement, not incidental cleanup.

## Validation / evidence plan

- Review `scripts/mac/lima/substrate.yaml` and confirm the version contract accounts for every capability the profile already assumes.
- Review `crates/world-mac-lima/src/forwarding.rs` and `crates/world-mac-lima/src/transport.rs` and confirm the transport baseline captures the current port and fallback inconsistencies.
- Review `docs/cross-platform/mac_world_setup.md` and `docs/WORLD.md` and enumerate every direct guest command that should be labeled breakglass in later docs work.
- Require future implementation-phase plans to cite this milestone when changing:
  - default transport selection
  - TCP listener posture
  - mount exposure
  - operator troubleshooting guidance

## Risks / open questions

- If the hardened mode requires a narrower Lima feature set than current contributors use, the support matrix may tighten before the replacement workflows are fully built.
- Some manual `limactl` escape hatches may remain necessary longer than desired while Substrate-owned repair commands are still missing.
- The exact version floor still needs a concrete selection during milestone execution; until then, this SOW names the decision and evidence required but not the final version number.
