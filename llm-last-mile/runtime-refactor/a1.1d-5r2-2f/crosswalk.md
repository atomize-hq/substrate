**Kind:** crosswalk
**Stable ID:** `A1.1d-5R2-2F-family`
**Canonical for:** A1.1d-5R2-2F family crosswalk
**Status:** canonical
**Authority scope:** exact extracted family-local source bodies only
**Source span:** composite of the preserved root compatibility spans listed in Source provenance below
**Supersedes:** canonical ownership of the extracted source bodies; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R2-2F family crosswalk

### F0 test-proof prerequisite crosswalk

| Scope | Current state | Required F0 boundary | Exact source disposition | Required proof |
|---|---|---|---|---|
| Shell library-test process | Sixty-six test-time world-socket mutators exist. Fifty-one orchestrator call sites use a local guard or manual restoration; four macOS platform tests use an unlocked closure helper; two persistent-session and two routing tests mutate directly; one world-enable path test restores manually. Of five world-gateway calls, four use an exact RAII environment guard while one classification-test closure restores only after normal return; one async-REPL call is already compliant. | One RAII world-socket guard uses `world_env_guard`, captures exact `OsString`/absence, restores before unlock during normal return and unwinding, safely supports same-thread nesting, and relies on non-poisoning `parking_lot::ReentrantMutex` behavior. Every noncompliant mutator migrates; stable readers participate through the same lock. | Test-only hunks in `crates/shell/src/execution/mod.rs`, `crates/shell/src/execution/orchestrator_world_dispatch.rs`, `crates/shell/src/execution/platform/macos.rs`, `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`, `crates/shell/src/execution/routing/world.rs`, `crates/shell/src/builtins/world_enable/runner/paths.rs`, and the classification-test helper/call in `crates/shell/src/builtins/world_gateway.rs`. Four world-gateway RAII sites and the async-REPL site are source-reviewed and unchanged. | Exact pair 100 parallel/20 serial; prior value/absence, panic, non-poisoning recovery, nesting, blocking/cleanup, no competitor observation; neighboring parallel combinations; three default-parallel broad walls and one serial wall with stable inherited names/signatures. |
| Shell integration-test binaries and subprocess fixtures | `Command::env`/`env_remove` configure only the spawned child; each integration source is a separate test binary. | Preserve process-local child injection. No cross-process lock or production environment change. | No integration-test migration unless later source evidence proves an in-process mutation. | Inventory validation and unchanged integration behavior. |
| Production readers and socket resolution | Product code reads the process environment under existing compatibility contracts. | Frozen. F0 does not claim general Unix process-environment safety and does not alter production resolution or readiness. | No production symbol edit, capability change, retry, sleep, or readiness relaxation. | GitNexus containment to test dependents and byte-identical production paths. |

### A1.1d-5R2-2F0a — SUBSTRATE_HOME test isolation

F0a HOME isolation and F0 cleanup use this combined crosswalk:

| Scope | Proven current state | Required combined boundary | Exact source disposition | Required proof |
|---|---|---|---|---|
| Shell library-test HOME mutation | Source closure identifies 435 mutating test functions across 19 files. Five `with_store` families cover 223 dependent tests; two agents-command helpers cover five; direct/local guards cover the remainder. Four mutating tests are unannotated, and all can overlap `#[serial]` tests. The exact target is `dispatch_contract_adapter_active_task_resolution_requires_supervisor_claim`; a minimal competitor is `prompt_submit_continuity_prefers_persisted_session_contract`. The stable same-process serial harness proved only competitor-then-target at 0/10; separate-process sequential runs passed both directions, and reverse same-process order was not controllable or claimed. | One process-global authority-environment lock protects HOME and world socket. Capture exact prior `OsString`/absence; hold through mutation, dependent async/process work, cleanup, and restoration; release only afterward. Panic, poison/recovery, nesting, and intentional child inheritance are explicit. Readers needing a stable authority snapshot participate. | Test-only HOME migrations in `crates/shell/src/builtins/{shim_doctor/report.rs,world_deps/mod.rs,world_enable/runner/manager_env.rs,world_gateway.rs}`, `crates/shell/src/execution/{agent_inventory.rs,agents_cmd.rs,config_model.rs,env_scripts.rs,host_inbox_materialization.rs,invocation/tests.rs,orchestrator_world_dispatch.rs,routing/builtin/tests.rs,settings/tests.rs}`, `crates/shell/src/execution/agent_runtime/{auto_attach.rs,control.rs,state_store.rs,tool_invocation_contract.rs,host_session_authority/store_tests.rs}`, and `crates/shell/src/repl/async_repl.rs`, plus the shared test-only boundary in `crates/shell/src/execution/mod.rs`. No production symbol. | Exact HOME pair at least 100 parallel/20 serial; the preserved candidate proves both controlled nonoverlapping orderings; combined HOME/socket neighbor matrix; prior value/absence/non-Unicode restoration; panic, poison/recovery, nesting, exclusion, child inheritance, and leak proof. |
| Integration-test parent mutation | Three independent integration-test source families mutate HOME in their own test binaries: `shim_deployment.rs` (nine serialized callers), `agent_successor_contract_ahcsitc0.rs` (one caller), and `support/mod.rs` (one ignored one-test binary and one serialized caller). Other integration occurrences are reads or child-only `Command::env` inputs. | Per-binary exact restoration where parent mutation exists; no cross-process lock. `#[serial]` may supplement but not replace the reviewed per-process disposition. | Test-only helpers/callers in all three inventoried families, `crates/shell/tests/{shim_deployment.rs,agent_successor_contract_ahcsitc0.rs,support/mod.rs}`, must use equivalent exact, panic-safe restoration; child-only integration files stay unchanged. | Exact prior-value/absence/panic proof in every inventoried integration helper and unchanged child inheritance. |
| Joint HOME/socket topology | At least 88 shell-library tests depend jointly on both variables. Existing helper order is inverted across modules, so separate locks could deadlock or expose mixed authority state. | One authority-environment lock; no independent-lock ordering contract. Same-thread nested scopes restore in stack order. | Generalize/reuse the test-only F0 boundary; do not add a production API or a cross-process lock. | Concurrent mixed-variable exclusion, nested restoration, and no lock-order inversion. |
| F0 async server cleanup | Nine F0-migrated orchestrator tests abort a socket-owning `JoinHandle` without awaiting it. One only yields; declaration/drop order does not confirm cancellation. | `abort` → await confirmed task termination → complete/confirm fixture socket cleanup → restore environment → release the authority-environment lock. | Test bodies only in `crates/shell/src/execution/orchestrator_world_dispatch.rs`; no server lifecycle or readiness production edit. | All nine named fixtures prove termination and zero leaked task/socket/temp-root state before restoration/unlock. |
| Production HOME/socket readers | Product resolution and runtime behavior are frozen. The non-Unix production `world_enable` HOME mutation is not a test helper and cannot be changed by F0a. | No product lock, resolver, retry, sleep, or behavior change. | No production hunk. Tests invoking production code hold the test boundary externally when required. | Byte-identical production paths and GitNexus containment to test dependents. |

### A1.1d-5R2-2F0b — deterministic renderer-output test isolation

F0b adds no production inventory row or promoted seam. Its complete bounded crosswalk is:

| Scope | Proven current state | Required F0b boundary | Exact future source disposition | Required proof |
|---|---|---|---|---|
| Private renderer core | Private Unix-only `PublicPromptRenderer::render` directly selects stdout/stderr. Its JSON-envelope serialization error propagates with the existing context; other renderer writes and all flush results are intentionally ignored as today. | Keep `render` as the production entry point and delegate to a private explicit-writer core or equivalent private sink adapter. Select and lock only the current stream at the same point; preserve exact bytes, order, newline, flush, error treatment, redaction, and bounded fallback. | One file only: `crates/shell/src/execution/agent_runtime/control.rs`. EDIT `PublicPromptRenderer::render` only for mechanical delegation; ADD private Unix-only internal core/adapter symbols. `PublicPromptRenderer::new` and all caller bodies are frozen. | Production differential proves byte-identical stdout/stderr selection and bytes, unchanged ordering/flush/error behavior, no new execution process, and unchanged semantics in the existing `handle_agent_command` process family. |
| Stdout fallback test | `capture_stdout_once` replaces process fd 1 with `dup2`. Libtest's parallel reporter can enter that pipe; exact observed bytes are `".[codex] task_progress: fields=alpha, beta, gamma (+1 more)\n"`. Existing partial-line search masks the shared-boundary error until the reporter prefix changes the expected match. | Remove raw descriptor capture. Give the private core a test-owned stdout buffer and assert the complete exact bytes plus empty stderr. No trimming, searching, tolerated prefix, or reporter filtering. | DELETE `capture_stdout_once`; EDIT only `public_prompt_renderer_renders_bounded_structured_fallback_when_decode_fails`. Test name and renderer contract remain unchanged. | Exact target isolated and forced parallel, with 100/100 private-buffer pair stress in implementation closeout; exact byte equality, no unrelated output admitted. |
| Stderr fallback test | `capture_stderr_once` has the same process-global fd 2 replacement and ownership even though it did not appear in the broad-wall failure. | Migrate with stdout to a test-owned stderr buffer; assert its complete exact bytes plus empty stdout. | DELETE `capture_stderr_once`; EDIT only `public_prompt_renderer_renders_bounded_structured_stderr_fallback_when_decode_fails`. | Exact stderr bytes, stream choice, and private-buffer isolation under forced parallel reporting. |
| Production callers | Source closure finds two production construction/call paths: `run_hidden_owner_helper_startup_prompt_stream_with_projection` and `run_public_prompt_command`. | Frozen bodies and call shape. Real stdout/stderr remain the default and only production sinks. | No caller edit, public API, transport/schema, writer registry, side table, global output lock, environment selector, or eager dual-stream lock. | GitNexus/source closure and diff prove caller bodies unchanged; no product or user-visible behavior change. |

The causal controls are binding evidence: forced same-process parallel passed 376 and failed 124 of
500 with the exact wall signature; isolated target, same-process serial with identical neighbors,
and separate-process control each passed 100/100; parallel pretty reporting passed 99/100.
Candidate introduction is unnecessary because the target and helper are byte-identical to clean E.
`#[serial]` cannot exclude libtest's reporter and may be supplemental only. Sleeps, retries,
reporter suppression, whole-suite serialization, test-thread reduction, ignore, removal, rename,
substitution, and assertion weakening cannot satisfy F0b.

PI-082's dedicated bootstrap action corrects only the command trigger for the already-owned
explicit-context private-home edge. It does not add a new inventory edge or transfer world-deps
command processing from PI-036/R2-2. In R2-1, the focused R1 private-home integration file may
replace historical `--version` bootstrap invocations with the authenticated hidden action and may
add version nonmutation proof; its fixture topology, security assertions, and production ownership
remain unchanged.

R2-1 closes exactly its 19 owned inventory rows at runtime commit
`2653c2ef20ae2e119a444811e6fb46e86d1a6ec6`: PI-001–PI-004, PI-010–PI-011, PI-032–PI-034,
PI-061–PI-063, PI-067, PI-082–PI-085, PI-104, and PI-117. PI-050 remains an unchanged guardrail;
PI-064 and PI-092–PI-093 remain frozen R3 lifecycle rows. PI-118, release/sudo/service/runtime rows,
platform mappings, and joined R2-4 proof remain unstarted. No row changes classification and no seam
is promoted.

R2-3 closes exactly these inventory rows at source checkpoint
`c583c5f293644fab75d8d42bd3bcad63f114d4fe`: PI-009, PI-022, PI-039–PI-046, PI-048–PI-049,
PI-052, PI-054–PI-058, PI-060, PI-068–PI-070, PI-075–PI-076, PI-079, PI-081, PI-090–PI-091,
PI-112, PI-115–PI-116, and PI-118. The accepted direct shell-library proof is
`1322 discovered / 1274 passed / 48 failed / 0 ignored`, with failure-name SHA-256
`c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9` and normalized-signature
SHA-256 `2a0df9b340cc7e5e1b6e4f76e60e6f937b7442008142d78a7ae24bbcd2f60a90`; the historical 45-failure
inventory remains intact and the only additional failures are the three separately classified
world-deps/report expectations named in `00` and `05` and summarized in `03`. The refreshed source-bound native
macOS and Windows receipts are `EVIDENCE_CLEAN` for `R2-DIAG-01`, `R2-MAP-MAC-01`, and
`R2-MAP-WIN-01` at that same source. The R2-3-owned exceptions remain PI-059 as harness-only and
PI-077/PI-078 as byte-frozen fail-closed guards; PI-050 remains the R2-4 guardrail; PI-080 remains satisfied by earlier R2-2 Linux
restart-scope work and is not reopened here; and no R3 lifecycle/forwarding/provisioning/cleanup/
rollback/convergence row changes classification or ownership here.

Inventory closure rules:

1. Every implementation diff must cite affected `PI-*` rows and may not add an unclassified
   home/root/prefix/principal child edge. A new edge first updates this existing table.
2. No R2 row grants deletion authority. Rows PI-007, PI-019, PI-049, PI-070, PI-089, PI-090,
   and PI-115 transport or classify only. PI-116 consumes only the A-scoped manager projection;
   PI-117 adds the default-preserving shell product trace/policy posture; PI-118 plus PI-091 migrate
   physical-shim/replay/platform callers and close the temporary compatibility posture while leaving
   the neutral setter and trace rotation/retention byte-frozen. None grants prefix selection,
   installer cleanup, replacement, or deletion authority. The twenty-two
   `R3CleanupOnly` rows—PI-012, PI-026, PI-031,
   PI-047, PI-051, PI-053, PI-064, PI-074, PI-092–PI-103, and PI-113–PI-114—reserve their lifecycle
   actions for R3; carrier work in a sibling row cannot reclassify those actions.
3. No generated projection, inherited environment, CWD, `$HOME`, `$USERPROFILE`, outer
   `SUBSTRATE_HOME`, guest default, or runtime side table can replace `IH` or `PM`.
4. Static macOS/Windows/WSL review can make a packet review-clean but can never satisfy its named
   native proof assignment.
## A1.1d-5R2-2F0-HC corrected consolidated harness crosswalk

F0-HC is a test-proof prerequisite and promotes no crosswalk seam. The previously published F0
seven-file socket allowlist, F0a 19-file shell-library HOME allowlist plus three integration-helper
families, and F0b one-file renderer contract remain exact and unchanged. In particular, F0b remains
limited to `crates/shell/src/execution/agent_runtime/control.rs`, its private writer core or sink
adapter, deletion of `capture_stdout_once`/`capture_stderr_once`, and its two named fallback tests.
The consolidated boundary below is additive only where source closure proved a further
same-process blocker. The corrected environment inventory contains 86 exact names: 74
parent-mutated and 12 child-only or read-only. Its parent-mutating union is corrected from 518 to
534 tests in the same 35 files, while A2 is corrected to 129 direct tests: 116 additive and 13
already-authorized F0a tests.

| Mechanism | Exact future test-only file/symbol allowlist | Exact tests/calls to migrate | Frozen boundary and GitNexus posture |
|---|---|---|---|
| Authority environment | Existing F0/F0a rows; `crates/shell/src/execution/mod.rs::{WORLD_ENV_LOCK,world_env_guard}` may become the one test-only authority-environment coordinator. The same test-only guard may replace direct/wrapper mutation only in the exact 35-file environment union below. | All 456 HOME/`SUBSTRATE_HOME`/`SUBSTRATE_WORLD_SOCKET` dependent tests covered by the unchanged F0/F0a manifests, including thirteen existing-F0a wrapper tests, plus the exact 116-test additive A2 manifest below. Stable authority/path readers participate through named existing helper closure; no production reader body changes. | `world_env_guard` is CRITICAL: 70 impacted symbols, 35 direct test callers, five processes, and nine modules. `WORLD_ENV_LOCK` is graph-LOW/under-resolved. This is an adjacency warning, not authority to edit production bodies/callers; only the enumerated `cfg(test)` helpers and test call sites may change. |
| CWD lane | Add/reuse a `cfg(test)` exact `PathBuf` guard in `crates/shell/src/execution/routing/test_utils.rs`; migrate test-only CWD sites in `builtins/shim_doctor/report.rs`, `builtins/world_gateway.rs`, `execution/config_cmd.rs`, `execution/invocation/tests.rs`, `execution/orchestrator_world_dispatch.rs`, `execution/routing/builtin/tests.rs`, `execution/routing/dispatch/tests/host_replay.rs`, `execution/settings/tests.rs`, `repl/async_repl.rs`, and `execution/agent_runtime/host_session_authority/store_tests.rs`. | The 100 helper-closed shell-library tests, including the 12 direct mutators; `effective_root_uses_current_directory_for_follow_mode` is the focused stable reader. | `WorldRootSettings::effective_root` is HIGH (two direct callers, one process family). Its body and every production caller remain frozen; only test guards/calls change. |
| Event registry | Source-closed resource family `crates/shell/src/execution/agent_events.rs::{AGENT_EVENT_SENDER,EVENT_TEST_GUARD,acquire_event_test_guard,init_event_channel,clear_agent_event_sender,publish_agent_event,agent_event_sender}`. Future edits are limited to `cfg(test)` child wrappers and the exact ten event-channel-owning test entry points below; all named resource/production symbols remain frozen. | Run each exact test in a bounded child test/helper process; no parent-process publisher/reader needs a new lock and no production read/publish symbol enters the edit allowlist. | No production `init_event_channel`, `clear_agent_event_sender`, `publish_agent_event`, `agent_event_sender`, sender semantics, registry shape, or reader call-site change. Child exit discards registry state after owned event work terminates; no event payload or secret is logged. |
| Global trace isolation | Test helpers only: `execution/routing/test_utils.rs::test_shell_config`, `execution/manager.rs::tests::test_shell_config`, and the exact thirteen callers below. | Run each affected helper family through a bounded child test/helper process; do not add a production trace reset or side table. | `TraceContext::init_trace` is MEDIUM with nine direct routing callers plus four manager callers; helper graph is under-resolved. All trace production initialization, append, rotation, policy, and output semantics are frozen. |
| Socket-activation cache isolation | `crates/shell/src/execution/socket_activation.rs::systemctl_timeout_is_fail_fast` test invocation only. | Move that exact test body to a bounded helper-process path so `REPORT_CACHE` cannot retain its mutated PATH/timeout-derived report in the parent. | `refresh_socket_activation_report` is HIGH (three direct callers, three production process families); its body and `REPORT_CACHE` production semantics remain frozen. |
| stdin/PTY isolation | `execution/routing/dispatch/tests/pty.rs::test_stdin_nonblock_roundtrip`; `execution/pty/io/types.rs::{minimal_terminal_guard_handles_creation,active_guard_resets_flag_on_drop}`; and `execution/pty/control.rs::active_control_dispatches_commands_and_clears_on_drop` only. | Execute each exact case in a bounded helper process. `test_stdin_nonblock_roundtrip` receives `Stdio::null()` or a dedicated pipe as a fresh fd-0 open-file description. `minimal_terminal_guard_handles_creation` receives a fresh child-owned PTY slave on Unix or child-owned console on Windows. No case inherits parent fd 0; exact assertions remain. | No fd policy, PTY routing, reporter mode, thread count, terminal guard, production atomic/registry, `PtyActiveGuard`, or `ActivePtyGuard` behavior changes. The child protocol is recursion-proof, timeout-bounded, propagates nonzero/signal status, kill-then-waits/reaps on timeout, exercises the terminal/console mutation, and proves parent fd-0 flags/mode unchanged after ordinary and panic/abort child outcomes. |
| Global broker isolation | `execution/routing/dispatch/tests/support.rs::with_test_mode` and the exact 23 broker-mutating callers below, including `execution/platform/mod.rs::world_enable_exports_policy_fs_mode`. | Execute each listed test in an ordinary bounded child test/helper process so `substrate_broker::GLOBAL_BROKER` cannot retain or expose another test's mutable policy mode in the libtest parent. | No broker reset, production side table, `BrokerHandle`, `GLOBAL_BROKER`, policy parsing, reload, selection, or enforcement change. `set_policy_mode` is graph-LOW/under-resolved; source closure and forced overlap are binding. |
| Explicit private hook | Test-only sections of `execution/orchestrator_world_dispatch.rs::{PrivateStopTransportRetryHookGuard,private_stop_transport_retry_hook}`. | The nine existing hook-installing tests receive a test-owned explicit callback/fixture instead of replacing one global slot. | Production `request_private_stop*` behavior, retry sequence, deadlines, and errors are frozen. Graph is source-closed/test-private. |
| Explicit dispatch identity | Test-only construction around `world_dispatch_concurrency_tracker` in `execution/orchestrator_world_dispatch.rs`. | The exact four tracker tests below receive unique explicit test identities; shared fixture constructors may derive those identities only for those four callers. No production tracker reset. | Tracker production acquisition/release and cap semantics are frozen. Future test identity injection is private and carries no product authority. |
| Explicit private transport root | Only the exact 62 test bodies enumerated below in `execution/orchestrator_world_dispatch.rs` and `repl/async_repl.rs`; test-owned short store/home/root constructors used exclusively by those bodies may be adjusted. | Private stop/cancel/prompt/startup/toolbox fixtures receive unique short fixture-owned roots so no Unix-path-length fallback can alias a predictable `sessdispatch-*` name; each binding test verifies unlink after confirmed termination. | No production `private_*_transport_path`, `toolbox_transport_path*`, `internal_toolbox_transport_path`, endpoint, naming rule, Unix fallback, socket contract, or cleanup authority change. |
| Termination-confirmed teardown | The nine named F0 tests already listed in `03`, plus `wait_for_fork_child_durable_publication_keeps_stop_transport_timeout_short_once_child_is_visible`, all in `execution/orchestrator_world_dispatch.rs`. | Exactly ten `abort()`-without-await test bodies: abort, await cancellation, confirm fixture/socket cleanup, then restore/unlock. | Production server lifecycle/readiness/retry bodies remain frozen; all other abort-plus-await paths remain unchanged. |
| Deterministic ordering | Test-only waits in `execution/agent_runtime/control.rs`, `execution/agent_runtime/host_session_authority/store_tests.rs`, `execution/agent_runtime/state_store.rs`, `execution/lock.rs`, `execution/orchestrator_world_dispatch.rs`, `execution/routing/dispatch/world_ops.rs`, and `repl/async_repl.rs`. | Only the exact 19 ordering tests below may replace a direct scheduling sleep with a barrier/readiness publication. Sleeps that model a protocol deadline after explicit readiness remain byte-frozen. | No production timeout, retry, scheduler, clock, or error semantic changes. No global clock override is authorized. |

### Corrected environment source closure and ownership

The first audit's helper-reachability parser found 506 tests by same-file traversal, while its
exact-name phase extracted literals only from direct
`std::env::{set_var,remove_var}` arguments. It recognized wrapper bodies without resolving every
wrapper argument/callsite. The later manual dynamic-wrapper repair preserved the settings tests as
`SUBSTRATE_ROOT` dependents without extracting their three override-name arguments and did not
close `AmbientSelectionGuard::set` at all. GitNexus also under-resolved this `cfg(test)` gateway
call edge. The published audit manually added twelve disjoint dynamic-wrapper tests to reach 518,
but ambiguous same-file bare-name traversal also introduced two non-mutating B1 acknowledgement
tests. Independent correction review then proved the first 405-row and later 604-row evidence
passes still omitted cross-file or duplicate-name callsites for `with_store`,
`ProjectionEnvGuard::capture`, `install_bootstrap_projections`, `apply_world_root_env`,
`export_runtime_config_env`, `update_world_env`, and the macOS `restore` family. The defect is
therefore wrapper-argument and wrapper-callsite non-closure plus ambiguous-name overreach, not
stale source, child projection, or arithmetic retained for compatibility.

The corrected delta is exact: add the XDG gateway test,
`codex_auth_projection_errors_redact_committed_account_home`, both
`platform_tests::update_world_env_sets_*_flags` tests, and the fourteen `with_test_mode` PTY
callers that do not mutate another environment name directly; remove
`b1_task_acknowledgement_rejects_nonstart_eof_mismatch_and_repetition_before_persist` and
`b1_retained_acknowledgement_joins_exact_request_and_rejects_terminal_or_drift`, which never
mutate or read the parent environment. The complete source-closed result is therefore 534 tests,
not a compatibility-preserved total.

The corrected lexical pass accounts for all 395 direct parent-mutation primitive invocations in
immutable source, including 73 invocations whose key expression is a wrapper parameter or named
constant. A second wrapper/callsite pass resolves every literal, constant, array/table, loop, and
parameterized argument through the exact 35-file and test manifests below. The binding validator
also freezes 43 dynamic mutation-sink owners, 1,005 resolved parent-mutation callsites, and all 534
mutating test symbols; parses projection/local-key tables from source; and runs negative
perturbation checks. Generic routing
`export`/`unset` helpers are closed by their exact test inputs; fixed install/runtime projection
maps are closed by their constants. No libc `setenv`, `unsetenv`, `putenv`, or `clearenv`, and no
imported mutation alias, exists in this source. `Command::{env,env_remove,env_clear}` is classified
as child-only unless the same path separately invokes a parent mutation primitive. Reads alone do
not become mutators.

Every primitive call maps to exactly one of A1, A2, or the parent-restoration side of the same
guard; every child-only projection maps to A4; every read-only name maps to A3. The externally held
deterministic validator fails on a changed primitive count, a new dynamic key expression, a missing
wrapper callsite, a changed literal/constant argument set, a parent/child misclassification, or a
mutating test absent from this migration manifest. Dynamic names are failures, never silently
dropped.

| Corrected names | Parent primitive → wrapper → exact callsite/test | Stable readers / current protection | Required disposition and exact future allowlist |
|---|---|---|---|
| `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME` | `std::env::{set_var,remove_var}` → `builtins/world_gateway.rs::classification_tests::AmbientSelectionGuard::{set,drop}` → `authenticated_gateway_projection_ignores_named_ambient_roots_and_uses_account_db_codex_home` | Gateway request-context construction, `codex_auth_state_path`, inherited child config/path consumers, and the test's negative-authority assertions; today only exact local RAII restoration, with no process-wide exclusion | A2 / `UnifiedProcessStateLock`; future edits are limited to `execution/mod.rs::{WORLD_ENV_LOCK,world_env_guard}` and the named test-only guard/test plus the mechanically necessary coordinator-guard call. Preserve all nine conflicting ambient entries, exact `OsString`/absence restoration, authenticated-A authority, account-database Codex-home authority, and every assertion. No production reader edit. |
| `SUBSTRATE_OVERRIDE_ANCHOR_MODE`, `SUBSTRATE_OVERRIDE_ANCHOR_PATH`, `SUBSTRATE_OVERRIDE_CAGED` | `std::env::{set_var,remove_var}` → `execution/settings/tests.rs::EnvGuard::{new,drop}` → `resolve_world_root_respects_env_when_no_configs`, `resolve_world_root_env_overrides_global_config` | `config_model.rs::parse_env_overrides` through effective-config/world-root resolution; today the local guard restores only `String`/absence, so a prior non-Unicode value collapses to absence, and there is no process-wide exclusion | A2 / `UnifiedProcessStateLock`; future edits are limited to `execution/mod.rs::{WORLD_ENV_LOCK,world_env_guard}` and the named settings-test guard/tests plus the mechanically necessary coordinator call. Migration must capture/restore exact `OsString`/absence. Production parser/resolver bodies and assertions remain frozen. |
| Already-listed `SUBSTRATE_SHELL`, corrected from non-parent to parent | `std::env::{set_var,remove_var}` → `execution/routing/test_utils.rs::{set_env,restore_env}` → `execution/routing/dispatch/tests/host_replay.rs::async_repl_host_commands_record_replay_context` | `crates/trace/src/span.rs::SpanBuilder::new`; `DirGuard` holds the environment/CWD lanes, but three early returns precede manual restoration and the helper captures `Option<String>`, so persistence and non-Unicode loss remain possible | A2 / `UnifiedProcessStateLock`; future edits are limited to `execution/mod.rs::{WORLD_ENV_LOCK,world_env_guard}`, the named test-only helper pair, and the named host-replay test plus mechanically necessary RAII guard use. Restore exact `OsString`/absence on return, early return, and unwind. `SpanBuilder::new` remains byte-frozen. |
| Already-listed `SUBSTRATE_WORLD`, `SUBSTRATE_WORLD_ENABLED`, `SUBSTRATE_WORLD_FS_MODE`, `SUBSTRATE_WORLD_FS_ISOLATION`, `SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING`, `SUBSTRATE_WORLD_REQUIRE_WORLD`; two tests newly source-closed | production-frozen `execution/platform/mod.rs::update_world_env` → `execution/platform/macos.rs::platform_tests::{snapshot,restore}` → `update_world_env_sets_enabled_flags`, `update_world_env_sets_disabled_flags` | World-policy, routing, and platform readers; both tests already hold `world_env_guard`, but `snapshot`/`restore` use `Option<String>` and cover only the first two names, so four mutated names persist and non-Unicode prior values are not exact | A2 / `UnifiedProcessStateLock`; future edits are limited to `execution/platform/macos.rs::platform_tests::{snapshot,restore,update_world_env_sets_enabled_flags,update_world_env_sets_disabled_flags}` plus the mechanically necessary existing coordinator use. Capture/restore all six as exact `OsString`/absence. Production `update_world_env`, policy readers, and assertions remain frozen. GitNexus: helper functions LOW, two direct test callers, zero processes; both tests LOW, zero callers/processes. The Codex-auth test and fourteen newly closed PTY test symbols are likewise LOW with zero callers/processes; the shared `with_test_mode` helper retains its prior HIGH adjacency warning. |

The remaining 80 names retain their previously assigned A1/A2/A3/A4 ownership except for the
explicit `SUBSTRATE_SHELL` reclassification above, and they do not retain the old completeness
claim. The exact corrected 86-name enumeration and the 74-parent/12-non-parent split
are binding in `04`.

### Binding additive source and test manifest

Paths in this subsection are relative to `crates/shell/src/`. This is the canonical exhaustive
manifest for additive Harness work. A name absent here and absent from the already-published
F0/F0a/F0b manifests is not authorized.

The complete A1/A2 environment test-file union is exactly these 35 files:

`builtins/shim_doctor/report.rs`, `builtins/world_deps/mod.rs`,
`builtins/world_enable/runner/manager_env.rs`, `builtins/world_enable/runner/paths.rs`,
`builtins/world_gateway.rs`, `execution/agent_events.rs`, `execution/agent_inventory.rs`,
`execution/agent_runtime/auto_attach.rs`, `execution/agent_runtime/control.rs`,
`execution/agent_runtime/host_session_authority/store_tests.rs`,
`execution/agent_runtime/state_store.rs`, `execution/agent_runtime/tool_invocation_contract.rs`,
`execution/agent_runtime/validator.rs`, `execution/agents_cmd.rs`, `execution/config_model.rs`,
`execution/env_scripts.rs`,
`execution/host_inbox_materialization.rs`, `execution/invocation/tests.rs`,
`execution/manager.rs`, `execution/manager_init/tests.rs`,
`execution/orchestrator_world_dispatch.rs`, `execution/platform/macos.rs`,
`execution/platform/mod.rs`, `execution/platform_world/windows.rs`, `execution/pty/io/types.rs`,
`execution/routing/builtin/tests.rs`, `execution/routing/dispatch/tests/host_replay.rs`,
`execution/routing/dispatch/tests/linux_world.rs`, `execution/routing/dispatch/tests/pty.rs`,
`execution/routing/dispatch/world_ops.rs`,
`execution/routing/dispatch/world_persistent_session.rs`, `execution/routing/world.rs`,
`execution/settings/tests.rs`, `execution/socket_activation.rs`, and `repl/async_repl.rs`.

The non-F0a additive portion of the A2 ambient-selector migration is exactly these 116 tests in 18
of those files; names on one line share the preceding file:

- `builtins/world_gateway.rs`: `authenticated_gateway_context_uses_a_for_config_policy_inventory_and_network`, `ambient_world_disable_toggles_apply_only_after_authenticated_a`, `ambient_world_disable_toggles_cannot_bypass_tampered_a`, `codex_auth_projection_errors_redact_committed_account_home`.
- `execution/agent_events.rs`: `publish_command_completion_success_emits_task_end_when_enabled`.
- `execution/agent_runtime/validator.rs`: `validate_member_selection_returns_descriptor_for_unique_world_member`, `validate_member_selection_resolves_alias_backend_to_canonical_runtime_family`, `validate_member_selection_prefers_world_alias_while_host_codex_remains_distinct`, `validate_exact_backend_selection_rejects_unqualified_preplacement_backend_ids`, `validate_exact_backend_selection_rejects_split_world_backend_ids`, `validate_exact_backend_selection_accepts_placement_qualified_version_2_ids`, `validate_runtime_realizability_accepts_world_scoped_codex_guest_entrypoint_contract`, `validate_runtime_realizability_rejects_world_scoped_codex_when_guest_entrypoint_is_absent`, `validate_runtime_realizability_distinguishes_world_scoped_codex_permission_denied`, `validate_runtime_realizability_keeps_host_path_codex_truth_separate_from_world_codex_truth`.
- `execution/invocation/tests.rs`: `wrap_mode_uses_cli_shell_and_shimmed_path`, `skip_shims_and_no_world_disable_shimmed_path`.
- `execution/manager.rs`: `configure_manager_init_generates_snippet_and_exports_env`, `configure_manager_init_honors_overlay_file`, `manager_env_script_sources_manager_and_legacy_snippets`, `configure_child_shell_env_clears_inherited_manager_env_guard`, `manager_manifest_base_path_prefers_env_override`.
- `execution/manager_init/tests.rs`: `detect_env_matches_expected_values`, `detect_script_runs_powershell_commands`, `config_from_env_respects_skip_and_debug_flags`.
- `execution/orchestrator_world_dispatch.rs`: `dispatch_contract_continue_world_worker_fork_command_submits_rendered_prompt_to_exact_retained_source`, `dispatch_contract_continue_world_worker_fork_command_tolerates_late_child_durable_publication_after_authoritative_registration`, `dispatch_contract_continue_world_worker_fork_command_preserves_spawn_bootstrap_error_chain`, `dispatch_contract_continue_world_worker_control_directive_returns_explanation_ready_error_when_delivery_fails`, `b1_b21_0_ordinary_continue_joins_canonical_retained_target_and_supervision`, `active_ephemeral_terminal_truth_guard_publishes_failed_terminal_truth_on_drop_after_start`, `principal_aware_direct_spawn_projects_exact_intended_codex_home`, `principal_aware_prepared_spawn_projects_exact_intended_codex_home`, `compatibility_spawn_without_credential_projection_clears_poisoned_seed`, `compatibility_spawn_with_credential_projection_fails_without_principal`, `dispatch_contract_spawn_world_worker_uses_authoritative_workspace_root_for_bootstrap`, `dispatch_contract_fork_world_worker_returns_typed_lineage_after_authoritative_bootstrap`, `dispatch_contract_fork_world_worker_waits_for_child_durable_publication_before_persisting_lineage`, `dispatch_contract_fork_world_worker_tolerates_late_child_durable_publication_after_authoritative_registration`, `dispatch_contract_fork_world_worker_rolls_back_child_when_lineage_persist_fails`, `dispatch_contract_fork_world_worker_times_out_missing_child_registration_and_rolls_back`, `dispatch_contract_fork_world_worker_times_out_when_stop_transport_publication_never_completes`.
- `execution/platform/macos.rs`: `doctor_ok_json`, `doctor_resolves_override_vm_name_and_reports_it`, `world_doctor_json_uses_override_vm_name`, `host_doctor_json_uses_override_vm_name`, `world_doctor_json_reports_running_vm_with_inactive_service_as_not_provisioned`, `update_world_env_sets_enabled_flags`, `update_world_env_sets_disabled_flags`.
- `execution/platform/mod.rs`: `world_enable_exports_policy_fs_mode`.
- `execution/platform_world/windows.rs`: `ensure_world_ready_sets_env_on_success`, `ensure_world_ready_ignores_disabled_env_when_forced`, `ensure_world_ready_respects_no_world_flag`.
- `execution/pty/io/types.rs`: `terminal_size_uses_non_zero_dimensions`.
- `execution/routing/builtin/tests.rs`: `export_builtin_sets_plain_pairs`, `export_builtin_defers_when_value_needs_shell`, `unset_builtin_clears_variables`, `world_flag_overrides_disabled_config_and_env`, `world_flag_honors_directory_world_root_settings`, `anchor_flags_override_configs_and_export_anchor_env`, `no_world_flag_disables_world_and_sets_root_exports`, `cd_bounces_when_caged_without_world`, `cd_bounces_when_caged_with_world_enabled`, `anchor_guard_bounces_chained_cd_when_world_disabled`, `anchor_guard_bounces_chained_cd_when_world_enabled`, `cd_allows_uncaged_escape_from_anchor`.
- `execution/routing/dispatch/tests/host_replay.rs`: `async_repl_host_commands_record_replay_context`.
- `execution/routing/dispatch/tests/linux_world.rs`: `agent_probe_enables_world`, `fallback_uses_local_backend_stub`, `disabled_skips_initialization`.
- `execution/routing/dispatch/tests/pty.rs`: `test_needs_pty_ssh`, `test_needs_pty_known_tuis`, `test_needs_pty_shell_meta`, `test_is_force_pty_command`, `test_is_pty_disabled`, `test_needs_pty_integration`, `test_needs_pty_ssh_variations`, `test_needs_pty_quoted_args`, `test_needs_pty_pipes_redirects`, `test_repl_heuristic`, `test_debugger_pty`, `test_windows_exe_handling`, `test_container_k8s_pty`, `test_wrapper_commands`, `test_pipeline_last_tui`, `test_ssh_spacing_edge_cases`, `test_force_vs_disable_precedence`, `test_git_commit_edit_flag`.
- `execution/routing/dispatch/world_ops.rs`: `preserve_world_project_dir_override_records_logical_root`, `preserve_world_project_dir_override_uses_dispatch_cwd_for_follow_cwd`, `current_world_request_profile_accepts_non_reserved_values`, `current_world_request_profile_rejects_reserved_world_deps_profiles`.
- `execution/socket_activation.rs`: `systemctl_timeout_is_fail_fast`.
- `repl/async_repl.rs`: `greenfield_host_start_proposal_applies_and_exact_joins_without_world_binding`, `prepare_repl_dormant_host_launch_plan_keeps_world_mode_lazy_until_first_targeted_turn`, `prepare_repl_dormant_host_launch_plan_keeps_no_world_mode_lazy`, `refresh_member_runtime_binding_from_shared_world_metadata_after_mismatch_declines_without_metadata_even_when_persisted_session_truth_differs`, `refresh_member_runtime_binding_from_shared_world_metadata_after_mismatch_fails_closed_when_metadata_is_unreadable_even_if_live_member_truth_exists`, `synchronize_repl_authoritative_world_binding_repairs_from_persisted_session_truth`, `synchronize_repl_authoritative_world_binding_prefers_shared_world_metadata_over_stale_session_truth`, `synchronize_repl_authoritative_world_binding_fails_closed_when_metadata_is_unreadable_even_if_live_member_truth_exists`, `refresh_run_world_task_request_binding_after_mismatch_repairs_retry_request`, `dispatch_run_world_task_request_with_binding_retry_retries_once_after_exact_binding_mismatch`, `start_remote_member_runtime_with_binding_retry_retries_once_after_exact_binding_mismatch`, `start_remote_member_runtime_with_binding_retry_does_not_retry_non_exact_mismatch_error`, `prepare_member_runtime_startup_for_descriptor_accepts_parked_detached_orchestrator_parent`, `start_member_runtime_reuses_parent_session_and_persists_world_binding`, `orchestrator_world_dispatch_surface_spawns_authoritative_member_runtime`, `orchestrator_world_dispatch_surface_routes_stop_into_durable_closeout_and_runtime_cleanup`, `orchestrator_world_dispatch_surface_routes_valid_cancel_requests_into_typed_cancel_closeout`, `orchestrator_world_dispatch_surface_routes_valid_fork_requests_into_packet_three_retained_bootstrap_launch`, `orchestrator_world_dispatch_surface_stops_fork_child_without_eviction_of_source_runtime`, `orchestrator_world_dispatch_surface_round_trips_successful_inspect_without_mutating_retained_runtime_handles`, `prepare_member_replacement_runtime_preserves_resumed_from_lineage`, `build_member_dispatch_transport_request_uses_shared_contract_parity_subset`, `retained_member_dispatch_parity_subset_prefers_pending_replacement_truth`.

The additional thirteen wrapper-mediated tests already lie inside the F0a file boundary and also
belong to A2; this normalized list is mechanically binding:

- `builtins/shim_doctor/report.rs`: `world_deps_section_forwards_authenticated_a_under_conflicting_ambient_b`.
- `builtins/world_deps/mod.rs`: `doctor_snapshot_uses_authenticated_a_under_conflicting_ambient_b_without_mutation`, `doctor_snapshot_rejects_tampered_context_without_mutation_or_disclosure`.
- `execution/env_scripts.rs`: `write_env_sh_preserves_authenticated_install_projection`.
- `builtins/world_gateway.rs`: `authenticated_gateway_projection_ignores_named_ambient_roots_and_uses_account_db_codex_home`.
- `execution/settings/tests.rs`: `resolve_world_root_defaults_to_launch_dir_project`, `resolve_world_root_refuses_legacy_workspace_settings_yaml`, `resolve_world_root_respects_env_when_no_configs`, `resolve_world_root_env_overrides_global_config`, `resolve_world_root_prefers_workspace_config_over_global_when_env_unset`, `resolve_world_root_prefers_cli_over_all_other_sources`, `resolve_world_root_requires_anchor_path_for_custom_mode`, `effective_root_uses_current_directory_for_follow_mode`.

The gateway test and the two macOS platform tests are the corrected migration-manifest additions;
the other twelve F0a tests were already named. These sites are not a new micro-packet; their exact
test-only authorization is
consolidated here.

CWD work is confined to these existing test symbols in the ten named files: `builtins/shim_doctor/report.rs::tests::ProcessStateGuard`; `builtins/world_gateway.rs::tests::{CurrentDirGuard,with_current_dir,GatewayCurrentDirGuard}`; `execution/config_cmd.rs::tests::CwdGuard`; `execution/invocation/tests.rs::CurrentDirGuard`; `execution/orchestrator_world_dispatch.rs::tests::CurrentDirGuard`; `execution/routing/test_utils.rs::{DirGuard,cwd_lock}` and its nine direct builtin callers `world_flag_overrides_disabled_config_and_env`, `world_flag_honors_directory_world_root_settings`, `anchor_flags_override_configs_and_export_anchor_env`, `no_world_flag_disables_world_and_sets_root_exports`, `cd_bounces_when_caged_without_world`, `cd_bounces_when_caged_with_world_enabled`, `anchor_guard_bounces_chained_cd_when_world_disabled`, `anchor_guard_bounces_chained_cd_when_world_enabled`, `cd_allows_uncaged_escape_from_anchor`; `execution/routing/dispatch/tests/host_replay.rs::async_repl_host_commands_record_replay_context`; `execution/settings/tests.rs::{CwdGuard,effective_root_uses_current_directory_for_follow_mode}`; `repl/async_repl.rs::tests::CurrentDirGuard`; and `execution/agent_runtime/host_session_authority/store_tests.rs::legacy_transaction_ignores_environment_and_cwd_after_admission`. Their 100-test call closure changes only through those named guards/call sites; `WorldRootSettings::effective_root` and every other production reader remain byte-frozen.

The event-registry ten are: `execution/agent_events.rs::{schedule_demo_burst_emits_expected_events,publish_command_completion_failure_emits_task_end_with_cmd_id,publish_command_completion_none_emits_no_agent_event,publish_command_completion_success_emits_task_end_when_enabled}`; `execution/routing/dispatch/world_ops.rs::{emit_stream_chunk_some_emits_orchestration_scoped_agent_event,emit_stream_chunk_none_emits_no_orchestration_scoped_agent_event,process_agent_stream_body_uses_launch_owned_run_id_for_stream_rows}`; `execution/routing/dispatch/tests/telemetry.rs::{consume_agent_stream_buffer_without_context_suppresses_agent_events,consume_agent_stream_buffer_preserves_runtime_event_identity_unchanged}`; and `repl/async_repl.rs::emit_world_restarted_alert_only_emits_with_orchestration_context`.

The global-trace thirteen are the four `execution/manager.rs` callers `configure_manager_init_generates_snippet_and_exports_env`, `configure_manager_init_honors_overlay_file`, `manager_env_script_sources_manager_and_legacy_snippets`, and `configure_child_shell_env_clears_inherited_manager_env_guard`; the eight `execution/routing/builtin/tests.rs` callers `export_builtin_sets_plain_pairs`, `export_builtin_defers_when_value_needs_shell`, `unset_builtin_clears_variables`, `cd_bounces_when_caged_without_world`, `cd_bounces_when_caged_with_world_enabled`, `anchor_guard_bounces_chained_cd_when_world_disabled`, `anchor_guard_bounces_chained_cd_when_world_enabled`, and `cd_allows_uncaged_escape_from_anchor`; and `execution/routing/dispatch/tests/host_replay.rs::async_repl_host_commands_record_replay_context`.

The global-broker 23 are `execution/invocation/tests.rs::{wrap_mode_uses_cli_shell_and_shimmed_path,skip_shims_and_no_world_disable_shimmed_path}`; `execution/platform/mod.rs::world_enable_exports_policy_fs_mode`; `execution/routing/builtin/tests.rs::{world_flag_overrides_disabled_config_and_env,world_flag_honors_directory_world_root_settings,anchor_flags_override_configs_and_export_anchor_env,no_world_flag_disables_world_and_sets_root_exports}`; `execution/routing/dispatch/tests/host_replay.rs::async_repl_host_commands_record_replay_context`; and the fifteen `execution/routing/dispatch/tests/pty.rs` callers of `support::with_test_mode`: `test_needs_pty_ssh`, `test_needs_pty_known_tuis`, `test_needs_pty_shell_meta`, `test_needs_pty_integration`, `test_needs_pty_ssh_variations`, `test_needs_pty_quoted_args`, `test_needs_pty_pipes_redirects`, `test_repl_heuristic`, `test_debugger_pty`, `test_windows_exe_handling`, `test_container_k8s_pty`, `test_wrapper_commands`, `test_pipeline_last_tui`, `test_ssh_spacing_edge_cases`, and `test_git_commit_edit_flag`.

The private retry-hook nine are `request_private_stop_after_transport_registration_for_stop_episode_retries_response_level_refusals_after_recovered_owner_path`, `request_private_stop_after_transport_registration_for_stop_episode_returns_original_owner_unreachable_when_recovery_never_becomes_ready`, `request_private_stop_after_transport_registration_for_stop_episode_does_not_retry_response_level_protocol_error`, `request_private_stop_after_transport_registration_for_stop_episode_retries_initial_connection_refused_transport_after_already_recovered_owner_path`, `dispatch_contract_stop_world_worker_persists_detached_closeout_after_refused_transport_for_same_caller_active_attached_truth`, `dispatch_contract_stop_world_worker_spec64_recovery_harness_drives_the_real_refreshed_transport_failure_branch`, `dispatch_contract_stop_world_worker_persists_detached_closeout_after_refused_transport_when_session_parks_without_sanctioned_owner`, `dispatch_contract_stop_world_worker_surfaces_detached_revalidation_contract_error_after_refused_transport`, and `dispatch_contract_stop_world_worker_surfaces_sanctioned_refresh_contract_error_after_owner_unreachable`, all in `execution/orchestrator_world_dispatch.rs`.

The dispatch-tracker four are `dispatch_contract_continue_world_worker_fork_command_rejects_live_retained_worker_cap_before_delivery`, `dispatch_contract_steering_policy_rejects_ephemeral_concurrency_cap_exceeded`, `dispatch_contract_steering_policy_rejects_live_retained_worker_cap_exceeded`, and `dispatch_contract_steering_policy_rejects_fork_when_live_retained_worker_cap_is_exceeded`, all in `execution/orchestrator_world_dispatch.rs`.

The deterministic-ordering 19 are `execution/agent_runtime/control.rs::inflight_attach_join_does_not_accept_detached_live_owner_as_attached_success`; `execution/agent_runtime/host_session_authority/store_tests.rs::legacy_owned_lock_is_released_when_holder_process_exits`; `execution/agent_runtime/state_store.rs::preactivation_state_store_writer_uses_shared_cross_process_root_lock`; `execution/lock.rs::test_concurrent_lock_attempts`; `execution/orchestrator_world_dispatch.rs::{continue_fork_compatibility_returns_at_exit_without_waiting_for_transport_eof,dispatch_contract_continue_world_worker_fork_command_submits_rendered_prompt_to_exact_retained_source,dispatch_contract_continue_world_worker_fork_command_tolerates_late_child_durable_publication_after_authoritative_registration,b1_b21_0_ordinary_continue_joins_canonical_retained_target_and_supervision,dispatch_contract_cancel_world_work_ephemeral_routes_exact_active_task_over_execute_cancel,dispatch_contract_stop_world_worker_waits_for_late_private_stop_transport_publication_after_recovered_owner_path,dispatch_contract_fork_world_worker_returns_typed_lineage_after_authoritative_bootstrap,dispatch_contract_fork_world_worker_waits_for_child_durable_publication_before_persisting_lineage,dispatch_contract_fork_world_worker_tolerates_late_child_durable_publication_after_authoritative_registration,dispatch_contract_fork_world_worker_rolls_back_child_when_lineage_persist_fails,wait_for_fork_child_durable_publication_allows_late_child_visibility_without_extending_stop_transport_budget,wait_for_fork_child_durable_publication_keeps_stop_transport_timeout_short_once_child_is_visible}`; `execution/routing/dispatch/world_ops.rs::process_agent_stream_requests_cancel_after_start_frame`; and `repl/async_repl.rs::{shutdown_host_orchestrator_runtime_waits_for_cancel_completion_before_stopping,hidden_owner_private_stop_fails_closed_when_completion_never_resolves}`. Only the sleep expression that establishes ordering is editable; every protocol deadline remains frozen.

The termination-confirmed ten are the nine existing F0 tests named in `03` plus
`execution/orchestrator_world_dispatch.rs::wait_for_fork_child_durable_publication_keeps_stop_transport_timeout_short_once_child_is_visible`.

The private-transport-root allowlist is the exact union of these 62 test bodies found by direct
path-call plus registration-helper closure, with no wildcard:

- the 15 `execution/orchestrator_world_dispatch.rs` tests reaching a private transport registration helper: `dispatch_contract_continue_world_worker_fork_command_submits_rendered_prompt_to_exact_retained_source`, `dispatch_contract_continue_world_worker_fork_command_tolerates_late_child_durable_publication_after_authoritative_registration`, `dispatch_contract_cancel_world_work_returns_typed_closeout_after_authoritative_cancel`, `dispatch_contract_stop_world_worker_returns_typed_closeout_after_authoritative_stop`, `dispatch_contract_stop_world_worker_fails_closed_when_participant_reaches_stopped_without_terminal_proof`, `dispatch_contract_stop_world_worker_late_terminal_proof_does_not_retroactively_convert_earlier_failure`, `dispatch_contract_stop_world_worker_waits_for_late_private_stop_transport_publication_after_recovered_owner_path`, `dispatch_contract_stop_world_worker_prefers_private_stop_surface_for_parked_resumable_with_live_transport`, `dispatch_contract_fork_world_worker_returns_typed_lineage_after_authoritative_bootstrap`, `dispatch_contract_fork_world_worker_waits_for_child_durable_publication_before_persisting_lineage`, `dispatch_contract_fork_world_worker_tolerates_late_child_durable_publication_after_authoritative_registration`, `dispatch_contract_fork_world_worker_rolls_back_child_when_lineage_persist_fails`, `dispatch_contract_fork_world_worker_times_out_missing_child_registration_and_rolls_back`, `wait_for_fork_child_durable_publication_allows_late_child_visibility_without_extending_stop_transport_budget`, and `wait_for_fork_child_durable_publication_keeps_stop_transport_timeout_short_once_child_is_visible`;
- the 21 additional direct stop-path fixtures in that file: `detached_stop_world_worker_closeout_availability_accepts_parked_refused_transport_when_target_runtime_truth_is_stale`, `detached_stop_world_worker_closeout_availability_fails_closed_for_non_refused_transport_errors`, `detached_stop_world_worker_closeout_availability_recheck_accepts_parked_truth_without_sanctioned_owner`, `detached_stop_world_worker_closeout_availability_recheck_accepts_same_caller_active_attached_after_refused_transport`, `detached_stop_world_worker_closeout_availability_recheck_stays_closed_for_missing_socket_not_found`, `detached_stop_world_worker_closeout_availability_rechecks_fresh_session_truth_after_refused_transport`, `detached_stop_world_worker_closeout_availability_rejects_active_missing_transport_after_not_found_error`, `detached_stop_world_worker_closeout_availability_rejects_active_missing_transport_without_observed_unavailability`, `detached_stop_world_worker_closeout_availability_rejects_live_active_attached_refused_transport`, `dispatch_contract_stop_world_worker_persists_detached_closeout_after_refused_transport_for_same_caller_active_attached_truth`, `dispatch_contract_stop_world_worker_persists_detached_closeout_after_refused_transport_when_session_parks_without_sanctioned_owner`, `dispatch_contract_stop_world_worker_persists_detached_recoverable_closeout_for_stale_active_attached_owner`, `dispatch_contract_stop_world_worker_persists_parked_resumable_closeout_when_private_stop_socket_is_stale`, `dispatch_contract_stop_world_worker_persists_parked_resumable_closeout_without_private_transport`, `dispatch_contract_stop_world_worker_spec64_recovery_harness_drives_the_real_refreshed_transport_failure_branch`, `dispatch_contract_stop_world_worker_surfaces_detached_revalidation_contract_error_after_refused_transport`, `dispatch_contract_stop_world_worker_surfaces_sanctioned_refresh_contract_error_after_owner_unreachable`, `request_private_stop_after_transport_registration_for_stop_episode_does_not_retry_response_level_protocol_error`, `request_private_stop_after_transport_registration_for_stop_episode_retries_initial_connection_refused_transport_after_already_recovered_owner_path`, `request_private_stop_after_transport_registration_for_stop_episode_retries_response_level_refusals_after_recovered_owner_path`, and `request_private_stop_after_transport_registration_for_stop_episode_returns_original_owner_unreachable_when_recovery_never_becomes_ready`;
- the 26 `repl/async_repl.rs` tests reaching direct paths or a registration helper: `start_host_orchestrator_runtime_persists_participant_snapshots_across_lifecycle_states`, `start_host_orchestrator_runtime_fails_closed_when_attached_control_exits_before_stable_startup`, `start_host_orchestrator_runtime_does_not_persist_live_manifest_without_session_handle`, `shutdown_host_orchestrator_runtime_waits_for_cancel_completion_before_stopping`, `hidden_owner_private_stop_fails_closed_when_completion_never_resolves`, `greenfield_host_start_proposal_applies_exact_host_world_binding_without_legacy_placement`, `hidden_owner_helper_attach_startup_reaches_ready_attached`, `hidden_owner_helper_attach_startup_fails_closed_without_continuity`, `shutdown_host_orchestrator_runtime_parks_resumable_host_session_on_detach`, `shutdown_host_orchestrator_runtime_fails_closed_when_detached_continuity_breaks`, `start_member_runtime_reuses_parent_session_and_persists_world_binding`, `orchestrator_world_dispatch_surface_spawns_authoritative_member_runtime`, `orchestrator_world_dispatch_surface_routes_stop_into_durable_closeout_and_runtime_cleanup`, `orchestrator_world_dispatch_surface_routes_valid_cancel_requests_into_typed_cancel_closeout`, `orchestrator_world_dispatch_surface_validates_cancel_requests_before_packet_one_unsupported_dispatch`, `orchestrator_world_dispatch_surface_rejects_denied_cancel_requests_before_packet_one_unsupported_dispatch`, `orchestrator_world_dispatch_surface_routes_valid_fork_requests_into_packet_three_retained_bootstrap_launch`, `orchestrator_world_dispatch_surface_stops_fork_child_without_eviction_of_source_runtime`, `orchestrator_world_dispatch_surface_validates_fork_requests_before_packet_one_unsupported_dispatch`, `orchestrator_world_dispatch_surface_rejects_denied_fork_requests_before_packet_one_unsupported_dispatch`, `orchestrator_world_dispatch_surface_validates_stop_requests_before_packet_three_routing`, `orchestrator_world_dispatch_surface_validates_inspect_requests_before_dispatch_routing`, `orchestrator_world_dispatch_surface_routes_well_formed_inspect_into_runtime_resolution`, `orchestrator_world_dispatch_surface_round_trips_successful_inspect_without_mutating_retained_runtime_handles`, `prepare_member_replacement_runtime_preserves_resumed_from_lineage`, and `retained_member_dispatch_parity_subset_prefers_pending_replacement_truth`.

Only test-owned short root/home/store construction and cleanup in those bodies may change. The
production path functions and every other consumer remain frozen.

The source list above is closed: no Rust file outside it or the already-published F0/F0a/F0b
tables may change. New test-only helpers may be private within those exact files; no new tracked
file is authorized. The only production-adjacent private seam remains the one already authorized
F0b `PublicPromptRenderer::render` delegation. Every other production function, public signature,
runtime caller, cache/registry representation, global trace API, descriptor behavior, socket
endpoint, timeout, retry, policy, capability, credential, platform selector, and service lifecycle
is frozen.

The refreshed graph reported CRITICAL for `world_env_guard` (70 impacted, 35 direct, five process
flows, nine modules); MEDIUM for `execution/settings/tests.rs::EnvGuard::new` (eight direct
same-file test callers); HIGH for `is_force_pty_command`,
`WorldRootSettings::effective_root`, and `refresh_socket_activation_report`; MEDIUM for
`TraceContext::init_trace`; and LOW/graph-under-resolved for `WORLD_ENV_LOCK`,
`AmbientSelectionGuard::{set,drop}`, its named gateway test, `EnvGuard::drop`, the two named
settings tests, `routing/test_utils.rs::{set_env,restore_env}`, and the host-replay test. These are
future adjacency warnings, not authorization to edit their production bodies. The docs-only F0-HC
change affects zero execution processes.

## A1.1d historical differential authority crosswalk

This correction moves no runtime seam. It only assigns each retained artifact the claim it can
support. Its bounded provenance result is `HistoricalParallelArtifactUnavailable`:

| Evidence | Authority | Permitted claim | Prohibited claim |
|---|---|---|---|
| Complete historical serial wall | Semantic differential authority | Exact names, normalized signatures, and the complete serial transition matrix | None of its rows may be silently reclassified or waived |
| Three final parallel walls plus one final serial wall | Final concurrency authority | Exact four-wall identity proves no candidate outcome depends on parallel versus serial mode | It does not reconstruct historical parallel names |
| Historical parallel aggregate `1263/1113/150/0` and transcript fragments | Diagnostic evidence only | The old wall exhibited materially different aggregate parallel behavior; the same-run panic-header/summary union recovers 150 names, but only 37 have complete retained panic output | Any named transition, historical signature comparison, parallel `PassToFail`, or “105 named failures became passes” claim |

The 17 and only 17 authorized `NewPass` tests are:

- `execution/mod.rs::authority_env_tempdir_cleanup_precedes_unlock_during_panic`;
- `execution/mod.rs::authority_env_test_guard_blocks_competitor_until_pair_cleanup_and_restoration`;
- `execution/mod.rs::authority_env_test_guard_bounds_subprocess_inheritance_through_wait_and_output`;
- `execution/mod.rs::authority_env_test_guard_holds_boundary_through_aborted_server_cleanup`;
- `execution/mod.rs::authority_env_test_guard_nested_home_override_preserves_outer_pair`;
- `execution/mod.rs::authority_env_test_guard_panic_releases_boundary_for_cross_thread_reacquisition`;
- `execution/mod.rs::authority_env_test_guard_preserve_restores_direct_mutations`;
- `execution/mod.rs::authority_env_test_guard_preserves_unicode_and_empty_values`;
- `execution/mod.rs::authority_env_test_guard_restores_exact_non_unicode_pair`;
- `execution/mod.rs::authority_env_test_guard_restores_home_and_socket_as_one_snapshot`;
- `execution/mod.rs::authority_env_test_guard_restores_pair_after_panic_without_poisoning`;
- `execution/mod.rs::world_socket_test_guard_blocks_competitor_until_cleanup_and_restoration`;
- `execution/mod.rs::world_socket_test_guard_nested_acquisition_restores_in_stack_order`;
- `execution/mod.rs::world_socket_test_guard_restores_after_panic_and_allows_reacquisition`;
- `execution/mod.rs::world_socket_test_guard_restores_exact_non_unicode_prior_value`;
- `execution/mod.rs::world_socket_test_guard_restores_prior_absence`; and
- `execution/orchestrator_world_dispatch.rs::fork_publication_stable_reader_waits_for_authority_home_fixture_cleanup`.

Deterministic endpoint listings prove zero removed, renamed, substituted, or newly ignored tests.
The exact 45-file candidate is committed as `770a6a9de9f537f7bc179c75421abbc3fff05b8d`
(tree `61fdd2e9476f1ce3e041720ce106f7c3427895be`). Its manifest, per-file fingerprints,
ordinary patch, and full-index patch SHA-256 values are respectively
`b9e3a44dd671409f66e2d62d48cb494ab147069a74ed71f2308e031f56372ae6`,
`41cb1a325add4efd8198872b456c3ae0fba73f8c4943e4b59c06e2bc76d8b479`,
`7ab220a715f4ae3389be314da2e6b0e614fcffb7a92166f04fde999570801861`, and
`adc1c5952e2ac4cc97881a1bf4df8e24b00d4d4ba337e692bf21965151d5c0c4`.

The corrected validator closes 86 environment names as 74 parent-mutated plus 12 child-only or
read-only, with 395 primitives, 73 dynamic calls, 43 sinks, 1,005 resolved callsites, 534 tests in
35 files, 38 resource rows, and zero unresolved dynamic rows. All 38 dispositions are implemented
or retained. Focused proof covers HOME/socket overlap, renderer isolation, negative projections,
environment/CWD restoration, global state, fork publication, async termination, subprocess
isolation, exact absence/non-Unicode restoration, panic/poison/nesting/reacquisition, and child
failure propagation.

Three fresh parallel walls and one serial wall each report `1280/1235/45/0`; every wall has failure
name-set SHA-256 `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`
and normalized-signature-set SHA-256
`33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`.
The serial matrix is exactly `1202/0/45/0/16/0/0/17/0/0` in the contract's row order. The 16
`FailToPass` rows resolve causally to seven detached-availability, two deterministic fork
publication, six stop-dispatch, and one hidden-owner harness fixes. The 17 `NewPass` rows are
exactly the authorized list above.

The three prior implementation reviews—`/root/final_review_env_closure`,
`/root/final_review_renderer_injection`, and `/root/final_review_lifecycle_subprocess`—remain CLEAN
for these exact candidate bytes. Fresh reviewer `/root/final_containment_corrected_authority`
(task `019f8681-7957-7cc3-88fa-37ab3ad2fc87`) returned CLEAN. GitNexus change detection attributes
one generated MEDIUM process label to the test-only `AuthorityEnvTestTempDir::new`; source closure
finds no changed production execution flow. The sole production hunk is the authorized mechanical
F0b writer delegation with byte-identical output. Product and user-facing behavior do not change,
and no seam is promoted. At that historical checkpoint F0, F0a, F0b, and F0-HC were complete and
the exact next packet was **A1.1d-5R2-2F — Authenticated world-deps and truthful doctor
composition**.

## A1.1d-5R2-2F readiness source closure

The earlier F status at the historical F0 closeout is retained as history. That readiness closure
recorded F1/F2 complete, F3/F4 blocked and preserved, and F incomplete. The correction added no
seam row and does not move lifecycle ownership.

| Symbol or caller | Direct/transitive route and cfg reach | GitNexus impact | Required disposition | Ownership |
|---|---|---|---|---|
| `world_ops.rs::ensure_world_service_ready` | Linux owner; directly reached by ordinary PTY execution, ordinary non-PTY request construction, pending-diff request construction, persistent-session WebSocket setup, and Linux initialization through a function-pointer probe. | **HIGH**; three graph-visible direct callers and three affected process groups. Source closure adds the persistent-session and function-pointer callers that reverse indexing omitted. | EDIT compatibility wrapper/delegation only; preserve signature and behavior. | Existing Linux lifecycle compatibility owner. |
| Private explicit-target extraction site in `ensure_world_service_ready` | Same Linux module and same probe/activation/stale/spawn/readiness/error flow. | Same **HIGH** owner boundary; no new process family. | ADD one private core or equivalent; no copied logic. | F may consume; ownership remains lifecycle. |
| `execute_world_pty_over_ws` | Ordinary world PTY dispatch. | LOW, zero upstream impacts from the caller label. | NO EDIT; remains on compatibility wrapper. | Compatibility. |
| `build_agent_client_and_request_impl` | Ordinary non-PTY request builder used by streaming and legacy world execution. | LOW, zero upstream impacts from the caller label. | NO EDIT; remains on compatibility wrapper. | Compatibility. |
| `build_agent_client_and_pending_diff_request_impl` | Pending-diff, workspace, recovery, and retained request paths. | LOW, zero upstream impacts from the caller label. | NO EDIT; remains on compatibility wrapper. | Compatibility/retained lifecycle. |
| `world_persistent_session::build_ws_and_start_session_frame` | Persistent-session start, reached from REPL and agents-command clients. | LOW, zero upstream impacts from the caller label. | NO EDIT; remains on compatibility wrapper. | Retained lifecycle. |
| `init_linux_world` | `initialize_world` -> `init_linux_world_default` -> function-pointer injection of the owner. | LOW, zero upstream impacts from the caller label. | NO EDIT; function-pointer type and behavior remain unchanged. | Linux initialization lifecycle. |
| `build_authenticated_world_deps_client_and_request` | Additive F-only builder; private Linux implementation and non-Linux fail-closed/static adapter. | LOW, zero upstream impacts. | EDIT builder only to validate first, call explicit readiness, and construct an exact environment. | F. |
| `surfaces.rs::run_world_command_for_deps_at` | Normal current/global/workspace/runtime world-deps leaf. | LOW, zero upstream impacts. | EDIT only as the first exact builder consumer. | F. |
| `provision_deps.rs::execute_with_profile` | World-enable dependency probe/install leaf. | LOW, zero upstream impacts. | EDIT only as the second exact builder consumer. | F. |

Graph-visible processes were the existing
`Build_agent_client_and_request_impl -> Resolved_socket_path`,
`Execute_world_pty_over_ws -> Resolved_socket_path`, and
`Build_agent_client_and_pending_diff_request_impl -> Resolved_socket_path` families, including
their capability-connect and activation-mode subflows. The owner impact returned HIGH because it
is shared lifecycle code; the correction is authorized only after that risk is recorded and only
for mechanical extraction/delegation. It is not authority to edit an existing caller or admit
R2-3/R2-4/R3 lifecycle work.

### Readiness semantics closed over the owner

| Existing behavior | Source-closed contract |
|---|---|
| Socket selection | Compatibility continues to resolve ambient `SUBSTRATE_WORLD_SOCKET` or `/run/substrate.sock`. F supplies `/run/substrate.sock` directly; the private core never resolves or replaces it. |
| Capability probe | Unix socket connect, 150 ms read/write timeouts, `GET /v1/capabilities`, and current 200-response interpretation remain unchanged. |
| Activation | Existing fixed service/socket-unit observation and activation-mode classification remain; retry is 100 ms through the existing 2,000 ms window. The explicit core may use mode, never the report's socket path. |
| Stale socket | Remove only in existing Manual mode; preserve in Unknown; no new cleanup authority. |
| Override | Existing compatibility socket-override no-spawn behavior and error remain. Ambient socket override cannot select or affect F's target. |
| Binary and spawn | Compatibility keeps environment/PATH/relative discovery order. F uses only immutable `/usr/local/bin/substrate-world-service`; spawn ownership and null stdio remain in the owner. |
| Readiness completion | Existing 50 ms polling through 1,000 ms and current error classification/text remain. |

`socket_activation.rs`, service units, installers, world-service, transport types, and non-Linux
adapters are inspected dependencies but are not editable F files. No callsite signature adaptation
is necessary. If implementation cannot preserve these rows within `world_ops.rs`, stop as
`ArchitecturalDecisionRequired`.

## Historical F5-PD nested diagnostic seam crosswalk

At that checkpoint this crosswalk superseded the prior statement that F5 could follow F3/F4
directly. F3/F4 were complete and F5 was blocked until F5-PD landed. The completed-F crosswalk
below supersedes this historical packet status. The existing public World doctor remains an
active compatibility diagnostic. The new passive seam is private to authenticated shim/Health
composition and branches before ordinary CLI bootstrap.

This section also supersedes PI-108's Route D-era authenticated Linux World Doctor
fixture-or-public-child clause. After F5-PD, that Linux branch uses the authenticated passive
child whenever the frozen World-enabled composition reaches it; the existing World-disabled
short-circuit still spawns no child and reads no fixture. `A/health/world_doctor.json` is test-only
on the enabled path. PI-108's separate world-deps fixture and
collector remain F5-owned. Non-Unix compatibility remains byte-frozen and is barred from proof.

### Source closure

| Stage and source location | Existing owner and observed behavior | F5-PD disposition |
|---|---|---|
| `crates/shell/src/builtins/shim_doctor/report.rs::{build_report,disabled_world_doctor_snapshot}` | `build_report` calls `gather_world_doctor_snapshot` only when World is enabled; otherwise it returns `disabled_world_doctor_snapshot` without a child or fixture lookup | Frozen compatibility owners; bodies/signatures and disabled classification remain unchanged |
| `crates/shell/src/builtins/shim_doctor/report.rs::gather_world_doctor_snapshot` | On Linux currently selects `A/health/world_doctor.json` before constructing the nested `world doctor --json` child | EDIT only the Linux branch reached from enabled composition to remove production fixture selection and construct the authenticated hidden passive child; Linux value decoding remains reachable only from `cfg(test)` direct evidence tests; non-Linux compatibility stays unchanged |
| `report.rs::run_json_subcommand` | Encodes the exact authenticated carrier on hidden argv, overwrites A-derived checked child projections, executes the current product binary, and currently parses raw stdout directly into `serde_json::Value`, which collapses duplicate object keys before later validation | EDIT only the Linux raw-stdout decode statement to call the additive exact-schema decoder before constructing `JsonCommandOutput`; child construction, carrier/projections, process execution, signature, exit/stderr handling, selected-access redaction, and all non-Linux code remain frozen |
| `report.rs::{snapshot_from_command,snapshot_from_value}` | The Linux command decoder currently infers success from absent `ok`/exit 0 and retains arbitrary child details/stderr; the value decoder does the same for fixture JSON | EDIT/add cfg-specific Linux implementations to require exact passive schema, identity, exit, and redaction closure; make Linux value decoding `cfg(test)`-only and route direct evidence through the same fail-closed validator; preserve existing non-Linux signatures/bodies/behavior |
| `crates/shell/src/execution/cli.rs::WorldAction::Doctor` | Parses the normal public World Doctor action | EDIT only this variant to add hidden bool `internal_passive_world_doctor_v1`; `Cli` itself and normal grammar/help are frozen |
| `crates/shell/src/execution/routing.rs::run_shell_with_cli` | Decodes/binds carrier at lines 369–389, then installs projections at 401 and calls the mutating home scaffold at 421 before config/routing | EDIT one exhaustive hidden-mode check and early return after authenticated decode/bind but before projection installation; this is the earliest bounded branch |
| `install_bootstrap.rs::decode_and_bind_unix_install_bootstrap_context` | Validates canonical carrier, commitment, current Unix principal, optional declared selector, and conflicting checked projections | Frozen authenticated compatibility owner; called unchanged |
| `install_bootstrap.rs::install_bootstrap_projections`; `home_bootstrap.rs::ensure_substrate_home_deps_scaffold_for_context`; `ShellConfig::from_cli` | Mutate process projections, may create scaffold, initialize later config/trace/routing state | Frozen and unreachable from passive mode |
| `execution/platform/mod.rs::handle_world_command` Doctor arm | Resolves launch CWD, binds authenticated world-deps config/policy, exports runtime env, and dispatches platform doctor | EDIT only the Doctor pattern to reject a leaked true internal bit before those actions; its ordinary false path and all public behavior are frozen |
| `execution/platform/linux.rs::world_doctor_main` | Reads host capability/filesystem state and socket-activation report; connects via `probe_world_socket`; calls the client doctor endpoint; handles public output/errors | Frozen public compatibility owner |
| `execution/socket_activation.rs::socket_activation_report` | Reads an ambient socket override, stats the socket, spawns `systemctl show`, and caches the report | CRITICAL compatibility owner; forbidden and unreachable from passive mode |
| `execution/platform/linux.rs::probe_world_socket` | `UnixStream::connect` plus `/v1/capabilities`; connecting the listening socket can activate the service | Frozen; forbidden and unreachable |
| `transport-api-client::AgentClient::doctor_world`; `world-service/src/lib.rs` route registration | Sends `GET /v1/doctor/world` to `handlers::doctor_world` | Frozen; forbidden and unreachable |
| `world-service/src/handlers.rs::doctor_world`; `world/src/overlayfs/strategy.rs::{select_strategy,run_enumeration_probe}` | Selects/mounts overlay/FUSE state and creates, lists, removes, and cleans a probe file/overlay | Frozen active probe owner; forbidden and unreachable |
| `execution/platform/linux.rs::legacy_world_doctor_report_v1_via_execute` | On HTTP 404 builds `/v1/execute` with `/tmp`, `mkdir`, `touch`, `ls`, `rm`, and `rmdir` probe work | Frozen execute fallback; forbidden and unreachable |
| F5 report composition | Converts the bounded validated snapshot into the existing report/human views and later validates exact A constituent identity | F5-owned cross-constituent composition remains blocked; F5-PD changes no report or transport schema |

The source path confirms the contradiction in the superseded contract: the required no-fixture
child was the normal public World doctor, while the same contract prohibited its possible socket
activation, endpoint, execute, and filesystem effects. Removing the child or replacing it inside
the parent with a fabricated unavailable value would also violate the child/carrier proof. F5-PD
keeps the real child boundary and carrier proof but selects a pre-bootstrap passive branch.

Authenticated Linux production never reads `A/health/world_doctor.json` after F5-PD. Its retained
value decoder is a test-only proof surface and cannot supply installed-product authority. The Linux
command decoder receives
the expected A prefix/commitment and accepts only schema 1, the compile-time platform label,
explicit top-level/host/world `ok=false`, `world.status=unavailable`, exact duplicate A identity,
no additional field, exit 4, and empty stderr. It then discards the raw JSON and constructs
`WorldDoctorStatus::NeedsAttention`, `ok=false`, source `command`, exit 4, no stderr/details, and
the exact bounded error `passive world doctor unavailable`. Missing/malformed/mixed/tampered,
wrong-exit, stderr-bearing, or recursively secret/request-bearing input constructs the same bounded
closed snapshot with error `passive world doctor incoherent`; it never echoes rejected bytes.

The private additive `decode_passive_world_doctor_child_v1` and its three exact
`PassiveWorldDoctor{Child,Host,World}V1` wire structs reject duplicate/unknown fields while parsing
raw Linux child stdout. The separate private `validate_passive_world_doctor_child_v1` owns
identity, exit, and redaction validation after that lossless gate. Its forbidden-key matching is case-insensitive and separator-insensitive for credentials,
tokens, API/private keys, authorization, passwords, secrets, prompts, request bodies/bytes/input,
carrier/auth-bundle bytes, parent/full environment, and commitment preimages. Selected A and its
non-secret commitment remain the only host identity permitted in the validated child input, but
the parent does not retain even those in the nested snapshot details. The broader F5 report owns
the already-public top-level identity fields when F5 later resumes. Non-Linux decoder/fixture/
public-child compatibility is unchanged, is not passive-path evidence, and cannot satisfy an
authenticated or native proof gate.

### Impact and ownership closure

GitNexus was current at the audited F3/F4 head. Counts below are upstream impact observations,
not permission to widen implementation. The frozen lifecycle rows were rerun with test edges
included and `maxDepth=5`: `ensure_world_service_ready` returned CRITICAL, 4 direct/10 impacted
graph nodes (4 direct/10 total impacted), four process roots, and five modules;
`socket_activation_report` returned CRITICAL, 6 direct/12 total impacted nodes, four process roots,
and five modules. Omitting test edges produces a narrower HIGH/four-module view; that
is not the recorded conservative closure.

| Existing symbol | Direct / total | Processes / modules | Risk | Ownership and authorization |
|---|---:|---:|---|---|
| `WorldAction` | 0 / 0 | 0 / 0 | LOW | Existing parser enum; one named hidden Doctor field authorized; this is an additive Rust enum-layout change, not a public CLI behavior change |
| `run_shell_with_cli` | 1 / 2 | 0 / 1 | LOW | Existing CLI root; bounded early branch authorized; `run_shell` and `main` frozen |
| `gather_world_doctor_snapshot` | 0 / 0 | 0 / 0 | LOW | Existing F5 composition owner; child argv edit authorized |
| `snapshot_from_command` | 1 / 1 | 0 / 1 | LOW | Direct caller is `gather_world_doctor_snapshot`; exact passive validation and bounded construction authorized |
| `snapshot_from_value` | 1 / 1 | 0 / 1 | LOW | Direct caller is currently `gather_world_doctor_snapshot`; Linux production call is removed and Linux decoding becomes test-only, while non-Linux compatibility is frozen |
| `handle_world_command` | 0 / 0 | 0 / 0 | LOW | Existing public dispatch owner; only a fail-closed guard/pattern for leaked hidden mode is authorized |
| `run_json_subcommand` | 0 / 0 | 0 / 0 | LOW | EDIT only its Linux raw-stdout parse expression to reject duplicate/unknown fields through the exact additive decoder; all transport, carrier, process, signature, stderr/exit, redaction, and non-Linux behavior is frozen |
| `JsonCommandOutput` | 1 / 1 | 0 / 1 | LOW | Existing captured JSON/exit/stderr container; layout frozen |
| `build_report` | 1 / 5 | 1 / 1 | LOW | Frozen caller; its enabled/disabled branch and composition remain unchanged |
| `disabled_world_doctor_snapshot` | 1 / 6 | 1 / 2 | LOW | Frozen disabled classification; no child or fixture lookup is added |
| `SubCommands`; `WorldCmd`; `WorldDoctorStatus` | 0 / 0 each | 0 / 0 each | LOW | Existing grammar/classification containers; matched or constructed without edit |
| `InstallBootstrapContextCarrierV1` struct | 1 / 1 | 0 / 1 | LOW | Existing carrier representation; called/imported unchanged, bytes and validation frozen |
| `decode_and_bind_unix_install_bootstrap_context` | 1 / 3 | 1 / 1 | LOW | Authentication owner; call unchanged, body/signature frozen |
| `install_bootstrap_projections` | 4 / 11 | 1 / 3 | HIGH | Frozen; passive branch must precede it |
| `ShellConfig::from_cli` | 7 / 8 | 1 / 3 | HIGH | Frozen and bypassed only by the authenticated hidden branch |
| `WorldDoctorSnapshot` | 4 / 10 | 1 / 3 | HIGH | Existing report schema; frozen, with no field/variant/signature change |
| `WorldDoctorReportV1` | 2 / 7 | 1 / 4 | HIGH | Existing wire schema; frozen |
| `ensure_world_service_ready` | 4 / 10 | 4 / 5 | CRITICAL | Lifecycle owner; no edit or passive call authorized |
| `socket_activation_report` | 6 / 12 | 4 / 5 | CRITICAL | Service-observation compatibility owner; no edit or passive call authorized |
| `probe_world_socket` | 2 / 2 | 2 / 1 | LOW | Public Host/World doctor transport; frozen |
| `AgentClient::doctor_world` | 3 / 16 | 1 / 2 | LOW | Linux/macOS/Windows transport owner; frozen |
| `legacy_world_doctor_report_v1_via_execute` | 1 / 1 | 1 / 1 | LOW | Public Linux fallback; frozen |
| `handlers::doctor_world` | 4 / 4 | 0 / 2 | LOW | World-service endpoint owner; frozen |
| `select_strategy`; `run_enumeration_probe` | 1 / 5 each | 0 / 2 each | LOW | Overlay probe owners; frozen |

Manual Rust-literal/pattern closure supplements the graph: `Cli` has one non-parser struct literal,
`auto_sync.rs::cli_for_auto_sync`, which is why F5-PD does not edit `Cli`. The only production
`WorldAction::Doctor` destructure is `platform/mod.rs::handle_world_command`; the Gateway-only test
match does not construct or destructure Doctor. The hidden nested field therefore requires exactly
that one allowlisted production pattern adaptation and no `auto_sync.rs` edit.

Additive private symbols have no pre-edit graph node. They own only hidden-mode exclusivity,
duplicate-rejecting private-wire decode, and bounded unavailable rendering; they do not own authentication, config, policy, service state,
lifecycle, transport, or world health. No HIGH or CRITICAL edit is authorized. Encountering a need
to edit any HIGH/CRITICAL row, add a process family/module owner, or call a frozen active row is a
mandatory stop, regardless of passing tests.

## A1.1d-5R2-2F completed seam crosswalk

This section is the controlling historical status for the F rows above. It supersedes their live
phrases that name F5-PD or F5 as the next packet; those phrases remain historical checkpoint
evidence. At that checkpoint F was complete, while renewed R2-2 integration closeout and every
later slice were still unstarted. Later RP3/RP4/RP5 publication work and the accepted R2-3 suffix
closeout now sit above that historical F checkpoint without rewriting it.

| Increment | Runtime proof commit before documentation replay | Exact owner/path closure | Final disposition |
|---|---|---|---|
| F1 | `922e1792fe886ebe871400e999858e451f8fcb01` | Sole `AuthenticatedWorldDepsContextV1` binder in `world_deps/mod.rs` | Exact A and commitment bind before world-deps reads or effects |
| F2 | `77fdcd8a139f3b72f7994f1a1692ddb4dfbf47e4` | Current/global/workspace, world-enable, Host/World dispatch propagation | Normal dependency scopes receive one request-scoped authenticated context |
| F3 | `0cd1d7af40347f3a149f2b84aca4d26ff1f09a3d` | Additive authenticated request builder in `world_ops.rs` plus export wiring | Exactly two authorized consumers; explicit readiness owner reused |
| F4 | `af2a3da6d85a6ca3f3f05bedf6768e4f89fd8e30` | Runtime probe/install/sync and provision/post-sync leaves | Exact fixed target and seven-entry generated environment; zero ambient forwarding |
| F5-PD | `653a7d91489563bc2a8e3395feeb53e159254240` | Hidden authenticated Linux passive World child and strict private decoder | No public Doctor change and no active lifecycle/probe path reachable |
| F5 | `2bb4696d7181d974c1b02e33d82e09422cdae7de` | Linux `WorldDepsDoctorSnapshotV1`, passive collector, exact-A parent composition, and bounded tests | Missing health evidence is unavailable; mixed/malformed/fixture runtime claims are incoherent; disabled remains child-free |

The final PI-106/PI-107/PI-108 F path is:

`authenticated A` → `A-derived config/global inventory plus request-scoped launch-CWD/workspace`
`inventory and world-deps scope` → `passive world-deps snapshot`
plus `authenticated F5-PD World child` → `exact-A fail-closed composition` → existing Host/World/
shim-doctor/Health JSON and human rendering.

There is no new seam owner. Carrier identity and commitment establish provenance, not runtime
health. A fixture cannot promote itself from compatibility evidence to product authority. No
ambient B, environment-only selector, contextless state, public Doctor invocation, readiness,
socket/service transport, execution endpoint, active probe, or side table participates in the
authenticated Linux result. Rejected evidence is not retained as a healthy report.

The final GitNexus comparison for F5 is LOW: four changed files, 27 mapped changed symbols
including test/line-shift attributions, zero affected execution flows, and no new process family.
The F5 semantic production delta is limited to the Linux cfg portions of
`WorldDepsDoctorSnapshotV1`, `collect_doctor_snapshot_v1`, `gather_world_deps_section`, and
`status_for_world_deps_report`. The same source hunks add mechanical
`cfg(not(target_os = "linux"))` separation to the two existing non-Linux functions and use
equivalent imported path qualifications; non-Linux behavior and wire output remain unchanged.

The landed managed-gateway secure-FD seam remains unchanged and regression-green. Its existence
does not close direct-member adoption: `RG-CONFIG-02`, `RG-CONFIG-04`, `RG-UAA-02`, and
`RG-UAA-03` remain open under D1/E3/D3. `RG-HOME-01` and `RG-INSTALL-01` also remain open. F does
not change policy/network/world-fs/caging/capability/lifecycle ownership, provide privileged smoke,
or promote any target seam.

At the completed F checkpoint, the exact historical next crosswalk node was **A1.1d-5R2-2 renewed
production-fix-free integration closeout**. The
[closeout-remediation crosswalk](../a1.1d-5r2-2-renewed-closeout/crosswalk.md#closeout-remediation-crosswalk)
supersedes only that next-node disposition: planning → R1 → P1 → fresh baseline now precede the renewed
production-fix-free closeout, and final docs/publication still precede R2-3.

**Source provenance:**
- extracted from [`02-seam-crosswalk.md#f0-test-proof-prerequisite-crosswalk`](../02-seam-crosswalk.md#f0-test-proof-prerequisite-crosswalk), lines 335–420; baseline span SHA-256 `8b99f2151dfa4a30d8a8d65905f2316bbc6c1345798ea0c9e661bcc2ee8b9032`
- extracted from [`02-seam-crosswalk.md#a11d-5r2-2f0-hc-corrected-consolidated-harness-crosswalk`](../02-seam-crosswalk.md#a11d-5r2-2f0-hc-corrected-consolidated-harness-crosswalk), lines 899–1367; baseline span SHA-256 `f9978fefb79bc25c568ff6054dbc88a659ede171dc52ca27e0c36e2ac9aaf490`
