Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3M2. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: 3e6f4eddbb1670f9148190565046e1be8f517fc1f89970109228bf1c7a07e7ed
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3M2
- packet_id: A1.1d-5R2-3M2
- next_increment: R2-3M3

INCREMENT-TASK IDENTITY BARRIER

Do not edit, delegate, run implementation checks, or publish until the meta orchestrator sends a
follow-up binding your own real increment-task thread ID and host ID to this dispatch nonce. Echo
those exact IDs in every terminal receipt.

REPOSITORY BOUNDARY

Work only in the task-assigned checkout:

the task-assigned Codex worktree

Never mutate these protected checkouts:

- /home/spenser/__Active_code/substrate
- /home/spenser/__Active_code/substrate-r2-3
- /home/spenser/__Active_code/substrate-r2-3-meta-orchestration

Canonical starting state:

- remote: origin
- target ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
- expected base commit: 1b182a70523a5aa3001ed8a113addaf3f28f9f2d
- expected base tree: b76781c9aca1734a331aacb081baf2acac1fb497
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

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3M2

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
subagent must use GPT-5.4 with Extra High reasoning. Ground decisions in current repository truth,
official Rust/Lima documentation where external behavior matters, and the published
runtime-refactor control pack. Do not reopen R2-2 or R2-3A through R2-3M1 without a concrete
contradiction.

Load only:

- current status in `llm-last-mile/runtime-refactor/00-README.md`
- the parent A1.1d-5R2-3 row, common child-subdivision rules, and R2-3M2 section in
  `llm-last-mile/runtime-refactor/03-phase-slice-map.md`
- PI-041, PI-054–PI-056, PI-075, and PI-090–PI-091 current-state and required-action rows in
  `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`
- the host-context boundary-carrier, platform-mapping construction/verification,
  mapping/factory, generated-projection/diagnostic, and common bounded-review contracts in
  `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`
- R2-MAP-MAC-01 and R2-DIAG-01 in
  `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`
- `06-review-finding-inventory.md` only for actual P3/P4 deduplication

SELECTED OUTCOME

Make the macOS Lima backend primitives consume an explicit verified
`InstallBootstrapContextV1`/`PlatformBootstrapMappingV1` projection, including the exact typed
Lima control root, fixed future host/guest socket identity, and scrubbed `limactl` child
environment. Construction must select no prefix, home, VM, socket, or transport from ambient state.

This increment prepares typed backend primitives only. It does not migrate factory/shell/shim/
replay callers and does not activate forwarding.

EXACT COMPLETION CLAIM

R2-3M2 supplies only the typed macOS backend-primitives prerequisite for PI-041, PI-054–PI-056,
PI-075, and PI-090–PI-091. It completes none of those cross-caller PI rows, no product transport,
no native macOS proof, and no R2-3 parent gate.

EXACT ALLOWLIST

Only:

- `crates/world-mac-lima/src/lib.rs`
- `crates/world-mac-lima/src/transport.rs`
- `crates/world-mac-lima/src/limactl.rs`
- `crates/world-mac-lima/src/vm.rs`

Tests may be added only as colocated tests in those four files. Do not edit any manifest,
`Cargo.lock`, forwarding source, factory, shell, shim, replay, scripts, profiles, unit/socket
templates, control-pack documents, generated files, `AGENTS.md`, or `CLAUDE.md`. If another file
is required, stop with `BLOCKED_SCOPE_EXPANSION`.

EXACT EXISTING SYMBOL SCOPE

Only these existing production symbols may be edited:

- `MacLimaBackend::new`
- `MacLimaBackend::new_with_vm_name`
- `MacLimaBackend::ensure_vm_running`
- `MacLimaBackend::get_agent_endpoint`
- `managed_host_socket_path`
- `managed_host_socket_path_from`
- `Transport::auto_select`
- `limactl::path`
- `limactl::command`
- `LimaVM::new`
- `LimaVM::status`
- `LimaVM::ensure_running`
- `LimaVM::start`
- `LimaVM::wait_for_running`
- `LimaVM::exec`
- `LimaVM::info`
- `run_limactl`

Add exactly these production symbols:

- `MacLimaBackend::new_with_mapping`
- `managed_host_socket_path_for_mapping`
- `limactl::command_for_control_root`

Do not add another public or crate-visible production symbol. Private helpers are permitted only
when directly necessary to validate the existing shared types, build identical backend fields, or
keep tests deterministic; they must not become another authority type, mapping constructor, path
normalizer, transport selector, or side table.

Every other production function is byte-frozen, including `ensure_forwarding`,
`forwarding::auto_select`, forwarding constructors/handles/drop, world/replay semantics, policy
logic, client request conversion, and runtime lifecycle outside exact typed argument plumbing in
the named symbols.

PRE-EDIT IMPACT AND SOURCE CLOSURE

Before any edit, run file-qualified upstream GitNexus impact with tests included for every named
existing symbol. Record direct callers, affected processes/modules, and risk. Treat
`MacLimaBackend::new`, `MacLimaBackend::new_with_vm_name`, and `managed_host_socket_path` as
CRITICAL even if cfg-aware indexing under-reports them. Inspect full context for both backend
constructors, `ensure_vm_running`, `get_agent_endpoint`, `Transport::auto_select`, and
`run_limactl`.

Warn before proceeding on HIGH/CRITICAL. Continue only when the work remains a typed-input
primitive change with no new caller, forwarding, lifecycle, or unrelated execution-flow
reachability. A risk ceiling expansion requires a bounded stop, not a silent widening.

TYPED INPUT AND VALIDATION

`MacLimaBackend::new_with_mapping` is the only contract-correct constructor in this increment. It
must receive the already-verified shared IH/PM values or carriers; it must not reconstruct them.
Before allocating backend state it must:

- strictly validate both existing shared records and recompute the IH commitment;
- require `PM.host_context_commitment` to equal the recomputed IH commitment;
- require Lima platform and transport variants;
- require one exact nonempty declared VM name and lowercase 32-hex guest machine ID;
- require a normalized absolute host Lima control root;
- require normalized absolute guest realized home and canonical Unix account+UID;
- require the exact future transport pair
  `<selected_host_prefix>/sock/agent.sock` to `/run/substrate.sock`; and
- reject every mismatch before any `limactl`, socket, forwarding, client, runtime, or filesystem
  action.

Use the existing `transport-api-types` model and normalizers. Do not add a local IH/PM/context
struct, mapping codec, commitment cache, process-global record, side table, or backend-selected
home.

The new constructor may create only in-memory backend fields and its private Tokio runtime. It
must not inspect/start a VM, connect to a socket, create directories, read the filesystem, or
start forwarding during construction.

COMPATIBILITY CONSTRUCTORS

`MacLimaBackend::new` and `new_with_vm_name` may remain for source compatibility only when their
diagnostic/non-product posture is explicit and they cannot satisfy typed mapping proof. They must
not fabricate a PM, call `new_with_mapping` with inferred values, or become a second normal-product
constructor.

Do not migrate any current caller in this increment; caller migration belongs to M4/F/R/S1. Do not
change a caller outside the allowlist merely to make a constructor compile. If exact source
compatibility cannot be retained without an unauthorized caller edit, stop with
`BLOCKED_SCOPE_EXPANSION`.

HOST SOCKET AND TRANSPORT

`managed_host_socket_path_for_mapping` must derive and return exactly the PM-declared host socket
after full IH/PM validation:

```text
<selected_host_prefix>/sock/agent.sock
```

It must never read `SUBSTRATE_HOME`, `HOME`, `LIMA_HOME`, `dirs::home_dir`, CWD, a repository path,
socket existence, or tool availability. A different path, guest socket, platform kind, commitment,
or VM fails closed.

`managed_host_socket_path` and `managed_host_socket_path_from` may remain only as explicitly
diagnostic compatibility primitives. The typed constructor and every typed-path test must use
`managed_host_socket_path_for_mapping`.

The typed backend stores the fixed future `Transport::UnixSocket` identity only.
`Transport::auto_select` may remain only as explicitly diagnostic/non-product compatibility and
must not be called by `new_with_mapping` or any typed path. VSock, TCP, SSH availability, ambient
endpoints, and socket presence are not mapping authority. Do not delete the compatibility variants
or redesign the transport enum.

LIMACTL CONTROL ROOT AND CHILD ENVIRONMENT

`limactl::command_for_control_root` must receive the exact host account-database home and PM
`host_platform_control_root`. It must:

- validate both as normalized absolute paths;
- require the control root to equal `<host_account_database_home>/.lima`;
- remove any inherited `HOME` and `LIMA_HOME` selection;
- set `HOME=<host_account_database_home>`;
- set `LIMA_HOME=<host_platform_control_root>`; and
- construct the same resolved `limactl` executable without launching it.

All `limactl` operations reached from a typed backend or typed `LimaVM` must use this command
constructor. A directly injected conflicting internal projection fails closed. Outer ambient B is
overwritten and cannot select or block A.

`limactl::path` remains executable discovery only; executable lookup cannot choose IH, PM, a Lima
store, VM, socket, or transport. `limactl::command` may remain only as diagnostic compatibility and
must not be used by the typed mapping path.

LIMA VM PRIMITIVES

Thread the exact typed control-root inputs through `LimaVM::new`, `status`, `ensure_running`,
`start`, `wait_for_running`, `exec`, `info`, and `run_limactl` without changing their existing
status/start/wait/exec/info semantics beyond:

- every child uses `command_for_control_root`;
- status/info scope to the exact declared VM and reject malformed/ambiguous output;
- no function guesses a guest home or host control root; and
- any revalidation required by the existing mapping fails before later action.

Do not add create/delete/rebuild/stop/cleanup authority. Existing `LimaVM::stop`, timeout-kill/wait,
retry, and process semantics are frozen unless exact argument plumbing from a named caller is
strictly required; if that requires editing an unnamed symbol, stop. M2 may not use VM lifecycle
execution as proof.

BACKEND ACTION BOUNDARY

`ensure_vm_running` must consume the typed control root and declared instance already stored by the
validated constructor. It cannot read ambient state or select a different instance. Existing
diagnostic compatibility behavior may remain isolated and labeled, but the typed path must reject
mapping inconsistency.

`get_agent_endpoint` must use only the fixed PM host socket identity and must not auto-select a
transport. When the R3 forwarding prerequisite is absent it fails without creating/unlinking a
socket or launching/killing/waiting a child. Do not edit or invoke `ensure_forwarding` as M2 proof.

No new typed constructor is wired into a product caller in this increment. Therefore this change
must add no newly reachable forwarding, VM lifecycle, socket mutation, or product execution flow.
M3 owns the forwarding boundary and explicit R3 prerequisite; M4/F own later caller projection.

TEST-DRIVEN PROOF

Add colocated deterministic tests covering at least:

- valid IH/PM constructs exact VM name, typed Lima control root, fixed `UnixSocket`, and exact
  A-scoped host socket without invoking a child;
- PM commitment, platform kind, transport kind, VM, machine ID, control root, realized
  principal/home, host socket, and guest socket mismatches all reject;
- ambient conflicting `HOME`, `LIMA_HOME`, `SUBSTRATE_HOME`, tool availability, VSock, TCP, and SSH
  do not affect the typed result;
- `command_for_control_root` overwrites exact `HOME`/`LIMA_HOME`;
- typed `LimaVM` commands retain the exact declared VM and mapping environment;
- contextless/auto-select constructors are explicitly diagnostic and cannot satisfy typed proof;
- the typed constructor performs no command, socket, filesystem, or forwarding action;
- `get_agent_endpoint` fails at the unactivated R3 boundary without mutation; and
- existing compatibility tests remain behavior-equivalent where outside the typed product proof.

Tests must not start Lima, launch forwarding, create/unlink a managed socket, kill/wait a child,
exercise `ForwardingHandle::drop`, or claim native macOS evidence. Use command construction and
pure validation observations only.

REQUIRED VERIFICATION

Run:

- `cargo fmt --all -- --check`;
- `cargo check --locked -p world-mac-lima`;
- `cargo test --locked -p world-mac-lima`;
- `cargo clippy --locked -p world-mac-lima --all-targets -- -D warnings`;
- static target-macOS check/test compilation for an already-installed supported Apple Rust target
  when available, recording exact target/SDK/linker availability and without installing or
  simulating a missing platform;
- a focused external compile/test harness when host cfg prevents the colocated macOS module tests
  from compiling on Linux, using only process artifacts outside the tracked checkout;
- `git diff --check`;
- exact four-file unstaged and staged allowlist/status checks;
- confirmation that every manifest, `Cargo.lock`, forwarding file, shell/factory/shim/replay file,
  script, profile, unit/socket template, control document, and generated artifact is unchanged;
- exact-source proof that `new_with_mapping` calls no auto-select/forwarding constructor and no
  typed test performs lifecycle action;
- post-change source/caller closure for every edited existing symbol; and
- `gitnexus_detect_changes()` before commit, with every affected flow inspected.

If the exact clippy command exposes only pre-existing warnings in untouched forbidden files,
preserve the raw failure and run a narrowly scoped subject supplement without weakening the
required command. Do not relabel unavailable native/macOS compilation as a pass.

SUBJECT FINGERPRINT

After deterministic formatting/checks and before discovery review:

1. Record the pre-edit base commit.
2. Build a sorted manifest containing that commit plus each exact allowlisted path, its Git mode,
   and `git hash-object --no-filters` blob ID.
3. SHA-256 the manifest and use `sha256:<digest>` as the review subject fingerprint.

BOUNDED REVIEW

Open and validate one V1 record with packet ID `A1.1d-5R2-3M2`. Use fresh read-only lenses for:

1. IH/PM validation, control-root authority, child-environment scrubbing, and no ambient fallback;
2. backend/socket/transport construction, mapping mismatch rejection, and no forwarding/lifecycle
   activation;
3. exact symbol/file allowlist, caller compatibility, static-proof honesty, and regression tests.

Apply the common causal bounded-review contract. P1/P2 block. Valid unfixed P3/P4 must be
deduplicated/inventoried before completion; if that requires a fifth tracked file, stop with
`AUTHORITY_REQUIRED` for a separate inventory-only action. CLEAN is terminal.

PUBLICATION

After every gate passes and review is CLEAN:

1. Stage only the exact four allowlisted paths.
2. Inspect the staged diff and rerun format, focused tests, allowlist, no-forwarding/lifecycle, and
   frozen-file checks.
3. Commit one atomic Conventional Commit of at most 72 characters.
4. Fetch/query the live product ref and require it still equals the expected base.
5. Perform a normal fast-forward push of `HEAD` to the target ref.
6. Verify live remote equality, refresh GitNexus, restore analyzer-only `AGENTS.md`/`CLAUDE.md`
   count changes to committed bytes, and finish fully clean with 0 ahead/0 behind.

Send a schema-valid `codex.top-level-task-receipt.v1` using the exact nested protocol fields. Do
not create another branch, merge, rebase, force-push, open a PR, or begin/render R2-3M3.

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

Do not generate the next increment prompt and do not begin R2-3M3. The meta
orchestrator owns independent verification and subsequent dispatch.
