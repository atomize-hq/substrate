**Kind:** contract/gate
**Stable ID:** `A1.1d-5R2-2F-family`
**Canonical for:** A1.1d-5R2-2F family contracts and gates
**Status:** canonical
**Authority scope:** exact extracted family-local source bodies only
**Source span:** composite of the preserved root compatibility spans listed in Source provenance below
**Supersedes:** canonical ownership of the extracted source bodies; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R2-2F family contracts and gates

##### Remaining R2-2 same-process carrier closure

The four remaining routes are closed only by explicit arguments:

```text
validated typed IH
  ├─ Host / Health / Config / Policy → named typed consumer
  ├─ intended_host_principal: PlatformPrincipalV1
  │    → REPL or hidden owner-helper runtime
  │    → prepared member dispatch
  │    → Unix account+UID database round-trip
  │    → account home/.codex
  ├─ host shell doctor → optional host diagnostic projection
  ├─ Health
  │    → health.rs
  │    → crate-private shim_doctor::collect_report_for_context name exposure
  │    → existing report::collect_report_for_context
  └─ direct Unix shim doctor
       → unchanged shim_doctor::run_doctor
       → existing report::collect_report_for_context
       → build_report
       ├─ [World enabled] gather_world_doctor_snapshot
       │    → run_json_subcommand
       │    → authenticated hidden passive world doctor child
       ├─ [World disabled] disabled_world_doctor_snapshot
       │    → no child or fixture lookup
       └─ [World enabled] A/health/world_deps.json, or gather_world_deps_section
            → collect_doctor_snapshot_v1
            → A-derived config/dependency paths
```

The principal projection is immutable and process/request-scoped. It is provenance-bound to the
validated IH, travels as a separate explicit argument alongside rather than inside prepared world
dispatch values, and is not independently authoritative or persisted into `HostSessionAuthority`,
orchestration/session records, receipts, supervisor state, retained-worker identity, policy
snapshots, or any other durable state, and never reconstructed from environment or process globals.
The Codex seed resolver requires `PlatformPrincipalV1::Unix`, calls unchanged read-only
`crates/shell/src/execution/install_bootstrap.rs::unix_account_home_for_principal` for the exact
account and UID round-trip through the account database, and derives only that account's home plus
`/.codex`. The terminal helper remains outside the editable allowlist and may not be duplicated. A,
`dirs::home_dir`, `HOME`, `USERPROFILE`, root home, CWD, PID, and helper/session state are forbidden
credential-source selectors. Policy still decides whether the projection is permitted, and no
prompt, credential, auth payload, or secret enters logs, traces, errors, receipts, or new state.

The principal-less compatibility entrypoint
`dispatch_run_world_task_request_with_started_task_run_id_tx` remains intentionally uncalled by the
typed route and must fail closed before any allowlisted Codex seed injection. It may receive only an
item-level `dead_code` allowance whose reason names principal-less compatibility, intentional
non-use, and temporary R2-3 ownership. That annotation retains the frozen surface only; it changes
no visibility, signature, body, cfg, caller, output, error, or runtime behavior, and no second lint
allowance is permitted.

Route B containment is patch-bound to preserved ordinary/binary SHA-256
`f8f845ef6aa6688fdf60be6ed983bb119971d65895a0f38c25fc1dad7643a77f`, preservation commit
`a6e29a10ea3dc9cb673a22912002efb83658e7b2`, and the exact six-file PI-105 manifest. The refreshed
CRITICAL result attributes 17 symbols and 25 existing labels to only three semantic roots:
`run_shell_with_cli`, `handle_agent_command`, and
`build_agent_client_and_member_dispatch_request_impl`; the complete label-by-label mapping is in
`03-phase-slice-map.md`. Final containment requires the same six files, the preserved production
hunks plus only the item-level annotation, no new module or execution-family root, and fresh
semantic review. Raw future symbol-count movement is diagnostic rather than authority, but any
seventh file, other hunk, unmappable label, new owner/family, compatibility-body change, or Route
C/Route D/R2-3 semantic import is an `ImpactDecisionRequired` stop.

The exact post-lint, pre-format Route B candidate is preserved at ordinary/binary SHA-256
`e9da85eb206be522645120dfee459ee79ce48793bc61161487d4b5c4d2fda243`, commit
`07f3117aa0d2f3d57279d20fec204757bba8383a`, with the same six-file manifest. A single mechanical
successor may be produced only by repository-default `cargo fmt --all`. Relative to that candidate,
rustfmt may change layout only in `execution/agents_cmd.rs`,
`execution/orchestrator_world_dispatch.rs`, `execution/routing/dispatch/world_ops.rs`, and
`repl/async_repl.rs`; `execution/invocation/plan.rs` and `execution/routing.rs` remain byte-identical.
No identifier, expression, type, import membership, cfg, comment/string content, test, lint,
signature, caller, or control-flow change is authorized. The successor receives a new exact patch
hash and fingerprints only after an exact hunk audit, zero semantic diff with whitespace ignored,
format-check proof, fresh GitNexus label-to-root mapping, and fresh independent runtime review.
GitNexus count/label movement is formatting attribution drift only while the same six files, module
owners, semantic Route B roots, and execution families remain fixed. Any broader change stops under
the categories in `03-phase-slice-map.md`; this formatting exception cannot authorize Route C,
Route D, R2-3, capability, cleanup, or lifecycle behavior.

The reviewed formatted Route B candidate is preserved at ordinary/binary SHA-256
`ea9cf3e582650007083812ad70e0bf3198405e73d0cde0fb8ecfbedeb49884a2`, commit
`57e13d291abf1239aacee0020ac444ec05e11d56`, with the same exact six-file manifest. Its sole
security-remediation successor preserves that manifest and may change only three contracts plus
their focused colocated tests:

1. `SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME` is a reserved Substrate-owned output key.
   `maybe_inject_codex_auth_seed_home_for_policy` removes it before any backend, allowlist, or
   principal branch and before every early return. Non-Codex, non-allowlisted, missing-principal,
   and account/UID-resolution failure all leave the key absent. Only successful allowlisted Codex
   resolution from the exact typed principal inserts `account-home/.codex`, overwriting poison.
   Policy eligibility is unchanged, and no process-global environment mutation or ambient override
   is created.
2. Spawn keeps principal truth as a separate request-scoped argument. Compatibility direct and
   prepared entrypoints pass `None`; principal-aware direct and prepared entrypoints pass
   `Some(exact principal)` through `spawn_world_worker` and `spawn_prepared_world_worker` into the
   existing `execute_spawn_world_worker_stream` parameter. `PreparedSpawnWorldWorkerBootstrap`,
   HSA, admission, receipts, supervision, retained workers, policy snapshots, manifests, and wire
   schemas do not acquire the projection. Compatibility remains valid without credential
   projection and fails closed before allowlisted Codex injection when principal truth is absent.
3. The Unix account resolver and policy helper use compile-time
   `cfg(any(target_os = "linux", all(test, unix)))`, with only mechanically matching constant,
   import, and test-item cfg. Linux production remains enabled; Linux/macOS Unix tests may compile
   the helper; Windows tests do not compile Unix account calls. The Linux-only member-dispatch
   injector and `install_bootstrap.rs` ownership remain unchanged.

The bounded source/impact audit is:

| Return/error path | Current poisoned-key result | Required result |
|---|---|---|
| non-Codex backend | preexisting reserved value survives | key absent |
| Codex backend not exactly allowlisted | preexisting reserved value survives | key absent |
| allowlisted Codex with no typed principal | error with preexisting value retained | error with key absent |
| allowlisted Codex with invalid account/UID | error with preexisting value retained | error with key absent |
| allowlisted Codex with exact typed principal | resolved value overwrites poison | exact resolved value; poison absent |

| Function | Principal available? | Currently forwarded? | Required argument |
|---|---:|---:|---|
| `dispatch_orchestrator_world_request` direct Spawn | no | no | `None` |
| `dispatch_orchestrator_world_request_for_principal` direct Spawn | yes | no | `Some(&intended_host_principal)` |
| `dispatch_prepared_orchestrator_world_request` prepared Spawn | no | no | `None` |
| `dispatch_prepared_orchestrator_world_request_for_principal` prepared Spawn | yes | no | `Some(&intended_host_principal)` |
| Linux `spawn_world_worker` | caller-dependent | no | forward the separate option unchanged |
| `spawn_prepared_world_worker` | caller-dependent | no | forward the separate option unchanged |
| `execute_spawn_world_worker_stream` | option already accepted | yes for Fork/Continue-fork, `None` for Spawn | receive the exact Spawn option; body semantics unchanged |

| Symbol/test surface | Linux production | Unix test | Windows test |
|---|---:|---:|---:|
| `resolve_host_codex_seed_home` | compiled | compiled | excluded |
| `maybe_inject_codex_auth_seed_home_for_policy` | compiled | compiled | excluded |
| `maybe_inject_codex_auth_seed_home_for_member_dispatch` | compiled | not newly broadened | excluded |
| colocated Unix account/seed tests and imports | not applicable | compiled | excluded item by item |

GitNexus resolves the policy injector as LOW with four direct callers, one existing
`build_agent_client_and_member_dispatch_request_impl` process root (eleven generated labels), and
the Dispatch module; it resolves the account-home resolver as LOW with one direct caller and the
same process/module. It under-resolves the large Spawn functions. Manual source closure therefore
binds every Spawn call. Compatibility/principal-aware direct dispatch first calls unchanged
`prepare_authority_bound_spawn_world_worker`, then carries separate `None`/`Some` alongside its
result into `spawn_prepared_world_worker`. Compatibility/principal-aware prepared dispatch reaches
`spawn_world_worker` with `None`/`Some`; that function calls unchanged
`prepare_spawn_world_worker_bootstrap`, whose Linux arm calls unchanged
`prepare_authority_bound_spawn_world_worker`, then carries the separate option alongside the
prepared result into `spawn_prepared_world_worker`. Neither preparation function receives or stores
principal truth, and `PreparedSpawnWorldWorkerBootstrap` stays unchanged. The final hop forwards
the option into the already-principal-aware stream function. Fork and continue-fork already use
that final parameter and remain unchanged. A new exact successor patch, fingerprints, hunk
authorization map, fresh GitNexus output, and independent semantic review are required. A seventh
file, additional semantic hunk, durable principal representation, new module or execution-family
root, Route C/D, R2-3, capability, cleanup, or lifecycle change is not authorized.

##### Route C authenticated Host/World doctor projection

Route B remains complete, security-review-clean, Spawn-review-clean, cfg-review-clean, and
semantically unchanged at commit `6cee990f0370013c8b05a5495301db7aea642cd5`. Its exact
ordinary/binary patch from its parent is
`28e6b9da35f96b5f93c49369cbde0eda77e9a145b54f9c412bae8e5a83871674`, and its manifest remains
the reviewed six files. Route C starts only from preservation commit
`41b82327e23798719ed9a0b4cae1f557fb593670`, whose parent is the exact Route B commit/tree. The
starting diff is 182 insertions and five deletions in exactly five files. Its canonical
`git diff --full-index --binary` SHA-256 is
`5b436d5dfeaf6e513cbaa306de65848cbe3d5b003fa8c00589be09e54b630277`; its abbreviated-index
ordinary/binary rendering is
`7c5a908a6760244d9ad6922dfcf7e944cfce3c723d994e279a3aeaded7a8ffa0`. Canonical per-file
full-index binary fingerprints are:

| Route C file | SHA-256 |
|---|---|
| `crates/shell/src/execution/platform/linux.rs` | `d1c53712aef41e01cb44408e42735511af75b8432b8de651cf072a4d13be80a1` |
| `crates/shell/src/execution/platform/mod.rs` | `08340fc50773063113b032422a0a63742aee50d04c8cd8b4edd6c9c4ada13f4c` |
| `crates/shell/tests/doctor_scopes_ds0.rs` | `7f13f02e382dbbe55b53fc15a32183ab889d73a08fe8661b67169161bbe90ba7` |
| `crates/transport-api-types/src/lib.rs` | `82e510bd76fefec4c256c93e271761b1f9ba3047a003a03329979862db631457` |
| `crates/world-service/src/handlers.rs` | `cd24633ffd3049b4dcd30ad3da6d769302483061dcf0a2c473d4d38927bb6018` |

Those bytes authorize a starting candidate, not immutable completion bytes. Within the same five
files, review-driven remediation may correct only Route C tests or diagnostic projection while
remaining inside this semantic envelope; every such correction establishes a new exact patch hash
and per-file fingerprint manifest and reruns impact, focused/broad proof, differential, and fresh
review. A sixth file or a broader authority owner requires `CrossDocumentChangeRequired`.

The authority and disclosure contract is closed:

1. The already-authenticated typed `InstallBootstrapContextCarrierV1` is the only source for A and
   its commitment. `handle_host_command` and `handle_world_command` only forward that typed value;
   `host_doctor_main` and `world_doctor_main` only project it. No second resolver, ambient recovery,
   generated-record authority, process-global state, or side table is permitted.
2. `WorldDoctorReportV1` host fields are optional/defaulted and omit when absent. They report
   `selected_host_prefix` and the non-secret host-context commitment only; they cannot establish,
   validate, replace, or mutate the authenticated context. The world-service `doctor_world`
   producer sets them to `None` and reads no host carrier, HOME/root/prefix, principal, or authority
   state. The legacy adapter may populate them only from explicitly supplied typed IH.
3. Diagnostics return or log no hidden carrier bytes, credential or secret bytes, request bytes, or
   sensitive principal material. The change cannot mutate installation, service, world, policy,
   capabilities, filesystem/network enforcement, placement, caging, credentials, receipts,
   supervision, retained workers, cleanup, or lifecycle/execution behavior.
4. Every source-level process attribution must map to exactly the existing diagnostic roots
   `handle_host_command`, `host_doctor_main`, `handle_world_command`, or `world_doctor_main`.
   Representation/test symbols, the legacy helper, and the mechanical world-service `None`
   producer create no new process root. Route D and R2-3 ownership remain untouched; no seam is
   promoted.

The preservation-time GitNexus result was CRITICAL with 25 attributed symbols, 32 process labels,
and the exact five files. A pre-authorization refresh over byte-identical files reported CRITICAL
19/32/five. Both results are retained as observed evidence. A raw count or generated-label change
is attribution-only only when the patch/fingerprints and five-file manifest remain exact, every
source-level label still maps to the same four roots, no module owner/execution family appears, and
a fresh independent reviewer confirms semantic containment. An actual new call path, sixth file,
module owner, execution family, or authority source is `ImpactDecisionRequired`; stale-index
pinning is forbidden.

Route C proof requires transport API tests; focused and full world-service library tests; installed-
witness Host projection; World doctor typed-A JSON; conflicting ambient-B negatives; malformed or
missing typed-context fail-closed cases where applicable; warnings-denied Clippy for touched crates;
shell all-target compilation; workspace all-target check; formatting; and diff checks. The inherited
disabled-world doctor failure is first reproduced at the clean replayed Route B baseline and then
compared exactly; Route C may not fix, suppress, rename, weaken, or reinterpret it. The broad shell
differential requires `PassToFail = 0`, `NewFail = 0`, no removed/renamed/substituted/weakened tests,
unchanged normalized signatures for retained baseline failures, and a causal audit of every
`FailToPass` that excludes test bypass. Completion additionally requires fresh independent
authority/security, call-path/impact, and cross-platform/regression reviews. Route C remains
uncommittable until all are CLEAN.

`WorldDoctorReportV1` remains a world-service/world-enforcement report with additive optional,
defaulted, omit-when-absent host prefix and commitment fields. The in-world `doctor_world` producer
sets them to `None` and reads no host IH, carrier, HOME, prefix, principal, or authority projection.
Only the host shell may enrich final host-visible doctor output from typed IH, and it does so before
either JSON serialization or text rendering; the legacy Linux fallback may populate the same fields
only when typed IH is explicitly supplied. Old JSON without
the fields remains valid, and world enforcement, Landlock, netfilter, filesystem strategy, policy,
health, and service behavior remain unchanged.

For shell/shim doctor, `shim_doctor/mod.rs` performs name visibility only: on Unix it exposes the
existing `report::collect_report_for_context` with crate-private visibility so sibling `health.rs`
can continue the typed route. It does not make the report module public, add a wrapper or resolver,
or alter either collector. The existing crate-private `collect_report` compatibility re-export may
receive only an item-level Unix `unused_imports` allowance with a reason naming temporary R2-3
compatibility ownership. The `report.rs::collect_report` function may receive only an item-level
Unix `dead_code` allowance with the same temporary ownership. These annotations retain code only:
they do not change visibility, signatures, bodies, cfg branches, callers, outputs, environment
behavior, or non-Unix behavior, and no file-/module-level or other lint suppression is permitted.
`gather_world_deps_section`, `try_load_health_fixture`, `health_fixture_path`, and
`collect_doctor_snapshot_v1` retain the Route D/F5-owned A-rooted world-deps fixture/collector
closure. For nested World Doctor, the later historical F5-PD contract, whose boundary the
completed-F gate record retains, superseded the historical fixture-or-public-child behavior:
whenever Linux World-enabled production reaches
`gather_world_doctor_snapshot`, it invokes the authenticated hidden passive child, and
`A/health/world_doctor.json` is `cfg(test)` evidence only on that path. The World-disabled
short-circuit remains unchanged and spawns no child. Non-Linux compatibility remains
behavior-frozen and cannot satisfy proof.
The child receives the same canonical hidden argv carrier and context-derived checked projections;
no contextless repository-binary invocation may satisfy the path. The parent environment is not
changed. The child transport never formats the carrier into a command string, output, error, log,
trace, snapshot, or fixture, and no credential, request byte, non-public commitment, or sensitive
principal datum is added to diagnostics. World-deps selected-A fixture payload/source semantics
and non-Unix cfg behavior remain unchanged.

Unix `collect_report` is the explicitly named compatibility path: it retains its checked-projection
behavior unchanged, cannot be selected by typed Health, and cannot satisfy R2-2 product proof. Its
body, visibility, cfg, caller, output, and crate-private re-export remain frozen apart from the
already-authorized item-level lint retention. Physical-shim, global-trace, replay, and
platform-native compatibility migration remains R2-3. No diagnostic reconstructs a default home,
creates a second context constructor/resolver, or changes world, policy, capability, installation,
cleanup, service, credential, receipt, supervisor, retained-worker, or lifecycle semantics.

The Route D source-closure impact observations are LOW: `gather_world_doctor_snapshot` has one
direct/five transitive impacts, `try_load_health_fixture` two/four, `health_fixture_path` one/four,
and `run_json_subcommand` one/three. All resolve to the existing Health/shim-doctor family. These
counts are evidence rather than an exact ceiling; final containment is decided by the approved
source paths and symbols, unchanged owner/family set, exact manifest/patch, mapped GitNexus output,
and fresh read-only semantic review. A new execution family, authority owner, module, resolver,
schema, or R2-3 behavior requires a cross-document stop.

#### R2-2 source closure, E closeout, F0/F0a/F0b prerequisites, and remaining F contract

The failed Routes A–D integration closeout established that those four routes were individually
review-clean but did not close every authenticated-context consumer. R2-2E has since implemented and
proved the gateway projection without changing Routes A–D; all five routes are individually
review-clean. F0/F0a/F0b/F0-HC are now implemented, proof-complete, review-clean, canonically
closed out, and preserved. R2-2 stays incomplete pending F and the renewed integration closeout. The
following table preserves E's reviewed historical source-closure boundary
and freezes the completed F0/F0a/F0b boundaries plus F's remaining boundary;
brace groups are exact symbol sets, not file wildcards.

| Increment | File | Symbol | Pre-increment authority | Required carrier | Callers | Process family | Platform cfg | GitNexus risk | Inventory ID | Test | Reviewed allowlist disposition |
|---|---|---|---|---|---|---|---|---|---|---|---|
| R2-2E | `crates/shell/src/execution/platform/mod.rs` | `handle_world_command` Gateway arm | typed carrier arrives but is dropped | already-authenticated A/request context | `ShellConfig::from_cli` | shell World dispatch | Unix authenticated; non-Unix fails before ambient selection | LOW; manual arm closure | PI-111 | `world_gateway.rs`; `agent_successor_contract_ahcsitc0.rs` | EDIT arm only |
| R2-2E | `crates/shell/src/builtins/world_gateway.rs` | `GatewayLifecycleRequestContext`; `run`; `run_inner`; `run_typed_action_with_status_args`; `run_typed_action`; `call_gateway_action`; `build_gateway_request_context` | ambient config/policy/network/inventory and pre-validation disabled routing | one validated A-derived gateway context | World Gateway status/sync/restart | gateway lifecycle | all; cfg clients below | LOW; struct one direct/five total | PI-111 | colocated; `world_gateway.rs` | EDIT |
| R2-2E | same | `validate_gateway_backend_selection`; `resolve_integrated_auth_payload`; `resolve_cli_codex_integrated_auth`; `codex_auth_state_path` | ambient inventory and `dirs`/HOME/USERPROFILE Codex home | explicit inventory plus committed-principal account home | `build_gateway_request_context` and auth helpers | gateway projection/credential handoff | Unix account leaf; cfg-coherent others | LOW | PI-111 | colocated; `agent_successor_contract_ahcsitc0.rs` | EDIT; credential content/lifecycle frozen |
| R2-2E | same | `world_routing_disabled` | ambient toggles can bypass failed context | validated A-derived effective config before classification | two typed action functions | gateway status/action | all | LOW; two direct/four total | PI-111 | colocated; `world_gateway.rs` | EDIT only after A validation |
| R2-2E | same | `synthesized_unavailable_response_without_context` | contextless unavailable response can bypass authentication | no contextless route | two typed action functions | gateway status/action | all | LOW; two direct/four total | PI-111 | colocated; `world_gateway.rs` | DELETE only |
| R2-2E | same | `build_gateway_client` Linux cfg | ambient `SUBSTRATE_WORLD_SOCKET` | fixed authenticated-route `/run/substrate.sock` | `call_gateway_action` non-macOS | gateway transport selection | Linux | LOW/graph under-resolved | PI-111 | colocated; `world_gateway.rs` | EDIT Linux cfg only |
| R2-2E | same | `build_macos_gateway_client`; `resolve_macos_gateway_client_endpoint`; `resolve_macos_host_gateway_socket`; `macos_default_world_socket_path` | ambient socket/home/VM and lower `auto_select`; endpoint helper is production-compiled dead/test support | no carrier: E fails in `call_gateway_action` before these symbols | `call_gateway_action` plus four test-only endpoint-helper callers | gateway transport selection | macOS | LOW locally; endpoint helper four test callers; lower adapter risk frozen | PI-111 | unchanged colocated cfg tests plus new pre-client rejection proof | SOURCE-CLOSURE INSPECTED, FROZEN R2-3; no edit/delete/call |
| R2-2E | `crates/shell/src/execution/policy_snapshot.rs` | `resolve_policy_snapshot_for_bootstrap_home`; additive explicit-bootstrap-home world-network resolver | snapshot resolver is explicit; network resolver re-enters ambient config | A bootstrap home and explicit config | gateway context; later F authenticated builder | policy/network projection | all | LOW | PI-111 | colocated | REUSE plus ADD; sole owner |
| R2-2E | `crates/shell/src/execution/agent_inventory.rs` | `resolve_gateway_backend_inventory_entry`; additive bootstrap-home gateway resolver; `load_effective_agent_inventory_for_bootstrap_home` | gateway resolver loads ambient inventory | A bootstrap home | gateway backend validation | runtime-family inventory | all | LOW | PI-111 | colocated | EDIT/ADD/REUSE |
| R2-2F0 | `crates/shell/src/execution/mod.rs` | existing `WORLD_ENV_LOCK`/`world_env_guard`; additive `WorldSocketTestGuard` and focused tests | shared reentrant lock exists but socket mutation/restoration is decentralized | exact `OsString`/absence plus same shared lock | shell library unit tests only | test harness | `cfg(test)` | HIGH historical 35 direct/70 total; fresh 13/19 | none; proof prerequisite | colocated prior/absence/panic/recovery/nesting/concurrency | ADD test-only helper/tests; existing helper body and production exports frozen |
| R2-2F0 | `crates/shell/src/execution/orchestrator_world_dispatch.rs` | 49 socket `EnvVarGuard::set_path` call sites and two manual restore blocks | local guards do not all acquire shared lock; manual blocks are not unwind-safe or exact non-Unicode restoration | `WorldSocketTestGuard` | named unit tests only | test harness | Linux `cfg(test)` module | graph-under-resolved; source closure binding | none; proof prerequisite | exact pair plus neighboring socket tests | EDIT test-only call sites; names/assertions and production symbols frozen |
| R2-2F0 | `crates/shell/src/execution/platform/macos.rs` | four test `with_env_var` socket call sites/readers | `#[serial]` and unlocked closure restoration | shared test guard around each complete closure | macOS unit tests | test harness | macOS tests | source reviewed | none; proof prerequisite | existing four tests plus guard proof | EDIT test helper/calls only; production platform adapter frozen |
| R2-2F0 | `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs` | two macOS tests with direct remove/set/remove | separate standard test lock; prior socket state not restored exactly | shared test guard retained through server/client cleanup | two unit tests | test harness | macOS tests | source reviewed | none; proof prerequisite | existing tests | EDIT test calls only; production persistent-session code frozen |
| R2-2F0 | `crates/shell/src/execution/routing/world.rs` | two direct socket mutations | shared lock is held, but mutation is not RAII-restored | shared test guard | two unit tests | test harness | macOS/Windows tests | source reviewed | none; proof prerequisite | existing tests | EDIT test calls only; production routing frozen |
| R2-2F0 | `crates/shell/src/builtins/world_enable/runner/paths.rs` | `resolve_world_socket_path_normalizes_relative_components` manual set/restore | `#[serial]` only; restoration is not unwind-safe | shared test guard | one unit test | test harness | tests | source reviewed | none; proof prerequisite | existing test | EDIT test call only; production path resolver frozen |
| R2-2F0 | `crates/shell/src/builtins/world_gateway.rs` | `classification_tests::with_env_var`; one Linux socket call | shared lock releases after panic without restoration because the closure helper restores only after normal return | shared test guard for the socket branch through the complete closure | one unit-test call; four separate RAII world-gateway calls unchanged | test harness | `cfg(test)` classification module | source reviewed | none; proof prerequisite | existing Linux fixed-socket test plus guard proof | EDIT test helper/call only; production gateway and four compliant RAII sites frozen |
| R2-2F0a | `crates/shell/src/execution/mod.rs` | generalize/reuse test-only `WORLD_ENV_LOCK` boundary for HOME plus socket; combined focused tests | socket guard candidate exists only on preserved branch; clean source has shared reentrant lock but decentralized HOME restoration | exact `OsString`/absence under one authority-environment lock | all same-process HOME/socket mutators and stable authority readers | test harness | `cfg(test)` | CRITICAL 35 direct/70 total | none; proof prerequisite | prior/absence/non-Unicode/panic/poison/nesting/concurrency/mixed-variable | EDIT/ADD test-only; no production export or behavior |
| R2-2F0a | `crates/shell/src/execution/agent_runtime/{auto_attach.rs,control.rs,state_store.rs,tool_invocation_contract.rs}`; `crates/shell/src/execution/host_inbox_materialization.rs`; `crates/shell/src/execution/agents_cmd.rs` | five `with_store` families (223 dependent tests) plus two agents-command helpers (five tests) | manual set/remove or local string guard; four callers are unannotated; restoration is not uniformly exact or unwind-safe | shared authority-environment guard held across complete helper callback | 228 dependent tests | test harness | shell library tests | tool-contract helper HIGH 12 direct; others MEDIUM/LOW or graph-under-resolved | none | exact HOME pair and all helper dependents | EDIT test helpers/calls only; names/assertions frozen |
| R2-2F0a | `crates/shell/src/builtins/{shim_doctor/report.rs,world_deps/mod.rs,world_enable/runner/manager_env.rs,world_gateway.rs}`; `crates/shell/src/execution/{agent_inventory.rs,config_model.rs,env_scripts.rs,invocation/tests.rs,orchestrator_world_dispatch.rs,routing/builtin/tests.rs,settings/tests.rs}`; `crates/shell/src/execution/agent_runtime/host_session_authority/store_tests.rs`; `crates/shell/src/repl/async_repl.rs` | remaining direct/local-guard/manual HOME mutators in the 435-test shell-library inventory | `#[serial]` and unrelated local guards do not exclude unannotated peers; some restore `String` rather than exact `OsString` | same authority-environment guard; dependent async/process lifetime ends before restore/unlock | remaining same-process mutating tests and stable readers | test harness | test modules only | source reviewed; large modules graph-under-resolved | none | combined HOME/socket neighbor and restoration wall | EDIT test-only sites/helpers; production symbols frozen |
| R2-2F0a | `crates/shell/tests/{shim_deployment.rs,agent_successor_contract_ahcsitc0.rs,support/mod.rs}` | parent-process HOME mutation in three independent integration source families | separate binaries; restoration is manual and not uniformly panic-safe/exact | equivalent per-binary exact-restoration boundary in every inventoried parent-mutating helper; no cross-process lock | nine serialized shim callers, one successor caller, one ignored one-test helper binary, one serialized support caller | integration test harnesses | existing cfgs | source reviewed | none | per-helper prior/absence/panic plus child inheritance | EDIT all three inventoried integration helper/call families; child-only `Command::env` files frozen |
| R2-2F0 | `crates/shell/src/execution/orchestrator_world_dispatch.rs` | nine named socket-owning tests with `server.abort()` and no awaited termination | task cancellation may remain pending while guard restores/unlocks and temp socket/root cleanup begins; one scheduler yield is insufficient | abort → await confirmed termination → complete/confirm fixture cleanup → restore environment → unlock | nine named tests in `03`/`05` | test harness | Linux test module | test symbols graph-under-resolved; source closure exact | none | termination-before-restore and zero task/socket/root leaks | EDIT nine test bodies only; no production server/readiness lifecycle change |
| R2-2F0b | `crates/shell/src/execution/agent_runtime/control.rs` | private Unix-only `PublicPromptRenderer::render`; ADD private explicit-writer core/adapter; DELETE `capture_stdout_once` and `capture_stderr_once`; two exact fallback tests | production render selects real stdout/stderr directly; tests replace process fd 1/2 with `dup2`, so parallel libtest reporter output can enter the private pipe | private explicit stdout/stderr writers for the core; production `render` delegates through real selected stream; tests own memory buffers | source-exact production callers `run_hidden_owner_helper_startup_prompt_stream_with_projection` and `run_public_prompt_command` remain frozen; two test callers migrate | production renderer plus test harness | Unix; helpers/tests under `cfg(test)` | `render` MEDIUM 4 direct/39 total/one process; generic-`new` HIGH is source-proven over-attribution and frozen; helpers/tests LOW | none; proof prerequisite | exact stdout/stderr bytes, stream selection, forced reporter overlap, private-buffer stress | EDIT `render` delegation and two tests; ADD private core/adapter; DELETE only two capture helpers; no caller/public API/production contract change |
| R2-2F | `crates/shell/src/execution/platform/mod.rs` | `handle_world_command` Doctor/Deps arms; `handle_host_command` Doctor arm | World doctor/deps resolve ambient config/policy or drop A; Host policy remains ambient | `AuthenticatedWorldDepsContextV1`-equivalent plus A-derived doctor inputs | `ShellConfig::from_cli` | World/Host dispatch | authenticated Unix/Linux; non-Unix compatibility unproven/no A claim | LOW; manual arm closure | PI-106/PI-107 | `doctor_scopes_ds0.rs`; world-deps suites | EDIT arms only |
| R2-2F | `crates/shell/src/execution/platform/linux.rs` | `host_doctor_main`; `world_doctor_main` | `detect_profile` plus global `world_fs_policy` | explicit A-derived world-fs policy and identity | platform handlers | Linux Host/World doctor | Linux | LOW | PI-106/PI-107 | `doctor_scopes_ds0.rs` | EDIT identity/policy projection only; public Doctor's active readiness/socket/service/endpoint/probe behavior stays unchanged |
| R2-2F | `crates/shell/src/builtins/world_deps/mod.rs` | new authenticated context/binder; `WorldDepsDoctorSnapshotV1`; `collect_doctor_snapshot_v1`; `resolve_effective_enabled_provisioning_requirements_v1` | diagnostic Unix path partly explicit; provisioning and non-Unix ambient; snapshot lacks identity | one validated A context with root/config/policy/deps/CWD/runtime projection | shim doctor; world-enable; surfaces | dependency resolution/diagnostics | Unix/Linux validation; non-Unix unavailable or explicit R2-3 ambient compatibility only | existing symbols LOW; new symbols N/A | PI-106/PI-107 | colocated; inventory/enabled/provision suites | ADD/EDIT |
| R2-2F | `crates/shell/src/builtins/world_deps/surfaces.rs` | `run`; `run_current`; `run_global`; `run_workspace`; `run_current_list`; `run_current_show`; `run_current_install`; `run_current_sync`; `run_global_list`; `run_global_add`; `run_global_remove`; `run_global_reset`; `run_workspace_list`; `run_workspace_add`; `run_workspace_remove`; `run_workspace_reset`; `resolve_global_available_inventory_view`; `build_current_show_explain_v1` | ambient H/config/deps and lower `current_dir` | shared context; explicit workspace CWD | world dispatch and post-provision sync | normal dependency CLI | symbols compile across existing cfgs; authenticated guarantee Unix/Linux only, other adapters R2-3/unproven or fail closed | LOW per entry/action | PI-106/PI-107 | all exact world-deps suites in `03` | EDIT exact symbols |
| R2-2F | same | `run_current_list_applied`; `compute_current_applied_items_v1`; `preflight_runtime_system_requirements_v1`; `probe_world_apt_requirements_v1`; `probe_world_pacman_requirements_v1`; `apply_install_plan_v1`; `reconcile_world_deps_bin_v1`; `apply_apt_entrypoint_wrappers_v1`; `apply_script_package_v1`; `resolve_script_body_for_package_v1`; `current_codex_runtime_target_triple_v1`; `run_world_command_output_for_deps`; `run_world_command_output_for_deps_with_profile`; `run_world_command_checked_for_deps`; `query_world_package_presence`; `query_world_package_entrypoint_presence`; `run_world_presence_check_v1`; `ensure_world_backend_available`; `run_world_command_for_deps`; `run_world_command_for_deps_at` | A inventory can feed ambient B request builder | shared context and authenticated request projection | current applied/show/install/sync and doctor | dependency runtime execution | symbols compile across existing cfgs; authenticated guarantee Unix/Linux only; profile-specific behavior frozen | LOW per symbol; frozen `resolve_current_inventory_view` HIGH | PI-106/PI-107 | applied/present/apt/dry-run/script/install suites | EDIT exact symbols; frozen reuse helper untouched |
| R2-2F | `crates/shell/src/builtins/world_enable/runner.rs`; `runner/provision_deps.rs` | `run_enable_with_provision_deps`; `run_sync_after_provisioning`; `probe_world_manager`; `probe_requirements`; `provision_apt_requirements`; `provision_pacman_requirements`; `execute_with_profile` | A reaches runner then is dropped before probes/install/sync | same authenticated world-deps context | `run_enable` and provisioning helpers | world-enable provisioning | Unix carrier; existing non-Unix restrictions | LOW | PI-106/PI-107 | `world_enable.rs`; `world_enable_provision_deps_wdap0.rs` | EDIT |
| R2-2F | `crates/shell/src/execution/routing/dispatch/world_ops.rs`; `routing/dispatch/prelude.rs`; `routing.rs` | one new authenticated builder/private cfg implementations and export-only re-exports | existing shared builder resolves config/policy/network/socket ambient | prevalidated F context reusing E projection | only `run_world_command_for_deps_at` and `execute_with_profile` | world execution request construction | cfg implementations | new N/A; frozen ambient builder HIGH, trace variant CRITICAL | PI-106/PI-107 | colocated world-ops plus world-deps/provision suites | ADD plus export-only; ambient builders FROZEN |
| R2-2F | `crates/shell/src/builtins/shim_doctor/report.rs` | `gather_world_doctor_snapshot`; `gather_world_deps_section`; `status_for_world_deps_report`; `snapshot_from_value`; `snapshot_from_command`; shared identity validator | fixtures/children can omit/mismatch A; missing `ok` may become healthy | expected non-secret A prefix/commitment | `build_report` | Health/shim-doctor composition | Unix typed path; non-Unix R2-3 compatibility/unproven and cannot claim A | LOW | PI-106/PI-107; PI-108 mechanics preserved | `common.rs`; `shim_doctor.rs`; `shim_health.rs`; doctor suites | EDIT coherence only |

R2-2E's binding contract is exact:

1. The gateway consumes an already-authenticated `InstallBootstrapContextV1`; missing, malformed,
   tampered, mismatched, wrong-principal, or unavailable A fails before classification or effects.
2. Existing explicit-home config and policy resolvers select A. `policy_snapshot.rs` owns the only
   policy snapshot and network-policy projection; duplication inside `world_gateway.rs` is forbidden.
3. `SUBSTRATE_HOME`, `SUBSTRATE_ROOT`, CWD, account-home variables, XDG, manager projections,
   `CODEX_HOME`, `.codex`, `config.toml`, and platform-control roots never replace A. CWD may be an
   explicit workspace-scope input only. Runtime-native files remain projections.
4. Network allow/deny semantics and gateway lifecycle behavior are unchanged; only selection
   authority becomes exact. Credentials remain launch-time in-world handoff under the existing
   contract, and no durable host credential authority is created.
5. Linux uses the fixed socket. E has no authenticated macOS endpoint source, so macOS fails before
   client construction or ambient forwarding; Windows/other contextless entries fail before any
   ambient selector. Existing non-Linux clients remain frozen/unreachable from E. No
   `world-mac-lima` edit or native macOS/Windows claim is authorized.
6. Output may include only intended non-secret references/commitments. Carrier, prompt/request,
   credential/token, and sensitive-principal bytes are forbidden everywhere observable.

Its required tests cover custom A with conflicting B; A-only config, policy, network policy,
inventory/runtime-family, and Codex projection; environment-only/malformed/tampered input;
fail-before-mutation/forwarding/launch; disabled-routing validation; disclosure scans; unchanged
network allow/deny behavior; Linux fixed socket; macOS/Windows/other fail-before-ambient selection;
and non-Unix build/static cfg preservation without an A-bound/native proof claim.

R2-2E is now **implemented, proof-complete, review-clean, committed, and preserved** with this exact
evidence:

- commit `7e8e83802885c0ece93efcaacccc26503eeb6715`; tree
  `02a1a2f6b7e4be47ed9ec38537c805ae348c96b6`; ordinary patch SHA-256
  `fb7b65b02cdac46857750b64ebd5ceab651e8e99910c05240b187c3e729444ff`; full-index patch SHA-256
  `c712f467ac92efb0de7b524272479741ac6b318201cf45dbd994116ec0e8b862`; preservation branch
  `feat/preserve-a1-1d-5r2-2e-c712f467`;
- exact manifest: `crates/shell/src/execution/platform/mod.rs`,
  `crates/shell/src/builtins/world_gateway.rs`,
  `crates/shell/src/execution/agent_inventory.rs`,
  `crates/shell/src/execution/policy_snapshot.rs`, and `crates/shell/tests/world_gateway.rs`;
- changed ownership is confined to the Gateway arm's authenticated binding, the existing gateway
  request/projection call chain, additive explicit-bootstrap-home inventory and network projection,
  and its tests. The sole deleted symbol is
  `synthesized_unavailable_response_without_context`; canonical config/policy resolver ownership,
  network allow/deny meaning, and Routes A–D bytes are unchanged;
- focused results: gateway classification 13/13; config resolution 21/21; effective policy 14/14;
  policy snapshot/network 10/10; inventory 17/17; install-bootstrap context 8/8; explicit
  `HostSessionAuthority` composition 1/1; new integration negatives 3/3; managed auth bundle 7/7;
  world-service gateway runtime 32/32; gateway receiver/server 18/18;
- the **clean Route D comparison baseline** is `1101 passed / 149 failed / 0 ignored`; final E
  produced the genuine **post-E pre-F0 success observation**, `1114 passed / 149 failed / 0
  ignored`.
  `PassToFail=0`, `NewFail=0`, `FailToChangedFailure=0`, and `FailToPass=0`; the 149 failure-name set
  is identical, with only the recorded nondeterministic orchestration identifiers normalizing. The
  same runtime can instead produce the known interference observation `1113 passed / 150 failed /
  0 ignored`; neither pre-F0 result is an F comparison baseline;
- final GitNexus output is semantically contained to the approved owners and existing process
  families. Its aggregate CRITICAL adjacency is diagnostic over-attribution; each exact edited
  existing symbol was LOW, and no new process family, resolver owner, schema, or capability was
  introduced;
- final isolated read-only reviewers `e_final_gateway_authority`, `e_final_policy_network`,
  `e_final_credential_boundary`, and `e_final_platform_regression_replacement` returned CLEAN. The
  original platform reviewer is excluded because it violated the required read-only process;
- Linux fixed-socket and regression proof is complete. macOS fails before ambient client/forwarding
  selection, while Windows/other entries fail before ambient selection; static cfg preservation is
  recorded, but no native macOS/Windows or privileged R2-4 product proof is claimed.

PI-111's **R2-2E implementation/proof clause only** is complete. This does not close the full
gateway credential/config architecture: the managed-gateway secure-FD producer, receiver, bundle
schema, and lifecycle are landed, regression-proven, and unchanged by E, while direct-member
Codex/UAA gateway adoption remains unresolved, transitional compatibility, non-promotable, and
owned by E3/D1/D3. `RG-CONFIG-02`, `RG-CONFIG-04`, `RG-UAA-02`, and `RG-UAA-03` remain open and
unchanged. E changes no user/world capability, policy meaning, credential transport, or service
lifecycle and promotes no seam.

**A1.1d-5R2-2F0a — SUBSTRATE_HOME test isolation** joins R2-2F0 under this exact binding contract:

At this historical authorization checkpoint, F0/F0a were incomplete and test-harness-only. Their
exact pre- and post-fork-remediation candidates were preserved rather than landed. Focused proof was green (socket pair 100/100 parallel and 20/20 serial;
four-test neighbor matrix 20/20; prior absence, non-Unicode value, panic/reacquisition, nesting, and
concurrent exclusion all pass), but its final broad runs were `1118 passed / 150 failed` and `1119
passed / 149 failed`. The extra failure is
`dispatch_contract_adapter_active_task_resolution_requires_supervisor_claim`, normalized as
`resolve exact B-owned acceptance authority: open activated versioned authority layout`. It passes
alone. F0a remains an authorized part of the combined candidate, not a waiver of F0 proof. The
later exact combined candidate has now passed restoration, corrected differential proof, fresh
containment review, canonical closeout, and runtime preservation.

1. The exact HOME minimal pair is the target above plus
   `prompt_submit_continuity_prefers_persisted_session_contract`. In canonical same-process
   `--exact --test-threads=2` runs it failed 20/20. The stable same-process
   `--test-threads=1` harness controlled competitor-then-target and failed 0/10; separate-process
   sequential runs passed both directions. Stable libtest could not force reverse same-process
   order, so no reverse same-process result is claimed. Non-source tracing captured the competitor setting a private HOME,
   the target installing its own private HOME, then the competitor removing HOME before the target
   opens its authority layout. Three other unannotated HOME mutators independently reproduce 20/20.
   Commit `f5a150f94d585b1f55ec0067845cd5d715773c78` first adds the selected competitor.
   `83101dcbcc750e6e8fb8979bea19f1f777792188` later adds the target and its HOME-mutating fixture
   and is the first source commit where the exact pair coexists. This remains
   `TestIsolationDefectConfirmed`, not a production defect.
2. One `cfg(test)` RAII authority-environment guard acquires the existing shared reentrant lock and
   jointly protects HOME and world socket. It captures exact prior `OsString` or absence, installs
   the test value or absence, spans dependent socket/server/helper/child/process work and cleanup,
   restores on normal return and panic unwind, and releases only after restoration. At least 88
   tests depend jointly on HOME/socket, while existing helpers acquire them in opposite orders;
   separate locks are forbidden because they admit mixed snapshots and lock-order inversion.
3. Same-thread nesting must restore in stack order without deadlock. Poison/non-poison behavior and
   later reacquisition are explicit; no panic may strand a value or silently bypass the lock.
   `#[serial]` is supplemental only. Stable readers participate whenever source evidence requires
   one authority snapshot. Child environment inheritance is intentional and bounded; integration
   binaries use their own process-local disposition and no cross-process lock.
4. Source closure identifies 435 shell-library mutating test functions across 19 files. Five
   `with_store` families cover 223 dependent tests; two agents-command helpers cover five. The four
   unannotated mutators are the two control continuity and two agents-command toolbox-status tests.
   The table above is the complete test-only allowlist. Production-only non-Unix world-enable HOME
   mutation remains frozen; any test invoking it must hold the test boundary externally.
5. Nine named F0 orchestrator fixtures own an async server and guarded socket but abort without
   awaiting termination. They must execute `abort` → await confirmed task termination → complete
   and confirm fixture-owned socket/task cleanup → restore environment → release lock. Reverse
   declaration/drop order and `yield_now` do not prove termination. No production server lifecycle,
   readiness semantic, retry, or world-service behavior changes.
6. Combined focused proof reruns the F0 socket pair at least 100 parallel/20 serial and the F0a HOME
   pair at least 100 parallel/20 serial, then a mixed HOME/socket neighbor matrix. It proves exact
   prior value/absence/non-Unicode restoration, panic recovery, poison behavior, nesting, concurrent
   exclusion, bounded child inheritance, async cleanup ordering, and zero leaked task/socket/helper/
   temp-root state. The deliberate retained-registration conflicting child remains internal while
   its parent passes; retry validation is never weakened.
7. Combined F0/F0a/F0b broad proof is at least three independent exact final-candidate default-parallel shell
   walls and one canonical single-thread shell wall. Failure names and normalized signatures are
   identical; no test is removed, renamed, substituted, weakened, ignored, or made
   environment-authoritative; no unexplained count variance is accepted. Added passing tests have
   an explicit count delta. Only combined F0/F0a/F0b closeout records the deterministic F
   comparison baseline.
8. Production `std::env` readers, HOME/world-socket resolution, readiness, retries, policies,
   capabilities, world-service behavior, retained-worker state, credential transport, and the
   managed secure-FD path are frozen. F0/F0a alter no production byte. A required production change
   stops as `CrossDocumentChangeRequired`.

**A1.1d-5R2-2F0b — deterministic renderer-output test isolation** is bound by this exact
test-infrastructure contract:

1. Primary classification is `TestIsolationDefectConfirmed`. The exact post-fork parallel walls
   were `1134 passed / 146 failed / 0 ignored`, `1134 passed / 146 failed / 0 ignored`, and
   `1133 passed / 147 failed / 0 ignored`. Only wall 3 added
   `public_prompt_renderer_renders_bounded_structured_fallback_when_decode_fails`, so canonical
   closeout is invalid. The helper redirects process fd 1 with `dup2`; libtest's parallel reporter
   wrote through the same descriptor. Exact captured bytes were
   `".[codex] task_progress: fields=alpha, beta, gamma (+1 more)\n"`.
2. Causal stress is binding: forced same-process parallel passed 376 and failed 124 of 500 with one
   normalized signature; isolated, identical-neighbor same-process serial, and separate-process
   controls each passed 100/100; parallel pretty reporting passed 99/100. Candidate introduction
   is unnecessary because the capture helper and target test are byte-identical to clean E. This
   is neither random flakiness nor a production renderer defect.
3. Future implementation may edit only
   `crates/shell/src/execution/agent_runtime/control.rs`. It may mechanically delegate
   `PublicPromptRenderer::render` to a new private Unix-only explicit-writer core or equivalent
   private sink adapter, delete `capture_stdout_once` and `capture_stderr_once`, and migrate only
   `public_prompt_renderer_renders_bounded_structured_fallback_when_decode_fails` and
   `public_prompt_renderer_renders_bounded_structured_stderr_fallback_when_decode_fails`. No other file or symbol
   is authorized.
4. `PublicPromptRenderer::render` remains the production entry point with its signature frozen.
   `PublicPromptRenderer::new` and the bodies of
   `run_hidden_owner_helper_startup_prompt_stream_with_projection` and
   `run_public_prompt_command` remain frozen. Production delegates through real stdout/stderr,
   chooses and locks only the selected stream at the same point, and preserves JSON-envelope
   stdout, completed/normal-event stdout, warning/failure/stderr-event stderr, byte ordering,
   newlines, flushes, serialization/write error propagation or intentional suppression, redaction,
   and bounded fallback exactly.
5. Tests supply distinct in-memory stdout and stderr writers and assert the complete exact bytes
   emitted to the selected buffer plus an empty nonselected buffer. They may not strip, search
   around, or tolerate unrelated prefixes. The same-owner stderr helper is structurally unsafe and
   is migrated even though stdout alone appeared in the broad wall. Raw fd replacement remains
   forbidden as the final mechanism.
6. No public output API or transport/schema, process-global output lock, writer registry, side
   table, environment-selected sink, eager dual-stream lock, reporter suppression, sleep, retry,
   larger timeout, thread reduction, whole-suite serialization, ignore, removal, rename,
   substitution, or assertion weakening may satisfy F0b. `#[serial]` may remain supplemental but
   cannot exclude libtest reporter writes.
7. GitNexus reports `PublicPromptRenderer::render` MEDIUM (four direct, 39 total, one
   `handle_agent_command` process family; Agent_runtime direct and Execution indirect).
   `PublicPromptRenderer::new` reports HIGH (19 direct, 43 total, two process labels, three
   modules), but exact source closure proves generic-`new` over-attribution and freezes its body.
   The renderer type, capture helpers, and tests report LOW/zero-process. No CRITICAL impact, new
   production process family, or changed production process semantics are authorized.
8. Required proof includes both exact tests isolated; at least 100/100 exact renderer-pair parallel
   and 20/20 serial; forced reporter overlap; exact private stdout/stderr bytes; unchanged
   JSON/completed/warning/failure/event selection, order, newline, flush, and error behavior; then
   every combined F0/F0a/F0b focused, compile, lint, format, and four-wall gate. Production,
   user-facing, world, policy, credential, secure-FD, gateway, receipt, supervisor, worker,
   placement, caging, lifecycle, and capability behavior remains unchanged. No seam is promoted.

The following is the historical pre-F5-PD R2-2F binding contract; the later historical F5-PD
contract, whose boundary the completed-F gate record retains, superseded its immediate-next and
nested-doctor side-effect statements:

At that checkpoint F was the **exact next increment** after the then-complete corrected
differential proof, fresh containment review, and harness closeout, but it was not active or
implemented. It starts from the post-closeout replayed runtime commit and tree recorded by the
dedicated preservation ref and completion checkpoint. Routes A–E and F0/F0a/F0b are immutable prior
evidence. Its clean comparison baseline is the deterministic value recorded by combined closeout,
not either genuine but nondeterministic post-E observation (`1114 passed / 149 failed / 0 ignored`
or `1113 passed / 150 failed / 0 ignored`), either F0-candidate broad result (`1118/150` or
`1119/149`), any post-fork candidate wall (`1134/146`, `1134/146`, or `1133/147`), the
**R2-2 historical starting baseline** (`1089 passed / 149 failed`), or the
**clean Route D comparison baseline** (`1101 passed / 149 failed / 0 ignored`). A renewed
production-fix-free Routes A–F
integration closeout follows F; R2-3, R2-4, and R3 remain after that and are unstarted.

1. Normal current, global, workspace, runtime, provision-deps, and post-sync paths receive the one
   shared authenticated context. A selects global config/inventory/dependency roots; explicit launch
   CWD selects workspace scope only. No normal product path re-enters ambient resolution.
2. The additive authenticated request builder is consumed by exactly two leaves and reuses E's
   projection. The existing HIGH/CRITICAL ambient builders, `resolve_current_inventory_view`,
   resolver bodies, schemas, services, lower adapters, and compatibility callers remain frozen.
3. World-deps mutation/readiness/execution behavior is unchanged except for selecting the correct
   A root. Validation precedes every read that can influence mutation and every effect. Only the
   F5-PD nested World Doctor child is guaranteed non-mutating.
4. On the authenticated Unix/Linux route, doctor may identify A only when configuration,
   policy/world-fs policy, inventory, dependency,
   runtime observation, fixture, and child evidence all match A. Missing `ok` or identity, mixed
   A/B, or mismatch is unavailable/incoherent with `ok=false`, never success. Rejected reports are
   not retained as healthy payloads.
5. macOS, Windows, fallback, and other non-Unix world-deps/Health/doctor routes remain explicit
   R2-3 compatibility/unproven paths. They preserve labeled ambient compatibility or report
   unavailable/fail closed, but never claim A-bound success or consume the F context.
6. Missing/malformed/tampered/mismatched context fails closed. No carrier, credential, request,
   prompt, token, or sensitive-principal material appears in output, errors, fixtures, logs, or
   traces. Filesystem, network, caging, placement, service, receipt, supervisor, retained-worker,
   and lifecycle capabilities do not change.

Its required tests cover conflicting A/B for current/global/workspace/runtime; mutation and
nonmutation; current applied/show probes; apt/pacman/runtime installs; manager probes and
post-provision sync; direct no-witness and tamper failures before mutation; exact fixture/child
identity; mixed-source rejection; truthful unavailable output; existing A-rooted snapshot behavior;
explicit compatibility; disclosure scans; and non-Unix build/static preservation with no A-bound
success claim. The broad comparison is against the deterministic F baseline recorded by combined
F0/F0a/F0b closeout and requires
zero pass-to-fail, new-fail, changed-failure, removed, renamed, substituted, weakened, or newly
ignored tests plus the identical retained 149-name failure set.

The binding stop conditions for E, F0, F0a, F0b, and F are: any file/symbol outside `03`'s exact allowlist;
any new resolver or side table; any edit to a frozen HIGH/CRITICAL builder; any change to another
shared caller, network meaning, request schema, service, lower platform adapter, capability,
credential lifecycle, cleanup, or deletion; any mutation before validation; any mixed-authority
healthy diagnostic; any non-Linux compatibility path presented as E/F-authenticated; or any
privileged/native platform proof claim without evidence. Encountering
one stops as `ImpactDecisionRequired` or `CrossDocumentChangeRequired`; it never silently widens an
increment.

For this exact Route A closure, the immutable production baseline is fixed at patch SHA-256
`ea4abf43043013994e698d54f21c04bba58833b3bb3247bf6d5d3b454f51b943`. Its manifest and
file SHA-256 fingerprints are:

| File | SHA-256 |
|---|---|
| `crates/shell/src/builtins/health.rs` | `997f138b72dce898cc707c128bf726e0f882da7f3a0c1cdcc3ff71e9b94e7405` |
| `crates/shell/src/builtins/shim_doctor/mod.rs` | `3035df6e3bca693c5e8c10e5f5cfee3fd1e1a6ad57d40c443979d25282bda748` |
| `crates/shell/src/builtins/shim_doctor/report.rs` | `0337379b8c4f1ce5169a60eb099b4d66cf7e9dfc32cc9f314ca4194ddd6f0b60` |
| `crates/shell/src/execution/config_cmd.rs` | `ef102e015759a784c8601a9cc4a8d2c220c467679f3eac111bb87f527eca12c6` |
| `crates/shell/src/execution/config_model.rs` | `42b7e851f7c3a63f3c739295bd12bacca1e51f4ecf9fb7037e919a3ea1985a07` |
| `crates/shell/src/execution/invocation/plan.rs` | `cc8a9d52db3bad5d571f64012ca37ec2b6219757b47d1802e91cae804c730c01` |
| `crates/shell/src/execution/platform/mod.rs` | `b93601f17f120bc3c28b544eb569b671750224b741d893ed0eb2d02252f04274` |
| `crates/shell/src/execution/policy_cmd.rs` | `a9f90acf883b264d0b76bbda4ca5dc9b5fdcf42f0d3119c49843981086a25d7b` |
| `crates/shell/src/execution/policy_model.rs` | `a04940b54c68d284dcc1ed65245180676b2822dbe596386e6bbba544507f2931` |
| `crates/shell/tests/config_show.rs` | `7a9819b02b507c35ef289a492cbd9c9b3e75ba462644e02914b1959b4de0f444` |
| `crates/shell/tests/policy_discovery.rs` | `7bad42c95925c1fb8f74c277e37710c8c8d69d2e866f7d4eb4f34a65df6226d1` |

The review-remediation successor preserves every production hunk above byte-for-byte. It may differ
from the base only by the following test delta:

| File/symbol | Sole successor-only change | Review finding |
|---|---|---|
| `config_model.rs::tests::explicit_bootstrap_home_explain_uses_selected_global_config_path` | Add item-level `#[cfg(unix)]` to the test and its sole test-only `HostSessionAuthority` import; preserve the name, body, assertions, fixtures, every other import, module cfg, and all production bytes. | Unix-only `HostSessionAuthority::open` must not make the test fail on unsupported non-Unix authority-store implementations, and the gated import must not trigger warnings-denied unused-import failure. |
| `policy_model.rs::tests::explicit_bootstrap_home_explain_uses_selected_global_policy_path` | Add item-level `#[cfg(unix)]` to the test, its sole test-only `HostSessionAuthority` import, and the test module's sole `tempfile::TempDir` import; preserve the name, body, assertions, fixtures, every other import, module cfg, and all production bytes. | Same cross-platform cfg/import correction for the policy test, including the warnings-denied import made unused by the item-level test gate. |
| `crates/shell/tests/shim_health.rs` focused Route A test | Add one focused regression or bounded strengthening that selects A through typed IH while H/R/HOME name conflicting B, proves A-derived Health state, uses syscall/path interception that fails on any B operation, compares a complete B-tree snapshot before/after, preserves classification/output assertions, and cannot be satisfied by `collect_report`. | Existing Health proof did not conflict ambient B or independently exclude the compatibility collector; mode bits and selected-file assertions alone do not prove absence of metadata probes or writes elsewhere under B. |
| `crates/shell/tests/doctor_scopes_ds0.rs` focused Host test | Add one focused regression or bounded strengthening using the canonical R2-1 installed-product invocation witness for A, no explicit selector, conflicting ambient B, A-derived Host/config output and expected dependency scaffold, absence of B dependency artifacts, complete B-tree preservation across the installed-witness dispatch, and complete B-tree preservation across the direct-repository no-witness fail-before-mutation case. | Existing Host test omitted both selector and witness and therefore never reached the changed typed-IH route; selected config bytes alone do not prove dependency routing or complete fail-before-mutation behavior. |
| `config_show.rs::config_current_show_uses_declared_prefix_under_conflicting_ambient_home` | Replace only the `/run/user/<effective uid>` fallback used to parent its unique selected-A root: use nonempty `XDG_RUNTIME_DIR`; otherwise resolve the current account's nonempty home and use `.cache` or a repository-established fixture subdirectory beneath that same account home; create the parent as needed; keep the selected root `0700`; and fail setup explicitly if neither source exists. The account home is current-account-derived, not the command's ambient/conflicting HOME. Preserve the name, cfg, command, A/B fixtures, config bytes, assertions, output, and production route. | The existing Unix test-parent fallback is Linux-specific and fails on macOS when `XDG_RUNTIME_DIR` is absent; a portable owner-controlled parent is required without weakening conflicting-B authority proof. |
| `policy_discovery.rs::policy_current_show_uses_declared_prefix_under_conflicting_ambient_home` | Apply the identical bounded secure-parent replacement while preserving the name, cfg, command, A/B fixtures, policy bytes, assertions, output, and production route. | The same Linux-only fallback makes the policy proof nonportable even though its Route A authority assertions are otherwise valid. |

The preserved current successor before the final test-parent correction is exact patch SHA-256
`0a53e8b60326e21b4237392bfa99671fa8c19c0cd45eb4c573cae5140e808720` at preservation commit
`ad139789ef317b978e96c36ee8170dd503bff8a4`, with exactly thirteen files. The two portable-parent
tests already belong to that manifest, so the corrected candidate remains exactly thirteen files;
all production fingerprints and all other test fingerprints remain unchanged. Its new patch hash is
computed only after those two authorized hunks are applied. Before each existing test symbol is
edited, run fresh upstream impact analysis; the recorded impacts for both parent-selection tests are
LOW with zero callers, processes, or modules. After implementation, create a deterministic
base-to-successor delta containing the new ordinary/binary patch SHA-256, exact manifest and
fingerprints, and a hunk table mapping every successor-only line to one row above. Existing tests
cannot be removed, renamed, substituted, ignored, or weakened; the two tests remain active on Unix,
and only their intentional non-Unix cfg exclusion is authorized. Any changed production byte, changed
base hunk outside the item-level cfg attributes and the two exact portable-parent setup hunks, or
successor-only hunk that does not map exactly to one of the six authorized delta rows is
`SuccessorPatchScopeMismatch`.

For both final parent-selection hunks, empty environment values are unavailable rather than paths.
The allowed precedence is nonempty `XDG_RUNTIME_DIR`, then the current account's nonempty home under
`.cache` or a repository-established fixture subdirectory beneath that same account home. The latter
is not a third source, and the command's ambient/conflicting HOME does not select it. The test creates
that parent if needed and creates its unique selected-A root beneath it with owner-only `0700`. It
fails with an explicit setup error when neither source is usable. It never falls back to `/tmp`, `/var/tmp`,
`/run/user/<uid>`, CWD, ambient `SUBSTRATE_HOME`, or B. This changes fixture placement only: explicit
A still wins, B remains conflicting and unread, and the typed Config/Policy production route and all
assertions remain identical.

GitNexus observations over the identical or mechanically extended Route A bytes are diagnostic
provenance: 33 symbols/18 process labels; 11/18 in the reverse committed comparison; 37/18; 39/18;
and, after replay and a current-index refresh, CRITICAL with 24 attributed symbols and 31 labels.
The incompatible raw counts and generated label sets are not stable semantic authority for this
exact patch. Do not pin or restore a stale index.

Base containment is determined in this order: exact patch bytes; the exact manifest and fingerprints
above; the exact source-level production functions and permitted tests. Successor containment then
requires byte-identical production hunks plus the deterministic six-row test delta above; manual call-path mapping to
the approved `ShellConfig::from_cli` Host/Health/Config/Policy/context-aware shim-doctor roots; no
new source module, authority source, call path, execution-family root, side table, environment
fallback, or lifecycle behavior; and fresh independent semantic-diff review. The exact production
diff is limited to `ShellConfig::from_cli`, `handle_host_command`, `handle_health_command`,
`health::run`, the crate-private shim-doctor exports, the item-level `collect_report` annotation,
Config's handler/current-show and two explicit-bootstrap-home resolver functions, and Policy's
handler/current-show and two explicit-bootstrap-home resolver functions. Permitted test changes are
the mechanical Config test carrier, the two explicit-home explain tests, the two focused integration
tests, and the secure-parent setup only in the named `config_show.rs` and `policy_discovery.rs` tests
already in the manifest. No other production function changes behavior.

Every fresh GitNexus result must be retained and every label mapped to one of
`AuthorizedRouteA`, `LineOrHunkAttributionOnly`, `TestColocationOnly`, or
`UnexpectedSemanticPath`. Count/label drift alone does not reopen the docs when the base production
bytes, successor test delta, fingerprints, source-level diff, mapping, module set, and semantic roots
remain exact. The six review-remediation edits restore ordinary pre-edit impact and fresh
change-detection requirements but need no new raw-count ceiling. Any `UnexpectedSemanticPath`,
production-byte difference, unclassified test difference, new module or semantic execution root,
authority source, or behavior expansion is an `ImpactDecisionRequired` stop. This rule is specific
to the exact Route A base plus its bounded test-only successor and cannot
apply to Route B–D, R2-3, another HIGH/CRITICAL increment, environment/global authority,
physical-shim migration, product capability, output, repair, cleanup, rollback, or lifecycle
behavior.

The current refreshed 31-label attribution audit is:

| Current label | Root symbol/module | Route A family | Source diff touches behavior? | Classification |
|---|---|---|---|---|
| `Handle_host_command → AuthorityFacadeError` | `platform::handle_host_command` | Host | Yes; typed IH selects the opened global-config root and this trace follows the new explicit-home resolver. | `AuthorizedRouteA` |
| `Handle_host_command → New` | `platform::handle_host_command` | Host | Yes; typed IH selects the opened global-config root and this trace follows the new explicit-home resolver. | `AuthorizedRouteA` |
| `Run → Lookup_unix_account_by_uid` | `health::run` | Health/shim-doctor | Yes; Unix Health now calls the typed context-aware collector and validates the committed principal. | `AuthorizedRouteA` |
| `Run → As_path` | `health::run` | Health/shim-doctor | Yes at the typed collector root; the reported configuration leaf is unchanged. | `AuthorizedRouteA` |
| `Run → WorldDisableAttribution` | `health::run` | Health/shim-doctor | Yes at the typed collector root; diagnostic classification is unchanged. | `AuthorizedRouteA` |
| `Run → WorldDisableSource` | `health::run` | Health/shim-doctor | Yes at the typed collector root; diagnostic classification is unchanged. | `AuthorizedRouteA` |
| `Run → As_str` | `health::run` | Health/shim-doctor | Yes at the typed collector root; the reported formatting leaf is unchanged. | `AuthorizedRouteA` |
| `Run → Validate` | `health::run` | Health/shim-doctor | No; GitNexus joins the Unix typed Health root to the cfg-incompatible compatibility collector. | `LineOrHunkAttributionOnly` |
| `Run → Current_dir` | `health::run` | Health/shim-doctor | No; the generated trace enters the compatibility collector before the typed collector, which typed Unix Health cannot do. | `LineOrHunkAttributionOnly` |
| `Run → CliConfigOverrides` | `health::run` | Health/shim-doctor | No; the generated trace enters the compatibility collector before the typed collector, which typed Unix Health cannot do. | `LineOrHunkAttributionOnly` |
| `Handle_config_command → ActionableError` | `config_cmd::handle_config_command` | Config | No; the trace is the unchanged workspace-set leaf attributed through the handler signature hunk. | `LineOrHunkAttributionOnly` |
| `Handle_config_command → New` | `config_cmd::handle_config_command` | Config | No; the trace is the unchanged workspace-set parse-error leaf attributed through the handler signature hunk. | `LineOrHunkAttributionOnly` |
| `Handle_config_command → Parent` | `config_cmd::handle_config_command` | Config | No; the trace is the unchanged global-init write leaf. | `LineOrHunkAttributionOnly` |
| `Handle_config_command → Write_all` | `config_cmd::handle_config_command` | Config | No; the trace is the unchanged global-init write leaf. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → As_str` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → Workspace_legacy_settings_path` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → For_path` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → Resolve_replace` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → ConfigExplainKey` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → I64ClampInfo` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → ConfigExplainSource` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |
| `Handle_config_command → Exists` | `config_cmd::handle_config_command` | Config | No; the trace is the unchanged global-init existence leaf. | `LineOrHunkAttributionOnly` |
| `Handle_config_command → Print_patch` | `config_cmd::handle_config_command` | Config | No; the trace is the unchanged global-show leaf, not current-show. | `LineOrHunkAttributionOnly` |
| `Handle_policy_command → Parent` | `policy_cmd::handle_policy_command` | Policy | No; the trace is the unchanged workspace-init write leaf. | `LineOrHunkAttributionOnly` |
| `Handle_policy_command → Write_all` | `policy_cmd::handle_policy_command` | Policy | No; the trace is the unchanged workspace-init write leaf. | `LineOrHunkAttributionOnly` |
| `Handle_policy_command → As_bytes` | `policy_cmd::handle_policy_command` | Policy | No; the trace is the unchanged workspace-init hashing/write leaf. | `LineOrHunkAttributionOnly` |
| `Handle_policy_command → Exists` | `policy_cmd::handle_policy_command` | Policy | No; the trace is the unchanged global-init existence leaf. | `LineOrHunkAttributionOnly` |
| `Handle_policy_command → Invalidate_policy_snapshot_cache` | `policy_cmd::handle_policy_command` | Policy | No; the trace is the unchanged global-init cache invalidation leaf. | `LineOrHunkAttributionOnly` |
| `Handle_policy_command → Print_patch` | `policy_cmd::handle_policy_command` | Policy | No; the trace is the unchanged global-show leaf, not current-show. | `LineOrHunkAttributionOnly` |
| `Handle_policy_command → Current_dir` | `policy_cmd::handle_policy_command` | Policy | No; the trace is the unchanged workspace-show leaf, not current-show. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → As_path` | `platform::handle_host_command` | Host | No on Unix; the trace follows the cfg-incompatible legacy resolver retained for non-Unix. | `LineOrHunkAttributionOnly` |

The three labels recorded in the prior 18-label output but absent after refresh are also mapped:

| Historical label now absent | Root/family | Source-level account | Classification |
|---|---|---|---|
| `Run → Encode` | `health::run`; Health/shim-doctor | No Route A source hunk encodes a carrier; this was generated attribution breadth. | `LineOrHunkAttributionOnly` |
| `Handle_host_command → WorldDisableAttribution` | `platform::handle_host_command`; Host | Typed IH changes selected config input; the diagnostic classification leaf remains unchanged. | `AuthorizedRouteA` |
| `Handle_host_command → WorldDisableSource` | `platform::handle_host_command`; Host | Typed IH changes selected config input; the diagnostic classification leaf remains unchanged. | `AuthorizedRouteA` |
## A1.1d-5R2-2F0-HC corrected complete process-resource ledger

This is the durable closure inventory for the process running the shell-library wall under the
[canonical broad-wall invocation contract](../a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#canonical-shell-library-broad-wall-invocation-contract). It was
formed by two independent methods: lexical scanning of all `crates/shell/src` test modules found
1,303 source test functions, while the same-file helper/call parser closed 5,070 functions and
initially recognized 1,302 tests before the independent scan restored the multiline-attribute
`execution/lock.rs::test_concurrent_lock_attempts`; and symbol, caller, reader, and execution-flow
inspection used refreshed GitNexus plus source closure where the graph under-resolved `cfg(test)`
code. The first parser found 506 environment-dependent tests in 31 files; the published audit then
manually added twelve disjoint dynamic-wrapper tests to claim 518 in 35 files. Complete lexical and
resolved-call closure instead finds 534 parent-mutating tests in the same 35 files: eighteen real
mutators were absent, while two B1 acknowledgement tests were false positives from ambiguous
same-file bare-name fanout. The exact additions and removals are enumerated in `02` and `05`. A2 is
therefore 129 direct tests in 22 files—116 additive plus 13 F0a—not the earlier 113 or intermediate
116. The Linux test binary discovered 1,263 runnable tests. Known integration parent-mutators were
inspected only for helper and inherited-child contracts.

The corrected environment closure found 86 exact names (some are only projected into a child or
read without parent mutation):
`ANTHROPIC_API_KEY`, `API_TOKEN`, `BASH_ENV`, `CODEX_HOME`, `COLUMNS`, `EXPORT_COMPLEX`, `HOME`,
`LIMA_VM_NAME`, `LINES`, `OLDPWD`, `OPENAI_API_KEY`, `PATH`, `PLAIN_VALUE`, `PWD`, `SHIM_ACTIVE`,
`SHIM_ORIGINAL_PATH`, `SHIM_PARENT_SPAN`, `SHIM_SESSION_ID`, `SHIM_TRACE_LOG`,
`SUBSTRATE_A1_REPLACEMENT_CHILD_ROOT`, `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT`,
`SUBSTRATE_AGENT_TOOLBOX_VERSION`, `SUBSTRATE_ANCHOR_MODE`, `SUBSTRATE_ANCHOR_PATH`,
`SUBSTRATE_CAGED`, `SUBSTRATE_COMMAND_SUCCESS_EVENTS`, `SUBSTRATE_DISABLE_PTY`,
`SUBSTRATE_ENABLE_PREEXEC`, `SUBSTRATE_FORCE_PTY`, `SUBSTRATE_HOME`,
`SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1`, `SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT`,
`SUBSTRATE_INSTALL_PRIMARY_UID`, `SUBSTRATE_INSTALL_PRIMARY_USER`,
`SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME`,
`SUBSTRATE_LIMA_VM_NAME`,
`SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`,
`SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID`, `SUBSTRATE_MANAGER_ENV`,
`SUBSTRATE_MANAGER_ENV_ACTIVE`, `SUBSTRATE_MANAGER_INIT`, `SUBSTRATE_MANAGER_INIT_DEBUG`,
`SUBSTRATE_MANAGER_INIT_POWERSHELL`, `SUBSTRATE_MANAGER_MANIFEST`, `SUBSTRATE_NO_SHIMS`,
`SUBSTRATE_ORIGINAL_BASH_ENV`, `SUBSTRATE_OVERRIDE_ANCHOR_MODE`,
`SUBSTRATE_OVERRIDE_ANCHOR_PATH`, `SUBSTRATE_OVERRIDE_CAGED`, `SUBSTRATE_OVERRIDE_WORLD`,
`SUBSTRATE_PARENT_SPAN_ID`,
`SUBSTRATE_POLICY_MODE`, `SUBSTRATE_PTY_PIPELINE_LAST`,
`SUBSTRATE_R0_RETAINED_SUBPROCESS_VARIANT`, `SUBSTRATE_REPLAY_USE_WORLD`,
`SUBSTRATE_REPLAY_VERBOSE`, `SUBSTRATE_ROOT`, `SUBSTRATE_SHELL`, `SUBSTRATE_SKIP_MANAGER_INIT`,
`SUBSTRATE_SKIP_MANAGER_INIT_LIST`, `SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE`,
`SUBSTRATE_SYSTEMCTL_TIMEOUT_MS`, `SUBSTRATE_TEST_LOCAL_WORLD_ID`,
`SUBSTRATE_TEST_SHARED_WORLD_METADATA_ROOT`, `SUBSTRATE_WORLD`,
`SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR`, `SUBSTRATE_WORLD_DEPS_SKIP_APT`,
`SUBSTRATE_WORLD_DEPS_SKIP_PACMAN`, `SUBSTRATE_WORLD_ENABLED`,
`SUBSTRATE_WORLD_FAIL_CLOSED_ROUTING`, `SUBSTRATE_WORLD_FS_ENFORCEMENT_PLAN_B64`,
`SUBSTRATE_WORLD_FS_ISOLATION`, `SUBSTRATE_WORLD_FS_MODE`, `SUBSTRATE_WORLD_ID`,
`SUBSTRATE_WORLD_NET_FILTER`, `SUBSTRATE_WORLD_PROJECT_DIR`,
`SUBSTRATE_WORLD_REQUEST_PROFILE`, `SUBSTRATE_WORLD_REQUIRE_WORLD`,
`SUBSTRATE_WORLD_SOCKET`, `TEST_ENV_KEY`, `TEST_MODE`, `UNSET_ME`, `USERPROFILE`,
`XDG_CONFIG_HOME`, `XDG_DATA_HOME`, and `XDG_STATE_HOME`.

Exactly 74 of those names are parent-mutated. The 12 names that are child-only projections or
read-only in this bounded source are `ANTHROPIC_API_KEY`, `BASH_ENV`,
`SUBSTRATE_A1_REPLACEMENT_CHILD_ROOT`, `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT`,
`SUBSTRATE_AGENT_TOOLBOX_VERSION`, `SUBSTRATE_ENABLE_PREEXEC`,
`SUBSTRATE_MANAGER_ENV_ACTIVE`, `SUBSTRATE_ORIGINAL_BASH_ENV`,
`SUBSTRATE_PARENT_SPAN_ID`, `SUBSTRATE_R0_RETAINED_SUBPROCESS_VARIANT`,
`SUBSTRATE_WORLD_FS_ENFORCEMENT_PLAN_B64`, and `SUBSTRATE_WORLD_PROJECT_DIR`. The other 74,
including all three XDG names and `SUBSTRATE_SHELL`, are parent mutations. `SUBSTRATE_ROOT` is mutated by the
wrapper/F0a families enumerated in `02` and belongs to A2. Environment values, credentials, and
complete paths were not emitted by diagnostic probes.

The prior 80-name claim is withdrawn. Its test-reachability pass recognized wrapper bodies, but
its exact-name phase scanned direct primitive arguments and did not resolve every wrapper
argument/callsite. The manual dynamic-wrapper repair kept the two settings tests without extracting
their `SUBSTRATE_OVERRIDE_*` arguments and omitted the gateway `AmbientSelectionGuard` call. This
was not a child-only XDG projection and was not stale-source drift: immutable source invokes
`std::env::{set_var,remove_var}` in the parent for all three XDG entries. The corrected audit
reconciles 395 direct primitive invocations, including 73 dynamic-key invocations, with 43 dynamic
mutation-sink owners, 1,005 resolved mutation callsites, and 534 exact mutating tests, and reports
zero unclassified dynamic rows. Independent correction review rejected both the earlier 405-row
and later 604-row evidence because they omitted cross-file or duplicate-name calls through
`with_store`, `ProjectionEnvGuard::capture`, `install_bootstrap_projections`,
`apply_world_root_env`, `export_runtime_config_env`, `update_world_env`, and the macOS `restore`
family. Its prevention rule is binding: any absent
wrapper callsite, unresolved literal/constant/table/parameterized name, silently dropped dynamic
name, parent-as-child classification, or mutating test missing from the migration manifest fails
the gate. Full callsite closure additionally proves `SUBSTRATE_SHELL` is mutated by
`routing/test_utils.rs::{set_env,restore_env}` in
`async_repl_host_commands_record_replay_context`; `crates/trace/src/span.rs::SpanBuilder::new` is
the overlapping stable reader. Three early returns precede the manual restore, and `set_env`
captures `Option<String>`, so current restoration is neither early-return/unwind safe nor exact for
non-Unicode values. The test is already in the A2 116-test manifest and 35-file union. The two
macOS platform tests restore only `SUBSTRATE_WORLD` and `SUBSTRATE_WORLD_ENABLED` as
`String`/absence even though `update_world_env` changes six names; their future test-only helper
must restore all six as exact `OsString`/absence under the same coordinator.

The evidence validator freezes the normalized 395-call primitive manifest at SHA-256
`df0254488f37d4f53f05c81bbbd47c05bbbe410496d465827daade80922e0f4a`, the 1,005-call resolved
mutation manifest at `350ccd1876ecc07cced81df89b523e3d24f9674f5dc6a5e33f052d65f4489772`, and the 534-test
manifest at `de7e2bc43c0fcab74096dc272340726c919aeb72004fd0208ab5b4dffb802f4a`. It also freezes 43
dynamic mutation-sink owners at SHA-256
`133e28a495e5be69016fb7f6483018cfb406b44c8ca5e2d8779fdf82ded0c66c`, source-parses fixed
projection and local restore-key tables, and runs negative perturbation checks. The rejected
604-row intermediate remains historical evidence only. The validator parses the canonical
35-file, 116-additive-test, and 13-F0a-test lists, checks 129 unique source symbols, and compares
their exact manifest hashes. A duplicate removal, changed argument, missing callsite, swapped path,
or missing test therefore fails even when the set of environment names would remain unchanged.

### Identity and use closure

“Direct tests” counts the source-closed tests that directly use, or reach a same-file helper using,
the resource. Counts overlap between rows and are not test totals. `All` means the single
shell-library process; `child` means an intentionally inherited helper process.

| ID | Category and exact resource | Mutating symbols | Reading symbols / helper family | Direct tests | Scope / cfg | Current guard or isolation | Known evidence / competing tests | GitNexus impact |
|---|---|---|---|---:|---|---|---|---|
| A1 | Environment: `HOME`, `SUBSTRATE_HOME`, `SUBSTRATE_WORLD_SOCKET` | `std::env::{set_var,remove_var}`, `EnvVarGuard`, `with_store`, `with_env_var`, manual save/restore | authority/store/socket/path resolvers; F0/F0a helper families | 456 / 20 files | All; Unix plus cfg-specific readers | reentrant `WORLD_ENV_LOCK`, local guards, `#[serial]` islands | HOME and socket races proven; 11 unannotated dependents; mixed snapshots possible | `with_test_mode` HIGH; many test symbols under-resolved |
| A2 | Other ambient selectors, including `SUBSTRATE_ROOT`, `SUBSTRATE_SHELL`, the three XDG roots, and three `SUBSTRATE_OVERRIDE_*` selectors: the remaining directly or dynamically mutated names in the 86-name closure | same APIs plus `set_env`/`restore_env`, `ProcessStateGuard::set`, `EnvGuard::apply`, `ProjectionEnvGuard`, `AmbientSelectionGuard::{set,drop}`, `platform_tests::{snapshot,restore}`, and file-local wrappers | PTY, world, shim, manager, trace, policy, profile, install, terminal-size, gateway authority, settings, platform/world-policy, and `SpanBuilder::new` readers | 129 / 22 files: 116 additive plus 13 existing-source F0a | All; cfg varies | local guards and `#[serial]`; 27 additive unannotated | `SUBSTRATE_FORCE_PTY` stable-reader race proven; XDG/settings callsites, the early-return-unsafe host-replay mutation, and incomplete six-name macOS restoration now source-closed; any concurrent ambient mutator competes | `world_env_guard` CRITICAL; `EnvGuard::new` MEDIUM; macOS snapshot/restore LOW with two direct test callers and zero processes; gateway/routing test helpers LOW/under-resolved; production readers frozen |
| A3 | The read-only subset of the 12 non-parent names in the corrected closure | none in shell-library tests | corresponding config/home/root readers | 0 mutators | All / cfg varies | immutable during this binary | source closure found no parent writer; cannot form a same-process pair | source-only; no future edit |
| A4 | Child-only `Command::{env,env_remove,env_clear}` projections among the 12 non-parent names | command builders in test helpers | child bootstrap/probe decoders | 14 child spawners plus helper callers | child / cfg varies | per-`Command` environment | parent process is not mutated; exact child inheritance is intentional | LOW/source-closed |
| B1 | fd 1/fd 2 replacement: `dup`, `dup2`, `pipe`, `close` in capture helpers | `capture_stdout_once`, `capture_stderr_once` | two `PublicPromptRenderer` fallback tests; libtest reporter is competing writer | 2 | All / Unix | manual descriptor save/restore | stdout contamination proven; stderr structurally identical | renderer MEDIUM; capture helpers LOW |
| B2 | fd 0 `O_NONBLOCK` flag and terminal/console input mode | manual `fcntl(F_SETFL)` probes plus `minimal_terminal_guard_handles_creation` via `MinimalTerminalGuard::new` | libtest/input consumers and terminal/console state readers | 2 | All / Unix flag plus Unix/Windows terminal cfg | manual exact flag/mode restore | fixed descriptor is shared despite restoration; inherited child fd 0 would share the same open-file description | test symbols under-resolved |
| B3 | Child/local descriptors and inherited listener descriptors | pipe/listener/child builders | same fixture owner | 103 listener-dependent; 14 child-spawn-dependent | test/child / cfg varies | OS ownership plus joined fixture | no fixed-number replacement in parent; ownership remains local | LOW/source-closed |
| C1 | Process CWD | `set_current_dir`, `DirGuard`, direct/manual wrappers | `WorldRootSettings::effective_root`, config/workspace/root readers | 100 / 10 files | All / all platforms | `cwd_lock` in routing helpers, `#[serial]`, local guards | forced stable-reader race proven; one unannotated helper-closed test | `effective_root` HIGH, 2 direct / 1 process |
| C2 | umask | self-spawned sentinel test child | child filesystem creation | 22 helper-closed; 1 direct | child / Unix | child saves and restores exact mode | parent umask never changes; child exits after restore | LOW/source-closed |
| C3 | locale, timezone, process title, current-user/account overrides | none found | ordinary OS/account readers | 0 | All / cfg varies | immutable in this test process | exhaustive lexical categories found no mutator | no future impact |
| D1 | Global trace context/output | test `set_global_trace_context` + `init_trace(Some(...))` in routing/manager helpers | `append_to_trace`, telemetry and trace writers | 13 direct helper callers | All / all platforms | reset then global initialize; no ownership guard | forced output-owner replacement proven | `TraceContext::init_trace` MEDIUM; helper graph under-resolved |
| D2 | Private stop retry callback slot | `PrivateStopTransportRetryHookGuard::install` / Drop-clear | `request_private_stop*` hook read | 9 | All / Unix tests | mutex protects slot access, not owner lifetime | forced hook replacement proven | private/test graph under-resolved |
| D3 | Production signal/Ctrl-C handlers, including `initialize_global_sigwinch_handler_impl` | production initialization only | runtime signal consumers and the SIGWINCH background thread | 0 test installers/callers | product process / cfg varies | production lifecycle owner | source call closure proves no shell-library unit test calls `execute_with_pty`, so this handler/thread is never installed by this binary | production PTY/signal lifecycle owner |
| D4 | Panic hooks, tracing subscribers, global logger, allocator/runtime hooks | none found in shell-library tests | default libtest/runtime infrastructure | 0 | All | immutable/default | lexical and helper closure found no installer | no future impact |
| E1 | Existing environment coordination topology | `WORLD_ENV_LOCK`, `world_env_guard`, file-local env mutexes | all A1/A2 writers and stable readers | 534 / 35 files | All | one partial reentrant lock plus lock islands | opposite helper order and incomplete participation; `#[serial]` insufficient | `world_env_guard` CRITICAL: 70 impacted, 35 direct, five processes, nine modules; future test-only boundary only |
| E2 | `AGENT_EVENT_SENDER` / `EVENT_TEST_GUARD` | `init_event_channel`, `clear_agent_event_sender`; guard accessor `acquire_event_test_guard` | `publish_agent_event`, `agent_event_sender`, and publication readers | 10 guarded tests | All | two mutexes; poison `expect`/manual clear | competing stable readers are not excluded by file-local guard | private registry, source-closed |
| E3 | Non-keyed `REPORT_CACHE` | `refresh_socket_activation_report` | `socket_activation_report` | 1 mutating focused test | All / Linux behavior | mutex, no key or per-test reset | cache survives exact PATH/timeout restoration; proven | refresh HIGH, 3 direct / 3 process families |
| E4 | `CONFIG_PATCH_CACHE`, `POLICY_SNAPSHOT_CACHE` | keyed cache insert/update | exact path/file-stat keyed readers | multiple temp-root tests | All | mutex and exact workspace/global path keys | different fixtures cannot alias without same exact key/stat | production-adjacent but no edit |
| E5 | `WorldDispatchConcurrencyTracker` global maps | acquire/release test paths | cap/session lookup | 4 direct tracker tests plus fixture callers | All | mutex; key is caller session ID | forced reused-ID cap consumption proven | production tracker adjacency; private injection only |
| E6 | `PTY_ACTIVE` atomic | `active_guard_resets_flag_on_drop` | PTY routing/load checks | 1 direct mutator | All | atomic and RAII for product work | test can overwrite a real concurrent PTY state | production readers frozen |
| E7 | `world::SessionWorld` shared test-root override | two shell tests set/reset override | world metadata root readers | 2 | All / tests | currently held under world environment guard | safe only while unified guard participation remains complete | graph source-closed |
| E8 | Platform `GLOBAL_CTX` / Windows `CONTEXT` non-resettable initialization | no Linux-wall test mutation | platform context readers | 0 in Linux wall | platform cfg | once-only immutable initialization | cannot change after initialization and absent from active cfg | production platform owner frozen |
| E9 | Immutable `OnceLock` values, regexes, instance IDs, monotonic atomics | one-time initialization / atomic increments | same thread-safe primitives | many readers | All | `OnceLock`, immutable values, atomics | values do not depend on mutable env/CWD/user/policy paths | LOW/source-closed |
| E10 | `ACTIVE_PTY` registry and Windows `WIN_PTY_INPUT_GATE` | `ActivePtyGuard::register` / Drop-clear; Windows guard toggles the input gate | `active_pty_control` and PTY control dispatch readers | 1 direct test | All; additional gate on Windows | mutex/RAII, but one process-global owner slot | forced overlap proved a second registration replaces the first test's control | `ActivePtyGuard::register` LOW, one direct caller |
| E11 | `substrate_broker::GLOBAL_BROKER` mutable broker singleton | `with_test_mode`, `set_policy_mode`, policy initialization/reload paths | `policy_mode`, broker evaluation and selector readers | 23 exact tests | All / cfg varies | `OnceLock<BrokerHandle>` containing mutable `RwLock<PolicyBroker>`; no owner-scoped reset | forced overlap proved a stable reader observes another test's policy mutation | `set_policy_mode` LOW/under-resolved; source closure exact |
| F1 | Predictable private stop/cancel/prompt/startup/toolbox socket paths, especially `/tmp/substrate-agent-hub-stop/sessdispatch-*.sock` | exact 62 test bodies and their test-owned short root/home/store construction in `02` | corresponding stop/cancel/prompt/startup/toolbox clients | 62 exact tests: 36 orchestrator plus 26 async-REPL | host filesystem / Unix | some TempDir roots; some path-length fallbacks to predictable `/tmp` names | stale `sessdispatch-ashmember.sock` caused `AddrInUse`; six new stale stop nodes observed | direct path plus registration-helper closure exact in `02` |
| F2 | Unique TempDir Unix/TCP listeners | `UnixListener::bind`, `TcpListener::bind` on fixture-owned roots/port 0 | paired client/tasks | 103 helper-closed; 68 direct | fixture / cfg varies | unique temp root or kernel-assigned port; awaited owner | no cross-test alias when teardown is awaited | LOW/source-closed |
| F3 | Fixed TCP ports and inherited service endpoints | no fixed test bind found; child endpoint projection only | test client fixtures | 0 fixed-port mutators | child/fixture | kernel port 0 or explicit unique endpoint | no competing fixed port in shell lib process | no future impact |
| F4 | Cross-process lock files and private authority/temp roots | child/store fixture writers | exact-root store readers | 14 child spawners plus store suites | filesystem/child | exact private root, durable lock/CAS | proven safe when each test owns its root; parent ambient roots handled by A1 | store symbols production-adjacent, frozen |
| G1 | Wall-clock sleeps used to create ordering | 19 exact direct sleep tests in seven allowlisted files | competing task/thread readiness | 19 | test runtime | elapsed sleep only | scheduler load can invert intended order | source-closed tests; exact manifest in `02` |
| G2 | Bounded protocol timeouts/retry waits after readiness | `timeout`, retry loops, remaining helper sleeps | ready-published servers/clients | 94 timeout-dependent; remaining sleep closure | test runtime | explicit listener/state readiness plus deadline | timeout tests are safe where readiness precedes the clock | production timeout bodies frozen |
| G3 | Paused/advanced Tokio clocks or global clock override | none found | Tokio test-local time | 0 | runtime-local | none needed | no shared/global clock manipulation | no future impact |
| H1 | Abort without awaited termination | ten named orchestrator test servers/tasks | socket/root/env teardown | 10 | async test runtime / Unix | abort and sometimes one yield; no join | tenth case found; stale task/socket can outlive restore/unlock | test bodies under-resolved |
| H2 | Abort followed by awaited termination | remaining abort paths | same task owner | 28 helper-closed | async test runtime | `abort`; await `JoinHandle` | cancellation completion is observed before fixture drop | source-proven safe |
| H3 | Joined threads/children and runtime-scoped background tasks | 127 Tokio-spawn-, 48 thread-spawn-, 14 child-spawn-dependent tests | corresponding join/wait/Drop owners | overlapping aggregate | runtime/child | join/wait or runtime completion | no unjoined owner found outside H1; child death tests wait exact exit | source-proven safe |
| I1 | `#[serial]` groups | 550 source annotations | serial_test scheduler | 550 | libtest | named/default serial groups | excludes only participating tests, not unannotated readers or reporter | test-only; no graph node |
| I2 | Custom lock islands and order | env, CWD, event, per-helper mutexes | nested helpers | 4 lock families plus call closure | All | partial/reentrant and non-reentrant mutexes | mixed ENV/CWD order and poison semantics require one reviewed topology | helper graph under-resolved |
| I3 | libtest reporter and thread-count assumptions | reporter writes fds; no resource mutation by tests | all 1,263 discovered Linux tests | 1,263 | All | libtest | reporter is safe until a test captures fixed fd; no test may rely on thread count | runner external; B1 owns defect |

### Semantics, ownership, and primary dispositions

The compact fields below are binding. “Exact” means prior value or absence is restored without
Unicode conversion; “local” means no parent-process restoration exists because the resource is
owned by a child/fixture. Every row has exactly one primary disposition.

| ID | Restoration / panic / nesting | Async lifetime and subprocess inheritance | Potential competitors / proof | Primary disposition | Future owner packet and allowlist |
|---|---|---|---|---|---|
| A1 | exact `OsString`/absence; unwind-safe; stack-safe nesting or reject before mutation | retain through awaited cleanup; intentional bounded inheritance | all HOME/socket mutators and stable authority readers; proven races | `UnifiedProcessStateLock` | combined Harness; existing F0/F0a allowlists unchanged |
| A2 | exact `OsString`/absence; same panic/poison/nesting contract | retain across dependent child construction/termination | every ambient mutator versus stable selector reader; forced pair proven; XDG/settings corrections source-closed | `UnifiedProcessStateLock` | combined Harness; exact test-only files/symbols/tests in `02`; no XDG-specific lock |
| A3 | no mutation/restoration | immutable read | no parent writer in bounded process | `ProvenConcurrencySafe` | frozen; no future allowlist |
| A4 | `Command`-local bytes; parent unchanged | child receives explicit/inherited bounded projection and is waited | other processes cannot mutate parent | `ProvenConcurrencySafe` | frozen except already-authorized parent guard migration |
| B1 | exact descriptor restoration is insufficient; panic could strand redirection | reporter writes concurrently; child isolation not needed | libtest reporter; stdout failure and stderr structural proof | `ExplicitDependencyInjection` | F0b; exact one-file contract unchanged |
| B2 | exact fd flags/mode currently restored, but panic window remains | flag child owns null/pipe fd 0; mode child owns a PTY slave/console; neither inherits parent fd 0 | any stdin/terminal consumer in process; mode mutation must actually execute | `SubprocessIsolation` | Harness; exact two tests in `02` |
| B3 | OS closes local owned handles; normal RAII/unwind | join/await before fixture drop | unique owned endpoints only | `ProvenConcurrencySafe` | frozen |
| C1 | exact `PathBuf`; unwind-safe; stack-safe/rejected nesting | retain through dependent async/child work | all CWD mutators and stable root readers; forced race proven | `UnifiedProcessStateLock` | Harness; ten exact files in `02` |
| C2 | child restores exact mode; child exit is final containment | parent waits child | no parent-process competitor | `ProvenConcurrencySafe` | frozen |
| C3 | no mutation/restoration | none | no pair exists | `ProvenConcurrencySafe` | frozen |
| D1 | global API has no owner-scoped exact restore; reset is not composable | writers can outlive helper return | routing versus manager trace owner; forced race proven | `SubprocessIsolation` | Harness; exact helper callers in `02` |
| D2 | explicit callback lifetime replaces global install/clear; unwind-safe | callback belongs to one test server episode | nine hook installers; forced replacement proven | `ExplicitDependencyInjection` | Harness; private test sections in orchestrator file |
| D3 | production lifecycle restoration not a test concern | product-owned signal task/thread | no `execute_with_pty` caller or installer in this unit-test binary | `SeparatelyOwnedDeferred` | production PTY/signal lifecycle; cannot perturb this wall because source closure proves it is never installed here |
| D4 | no mutation | none | no pair exists | `ProvenConcurrencySafe` | frozen |
| E1 | coordinator owns exact restore, panic/poison, stack nesting | same as A1/A2 | every lock island; topology defect source-proven | `UnifiedProcessStateLock` | combined Harness; `execution/mod.rs` test-only coordinator |
| E2 | no parent restoration: bounded child exit discards its registry after owned event work ends | wait/reap child before parent test returns | channel initializers versus unguarded parent readers; child containment removes the pair | `SubprocessIsolation` | Harness; exact ten test entry points in four files in `02`; resource and production symbols frozen |
| E3 | parent cache is intentionally non-resettable | helper process exit discards cache | mutated env test versus later report reader; persistence proven | `SubprocessIsolation` | Harness; exact socket-activation test only |
| E4 | keyed replacement under mutex; panic-safe lock semantics | values scoped by exact path/stat key | different TempDirs cannot share key | `ProvenConcurrencySafe` | frozen |
| E5 | no global reset; unique fixture key removes collision | task releases production guard normally | repeated literal session IDs; cap theft proven | `ExplicitDependencyInjection` | Harness; test ID constructors only |
| E6 | atomic swap/restore is not owner-scoped | product PTY work may overlap | sole mutator versus routing readers | `SubprocessIsolation` | Harness; exact one test only |
| E7 | exact test override restore under environment coordinator | await world fixture work before restore | two writers/readers share same lane | `UnifiedProcessStateLock` | combined Harness; existing test sites |
| E8 | non-resettable initialization is immutable after set | platform-local | inactive cfg in Linux wall; no mutation | `ProvenConcurrencySafe` | frozen/platform owner |
| E9 | immutable/atomic semantics; no restoration | thread-safe | no environment/path-dependent mutable payload | `ProvenConcurrencySafe` | frozen |
| E10 | no composable exact restore for overlapping owners; RAII only clears its own current slot | bounded child owns all control/task lifetime and is awaited | two `ActivePtyGuard` owners; forced replacement proven | `SubprocessIsolation` | Harness; exact one test in `02` |
| E11 | no owner-scoped broker reset; process exit discards the test mutation | bounded child is waited/reaped before parent returns | broker mutator versus stable `policy_mode` reader; forced pair proven | `SubprocessIsolation` | Harness; exact 23 tests in `02` |
| F1 | fixture-owned path removed only after server termination; unwind cleanup | confirm task exit and socket unlink | predictable same path across tests/processes; stale failure proven | `ExplicitDependencyInjection` | Harness; exact 62 test bodies and test-owned root construction in two files in `02` |
| F2 | TempDir/socket RAII after awaited owner | await/join confirmed | unique roots/port 0 | `ProvenConcurrencySafe` | frozen |
| F3 | local endpoint ownership | child/listener waited | no fixed port found | `ProvenConcurrencySafe` | frozen |
| F4 | exact private path/CAS semantics | wait child before root drop | distinct private roots; shared ambient roots fall under A1 | `ProvenConcurrencySafe` | frozen |
| G1 | no state to restore; remove scheduling assumption | barrier/readiness wait completes deterministically | scheduler load; ordering intent source-classified | `DeterministicSynchronization` | Harness; exact 19 test waits in seven files |
| G2 | no global state; deadline starts after readiness | owner awaits/join | shared load cannot change established ordering contract | `ProvenConcurrencySafe` | frozen unless a focused test is listed in G1 |
| G3 | no override | runtime-local | none | `ProvenConcurrencySafe` | frozen |
| H1 | teardown order is binding and unwind-safe | abort/stop → await/join → verify cleanup → restore → unlock | all ten named tests; stale socket/task evidence | `TerminationConfirmedTeardown` | combined Harness; ten exact test bodies |
| H2 | await observes termination before Drop | confirmed | none after join | `ProvenConcurrencySafe` | frozen |
| H3 | join/wait/runtime completion precedes return | confirmed or runtime-scoped | no owner crosses return outside H1 | `ProvenConcurrencySafe` | frozen |
| I1 | supplemental only; no restoration authority | cannot cover reporter/unannotated work | 550 annotations versus 24 unannotated env dependents in broad closure | `UnifiedProcessStateLock` | Harness; participation migration only, no “serial and stop” |
| I2 | ENV → CWD; restore each before reverse unlock; explicit poison/nesting | retain all held lanes through owned cleanup | existing reverse/partial acquisition; event locks remain child-local | `UnifiedProcessStateLock` | Harness; test-only coordination helpers |
| I3 | no test-owned state | reporter lifetime equals process | reporter only conflicts with B1 descriptor capture | `ProvenConcurrencySafe` | runner frozen; no suppression/thread-count gate |

Disposition totals reconcile exactly: 7 `UnifiedProcessStateLock`, 4
`ExplicitDependencyInjection`, 1 `TerminationConfirmedTeardown`, 1
`DeterministicSynchronization`, 7 `SubprocessIsolation`, 17 `ProvenConcurrencySafe`, and 1
`SeparatelyOwnedDeferred` = 38. Category totals reconcile exactly: A 4 + B 3 + C 3 + D 4 + E
11 + F 4 + G 3 + H 3 + I 3 = 38. No broad-wall-capable row is deferred.

### Frozen combined implementation gate

The preserved combined F0/F0a/F0b/Harness implementation remained within the
corrected consolidated allowlists and additive file/symbol table in `02`; no new tracked file was
added. The restored candidate's direct-primitive and wrapper/callsite
inventories reconcile to zero unclassified rows. All production callers
and behavior are frozen except F0b's private writer delegation with byte-identical production
output. Focused proof covers all eleven proven interference families, exact restoration,
non-Unicode values, panic, poison, nesting, child inheritance, unique ID/path ownership, descriptor
bytes, confirmed async termination, and deterministic readiness. Every helper child uses a
recursion-proof sentinel and bounded timeout, propagates nonzero/signal status, kill-then-waits and
reaps on timeout, and reports no secret-bearing state. The B2 flag child owns a fresh null/pipe
stdin open-file description; the mode child owns a PTY slave/console and proves the guarded
terminal/console mutation actually ran. Neither inherits parent fd 0, and both prove the parent's
fd-0 flags/mode unchanged after ordinary and panic/abort child outcomes. Final proof is at least three
independent default-parallel shell-library walls and one canonical serial wall with identical
counts, failure names, and normalized signatures. That proof is complete and establishes F's clean
comparison baseline as `1280 discovered / 1235 passed / 45 failed / 0 ignored`.

F0-HC by itself authorized no implementation beyond its consolidated harness contract. The exact
candidate now marks F0, F0a, F0b, F0-HC, and the combined closeout complete. Its environment
inventory is corrected; the implementation changes no runtime/user behavior and promotes no seam.
At that historical F0-HC checkpoint, F remained unstarted and was the next packet. The completed-F
gate record below supersedes that next-task status.

### Corrected historical differential evidence-authority contract

This contract supersedes only the impossible assignment of transition-matrix authority to the
incomplete historical parallel output. It waives no regression gate.
The bounded provenance result is `HistoricalParallelArtifactUnavailable`.

1. **Historical serial semantic authority.** The authenticated complete serial wall has 1,263
   discovered, 1,202 passed, 61 failed, and 0 ignored tests, with complete names and normalized
   signatures. Its exact candidate transition matrix is 1,202 `PassToPass`, zero `PassToFail`, 45
   `FailToSameFailure`, zero `FailToChangedFailure`, 16 `FailToPass`, zero `Removed`, zero
   `RenamedOrSubstituted`, 17 `NewPass`, zero `NewFail`, and zero `NewIgnored`. Every one of the 16
   `FailToPass` rows requires an exact causal audit; every one of the 17 `NewPass` rows must be an
   added authorized test. Deterministic listings at the serial baseline and candidate must prove
   zero removed, renamed, or substituted tests.
2. **Final concurrency authority.** Three independent default-parallel walls and one canonical
   serial wall of the exact candidate must each report 1,280 discovered, 1,235 passed, 45 failed,
   and 0 ignored, with identical failure-name sets and normalized signatures. Any disagreement is
   a hard stop. This proves the repaired harness has no parallel-versus-serial semantic outcome
   change.
3. **Historical parallel diagnostic evidence.** The authenticated aggregate remains 1,263
   discovered, 1,113 passed, 150 failed, and 0 ignored. Panic headers and the final-summary tail
   from the same transcript yield all 150 names, but retained panic output is complete for only 37
   names. Because the artifact lacks complete normalized signatures, it is diagnostic only. It
   cannot prove a parallel
   `PassToFail`, a historical parallel signature comparison, membership of the final failures, or
   exactly 105 named historical failures becoming passes. Rerunning the nondeterministic historical
   code creates a new run and cannot recover this historical artifact.
4. **Source and inventory continuity.** Completion still requires the exact 45-file candidate
   manifest and per-file fingerprints, deterministic endpoint listings, the exact 17 additions in
   `02`, zero removed/renamed/substituted tests, no changed production execution flow, the exact
   serial matrix, and final four-wall identity. The manifest SHA-256 is
   `b9e3a44dd671409f66e2d62d48cb494ab147069a74ed71f2308e031f56372ae6`; the fingerprint aggregate
   is `41cb1a325add4efd8198872b456c3ae0fba73f8c4943e4b59c06e2bc76d8b479`; the ordinary patch is
   `7ab220a715f4ae3389be314da2e6b0e614fcffb7a92166f04fde999570801861`; and the full-index patch is
   `adc1c5952e2ac4cc97881a1bf4df8e24b00d4d4ba337e692bf21965151d5c0c4`.

The original exact candidate remains preserved at commit
`86ed6f5620787121b1c2e5b033ee8d6f9ff369d3` and tree
`f6480f3986d43bb41e5387fa1ba5b68ae53f598b`. Its byte-identical patch is committed on the corrected
source base as `770a6a9de9f537f7bc179c75421abbc3fff05b8d` with tree
`61fdd2e9476f1ce3e041720ce106f7c3427895be`. Environment closure, renderer/injection, and
lifecycle/subprocess implementation reviews are CLEAN. The old containment/differential review is
not reused; fresh reviewer `/root/final_containment_corrected_authority`
(`019f8681-7957-7cc3-88fa-37ab3ad2fc87`) returned CLEAN under this contract.

`PassToFail=0`, `FailToChangedFailure=0`, `Removed=0`, `RenamedOrSubstituted=0`, `NewFail=0`, and
`NewIgnored=0` are satisfied hard requirements. At that historical harness checkpoint no product
behavior changed, F0/F0a/F0b/F0-HC were complete, F remained unstarted, and no seam was promoted.
The completed-F gate record below supersedes that packet status.

### Canonical harness gate result

| Gate | Canonical result |
|---|---|
| Candidate identity | 45 files, 5,630 insertions, 2,104 deletions; commit `770a6a9de9f537f7bc179c75421abbc3fff05b8d`; tree `61fdd2e9476f1ce3e041720ce106f7c3427895be`; manifest `b9e3a44dd671409f66e2d62d48cb494ab147069a74ed71f2308e031f56372ae6`; fingerprints `41cb1a325add4efd8198872b456c3ae0fba73f8c4943e4b59c06e2bc76d8b479`; ordinary patch `7ab220a715f4ae3389be314da2e6b0e614fcffb7a92166f04fde999570801861`; full-index patch `adc1c5952e2ac4cc97881a1bf4df8e24b00d4d4ba337e692bf21965151d5c0c4` |
| Corrected inventory | 86 names = 74 parent-mutated + 12 child-only/read-only; 395 primitives; 73 dynamic calls; 43 sinks; 1,005 resolved callsites; 534 tests across 35 files; 38 resource rows; zero unresolved dynamic rows; all 38 dispositions implemented or retained |
| Focused and quality proof | All eleven interference families plus HOME/socket overlap, renderer isolation, negative projection, environment/CWD restoration, fork publication, async termination, subprocess isolation, exact absence/non-Unicode, panic/poison/nesting/reacquisition, and child status propagation; shell/workspace all-target checks, Clippy `-D warnings`, format, and diff checks pass |
| Final concurrency authority | Three parallel walls plus one serial wall each `1280/1235/45/0`; common failure-name hash `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`; common normalized-signature hash `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3` |
| Serial semantic authority | `1202 PassToPass`; `0 PassToFail`; `45 FailToSameFailure`; `0 FailToChangedFailure`; `16 FailToPass`; `0 Removed`; `0 RenamedOrSubstituted`; `17 NewPass`; `0 NewFail`; `0 NewIgnored` |
| Transition audit | The 16 `FailToPass` rows are causally tied to seven detached-availability, two fork-publication, six stop-dispatch, and one hidden-owner harness fixes. The 17 `NewPass` rows are exactly the authorized tests listed in `02`. |
| Historical parallel limitation | `1263/1113/150/0` is diagnostic only; all 150 names are recovered, only 37 complete panic bodies remain, and 113 normalized signatures are unavailable. No historical parallel transition matrix or 105-transition claim exists. |
| Containment and review | GitNexus's one MEDIUM process label resolves to test-only `AuthorityEnvTestTempDir::new`; no production flow changes. The only production hunk is authorized mechanical F0b delegation with byte-identical output. Three implementation reviews and the fresh containment review are CLEAN. |
| Historical status and sequence | F0/F0a/F0b/F0-HC complete; no production or user-facing behavior change; no seam promotion; `Routes A–E → F0/F0a/F0b/F0-HC complete → F → renewed R2-2 closeout → R2-3 → R2-4 → R3`; F was unstarted at that checkpoint. The completed-F gate record below supersedes this status. |

## A1.1d-5R2-2F readiness and outbound-environment correction

This contract supersedes the earlier F builder/readiness boundary. F1 and F2 are complete local
commits; F3/F4 are incomplete, blocked, and preserved; F itself is not complete.

### Exact outbound command environment

The authenticated builder constructs this entire `ExecuteRequest.env` map from immutable guest
constants. Values shown here are generated values, not forwarded parent `HOME`, XDG, PATH, or other
environment state.

| Exact name | Exact value | Classification and reason |
|---|---|---|
| `SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR` | `/var/lib/substrate/world-deps/bin` | Required non-secret guest runtime projection; ambient override forbidden. |
| `PATH` | `/var/lib/substrate/world-deps/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin` | Required deterministic guest command discovery; not inherited. |
| `HOME` | `/root` | Fixed guest execution home required by the existing world command envelope; host HOME is never read or copied. |
| `XDG_CONFIG_HOME` | `/root/.config` | Fixed guest runtime path; ambient XDG input is never read or copied. |
| `XDG_DATA_HOME` | `/root/.local/share` | Fixed guest runtime path; ambient XDG input is never read or copied. |
| `XDG_CACHE_HOME` | `/root/.cache` | Fixed guest runtime path; ambient XDG input is never read or copied. |
| `TERM` | `xterm-256color` | Fixed non-secret command-runtime posture; ambient TERM is not inherited. |

No other entry is allowed. Locale, timezone, and color variables are unnecessary because source
closure found no authenticated-path requirement for them. CWD, request profile, policy snapshot,
world-network policy, and filesystem mode remain exact structured `ExecuteRequest` fields.
World-service already derives any internal enforcement environment from those structured fields;
the shell must not duplicate it in the outbound command environment. No schema or side channel is
added.

### Forbidden input and forwarding table

| Class | Exact examples or pattern | Required disposition |
|---|---|---|
| Wildcard/prefix forwarding | all ambient `SUBSTRATE_*`, all ambient `WORLD_*`, all `LC_*`, or the complete parent environment | FORBIDDEN. An allowlist must enumerate all seven names above and generate every value. |
| Authority selectors | `SUBSTRATE_HOME`, `SUBSTRATE_ROOT`, `SUBSTRATE_WORLD_SOCKET`, `HOME`, all ambient XDG variables, `CODEX_HOME`, anchor/project/workspace selectors | FORBIDDEN. They cannot accompany A as command authority. The fixed guest values above are constants, not selectors. |
| Service/backend selectors | `SUBSTRATE_WORLD_AGENT_BIN`, socket-activation overrides/timeouts, world backend/socket/request-profile selectors, guest-bin-dir override | FORBIDDEN in request environment and forbidden as explicit-F readiness input. |
| Policy/config selectors | policy-mode, world network/fs/caging selectors, enforcement-plan payloads, config roots, generated manager/bootstrap selectors | FORBIDDEN. Structured authenticated request fields remain authoritative. |
| Hidden authority carriers | bootstrap carrier, install context/home, commitment preimage, selected user/UID, shim caller/stack/depth state | FORBIDDEN. No hidden transport can reconstruct or supplement A. |
| Secrets and credentials | any credential, session secret, API key, access key, private key, authorization value, or provider token | FORBIDDEN regardless of name or prefix. |
| Prompt/request material | prompt text, request preimage, serialized carrier/request bytes, raw policy/config payloads | FORBIDDEN. |
| Compatibility-only or unnecessary state | ambient locale, timezone, color, terminal, user/shell/temp/editor variables | EXCLUDED. No downstream precedence rule may be used as a security boundary. |

The builder may retain an existing non-environment request metadata field only if its current
non-authority semantics are source-proven and unchanged; that field cannot select A, readiness,
policy, backend, or credentials. This is not permission to add a transport field.

### Explicit readiness contract

1. `ensure_world_service_ready()` keeps its current signature and compatibility behavior.
2. One private Linux core receives the exact socket path plus the minimum private service posture.
3. The core never resolves the target socket from environment, home/root state, CWD, activation
   report path, or a global side table.
4. Compatibility delegates with legacy socket/binary inputs and retains every existing caller.
5. F delegates with `/run/substrate.sock` and immutable installed-product binary
   `/usr/local/bin/substrate-world-service`; environment selection is unavailable.
6. Probe, activation wait, stale-socket safety, spawn, readiness polling, timeouts, and error
   classification exist only in that owner.
7. Authenticated validation precedes readiness; readiness precedes request construction.
8. Existing capability, policy, systemd/service, service unit, socket, lifecycle, non-Linux,
   world-service, transport, receipt, supervisor, and retained-worker behavior is frozen.

### Required future F3/F4 proof

1. Under conflicting ambient `SUBSTRATE_WORLD_SOCKET`, explicit readiness probes and connects only
   to the authenticated fixed socket; environment-only socket selection cannot affect F.
2. Existing no-argument readiness callers retain their current behavior, including compatibility
   override behavior.
3. Capability probe, activation detection/wait, stale-socket handling, compatibility spawn fallback,
   polling intervals, timeout values, and error classification/text remain equivalent.
4. Source inspection and tests prove no readiness code is duplicated in the F builder or either
   consumer.
5. The serialized runtime request contains exactly the seven environment entries above, with exact
   values, and no parent-environment entry.
6. Poisoned authority, backend, policy, HOME/XDG, shim/bootstrap, locale, and unknown-prefix values
   do not appear in the request or alter its target.
7. Synthetic credential and request markers do not appear in the request environment, serialized
   transport, errors, logs, traces, doctor output, or stored fixtures.
8. A under conflicting B executes against A. No downstream precedence or overwrite is accepted as
   proof.
9. Missing, malformed, tampered, wrong-principal, or mismatched authority fails before readiness,
   request construction, mutation, or launch.
10. Both exact consumers use the additive builder; no third consumer is redirected.
11. Existing world execution, network allow/deny, filesystem, caging, capability, lifecycle,
    receipt, supervisor, and retained-worker behavior remains unchanged.
12. Non-Linux behavior remains frozen under R2-3 ownership.

Any credential/request marker crossing the boundary, any need for a transport/world-service edit,
or any inability to preserve compatibility callers is a hard stop. Passing these tests closes only
the corrected F3/F4 work; it does not start F5 or the final F walls and does not promote a seam.

## Historical F5-PD non-mutating nested Doctor contract

This was the controlling contract at the post-F3/F4 checkpoint. Earlier text that called F3/F4
incomplete or placed F5 immediately after them was historical and superseded there. F3/F4 were
complete, F5-PD was unimplemented, and F5/later nodes were unstarted. The completed-F gate record
below supersedes this historical packet status while preserving the contract's boundary decisions.

### Compatibility invariant

Normal public `substrate world doctor --json` is behaviorally frozen. Its readiness behavior,
socket-activation observation and connection, service probing, world-service calls, filesystem and
capability probes, `/v1/execute` fallback, output, exit codes, errors, and platform behavior are not
changed or represented as side-effect-free. The Host Doctor paths and all public callers remain
unchanged as well.

### Authenticated internal mode

Whenever the existing authenticated Linux F5 composition has World enabled and reaches
`gather_world_doctor_snapshot`, the parent must still create a real product-CLI child. The frozen
World-disabled `build_report` branch continues to return `disabled_world_doctor_snapshot` without
spawning a child or consulting a fixture. The enabled parent adds a hidden argv
discriminator named `--internal-passive-world-doctor-v1` inside the existing
`WorldAction::Doctor` grammar beside the canonical encoded `InstallBootstrapContextCarrierV1` and
exactly `world doctor --json`. The child must:

1. require the hidden carrier on argv; environment state cannot select or repair the mode;
2. require an exclusive World Doctor JSON action and reject every other flag/action combination;
3. decode and validate canonical carrier bytes, commitment, selected prefix, current Unix
   account+UID, optional declared selector, and conflicting checked projections;
4. branch immediately after `decode_and_bind_unix_install_bootstrap_context` and before
   `install_bootstrap_projections`;
5. avoid home scaffold, trace setup, config/policy/platform/socket/service selection, and ordinary
   routing;
6. emit only the existing bounded diagnostic envelope with `ok=false`, exact non-secret A
   diagnostic identity, platform label, and unavailable status, then return the existing
   diagnostic failure class; and
7. never fabricate coherent success.

The initial child JSON shape is exact and private to this authenticated mode:

```json
{
  "schema_version": 1,
  "platform": "<compile-time target>",
  "ok": false,
  "host": {
    "platform": "<compile-time target>",
    "ok": false,
    "selected_host_prefix": "<validated A>",
    "host_context_commitment": "<validated non-secret commitment>"
  },
  "world": {
    "status": "unavailable",
    "ok": false,
    "selected_host_prefix": "<validated A>",
    "host_context_commitment": "<validated non-secret commitment>"
  }
}
```

No additional field is permitted. Both identity copies come from the same validated typed carrier;
they are diagnostic proof for the parent validator, not independent authority or health evidence.
The child exits 4 after emitting the JSON. F5 composition may map that bounded failure through its
existing `NeedsAttention`/unavailable path but cannot retain it as healthy.

Whenever the Linux World-enabled branch reaches the gather function, the parent spawns this child
and does not inspect `A/health/world_doctor.json`. At the raw-stdout boundary,
`run_json_subcommand` must decode through exact private
`#[serde(deny_unknown_fields)]` wire structs before constructing a `serde_json::Value`; this rejects
duplicate or unknown fields before they can be collapsed. The Linux command decoder then receives
expected A prefix/commitment and accepts only the exact object above, exit 4, and empty stderr. It
discards the raw result and constructs only `WorldDoctorStatus::NeedsAttention`,
`ok=false`, platform from the compile-time label, source `command`, exit 4, empty stderr/details,
and error `passive world doctor unavailable`. Any absent/additional/duplicate/malformed/mixed/tampered field,
wrong platform/identity/exit, nonempty stderr, or secret/request-bearing key constructs the same
bounded closed shape with error `passive world doctor incoherent`; rejected bytes are never echoed.
The Linux value decoder becomes `cfg(test)`-only and direct fixtures pass through the same
validator. Non-Linux gather/decoder/fixture/public-child compatibility remains unchanged and is
barred from authenticated/native proof.
The validator recursively normalizes key case and separators and rejects credential, token,
API/private-key, authorization, password, secret, prompt, request/body/bytes/input,
carrier/auth-bundle, parent/full-environment, and commitment-preimage keys. It does not scan by
printing values and does not include a rejected key or value in any error.

The mode must not call service readiness; start/restart world-service; stat/connect/create/remove
an activation socket; invoke `socket_activation_report`; call `/v1/capabilities`,
`/v1/doctor/world`, `/v1/execute`, or a WebSocket endpoint; construct a Tokio/runtime transport;
create a world/overlay/probe; read active capability/network/filesystem state; install, provision,
sync, migrate, lock-promote, touch, truncate, rewrite, repair, or clean anything; recover authority
from environment, HOME/XDG, CWD, account defaults, repository paths, or global state; or expose
carrier/credential/request/prompt/preimage bytes.

### Evidence and truthful classification

The authenticated carrier is authoritative only for A identity. Compile-time platform is a
diagnostic label. Neither is runtime-health proof. No existing production artifact is presently
authorized as durable coherent World Doctor evidence. Static socket/unit/process/filesystem state,
even when read-only, is insufficient because it cannot prove endpoint/world/probe coherence;
inactive, absent, stale, or unprovable state yields unavailable and must not trigger startup.

The existing `WorldDoctorSnapshot`, `WorldDoctorStatus`, `WorldDoctorReportV1`, human renderer, and
public transport JSON remain frozen. The existing `NeedsAttention` plus bounded error
representation is the contract-owned fail-closed class for passive unavailable/incoherent state;
`details` and `stderr` are `None` for both outcomes.
The child exits through the existing failure class so human output names the unavailable reason;
JSON carries `ok=false` and the same class. A malformed, mixed, stale, conflicting, secret-bearing,
or tampered result is incoherent/closed and is not retained as a healthy payload. This is honest
without adding an enum variant or changing a wire field.

If implementation discovers that this representation cannot express the same truthful human and
JSON outcome, it stops as `CrossDocumentChangeRequired`; F5-PD does not authorize a schema change.
Likewise, finding a candidate durable evidence source does not authorize its use: source-close its
authority, freshness, nonmutation, identity, platform behavior, and impact in a separate decision
first. Do not add a side table, endpoint, daemon, broker, or persistence format.

### Exact implementation allowlist

| File | Existing symbol authorization | Additive private authorization | Frozen boundary |
|---|---|---|---|
| `crates/shell/src/execution/cli.rs` | EDIT `WorldAction::Doctor` only to add hidden bool `internal_passive_world_doctor_v1` and its colocated parser tests | None | Top-level `Cli`, every other field/variant, normal `world doctor [--json]` grammar/help/behavior |
| `crates/shell/src/execution/routing.rs` | EDIT `run_shell_with_cli` only for Linux exhaustive hidden-action validation/early return and non-Linux hidden-bit rejection | ADD one Linux-only private `passive_world_doctor_action_is_exclusive` predicate | `run_shell`, normal/no-bit action ordering, authentication owner, projection/scaffold/config/trace behavior for every other action |
| `crates/shell/src/execution/platform/mod.rs` | EDIT `handle_world_command` only to reject a leaked true hidden bit before the Doctor arm performs work | ADD one Linux-only, at-most-`pub(super)` `emit_authenticated_passive_world_doctor_v1` accepting `&InstallBootstrapContextCarrierV1` and returning the existing CLI result/exit form; colocated tests only | False-bit public Doctor body, `handle_host_command`, `resolve_doctor_world_disable_attribution`, platform adapters and all public Doctor semantics |
| `crates/shell/src/builtins/shim_doctor/report.rs` | EDIT only Linux cfg implementations of `gather_world_doctor_snapshot` when reached from the frozen enabled branch to remove production World Doctor fixture selection and add the hidden child discriminator; `run_json_subcommand` only at its Linux raw-stdout parse expression for strict private-wire decode; `snapshot_from_command` for expected-A/exact-schema/exit/redaction validation and bounded construction; and `snapshot_from_value` to become Linux-test-only and use the same validator | ADD exactly Linux-private `PassiveWorldDoctorChildV1`, `PassiveWorldDoctorHostV1`, and `PassiveWorldDoctorWorldV1` structs with `deny_unknown_fields`, plus `decode_passive_world_doctor_child_v1` and `validate_passive_world_doctor_child_v1` helpers | `build_report`, `disabled_world_doctor_snapshot`, `JsonCommandOutput`, all non-Linux implementations, every other statement in `run_json_subcommand`, `WorldDoctorSnapshot`, `WorldDoctorStatus`, report/wire schemas, renderer, world-deps fixture/composition, and broader F5 composition |

Permitted integration test files are exactly `crates/shell/tests/doctor_scopes_ds0.rs`,
`crates/shell/tests/shim_doctor.rs`, and `crates/shell/tests/shim_health.rs`. Colocated tests may be
edited only in the four production files above. No new file, re-export, public signature,
directory/module-wide authority, or third-party dependency is permitted.

The sole public Rust type-layout change is the additive hidden bool inside
`WorldAction::Doctor`; top-level `Cli` and `auto_sync.rs::cli_for_auto_sync` remain byte-for-byte
unchanged. Linux-private `snapshot_from_command` and Linux-test-only value decoding may add expected
A prefix/commitment parameters through cfg-specific implementations. All non-Linux private
signatures/bodies and all other existing signatures/visibility remain byte-for-byte unchanged. The
emitter is not exported outside the `execution` parent. Every non-Linux target rejects the hidden
mode and retains its ordinary compatibility behavior without a native/A-bound claim.

### Exact impact authorization

The existing edited symbols are LOW: `WorldAction` has 0 direct/0 total graph dependents;
`run_shell_with_cli` has one direct/two total in Execution and no attributed process;
`handle_world_command` has 0/0; `gather_world_doctor_snapshot` and `run_json_subcommand` each have
0/0; and
`snapshot_from_command`/`snapshot_from_value` each have one direct/one total caller in Shim-doctor
and no attributed process. Frozen `build_report` has one direct/five total, one process/one module;
frozen `disabled_world_doctor_snapshot` has one/six, one/two. The seven additive private symbols
have no pre-edit node. Those are the complete impact authorizations.

The following observations are explicit non-authorizations: `install_bootstrap_projections` HIGH
(4 direct/11 total, one process, three modules); `ShellConfig::from_cli` HIGH (7/8, one/three);
`WorldDoctorSnapshot` HIGH (4/10, one/three); `WorldDoctorReportV1` HIGH (2/7, one/four);
`ensure_world_service_ready` CRITICAL (4 direct/10 total, four/five); and
`socket_activation_report` CRITICAL (6 direct/12 total, four/five). Their bodies, signatures,
callers, process families, and module ownership are
frozen. The passive branch may occur before them; it may not edit, wrap, redirect, or call them.
The two CRITICAL lifecycle values are the test-inclusive `maxDepth=5` GitNexus results; the
test-excluded view is narrower and is not used as authorization evidence.

### Required proof

Implementation exit requires all twenty exact regressions:

1. Missing or malformed authenticated carrier rejects before observation.
2. Environment-only authority rejects.
3. Conflicting ambient B cannot affect the passive child.
4. Inactive service remains inactive; the frozen World-disabled branch returns its current
   disabled snapshot without spawning a child.
5. No activation socket connection occurs.
6. No service process starts.
7. No `/v1/doctor/world` request occurs.
8. No `/v1/execute` request occurs.
9. No world or filesystem probe is created.
10. No config, policy, metadata, socket, fixture, or service state mutates.
11. Existing passive evidence for exact A can compose truthfully. In the initial implementation,
    the existing permitted evidence proves only authenticated A identity, so truthful composition
    is unavailable, not coherent health.
12. Missing evidence returns unavailable.
13. Mixed A/B evidence returns incoherent or the exact contract-owned fail-closed class.
14. Malformed, tampered, duplicate-key, or unknown-field evidence fails closed before lossy value
    construction; include conflicting top-level and nested duplicates and B-then-A identity keys.
15. JSON and human classifications agree.
16. No credential, prompt, request, carrier, preimage, unselected host-path, or synthetic marker
    leaks; the already-public selected-prefix/commitment diagnostic fields remain bounded.
17. Normal public `world doctor --json` compatibility tests remain unchanged.
18. F3/F4 exact environment and readiness tests remain unchanged.
19. Non-Linux compatibility remains frozen and no native proof is fabricated.
20. No test is removed, renamed, ignored, substituted, or weakened.

The service/socket proof must observe accepts, requests, process identity, and before/after state
without invoking product lifecycle actions. The no-mutation proof snapshots exact relevant paths,
metadata, content hashes, socket identity, and service state before and after. Marker scans cover
stdout, stderr, JSON, human output, logs/traces created by the test harness, and retained fixture
payloads. Test fixtures never become production evidence.

### Mandatory stops and exit sequence

Stop on `RepositoryStateMismatch`; `ControlPackStateMismatch`; unapproved HIGH/CRITICAL impact;
new module/process-family ownership; transport/report-schema need; world-service or lifecycle
behavior need; inability to avoid socket activation; inability to represent unavailable/
incoherent truthfully; credential/request/carrier/preimage or unselected-path leakage; public or
product behavior regression; non-deterministic differential; or review infrastructure failure.
No test result waives a stop condition.

After implementation, focused proof and three fresh isolated read-only reviews must be CLEAN before
F5 can resume. Then the only legal sequence is `F5-PD → F5 → final F walls → F closeout → renewed
R2-2 integration closeout`. R2-3, R2-4, and R3 remain later and unstarted.

## A1.1d-5R2-2F completed gate record

The required sequence above completed through F closeout. It did not enter renewed R2-2
integration closeout. The following record is binding for the completed F boundary.

### Final composition contract

1. Request-scoped authenticated A is the sole identity owner. Selected prefix, non-secret
   commitment, CWD/workspace scope, config, inventory, dependency scope, passive child, and every
   retained diagnostic constituent must join to that exact A.
2. Linux `collect_doctor_snapshot_v1` validates A and reads A-derived config/global inventory plus
   request-scoped launch-CWD/workspace inventory through the shared authenticated context. It
   performs no applied/runtime-health probe and reports the bounded unavailable reason.
3. Linux no-fixture World-enabled composition uses the frozen F5-PD authenticated passive child.
   Carrier identity is not runtime-health authority. Missing evidence is unavailable, never success.
4. `A/health/world_deps.json` remains compatibility/test evidence only. Its physical canonical path,
   identity, commitment, CWD, enums, item shape, and duplicates are checked. Runtime claims are
   rejected, and accepted fixture bytes are not retained as production truth.
5. Duplicate, unknown, malformed, partial, stale, mixed A/B, tampered, conflicting, or
   secret-bearing evidence is incoherent/fail-closed. Rejected raw values are not echoed to JSON,
   human output, errors, logs, traces, snapshots, or retained fixtures/state.
6. World-disabled composition returns the pre-existing disabled snapshot before any child, fixture,
   or runtime evidence selection.
7. Coherent success still requires all contract-mandated adequate runtime evidence. Because no such
   passive evidence exists, current Linux authenticated composition truthfully remains unavailable.
8. Public `world doctor --json`, report/transport schemas, readiness, exact seven-entry generated
   environment, socket/service lifecycle, world-service, execute, probes, policy/network/world-fs,
   caging/capability, secure-FD, receipt/supervisor, cleanup/rollback, and non-Linux compatibility
   remain frozen.

### Final regression authority

| Gate | Final result |
|---|---|
| Shell library, parallel wall 1 | 1,309 discovered; 1,264 passed; 45 failed; 0 ignored |
| Shell library, parallel wall 2 | 1,309 discovered; 1,264 passed; 45 failed; 0 ignored |
| Shell library, parallel wall 3 | 1,309 discovered; 1,264 passed; 45 failed; 0 ignored |
| Shell library, serial wall | 1,309 discovered; 1,264 passed; 45 failed; 0 ignored |
| Failure names | 45; SHA-256 `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70` |
| Normalized signatures | 45; SHA-256 `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3` |
| Clean F5-PD → F5 transition | two library `NewPass`, three focused integration `NewPass`; every adverse/removal/weakening transition zero |
| F5 doctor/Health suites | shim-doctor 20/20; shim-health 8/8; world-enable 21/21 |
| Auth/request/readiness/passive/policy focused library suites | 97/97 across the exact eight recorded filters |
| Managed secure-FD regressions | common 46/46; world-service launcher 18/18; gateway receiver 18/18; bounded gateway runtime 21/21 |
| Policy/network/capability regressions | shell policy snapshot 10/10; policy model 14/14; broker in-process 67/67 with eight frozen pre-witness shell-spawn failures; world-service routing 6/6 plus four exact authority/netfilter tests |
| Compilation and formatting | shell all-target, workspace all-target, `cargo fmt --all -- --check`, and `git diff --check` pass |
| Clippy | raw `-D warnings` retains exactly 20 inherited `needless_borrow` findings and no other warning; differential allows only that lint and passes |
| GitNexus | LOW; four F5 files; 27 mapped symbols; zero affected flows; no new process family |

The auxiliary full world-service unit run completed 131 tests without failure before two existing
FUSE doctor tests stalled on environmental unmount cleanup; it was terminated and is not claimed as
a full pass, privileged smoke, or a substitute for the bounded gates above.

Fresh whole-F read-only reviews `/root/f_final_authority_security`,
`/root/f_final_lifecycle_regression`, and `/root/f_final_source_platform` are CLEAN. F5 preservation
is `feat/preserve-a1-f5-runtime-20260722`; review-clean complete-F preservation is
`feat/preserve-a1-f-complete-runtime-20260722`. The F5-PD and blocked-donor preservation refs remain
unchanged, and the blocked donor was neither merged, cherry-picked, nor restored.

At that completed-F checkpoint, the exact historical next gate was **A1.1d-5R2-2 renewed
production-fix-free integration closeout**. It was a new proof node, not part of that F closeout,
and could not repair production code. The
[remediation-planning gate](../a1.1d-5r2-2-renewed-closeout/contracts-and-gates.md#a11d-5r2-2-closeout-remediation-contracts-historical-rp1rp2-record)
supersedes only that
next-gate disposition. R2-3, R2-4, R3, privileged product smoke, and direct-member Codex/UAA
gateway adoption remain outside the later closeout.

**Source provenance:**
- extracted from [`04-contracts-and-gates.md#remaining-r2-2-same-process-carrier-closure`](../04-contracts-and-gates.md#remaining-r2-2-same-process-carrier-closure), lines 585–1279; baseline span SHA-256 `086c84da784d994f860c3563607f83a4049db14a9b4936338b3e380c7b3b620b`
- extracted from [`04-contracts-and-gates.md#a11d-5r2-2f0-hc-corrected-complete-process-resource-ledger`](../04-contracts-and-gates.md#a11d-5r2-2f0-hc-corrected-complete-process-resource-ledger), lines 6192–6842; baseline span SHA-256 `75e58cb9aca728e69f8e9ea2e8115614e5fee65b27817324508d9419e0b13661`
