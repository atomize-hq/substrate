# Milestone 1.3: Doctor / Smoke Readiness Parity SOW

Status: Draft

Owner: Substrate operator UX / macOS validation surfaces

Last updated: 2026-04-28

## Purpose / Outcome

Make the macOS readiness story prove the same routed Substrate contract that
Linux relies on, and stop treating direct guest administration as the normal
path for setup, diagnosis, and smoke validation.

The concrete outcome is that CLI doctors and macOS smoke validation become the
authoritative readiness evidence, while direct `limactl shell` commands are
explicitly breakglass-only diagnostics.

## Why This Milestone Exists

Current readiness and setup surfaces still normalize direct guest entry:

- [scripts/mac/lima-doctor.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-doctor.sh)
  shells into the guest for core health checks.
- [scripts/mac/smoke.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh)
  mixes routed Substrate checks with direct guest probes.
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md:148)
  and
  [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md:85)
  still present `limactl shell`, in-guest `curl`, guest `systemctl`, and direct
  guest logs as normal operator behavior.

That posture undermines the same-user hardening story and also weakens parity:
it can prove that the guest is reachable, not that Substrate’s routed path is
healthy.

## In Scope

- Reframe `substrate host doctor` and `substrate world doctor` as the canonical
  readiness interfaces for macOS.
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

## Dependencies / Sequencing

- Depends on Milestone 1.1 so the doctor/smoke surfaces prove one transport
  contract.
- Depends on Milestone 1.2 so the readiness flows prove backend policy parity
  rather than raw guest reachability.
- This is the phase signoff milestone for runtime parity foundation.

## Concrete Repo Surfaces and File Pointers

Primary readiness and docs surfaces:

- [crates/shell/src/execution/platform/macos.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/platform/macos.rs)
- [scripts/mac/lima-doctor.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-doctor.sh)
- [scripts/mac/smoke.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh)
- [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh)
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
- [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md)

Current normalization of direct guest administration:

- [docs/cross-platform/mac_world_setup.md:85](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md:85)
- [docs/cross-platform/mac_world_setup.md:121](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md:121)
- [docs/cross-platform/mac_world_setup.md:141](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md:141)
- [scripts/mac/lima-doctor.sh:62](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-doctor.sh:62)
- [scripts/mac/smoke.sh:155](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh:155)
- [scripts/mac/lima-warm.sh:788](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh:788)

## Deliverables

- Updated doctor/reporting behavior or output fields needed to make routed
  transport and policy state visible.
- A macOS smoke path that asserts routed PTY and non-PTY behavior before any
  guest-direct checks.
- Updated readiness-oriented excerpts in macOS setup and world docs that
  clearly separate routed readiness validation from breakglass diagnostics.
- A validation matrix for operator evidence capture on macOS that mirrors Linux
  intent even when host ownership differs.

## Acceptance Criteria

- The happy-path macOS validation docs for an already provisioned backend use
  routed Substrate readiness commands first and do not require direct guest
  `curl` or `systemctl` for routine verification.
- `scripts/mac/lima-doctor.sh` fails when the routed Substrate path is unhealthy
  even if direct in-guest probing still succeeds.
- `scripts/mac/smoke.sh` proves routed PTY and non-PTY behavior plus doctor
  readiness against the canonical transport contract.
- Direct guest commands remain documented only as breakglass/unsupported
  procedures.
- The docs explicitly state that same-user Lima still does not provide the Linux
  ownership boundary, even after readiness parity is achieved.

## Validation / Evidence Plan

Required evidence for this milestone:

- `substrate host doctor --json`
- `substrate world doctor --json`
- `scripts/mac/lima-doctor.sh`
- `scripts/mac/smoke.sh`
- updated doc excerpts in the implementation PR showing the happy-path command
  flow for readiness validation, not full lifecycle ownership

Evidence should demonstrate that a user can validate and smoke-test an already
provisioned macOS backend through routed Substrate commands first, with direct
guest commands only used for deeper post-failure diagnosis. Full lifecycle
ownership and doc cutover remain Phase 3 scope.

## Risks / Open Questions

- Some bootstrap failures may be impossible to diagnose without direct guest
  commands. The docs must preserve that escape hatch without letting it define
  the standard workflow.
- If doctor/reporting needs new transport or policy-parity fields, the CLI JSON
  schema may require deliberate extension.
- Setup documentation may need careful wording so developers can still perform
  guest-only repair tasks without interpreting them as part of the normal trust
  boundary.
