# macOS Ownership-Separated World

Status: Proposed

Owner: Substrate world backend / macOS control plane

Last updated: 2026-05-19

## Purpose / Outcome

Define and sequence the work required to make the macOS Lima-backed world
meaningfully Substrate-controlled instead of developer-user-controlled. The
target end state is a dedicated Substrate-owned host control plane that owns
`LIMA_HOME`, VM state, credentials, forwarding, and the only routine
host-reachable broker endpoint for both world execution and gateway lifecycle.

## Why This Feature Exists

The current macOS backend already gives Substrate a same-user brokered path for
execution, doctoring, and gateway lifecycle, but it still does not give
Substrate exclusive control of the VM. Today the same logged-in developer user
can still:

- read and mutate Lima state under the default user-owned `LIMA_HOME`
- invoke `limactl shell`
- use direct SSH-based transport paths
- reach the guest socket and services through host-local forwarding
- exercise `substrate world gateway sync|status|restart` over a path that still
  depends on same-user Lima ownership and forwarding setup

That is sufficient for convenience and parity, but it is not sufficient for the
security claim that only Substrate controls the macOS world. This planning set
exists to convert the macOS backend from wrapper-hardening to host ownership
separation while preserving the already-landed operator surfaces.

## In-Scope

- A host ownership model where a dedicated daemon or service account owns Lima
  state and lifecycle.
- A control-plane split where developer-facing CLI code no longer owns or
  directly manipulates host-side VM artifacts.
- Replacement of direct host-local guest reachability with one
  Substrate-owned broker endpoint and authenticated broker-to-agent traffic.
- Extension of ownership separation to the current gateway lifecycle path so
  `substrate world gateway sync|status|restart` no longer depend on
  developer-owned forwarding.
- A breakglass path for maintenance that is explicit, auditable, and disabled
  for normal runtime use.
- Repo planning for phase 0 architecture/contracts, phase 1 prototype
  implementation, and phase 2 productization and migration.

## Out-of-Scope

- Full productionization of macOS service packaging, notarization, or MDM
  deployment.
- Replacing the Linux guest or the in-guest `world-service` API itself.
- Redesigning Linux or Windows world ownership models in this workstream.
- Broad product work unrelated to the macOS control-plane ownership boundary.

## Architectural Approach

The feature is delivered in three phases:

1. Phase 0 defines the contracts. It locks the host ownership model, the daemon
   responsibilities, the broker/auth boundary, the gateway lifecycle/status
   owner lines, and the breakglass rules so implementation work does not
   preserve same-user escape hatches by accident.
2. Phase 1 builds a working prototype. It moves `LIMA_HOME` and instance
   lifecycle under daemon ownership, changes the forwarding model to a private
   daemon-managed channel, exposes one developer-reachable broker endpoint, and
   carries existing world and gateway lifecycle traffic over that path.
3. Phase 2 productizes the model. It covers installer, upgrade, migration,
   doctor, supportability, and GA validation so the stronger ownership boundary
   becomes an operable product contract instead of a prototype.

The intent is not to incrementally harden the existing `limactl` and SSH flows
in place. The intent is to remove them from the standard runtime path.

## Dependencies / Sequencing

- This track depends on the same-user hardening track establishing the cleaned
  up transport contract, policy semantics, mount posture, gateway runtime
  parity, and Substrate-owned operational vocabulary that ownership separation
  will build on rather than duplicate.
- At minimum, ownership-separated phase 1 assumes the same-user track has
  completed phase 1 milestone 1.1 through milestone 1.3 plus phase 3 milestone
  3.2, so transport, policy, gateway lifecycle vocabulary, and operator-state
  terminology are already normalized.
- Ownership-separated phase 2 assumes the same-user track has also completed
  phase 2 milestone 2.2 and phase 3 milestone 3.1, so ingress/sync and
  Substrate-owned lifecycle surfaces are available for migration planning.
- Phase 0 must complete before phase 1 implementation starts.
- Milestone 0.1 must land before milestone 0.2 because the broker/auth
  contract depends on the daemon ownership and lifecycle contract.
- Milestone 1.1 must land before milestone 1.2 because the private forwarding
  model depends on daemon-owned VM state and credentials.
- Milestone 1.3 depends on 1.1 and 1.2 because breakglass policy is defined
  after routine runtime paths are removed.
- Phase 2 depends on the phase 1 prototype proving that daemon-owned state,
  private forwarding, and the broker boundary can replace the current same-user
  runtime path for both execution and gateway lifecycle.

## Concrete Repo Surfaces and File Pointers

- `crates/world-mac-lima/src/lib.rs`
  Current host-side lifecycle ownership, VM ensure/start logic, agent
  reachability checks, and endpoint selection.
- `crates/world-mac-lima/src/forwarding.rs`
  Current direct VSock/SSH forwarding selection and host-visible socket creation
  under `~/.substrate/sock/agent.sock`.
- `crates/world-mac-lima/src/transport.rs`
  Current transport assumptions that treat VSock and SSH as directly
  host-usable options.
- `crates/shell/src/builtins/world_gateway.rs`
  Current macOS gateway lifecycle/status client that still rides the same
  Lima-backed forwarded path.
- `crates/world-service/src/gateway_runtime.rs`
  Current managed gateway runtime root, lifecycle execution, and auth-bundle
  handoff inside the guest.
- `scripts/mac/lima-warm.sh`
  Current same-user VM creation, guest provisioning, socket activation, and
  guest artifact ownership assumptions.
- `scripts/mac/lima/substrate.yaml`
  Current Lima instance definition and guest-side service/socket configuration.
- `scripts/mac/smoke.sh`
  Current macOS proof harness, including gateway lifecycle proof.
- `scripts/substrate/world-enable.sh`
  Current post-install world provisioning entrypoint that assumes the CLI can
  provision the macOS world directly.
- `scripts/substrate/dev-install-substrate.sh`
  Current same-user dev lifecycle entrypoint that stages guest binaries and
  writes world metadata.
- `scripts/substrate/install-substrate.sh`
  Current installer path that provisions the macOS world by calling
  `scripts/mac/lima-warm.sh` and verifies `substrate-gateway` in the guest.
- `scripts/substrate/uninstall-substrate.sh`
  Current teardown path that deletes the Lima VM and the host-forwarded socket.
- `../../docs/WORLD.md`
  Descriptive architecture evidence for current transport and runtime posture.
- `../../docs/contracts/substrate-gateway-operator-contract.md`
  Durable owner of the gateway lifecycle command family.
- `../../docs/contracts/substrate-gateway-policy-evaluation.md`
  Durable owner of the integrated auth and policy-evaluation boundary.
- `../../docs/contracts/substrate-gateway-status-schema.md`
  Durable owner of `substrate world gateway status --json`.

## Deliverables

- Feature overview and phase breakdown for ownership-separated macOS world
  control.
- Phase 0 milestone SOWs covering the host owner model and broker/auth
  boundary.
- Phase 1 milestone SOWs covering daemon-owned Lima state, private forwarding,
  and breakglass maintenance.
- Phase 2 milestone SOWs covering installer and migration, operator
  supportability, and GA validation.
- Clear acceptance and evidence expectations that future implementation PRs can
  execute against.

## Acceptance Criteria

- The planning set states an end state where the developer user no longer owns
  routine Lima control-plane artifacts.
- The planning set makes direct `limactl shell`, direct SSH, guest
  `systemctl`, and guest `curl` breakglass only.
- The planning set preserves the current `substrate host doctor`,
  `substrate world doctor`, `substrate health`, and
  `substrate world gateway sync|status|restart` operator surfaces, but moves
  them behind a new ownership model.
- `docs/WORLD.md` is treated as descriptive evidence, not the owner of operator
  or status semantics.
- Each phase and milestone has concrete repo surfaces, deliverables, validation
  evidence, and dependency ordering.

## Validation / Evidence Plan

- Architecture reviews for phase 0 must produce explicit control-flow diagrams
  and contract deltas against current repo surfaces.
- Prototype work in phase 1 must demonstrate daemon-owned state paths, removal
  of developer-visible forwarded guest sockets from the default path, and
  broker-mediated execution plus gateway lifecycle calls.
- Operator docs and installer flows must be mapped to the new architecture
  before implementation is considered complete.
- Validation evidence should include:
  - `substrate host doctor --json`
  - `substrate world doctor --json`
  - `substrate health --json`
  - `substrate world gateway status --json`

## Risks / Open Questions

- macOS service-account mechanics need a final decision between a root-owned
  `launchd` daemon and an app-managed service installed via `SMAppService`.
- Lima may still require some root or user-session integration details that
  change how service ownership is packaged.
- Broker-to-agent attestation needs a concrete trust anchor that works with
  Lima and survives VM recreation.
- Migration from existing user-owned `~/.lima/substrate` installs may require a
  destructive or semi-destructive conversion path.

## Phase Index

- [Phase 0: Architecture and Host Ownership Model](./phase-0-architecture-and-host-ownership-model/README.md)
- [Phase 1: Control Plane Separation Prototype](./phase-1-control-plane-separation-prototype/README.md)
- [Phase 2: Productization and Migration](./phase-2-productization-and-migration/README.md)
