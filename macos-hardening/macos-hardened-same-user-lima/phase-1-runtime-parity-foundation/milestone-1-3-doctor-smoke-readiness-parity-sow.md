# Milestone 1.3: Doctor / Smoke Readiness Parity SOW

Status: Draft

Owner: Substrate operator UX / macOS validation surfaces

Last updated: 2026-05-19

## Purpose / Outcome

Make the macOS readiness story prove the same routed Substrate contract that
Linux relies on, and stop treating direct guest administration as the normal
path for setup, diagnosis, and smoke validation.

The concrete outcome is not inventing doctors or gateway lifecycle from
scratch. Those surfaces already exist. The outcome is to make existing CLI
doctors, gateway lifecycle/status, and macOS smoke validation the authoritative
readiness evidence, while direct `limactl shell` commands become explicitly
breakglass-only diagnostics.

## Why This Milestone Exists

Current readiness surfaces are partly landed, but the helpers and docs still
normalize direct guest entry.

- `substrate host doctor` and `substrate world doctor` already exist and are
  canonical CLI readiness surfaces.
- `substrate world gateway sync|status|restart` and
  `substrate world gateway status --json` already exist as canonical gateway
  lifecycle/status surfaces.
- `scripts/mac/smoke.sh` already includes gateway lifecycle smoke coverage.
- `scripts/mac/lima-doctor.sh` still shells into the guest for core health
  checks, and `scripts/mac/smoke.sh` still mixes routed checks with direct
  guest probes.
- `crates/shell/src/execution/platform/macos.rs` still falls back from host UDS
  to host TCP `17788` or in-VM probing when collecting doctor evidence.
- `docs/WORLD.md` and `docs/cross-platform/mac_world_setup.md` still present
  `limactl shell`, in-guest `curl`, guest `systemctl`, and direct guest logs as
  normal operator behavior.

That posture undermines the same-user hardening story and also weakens parity:
it can prove that the guest is reachable, not that Substrate’s routed path is
healthy.

## In Scope

- Reframe `substrate host doctor` and `substrate world doctor` as the canonical
  readiness interfaces for macOS.
- Reframe `substrate world gateway sync|status|restart` and status JSON as
  canonical readiness/support surfaces for managed gateway lifecycle.
- Align `scripts/mac/lima-doctor.sh` and `scripts/mac/smoke.sh` to validate the
  routed Substrate path first.
- Update readiness-oriented doc sections so direct guest commands are marked as
  breakglass/unsupported for readiness validation rather than the happy path.
- Ensure readiness evidence covers transport selection, policy application, and
  PTY/non-PTY routed behavior.

## Out of Scope

- Removing every internal `limactl shell` use from helper scripts.
- Redesigning the provisioning flow or ownership boundary.
- Full same-user hardening of mounts, listeners, or service accounts.
- Building a dedicated breakglass subsystem beyond doc classification and
  command-path discipline.

## Architectural Approach

This milestone should make the readiness stack flow in layers:

1. Routed CLI checks first:
   - `substrate host doctor`
   - `substrate world doctor`
   - `substrate world gateway sync`
   - `substrate world gateway status --json`
   - `substrate world gateway restart`
   - routed `substrate --world` smoke operations
2. Script wrappers second:
   - `scripts/mac/lima-doctor.sh`
   - `scripts/mac/smoke.sh`
3. Breakglass guest introspection only when the routed path fails and deeper
   guest diagnosis is needed

The docs should match that order. Setup can still mention that Lima exists, but
the operator contract should prefer Substrate-owned commands and explain direct
guest entry as a breakglass-only diagnostic path, not as the default workflow
or a degraded-but-supported middle tier.

Host-side `SUBSTRATE_WORLD_SOCKET` override use is outside the normal Lima path
for this milestone. If docs mention it at all, it should be classified as
advanced/test/breakglass.

## Dependencies / Sequencing

- Depends on Milestone 1.1 so the doctor/smoke surfaces prove one transport
  contract.
- Depends on Milestone 1.2 so the readiness flows prove backend policy parity
  rather than raw guest reachability.
- This is the phase signoff milestone for runtime parity foundation.

## Concrete Repo Surfaces and File Pointers

Primary readiness and docs surfaces:

- `crates/shell/src/execution/platform/macos.rs`
- `crates/shell/src/builtins/world_gateway.rs`
- `docs/contracts/substrate-gateway-operator-contract.md`
- `docs/contracts/substrate-gateway-status-schema.md`
- `scripts/mac/lima-doctor.sh`
- `scripts/mac/smoke.sh`
- `scripts/mac/lima-warm.sh`
- `docs/WORLD.md`
- `docs/cross-platform/mac_world_setup.md`

Current normalization of direct guest administration:

- `docs/cross-platform/mac_world_setup.md`
- `scripts/mac/lima-doctor.sh`
- `scripts/mac/smoke.sh`
- `scripts/mac/lima-warm.sh`

## Deliverables

- Updated doctor/reporting behavior or output fields needed to make routed
  transport and policy state visible.
- A macOS smoke path that asserts routed PTY and non-PTY behavior before any
  guest-direct checks.
- A readiness story that treats gateway lifecycle/status as a normal support
  surface alongside doctor JSON.
- Updated readiness-oriented excerpts in macOS setup and world docs that
  clearly separate routed readiness validation from breakglass diagnostics.
- A validation matrix for operator evidence capture on macOS that mirrors Linux
  intent even when host ownership differs.

## Acceptance Criteria

- The happy-path macOS validation docs for an already provisioned backend use
  routed Substrate readiness commands first and do not require direct guest
  `curl` or `systemctl` for routine verification.
- The happy-path docs also lead with `substrate world gateway sync|status|restart`
  for managed gateway lifecycle verification instead of raw guest probing.
- `scripts/mac/lima-doctor.sh` fails when the routed Substrate path is unhealthy
  even if direct in-guest probing still succeeds.
- `scripts/mac/smoke.sh` proves routed PTY and non-PTY behavior plus doctor
  readiness against the canonical transport contract while preserving the
  already-landed gateway lifecycle smoke proof.
- Direct guest commands remain documented only as breakglass/unsupported
  procedures.
- The docs explicitly state that same-user Lima still does not provide the Linux
  ownership boundary, even after readiness parity is achieved.
- Host-side `SUBSTRATE_WORLD_SOCKET` override use is not documented as the
  default Lima path.

## Validation / Evidence Plan

Required evidence for this milestone:

- `substrate host doctor --json`
- `substrate world doctor --json`
- `substrate world gateway sync`
- `substrate world gateway status --json`
- `substrate world gateway restart`
- `scripts/mac/lima-doctor.sh`
- `scripts/mac/smoke.sh`
- updated doc excerpts in the implementation PR showing the happy-path command
  flow for readiness validation, not full lifecycle ownership

Evidence should demonstrate that a user can validate and smoke-test an already
provisioned macOS backend through routed Substrate doctor and gateway commands
first, with direct guest commands only used for deeper post-failure diagnosis.
Full lifecycle ownership and doc cutover remain Phase 3 scope.

## Risks / Open Questions

- Some bootstrap failures may be impossible to diagnose without direct guest
  commands. The docs must preserve that escape hatch without letting it define
  the standard workflow.
- If doctor/reporting needs new transport or policy-parity fields, the CLI JSON
  schema may require deliberate extension.
- Setup documentation may need careful wording so developers can still perform
  guest-only repair tasks without interpreting them as part of the normal trust
  boundary.
