Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3F. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: c73ccaae7980f72393c7a01268e55c2d3330680d774b897fe8a9b97774de7667
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3F
- packet_id: A1.1d-5R2-3F
- next_increment: R2-3R

INCREMENT-TASK IDENTITY BARRIER

Do not edit, delegate, run implementation checks, or publish until the meta orchestrator sends a
follow-up binding your own real increment-task thread ID and host ID to this dispatch nonce. Echo
those exact IDs in every terminal receipt.

REPOSITORY BOUNDARY

Work only in the task-assigned checkout:

the task-assigned Linux Codex worktree

Never mutate these protected checkouts:

- /home/spenser/__Active_code/substrate
- /home/spenser/__Active_code/substrate-r2-3
- /home/spenser/__Active_code/substrate-r2-3-meta-orchestration
- C:\Users\spmcc\Documents\__Project_Code\substrate-r2-3

Canonical starting state:

- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- expected base commit: b9f975ab98a5d15fd63225366e0d32e399b7c608
- expected base tree: 4f59b0740de523708623442f564122c4722b6221
- required ancestor: 0f1e147fb735791b44a65099a65167cbdc1803af

Before editing, fetch/query the live remote and verify the exact base, tree, ancestry, cleanliness,
0 ahead/0 behind, and current GitNexus index. If they differ, send BASE_DRIFT or
BLOCKED_CONTRADICTION. Do not reconcile, merge, rebase, reset, clean, or force-push.

SUBAGENT ORCHESTRATION

Use repository-required model/reasoning settings for every subagent. Complete required pre-edit
impact analysis before any subagent edits an existing symbol. Give editing subagents mutually
exclusive ownership when practical. Use fresh read-only subagents for independent review. Do not
allow a reviewer to review implementation it authored.

INCREMENT CONTRACT

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3F

Use:

- using-agent-skills
- context-engineering
- source-driven-development
- api-and-interface-design
- test-driven-development
- incremental-implementation
- gitnexus-impact-analysis
- security-and-hardening
- code-review-and-quality
- git-workflow-and-versioning

This top-level task and every subagent must run on Standard/default speed, never Fast. Every
subagent must use GPT-5.4 with Extra High reasoning. Ground macOS/Windows factory, platform
adapter, gateway, diagnostic, and typed-mapping decisions in current repository truth, official
platform documentation where needed, and the published runtime-refactor control pack. Do not
reopen R2-2 or R2-3A through R2-3W5 without a concrete contradiction.

Load only:

- current status in `llm-last-mile/runtime-refactor/00-README.md`;
- the parent A1.1d-5R2-3 row, common child-subdivision rules, and R2-3F section in
  `llm-last-mile/runtime-refactor/03-phase-slice-map.md`;
- PI-054 through PI-058 and PI-075 through PI-076 current-state and required-action rows in
  `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`;
- the host-context boundary-carrier, `PlatformBootstrapMappingV1` construction/verification,
  typed factory projection, platform adapter, gateway/diagnostic, Lima R3-prerequisite, Windows
  mapping, and R3-exclusive lifecycle contracts in
  `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`;
- R2-MAP-MAC-01, R2-MAP-WIN-01, R2-DIAG-01, and only the applicable unchanged
  R2-RUNTIME-01 differential in `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`;
  and
- `06-review-finding-inventory.md` only for actual P3/P4 deduplication.

SELECTED OUTCOME

Make every macOS and Windows shell-side backend, platform-world, gateway, and doctor path consume
the already-verified typed `PlatformBootstrapMappingV1` projection. Make
`world-backend-factory::factory` require that explicit mapping on macOS/Windows and remove or
fail closed every contextless platform constructor before client/backend selection. Preserve
Linux's platform-independent factory behavior exactly.

EXACT COMPLETION CLAIM

R2-3F completes only PI-054 through PI-058 and PI-075 through PI-076. It supports PI-090 and
PI-091. It does not migrate replay, physical-shim, trace, or manager-hint callers; activate macOS
forwarding or Windows provisioning; change gateway policy/config/network/credential semantics;
change Linux factory behavior; create a new mapping authority; claim native macOS or Windows
evidence; complete any R3 row; or complete the R2-3 parent packet.

EXACT ALLOWLIST

Production changes are limited to:

- `crates/world-backend-factory/src/lib.rs`
- `crates/shell/src/execution/invocation/plan.rs`
- `crates/shell/src/execution/platform/macos.rs`
- `crates/shell/src/execution/platform/windows.rs`
- `crates/shell/src/execution/platform_world/mod.rs`
- `crates/shell/src/execution/platform_world/windows.rs`
- `crates/shell/src/builtins/world_gateway.rs`

Test changes are limited to colocated `#[cfg(test)]` modules inside those seven files. No external
fixture or integration test file is authorized. Use mapping-only macOS/Windows focused cases and
the existing gateway differential inside the allowlisted files.

Do not edit any manifest, `Cargo.lock`, backend implementation crate, forwarder, shim, replay,
trace, installer, PowerShell/shell script, macOS/Windows provisioning or lifecycle file, another
Rust file, control document, generated analyzer file, `AGENTS.md`, or `CLAUDE.md`. If another
production file, dependency, feature, or test file is required, stop with
`BLOCKED_SCOPE_EXPANSION`.

EXACT SYMBOL SCOPE

Only these existing production Rust items/functions may change:

- every platform cfg `factory` in `crates/world-backend-factory/src/lib.rs`;
- `ShellConfig::from_args`;
- `ShellConfig::from_cli`;
- macOS `host_doctor_main`;
- macOS `world_doctor_main`;
- Windows `host_doctor_main`;
- Windows `world_doctor_main`;
- `connect_transport_stream_ws`;
- each platform cfg `detect` in `execution/platform_world/mod.rs`;
- Windows `context`;
- Windows `get_backend`;
- Windows `build_agent_client`;
- `build_macos_gateway_client`;
- `resolve_macos_gateway_client_endpoint`;
- `resolve_macos_host_gateway_socket`;
- `macos_default_world_socket_path`; and
- every platform cfg `build_gateway_client`.

The only new production field/symbol authorized is:

- `PlatformWorldContext::bootstrap_mapping`

Add no other public type, trait, constructor, global cache, side table, environment API, selector,
or authority source. Minimum private helpers inside an allowlisted file are permitted only to
validate, clone, or route the already-verified shared mapping and must not construct a second
mapping or broaden the public API. Every other production symbol is byte-frozen except minimum
import/field/signature/call plumbing required by the named items.

PRE-EDIT SOURCE CLOSURE AND IMPACT

Before editing, perform exact source/caller closure across all seven files, their colocated tests,
the landed M4/W5 platform mapping producers/backends, and the future R/R2-3S callers for:

- every cfg `world_backend_factory::factory` definition and call;
- every `MacLimaBackend` and `WindowsWslBackend` constructor reachable from these files;
- every `ShellConfig::from_args`/`from_cli` producer and platform mapping field flow;
- every `PlatformWorldContext` constructor/destructure/clone and `detect` caller;
- every doctor entry and reported mapping/control-root/transport field;
- every platform-world backend/client/transport construction;
- every gateway client builder, macOS endpoint/socket resolver, Windows agent-client builder, and
  fallback/default branch;
- every read of `SUBSTRATE_HOME`, `SUBSTRATE_ROOT`, mapping/carrier variables, `HOME`,
  `LIMA_HOME`, `LOCALAPPDATA`, `USERPROFILE`, pipe/socket/default endpoint, CWD, executable path,
  repository path, or backend auto-selector;
- every call that could launch forwarding, realize/provision a VM/distro, open a listener, mutate
  a socket/pipe, kill/wait a child, delete state, or cross an R3 lifecycle boundary; and
- all Linux callers and behavior that must remain platform-independent and byte/semantic stable.

Run file-qualified upstream GitNexus impact with tests included for every named existing Rust
symbol. Record direct callers, affected processes/modules, and risk. Treat platform constructors,
`ShellConfig` construction, `PlatformWorldContext`, gateway builders, and doctor/client selection
as HIGH/CRITICAL even when cfg indexing reports LOW or omits them. Warn before proceeding. A
missing caller, new authority root, platform lifecycle reachability, gateway contract change, or
Linux semantic change stops F.

TYPED FACTORY CONTRACT

`world-backend-factory::factory` must consume the existing shared typed
`PlatformBootstrapMappingV1` projection on macOS and Windows. The projection must already be
validated against its authenticated host carrier and live platform instance by its owning adapter;
the factory revalidates the mapping fields required by the selected backend and never reconstructs
host context, guest home/principal, platform control root, instance, socket, pipe, or commitment
from ambient state.

On macOS:

- the mapping must be Lima/Lima;
- the backend receives the exact VM identity, account-database-realized guest principal/home,
  committed Lima control root, host socket under A, `/run/substrate.sock`, and commitment;
- contextless construction and transport auto-selection are unreachable;
- the validated product path preserves the explicit R3 forwarding-lifecycle prerequisite and
  fails before forwarding launch, socket create/unlink, SSH bind, timeout kill/wait, or handle
  teardown.

On Windows:

- the mapping must be WSL/WSL;
- the backend receives the exact registered distro, machine identity, guest principal/home,
  token-Known-Folder control root, normalized PM pipe, `/run/substrate.sock`, and commitment;
- contextless/default/env pipe or distro construction is unreachable; and
- no guarded provisioning, process replacement, listener lifecycle, stop/unregister, PID removal,
  or deletion becomes reachable.

Linux keeps its existing platform-independent `factory` behavior because it selects no guest
mapping, home, socket, or pipe. Do not force a dummy platform mapping onto Linux, change its
backend type, routing, errors, tests, or public behavior. Unsupported targets remain fail-closed
with their existing semantics.

SHELL CONFIGURATION AND MAPPING FLOW

`ShellConfig::from_args` and `ShellConfig::from_cli` may only accept, validate, preserve, and
transport the explicit typed mapping already supplied by the authenticated platform entry.
They may not:

- decode environment as a new public constructor;
- select A from home/root/environment;
- infer a platform instance, control root, guest home/principal, host socket, or pipe;
- call a contextless platform factory; or
- silently omit a required mapping on macOS/Windows.

Existing checked carrier/mapping environment projections may be consumed only after the owning
entry has authenticated and canonically bound them; a conflicting or partial projection fails
before backend/client/doctor selection. Linux and no-world paths remain unchanged.

`PlatformWorldContext::bootstrap_mapping` is the sole new field. On macOS/Windows it holds the
exact verified mapping used by the selected backend/transport/client. It is not optional authority
that downstream code may replace from defaults. On Linux it remains absent/non-applicable without
changing Linux semantics. Every constructor and consumer in the exact allowlist must handle the
field explicitly so no hidden contextless platform fallback remains.

PLATFORM AND DOCTOR CALLERS

The macOS and Windows `host_doctor_main`/`world_doctor_main` paths consume the same verified
mapping as the selected backend/client. Diagnostic fields must report the exact host commitment,
platform kind/instance, host platform-control root, guest identity/home, and transport identity
already present in PM. Missing, partial, conflicting, or mixed mapping data is unavailable/
incoherent and cannot report healthy success.

macOS doctor and transport paths must report the future A-scoped SSH-UDS mapping and the explicit
R3 lifecycle prerequisite without launching or cleaning a forwarder. Available VSock/TCP, ambient
endpoint, `HOME`, or `LIMA_HOME` cannot replace the mapping. Windows doctor/client paths use the
exact PM distro/pipe/control-root/commitment; `LOCALAPPDATA`, `USERPROFILE`, target/config/env
defaults, or another pipe cannot select.

`connect_transport_stream_ws`, each platform `detect`, Windows `context`/`get_backend`/
`build_agent_client`, and related callers must carry the same mapping to factory/backend/client
construction. A macOS/Windows caller without explicit verified bootstrap mapping fails before
backend, client, socket, pipe, transport, or diagnostic selection.

GATEWAY CONTRACT FREEZE

`build_macos_gateway_client`, `resolve_macos_gateway_client_endpoint`,
`resolve_macos_host_gateway_socket`, `macos_default_world_socket_path`, and every platform
`build_gateway_client` may change only to consume the verified `PlatformWorldContext` mapping.
The gateway uses PM's platform transport identity; it may not rediscover a socket/pipe from
ambient home, a hard-coded default, config, or backend auto-selection.

Preserve the existing gateway differential and all configuration, effective-policy, network
policy, runtime-family inventory, credential/Codex-home, disabled routing, error classification,
REST/WebSocket, timeout, and request semantics. F does not reopen R2-2E/F or add endpoint
capability. On macOS, if the R3 prerequisite keeps product forwarding inactive, fail with the
existing honest unavailable/prerequisite posture before client construction. On Windows, use the
exact PM pipe/client projection without provisioning or lifecycle action.

LIFECYCLE AND CAPABILITY FREEZE

R3 exclusively owns Lima forwarding activation, socket unlink/bind cleanup, timeout kill/wait,
handle drop teardown, Windows process replacement/timeout kill, stop/unregister, PID/artifact
deletion, rollback, convergence, and ownership manifests. F may transport identity only.

Preserve byte/semantics for all backend lifecycle, forwarding, VM/WSL realization, listener,
stream, process wait/kill, cleanup, retry, convergence, policy, and capability code outside the
named call plumbing. The WSL warm/provision guards remain unreachable and unchanged. No test may
perform a real forwarding, provisioning, listener, pipe/socket mutation, process, or deletion
action.

TEST-DRIVEN PROOF

Use deterministic typed fixtures, mocks/fakes, and temporary sandboxes. Cover at least:

- macOS/Windows factory construction succeeds only with the exact verified shared mapping;
- missing, partial, malformed, noncanonical, wrong-commitment, wrong-platform/transport,
  wrong-instance, wrong-control-root, wrong-guest-principal/home, wrong-host-socket/pipe, and
  wrong-guest-socket mapping rejection before backend/client selection;
- all `ShellConfig` and `PlatformWorldContext` constructors/consumers retain one exact mapping;
- contextless macOS/Windows factory and platform caller paths fail closed;
- macOS backend/doctor/gateway preserve the fixed future SSH-UDS mapping and R3 prerequisite
  without forwarding action;
- Windows backend/doctor/gateway consume the exact registered distro, normalized pipe, guest
  socket, control root, and commitment without ambient reselection;
- conflicting `HOME`, `LIMA_HOME`, `LOCALAPPDATA`, `USERPROFILE`, substrate home/root, pipe,
  distro, target, endpoint, config, CWD, executable, or repository values cannot retarget;
- gateway configuration/policy/network/credential/error/request behavior is unchanged;
- Linux factory/platform-world/gateway behavior and tests are byte/semantic equivalent;
- unsupported targets remain fail-closed; and
- no test reaches forwarding, provisioning, listener, process, timeout kill, stop/unregister,
  PID/artifact deletion, or convergence.

REQUIRED VERIFICATION

Run:

- `cargo fmt --all -- --check`;
- `cargo check --locked -p world-backend-factory`;
- `cargo test --locked -p world-backend-factory -- --nocapture`;
- `cargo clippy --locked -p world-backend-factory --all-targets -- -D warnings`;
- `cargo check --locked -p shell`;
- focused shell tests for every changed `ShellConfig`, platform, doctor, platform-world, and gateway
  path, plus the unchanged gateway differential;
- `cargo clippy --locked -p shell --all-targets -- -D warnings`, preserving and classifying only
  exact pre-existing failures in untouched forbidden code and running the narrowest subject
  supplement without weakening the required command;
- external temporary macOS and Windows cfg harnesses that copy only the allowlisted subject plus
  minimum unchanged dependency surfaces, expose each platform path on Linux, stub every backend
  lifecycle/client transport/process action, and execute the complete typed mapping/failure-order
  proof without changing tracked files;
- honest macOS and Windows target `cargo check`/test-no-run attempts when installed targets and
  cross-toolchains permit, recording SDK/linker unavailability rather than treating it as product
  regression or success;
- exact source assertions proving no contextless platform factory or caller remains reachable,
  PM supplies every platform backend/client/doctor/gateway value, Linux parity, ambient/default
  non-authority, and no lifecycle activation;
- exact frozen-block hashes for gateway policy/config/network/credential/request behavior and all
  platform forwarding/provisioning/listener/process/cleanup branches adjacent to the named calls;
- `git diff --check` and staged `git diff --cached --check`;
- exact unstaged and staged production/test allowlist and symbol containment;
- confirmation that every manifest, lockfile, backend implementation, forwarder, shim, replay,
  trace, script, control document, and generated artifact is unchanged; and
- `gitnexus_detect_changes()` before commit, with every reported flow inspected.

Linux-native cargo commands that compile out macOS/Windows code are baseline checks only and cannot
satisfy focused F proof by themselves. The external cfg harnesses and honest cross-target/static
checks must cover the platform subjects. Formal native platform evidence remains owned by R2-3Z
and must not be claimed here.

SUBJECT FINGERPRINT

After deterministic formatting/checks and before discovery review:

1. record the pre-edit base commit;
2. build a sorted manifest containing that commit plus every exact changed subject path, its Git
   mode (or `NEW`), and `git hash-object --no-filters` blob ID (or `MISSING`);
3. include only paths inside the exact F production/test allowlist; and
4. SHA-256 the manifest and use `sha256:<digest>` as the review subject fingerprint.

BOUNDED REVIEW

Persist and validate one actual V1 review-cycle JSON record with packet ID `A1.1d-5R2-3F` using
`llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py`. The record path in the
terminal receipt must point to that exact JSON file and its SHA-256 must match. Use fresh read-only
lenses for:

1. typed factory signature/projection, complete macOS/Windows caller closure, mapping canonicality,
   contextless failure ordering, and Linux parity;
2. shell/platform/doctor/platform-world/gateway propagation, exact mapping diagnostics,
   ambient/default non-authority, gateway contract preservation, and no capability activation; and
3. exact file/symbol/test allowlist, frozen lifecycle/gateway semantics, cfg-harness sufficiency,
   GitNexus undercoverage, and static/native-proof honesty.

Apply the common causal bounded-review contract. P1/P2 block. Valid unfixed P3/P4 must be
deduplicated or added to `06` only through the separately authorized inventory path. The final
record must validate, end CLEAN, have zero open P1/P2, and completely dispose P3/P4. CLEAN is
terminal; do not launch another review after CLEAN.

LANDING GATES

Before commit, require:

- exact F production/test allowlist and symbol containment;
- all required checks or honestly preserved pre-existing/environment limitations;
- independently recomputable subject fingerprint;
- validated CLEAN V1 review record with zero open P1/P2 and complete P3/P4 disposition;
- `gitnexus_detect_changes()` with every reported flow inspected;
- clean staged diff and no generated/analyzer churn; and
- live target ref still equals the expected base commit/tree.

Then create one Conventional Commit, normal fast-forward push `HEAD` to the target ref, verify the
live remote equals the landed commit/tree, verify 0 ahead/0 behind, restore only analyzer-count
churn if caused by authorized GitNexus refresh, and finish with a clean task worktree. Never force
push.

TERMINAL RECEIPT

Send exactly one `codex.top-level-task-receipt.v1` message to the bound meta task as the last
external action. For success, use `LANDED_CLEAN` and include the bound increment-task thread/host
IDs and nonce, expected base, landed commit/tree/live ref, sorted changed paths, independently
recomputable subject fingerprint, actual validated V1 review record path and digest, finding
disposition, exact checks and limitations, GitNexus result, clean/0-0 publication status,
completion/non-claim boundary, and `next_increment=R2-3R`.

For failure, send the exact blocked status, evidence, required authority or platform, and a
copy-ready handoff prompt. Do not begin, render, or dispatch R2-3R.

COMMON TERMINAL CONTRACT

Before publication:

1. Complete every increment-specific check and proof gate.
2. Verify the exact file/symbol/test allowlist.
3. Run `gitnexus_detect_changes()` and inspect every affected flow.
4. Complete and validate the bounded review sequence.
5. Require zero open blocking findings.
6. Fetch/query the live target again and require it still equals the expected base.

When authorized by the increment contract, create one atomic commit and perform a normal
fast-forward push of `HEAD` to refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap. Never force-push. Verify the live remote equals the
landed commit, refresh GitNexus, restore analyzer-only generated count changes if necessary, and
finish clean.

Send a `codex.top-level-task-receipt.v1` message to the meta task. For success, use
`LANDED_CLEAN` and include your bound increment-task thread/host IDs, expected base, landed
commit/tree, changed paths, subject fingerprint, validated review record and digest, finding
disposition, checks, GitNexus result, clean status, and next increment.

For failure, send the exact blocked status, evidence, required authority or platform, and a
complete continuation/handoff prompt.

The `send_message_to_thread` call is your final tool action. After it succeeds, make no more tool
calls or repository changes. Return only the human-readable final report.

Do not generate the next increment prompt and do not begin R2-3R. The meta
orchestrator owns independent verification and subsequent dispatch.
