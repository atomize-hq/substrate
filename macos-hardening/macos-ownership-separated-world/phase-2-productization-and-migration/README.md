# Phase 2: Productization and Migration

Status: Draft

Owner: macOS world backend / installer / operator UX

Last updated: 2026-04-28

## Purpose / Outcome

Phase 2 turns the ownership-separated macOS world design into a shippable
operator workflow. The outcome is not “a secure architecture sketch,” but a
macOS release path that can install, upgrade, migrate, diagnose, support, and
validate the ownership-separated backend without telling normal operators to use
direct `limactl shell`, direct guest `systemctl`, or direct guest `curl`.

At the end of this phase, a supported macOS user should be able to:

- install Substrate onto a clean machine
- upgrade from an older ownership-separated release
- migrate from the current same-user Lima model
- inspect health and status from canonical Substrate CLI surfaces
- collect support evidence and perform bounded repair
- remove Substrate cleanly
- prove GA readiness through a published validation matrix

## Why This Phase Exists

Phase 1 establishes the hard boundary: dedicated host-side ownership, private
`LIMA_HOME`, and a Substrate-owned broker path. That is necessary, but not
sufficient for productization.

The current repo still assumes same-user Lima operations in several places:

- install/uninstall scripts operate Lima as the invoking user
- docs instruct direct guest entry as a normal path
- doctor and smoke scripts prove readiness by shelling into the guest directly
- macOS status reporting is fragmented across shell code, helper scripts, and
  doc-only troubleshooting instructions

If those surfaces are not rewritten, the product still behaves like a developer
tooling prototype even if the runtime boundary becomes stronger underneath.

## In Scope

- macOS install, upgrade, migration, rollback, and uninstall workflows for the
  ownership-separated backend
- migration from same-user Lima state into the new control-plane owner model
- canonical doctor and status surfaces for host, world, and support workflows
- operator-facing supportability flows: log collection, repair guidance, state
  reporting, and breakglass boundaries
- a GA validation matrix covering install lifecycle, runtime lifecycle, and
  failure-mode evidence
- documentation updates required to make the new path the default operator
  contract

## Out of Scope

- defining the host ownership-separation architecture itself
- same-user Lima hardening work that belongs to the sibling track
- introducing new product scope unrelated to macOS world lifecycle
- Linux or Windows operator-contract redesign except where parity evidence is
  needed to explain macOS behavior
- open-ended breakglass tooling that bypasses the new boundary as a normal path

## Architectural Approach

Phase 2 standardizes around one operator contract:

1. Host lifecycle is owned by Substrate-managed install and runtime surfaces,
   not by user-owned Lima state.
2. CLI status and doctor commands are the primary support interface.
3. Docs, smoke coverage, and uninstall behavior follow the same contract.
4. Direct guest access becomes breakglass-only, explicitly segregated from the
   normal install and support path.

The implementation threads through existing repo surfaces rather than inventing
parallel tooling:

- installer/uninstaller entrypoints remain under `scripts/substrate/`
- macOS warm/doctor/smoke helpers remain under `scripts/mac/`, but must become
  contract followers rather than contract owners
- CLI doctor/status behavior remains rooted in
  `crates/shell/src/execution/platform/macos.rs`
- operator docs remain rooted in `docs/INSTALLATION.md`, `docs/UNINSTALL.md`,
  `docs/WORLD.md`, and `docs/cross-platform/mac_world_setup.md`

## Dependencies / Sequencing

Hard dependencies:

- The same-user hardening track should already have converged the transport,
  mount, and operator vocabulary so migration into ownership separation does not
  inherit ambiguous same-user workflows.
- More concretely, phase 2 assumes the same-user track has completed:
  - phase 2 milestone 2.2 mount minimization and ingress contract
  - phase 3 milestone 3.1 Substrate-managed diagnostics and lifecycle
  - phase 3 milestone 3.2 breakglass reclassification and doc cutover
- Phase 1 must already provide a dedicated macOS owner model, private
  `LIMA_HOME`, and a Substrate-owned broker path.
- The normal-path transport contract must already stop exposing same-user
  forwarded endpoints as the security boundary.

Execution order inside Phase 2:

1. Milestone 2.1 defines how the product gets onto the machine and how legacy
   same-user installs are migrated safely.
2. Milestone 2.2 makes that lifecycle operable through canonical status,
   doctor, and support surfaces.
3. Milestone 2.3 locks the release gate by defining and executing the GA
   validation matrix across clean install, upgrade, migration, repair, and
   uninstall scenarios.

## Concrete Repo Surfaces and File Pointers

Primary implementation and contract surfaces for this phase:

- `scripts/substrate/install-substrate.sh`
- `scripts/substrate/uninstall-substrate.sh`
- `scripts/substrate/dev-uninstall-substrate.sh`
- `scripts/mac/lima-warm.sh`
- `scripts/mac/lima-doctor.sh`
- `scripts/mac/smoke.sh`
- `crates/shell/src/execution/platform/macos.rs`
- `docs/INSTALLATION.md`
- `docs/UNINSTALL.md`
- `docs/WORLD.md`
- `docs/cross-platform/mac_world_setup.md`

Secondary reference surfaces likely touched by downstream execution:

- `scripts/substrate/install.sh`
- `scripts/substrate/uninstall.sh`
- `scripts/mac/lima/substrate.yaml`
- `scripts/mac/lima/substrate-dev.yaml`
- `crates/world-mac-lima/src/lib.rs`
- `crates/world-mac-lima/src/forwarding.rs`
- `crates/world-mac-lima/src/transport.rs`

## Milestones

### Milestone 2.1

[`milestone-2-1-installer-upgrade-and-migration-path-sow.md`](./milestone-2-1-installer-upgrade-and-migration-path-sow.md)

Define and land the supported install/upgrade/uninstall/migration path,
including same-user Lima migration and rollback posture.

### Milestone 2.2

[`milestone-2-2-operator-doctor-status-and-supportability-sow.md`](./milestone-2-2-operator-doctor-status-and-supportability-sow.md)

Define and land the canonical operator surfaces for health, status, repair
guidance, evidence collection, and support escalation.

### Milestone 2.3

[`milestone-2-3-ga-readiness-and-validation-matrix-sow.md`](./milestone-2-3-ga-readiness-and-validation-matrix-sow.md)

Define and execute the GA gate for macOS ownership separation, including the
validation matrix, evidence packaging, and release-blocking criteria.

## Deliverables

- one phase overview that fixes the productization scope and sequence
- three milestone SOWs with concrete repo surfaces, deliverables, and gates
- an explicit migration story from same-user Lima to ownership-separated macOS
- an explicit operator contract that replaces direct guest workflows
- a GA evidence plan that can be executed and audited before release

## Acceptance Criteria

- The phase docs describe one consistent normal-path operator contract across
  install, health checks, troubleshooting, and uninstall.
- Same-user Lima migration is treated as a first-class lifecycle path, not a
  footnote.
- Each milestone names concrete repo surfaces instead of generic workstreams.
- Phase sequencing is explicit enough that execution can proceed milestone by
  milestone without reopening the scope question.
- The phase does not rely on direct guest access as part of the normal support
  posture.

## Validation / Evidence Plan

- Use the repo-backed evidence in
  `thoughts/shared/research/2026-04-28-macos-lima-parity-lockdown.md` as the
  baseline problem statement.
- For milestone closeout, require install/uninstall transcripts, doctor/status
  JSON, smoke artifacts, and doc diffs from the concrete repo surfaces named in
  each SOW.
- Treat migration from an existing same-user Lima installation as mandatory
  evidence, not optional beta validation.
- Treat reboot, relogin, re-upgrade, and uninstall cleanup as mandatory
  evidence before GA sign-off.

## Risks / Open Questions

- The migration step may require careful transfer or recreation of Lima-backed
  assets that are currently user-owned under `~/.lima` and `~/.substrate`.
- Secure ownership transfer may require privilege or OS-registration steps that
  are awkward in noninteractive installer flows.
- Existing docs and helper scripts are deeply same-user oriented; replacing
  their assumptions without creating support regressions will require tight
  cross-document review.
- If the ownership-separated backend changes what host paths are visible to the
  guest, migration and smoke flows may expose latent ingress assumptions not yet
  modeled in today’s docs or scripts.
