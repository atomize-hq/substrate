**Kind:** historical architecture record
**Stable ID:** `runtime-refactor-authenticated-runtime-projections-history`
**Status:** canonical historical record
**Canonical for:** the exact deferred R2-2E/F0a/F0b `Remaining authenticated runtime projections` heading and body extracted by D12
**Authority scope:** historical packet/architecture projection only; not a shared invariant, current schedule, gate, implementation, or successor authority
**Source provenance:** extracted byte-for-byte from [`../01-target-architecture.md#remaining-authenticated-runtime-projections`](../01-target-architecture.md#remaining-authenticated-runtime-projections), baseline lines 89–190 inclusive (`8176` bytes)
**Source SHA-256:** `119c62be15f6a72f717efb56b170675977f9bc6726c1423263f4d943834926f8`
**Substantive rewrite:** none; no repository-relative Markdown target occurs in the preserved source span, so no link rebase is required
**Supersedes:** canonical ownership of only this exact historical source span; the root heading remains a compatibility anchor
**Superseded by:** none
**Projection consumers:** [`../01-target-architecture.md`](../01-target-architecture.md), [`../index/README.md`](../index/README.md), [`../index/by-id.md`](../index/by-id.md), [`../index/by-kind.md`](../index/by-kind.md), [`../index/by-packet.md`](../index/by-packet.md)

# Authenticated runtime projection history

> **Authority boundary:** This file preserves the deferred R2-2E/F0a/F0b packet/history block that D7 intentionally left root-canonical pending later placement. It does not reclassify packet history as a shared invariant, reopen a completed family, or authorize implementation, proof, promotion, remediation, dispatch, or successor work.

<!-- exact-extracted-body:remaining-authenticated-runtime-projections:start -->
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

<!-- exact-extracted-body:remaining-authenticated-runtime-projections:end -->
