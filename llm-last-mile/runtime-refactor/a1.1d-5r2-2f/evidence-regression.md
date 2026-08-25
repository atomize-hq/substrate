**Kind:** evidence/regression
**Stable ID:** `A1.1d-5R2-2F-family`
**Canonical for:** A1.1d-5R2-2F family evidence and regression
**Status:** canonical
**Authority scope:** exact extracted family-local source bodies only
**Source span:** composite of the preserved root compatibility spans listed in Source provenance below
**Supersedes:** canonical ownership of the extracted source bodies; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R2-2F family evidence and regression

### R2-2 historical failed integration closeout and remaining-seam correction

The integration attempt began from source branch
`feat/internal-host-orchestrator-world-dispatch-bootstrap` at Route D commit
`7a3e6ee424e726e3600d39e40b64df2d02cfff73`, tree
`945923975dbe85cfee42e1acf76bb11fcc64ddf2`, zero behind and nine runtime commits ahead of published
`b503f05053daf902649924e73c62d7cd0f44e665`. It stopped as
`A1.1d-5R2-2 cross-document change required`. No integration runtime commit, closeout documentation
commit, runtime push, seam promotion, or R2-3/R2-4/R3 implementation followed Route D.

Routes A–D remain individually review-clean and preserved. Their current pre-E replay commits and
ordinary/full-index binary patch SHA-256 pairs are below. The named preservation branches retain
patch-equivalent reviewed commits; Route D's replay preservation points at the listed commit
exactly.

| Route | Current pre-E replay commit | Ordinary patch | Full-index binary patch | Equivalent preservation branch |
|---|---|---|---|---|
| A | `3bf59b30e4c7348b8ff6315e3eb3658d74af2552` | `1d04e54cabc705b2070b9d104e779278b79723602c24488246b310c1fa32fe7d` | `649004caed540970b72d265c44b9f05b1c0370175bd1c83aaa1c86d8443e78b9` | `feat/preserve-a1-1d-5r2-2-route-a-successor-1d04e54c` |
| B | `1b5219d5c7471f492865ab55e4efc0d1ab0cac49` | `28e6b9da35f96b5f93c49369cbde0eda77e9a145b54f9c412bae8e5a83871674` | `671d7f35d1d988b920863198f45cd94be0c2eb53fcd2ef9d94357e68f2d365e1` | `feat/preserve-a1-1d-5r2-2-route-b-security-reviewed-28e6b9da` |
| C | `0290b829ebaf222fcdc942678178a5e4545de62c` | `7c5a908a6760244d9ad6922dfcf7e944cfce3c723d994e279a3aeaded7a8ffa0` | `5b436d5dfeaf6e513cbaa306de65848cbe3d5b003fa8c00589be09e54b630277` | `feat/preserve-a1-1d-5r2-2-route-c-final-5b436d5d` |
| D | `6452d3a0650df4075c3ef720bad37920a8e5859d` | `278a679060d79dbf6c2728fda65122e9bb5b4b5d92384f8c1ea69dac57a8241b` | `7be7a562b983d42c57466818428df98e53f1e36eb38e985653ee4e5eea578f97` | `feat/preserve-a1-1d-5r2-2-route-d-replayed-6452d3a0` |

The stopped tree also contained four reviewed comment-only ShellCheck remediations: eleven inserted
`# shellcheck disable=SC2034` comments, no deletion or semantic token/command/variable/control-flow/
test/behavior change, and identical ordinary/binary patch SHA-256
`0cb0ef48c76336fdab70582ae1200568916febfb0e9e26fd0188cd79854d33c6`. They are preserved outside
this correction at commit `90ff6056195207598a2d77358f5206b1d5385700` on remote branch
`feat/preserve-a1-1d-5r2-2-shellcheck-comments-0cb0ef48`, rooted at exact Route D, across only
`scripts/substrate/{uninstall-substrate.sh,uninstall.sh}` and
`tests/installers/{prefix_propagation_r2_2.sh,world_provision_smoke.sh}`. They remain renewed-closeout
WIP and are not part of R2-2E, R2-2F, or this docs patch.

At that historical closeout, the blockers were exact:

1. PI-111: `world_gateway.rs` still used ambient config, broker policy, network-policy config,
   runtime-family inventory, Codex home, routing-disable toggles, and platform client selection.
2. PI-106/PI-107: ordinary current/global/workspace world-deps, runtime probe/install/sync,
   provision-deps, and post-provision sync still dropped A or called an ambient request builder.
3. Doctor truth: World/Host world-fs policy and Health/shim fixture/child composition could combine
   an A label with ambient B config/policy/inventory/dependency inputs or missing identity evidence.

Source closure also fixes the platform posture. R2-2E's Linux Gateway path uses the fixed
`/run/substrate.sock`. No authenticated macOS platform-endpoint source exists inside E, so the
macOS route must reject before client construction or `auto_select`; Windows and other contextless
Gateway cfgs reject before any authority-bearing selection. The production-compiled
`resolve_macos_gateway_client_endpoint` helper and the other macOS builders/resolvers are source-
closure-inspected but frozen R2-3 compatibility; they cannot be edited, deleted, called, or used to
select an endpoint in the authenticated route. R2-2F's
authenticated world-deps and truthful-doctor guarantee is Unix/Linux only. macOS, Windows,
fallback, and other non-Unix adapters remain R2-3 compatibility, unproven, or unavailable/fail-
closed, and cannot report authenticated A from E or F.

The unavailable `$feature-seam-extractor` was replaced by read-only GitNexus query/context/impact,
manual caller/cfg/source closure, and two fresh isolated read-only reviews. Initial reviewers
`/root/r2_2e_source_review` and `/root/r2_2f_source_review` found the disabled-route/Linux/macOS
gateway boundary, workspace/runtime/provision request chain, and doctor identity omissions. Fresh
rereviewers `/root/r2_2e_source_rereview` and `/root/r2_2f_source_rereview` returned CLEAN after the
exact corrections recorded in `03`/`04`; neither modified repository state. The F audit records the
shared ambient `build_agent_client_and_request` as HIGH and its trace-metadata variant as CRITICAL;
both are frozen. The earlier HIGH result for `resolve_current_inventory_view` is likewise a frozen
reuse boundary. Counts are diagnostic; the exact symbol/caller/cfg tables are binding.

R2-2E's PI-111 implementation/proof clause and F0/F0a/F0b/F0-HC are now complete. R2-2F owns the
unresolved PI-106/PI-107 production paths. The required order is Routes A–E -> F0/F0a/F0b/F0-HC
complete -> R2-2F -> renewed R2-2 integration closeout -> R2-3 -> R2-4 -> R3. The
renewed closeout is production-fix-free and reruns
the entire Routes A–F wall. `RG-HOME-01` and `RG-INSTALL-01` remain open. No privileged, macOS, or
Windows proof is claimed, and no seam is promoted.

### A1.1d-5R2-2E authenticated gateway closeout

R2-2E is **implemented, proof-complete, review-clean, committed, and preserved**. Its exact original
runtime identity is:

| Evidence | Exact value |
|---|---|
| Commit | `7e8e83802885c0ece93efcaacccc26503eeb6715` |
| Tree | `02a1a2f6b7e4be47ed9ec38537c805ae348c96b6` |
| Ordinary patch SHA-256 | `fb7b65b02cdac46857750b64ebd5ceab651e8e99910c05240b187c3e729444ff` |
| Full-index patch SHA-256 | `c712f467ac92efb0de7b524272479741ac6b318201cf45dbd994116ec0e8b862` |
| Preservation branch | `feat/preserve-a1-1d-5r2-2e-c712f467` |
| Exact manifest | `crates/shell/src/execution/platform/mod.rs`; `crates/shell/src/builtins/world_gateway.rs`; `crates/shell/src/execution/agent_inventory.rs`; `crates/shell/src/execution/policy_snapshot.rs`; `crates/shell/tests/world_gateway.rs` |

The owner/symbol delta is confined to the `handle_world_command` Gateway binding; the existing
gateway request-context, validation, action, authentication, and client-selection chain; additive
explicit-bootstrap-home inventory and network projection; and focused tests. Only
`synthesized_unavailable_response_without_context` was deleted. Routes A–D are patch-identical;
frozen direct-member surfaces, the managed-gateway secure-FD producer and receiver, its bundle
schema, and its lifecycle are blob-identical to E's parent. Canonical config/policy resolver
ownership remains unchanged.

The resulting path is `handle_world_command` -> authenticated carrier validation -> trusted A
root/principal binding -> canonical explicit config -> canonical effective policy -> canonical
policy snapshot/network policy -> explicit inventory -> request construction -> fixed Linux gateway
client. Config, effective policy, network policy, inventory, and committed account-home auth
provenance use explicit A. Ambient HOME, XDG, CWD, `SUBSTRATE_HOME`, `SUBSTRATE_ROOT`, `CODEX_HOME`,
`.codex`, `config.toml`, world toggles, and socket overrides cannot become authority; CWD remains
explicit workspace scope only. Linux remains fixed to `/run/substrate.sock`; network allow/deny
meaning, credential transport, gateway lifecycle, user/world capability, and policy meaning are
unchanged. Compatibility is not promoted.

Focused proof is exact: world-gateway classification 13/13; config resolution 21/21; effective
policy 14/14; policy snapshot/network 10/10; inventory 17/17; install-bootstrap context 8/8;
explicit `HostSessionAuthority` composition 1/1; new integration negatives 3/3; managed auth bundle
7/7; world-service gateway runtime 32/32; gateway receiver/server 18/18.

The baseline terms remain distinct:

1. **R2-2 historical starting baseline:** `1089 passed / 149 failed`.
2. **Clean Route D comparison baseline:** `1101 passed / 149 failed / 0 ignored`.
3. **Genuine post-E pre-F0 success observation:** `1114 passed / 149 failed / 0 ignored`.
4. **Known post-E interference observation:** `1113 passed / 150 failed / 0 ignored`.

The third value is reproducible but was not deterministic before F0 and therefore never became the
F comparison baseline. The later completed F0/F0a/F0b/F0-HC closeout establishes the deterministic
`1280/1235/45/0` F baseline without deleting or rewriting either post-E observation or the later
`1118/150` versus `1119/149` F0 evidence.

The Route D-to-E differential has `PassToFail=0`, `NewFail=0`,
`FailToChangedFailure=0`, and `FailToPass=0`, with the identical 149 failure-name set. After
normalization, only the recorded nondeterministic orchestration identifiers differ. No test was
removed, renamed, substituted, weakened, or newly ignored. The non-reproducible parallel
retained-member-stream failure is retained only as a historical observation; it is counted as
neither success nor regression.

GitNexus final detection is semantically contained to the approved owners and existing process
families: aggregate CRITICAL adjacency is diagnostic over-attribution, while exact edited existing
symbols were LOW and no new resolver owner, schema, capability, or execution-process family was
introduced. Final isolated read-only reviews `e_final_gateway_authority`,
`e_final_policy_network`, `e_final_credential_boundary`, and
`e_final_platform_regression_replacement` are CLEAN. The original platform reviewer is excluded
because it violated the required read-only process.

Linux fixed-socket and regression proof is recorded. macOS fails before client construction or
ambient forwarding; Windows/other entries fail before ambient authority selection; non-Unix static
cfg preservation is recorded. None is a native macOS/Windows product claim, and privileged product
proof remains R2-4-owned.

The managed-gateway secure-FD path is **landed, regression-proven, and unchanged by R2-2E**.
Direct-member Codex/UAA gateway adoption is separately **unresolved, transitional compatibility,
non-promotable, and owned by E3/D1/D3**. Accordingly `RG-CONFIG-02`, `RG-CONFIG-04`, `RG-UAA-02`,
and `RG-UAA-03` remain open and unchanged. PI-111's R2-2E clause is complete, but the full gateway
credential/config architecture is not.

### A1.1d-5R2-2F0 deterministic world-socket test isolation authorization

The F preflight baseline investigation is classified `TestIsolationDefectConfirmed`. The exact
minimal pair is `continue_world_worker_classifies_real_retained_member_turn_streams` plus
`b21_retained_production_handoff_claims_before_next_frame`: the target failed 12/20 parallel
two-thread runs and 0/10 serial runs, while fixed serial order passed. Both fixtures mutate the
process-global `SUBSTRATE_WORLD_SOCKET`; the first holds `world_env_guard` but is unannotated, while
the B2.1 competitor is `#[serial]` but uses an independent local environment guard. Because
`#[serial]` does not exclude unannotated tests, the target can observe the peer's private, restored,
or deleted socket. The competing mutation first appears at
`c519024bd91b6ca6e332d0b8881f7d13ded940e0`, before Routes A–E.

The complete shell library-test-process inventory contains 66 mutating call sites. Sixty-one require
migration: 49 orchestrator local-guard sites, two orchestrator manual restoration blocks, four
macOS platform closure-helper sites, two persistent-session direct sites, two routing direct sites,
one world-enable path manual site, and one world-gateway classification-test closure call. Four
other world-gateway RAII sites and one async-REPL site already acquire the same shared lock and
restore exact `OsString`/absence during unwinding. Shell integration tests only
use child `Command::env`/`env_remove` in separate test processes and require no cross-process lock.
Production socket readers remain frozen.

F0 is authorized only for test-compiled hunks in `crates/shell/src/execution/mod.rs`,
`crates/shell/src/execution/orchestrator_world_dispatch.rs`,
`crates/shell/src/execution/platform/macos.rs`,
`crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`,
`crates/shell/src/execution/routing/world.rs`, and
`crates/shell/src/builtins/world_enable/runner/paths.rs`, plus the classification-test helper/call in
`crates/shell/src/builtins/world_gateway.rs`. One RAII guard must acquire the existing
reentrant lock, capture exact prior state, hold it through socket/server/helper lifecycle and
cleanup, restore on return or panic, and release only afterward. Same-thread nesting restores in
stack order. `parking_lot::ReentrantMutex` does not poison; panic recovery and later acquisition are
mandatory proof. `#[serial]` alone, sleeps, retries, global single-thread forcing, assertion
weakening, readiness changes, and production edits are forbidden.

Required proof is prior-value/absence restoration, non-Unicode prior value on Unix, panic and
non-poisoning recovery, nesting, concurrent blocking/cleanup ordering, no peer-socket observation,
no leaks/deadlocks, at least 100 parallel and 20 serial exact-pair iterations, neighboring mutator
combinations, three default-parallel broad walls, one serial broad wall, stable inherited names and
normalized signatures, and unchanged deliberate retained-registration loser/parent behavior.
Routes A–E, production retained-registration, the managed secure-FD path, capabilities, policies,
and user behavior are unaffected.

At this authorization checkpoint, the exact combined harness candidate was preserved and
unrestored. It has since passed identity revalidation, corrected differential proof, fresh
containment review, canonical closeout, and runtime preservation, recording the deterministic
comparison baseline. At that historical checkpoint R2-2F was the next packet and renewed R2-2
integration closeout remained after F; the completed-F ledger below supersedes that status. R2-3,
R2-4, and R3 remained unstarted; R2-2 remained incomplete; no seam was promoted.

### A1.1d-5R2-2F0a — SUBSTRATE_HOME test isolation

F0's exact uncommitted candidate was preserved before diagnosis and removed from the source
worktree. Preservation ref `feat/preserve-a1-1d-5r2-2f0-blocked-candidate-69fe8c08` points to commit
`32513cf41f98006020702f1eca8d6931bc6708ce`, parent
`0b9f8474a414d3ed24aff9603e9ff68c37569bd5`, tree
`fa920cc04e62d379a6f999485cf93e4b436b44ec`, ordinary patch SHA-256
`6fce969756c371ccd6b20b5c6c35e9159100939c484f07dd7ecb1c66b573ebc0`, and full-index/binary
patch SHA-256 `69fe8c08d47787294b7f43ddad56bddf6cfd3d2d5ce99dce539e01b539925888`.
Its exact manifest remains `crates/shell/src/builtins/world_enable/runner/paths.rs`,
`crates/shell/src/builtins/world_gateway.rs`, `crates/shell/src/execution/mod.rs`,
`crates/shell/src/execution/orchestrator_world_dispatch.rs`,
`crates/shell/src/execution/platform/macos.rs`,
`crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`, and
`crates/shell/src/execution/routing/world.rs`. It is evidence only: no F0 runtime commit or closeout
exists on the source branch.

Focused F0 proof remains valid: the exact socket pair passed 100/100 parallel and 20/20 serial
runs; the four-test socket-neighbor matrix passed 20/20; prior absence, non-Unicode prior value,
panic restoration/reacquisition, nested stack restoration, concurrent exclusion/restoration, and
the deliberate retained-registration conflicting-child parent all passed. Final broad proof did
not close: the two exact candidate walls were `1118 passed / 150 failed` and `1119 passed / 149
failed`. The extra failure was
`execution::agent_runtime::tool_invocation_contract::tests::dispatch_contract_adapter_active_task_resolution_requires_supervisor_claim`,
with normalized panic `tool_invocation_contract.rs:3554: resolve exact B-owned acceptance authority:
open activated versioned authority layout`; it passes in isolation.

Read-only same-process reduction proves the selected minimal HOME pair:

1. target: `dispatch_contract_adapter_active_task_resolution_requires_supervisor_claim`;
2. competitor: `prompt_submit_continuity_prefers_persisted_session_contract`.

The canonical shell-library test binary ran both exact filters with `--test-threads=2`: target
failure 20/20, always the same authority-layout signature. The stable same-process
`--test-threads=1` harness controlled competitor-then-target and failed 0/10. Separate-process
sequential competitor-then-target and target-then-competitor executions pass. Stable libtest could
not force reverse same-process order, so this investigation claims no reverse same-process result.
Three other unannotated mutators—the second control continuity test and both agents-command
toolbox-status tests—each independently failed the parallel pair 20/20. Non-source transition
tracing recorded separate thread identities and exact ordering: competitor sets its private test
HOME; target sets a distinct owner-only test HOME; competitor removes HOME; target fails while
opening its expected activated authority layout. Separate root device/inode identities prove this
is not one reused directory. No product state was read, changed, or cleaned.

Commit `f5a150f94d585b1f55ec0067845cd5d715773c78` first adds the selected unannotated competitor and
its manual `with_store` HOME mutation. The target and its HOME-mutating fixture appear later at
`83101dcbcc750e6e8fb8979bea19f1f777792188`, which is therefore the first source commit where the
exact pair coexists and the first source-proven pair-causal commit. A same-commit execution replay
was attempted in an isolated worktree but
stopped before linking because the host `/tmp` tmpfs had insufficient free space; only that exact
test-owned worktree/build tree was removed. The causal claim is source-history-proven, not presented
as an executed first-red-commit result.

The complete shell-library mutation inventory contains 435 mutating test functions across 19 files.
Five `with_store` families account for 223 dependent tests: auto-attach 10, control 11, state-store
183, tool-contract 12, and host-inbox 7. Two agents-command helpers account for five more. The
remaining direct/local-guard/manual owners are shim-doctor 1, world-deps 2, manager-env 1,
world-gateway 11, agent-inventory 6, direct state-store tests 2, direct tool-contract tests 10,
HostSessionAuthority store tests 1, config-model 11, environment scripts 1, invocation 2,
orchestrator dispatch 106, routing builtin tests 4, settings 8, and async REPL 41, with every test
function counted once in the 435 total.
All recognized direct owners are `#[serial]`; the only unannotated mutating callers are the four
named above. Existing restoration varies: some local guards preserve exact `OsString`, others store
UTF-8 `String`, and manual set/remove helpers are not panic-safe. `#[serial]` cannot exclude the
four unannotated mutators.

Integration occurrences are process-separated. Parent HOME mutation is confined to
`crates/shell/tests/shim_deployment.rs` (nine serialized callers),
`crates/shell/tests/agent_successor_contract_ahcsitc0.rs` (one caller), and
`crates/shell/tests/support/mod.rs` (one ignored one-test binary plus one serialized caller). Other
integration occurrences are read-only or configure spawned children with `Command::env`/
`env_remove`. They require intentional per-binary disposition, not a cross-process lock. The
non-Unix production `world_enable` HOME assignment remains product code and is frozen; tests must
hold their test boundary outside it.

Lock-topology source closure selects **Option A: one process-global authority-environment lock**.
At least 88 shell-library tests depend jointly on HOME and world socket: world-gateway 5,
agents-command 2, invocation 2, orchestrator dispatch 39, and async REPL 40. Existing helper paths
already acquire HOME then socket in some modules and socket then HOME in others. Separate locks
would therefore admit mixed authority snapshots and lock-order inversion. The combined guard must
capture exact `OsString`/absence, support safe same-thread nesting with stack-order restoration,
make poison/non-poison recovery explicit, hold through dependent async/process lifetime and cleanup,
and restore before unlock. `#[serial]` remains supplemental only.

Async cleanup source closure expands the known six locations to nine exact F0-migrated socket
fixtures in `crates/shell/src/execution/orchestrator_world_dispatch.rs`:

1. `dispatch_contract_cancel_world_work_ephemeral_routes_exact_active_task_over_execute_cancel`;
2. `active_ephemeral_terminal_wait_allows_multiple_waiters_to_observe_same_terminal_truth`;
3. `active_ephemeral_terminal_wait_registration_guard_releases_non_happy_path_registrations`;
4. `active_ephemeral_terminal_truth_guard_publishes_failed_terminal_truth_on_drop_after_start`;
5. `dispatch_contract_cancel_world_work_ephemeral_retry_reuses_shared_terminal_truth`;
6. `dispatch_contract_cancel_world_work_ephemeral_fails_closed_when_execute_cancel_is_not_delivered`;
7. `continue_world_worker_classifies_real_retained_member_turn_streams`;
8. `continue_world_worker_dispatch_returns_real_typed_internal_outcome`;
9. `dispatch_contract_fork_world_worker_rolls_back_child_when_lineage_persist_fails`.

Each declares a socket-owning Tokio server before or alongside the environment guard and calls
`server.abort()` without awaiting termination; one merely yields once. The authorized correction is
strictly test-only: abort, await confirmed task termination/cancellation, complete and confirm
fixture-owned socket/task cleanup, restore environment, then release the lock. Reverse declaration/
drop order is not termination proof. The already-awaited server at the earlier compatibility test
and unrelated aborts without a guarded socket lifecycle are source-reviewed exclusions.

GitNexus reports `world_env_guard` CRITICAL at 35 direct/70 total dependents and five affected
process groups; tool-contract `with_store` HIGH at 12 direct dependents; auto-attach/control/
host-inbox helpers MEDIUM at 10/11/7; agents-command helpers LOW at 4/1. State-store and the nine
large-module test symbols are graph-under-resolved, so their complete source caller/body audit is
binding. These breadth labels authorize no production edit.

The isolated documentation-worktree GitNexus refresh changed only generated symbol/relationship
count lines in `AGENTS.md` and `CLAUDE.md`; those exact lines were restored before review and commit.
Record: `GeneratedIndexDriftRemediated`.

The exact combined test-only allowlist is the preserved seven-file F0 manifest plus the HOME-only
shell-library files named in `03`/`04` and all three inventoried parent-mutating integration-test
families in `crates/shell/tests/{shim_deployment.rs,agent_successor_contract_ahcsitc0.rs,support/mod.rs}`.
No
other integration file, production symbol, resolver, socket owner, readiness path, retained-worker
or retained-registration validator, managed secure-FD path, capability, policy, or user behavior
may change. F0a and the F0 cleanup correction are not implemented by this authorization.

Combined F0/F0a/F0b closeout requires the F0 socket pair and F0a HOME pair at 100 parallel/20 serial minimum,
a mixed HOME/socket neighbor matrix, exact prior absence/value/non-Unicode restoration, panic,
poison/recovery, nesting, bounded subprocess inheritance, termination-before-restoration, and zero
leak proof. Three exact final-candidate default-parallel broad walls plus one serial wall must have
one stable inherited failure-name/signature set and no unexplained count variance. Historical
`1114/149`, `1113/150`, `1118/150`, and `1119/149` observations remain distinct evidence and are not
lowered, erased, or substituted for the deterministic F baseline.

`TestIsolationDefectConfirmed` remains the historical classification. The exact combined candidate
later passed restoration and identity revalidation, corrected differential proof, fresh containment
review, canonical closeout, and runtime preservation. At that historical F0a checkpoint F and the
later nodes remained unstarted and no seam was promoted; the completed-F ledger below supersedes
that packet status.

### A1.1d-5R2-2F0b — deterministic renderer-output test isolation

The post-fork-remediation F0/F0a candidate was preserved before this read-only source-closure and
removed from the source worktree. Preservation ref
`feat/preserve-a1-1d-5r2-2f0-f0a-post-fork-remediation-9e012c68` points locally and remotely to
commit `07ce368cc02d5a4e80db9c1ca80efee93009d064`, exact parent
`d52ad6de2db7f5ed7fa0ee68ff2e3fc98332021c`, tree
`51737000a17954a605f091f00f9b0c71f38a596c`, ordinary patch SHA-256
`5530e58a7289db4e9a62fc9934510b87add71853728a0311af2160328af3cbeb`, and full-index/binary
patch SHA-256 `9e012c68ea33107443cd38f2051aa40aae9f0f03d5c6abb4c630d418b0d7dee2`.
Its exact 27-file manifest is
`crates/shell/src/builtins/{shim_doctor/report.rs,world_deps/mod.rs,world_enable/runner/manager_env.rs,world_enable/runner/paths.rs,world_gateway.rs}`,
`crates/shell/src/execution/{agent_inventory.rs,agents_cmd.rs,config_model.rs,env_scripts.rs,host_inbox_materialization.rs,invocation/tests.rs,mod.rs,orchestrator_world_dispatch.rs,platform/macos.rs,routing/builtin/tests.rs,routing/dispatch/world_persistent_session.rs,routing/world.rs,settings/tests.rs}`,
`crates/shell/src/execution/agent_runtime/{auto_attach.rs,control.rs,host_session_authority/store_tests.rs,state_store.rs,tool_invocation_contract.rs}`,
`crates/shell/src/repl/async_repl.rs`, and
`crates/shell/tests/{agent_successor_contract_ahcsitc0.rs,shim_deployment.rs,support/mod.rs}`.
The preservation commit records the per-file SHA-256 fingerprints.

The original pre-remediation candidate remains preserved locally and remotely at
`feat/preserve-a1-1d-5r2-2f0-f0a-broad-blocker-9978655e`, commit
`78353f3383fdd80c176fe5638328c10f2805dcf0`, exact parent
`d52ad6de2db7f5ed7fa0ee68ff2e3fc98332021c`, tree
`55476fa63a6ac2d116758a0aad5476ffea655a3f`, ordinary patch SHA-256
`577a34da903992d0566fa51d164410caeca98592dd486f5c1d5520e34baaa5a1`, and full-index/binary
patch SHA-256 `9978655e0f4095880c49d863a93b8fc99fb2b769afd42554c758b3970103f710`.
Both refs are evidence only. Neither candidate is committed to or restored on the source branch,
and no F0b runtime implementation begins in this planning packet.

The mandatory post-fork broad wall was not deterministic:

| Run | Passed | Failed | Ignored | Exact distinguishing result |
|---|---:|---:|---:|---|
| Parallel wall 1 | 1134 | 146 | 0 | inherited failure set |
| Parallel wall 2 | 1134 | 146 | 0 | identical to wall 1 |
| Parallel wall 3 | 1133 | 147 | 0 | added `public_prompt_renderer_renders_bounded_structured_fallback_when_decode_fails` |

The third wall invalidates canonical closeout even though the first two agree. Its complete
captured output was
`".[codex] task_progress: fields=alpha, beta, gamma (+1 more)\n"`. Read-only reduction proves
that `capture_stdout_once` creates a pipe, duplicates stdout, replaces process fd 1 with `dup2`,
runs the closure, flushes, restores fd 1, and reads the pipe. Libtest's parallel reporter writes its
progress `.` through the same process-global descriptor during that interval. The current test
then searches for a line beginning with `[codex]`; the reporter prefix makes the only line begin
with `.`. `#[serial]` cannot correct this boundary because it serializes only enrolled tests,
not libtest's reporter.

The causal matrix is exact:

| Control | Result | Interpretation |
|---|---|---|
| Forced same-process parallel | 376 passed / 124 failed across 500 | Every failure had the broad-wall name and normalized signature |
| Isolated target | 100/100 passed | Renderer behavior itself is stable |
| Same-process serial, identical neighbors | 100/100 passed | Removing descriptor overlap removes the failure |
| Separate-process control | 100/100 passed | Process isolation removes shared-fd contamination |
| Parallel pretty reporter | 99/100 passed | Reporter scheduling can enter the fd-capture interval |

Candidate introduction is unnecessary: the target and capture helper are byte-identical to clean
E. This establishes `TestIsolationDefectConfirmed`, not product regression and not random
flakiness.

Read-only source closure is complete for the renderer boundary:

- private `#[cfg(unix)] PublicPromptRenderer` owns `new` and `render`;
- exact production construction/call paths are
  `run_hidden_owner_helper_startup_prompt_stream_with_projection` and
  `run_public_prompt_command`; each calls `render` for normal envelopes and a failure envelope;
- `capture_stdout_once` has exactly one caller,
  `public_prompt_renderer_renders_bounded_structured_fallback_when_decode_fails`;
- `capture_stderr_once` has exactly one caller,
  `public_prompt_renderer_renders_bounded_structured_stderr_fallback_when_decode_fails`;
- `capture_stderr_once` performs the same process-global replacement on fd 2 and therefore has
  the same structural race even though it did not fail in the recorded wall;
- no other raw `dup`/`dup2` capture helper exists in this renderer/test ownership boundary.
  Unrelated raw-fd operations elsewhere are deferred evidence and do not widen F0b.

Current production behavior is frozen exactly. JSON envelopes serialize to stdout and propagate
serialization/write failure with the existing context; Accepted emits nothing; Completed emits to
stdout; Warning and Failed emit to stderr; Event selects stderr only for the exact stderr kind and
otherwise stdout; decoded lines and bounded structured fallback retain their exact bytes and
newlines. Existing completed/event/warning/failure write-error suppression and all existing flush
result treatment remain unchanged. Stream selection and locking remain lazy and ordered as today;
an implementation must not eagerly lock both streams. Redaction and bounded fallback remain
unchanged.

GitNexus and source closure give this containment:

| Symbol | GitNexus upstream impact | Source-closure disposition |
|---|---|---|
| `PublicPromptRenderer` | LOW; 0 direct, 0 process, 0 module | Private Unix production type; retained |
| `PublicPromptRenderer::render` | MEDIUM; 4 direct, 39 total; one `handle_agent_command` process family; Agent_runtime direct, Execution indirect | Only existing production symbol permitted a mechanical delegation edit |
| `PublicPromptRenderer::new` | HIGH; 19 direct, 43 total; two process labels; three modules | Generic-`new` graph over-attribution; exact source has four construction sites; signature and body frozen |
| `capture_stdout_once` | LOW; one direct test caller; zero process | Delete after stdout test migration |
| `capture_stderr_once` | LOW; one direct test caller; zero process | Delete after same-owner stderr test migration |
| two exact fallback tests | LOW; zero process | Migrate only capture mechanism; names and behavioral assertions retained and strengthened to complete bytes |

There is no CRITICAL impact. The proposed one-file allowlist is exact:
`crates/shell/src/execution/agent_runtime/control.rs`. It may EDIT
`PublicPromptRenderer::render` only for delegation; ADD a private Unix-only explicit-writer
rendering core, private output-sink adapter, or equivalent bounded internal symbols; DELETE
`capture_stdout_once` and `capture_stderr_once`; and EDIT only the two exact fallback tests named
above. `PublicPromptRenderer` remains private, `PublicPromptRenderer::new` and both production
caller bodies are frozen, and no other file, symbol, dependency, or test is authorized.

The future tests provide their own in-memory stdout and stderr writers and assert the exact complete
selected bytes and exact empty nonselected buffer. They may not strip, search around, or tolerate
an unrelated prefix. Production `render` remains the sole entry point and delegates through the
default real streams with byte-for-byte equivalent bytes, stream choice, order, newline, flush,
errors, redaction, and fallback. A public API, output transport/schema, process-global output lock,
writer registry, side table, environment-selected sink, reporter filtering/suppression, sleeps,
retries, larger timeouts, test-thread reduction, whole-suite serialization, `#[ignore]`, test
removal/rename/substitution, or assertion weakening is forbidden. `#[serial]` may remain
supplemental only.

F0b is test-isolation infrastructure, not a product rendering change. It changes no production
execution, user-visible behavior, world, policy, credential, secure-FD, gateway, receipt,
supervisor, retained-worker, placement, caging, lifecycle, or capability semantic and promotes no
seam. F0/F0a/F0b/F0-HC are complete: the preserved candidate was restored and identity-checked,
the corrected transition audit and fresh containment review are CLEAN, and canonical closeout and
runtime preservation are complete. At that historical F0b checkpoint F and the later nodes remained
unstarted. The binding sequence was Routes A–E → F0/F0a/F0b/F0-HC complete → F → renewed R2-2
integration closeout → R2-3 → R2-4 → R3. The completed-F ledger below supersedes that packet
status.

For the Route A successor clause of `R2-RUNTIME-01`, the authorized Unix-only import cfg set also
includes the policy test module's sole `tempfile::TempDir` import; no other import or module gate is
authorized. Health proof must combine fail-closed syscall/path interception with a complete B-tree
before/after snapshot. Host proof must show the expected A dependency scaffold, no B dependency
artifact, and complete B-tree preservation across both the installed-witness dispatch and the
direct-repository no-witness rejection. This test-only clarification supersedes the narrower import
wording in the row without changing any production byte, Route B boundary, or R2-3 ownership.
The final portable-parent paragraph below extends that gate's sole successor list only with the two
named existing test setup hunks; it supersedes the row's earlier closed four-category enumeration.

The final Route A test-parent remediation is bounded against preserved successor patch SHA-256
`0a53e8b60326e21b4237392bfa99671fa8c19c0cd45eb4c573cae5140e808720` at commit
`ad139789ef317b978e96c36ee8170dd503bff8a4`. Only
`config_show::config_current_show_uses_declared_prefix_under_conflicting_ambient_home` and
`policy_discovery::policy_current_show_uses_declared_prefix_under_conflicting_ambient_home` may
replace their Linux-only `/run/user/<effective uid>` test-parent fallback. Each uses nonempty
`XDG_RUNTIME_DIR`; otherwise it resolves the current account's nonempty home and uses `.cache` or a
repository-established fixture subdirectory beneath that same account home. This is not a third
source, and the command's ambient/conflicting HOME cannot select it. Each creates that parent as
required and retains a unique `0700` selected-A root. Missing sources fail setup explicitly; `/tmp`, `/var/tmp`, `/run/user/<uid>`, CWD,
ambient `SUBSTRATE_HOME`, and B cannot substitute. Both pre-edit impacts are LOW with zero callers,
processes, or modules. The manifest remains exactly thirteen files because both tests are already
present; every production fingerprint and every other test fingerprint remains fixed. The final hash
is established after remediation, with any other hunk or a fourteenth file rejected as
`SuccessorPatchScopeMismatch`. Native Darwin/Windows proof remains unavailable when the Apple SDK or
MSVC tooling is missing and is recorded as unavailable rather than success or product regression.

R2-0 cross-document checkpoint rules are: inventory counts and owner/packet columns must agree;
every R2 packet has an exact production/test allowlist; R2-4 has no production allowlist; every R3
action above stays exclusive to R3; host and Lima/WSL paths/principals are never equated;
`InstallBootstrapContextV1` commitment framing is canonical and unambiguous; and the serial graph is
acyclic. At that historical R2-0 checkpoint, the next packet after a clean R2-0 commit was
**A1.1d-5R2-1 — Host context construction and Unix dev propagation**.

**Frozen planning validation record:** the inventory contains exactly 118 unique, contiguous rows
and dispositions: 19 owned by R2-1, 41 by R2-2, 35 by R2-3, one by R2-4, and 22 by R3. Each row
has exactly one approved edge class and one packet owner; the class totals are 16
`ContextConstruction`, 26 `ExplicitHostPropagation`, five `PrivilegeBoundaryPropagation`, 17
`PlatformMapping`, 13 `GeneratedProjectionConsumption`, 15 `DiagnosticProjection`, 22
`R3CleanupOnly`, and four `OutOfScope`. The R2-0-era frozen DAG was historically
R1 -> R2-0 -> R2-1 -> R2-2 -> R2-3 -> R2-4 -> R3. The corrected canonical DAG replaces the
R2-2 outgoing edge without changing inventory ownership; F0/F0a/F0b and F0-HC are the completed
proof prerequisites: R1 -> R2-0 -> R2-1 -> R2-2 Routes A-E -> F0/F0a/F0b/F0-HC complete ->
R2-2F -> renewed
R2-2 integration closeout -> R2-3 -> R2-4 -> R3. Mechanical validation covers row-ID,
field-count, class, owner, packet-count, table-column, fence-pair, relative-link, and allowlist-path
checks across 43 tables/749 pipe rows, 134 fence markers, and 12 relative links, plus
`git diff --check`, `cargo fmt --all -- --check`, and GitNexus change detection. The
GitNexus index was refreshed at the starting commit; only generated count lines changed during
refresh and those exact lines were restored, while final change detection reports six documentation
files, zero affected execution flows, and LOW risk. Successive isolated read-only review rounds found and
closed missing CLI pre-bootstrap ordering, automatic shim/repair paths, dev sudo and provisioning
children, shim/release/Windows/Lima cleanup ownership, shim-telemetry and replay factory callers,
two-stage Lima realization, Windows timeout-kill ownership, invalid native-smoke assignments,
symlink-shim invocation recovery, generated preexec and Codex-home projections, direct
world-deps/doctor/config/policy/gateway consumers, exact helper-versus-system-tool sudo carriers,
Lima known-hosts placement, and forwarding unlink/drop/timeout ownership.
The post-push fresh authority audit additionally closed source-layout omissions for the Unix dev
A/bin symlink and Windows release A/bin copy, and made current Unix account+UID/Windows account+SID
equality mandatory for every internal carrier, including forged-valid-carrier negative proof.

The R2-1 preimplementation trigger correction preserves that inventory and DAG. It authorizes the
hidden authenticated `--install-bootstrap-home-v1` action only inside the existing R2-1 production
files and adds only `crates/shell/tests/world_deps_home_scaffold_wdh3.rs` to the focused test
allowlist. That file may construct the shared IH carrier/current Unix principal, replace historical
`--version` bootstrap triggers, add version nonmutation proof, and perform the exact one-to-one rename
`test_bootstrap_scaffolds_deps_on_version` ->
`test_install_bootstrap_action_scaffolds_deps`. This is a canonical trigger rename, not removal or
substitution: the old/new fixture and scaffold assertions are identical and only the command trigger
changes; all other historical test names and all R1 security assertions remain unchanged. No
world-deps production symbol, cleanup/R3 behavior, user-facing feature, or capability is authorized.
The subsequent lifecycle allowlist review closed the remaining Windows implementation-surface gap:
R2-3 alone may add the exact `windows-sys` 0.52 features `Win32_Security`,
`Win32_Storage_FileSystem`, `Win32_System_Threading`, `Win32_System_Com`, and `Win32_UI_Shell` in
`crates/shell/Cargo.toml`; the last two are only for token-bound Known Folder resolution. Dependency,
version, target-scope, extra-feature, package-graph, and lockfile changes remain outside that authority.
The next fresh source audit corrected PI-030 to model the ACL bridge's actual mode/target/group-only
boundary and froze its three accepted tuples, then added PI-115 for the forwarder-to-WSL spawn and
PI-116 for physical-shim manager-manifest consumption. Those rows require PM-derived WSL child
argv/environment and A-scoped generated manager data; neither ambient target/`WSLENV` nor ambient
home/repository manifest fallback is authority.
The same review froze the only dependency edits: R2-1 adds existing `base64` 0.22/workspace `sha2`
to `transport-api-types`; R2-3 adds the existing `transport-api-types` 0.2.8 path edge to the
backend factory, forwarder, and physical shim. Only those four package dependency lists may change in
`Cargo.lock`; the shim list may additionally gain existing `windows-sys 0.52.0`, while its existing
`nix` dependency adds feature `user` without a lock hunk. No package/version/checksum change is
authorized. The physical shim's exact `context.rs` witness functions are the only added
OS-observation owner; they construct the shared IH and may not define a local principal/context
type or raw-FFI/ambient fallback.
The transport-identity review first froze SSH UDS as the only V1 normal-product target. A later
lifecycle review correctly found that activating it in R2 would newly reach the current
constructor's socket unlink, `StreamLocalBindUnlink`, timeout kill/wait, and handle-drop cleanup.
The corrected sequence therefore freezes PM and the A-scoped target in R2-3 but stops before
forwarder launch; R3 alone activates it after PI-101/PI-113/PI-114 lifecycle ownership lands.
Existing VSock and SSH-TCP code is not removed, but it remains diagnostic/test-only and cannot
satisfy PM, product, or native R2 proof.
The next fresh authority/platform review found two remaining ambient platform-control selectors:
macOS SSH config lookup still honored `LIMA_HOME`, and Windows forwarder config/log paths still
honored `LOCALAPPDATA`. The corrected PM now commits `host_platform_control_root`: Lima derives it
from the committed host principal's account-database home and scrubs/overwrites child
`HOME`/`LIMA_HOME`; WSL derives it from the current-token Known Folder plus the exact canonical
SID+registered-distro+machine-ID+pipe scope digest. Windows config/log projections are A-scoped and
explicit; the shared PID target is control-root-scoped. Ambient conflicts never select either
platform, and no R2 path gains replacement, rollback, removal, termination, or deletion authority.
The final platform allowlist audit also corrected one wording inconsistency: the R2-3 `Cargo.lock`
allowlist now names the three internal `transport-api-types` package-list edges and the already
frozen existing `windows-sys 0.52.0` shim package-list entry, while still forbidding every
package/version/checksum change.
The final authority review found two trace-specific ambient reads that the broad CLI/shim rows had
not made independently reviewable. The later preimplementation impact review then exposed a
CRITICAL shared-setter fan-out spanning shell, physical shim, replay/platform, tests, and unrelated
compatibility callers. The corrected sequence therefore leaves `set_global_trace_context`
signature and behavior unchanged. PI-117 gives R2-1 only an additive/default-preserving explicit
product-bound representation/constructor, shell binding to exact `A/trace.jsonl` plus policy Git A,
and additive `get_policy_git_hash_at`; all unmigrated default callers retain named non-promotable
`LegacyAmbientCompatibility`. PI-118 plus already-frozen replay/platform migration gives R2-3 the
physical-shim binding, compatibility removal/unreachability, and final global unbound-init failure
rule. No caller-identity side table is allowed. R2-1 CRITICAL breadth is authorized only when the
setter/default behavior and shim/replay/platform files and outputs are unchanged, only shell selects
the explicit posture, and final detection finds no new process family. Trace writer, serialization,
flush, rotation, retention, span/replay, environment-hash, and policy semantics remain frozen.
No reviewer modified the repository. Because the built-in reviewer-thread allocator would not
admit new threads, the required fresh isolated read-only `default` review turns used three distinct
existing built-in identities; each re-read the current diff/source rather than carrying forward its
prior verdict. The terminal verdicts are `/root/final_authority_security_retry` — **CLEAN** for
host-context authority/security,
`/root/final_lifecycle_ownership_retry/final_control_authority/final_control_lifecycle_retry` —
**CLEAN** for installer/uninstaller lifecycle and R2-versus-R3 ownership, and
`/root/final_lifecycle_ownership_retry/final_control_authority/final_control_platform_retry` —
**CLEAN** for cross-platform mapping, allowlists, regression assignments, and native-proof honesty.
No runtime implementation or native platform proof can be inferred from this record.
## A1.1d-5R2-2F0-HC empirical closure record

### Environment-inventory correction evidence

The current correction began from clean source branch
`feat/internal-host-orchestrator-world-dispatch-bootstrap` at
`a6c4a5a52c7e33d4efcc18f70dc2b7301ab0fcca` (tree
`17b83d26b3f9a6258d0234461543528c269338bb`), with source remote
`9a3b54edcf1560d50b6aafda7293699fce85c719` and divergence 0 behind / 10 ahead.
Before audit work, the exact 44-file partial harness WIP was preserved on
`feat/preserve-a1-1d-5r2-2-harness-xdg-blocker-6d1e303f` at commit
`8b203b6ef9bda407a1299f8208bdd0ef1a7a1997`, parent the exact source commit, tree
`ba80d26ba2c6a69036c4eb77e212f29e908dc70a`, ordinary patch SHA-256
`6d1e303f29b18f966cc61f710cae6bf84eed03bd890a0bce3e9f8d5c1d31b0d3`, and full-index/binary
patch SHA-256 `ec0d1a091ad324777f4d0447fa36eece6f64d7b883e7a1b6c1562d4283248023`.
Local/remote commit, parent, tree, 44-file manifest, and hashes matched before the source worktree
returned clean to the immutable commit. The earlier donor remains separately preserved with
ordinary patch `5530e58a7289db4e9a62fc9934510b87add71853728a0311af2160328af3cbeb` and
full-index patch `9e012c68ea33107443cd38f2051aa40aae9f0f03d5c6abb4c630d418b0d7dee2`.

The actionable correction classifications are `HarnessClosureIncomplete` and
`ControlPackStateMismatch`. The prior exact-name pass found direct
`std::env::{set_var,remove_var}` literals but did not source-close wrapper arguments. Its manual
dynamic-wrapper repair recognized twelve tests, yet treated the settings family as a
`SUBSTRATE_ROOT` dependency without extracting `SUBSTRATE_OVERRIDE_ANCHOR_MODE`,
`SUBSTRATE_OVERRIDE_ANCHOR_PATH`, or `SUBSTRATE_OVERRIDE_CAGED`; it also omitted the sole
`AmbientSelectionGuard::set` callsite and therefore `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, and
`XDG_STATE_HOME`. GitNexus's `cfg(test)` graph had no incoming edge for that guard and could not
substitute for lexical callsite closure. Immutable source proves the XDG values are parent-process
mutations, not `Command::env` projections. Independent correction review then rejected the first
405-call evidence manifest because it omitted `with_store`, `ProjectionEnvGuard::capture`,
`update_world_env`, and the macOS `restore` family; the later 604-row pass also omitted
`install_bootstrap_projections`, `apply_world_root_env`, `export_runtime_config_env`, and four
additional `with_store` families. These were callsite-closure defects, not new environment names.

Two independent methods now reconcile: lexical/static inspection accounts for all 395 direct
parent-mutation primitive calls, including 73 dynamic-key calls; wrapper/callsite plus GitNexus and
source closure resolves the 86-name universe and every dependent test. Exactly 74 names are
parent-mutated and 12 are child-only or read-only. The parent-mutating union is corrected from 518
to 534 tests in the same 35 files. A2 is corrected from 113 to 129 direct tests: 116 additive plus
13 existing-source F0a tests. The prior 518 was 506 same-file/literal results plus twelve disjoint
manual rows. Corrected closure adds eighteen real mutators and removes two false positives from
ambiguous same-file bare-name fanout. The additions are
`authenticated_gateway_projection_ignores_named_ambient_roots_and_uses_account_db_codex_home`,
`codex_auth_projection_errors_redact_committed_account_home`,
`update_world_env_sets_enabled_flags`, `update_world_env_sets_disabled_flags`, and the fourteen
`with_test_mode`-only PTY tests enumerated in `02`. The removed non-mutators are
`b1_task_acknowledgement_rejects_nonstart_eof_mismatch_and_repetition_before_persist` and
`b1_retained_acknowledgement_joins_exact_request_and_rejects_terminal_or_drift`;
the two settings tests `resolve_world_root_respects_env_when_no_configs` and
`resolve_world_root_env_overrides_global_config` were already present but now map to their exact
three override names.

GitNexus reports the newly allowlisted Codex-auth test and each of the fourteen newly closed PTY
test symbols LOW with zero callers, zero affected processes, and zero affected modules. The shared
`with_test_mode` helper retains the prior HIGH adjacency result. These results authorize only the
exact test/harness boundaries in `02`; production PTY classification, broker behavior, and gateway
auth resolution remain frozen.

Full closure also reclassifies already-listed `SUBSTRATE_SHELL` as parent-mutated.
`execution/routing/dispatch/tests/host_replay.rs::async_repl_host_commands_record_replay_context`
passes it to `execution/routing/test_utils.rs::{set_env,restore_env}`, and
`crates/trace/src/span.rs::SpanBuilder::new` is the overlapping stable reader. The test's three
early returns precede manual restoration, while `set_env` captures `Option<String>`; current
protection is therefore early-return/unwind unsafe and loses a prior non-Unicode value. The test
and file were already in the 116-test A2 manifest and 35-file union. The future migration is exact
`OsString`/absence RAII under `UnifiedProcessStateLock`; the trace reader remains frozen.

The two macOS tests already hold `world_env_guard`, but their `snapshot`/`restore` helper captures
only `Option<String>` and restores only `SUBSTRATE_WORLD` and `SUBSTRATE_WORLD_ENABLED` even
though production-frozen `update_world_env` changes six names. Their exact future allowlist is
`execution/platform/macos.rs::platform_tests::{snapshot,restore,update_world_env_sets_enabled_flags,update_world_env_sets_disabled_flags}`;
all six prior values or absence must be restored exactly under the same coordinator. GitNexus
reports both helpers LOW with two direct test callers and zero processes, and both tests LOW with
zero callers/processes.

The XDG negative-authority semantics are frozen: conflicting ambient XDG roots remain installed;
authenticated A and the account-database Codex home remain authoritative; ambient XDG values gain
no configuration, credential, or path authority; all assertions remain; and exact prior
`OsString` values or absence are restored. The exact six corrected names and their test-only
file/symbol/test allowlist are binding in `02`. All six belong to the existing A2/E1
`UnifiedProcessStateLock`; no XDG-specific lock, production owner, side table, reader edit, or
behavior change is authorized.

The prevention rule is mechanical: direct primitives and wrapper definitions/callsites are
separate inventories; literal, constant, array/table, loop, and parameterized arguments must all
resolve; dynamic names fail if unclassified; parent mutation and child-only projection are checked
separately; and every mutating test plus overlapping stable reader must map to an exact migration
or isolation disposition. The deterministic validator is evidence-only outside tracked repository
state and authorizes no seventh control-pack file.

The validator binds the normalized 395-call primitive, 1,005-call resolved mutation, and 534-test
manifests at SHA-256
`df0254488f37d4f53f05c81bbbd47c05bbbe410496d465827daade80922e0f4a`,
`350ccd1876ecc07cced81df89b523e3d24f9674f5dc6a5e33f052d65f4489772`, and
`de7e2bc43c0fcab74096dc272340726c919aeb72004fd0208ab5b4dffb802f4a`, respectively. It also
freezes the reproduced 506-test defective scanner at
`a9fbbe89128a2cbe9c341a0ec78a9fb256840d5ef65ad78f47e9d404e20cafed` and 43 dynamic mutation-sink owners at
`133e28a495e5be69016fb7f6483018cfb406b44c8ca5e2d8779fdf82ded0c66c`, parses fixed projection
and local-key tables from source, and runs negative perturbation checks. The rejected 604-row
manifest remains historical evidence, not a completeness claim. The validator parses the 35-file,
116-test, and 13-test canonical lists; verifies all 129 named source test
symbols exactly once; and checks the host-replay early-return, routing-helper capture, and
`SpanBuilder::new` reader closure. Counts without exact normalized manifests are not sufficient.

### Preflight and preservation

The earlier F0/F0a/F0b preflight remains preserved as historical evidence; the current correction
preflight and 44-file preservation are recorded above. That earlier preflight was:

- source branch `feat/internal-host-orchestrator-world-dispatch-bootstrap`; local HEAD
  `06c928443a93579899e5e5827b151f530e1be933`; tree
  `dfe15d93366655e9855bca40ad0607887f545637`; upstream/remote
  `c079aeed748120cff9b03079e996d00109a05800`; divergence 0 behind / 10 ahead; clean index,
  worktree, and untracked set;
- published F0b docs commit `c079aeed748120cff9b03079e996d00109a05800`, tree
  `905cf110f648188b1a2e9a0087b4ffbb4c89e168`, subject
  `docs: authorize deterministic renderer test isolation`;
- replay preservation `feat/preserve-a1-1d-5r2-2e-replayed-f0b-docs-c079aeed` resolves locally and
  remotely to `06c928443a93579899e5e5827b151f530e1be933` and therefore serves as the immutable start
  checkpoint without a redundant branch;
- broad candidate branch `feat/preserve-a1-1d-5r2-2f0-f0a-broad-blocker-9978655e`, commit
  `78353f3383fdd80c176fe5638328c10f2805dcf0`, ordinary patch
  `577a34da903992d0566fa51d164410caeca98592dd486f5c1d5520e34baaa5a1`, full-index patch
  `9978655e0f4095880c49d863a93b8fc99fb2b769afd42554c758b3970103f710`;
- post-fork-remediation branch
  `feat/preserve-a1-1d-5r2-2f0-f0a-post-fork-remediation-9e012c68`, commit
  `07ce368cc02d5a4e80db9c1ca80efee93009d064`, ordinary patch
  `5530e58a7289db4e9a62fc9934510b87add71853728a0311af2160328af3cbeb`, corrected full-index
  patch `9e012c68ea33107443cd38f2051aa40aae9f0f03d5c6abb4c630d418b0d7dee2`;
- exactly ten Routes A–E runtime commits were ahead; no F subject, change, or candidate restoration
  existed. Neither preserved candidate was checked out, restored, merged, cherry-picked, or replayed.

Refreshing GitNexus changed only generated symbol/relationship/flow count lines in `AGENTS.md` and
`CLAUDE.md`; those exact generated changes were restored before evidence collection. Recorded
status: `GeneratedIndexDriftRemediated`.

### Forced overlap matrix

All F0-HC probe tests named below were temporary, uncommitted diagnostic hooks. They used barriers
to force the precise overlap, exposed no values or secrets, and were removed exactly before the
documentation worktree was created. “Serial” is the same-process ordered control; “separate” is a
fresh-process control.

| Pair / exact diagnostic test | Forced same-process result | Serial control | Separate-process/control result | Normalized signature and classification |
|---|---:|---:|---:|---|
| `SUBSTRATE_FORCE_PTY` mutator → `is_force_pty_command`; `f0hc_env_same_process_overlap_signature` | 20/20 failed | 0/20 failed | 0/20 failed | `F0HC_ENV_OVERLAP: stable reader observed concurrent SUBSTRATE_FORCE_PTY mutation`; `TestIsolationDefectConfirmed` |
| CWD mutator → `WorldRootSettings::effective_root`; `f0hc_cwd_same_process_overlap_signature` | 20/20 failed | 0/20 failed | 0/20 failed | `F0HC_CWD_OVERLAP: stable reader observed concurrent process CWD mutation`; `TestIsolationDefectConfirmed` |
| routing trace owner → second global trace owner; `f0hc_trace_same_process_overlap_signature` | 20/20 failed | 0/20 failed | 0/20 failed | `F0HC_TRACE_OVERLAP: first owner wrote through the second test's global trace output`; `TestIsolationDefectConfirmed` |
| mutated PATH/timeout → `REPORT_CACHE`; `f0hc_socket_cache_persistence_signature` | 20/20 failed after exact env restore | cache-refresh control 0/20 failed | `systemctl_timeout_is_fail_fast` fresh process 0/10 failed | `F0HC_SOCKET_CACHE_PERSISTENCE: cached override survived exact environment restoration`; `TestIsolationDefectConfirmed` |
| first private retry hook → second hook install; `f0hc_private_retry_hook_same_process_overlap_signature` | 20/20 failed | 0/20 failed | 0/20 failed | `F0HC_PRIVATE_HOOK_OVERLAP: first test's hook was replaced by a concurrent test`; `TestIsolationDefectConfirmed` |
| repeated session fixture ID → `WorldDispatchConcurrencyTracker`; `f0hc_dispatch_tracker_same_process_overlap_signature` | 20/20 failed | 0/20 failed | 0/20 failed | `F0HC_DISPATCH_TRACKER_OVERLAP: reused fixture session ID consumed another test's cap`; `TestIsolationDefectConfirmed` |
| global broker policy mutator → stable `policy_mode` reader; `f0hc_broker_same_process_overlap_signature` | 20/20 failed | 0/20 failed | 0/20 failed | `F0HC_BROKER_OVERLAP: stable reader observed another test's global broker policy mutation`; `TestIsolationDefectConfirmed` |
| first `ActivePtyGuard` owner → second `ACTIVE_PTY` registration; `f0hc_active_pty_same_process_overlap_signature` | 20/20 failed | 0/20 failed | 0/20 failed | `F0HC_ACTIVE_PTY_OVERLAP: first test's active PTY control was replaced by another test`; `TestIsolationDefectConfirmed` |

The matrix proves eight new interference pairs. The already-established rows remain binding: the world-socket pair failed 12/20 clean parallel
runs and 0/10 serial runs, while the preserved candidate proof passed 100/100 parallel and 20/20
serial; the exact HOME pair failed 20/20 forced same-process runs, 0/10 stable serial runs, and
passed in separate processes, with three other unannotated HOME mutators independently reproducing
20/20; stdout reporter overlap passed 376 and failed 124 of 500, while isolated, identical-neighbor
serial, and separate-process controls passed 100/100 and pretty reporting passed 99/100. Its exact
captured contamination was `.[codex] task_progress: fields=alpha, beta, gamma (+1 more)\n`.

Static/source lifetime closure added the exact tenth no-await case
`wait_for_fork_child_durable_publication_keeps_stop_transport_timeout_short_once_child_is_visible`.
During the broad evidence wall, predictable private stop paths also left unowned inactive socket
nodes, including `sessdispatch-ashmember.sock`, and a later bind reported `Address already in use`.
This corroborates termination/path ownership; it does not authorize product socket cleanup.

### Bounded clean-code broad evidence

Exactly one clean-code default-parallel wall and one clean-code serial wall were run. They are
correlation evidence, not final closeout:

| Wall | Outcome | Retained diagnostic/semantic evidence |
|---|---|---|
| `cargo test -p shell --lib` | 1,263 discovered; `1113 passed / 150 failed / 0 ignored`; 165.43 s | The retained partial output exhibits legacy `StateStore`/authority-root interference, fixed private stop-path collisions, authority-layout absence, downstream cascades, and one environmental `/tmp` `ENOSPC`. All 150 names are recoverable from the same-run fragment union, but complete panic output survives for only 37; no complete normalized-signature artifact was retained, so this evidence is diagnostic only and is not an exact transition matrix. |
| `TMPDIR=<private disk-backed root> cargo test -p shell --lib -- --test-threads=1` | 1,263 discovered; `1202 passed / 61 failed / 0 ignored`; 388.62 s | The 89-count improvement proves a parallel-sensitive set. The retained 61 normalized into persistent authority-layout/legacy-writer-disabled-or-no-active-state failures, predictable private stop-socket collision, and their async/session cascades. Representative exact retained names were `continue_world_worker_dispatch_returns_real_typed_internal_outcome`, `detached_stop_world_worker_closeout_availability_recheck_accepts_parked_truth_without_sanctioned_owner`, and `prepare_member_runtime_startup_for_descriptor_accepts_parked_detached_orchestrator_parent`. |

Those representative serial failures were also run in fresh processes with unique `TMPDIR` roots.
They remained failures because the tests still consumed the real ambient HOME or a fixed `/tmp`
socket contract; separate-process scheduling alone does not isolate persistent parent paths. No
user-owned authority state was deleted or modified. The evidence therefore distinguishes two
classes: parallel process-global overlap and persistent predictable fixture/path leakage. Neither
class is a production concurrency or lifecycle regression, so `ProductRegressionDecisionRequired`
was not triggered.

Rows proved concurrency-safe include child-only command environments, child-contained umask,
owned descriptors, keyed config/policy caches, immutable OnceLocks/regexes/instance values,
unique TempDir listeners and kernel-selected ports, private-root cross-process lock/CAS tests,
readiness-established protocol timeouts, awaited abort paths, and joined threads/children. The sole
deferred row is production signal/Ctrl-C/SIGWINCH lifecycle, owned by the production PTY/signal
lifecycle packet. Source call closure proves no shell-library unit test calls `execute_with_pty`,
so `initialize_global_sigwinch_handler_impl` and its background thread are never installed in this
binary and cannot perturb this wall.

At that audit checkpoint, no fix or deterministic F comparison baseline existed and F had not
started. The later preserved candidate supplied the focused and three-parallel/one-serial proof
wall frozen in `03` and `04` without completing harness closeout.

The environment-inventory correction itself performed no runtime edit. The exact combined
candidate has since been restored, identity-checked, review-cleaned, canonically closed out, and
preserved. F remained unstarted and was the next packet at that historical checkpoint, before
renewed R2-2 integration closeout, R2-3, R2-4, and R3. The completed-F ledger below supersedes that
next-task status.

## A1.1d-5R2-2F0 historical parallel artifact recovery and authority correction

### Bounded recovery result

The terminal classification is `HistoricalParallelArtifactUnavailable`. One bounded, read-only
inventory exhausted the plausible retained evidence stores; it did not rerun historical code,
reconstruct names from source, combine different runs, or select a convenient later wall.

| Evidence location searched | Result |
|---|---|
| Saved packet checkpoints and current evidence under `/home/spenser/.codex/visualizations/2026/07/{20,21}` | Final-candidate walls, final manifests/fingerprints, and inventory validators exist; no complete historical `1113/150` parallel normalized-signature artifact exists. |
| Prior session transcripts under `/home/spenser/.codex/sessions/2026/07/{20,21}` | One authenticated historical execution was found. Its same-run fragment union yields all 150 names, but truncation leaves complete panic output for only 37; details follow. |
| Repository, worktrees, refs, and commit history under `/home/spenser/__Active_code/substrate` | Source commits and trees are retained. No tracked broad-wall log, differential name/signature manifest, or archived failure artifact exists in the searched history. |
| Preserved local case logs under `/home/spenser/a1-f-case-*` | Complete serial and later broad walls exist, but none has the historical parallel `1263/1113/150/0` identity. Later parallel results include `1206/57`, `1203/60`, `1205/58`, and `1204/59`; they are not substitutes. |
| Explicit temporary artifacts and manifests under `/tmp`, including `a1-f-case-*`, `substrate-f0f0a-walls.*`, `substrate-a1-f0f0a-walls.*`, and `f0hc-*` | Focused logs, inventory data, and later 1,279/1,280-test walls exist. None is the authenticated historical parallel artifact, and no matching complete name/signature manifest exists. |
| Build outputs under `target` and the retained case worktree target | Cargo fingerprints and binaries only; no historical broad-wall output or differential manifest. |
| `/home/spenser/.bash_history` and available shell-history stores | No retained complete output or artifact path. |
| Current application terminal transcript | No terminal session was attached. |
| GitHub Actions runs and artifacts for 2026-07-18 through 2026-07-22 and the relevant source commits | No branch/commit run or Actions artifact retained the historical wall. |
| Bounded content search under `/home/spenser` for the exact aggregate and representative historical failure material | Found the complete serial artifact and the truncated session transcript only; no complete parallel artifact. |

The authentic parallel command was `cargo test -p shell --lib`, run from a clean source at commit
`06c928443a93579899e5e5827b151f530e1be933`, tree
`dfe15d93366655e9855bca40ad0607887f545637`. The retained session file is
`/home/spenser/.codex/sessions/2026/07/20/rollout-2026-07-20T19-43-44-019f81e9-ff42-7ff0-a8a8-296d924a02fb.jsonl`,
SHA-256 `23e8400a26334d3561010f54b71797f06839f4d26f296ad709a675729f9e7e6e`, and records execution
session `94311`. Its final retained wrapper fragment has SHA-256
`c0a90bbe63e99753be61b50cc8f7c10377541243d9c3369ae3251be7aa9b0df2` and the exact aggregate
1,263 discovered, 1,113 passed, 150 failed, and 0 ignored.

That transcript contains an explicit `…2200 tokens truncated…` marker inside a panic body. It
retains 38 complete panic headers and 137 complete names in the final-summary tail; 25 names
overlap, so the same-run union is exactly 150 unambiguous failure names. Only 37 of those headers
retain complete panic bodies sufficient to derive normalized signatures, leaving 113 names without that
semantic material. The commit/tree, command, mode, counts, names, and transcript integrity are
authenticated, but the complete normalized-signature payload is absent. A digest without its input
manifest would not repair that absence. Rerunning the commit now would produce a new sample from a harness
already proven nondeterministic and is not historical recovery.

### Corrected semantic and concurrency authority

The complete serial artifact
`/home/spenser/a1-f-case-b-serial-corrected.F6aLQa/output.log`, SHA-256
`8cf7109d02d738c89bf446d6810f7e06d0140c681da803b88ba4d8adeda13bc7`, is historical semantic
authority. It contains 1,263 discovered, 1,202 passed, 61 failed, 0 ignored, complete failure names,
and sufficient output for normalized signatures. Its exact transition matrix is:

| Transition | Count |
|---|---:|
| `PassToPass` | 1,202 |
| `PassToFail` | 0 |
| `FailToSameFailure` | 45 |
| `FailToChangedFailure` | 0 |
| `FailToPass` | 16 |
| `Removed` | 0 |
| `RenamedOrSubstituted` | 0 |
| `NewPass` | 17 |
| `NewFail` | 0 |
| `NewIgnored` | 0 |

All 16 `FailToPass` rows have causal audits in the saved differential evidence. All 17 `NewPass`
rows are the added authorized tests named in `02`. Deterministic historical-serial and final test
listings prove zero removed and zero renamed/substituted tests. Arithmetic closes exactly:
`1202 + 61 = 1263`, `1202 + 16 + 17 = 1235`, `45` final failures, and `1263 + 17 = 1280`.

Final concurrency authority is the exact final candidate's three parallel walls and one serial
wall. Each discovered 1,280, passed 1,235, failed 45, ignored 0, and produced identical failure
names and normalized signatures. The retained wall hashes are:

| Wall | SHA-256 |
|---|---|
| parallel 1 | `89e8805528ad91ccc13153459cf79c7533e12e9036c29118f6a170bb12400e58` |
| parallel 2 | `f956cbaf4f575b99bb9d3bdd7418eb08438f8897464798572c8e2b880ccea68e` |
| parallel 3 | `7965550a83becfcf252a1a145c1e5ce7138836863d9e46e27d361db33b771fa7` |
| serial | `4d4c9cf00cf05e6f58129ae1b8752505dd8cfeb9dd9d8efb79e6d515eb085718` |

The different raw-log hashes reflect wall-local output while the parsed counts, complete
failure-name sets, and normalized signatures agree exactly. The final deterministic test list has
SHA-256 `a7b795a76c5ed7f158fa0b181facc701b87a4c1d141c6d1909cea1c3f62fe022`; its normalized test-name
list has SHA-256 `7aa03922dcc727ea8d67e4084ada3d93da926d95b95eb4f292776e55fa33e2b5`.
Every wall's failure-name set has SHA-256
`b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`; every normalized-signature
set has SHA-256 `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`.
The exact 17-test `NewPass` list has SHA-256
`490d41b6025f3a6a09e58dfa7fcd5b739b5fbf82ea00cf9e150d9c98520fae5a`; the exact 16-test
`FailToPass` list has SHA-256
`dafcc5a0417ed32917fcb0d81ff028f126f8acd9c94c9c08640cc48be9ab1801`.

The historical parallel aggregate remains useful diagnostic evidence of prior interference, but
it is not transition authority. This record does not claim that the final 45 failures belonged to
the historical parallel set, that exactly 105 named historical parallel failures became passes,
a historical parallel `PassToFail` count, or any historical parallel name/signature comparison.

### Canonical candidate preservation, proof, review, and stop boundary

The exact final candidate was committed without modification and remotely verified on
`feat/preserve-a1-1d-5r2-2-harness-final-baseline-blocker-7ab220a7` at commit
`86ed6f5620787121b1c2e5b033ee8d6f9ff369d3`, parent
`1e1be221ca8a9f4e94af93fbda7bda6e94901d01`, tree
`f6480f3986d43bb41e5387fa1ba5b68ae53f598b`. It changes exactly 45 files with 5,630 insertions
and 2,104 deletions. The exact manifest, per-file fingerprints, ordinary patch, and full-index
patch are respectively:

- `b9e3a44dd671409f66e2d62d48cb494ab147069a74ed71f2308e031f56372ae6`;
- `41cb1a325add4efd8198872b456c3ae0fba73f8c4943e4b59c06e2bc76d8b479`;
- `7ab220a715f4ae3389be314da2e6b0e614fcffb7a92166f04fde999570801861`; and
- `adc1c5952e2ac4cc97881a1bf4df8e24b00d4d4ba337e692bf21965151d5c0c4`.

That exact patch was restored onto corrected source head `4eca8773bf1cfaac1e4fcb7ba09b8a06c310ab7c`
and committed without byte changes as `770a6a9de9f537f7bc179c75421abbc3fff05b8d`, tree
`61fdd2e9476f1ce3e041720ce106f7c3427895be`. The dedicated review-clean runtime preservation branch
is `feat/preserve-a1-1d-5r2-2-harness-review-clean-7ab220a7`. The two whole-tree hashes differ only
because the corrected source base contains the six published differential-authority documents;
the 45-file patch and all four candidate hashes are identical.

The corrected inventory is exactly 86 names, 74 parent-mutated, 12 child-only/read-only, 395
primitives, 73 dynamic calls, 43 sinks, 1,005 resolved callsites, 534 tests across 35 files, 38
resource rows, and zero unresolved dynamic rows. Its validator and negative wrapper-callsite,
migration-row, unresolved-dynamic-key, and parent-as-child perturbations all behave correctly. All
38 resource dispositions are implemented or retained.

Focused proof passes for HOME/socket overlap; stdout/stderr renderer isolation; XDG/account-home
negative projection; environment/CWD restoration; trace/report-cache/retry-hook/tracker/broker and
`ACTIVE_PTY` isolation; deterministic fork publication; async termination confirmation;
subprocess isolation; exact absence and non-Unicode restoration; panic, poison, nesting, and
reacquisition; and child failure propagation. Shell and workspace all-target checks, relevant
Clippy with `-D warnings`, `cargo fmt --all -- --check`, and `git diff --check` pass.

The 16 `FailToPass` rows are causally assigned to seven detached-availability fixes, two
deterministic fork-publication fixes, six stop-dispatch fixes, and one hidden-owner fix. The 17
`NewPass` entries are exactly the authorized tests listed in `02`. No test was removed, renamed,
substituted, weakened, or newly ignored.

Environment closure, renderer/injection, and lifecycle/subprocess reviews
`/root/final_review_env_closure`, `/root/final_review_renderer_injection`, and
`/root/final_review_lifecycle_subprocess` are CLEAN for the exact patch. The prior blocked
containment review is not reused. Fresh isolated read-only reviewer
`/root/final_containment_corrected_authority`, task
`019f8681-7957-7cc3-88fa-37ab3ad2fc87`, returned CLEAN under the corrected authority contract.
GitNexus change detection reports one generated MEDIUM process label, resolved by source closure to
test-only `AuthorityEnvTestTempDir::new`; there is no changed production execution flow. The sole
production hunk is the authorized mechanical F0b writer delegation with byte-identical output.

This is an evidence-authority correction, not a waiver. `PassToFail`, `FailToChangedFailure`,
`Removed`, `RenamedOrSubstituted`, `NewFail`, and `NewIgnored` are satisfied zero gates. F0, F0a,
F0b, and F0-HC are complete. The runtime changes no product or user-facing behavior, promotes no
seam, and does not start F.
The binding sequence is:

`Routes A–E` → `F0/F0a/F0b/F0-HC complete` → `F` →
`renewed R2-2 closeout` → `R2-3` → `R2-4` → `R3`.

At that historical F0 closeout, F was unstarted and the next packet was **A1.1d-5R2-2F —
Authenticated world-deps and truthful doctor composition**. The completed-F ledger below
supersedes that next-task status.

## A1.1d-5R2-2F readiness-boundary evidence ledger

The prior line records the historical F0 closeout state. The current checkpoint is F1/F2 complete,
F3/F4 blocked, and F incomplete.

| Evidence | Exact result |
|---|---|
| Source base before local F commits | `7d642606205b4f0f93e157fbde27507a7c86e655` |
| F1 | `eae02af959f0b7066015bb242ffa45fc7a01d591`; tree `7bd7e8b5ca2d563b2991b4fea62ca4ea26266b0c`; message `feat: bind authenticated world-deps context`; ordinary patch `829b8a19c0d2deb53af0b9c2f277e7eab53b5001f41c59ebc3e7075c1a3e8b54`; full-index patch `f7c16056ce5b379848203b066bffec37ef9f6f46fdcb2753af6af4feac01886f`. |
| F2 | `d30d8cec764e2338fb48475747733091d3af22bf`; tree `a6161c7664fa6ca1173302dd3570b5d8d7e09ecc`; message `feat: propagate authenticated world-deps scope`; ordinary patch `42f5278a6cab739c9d5b5e5ca7ee2c3a960ba8d8197e2f772192a65e391ed31b`; full-index patch `b65e3146e870ac7579e43508fb87b4c1bdc63afae96bf67473b75a7285e8bfe8`. |
| F1/F2 preservation | `feat/preserve-a1-1d-5r2-2f-f1-f2-d30d8cec` points to exact F2. |
| Blocked F3/F4 preservation | Branch `feat/preserve-a1-1d-5r2-2f-f3-f4-blocked-7224dd30`; commit `a343f0796d19d66c168c5bb2797856710cff5708`; parent exact F2; tree `a377daa6f41693454e38c39163cc9895bbf3e828`. |
| Blocked manifest | `world_deps/mod.rs`; `world_deps/surfaces.rs`; `world_enable/runner.rs`; `world_enable/runner/provision_deps.rs`; `execution/routing.rs`; `execution/routing/dispatch/prelude.rs`; `execution/routing/dispatch/world_ops.rs` (all beneath `crates/shell/src/`). |
| Blocked fingerprints | Manifest `23a1dc1566d395ae5e606ebb4bae0d7e7d4c9c3f976958d7e3c7ce8a094f0cc8`; per-file aggregate `a8399a8a0989c675b05788fb12f2f91105cfa1e15ac0dca911df918594033edd`; ordinary patch `7224dd30c04e5fcd85913bfdddb62deb497f0b3eaba416b2f689d392f47b9b0c`; full-index patch `76fb0c5c183880dca8f31576647c8e9372d1ad026e2a731b48d5ef2642ef0a22`; 613 insertions/128 deletions. |
| Canonical harness baseline | Exact base wall: 1,280 discovered, 1,235 passed, 45 failed, 0 ignored. F1/F2 add three passing tests without changing the 45-failure set. |
| Generated index drift | `GeneratedIndexDriftRemediated`: GitNexus refresh changed only generated `AGENTS.md`/`CLAUDE.md` count lines; both exact generated changes were restored before preservation. |

### Reviewer finding and correction disposition

The preserved candidate's prefix-based ambient environment copy is rejected. It can disclose
credentials or request material and can pass authority/backend/policy selectors from B beside an
authenticated A request. The candidate also bypasses the canonical readiness owner and therefore
cannot provide correct Linux service activation/spawn behavior. These are security and ownership
defects, not permission to duplicate lifecycle logic.

GitNexus reported `ensure_world_service_ready` **HIGH**, with three graph-visible direct callers and
three affected existing process groups. Manual source closure found two additional direct uses: the
persistent-session WebSocket setup and Linux initialization's function-pointer probe. Impact on
each existing caller label, the additive F builder, `run_world_command_for_deps_at`, and
`execute_with_profile` was LOW with zero upstream impacts. The HIGH owner is approved only for
private extraction plus compatibility delegation; all existing callers remain unchanged.

The exact corrected disposition is:

- one readiness owner in `world_ops.rs`;
- one private explicit-target Linux core, with no copied probe/activation/stale/spawn/timeout logic;
- unchanged no-argument compatibility entry point and callers;
- fixed product socket and immutable installed-product service posture for authenticated F;
- exact seven-entry generated guest environment and zero ambient forwarding;
- authenticated failure before readiness side effects;
- exactly two additive-builder consumers;
- no production capability or policy change, no non-Linux change, and no seam promotion.

F3/F4 must be rebuilt from clean F2 under this contract; preservation bytes are not restored or
approved wholesale. F5 and final F walls remained unstarted. Renewed R2-2 integration closeout,
R2-3, R2-4, and R3 remained blocked. The exact next task at that historical checkpoint was
**Resume A1.1d-5R2-2F3/F4 under the corrected readiness and environment contract**. The F5-PD and
completed-F ledgers below supersede that next-task status.

## A1.1d-5R2-2F5-PD evidence and decision ledger

This latest ledger supersedes the live status/next-task conclusion above. F3/F4 were rebuilt from
the corrected boundary and are complete. F5 then stopped before closeout because its mandatory
nested child reached the active public Doctor path. No F5 implementation byte is accepted.

### Verified starting state and retained evidence

| Evidence | Verified result |
|---|---|
| Source branch before preservation | `feat/internal-host-orchestrator-world-dispatch-bootstrap`; HEAD `7802c44198625ea6849933140c390900ebe84190`; tree `21395f927779a49dce2b692a7f407c72a9964105`; parent `54a5ce663cf2724b69b0c124acf2c2758ff622dd`; source remote `2a982292de794dd0e4c4e00f151fe83dfb5b0b63`; divergence 0 behind / 15 ahead |
| F3 commit | `54a5ce663cf2724b69b0c124acf2c2758ff622dd`; tree `db4a51b153bc93b258d25ce4289c4ef3d477a646`; subject `feat: add authenticated world-deps request builder`; ordinary/full-index patch SHA-256 `c9ec972b7098ce8b5d633f7793678fccbb7ffebda4d2769bc3119a4791990c61` / `3db155ae7e308f50e9f9c80b74ea5ace50b2fcc6e8ffd5cf34e0c288955343d2`; stable patch ID `a8ad65e71975b236f082f4abeb3b0fdcd37f216e`; three files, 978 insertions, 51 deletions |
| F4 commit | `7802c44198625ea6849933140c390900ebe84190`; tree `21395f927779a49dce2b692a7f407c72a9964105`; subject `feat: propagate authenticated world-deps execution`; ordinary/full-index patch SHA-256 `179e67b16316ae62683085cf17d5e4e56c30fec7c51ca12b4edeb47d78d36e65` / `3996fba97a44fc20c2e95d9395971b21172ceeca963a8d235b2ab7a57579fd2a`; stable patch ID `016e4edb9ecd21150a4e788ab93b7ba0dc0df4c6`; six files, 741 insertions, 114 deletions |
| Clean F4 wall | 1,300 discovered / 1,255 passed / 45 failed / 0 ignored; failure-name SHA-256 `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`; normalized-signature SHA-256 `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3` |
| Attempted F5 focused evidence | Decoder tests: 13 passed. Shim-doctor integration: 17 passed / 1 failed. These are diagnostic evidence only and do not waive the boundary stop. |
| Boundary adjudication | Fresh isolated reviewer `/root/f5_boundary_adjudication` returned exact status `CrossDocumentChangeRequired`. |

### Exact blocked-candidate preservation

The original source worktree contained exactly four unstaged modified files and no staged or
untracked path. They were committed without byte changes to remote branch
`feat/preserve-a1-f5-blocked-candidate-20260722` at
`c4b93506480ae5e9708417067980e08e0e754b7a`, parent
`7802c44198625ea6849933140c390900ebe84190`, tree
`03bec2f517866e5213b8a0bed9ad99058c25ebc3`. Remote ref, parent, manifest, per-file bytes, ordinary
patch, full-index patch, and statistics were re-read and matched. The source branch was then
returned clean to its exact original HEAD; the candidate was not restored.

| Candidate file | SHA-256 |
|---|---|
| `crates/shell/src/builtins/shim_doctor/report.rs` | `e41ba1e79b451280af9135c7bfdb8523250848850d929957260a02b891cf92f5` |
| `crates/shell/src/builtins/world_deps/mod.rs` | `89283c62780c9fc58ff4d22cfaf10abf990ce4b039238b742eb343ce505e958c` |
| `crates/shell/tests/shim_doctor.rs` | `13b06c4db570193f68ecb6e72d37062474f61ee59e4a9610f261d76a4595069a` |
| `crates/shell/tests/shim_health.rs` | `e81005319e737c0ed2fcd50c26793aa28cff96496da807ec4fbadaf056df6be5` |

Ordinary patch SHA-256 is
`81151bdad6aa9e7dd1963b7b22f1f3dad66b46399174ff26bfa33ec08859e03a`; full-index/binary patch
SHA-256 is `7805a28defc11866902fadaaf99d7c752800c5aab257b9c927187920bd6cf908`.
Statistics are exactly four files, 1,326 insertions, and 65 deletions: `report.rs` 882/60,
`world_deps/mod.rs` 12/0, `shim_doctor.rs` 256/3, and `shim_health.rs` 176/2.

The preservation operation did not inspect, modify, move, or remove
`/home/spenser/.cache/substrate-user-home-archive/20260722-084820-substrate`.

### Control-pack contradiction and source decision

The pre-correction control pack required the no-fixture branch to run the exact existing child at
`00-README.md:292-301`, `02-seam-crosswalk.md:292`, `03-phase-slice-map.md:531-541`,
`04-contracts-and-gates.md:591-594,808-815`, and
`05-debug-regression-ledger.md:498-519`. It simultaneously required a side-effect-free/read-only
diagnostic at `01-target-architecture.md:395-409`, `03-phase-slice-map.md:655`,
`04-contracts-and-gates.md:1089-1099`, and `05-debug-regression-ledger.md:389`, while freezing the
platform/service/lifecycle owners at `01-target-architecture.md:414-420`,
`03-phase-slice-map.md:658,1231-1236`, and
`04-contracts-and-gates.md:820-826,1097-1103,1115-1122`.

Source closure proved the child was public `world doctor --json`: CLI routing entered the public
Doctor arm; Linux doctor gathered socket-activation state and connected the socket; the client
called `/v1/doctor/world`; world-service selected/mounted and mutated an enumeration probe; and a
404 invoked the `/v1/execute` filesystem fallback. A Unix socket connection can activate
world-service. Therefore the old pair of requirements could not both hold. Deleting the child or
synthesizing unavailable solely in the parent would have broken the separately required hidden
argv carrier/child-process proof.

On Linux, the earliest bounded branch is after
`routing.rs::decode_and_bind_unix_install_bootstrap_context` succeeds and before
`install_bootstrap_projections` and `ensure_substrate_home_deps_scaffold_for_context`. That branch
preserves a real authenticated child while making every active downstream owner unreachable. No
new lifecycle owner is necessary, so `PassiveDiagnosticArchitectureDecisionRequired` does not
apply. The existing fail-closed report can represent the outcome without a schema change, so the
schema-specific stop does not apply either.

Three review defects were source-closed before publication. First, whenever Linux World-enabled
production reaches `gather_world_doctor_snapshot`, it may not select
`A/health/world_doctor.json`: F5-PD removes that branch and runs the authenticated child, while
Linux value decoding becomes `cfg(test)`-only. The frozen World-disabled `build_report` branch
continues to return `disabled_world_doctor_snapshot` without a child or fixture lookup. The fixture
is therefore Linux test evidence rather than product authority. Non-Linux fixture/public-child
compatibility remains behavior-frozen and cannot satisfy proof. The separate world-deps fixture
remains F5-owned and outside F5-PD. Second, the existing Linux snapshot decoders cannot be frozen
because they infer success from missing `ok`/exit 0 and retain arbitrary details/stderr. F5-PD
authorizes Linux `snapshot_from_command`, the test-only value decoder, and one private validator to
require exact schema 1, compile-time platform, explicit false status, duplicate exact A identity,
exit 4, empty stderr, no extra or forbidden key, and bounded no-payload unavailable/incoherent
construction. Their GitNexus impacts are LOW at one direct/one total caller each, zero attributed
processes, and the Shim-doctor module only.

Third, the live Linux `run_json_subcommand` parses raw stdout directly into
`serde_json::Value`, whose last-wins duplicate-key behavior could collapse conflicting B-then-A
identity or `ok` fields before the validator observed them. The exact correction authorizes only
that symbol's Linux raw-stdout parse expression plus three private exact wire structs and one
private decoder using `deny_unknown_fields`; all duplicate/unknown top-level or nested keys reject
before value construction. `run_json_subcommand` is LOW 0/0. Its child construction, carrier and
projection transport, signature, process/exit/stderr behavior, selected-access redaction, and all
non-Linux code remain frozen. Regression 14 now includes conflicting top-level/nested duplicate
keys and B-then-A identity ordering. Frozen `build_report` is LOW one direct/five total and
`disabled_world_doctor_snapshot` is LOW one/six. The test-inclusive `maxDepth=5` lifecycle totals
are `ensure_world_service_ready` CRITICAL 4 direct/10 total and `socket_activation_report`
CRITICAL 6/12; neither is authorized for edit or passive call.

The hidden discriminator is carried by the existing `WorldAction::Doctor` variant, not the
top-level `Cli` struct. This source-closed choice leaves `auto_sync.rs::cli_for_auto_sync`
untouched. It is one additive hidden Rust enum field and an ordinary-path fail-closed guard in
`handle_world_command`; normal public CLI grammar/help/behavior remain unchanged. GitNexus reports
LOW 0/0 impact for both `WorldAction` and `handle_world_command`.

### Security decision

The threat boundary is the hidden child argv carrier versus inherited process environment and
ambient B. Spoofing is blocked by carrier commitment and current-principal validation; tampering
and mixed identity fail closed; duplicate/unknown raw JSON fields reject before lossy value
construction; elevation through environment-only mode selection is impossible;
information disclosure is bounded by rejecting secret/request/carrier/preimage markers and not
retaining rejected payloads; active endpoint/probe denial-of-service and lifecycle effects are
removed by the pre-bootstrap branch. No logs, traces, or side tables are created by passive mode.
Only the already-public selected-prefix/commitment diagnostic identity may be emitted.

The exact parent result for a valid child is existing status `needs_attention`, `ok=false`, source
`command`, exit 4, no stderr/details, and bounded error `passive world doctor unavailable`. Invalid
child evidence uses the same closed shape with error `passive world doctor incoherent`. The
recursive validator rejects case/separator variants of credentials, tokens, API/private keys,
authorization, passwords, secrets, prompts, request bodies/bytes/input, carrier/auth-bundle bytes,
parent/full environment, and commitment preimages without echoing the rejected key or value. JSON
and human output therefore share one existing representation without a transport/report-schema
change.

F5-PD changed documentation only at that checkpoint. Its runtime implementation and the F5 work
that followed are recorded by the controlling closeout below.

## A1.1d-5R2-2F final evidence and decision ledger

### Verified source and preservation identities

F5 started from clean F5-PD HEAD `653a7d91489563bc2a8e3395feeb53e159254240`, tree
`4e3457444c64bbddda7bcf19300c4e7e81082678`, on
`feat/internal-host-orchestrator-world-dispatch-bootstrap`. The published source remote remained
`2de15dfba9f11d874eccfdca5438c600be1045b2`; divergence was 0 behind / 16 ahead and the worktree,
index, and untracked set were clean. F5-PD preservation
`feat/preserve-a1-f5pd-runtime-20260722` remained exact at that commit with ordinary/full-index/
stable patch identities `b7a5b60f8c7dbd1960b7fec4870de047c827ed611ea1d4bfba33e6a9ed22e32d`,
`b6e7e4a61c24815ef24fb74660d3cbc499de256771d1857bb963b0fd5f31b03d`, and
`90dafa73a5887457114ad2d4073fab8980e5630e`.

The blocked donor preservation `feat/preserve-a1-f5-blocked-candidate-20260722` remained exact at
`c4b93506480ae5e9708417067980e08e0e754b7a`, parent
`7802c44198625ea6849933140c390900ebe84190`, tree
`03bec2f517866e5213b8a0bed9ad99058c25ebc3`, ordinary/full-index patches
`81151bdad6aa9e7dd1963b7b22f1f3dad66b46399174ff26bfa33ec08859e03a` and
`7805a28defc11866902fadaaf99d7c752800c5aab257b9c927187920bd6cf908`. It is not an ancestor of
the source branch and was not merged, cherry-picked, reset onto the source, or restored wholesale.

F5 is `2bb4696d7181d974c1b02e33d82e09422cdae7de`, parent exact F5-PD, tree
`313a8a613a6cd91e72c0cc1664f4b9842fefa305`, complete message
`fix: compose truthful authenticated doctor evidence`. Its ordinary patch is
`991859870123dca56f3ee080876742abb2e556f9ee0bdf62f3577fdf1ccfb189`, full-index patch is
`8e2be2f2b84dc5d3cd573b9670e4d3eccc3060893f478a414d057e39b002a943`, and stable patch ID is
`b768cb94c51802e2440fbf1eb7f29fc0693a446b`. The name-only manifest hash is
`3716e05de411351b410c31fba9b37aebf98f470a1e1170f6d6a5c35658dca883`; the complete name-status
manifest hash recorded during commit proof is
`69285de3d080d277cc73232173e6fb035ca898f336e818960fd036c862164c13`.
The per-file blob SHA-256 values below are the exact fingerprints.

| F5 file | Final blob SHA-256 | Insertions / deletions |
|---|---|---:|
| `crates/shell/src/builtins/shim_doctor/report.rs` | `f02cff00312eb847d3bce47ab1311fe6938b231d0fe7430015bc18232a1bfde1` | 427 / 1 |
| `crates/shell/src/builtins/world_deps/mod.rs` | `e4c22293ff8aeb541a5e0115996006c6368931803dd900e86aed558fd741020b` | 37 / 7 |
| `crates/shell/tests/shim_doctor.rs` | `d4af75efc5da8bb17427b5a006e2dc33e1762d82d929665a671ffb7d236c671a` | 134 / 0 |
| `crates/shell/tests/shim_health.rs` | `f135c7cffd9c236f642ef119887df09ada7330c55c853cf324e14630f2ce91db` | 155 / 0 |

Dedicated preservation branches `feat/preserve-a1-f5-runtime-20260722` and
`feat/preserve-a1-f-complete-runtime-20260722` both point remotely to exact F5. The source runtime
remained local/unpushed while preservation branches carried immutable proof.

### Blocked-donor hunk disposition

Every donor hunk was classified before production edits:

| Donor hunk family | Final disposition |
|---|---|
| World-deps selected-prefix/commitment fields | Applicable only after rebasing onto F5-PD; retained under Linux cfg with strict top-level decoding |
| Parent world-deps identity/status checks | Applicable after semantic correction; exact A/CWD/enums/duplicates are validated and absent health stays unavailable |
| Fixture selection and direct fixture payload retention | Requires semantic correction; fixture is physical A-bound compatibility/test evidence only, runtime claims reject, retained output is recomposed |
| Donor no-fixture public World Doctor child | Forbidden and replaced by F5-PD's authenticated passive child |
| Donor generic `serde_json::Value`-first decoding | Forbidden; typed/duplicate-rejecting validation precedes nested generic shape inspection |
| Donor inference of success from identity, exit, empty error, or missing runtime evidence | Forbidden; missing evidence is bounded unavailable |
| Active public Doctor, service/socket/readiness/HTTP/execute/probe behavior | Forbidden and absent |
| Ambient/contextless identity or fixture authority | Forbidden; request-scoped A is the sole identity owner |
| F5-PD hidden selector, early authenticated branch, passive emitter, and strict child decoder hunks | Replaced by completed F5-PD and frozen |
| Non-Linux compatibility, report/transport schema expansion, F3/F4 request/readiness/environment, secure-FD, policy/capability, lifecycle, cleanup, or later-packet hunks | Obsolete, outside F5, or forbidden |

### Final runtime behavior and proof

Linux `WorldDepsDoctorSnapshotV1` rejects unknown top-level fields and carries A's selected prefix
and non-secret commitment. Linux `collect_doctor_snapshot_v1` validates the authenticated context,
reads A-derived config/global inventory plus request-scoped launch-CWD/workspace inventory through
that shared context, and deliberately does not call the applied/runtime probe. It
returns no applied items and the bounded reason `passive runtime health evidence unavailable`.

Linux `gather_world_deps_section` keeps the existing disabled early return. On the enabled route it
accepts only the exact physical `A/health/world_deps.json` compatibility fixture, decodes it
strictly, rejects runtime claims and mixed/malformed evidence, then discards its payload and
recomposes from the passive A collector. Without a fixture it calls the same passive collector.
The result is unavailable until adequate permitted runtime evidence exists; no identity field or
fixture can fabricate coherent health. A→B fixture symlinks, raw applied errors, fixture counts,
known-field secret markers, nested unknown fields, duplicates, partial sets, and identity/
commitment/CWD mismatches fail closed without disclosure.

The exact no-fixture shim-doctor integration proves one authenticated passive child and zero public
Doctor, socket/service, HTTP, execute, probe, world, provisioning, repair, cleanup, or mutation
activity. Shim-doctor and Health JSON/human views agree. The public World Doctor tests remain
semantically unchanged. F3/F4 readiness and exact seven-entry environment are frozen.

The clean F5-PD shell-library baseline was 1,307 discovered, 1,262 passed, 45 failed, 0 ignored.
Each of three final parallel walls and the final serial wall was exactly 1,309 discovered, 1,264
passed, 45 failed, 0 ignored. The 45 failure names and normalized signatures retained SHA-256
`b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70` and
`33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3` on every wall. The serial
normalizer excludes libtest's standalone `FAILED` harness token; after that format-only exclusion
its panic evidence is byte-identical to the parallel authority.

| Transition from clean F5-PD | Count |
|---|---:|
| Existing pass → fail | 0 |
| New fail | 0 |
| Fail → changed failure | 0 |
| Fail → pass | 0 |
| New library pass | 2 |
| New focused integration pass | 3 |
| Removed | 0 |
| Renamed or substituted | 0 |
| Newly ignored | 0 |
| Weakened or redirected | 0 |

Focused F proof includes world-deps, current/global/workspace/runtime/provision/post-sync,
authenticated builder, exact readiness/environment, passive child/decoder, Host/World/Health/
shim-doctor, public Doctor compatibility, policy/network/world-fs, and managed secure-FD regression
suites. Shell and workspace all-target checks pass. Raw shell all-target Clippy remains non-green
only for the exact 20 inherited `needless_borrow` findings in untouched/frozen code; the documented
differential permits only that lint and otherwise denies every warning, and passes. Formatting and
diff checks pass.

The auxiliary full world-service unit wall completed 131 tests without failure and then stalled in
two existing FUSE doctor tests after an environmental busy-unmount diagnostic. It was terminated;
no complete world-service wall, privileged smoke, or product capability proof is claimed from that
run. The bounded secure-FD, request-routing, capability-gate, authority, and netfilter filters pass.

GitNexus final F5 detection is LOW with four changed files, 27 mapped changed symbols, zero affected
execution flows, and no new process/module owner. Fresh F5 increment reviewers
`/root/f5_inc_authority_r2`, `/root/f5_inc_lifecycle_r2`, and `/root/f5_inc_scope_r2` are CLEAN.
Fresh complete-F reviewers `/root/f_final_authority_security`,
`/root/f_final_lifecycle_regression`, and `/root/f_final_source_platform` are CLEAN.
After the first docs-only review findings were remediated, fresh replacement reviewers
`/root/f_docs_arch_authority_r2`, `/root/f_docs_evidence_diff_r2`, and
`/root/f_docs_sequence_stale_r2` returned CLEAN for architecture/authority,
evidence/differential, and cross-document sequencing/stale-status cleanup respectively.

### Final decision and next node

F1–F5 and F5-PD are runtime-complete, proof-complete, review-clean, preserved, and canonically
closed out. Linux authenticated truth is proven; macOS, Windows, and other non-Linux routes remain
frozen compatibility/unavailable/unproven. Public Doctor, managed secure-FD, direct-member
credential compatibility, policy, network, world-fs, caging, capability, service/lifecycle,
receipt/supervisor, cleanup/rollback, and platform ownership are unchanged. Open RG gates remain
open, direct-member Codex/UAA gateway adoption is not begun, privileged product smoke is not
claimed, and no seam is promoted.

The successful terminal state at that checkpoint was `A1.1d-5R2-2FComplete`. Its exact historical
next node was **A1.1d-5R2-2 renewed production-fix-free integration closeout**. The
[remediation planning ledger](../a1.1d-5r2-2-renewed-closeout/evidence-regression.md#closeout-remediation-planning-ledger-historical-rp0-checkpoint)
supersedes only that next-node disposition; R2-3, R2-4, and R3 remain after
the later closeout/publication sequence.

**Source provenance:**
- extracted from [`05-debug-regression-ledger.md#r2-2-historical-failed-integration-closeout-and-remaining-seam-correction`](../05-debug-regression-ledger.md#r2-2-historical-failed-integration-closeout-and-remaining-seam-correction), lines 537–1120; baseline span SHA-256 `73a70ad659cc987e3e51f162586d80e5dcfc49ef161dcffe94fd23598428b0c9`
- extracted from [`05-debug-regression-ledger.md#a11d-5r2-2f0-hc-empirical-closure-record`](../05-debug-regression-ledger.md#a11d-5r2-2f0-hc-empirical-closure-record), lines 2174–2855; baseline span SHA-256 `b64fa10be55becdb7b23472e60d9a10382c03063fa04b7cd6f098bbb0ffa4eab`
