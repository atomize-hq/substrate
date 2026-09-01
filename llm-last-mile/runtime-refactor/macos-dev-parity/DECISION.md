# Decision — macOS Developer Parity Boundary

## Status and effective date

- **Status:** accepted macOS-lane decision; global blocking status superseded on 2026-08-20.
- **Effective date:** 2026-08-19.
- **macOS lane authority gate:** `AUTHORITY_REQUIRED:MACOS_DEV_PARITY`.
- **Active global implementation packet:** none. [`A1.3-P1`](../linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md),
  A1.4, the enclosing A1 slice, A2, A3, and Track A are terminally complete; E2 awaits fresh
  admission and explicit dispatch.
- **Phase 0 preservation:** verified read-only at protected lifecycle checkpoint
  `ff48da180db4515147486f8b95f05626ca38e89b` on
  `refs/heads/feat/archive-r3-macos-protected-lifecycle-20260819` and its remote-tracking ref;
  earlier donor checkpoint `c9b67286193b41f942e41c52af66331d5ad2d702` on
  `refs/heads/feat/archive-r3-macos-donor-20260814` and its remote-tracking ref; and current product
  baseline `40015a6cfa508c112e6444086341d6df8473e875` on the bound branch and upstream.

This decision supersedes the former R3 macOS, recovery, retirement/finalizer, E03, and Windows
predecessor sequence **for active scheduling only**. The former documents remain archived
engineering evidence and chronology.

## Decision

Restore ordinary current-product macOS development through a user-owned developer corridor that
does not depend on the protected macOS lifecycle architecture. The protected publisher,
System-Keychain authority, privileged host lifecycle service, finalizer/terminal retirement, and
E03 assurance program are archived. They are not the active macOS developer-install architecture
and require a future, separately authorized production threat-model decision before any revival.

Archival is a scheduling and ownership decision, not a claim that the protected source never
existed or has already been removed from the active tree. Its history, source, preserved refs,
failure records, and security engineering remain valuable evidence.

This decision owns the macOS platform lane only. It no longer makes macOS parity a predecessor of
runtime-refactor product work; the Linux-first global schedule is controlled by
[`../linux-first-runtime-resumption/DECISION.md`](../linux-first-runtime-resumption/DECISION.md).

## Reasons

- The ordinary macOS developer loop needs a current binary, configuration, shim, Lima, forwarding,
  and world-service path; it does not intrinsically need durable privileged host authority.
- Coupling development restoration to protected publication, terminal retirement, and E03 made an
  unfinished production-security architecture a prerequisite for routine product work.
- A small, natively provable developer corridor restores useful product feedback while preserving
  the protected design for a later threat-model decision rather than weakening or deleting it.

## Active macOS development boundary

The later parity implementation is limited to:

- user-owned prefix install and uninstall;
- current shims, configuration, and binary staging;
- current Lima and `world-service` provisioning;
- typed selected-prefix and Lima-instance mapping;
- safe host-to-guest forwarding; and
- native install → exercise world → uninstall → verify → reinstall proof.

The corridor must preserve exact selected-prefix/instance ownership, reject ambiguous or unrelated
resources, and leave the current-product developer path independently testable.

## Retained Lima and runtime requirements

Lima remains the macOS world boundary. The developer path must retain the current requirements for
Lima readiness, the selected instance and guest identity, `world-service` readiness, typed
host-to-guest mapping, the intended SSH-UDS or other already-current supported forwarding route,
and exact forwarding/process/socket teardown for the developer attempt. Linux privilege required
inside the Lima guest does not create macOS host lifecycle authority.

## Explicitly excluded protected machinery

Ordinary macOS development must not require or invoke:

- System Keychain lifecycle authority or records;
- the protected publisher;
- a macOS lifecycle LaunchDaemon or privileged host helper;
- terminal retirement or finalizer behavior;
- E03, freeze, identity rotation, evidence-mirror, or assurance-evidence machinery; or
- any recovery, migration, cleanup, or revival route from the archived protected architecture.

## Linux disposition

Previously landed Linux R3 facts remain historical landed facts. This decision does not reopen,
expand, reinterpret, undo, or claim new Linux R3 implementation or evidence. The global
Linux-first runtime-refactor reentry is a separate control-plane lane. Any later shared-script edit
from this macOS lane must preserve Linux behavior and receive an explicit integration boundary.

## Windows disposition

Windows R3 is not a prerequisite for the current product trajectory. Windows remains untouched and
incomplete where applicable, is outside this lane, and is deferred until a separately authorized
post-runtime-refactor scheduling decision.

## Attempt 4 quarantine

The replacement developer installer does not own Attempt 4. It must not inspect or mutate Attempt
4 Keychain records; retire, migrate, overwrite, adopt, or clean its fixed privileged artifacts; or
use the unfinished lifecycle machinery as a cleanup route.

Before Phase 2 may mutate the developer corridor, a separately authorized exact **read-only**
overlap check must prove that the selected user prefix, Lima instance, forwarding endpoint, files,
services, identities, and other resources are disjoint from Attempt 4. The check must use declared
resource identities and no Keychain query. If an actual path or resource collision is found, work
stops for a separately authorized disposition; the parity task must not repair or work around it.

## macOS lane authority gate

The macOS-lane gate is:

`AUTHORITY_REQUIRED:MACOS_DEV_PARITY`

It covers only implementation and native closure of the current-product macOS developer corridor
listed above. It is not the global product-work predecessor and grants no authority merely by
appearing in this document.

## Phase 2 entry conditions

Phase 2 may begin only after a fresh task explicitly authorizes `AUTHORITY_REQUIRED:MACOS_DEV_PARITY`
and rebinds live branch, source baseline, ancestry, worktree/index, exact path and symbol fences,
current installer/runtime behavior, Linux-preservation obligations, and the exact read-only Attempt
4 overlap check. The task must define its native macOS actions and restoration/stop rules before
any effect. Phase 1 does not authorize Phase 2.

## Native acceptance outline

Native closure must demonstrate, on the freshly bound current product path:

1. install into the selected user-owned prefix without protected lifecycle dependencies;
2. stage and resolve the current binaries, shims, and configuration from that prefix;
3. provision and bind the selected Lima instance and `world-service` through typed mapping;
4. establish safe forwarding and exercise a real world operation;
5. uninstall only developer-corridor resources and verify their exact intended absence while
   preserving unrelated and pre-existing state; and
6. reinstall and repeat the required readiness/world checks.

Static checks do not substitute for the native proof. Any ambiguity, overlap, unrelated-state
change, protected-lifecycle invocation, or failed restoration leaves the parity gate open.

## Cross-lane continuation

macOS parity is not a predecessor of runtime-refactor product work. Its later implementation and
native closure neither block nor authorize the Linux-first runtime-refactor sequence. The reentry
gate remains closed as historical selection, and the separately selected
[`A1.3-P1 packet`](../linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md)
is terminally complete.
The held A1.3 and A1.3-P0 records remain preserved as historical runtime fences only. Closing
either lane does not dispatch the other's successor. A1.4 and the enclosing A1 slice are
terminally complete; A2, A3, and Track A are terminally complete under their separate closure
identities, and E2 awaits fresh admission and explicit dispatch. This decision does not authorize
macOS work within A1.3-P1, dispatch E2, Windows, E03, protected-lifecycle
revival, or any other successor.

## Rollback and revival rule

Rollback of this decision is another documentation/authority decision; it must not mutate archive
refs or native state. Reviving any protected lifecycle, publisher, finalizer, retirement, or E03
element requires a separately authorized production threat-model decision that states the concrete
root-owned host need, adversary model, upgrade/migration/retirement ownership, native proof, and
relationship to the ordinary developer corridor. Revival is not an implicit parity follow-up.

## Non-goals

- Implementing Phase 2 or changing product/source/script/test/manifest/Cargo bytes.
- Removing protected lifecycle source or rewriting its history.
- Cleaning, restoring, migrating, or querying Attempt 4.
- New Linux proof or behavior, Windows implementation, or cross-platform closeout.
- Native macOS execution, installer/uninstaller execution, Lima actions, or privileged operations.
- A new production security architecture specification.
- Blocking the Linux-first runtime-refactor reentry on macOS parity closure.
