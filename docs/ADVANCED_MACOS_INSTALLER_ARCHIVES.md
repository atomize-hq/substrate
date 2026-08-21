# Advanced macOS Installer Archive Map

## Status

- **Advanced/hardened installer:** deferred.
- **Current macOS direction:** restore the ordinary developer install loop without the protected
  lifecycle, publisher, Keychain, finalizer, or assurance machinery.
- **macOS lane branch:** planned `feat/macos-dev-installer-parity` (not created by this decision).
- **Current product branch:** `feat/internal-host-orchestrator-world-dispatch-bootstrap` resumes the
  Linux-first runtime-refactor control plane.
- **Archive verification date:** 2026-08-19.

The archive branches below preserve prior engineering work and failure history. They are reference
material, not active architecture, proof of correctness, or branches to merge wholesale. The future
macOS lane must retain the ordinary Lima/runtime behavior needed by macOS while removing the advanced
installer dependencies from the developer path. It is non-blocking for Linux-first runtime-refactor
work on the current product branch.

## Remote archive inventory

| Remote archive | Intended later use | Explicit limitation |
|---|---|---|
| `origin/feat/archive-r3-macos-protected-lifecycle-20260819` | Primary advanced implementation reference. It contains the advanced development installer and completed advanced uninstaller together, plus lifecycle control, publisher, Keychain, XPC, finalizer, signer/ACL, tests, and the recovery-stack history. Future advanced features should be recovered selectively and in independently testable phases. | Do not merge wholesale or treat as the current macOS architecture. Any revival requires a new production threat model and explicit ownership for installation, upgrade, recovery, and removal. |
| `origin/feat/archive-r3-macos-donor-20260814` | Earlier forensic snapshot of the unreviewed finalizer/signer donor state. Use only to inspect or recover an earlier state that is absent from the primary protected-lifecycle history. | It is not the source of the completed advanced installer/uninstaller and is not known good. |
| `origin/feat/archive-ace6-macos-lima-intermediate-20260819` | Limited secondary reference for partially restored Lima instance detection, create/start, readiness waiting, guest provisioning, binary/unit installation, and socket-activation behavior. | `UNREVIEWED`, `NOT KNOWN GOOD`, and `NOT FOR MERGE`. Do not use its forwarding, stop implementation, lifecycle executor, publisher integration, or lifecycle tests. The current branch and the last working pre-protected implementation remain the primary sources. |
| `origin/feat/archive-r3-mac-meta-orchestration-20260819` | Failure and decision chronology. It preserves the R2-R6 orchestration history, evidence attempts, blockers, corrections, FD3/XPC failures, consultations, authority amendments, and scratch material needed to avoid repeating prior dead ends. | Reference only; not product code. The preserved scratch scripts are not validated and must not be treated as executable guidance. |
| `origin/feat/archive-macos-dev-parity-phase2a-research-20260819` | Historical archaeology comparing the pre-protected and protected installer paths and recording why the developer-parity reset was selected. | It is not the active implementation plan. Its reconstruction plan is partly rejected as overengineered. |

## Reference order for future advanced work

When advanced macOS installation is reconsidered:

1. Start from the then-current product branch and a fresh threat model; do not restart by checking
   out an archive as the new product baseline.
2. Use the protected-lifecycle archive for implementation history and recover only one bounded,
   independently testable capability at a time.
3. Use the meta-orchestration archive to understand prior failures and decision constraints before
   selecting a capability.
4. Use the `ace6` archive only to compare ordinary Lima create/start/wait/provision behavior.
5. Use the donor archive only when investigating an earlier state missing from the primary archive.
6. Use the Phase 2A archive only for historical comparison, not task authority.

Likely advanced capabilities must be staged separately rather than revived as one system. Examples
include privileged host lifecycle, Keychain authority, publisher installation, upgrade/replacement,
recovery, finalization, and assurance evidence. A working developer install must not depend on
landing all of them together.

## macOS-lane simplification boundary

The planned macOS lane owns the simpler macOS developer path:

- user-owned installation and uninstallation;
- current binaries, shims, and configuration;
- the existing Lima world boundary and `world-service` provisioning;
- selected-instance create/start/readiness behavior;
- the current safe host-to-guest forwarding implementation; and
- an install, exercise, uninstall, verify, and reinstall loop comparable to Linux development.

The active simplification must not adopt archived System Keychain, protected publisher,
LaunchDaemon/privileged-helper, finalizer, terminal-retirement, E03, or assurance-evidence
dependencies. Windows work remains outside this macOS effort.

The macOS lane is governed by
[`../llm-last-mile/runtime-refactor/macos-dev-parity/DECISION.md`](../llm-last-mile/runtime-refactor/macos-dev-parity/DECISION.md).
The Linux-first runtime-refactor lane remains separately governed by
[`../llm-last-mile/runtime-refactor/linux-first-runtime-resumption/DECISION.md`](../llm-last-mile/runtime-refactor/linux-first-runtime-resumption/DECISION.md).
macOS remains excluded from that lane; neither lane's closure authorizes the other's successor.

## Explicitly excluded worktree

`/Users/spensermcconnell/.codex/worktrees/r3-macos-finalizer-proof-candidate/substrate` is not part of
the future reference set. Its remaining dirty state consists of unproven finalizer freeze,
root-install, and cleanup/recovery experiments with no identified need in either the simple path or
a future advanced phase. Do not cite it as an implementation source.

## Physical-worktree snapshot

As of 2026-08-19:

- the recovery-stack worktree was removed after verifying that its clean endpoint exactly matched
  the protected-lifecycle archive locally and remotely;
- the Phase 2A worktree was removed after its clean history was pushed to the research archive;
- the `ace6` and `eb49` worktrees were clean after their archive branches were pushed and verified;
  and
- the excluded finalizer proof-candidate worktree remained dirty and unarchived.

Physical worktree presence is not the preservation authority. The verified remote archive branches
are the durable references.

## Revival rule

Revisiting the hardened installer requires a separately authorized design and implementation
effort after the ordinary macOS developer loop works. That effort must define the concrete host
privilege need, adversary model, resource ownership, upgrade and migration path, failure recovery,
uninstallation/retirement behavior, and native verification for each phase. Existing archives
provide evidence and source material only; they grant no implementation or native-operation
authority.
