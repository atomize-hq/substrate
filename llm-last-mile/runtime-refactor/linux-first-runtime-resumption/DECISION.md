# Decision — Linux-First Runtime-Refactor Resumption

## Status and effective date

- **Status:** accepted for active scheduling; documentation-only decision.
- **Effective date:** 2026-08-20.
- **Reentry gate:** `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY` (closed by the selection below).
- **Active global implementation packet:** none. [`A1.3-P1`](A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md),
  A1.4, the enclosing A1 slice, A2, A3, and Track A are terminally complete; E2 awaits fresh
  admission and explicit dispatch.
- **Supersedes:** the 2026-08-19 macOS developer-parity decision only where it made
  `AUTHORITY_REQUIRED:MACOS_DEV_PARITY` the global product-work predecessor. It does not weaken,
  close, or authorize the macOS parity lane.

## Context

The 2026-08-19 reset correctly removed protected macOS lifecycle, Keychain, publisher, finalizer,
E03, and Windows work from the ordinary developer-install path. It also made macOS parity the sole
active gate, which unnecessarily blocked return to the remaining runtime-refactor control-plane
work. The product has already established Linux as the first adoption platform; macOS parity is a
separate platform-restoration concern, and Windows remains deferred.

## Decision

Resume runtime-refactor scheduling on the current product branch with Linux-first adoption. The
macOS developer-parity effort is a parallel, non-blocking platform lane. Its future work belongs in
the dedicated `feat/macos-dev-installer-parity` branch after a separate branch/worktree action; no
branch is created by this decision. Windows remains deferred until a later, separately authorized
scheduling decision after the runtime refactor is complete.

At the time of this scheduling decision, the sole global next step was the bounded
`AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY` control-plane packet. It was a rebind-and-selection
packet, not product implementation authority. Its required duties were to:

1. rebind the live product branch, ancestry, worktree/index, source baseline, current control-pack
   state, and the already-landed historical evidence it relies on;
2. select exactly one next runtime-refactor packet from live source and record its named owner,
   path and symbol fences, dependencies, Linux-first acceptance criteria, and exclusions; and
3. publish no runtime, installer, Lima, macOS, Windows, archive, or native-operation change.

Only the selected later packet may receive separate implementation authority. This decision did
not assume that A1.3, A1.4, or any other historical label was the next implementation packet until
the rebind was complete.

## Reentry selection (2026-08-20; historical selection record)

The reentry packet is closed as documentation/control-plane selection only. Its live branch,
ancestry, control-pack, historical-evidence, Linux-first, macOS-exclusion, and Windows-deferral
rebind selected **A1.3 — public/successor adoption and startup-result completion**. That selected
later packet remains preserved at
[`A1.3-LINUX-FIRST-PACKET.md`](A1.3-LINUX-FIRST-PACKET.md).

This historical selection itself performed no runtime, installer, Lima, macOS, Windows, archive,
or native operation and did not claim A1.3 completion.

## A1.3/P0 corrective replan (2026-08-21; controlling)

The frozen evidence packet at remote path
`llm-last-mile/runtime-refactor/review-control/a1-3-p0-scope-expansion-evidence.md`, bound to
commit `6bb9901e9c0f030224e4e6208ac5a399ebc65d85`, proved that the then-current
A1.3 fence would require unapproved expansion across the legacy hidden-owner launcher corridor and
its frozen auto-attach caller. Public `run_start`, `run_turn`, and `run_reattach` still reached the
legacy `launch_hidden_owner_helper` path; the authority-managed route still crossed readiness,
timeout-reconciliation, and legacy authority-write boundaries; and `agent_runtime/auto_attach.rs`
remained a separate compatibility caller on that same launcher.

The later corrective replan on bound target `a8044271cdeb5bbcbe6001b1b34eb2c7ab95f134` verified one
additional source seam: the real retained-turn request path still submits
`acceptance_context: None` even though B1 already owns the optional equality-only
`host_transition_correlation` carrier and A1.2b is already the sole production source for that
exact value.

Therefore neither the held A1.3 packet nor the held A1.3-P0 preparatory split remains executable
as the next implementation packet. The old A1.3 fence omitted the public transport path and the
exact retained-turn `orchestrator_world_dispatch.rs` acceptance-context construction/projection
seam. The A1.3-P0 split validly identified the legacy launcher corridor but is non-implementable
as a standalone prerequisite because the same real public CLI/helper/REPL path must land,
atomically, the new authority-managed public transport path, exact HostSessionAuthority
consumption and actor-event resolution, and only the strictly mechanical projection of already
HSA-authorized correlation through that exact retained-turn seam and into the B1 acceptance-context
field.

The corrective replan selected **A1.3-P1 — atomic public adoption**, recorded in
[`A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md`](A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md).
A1.3-P0 and the old A1.3 packet remain preserved as held historical fences and may not be
dispatched. At that selection point, A1.4, Windows, and all other later work remained undispatched
until their own fresh authority packets activated them.

## A1.3-P1 closure (2026-08-29; controlling)

A1.3-P1 is terminally complete at commit
`ee3b9f0dbcaf60f4a97aae9bdabe51570d400d09` and tree
`6d256d9e59cb2f4f74d07ccc60a4a0c3b57285e8` under the exact closure identities recorded in its
packet owner. All nine packet conditions are satisfied. `RG-BASE-01` remains open, unwaived, and
not green; its outstanding positive real-path Linux smoke belongs to A1.4's final A1 proof wall
under current sequencing authority and is not an unsatisfied A1.3-P1 implementation or closure
condition.

A1.4 — bounded auto-attach producer adoption and regression closure — is only the named
implementation successor. It awaits fresh admission and explicit dispatch. This closure does not
schedule, authorize, admit, dispatch, implement, or complete A1.4, and A1.4 cannot claim final
completion until it supplies the required `RG-BASE-01` proof.

## A2 closure (2026-08-31; controlling)

[A2 — HostExecutionEpisode demotion](../slices/a2-host-execution-episode-demotion.md#terminal-closure)
is terminally complete at implementation commit
`1d7176a25b7662368e6e1fb3f65b3521e6cab78b` and tree
`e71f731c279a8d65e1111245fc3d31940edd3afd` under its exact reviewed closure identities. It
consumed the landed HSA fork-successor prerequisite, while the public `run_stop` disposition
remains unchanged A1 work. Its Linux proof closes only A2's episode-demotion scope and does not
claim Track A, macOS, Windows, Track B, or Track C completion.

A3 — persistence and compatibility split — is only the existing canonical successor. It awaits
fresh admission and explicit dispatch; this closure does not admit, dispatch, implement, or
complete it.

## A3 and Track A closure (2026-09-01; controlling)

[A3 — persistence and compatibility split](../slices/a3-persistence-and-compatibility-split.md#terminal-closure)
is terminally complete at implementation commit
`627e0febf9e0a5143a8b92ba54f1c9377d3ffae5` and tree
`2fe1acc6d0abe2a50c2f44dc47f636f51223d390` under its exact reviewed closure identities. The
compatibility boundary is structurally read-only, while StateStore retains trusted-root locking,
atomic persistence, rollback, publication, and `fsync` ownership. No durable format, schema
version, or migration changed.

The Track A table contains only A0 through A3. A0's committed classified inventory satisfies its
diagnostic exit, and A1/A2/A3 are terminally complete, so Track A is terminally complete. The hard
dependency spine names E2 as the canonical successor, while E1 remains a separate unresolved
prerequisite; E2 awaits fresh admission and explicit dispatch. The full `RG-BASE-03` continuity
witness remains open, blocking, unwaived, and not green under C2. This closure admits or dispatches
none of E1, E2, or C2 and exercises no later-track authority.

## macOS lane

`AUTHORITY_REQUIRED:MACOS_DEV_PARITY` remains the lane-local gate for the ordinary user-prefix
macOS developer corridor. It retains every existing admission, Attempt 4 quarantine, protected
architecture exclusion, Linux-preservation, native proof, and restoration requirement in
[`../macos-dev-parity/DECISION.md`](../macos-dev-parity/DECISION.md). Its implementation and native
closure are not predecessors of runtime-refactor work, and closing either lane does not authorize
the other's successor.

If either lane needs a shared script or shared runtime surface, that change requires an explicit
integration packet with exact ownership and proof that Linux behavior is preserved. Neither lane
may silently adopt work from the other merely because the Git histories have a common base.

## Windows disposition

Windows is untouched, incomplete where applicable, and deferred. It is not a prerequisite for the
the selected A1.3 packet, Linux-first adoption, or macOS parity. No Windows implementation,
native proof, cleanup, or completion claim is authorized here.

## Alternatives considered

### Keep macOS parity as the global predecessor

Rejected. It continues to couple routine runtime-refactor progress to a native platform-restoration
loop whose security-hardening work has deliberately been archived.

### Resume the archived R3 macOS/Windows sequence

Rejected. The protected lifecycle architecture remains archived reference material and requires a
separate production threat-model decision before any revival.

### Declare an implementation packet immediately

Rejected at the time of this scheduling decision. The current source and control pack first had to
be rebound so that the next packet was bounded by live ownership, dependencies, and Linux-first
proof rather than a historical label. That rebind later selected A1.3.

## Consequences and non-goals

- The current branch may return to the runtime-refactor control plane without waiting for native
  macOS proof.
- The macOS lane remains deliberately narrow and may proceed independently only under its own
  fresh authorization.
- This scheduling decision itself did not create a branch, modify product source, re-open Linux
  R3, or perform a Linux proof run. Its separately recorded reentry selection first selected A1.3,
  the later frozen amendment selected A1.3-P0 as a narrower preparatory split, and the corrective
  replan held both older fences while selecting the now-terminally-complete A1.3-P1 atomic
  public-adoption packet. The later A1.4 implementation and enclosing A1 slice are terminally
  complete under their separate closure identities. The still-later A2 and A3 implementations and
  Track A are also terminally complete under their separate closure identities; E2 remains
  undispatched.
- This decision does not dispatch Windows, protected macOS lifecycle revival, Keychain activity,
  Attempt 4 inspection or remediation, installer execution, Lima actions, or native operations.
