Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3M3. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: 1e6274c6bcfb228cb7d9204b5a026820bfefbc4e5db2330303364f6ce0ca5e33
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3M3
- packet_id: A1.1d-5R2-3M3
- next_increment: R2-3M4

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
- expected base commit: 8296a7fce4b377311b3cc74df1a66cdc5a44101e
- expected base tree: e3279fe1dc606f31392116c723f104df09bc137b
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

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3M3

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
official Rust/OpenSSH/Lima documentation where external behavior matters, and the published
runtime-refactor control pack. Do not reopen R2-2 or R2-3A through R2-3M2 without a concrete
contradiction.

Load only:

- current status in `llm-last-mile/runtime-refactor/00-README.md`;
- the parent A1.1d-5R2-3 row, common child-subdivision rules, and R2-3M3 section in
  `llm-last-mile/runtime-refactor/03-phase-slice-map.md`;
- PI-041 and PI-112 current-state and required-action rows, plus frozen PI-101, PI-113, and
  PI-114 ownership rows, in `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`;
- the host-context boundary-carrier, platform-mapping construction/verification, mapping/factory,
  generated-projection/diagnostic, R3-exclusive lifecycle, and common bounded-review contracts in
  `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`;
- R2-MAP-MAC-01 and R2-GEN-01 in
  `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`; and
- `06-review-finding-inventory.md` only for actual P3/P4 deduplication.

SELECTED OUTCOME

Fix the future PM-bound macOS SSH-UDS target and A-scoped known-hosts projection, remove
auto-selection from the validated typed path, and make that path fail before any forwarding child
launch with the explicit R3 forwarding-lifecycle prerequisite.

This increment prepares and proves the macOS forwarding boundary only. It does not activate
forwarding.

EXACT COMPLETION CLAIM

R2-3M3 completes only PI-041 and PI-112. It does not complete or exercise PI-101, PI-113, or
PI-114; does not claim macOS product transport or native macOS evidence; completes no later caller
migration; and does not complete the R2-3 parent packet.

EXACT ALLOWLIST

Only:

- `crates/world-mac-lima/src/forwarding.rs`
- `crates/world-mac-lima/src/transport.rs`
- `crates/world-mac-lima/src/lib.rs`

Tests may be added or changed only as colocated tests in those three files. Do not edit any
manifest, `Cargo.lock`, shell/factory/shim/replay file, script, profile, unit/socket template,
control-pack document, generated file, `AGENTS.md`, or `CLAUDE.md`. If another file is required,
stop with `BLOCKED_SCOPE_EXPANSION`.

EXACT EXISTING SYMBOL SCOPE

Only these existing production symbols may be edited:

- `forwarding::auto_select`
- `create_ssh_uds_forwarding`
- `lima_home_dir`
- `lima_ssh_config_path`
- `Transport::auto_select`
- `MacLimaBackend::ensure_forwarding`
- `MacLimaBackend::get_agent_endpoint`

Add exactly these production symbols:

- `lima_home_dir_for_mapping`
- `lima_ssh_config_path_for_mapping`
- `r3_forwarding_activation_required`

Do not add another public or crate-visible production symbol. Private helpers are permitted only
when directly necessary to validate the existing IH/PM types, construct the exact future argument
projection without launching it, or keep tests deterministic. They must not become another
authority type, mapping constructor, transport selector, lifecycle seam, retry path, or side
table.

Every other production symbol is byte-frozen. In particular:

- `ForwardingHandle::drop` is byte-frozen;
- `create_vsock_forwarding` and the SSH-TCP compatibility constructor are byte-frozen;
- explicit host-socket removal and the SSH `StreamLocalBindUnlink` option inside
  `create_ssh_uds_forwarding` are byte-frozen;
- the SSH timeout `kill`/`wait` branch is byte-frozen;
- forwarding retry, readiness wait, process-group, child teardown, socket cleanup, and capability
  probing semantics are byte-frozen; and
- all backend lifecycle, client, world, policy, replay, and transport behavior outside exact
  typed boundary plumbing in the named symbols is byte-frozen.

PRE-EDIT IMPACT AND SOURCE CLOSURE

Before any edit, run file-qualified upstream GitNexus impact with tests included for every named
existing symbol and for `ForwardingHandle::drop`. Record direct callers, affected processes and
modules, and risk. Inspect full context for `forwarding::auto_select`,
`create_ssh_uds_forwarding`, `Transport::auto_select`,
`MacLimaBackend::ensure_forwarding`, `MacLimaBackend::get_agent_endpoint`, and
`ForwardingHandle::drop`.

Treat the forwarding selectors, backend boundary, SSH constructor, and handle drop as
HIGH/CRITICAL even when macOS cfg indexing under-reports them. Warn before proceeding. Continue
only when the change remains a pure typed future-projection/fail-closed boundary with no new
forwarder, socket, process, retry, teardown, or unrelated execution-flow reachability. A risk
ceiling expansion requires a bounded stop.

TYPED INPUT AND FUTURE TRANSPORT IDENTITY

The already-landed M2 `InstallBootstrapContextCarrierV1` /
`PlatformBootstrapMappingV1` projection is authoritative. Any M3 typed helper must strictly
validate both records, recompute the IH commitment, require a Lima instance and Lima transport,
and reject any mismatch before filesystem observation or child/process action.

For the validated typed mapping:

- the host socket is exactly `<selected_host_prefix>/sock/agent.sock`;
- the guest socket is exactly `/run/substrate.sock`;
- the SSH known-hosts projection is exactly
  `<selected_host_prefix>/lima_known_hosts`;
- the Lima control root is exactly the already-validated PM
  `host_platform_control_root`;
- the VM name is exactly the PM Lima instance name; and
- neither file existence/content nor `HOME`, `LIMA_HOME`, `SUBSTRATE_HOME`, CWD, repository
  location, `ssh`, `vsock-proxy`, TCP availability, or ambient endpoint state may select or
  replace any of those values.

The known-hosts path is a generated A-relative helper projection. It is not an authority source,
mapping record, existence gate, ownership manifest, or cleanup target.

LIMA HOME AND SSH CONFIG PROJECTION

`lima_home_dir_for_mapping` and `lima_ssh_config_path_for_mapping` must consume and validate the
exact typed IH/PM inputs. They must use only PM `host_platform_control_root`, the exact PM VM name,
and the fixed `ssh.config` suffix. They must not read process environment, default host home,
`dirs::home_dir`, or a repository path. A conflicting commitment, platform/transport kind, VM,
control root, host socket, guest socket, or selected prefix fails closed.

`lima_home_dir` and `lima_ssh_config_path` may remain only as explicitly labeled
diagnostic/non-product compatibility. They cannot satisfy typed mapping or product proof.
Compatibility behavior outside the typed path must remain equivalent.

FORWARDING CONSTRUCTOR BOUNDARY

`create_ssh_uds_forwarding` may receive exact future typed socket, SSH-config/control-root, and
known-hosts parameters so its future command projection is unambiguous. The validated normal path
must not call it in R2-3M3.

Do not execute or alter the existing lifecycle actions inside that constructor. In particular,
M3 may not:

- create the host socket directory;
- create, unlink, replace, or remove a socket;
- set or exercise `StreamLocalBindUnlink`;
- spawn `ssh`, `vsock-proxy`, or any forwarding child;
- sleep, retry, probe readiness, kill, or wait on a child; or
- construct or drop an active `ForwardingHandle`

as typed-path behavior or proof.

The existing VSock, SSH-UDS, and SSH-TCP constructors may remain reachable only from explicitly
labeled direct diagnostic/test compatibility paths. They cannot satisfy IH/PM, normal-product,
R2-MAP-MAC-01, or native mapping proof.

EXPLICIT R3 PREREQUISITE

`r3_forwarding_activation_required` is the single exact fail-closed boundary used by the
validated typed backend. It must return an explicit error identifying the unmet R3 forwarding
lifecycle prerequisite before any forwarding lock mutation, auto-selection, command construction,
filesystem/socket observation, child spawn, retry, kill/wait, or handle creation/drop.

`MacLimaBackend::ensure_forwarding` must distinguish the already-landed typed mapping posture from
diagnostic compatibility. For the typed posture it calls only the R3-prerequisite failure boundary
and cannot call `forwarding::auto_select` or any forwarding constructor. Existing contextless
diagnostic behavior may remain only explicitly labeled and cannot satisfy product proof.

`MacLimaBackend::get_agent_endpoint` must use only the already-stored PM-fixed future
`Transport::UnixSocket` identity for typed state, but it must fail at the same R3 prerequisite
until activation exists. It cannot auto-select, probe, create/unlink a socket, launch a child, or
return a VSock/TCP endpoint for typed state.

`forwarding::auto_select` and `Transport::auto_select` remain diagnostic compatibility only. No
validated typed caller may invoke them, and tool availability cannot influence the typed result.
Do not delete compatibility variants or redesign either transport enum.

R3 LIFECYCLE FREEZE

PI-101, PI-113, and PI-114 remain R3-exclusive. M3 transports only the exact future target and
known-hosts projection. It grants no replacement, rollback, current-attempt timeout, child
termination, handle-drop teardown, retry, ownership-manifest, socket cleanup, or convergence
authority.

Before and after editing, capture exact base snippets or hashes for:

- the full `ForwardingHandle::drop` body;
- the explicit pre-launch socket-removal block;
- the `StreamLocalBindUnlink=yes` SSH option; and
- the full timeout `kill`/`wait` block.

Require those frozen bytes to remain identical to the expected base. A required lifecycle-byte
change stops as `BLOCKED_SCOPE_EXPANSION`; do not repair it.

TEST-DRIVEN PROOF

Add or strengthen colocated deterministic tests covering at least:

- valid IH/PM produces exactly `A/sock/agent.sock` to `/run/substrate.sock`;
- the typed known-hosts projection is exactly `A/lima_known_hosts`;
- typed Lima home/SSH-config paths use the PM control root and exact PM VM;
- commitment, platform kind, transport kind, VM, control root, host socket, guest socket, and
  selected-prefix mismatches reject;
- conflicting ambient `HOME`, `LIMA_HOME`, `SUBSTRATE_HOME`, CWD, PATH, SSH, VSock, TCP, and
  endpoint state do not affect typed projections;
- typed `ensure_forwarding` and `get_agent_endpoint` fail with the explicit R3 prerequisite before
  command spawn, filesystem/socket mutation, lock-state mutation, or handle creation;
- `forwarding::auto_select`, `Transport::auto_select`, and every compatibility constructor are
  excluded from the typed call path; and
- the exact frozen R3 lifecycle bytes remain unchanged.

No test may activate forwarding, create/unlink the managed socket, set/exercise SSH-side unlink,
kill/wait a forwarding child, exercise `ForwardingHandle::drop`, or claim native macOS evidence.
Existing forwarding tests are compatibility/regression evidence only and may not count as
normal-product proof.

REQUIRED VERIFICATION

Run:

- `cargo fmt --all -- --check`;
- `cargo check --locked -p world-mac-lima`;
- `cargo test --locked -p world-mac-lima`;
- `cargo clippy --locked -p world-mac-lima --all-targets -- -D warnings`;
- static target-macOS check/test compilation for an already-installed supported Apple Rust target
  when available, recording exact target/SDK/linker availability and without installing or
  simulating a missing native toolchain;
- when crate-wide macOS cfg prevents the colocated module/tests from compiling on Linux, a focused
  external compile/test harness outside the tracked checkout using the current exact three-file
  subject and preserving production source bytes except removal of the crate-level host cfg;
- `git diff --check` and staged `git diff --cached --check`;
- exact three-file unstaged and staged allowlist/status checks;
- confirmation that every manifest, `Cargo.lock`, shell/factory/shim/replay/forwarder-consumer
  file, script, profile, unit/socket template, control document, and generated artifact is
  unchanged;
- exact-source/call-graph proof that no typed path calls auto-selection or a forwarding
  constructor and no typed test performs lifecycle action;
- exact frozen-byte comparison for every named R3 lifecycle block;
- post-change source/caller closure for every edited existing symbol; and
- `gitnexus_detect_changes()` before commit, with every affected flow inspected.

If native Apple compilation is unavailable because this Linux host lacks the Apple SDK/toolchain,
preserve the raw failure and report it as unavailable static compilation, not as a pass or native
evidence. If the exact clippy command exposes only pre-existing warnings in untouched forbidden
files, preserve the raw failure and run a narrowly scoped subject supplement without weakening the
required command.

SUBJECT FINGERPRINT

After deterministic formatting/checks and before discovery review:

1. record the pre-edit base commit;
2. build a sorted manifest containing that commit plus each exact allowlisted path, its Git mode,
   and `git hash-object --no-filters` blob ID; and
3. SHA-256 the manifest and use `sha256:<digest>` as the review subject fingerprint.

BOUNDED REVIEW

Open and validate one V1 review record with packet ID `A1.1d-5R2-3M3`. Use fresh read-only lenses
for:

1. IH/PM validation, exact future socket/control-root/SSH-config/known-hosts projection, and
   ambient non-authority;
2. explicit R3 prerequisite, no auto-selection/forwarder activation, and byte-identical
   PI-101/PI-113/PI-114 lifecycle regions; and
3. exact symbol/file allowlist, compatibility preservation, static-proof honesty, and regression
   sufficiency.

Apply the common causal bounded-review contract. P1/P2 block. Valid unfixed P3/P4 must be
deduplicated/inventoried before completion; if that requires a fourth tracked file, stop with
`BLOCKED_SCOPE_EXPANSION`. Validate the record after every returned cycle. A CLEAN cycle is
terminal; do not launch another review after CLEAN.

PUBLICATION

Before publication require:

- exact staged allowlist and frozen-lifecycle-byte proof;
- every required check and change-detection result;
- validated CLEAN review with zero P1/P2 and complete P3/P4 disposition;
- live target still equal to the expected base;
- one atomic Conventional Commit;
- normal fast-forward push only; and
- post-push live equality, clean checkout, current GitNexus index, and 0 ahead/0 behind.

The successful completion claim must say that M3 completes only PI-041 and PI-112 by fixing the
future typed macOS forwarding boundary and explicit R3 prerequisite. It must explicitly disclaim
forwarding activation, R3 lifecycle ownership, native macOS evidence, macOS product transport,
later caller migration, and R2-3 parent completion.

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

Do not generate the next increment prompt and do not begin R2-3M4. The meta
orchestrator owns independent verification and subsequent dispatch.
