# Target Architecture

> **Compatibility boundary:** Shared target architecture canonical content now lives under [`architecture/`](architecture/README.md). This root file preserves the legacy `01-target-architecture.md` headings and anchors until the D12 compatibility-index cutover. Packet-family-local forwarders below retain their existing D5/D6 owners.

## Executive decision

Canonical content: [`architecture/executive-target.md#executive-decision`](architecture/executive-target.md#executive-decision).

### Current B1/B2.1 control state

Canonical content: [`b1-b2-1/architecture.md#current-b1b21-control-state`](b1-b2-1/architecture.md#current-b1b21-control-state).

## Authority map

Canonical content: [`architecture/authority-map.md#authority-map`](architecture/authority-map.md#authority-map).

## Non-negotiable invariants

Canonical invariant index: [`architecture/invariants/README.md#non-negotiable-invariants`](architecture/invariants/README.md#non-negotiable-invariants).

### 1. Durable session truth is process-independent

Canonical content: [`architecture/invariants/01-durable-session-truth-is-process-independent.md#1-durable-session-truth-is-process-independent`](architecture/invariants/01-durable-session-truth-is-process-independent.md#1-durable-session-truth-is-process-independent).

#### Exact bound-world physical ownership

Canonical content: [`architecture/invariants/01-durable-session-truth-is-process-independent.md#exact-bound-world-physical-ownership`](architecture/invariants/01-durable-session-truth-is-process-independent.md#exact-bound-world-physical-ownership).

### 2. Private transports are fast paths

Canonical content: [`architecture/invariants/02-private-transports-are-fast-paths.md#2-private-transports-are-fast-paths`](architecture/invariants/02-private-transports-are-fast-paths.md#2-private-transports-are-fast-paths).

### 3. Routing is exact and fail-closed

Canonical content: [`architecture/invariants/03-routing-is-exact-and-fail-closed.md#3-routing-is-exact-and-fail-closed`](architecture/invariants/03-routing-is-exact-and-fail-closed.md#3-routing-is-exact-and-fail-closed).

### 4. Long-lived work accepts before it completes

Canonical content: [`architecture/invariants/04-long-lived-work-accepts-before-it-completes.md#4-long-lived-work-accepts-before-it-completes`](architecture/invariants/04-long-lived-work-accepts-before-it-completes.md#4-long-lived-work-accepts-before-it-completes).

#### B2.1-3 restart and producer-replay boundary

Canonical content: [`architecture/invariants/04-long-lived-work-accepts-before-it-completes.md#b21-3-restart-and-producer-replay-boundary`](architecture/invariants/04-long-lived-work-accepts-before-it-completes.md#b21-3-restart-and-producer-replay-boundary).

### 5. Cancel targets active work

Canonical content: [`architecture/invariants/05-cancel-targets-active-work.md#5-cancel-targets-active-work`](architecture/invariants/05-cancel-targets-active-work.md#5-cancel-targets-active-work).

### 6. Obligations are event-derived canonical truth

Canonical content: [`architecture/invariants/06-obligations-are-event-derived-canonical-truth.md#6-obligations-are-event-derived-canonical-truth`](architecture/invariants/06-obligations-are-event-derived-canonical-truth.md#6-obligations-are-event-derived-canonical-truth).

### 7. Auto-attach restores ownership only

Canonical content: [`architecture/invariants/07-auto-attach-restores-ownership-only.md#7-auto-attach-restores-ownership-only`](architecture/invariants/07-auto-attach-restores-ownership-only.md#7-auto-attach-restores-ownership-only).

### 8. World placement and policy mediation are separate proofs

Canonical content: [`architecture/invariants/08-world-placement-and-policy-mediation-are-separate-proofs.md#8-world-placement-and-policy-mediation-are-separate-proofs`](architecture/invariants/08-world-placement-and-policy-mediation-are-separate-proofs.md#8-world-placement-and-policy-mediation-are-separate-proofs).

### 9. `external_sandbox` assigns responsibility

Canonical content: [`architecture/invariants/09-external-sandbox-assigns-responsibility.md#9-external_sandbox-assigns-responsibility`](architecture/invariants/09-external-sandbox-assigns-responsibility.md#9-external_sandbox-assigns-responsibility).

### 10. Dispatch policy only narrows

Canonical content: [`architecture/invariants/10-dispatch-policy-only-narrows.md#10-dispatch-policy-only-narrows`](architecture/invariants/10-dispatch-policy-only-narrows.md#10-dispatch-policy-only-narrows).

### 11. Existing `world_fs` enforcement is the execution path

Canonical content: [`architecture/invariants/11-existing-world-fs-enforcement-is-the-execution-path.md#11-existing-world_fs-enforcement-is-the-execution-path`](architecture/invariants/11-existing-world-fs-enforcement-is-the-execution-path.md#11-existing-world_fs-enforcement-is-the-execution-path).

### 12. Runtime-native configuration is projection

Canonical content: [`architecture/invariants/12-runtime-native-configuration-is-projection.md#12-runtime-native-configuration-is-projection`](architecture/invariants/12-runtime-native-configuration-is-projection.md#12-runtime-native-configuration-is-projection).

### 13. Credentials are launch-time gateway handoff, not projected files

Canonical content: [`architecture/invariants/13-credentials-are-launch-time-gateway-handoff-not-projected-files.md#13-credentials-are-launch-time-gateway-handoff-not-projected-files`](architecture/invariants/13-credentials-are-launch-time-gateway-handoff-not-projected-files.md#13-credentials-are-launch-time-gateway-handoff-not-projected-files).

### 14. `SUBSTRATE_HOME` is private per-user authority state

Canonical content: [`architecture/invariants/14-substrate-home-is-private-per-user-authority-state.md#14-substrate_home-is-private-per-user-authority-state`](architecture/invariants/14-substrate-home-is-private-per-user-authority-state.md#14-substrate_home-is-private-per-user-authority-state).

#### Installation prefix authority and platform realization

Canonical content: [`architecture/invariants/14-substrate-home-is-private-per-user-authority-state.md#installation-prefix-authority-and-platform-realization`](architecture/invariants/14-substrate-home-is-private-per-user-authority-state.md#installation-prefix-authority-and-platform-realization).

#### Remaining authenticated runtime projections

Routes A–D are individually review-clean, but they do not exhaust the authenticated-context path.
The failed R2-2 integration closeout found two remaining projection seams and one diagnostic
composition invariant. R2-2E is implementation-, proof-, and review-complete. F0, F0a, F0b, and
F0-HC were then implemented, proof-complete, review-clean, committed, and preserved. R2-2F was the
exact next packet at that checkpoint. At the later F3/F4 checkpoint, F5-PD was the exact next
prerequisite beneath F5; the completed-F section below supersedes that historical status. A renewed
Routes A–F integration closeout follows F.

R2-2E makes world-gateway projection a pure consumer of already-authenticated A. The landed shell world
entry validates A before disabled/unavailable classification, then supplies it to one request-scoped
gateway context. Configuration and effective policy use their existing explicit-bootstrap-home
resolvers; `execution/policy_snapshot.rs` remains the sole policy-snapshot owner and adds the one
explicit-bootstrap-home world-network entrypoint; `execution/agent_inventory.rs` reuses its existing
bootstrap-home inventory loader. `world_gateway.rs` may combine those explicit outputs but may not
parse or re-resolve policy. Runtime-family and Codex paths are projections of A and the committed
Unix principal's account-database home. Environment-only invocation, malformed/tampered/mismatched
context, and a disabled route without valid A fail before gateway mutation, forwarding, or launch.
The contextless synthesized-unavailable constructor is removed. Linux uses the fixed canonical
`/run/substrate.sock` on this authenticated route; ambient socket overrides cannot retarget it.
On macOS, no authenticated typed endpoint source exists in E's closed allowlist: the authenticated
route therefore fails before client construction or ambient `auto_select`; it cannot invent or
accept an endpoint from environment/platform-control state. Lower Lima forwarding, known-hosts, and
transport realization remain R2-3. Windows and other contextless gateway entries likewise fail
before config, policy, inventory, disabled-state classification, or client selection until R2-3
supplies authenticated platform mapping. Existing ambient macOS/Windows compatibility clients remain
frozen and unreachable from the E route. Their cfg proof is build/static fail-closed preservation,
not native or authenticated product proof. Existing network allow/deny meaning, request schemas,
service behavior, and launch-time
gateway credential handoff do not change, and no host credential file becomes durable authority.

The exact landed E evidence is commit `7e8e83802885c0ece93efcaacccc26503eeb6715`, tree
`02a1a2f6b7e4be47ed9ec38537c805ae348c96b6`, ordinary/full-index patch identities
`fb7b65b02cdac46857750b64ebd5ceab651e8e99910c05240b187c3e729444ff` and
`c712f467ac92efb0de7b524272479741ac6b318201cf45dbd994116ec0e8b862`. This completes only
PI-111's R2-2E implementation/proof clause. The managed gateway secure-FD path is landed,
regression-proven, and unchanged by E; direct-member Codex/UAA gateway adoption remains unresolved,
transitional, non-promotable, and E3/D1/D3-owned. R2-2 itself remains incomplete, and no target seam
is promoted.

**A1.1d-5R2-2F0a — SUBSTRATE_HOME test isolation** joins R2-2F0 to establish one
test-process-only authority-environment boundary around every
same-process mutation of `SUBSTRATE_WORLD_SOCKET` or `SUBSTRATE_HOME`. F0's historically blocked candidate
already proves its exact socket pair 100/100 in parallel and 20/20 serially, but final broad runs
were `1118 passed / 150 failed` and `1119 passed / 149 failed`; the extra failure was
`dispatch_contract_adapter_active_task_resolution_requires_supervisor_claim`, which passes alone.
Non-source transition tracing proved that the unannotated
`prompt_submit_continuity_prefers_persisted_session_contract` fixture can set its private HOME,
the target can replace it, and the competitor can then remove the target's HOME while both overlap.
The pair failed 20/20 in parallel. The stable same-process serial harness controlled only
competitor-then-target and failed 0/10; separate-process sequential runs passed in both directions.
Stable libtest could not force reverse same-process order, so no such result is claimed. Commit
`f5a150f94d585b1f55ec0067845cd5d715773c78` first adds the exact unannotated competing mutation;
the target and its HOME-mutating fixture arrive later at
`83101dcbcc750e6e8fb8979bea19f1f777792188`, the first source commit where the exact pair coexists.

The reviewed topology is one process-global authority-environment lock, not separate HOME/socket
locks. At least 88 shell-library test functions depend jointly on HOME and socket state, and source
closure found opposite existing acquisition orders, so separate locks would permit mixed
authority snapshots and lock-order inversion. The shared boundary captures exact prior `OsString`
or absence, installs the test-owned value or absence, spans all dependent async/process work and
cleanup, restores during normal return or panic unwinding, and releases only after restoration.
Same-thread nesting must be explicitly safe and restore in stack order; panic/poison behavior must
be explicit and cannot strand later tests. `#[serial]` may remain but is never sufficient by
itself. Integration-test binaries remain process-local: direct parent mutations receive an
equivalent per-binary disposition, while child-only `Command::env` inputs need no cross-process
lock.

F0 also owns a source-closure-proven async cleanup correction in nine socket-owning tests in
`execution/orchestrator_world_dispatch.rs`: abort the server, await confirmed task cancellation,
finish fixture-owned socket/task cleanup, restore the environment, and release the isolation lock,
in that order. Reverse declaration/drop order and a single scheduler yield are not proof of task
termination. Production world-socket/HOME resolution, readiness, retained-worker behavior, retry
validation, and runtime bytes remain unchanged.

**A1.1d-5R2-2F0b — deterministic renderer-output test isolation** removes process-global
descriptor replacement from the two renderer fallback tests without changing the renderer's
production contract. Source closure binds the future change to the private Unix-only
`PublicPromptRenderer` ownership in
`crates/shell/src/execution/agent_runtime/control.rs`. `PublicPromptRenderer::render` remains
the production entry point; `PublicPromptRenderer::new` and the two production caller bodies,
`run_hidden_owner_helper_startup_prompt_stream_with_projection` and
`run_public_prompt_command`, remain frozen. A private explicit-writer core or equivalent private
sink adapter may sit beneath `render`; the production adapter must choose and lock only the
selected real stdout or stderr stream in the same order as today, write the same bytes and newline,
perform the same flush, and preserve which serialization/write errors propagate or are ignored.
No public API, output transport, registry, side table, process-global lock, environment-selected
sink, or eager dual-stream locking is permitted.

The observed stdout helper uses `dup2` on process fd 1. A parallel libtest reporter therefore
contributed its `.` to the test pipe, producing exactly
`".[codex] task_progress: fields=alpha, beta, gamma (+1 more)\n"`. The forced same-process
matrix passed 376 and failed 124 of 500 runs; isolated, same-process serial, and separate-process
controls passed 100/100, and parallel pretty reporting passed 99/100. The clean-E target/helper are
byte-identical to the F0/F0a candidate, so candidate introduction is unnecessary. The fd 2 helper
has the same ownership and structural race and is migrated with fd 1 even though only stdout
appeared in the wall. Tests receive their own stdout/stderr memory writers and compare the complete
exact selected bytes plus an empty nonselected stream. Filtering reporter bytes, partial-line
search, sleeps, retries, serial-only correctness, thread reduction, ignore, and weaker assertions
are outside the architecture.

### Historical pre-F architecture checkpoint

Canonical content: [`a1.1d-5r2-2f/architecture.md#historical-pre-f-architecture-checkpoint`](a1.1d-5r2-2f/architecture.md#historical-pre-f-architecture-checkpoint).

## Review question

Canonical content: [`architecture/review-question.md#review-question`](architecture/review-question.md#review-question).

## F0-HC test-process coordination topology

Canonical content: [`a1.1d-5r2-2f/architecture.md#f0-hc-test-process-coordination-topology`](a1.1d-5r2-2f/architecture.md#f0-hc-test-process-coordination-topology).

## Differential evidence authority after the final harness candidate

Canonical content: [`a1.1d-5r2-2f/architecture.md#differential-evidence-authority-after-the-final-harness-candidate`](a1.1d-5r2-2f/architecture.md#differential-evidence-authority-after-the-final-harness-candidate).

## Historical F explicit Linux readiness architecture

Canonical content: [`a1.1d-5r2-2f/architecture.md#historical-f-explicit-linux-readiness-architecture`](a1.1d-5r2-2f/architecture.md#historical-f-explicit-linux-readiness-architecture).

## F5-PD authenticated passive diagnostic architecture

Canonical content: [`a1.1d-5r2-2f/architecture.md#f5-pd-authenticated-passive-diagnostic-architecture`](a1.1d-5r2-2f/architecture.md#f5-pd-authenticated-passive-diagnostic-architecture).

## A1.1d-5R2-2F completed diagnostic architecture

Canonical content: [`a1.1d-5r2-2f/architecture.md#a11d-5r2-2f-completed-diagnostic-architecture`](a1.1d-5r2-2f/architecture.md#a11d-5r2-2f-completed-diagnostic-architecture).
## Renewed R2-2 attestation and publication ordering

Canonical content: [`a1.1d-5r2-2-renewed-closeout/architecture.md#renewed-r2-2-attestation-and-publication-ordering`](a1.1d-5r2-2-renewed-closeout/architecture.md#renewed-r2-2-attestation-and-publication-ordering).

## Broad-wall evidence architecture

Canonical content: [`a1.1d-5r2-2-renewed-closeout/architecture.md#broad-wall-evidence-architecture`](a1.1d-5r2-2-renewed-closeout/architecture.md#broad-wall-evidence-architecture).

## Closeout-remediation architecture: R1 and P1 (historical RP0/RP1/RP2 record)

Canonical content: [`a1.1d-5r2-2-renewed-closeout/architecture.md#closeout-remediation-architecture-r1-and-p1-historical-rp0rp1rp2-record`](a1.1d-5r2-2-renewed-closeout/architecture.md#closeout-remediation-architecture-r1-and-p1-historical-rp0rp1rp2-record).

### R1: separate execution argv from display argv

Canonical content: [`a1.1d-5r2-2-renewed-closeout/architecture.md#r1-separate-execution-argv-from-display-argv`](a1.1d-5r2-2-renewed-closeout/architecture.md#r1-separate-execution-argv-from-display-argv).

### P1: retired runner history and current Make authority

Canonical content: [`a1.1d-5r2-2-renewed-closeout/architecture.md#p1-retired-runner-history-and-current-make-authority`](a1.1d-5r2-2-renewed-closeout/architecture.md#p1-retired-runner-history-and-current-make-authority).

### Historical RP0 architecture status

Canonical content: [`a1.1d-5r2-2-renewed-closeout/architecture.md#historical-rp0-architecture-status`](a1.1d-5r2-2-renewed-closeout/architecture.md#historical-rp0-architecture-status).

## RP3/RP4/RP5 closeout architecture status and subsequent completion

Canonical content: [`a1.1d-5r2-2-renewed-closeout/architecture.md#rp3rp4rp5-closeout-architecture-status-and-subsequent-completion`](a1.1d-5r2-2-renewed-closeout/architecture.md#rp3rp4rp5-closeout-architecture-status-and-subsequent-completion).
## R2-4 bounded closeout architecture disposition

Canonical content: [`a1.1d-5r2-4/architecture-disposition.md#r2-4-bounded-closeout-architecture-disposition`](a1.1d-5r2-4/architecture-disposition.md#r2-4-bounded-closeout-architecture-disposition).

## A1.1d-5R3 lifecycle ownership architecture

Canonical content: [`a1.1d-5r3/architecture.md#a11d-5r3-lifecycle-ownership-architecture`](a1.1d-5r3/architecture.md#a11d-5r3-lifecycle-ownership-architecture).

### Authority domains and action classes

Canonical content: [`a1.1d-5r3/architecture.md#authority-domains-and-action-classes`](a1.1d-5r3/architecture.md#authority-domains-and-action-classes).

### Descriptor-bound private-home candidate rollback

Canonical content: [`a1.1d-5r3/architecture.md#descriptor-bound-private-home-candidate-rollback`](a1.1d-5r3/architecture.md#descriptor-bound-private-home-candidate-rollback).

### Managed-artifact manifest

Canonical content: [`a1.1d-5r3/architecture.md#managed-artifact-manifest`](a1.1d-5r3/architecture.md#managed-artifact-manifest).

### AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION (2026-08-10)

Canonical content: [`a1.1d-5r3/architecture.md#aux-r3-mac-system-keychain-software-signer-correction-2026-08-10`](a1.1d-5r3/architecture.md#aux-r3-mac-system-keychain-software-signer-correction-2026-08-10).

### Lifecycle state machines and rollback order

Canonical content: [`a1.1d-5r3/architecture.md#lifecycle-state-machines-and-rollback-order`](a1.1d-5r3/architecture.md#lifecycle-state-machines-and-rollback-order).

### Exact platform boundaries

Canonical content: [`a1.1d-5r3/architecture.md#exact-platform-boundaries`](a1.1d-5r3/architecture.md#exact-platform-boundaries).

### Publication and evidence architecture

Canonical content: [`a1.1d-5r3/architecture.md#publication-and-evidence-architecture`](a1.1d-5r3/architecture.md#publication-and-evidence-architecture).
