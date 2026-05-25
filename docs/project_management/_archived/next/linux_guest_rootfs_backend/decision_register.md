# linux_guest_rootfs_backend — Decision Register

Status: draft decisions accepted for ADR-0009 alignment.

## DR-0001 — Linux guest image format and builder

**Decision owner(s):** Shell / World / Installer maintainers  
**Date:** 2026-05-24  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`

**Problem / Context**
- Linux needs a lightweight guest userspace that is not coupled to the host distro.
- The first shipped guest image should be Ubuntu/Debian-family, but the backend contract must stay distro-flexible.

**Option A — Build the guest rootfs with `debootstrap`**
- **Pros:** straightforward Ubuntu-first bootstrap; apt-native.
- **Cons:** host-side tooling is Debian-shaped; weak fit for non-Debian hosts; future distro expansion becomes awkward.
- **Cascading implications:** image creation logic would be implicitly tied to one distro family.
- **Risks:** architectural overfitting to Ubuntu as the backend concept rather than the first shipped image.
- **Unlocks:** quick first prototype for Ubuntu-only hosts.
- **Quick wins / low-hanging fruit:** easy first implementation if host distro alignment is acceptable.

**Option B — OCI-style rootfs unpack**
- **Pros:** host-distro agnostic; fits the goal of decoupling guest distro from host distro; preserves future image flexibility.
- **Cons:** slightly more image acquisition/unpack machinery in first ship.
- **Cascading implications:** image identity and unpack lifecycle become first-class backend concerns.
- **Risks:** more upfront work around provenance and unpack flow.
- **Unlocks:** Ubuntu-first without baking Ubuntu into the backend abstraction.
- **Quick wins / low-hanging fruit:** first ship can still bless one Ubuntu/Debian image while keeping the contract general.

**Recommendation**
- **Selected:** Option B — OCI-style rootfs unpack
- **Rationale (crisp):** the backend exists to decouple world distro from host distro; `debootstrap` would hardcode the wrong abstraction too early.

**Follow-up tasks (explicit)**
- Define the blessed Ubuntu/Debian image source and unpack procedure in planning specs.
- Specify immutable base-image storage and verification requirements.
- Add smoke/manual validation that a non-Debian host can run the Ubuntu/Debian guest world.

## DR-0002 — Guest rootfs persistence model

**Decision owner(s):** Shell / World maintainers  
**Date:** 2026-05-24  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`

**Problem / Context**
- Provisioning only adds value if installed guest packages remain available across later executions.
- The persistence model also determines whether workspaces can contaminate one another.

**Option A — Immutable base image plus persistent writable overlay**
- **Pros:** keeps image base immutable; preserves apt-installed state; avoids cross-workspace mutation of the shared base.
- **Cons:** overlay lifecycle and cleanup are more complex.
- **Cascading implications:** planning must define overlay keying, persistence, and repair rules.
- **Risks:** more moving pieces in readiness checks and cleanup.
- **Unlocks:** cleaner future image pinning and reproducibility.
- **Quick wins / low-hanging fruit:** provisioning and repair can target the same persistent overlay path.

**Option B — One persistent mutable rootfs per image**
- **Pros:** simpler initial mental model; fewer storage layers.
- **Cons:** image base drifts over time; cross-workspace package bleed is likely; reproducibility degrades.
- **Cascading implications:** future image pinning or reset semantics become much messier.
- **Risks:** mutable global guest state becomes hard to reason about and debug.
- **Unlocks:** lower first-pass implementation complexity only.
- **Quick wins / low-hanging fruit:** minimal storage bookkeeping.

**Recommendation**
- **Selected:** Option A — Immutable base image plus persistent writable overlay
- **Rationale (crisp):** persistence is required, but a mutable global rootfs would undercut the reproducibility and safety story too early.

**Follow-up tasks (explicit)**
- Define overlay keying and lifecycle in the planning specs.
- Define what counts as repair vs reset for the persistent overlay.
- Add tests proving apt-installed packages persist while the base image remains immutable.

## DR-0003 — Storage location and ownership

**Decision owner(s):** Shell / World / Installer maintainers  
**Date:** 2026-05-24  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`

**Problem / Context**
- Guest images and overlays are backend infrastructure, not per-workspace artifacts.
- Storage location determines privilege posture, cleanup semantics, and multi-user behavior.

**Option A — System-owned Substrate storage under `/var/lib/substrate/...`**
- **Pros:** strong ownership boundary; clearer multi-user semantics; naturally stays out of workspaces and `$SUBSTRATE_HOME`.
- **Cons:** warm/repair requires explicit privilege use.
- **Cascading implications:** warm script and doctor/remediation must acknowledge privileged setup.
- **Risks:** slightly higher operator friction for initial warm flow.
- **Unlocks:** consistent backend storage posture with current Linux world-service/systemd model.
- **Quick wins / low-hanging fruit:** easier to reason about cleanup, ownership, and access boundaries.

**Option B — User-scoped storage under `$SUBSTRATE_HOME/...`**
- **Pros:** lower privilege friction.
- **Cons:** blurs backend vs user-cache ownership; weaker multi-user story; more duplication.
- **Cascading implications:** backend infrastructure would be mixed into user state.
- **Risks:** accidental drift into workspace/user-managed paths and weaker operational boundaries.
- **Unlocks:** easier local-only experimentation.
- **Quick wins / low-hanging fruit:** fewer privileged steps.

**Recommendation**
- **Selected:** Option A — System-owned Substrate storage under `/var/lib/substrate/...`
- **Rationale (crisp):** this is backend infrastructure, not a user convenience cache; it needs a system-owned boundary.

**Follow-up tasks (explicit)**
- Define exact directory layout under `/var/lib/substrate/...`.
- Add safety checks that reject workspace or `$SUBSTRATE_HOME` placement.
- Document required ownership and repair expectations in the warm playbook.

## DR-0004 — Warm command surface

**Decision owner(s):** Shell / Installer maintainers  
**Date:** 2026-05-24  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`

**Problem / Context**
- Operators need an explicit way to create and repair the guest rootfs/image store.
- The surface should match current Substrate backend bootstrap posture without freezing a premature CLI.

**Option A — New first-class CLI command**
- **Pros:** polished discoverability; future-friendly if Substrate grows a full image-management platform.
- **Cons:** commits a new CLI surface too early; increases current contract and docs scope.
- **Cascading implications:** CLI semantics and cross-platform parity would need to be locked now.
- **Risks:** the first command surface may ossify before the backend lifecycle is understood.
- **Unlocks:** eventual unified world-image UX.
- **Quick wins / low-hanging fruit:** none beyond discoverability.

**Option B — Script-first warm flow**
- **Pros:** matches the existing `scripts/mac/lima-warm.sh` pattern; keeps first ship operationally explicit; avoids premature CLI expansion.
- **Cons:** less elegant than a CLI surface; discoverability depends on doctor/remediation messaging.
- **Cascading implications:** warm/remediation text becomes part of the operator contract.
- **Risks:** script-first flows can feel less integrated if docs are weak.
- **Unlocks:** lets the backend lifecycle settle before freezing a CLI.
- **Quick wins / low-hanging fruit:** fastest alignment with existing Substrate backend bootstrap patterns.

**Recommendation**
- **Selected:** Option B — Script-first warm flow
- **Rationale (crisp):** the rootfs warm path is backend bootstrap, not a second provisioning UX; script-first matches current Substrate patterns and keeps the CLI surface smaller in v1.

**Follow-up tasks (explicit)**
- Create `scripts/linux/world-rootfs-warm.sh`.
- Ensure doctor and provisioning remediation point to that script.
- Define idempotent repair behavior and explicit privilege use in specs/playbooks.

## DR-0005 — Readiness and doctor surface

**Decision owner(s):** Shell / World maintainers  
**Date:** 2026-05-24  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`, `docs/BACKLOG.md`

**Problem / Context**
- Operators and planning need one public source of truth for whether guest-rootfs is selected, what image is active, and whether provisioning is actually possible.

**Option A — Doctor-backed readiness contract**
- **Pros:** one shared truth for operators, planning, and runtime remediation; aligns with the backlog need for world OS identity in doctor output.
- **Cons:** requires additive doctor schema work.
- **Cascading implications:** runtime provisioning and error handling should reuse the same readiness logic.
- **Risks:** doctor fields must stay stable once published.
- **Unlocks:** better troubleshooting and planning clarity.
- **Quick wins / low-hanging fruit:** one clear operator check before provisioning or execution.

**Option B — On-demand probing only**
- **Pros:** smaller initial surface; fewer immediately documented fields.
- **Cons:** poor observability; failures only appear at execution time; hidden duplicate readiness logic is likely.
- **Cascading implications:** provisioning and execution paths each need their own probe/remediation flow.
- **Risks:** operational confusion and inconsistent failure reporting.
- **Unlocks:** reduced upfront schema work only.
- **Quick wins / low-hanging fruit:** none that justify the visibility loss.

**Recommendation**
- **Selected:** Option A — Doctor-backed readiness contract
- **Rationale (crisp):** guest-rootfs state must be explicit and inspectable; hidden readiness logic would just recreate the same ambiguity this ADR is trying to remove.

**Follow-up tasks (explicit)**
- Define the additive `world doctor --json` fields for backend kind, image id, world OS identity, and provisioning readiness.
- Ensure provisioning and runtime remediation reuse that readiness logic.
- Add tests and docs for the new doctor fields.

## DR-0006 — Backend selection vs image selection

**Decision owner(s):** Shell / World maintainers  
**Date:** 2026-05-24  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`

**Problem / Context**
- The backend choice and the guest-image choice are different concerns.
- Collapsing them would make the first shipped Ubuntu image look like the architecture rather than the first implementation.

**Option A — Separate backend and image config surfaces**
- **Pros:** correct abstraction; future image expansion does not require redefining the backend concept; clearer doctor/runtime reporting.
- **Cons:** more validation rules.
- **Cascading implications:** config docs and planning specs must define the relationship between the two keys.
- **Risks:** slightly more surface area for misconfiguration.
- **Unlocks:** distro-flexible backend contract with Ubuntu-first implementation.
- **Quick wins / low-hanging fruit:** easier later addition of more guest images.

**Option B — One combined backend/profile selector**
- **Pros:** smaller first-ship surface; fewer invalid states in v1.
- **Cons:** bakes the first image into the backend abstraction; later growth becomes awkward.
- **Cascading implications:** every new image would force CLI/config contract churn.
- **Risks:** architectural overfitting to the first shipped image.
- **Unlocks:** marginally simpler initial config.
- **Quick wins / low-hanging fruit:** shorter docs for v1 only.

**Recommendation**
- **Selected:** Option A — Separate backend and image config surfaces
- **Rationale (crisp):** the point is to decouple host distro from world distro; that requires backend and image to be separate concepts, even if v1 only ships one blessed image.

**Follow-up tasks (explicit)**
- Define `world.linux.backend` and `world.linux.image` behavior in planning specs/docs.
- Define default-image behavior when backend is `guest_rootfs` and image is unset.
- Add config-validation tests and doctor rendering for both surfaces.

## DR-0007 — Linux backend rollout posture

**Decision owner(s):** Shell / World / Installer maintainers  
**Date:** 2026-05-24  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`

**Problem / Context**
- The backend is a substantial change in Linux execution posture.
- The rollout posture determines how much existing Linux behavior changes by default.

**Option A — Make `guest_rootfs` the new Linux default**
- **Pros:** maximum parity and faster adoption.
- **Cons:** high blast radius; increases operational and debugging risk for existing Linux users.
- **Cascading implications:** every Linux flow becomes subject to the new backend immediately.
- **Risks:** regressions are harder to contain; warm/readiness issues would impact all Linux users.
- **Unlocks:** strongest immediate dogfooding.
- **Quick wins / low-hanging fruit:** none that outweigh the rollout risk.

**Option B — Keep `host_native` default and make `guest_rootfs` explicit opt-in**
- **Pros:** preserves existing Linux behavior; constrains risk; makes the new backend a deliberate operator choice.
- **Cons:** adoption is slower; parity benefits are not automatic.
- **Cascading implications:** planning can validate the new backend without destabilizing existing Linux workflows.
- **Risks:** some users may not discover the new backend quickly.
- **Unlocks:** safer rollout and clearer troubleshooting.
- **Quick wins / low-hanging fruit:** immediate availability for targeted users without forcing migration.

**Recommendation**
- **Selected:** Option B — Keep `host_native` default and make `guest_rootfs` explicit opt-in
- **Rationale (crisp):** the backend is strategically important, but it is too large a shift to make default before it proves itself in real Linux usage.

**Follow-up tasks (explicit)**
- Define opt-in config and remediation docs.
- Keep existing Linux host-native behavior unchanged by default.
- Add explicit doctor output showing whether `guest_rootfs` is active or merely configured.
