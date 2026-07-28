Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3M4. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: 06e074065ef97155bf99e8efba635cb0668c14d0938d17b49a1f1582e5e2dbab
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3M4
- packet_id: A1.1d-5R2-3M4
- next_increment: R2-3W1

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
- expected base commit: 4325e7aa085ed1f5eeeb6535e0fc436a3663c3c2
- expected base tree: 60fa3b29df3abc6bf47e4539f52ec11230acebf9
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

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3M4

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
official Rust/systemd/Lima documentation where external behavior matters, and the published
runtime-refactor control pack. Do not reopen R2-2 or R2-3A through R2-3M3 without a concrete
contradiction.

Load only:

- current status in `llm-last-mile/runtime-refactor/00-README.md`;
- the parent A1.1d-5R2-3 row, common child-subdivision rules, and R2-3M4 section in
  `llm-last-mile/runtime-refactor/03-phase-slice-map.md`;
- PI-009, PI-022, PI-040, PI-055, PI-059, and PI-075 current-state and required-action rows in
  `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`;
- the host-context boundary-carrier, platform-mapping construction/verification, mapping/factory,
  generated-projection/diagnostic, R3-exclusive lifecycle, and common bounded-review contracts in
  `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`;
- R2-MAP-MAC-01, R2-GEN-01, and R2-DIAG-01 in
  `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`; and
- `06-review-finding-inventory.md` only for actual P3/P4 deduplication.

SELECTED OUTCOME

Carry the already-selected IH/PM through the dev/release macOS installer call sections, generated
guest unit/socket projections, and host doctor/invocation surfaces without activating forwarding
or changing lifecycle ownership.

EXACT COMPLETION CLAIM

R2-3M4 completes only PI-009, PI-022, and PI-040. It supplies prerequisites for PI-055 and PI-075,
which R2-3F completes, and may update PI-059 only as a harness call site. It does not activate
forwarding, complete a platform factory/caller migration, claim native macOS evidence or macOS
product transport, take R3 lifecycle ownership, or complete the R2-3 parent packet.

EXACT ALLOWLIST

Production changes are limited to:

- macOS call sections only in `scripts/substrate/dev-install-substrate.sh`
- macOS call sections only in `scripts/substrate/install-substrate.sh`
- mapping/projection-only sections in `scripts/mac/lima-warm.sh`
- only the canonical expected-unit `envsubst` projection inside
  `scripts/mac/lima-doctor.sh::check_rendered_unit_parity`
- `scripts/mac/lima/units/substrate-world-service.service.tmpl`
- `scripts/mac/lima/units/substrate-world-service.socket`
- exact named symbols in `crates/shell/src/execution/invocation/plan.rs`
- exact named symbols in `crates/shell/src/execution/platform/macos.rs`
- call-site-only `main` in `crates/world-mac-lima/examples/mac_backend_smoke.rs`

Test changes are limited to mapping/context-only cases in:

- `tests/mac/installer_parity_fixture.sh`
- `tests/mac/lima_doctor_fixture.sh`
- `tests/mac/prefix_mapping_r2_3.sh`
- colocated tests in the two allowlisted Rust source files

Do not edit any manifest, `Cargo.lock`, non-macOS installer section, Lima base profile, stop/smoke
runner, any other `lima-doctor.sh` statement or symbol, world-mac-lima library source,
factory/shim/replay/forwarder source, Windows source, control-pack document, generated analyzer
file, `AGENTS.md`, or `CLAUDE.md`. If another file or production symbol is required, stop with
`BLOCKED_SCOPE_EXPANSION`.

EXACT EXISTING SYMBOL SCOPE

Only these existing production symbols may be edited:

- `provision_macos_world`
- `install_macos`
- the dev installer's macOS `lima-warm.sh` call section
- `write_systemd_units`
- `configure_guest`
- only the canonical expected-unit `envsubst` projection statements in
  `check_rendered_unit_parity`
- `ShellConfig::from_args`
- `ShellConfig::from_cli`
- `host_doctor_main`
- `world_doctor_main`
- `resolve_lima_vm_name`
- `selected_host_visible_transports`
- `try_bootstrap_host_visible_transport`
- `collect_world_doctor_assessment`
- the example `main`

Add no new production symbol. Private test helpers may be added only inside the allowed test
surfaces and must not become an authority source, mapping constructor, ambient fallback,
transport selector, lifecycle seam, cleanup path, or side table.

Every other production symbol is byte-frozen. In particular:

- cleanup statements inside `write_systemd_units` and `configure_guest` are byte-frozen;
- every `check_rendered_unit_parity` statement except the exact environment assignments on its
  canonical service-unit `envsubst` invocation is byte-frozen;
- instance delete/rebuild, staged-tree/temp cleanup, legacy unit/socket deletion, runtime-socket
  cleanup, forwarding socket unlink, `StreamLocalBindUnlink`, child kill/wait, handle drop,
  retry, teardown, and convergence are byte-frozen;
- the current Lima base profile and every stop/smoke lifecycle runner are byte-frozen;
- diagnostic VSock/SSH-TCP compatibility does not become product authority; and
- policy, replay, world capability, transport protocol, and non-macOS behavior are byte-frozen.

PRE-EDIT IMPACT AND SOURCE CLOSURE

Before any edit, run file-qualified upstream GitNexus impact with tests included for every named
existing Rust symbol. Record direct callers, affected processes/modules, and risk. Inspect full
context for `ShellConfig::from_cli`, `host_doctor_main`, `world_doctor_main`,
`selected_host_visible_transports`, `try_bootstrap_host_visible_transport`,
`collect_world_doctor_assessment`, and the example `main`.

For shell functions and installer call sections, including `check_rendered_unit_parity`, perform
exact caller/source closure before edit and record every invocation and branch that can reach
them. Treat `ShellConfig::from_cli`, doctor transport selection, backend construction, and both
sides of unit projection/parity as HIGH posture even if cfg or shell indexing under-reports them.
Warn before proceeding. Continue only when the scope remains typed IH/PM propagation and
generated projection with no new authority, transport activation, lifecycle reachability, or
non-macOS behavior.

AUTHORITATIVE INPUT AND PROPAGATION

The already-landed `InstallBootstrapContextCarrierV1` and `PlatformBootstrapMappingV1` are the
only authority. M4 must preserve their strict validation and exact commitment. Public
dev/release installers may select A only through their already-authorized prefix/context rules,
construct or receive the exact IH, construct/verify the exact Lima PM, and carry those values on
the explicit child boundary into `lima-warm.sh`.

The macOS call sections must:

- pass the exact already-selected prefix/context/mapping values;
- prevent outer `HOME`, `LIMA_HOME`, `SUBSTRATE_HOME`, CWD, repository location, PATH, VM name,
  socket path, or transport availability from overriding them;
- reject a malformed, duplicate, tampered, forged-principal, wrong-instance, wrong-control-root,
  wrong-host-socket, wrong-guest-socket, or mismatched commitment before projection or action;
- preserve non-macOS branches byte-for-byte; and
- never log or render the authenticated carrier or sensitive principal material.

Do not introduce another carrier, environment authority, context reconstruction path, helper-local
default, or compatibility fallback.

GUEST UNIT AND SOCKET PROJECTION

`write_systemd_units` and `configure_guest` may project only fields already verified from IH/PM:

- the exact host commitment and Lima instance identity where the frozen contract requires them;
- the exact guest service/socket identity;
- `/run/substrate.sock` as the fixed future V1 guest socket; and
- the exact checked service environment required by the existing template contract.

The service and socket templates are generated projections, never selectors or authority records.
Their existence/content cannot select A, reconstruct PM, authorize cleanup, or activate
forwarding. Do not add a second socket, alternate target, ambient home/control-root lookup,
ownership manifest, or lifecycle marker.

The sole authorized `lima-doctor.sh` expansion must make the canonical expected service-unit
render receive the same already-verified values used by `write_systemd_units`:
`INSTALL_BOOTSTRAP_COMMITMENT`, `VM_NAME`, `HOST_PLATFORM_CONTROL_ROOT`,
`OBSERVED_TRANSPORT_HOST`, and `OBSERVED_TRANSPORT_GUEST_SOCKET`. It may add only the matching
five environment assignments immediately around the existing service-template `envsubst` call.
It may not alter mapping observation/verification, readiness, comparison, hashing, warnings,
recovery hints, cleanup, lifecycle behavior, or any other doctor statement. This is parity
projection only, not a new authority or transport source.

Before and after editing, capture exact base snippets or hashes for all cleanup statements within
`write_systemd_units` and `configure_guest`. Those bytes must remain identical. M4 may render or
compare the allowed verified projection but may not delete/replace legacy units, remove a runtime
socket, clean staging/temp trees, restart/stop an instance, or perform convergence.

SHELL INVOCATION AND DOCTOR PROJECTION

`ShellConfig::from_args` and `ShellConfig::from_cli` may carry the already-validated IH/PM only
through the exact macOS invocation/doctor path. They may not select A, infer a host home, consult
ambient state, widen a public CLI, or change non-macOS branches.

`resolve_lima_vm_name`, `selected_host_visible_transports`,
`try_bootstrap_host_visible_transport`, `host_doctor_main`, `world_doctor_main`, and
`collect_world_doctor_assessment` must consume the exact verified mapping and report:

- the exact PM Lima instance;
- the account-database-derived Lima control root already committed by PM;
- the A-scoped future host socket and `/run/substrate.sock` guest target;
- agreement between installer/unit/doctor commitment and mapping; and
- the explicit unmet R3 forwarding-lifecycle prerequisite.

Doctor output is diagnostic projection only. It must not launch or clean a forwarder, create or
unlink a socket, probe ambient transports as authority, silently fall back to VSock/TCP, mutate
units/instance state, or claim product transport. Any existing display of diagnostic transport
availability must remain clearly non-authoritative and excluded from PM/product proof.

The example `main` may change only enough to pass the already-verified typed context/mapping into
the current constructor surface and compile. It must not start forwarding or count as native or
product proof.

R3 LIFECYCLE AND FORWARDING FREEZE

M4 has no forwarding or cleanup authority. The already-landed M3 fail-closed R3 prerequisite must
remain the boundary before command spawn, forwarding lock mutation, filesystem/socket mutation,
child creation, retry, kill/wait, or handle creation/drop.

No M4 path or test may:

- call a forwarding constructor from the validated typed path;
- auto-select VSock, SSH-TCP, or any alternate product transport;
- create, unlink, replace, or remove the managed socket;
- set or exercise SSH-side unlink;
- start/stop/destroy/rebuild a Lima instance outside the existing M1 realization boundary;
- remove/replace units, sockets, staged trees, or temp paths;
- kill/wait a forwarding child or exercise handle-drop cleanup; or
- claim that the R3 prerequisite has been satisfied.

TEST-DRIVEN PROOF

Add or strengthen deterministic allowed tests covering at least:

- dev and release macOS call sections propagate the same authenticated IH/PM selected at the
  parent and reject outer B overrides;
- malformed, duplicate, tampered, forged-principal, and mismatched prefix/commitment/instance/
  control-root/socket carriers fail before projection or action;
- unit and socket projections use only the exact verified commitment/mapping/service/socket fields;
- the doctor canonical expected-unit render supplies exactly the same five verified mapping fields
  as `write_systemd_units`, without changing any other doctor behavior;
- the service and socket templates agree with doctor output on commitment, instance, control root,
  A-scoped future host socket, and `/run/substrate.sock`;
- conflicting ambient `HOME`, `LIMA_HOME`, `SUBSTRATE_HOME`, CWD, PATH, VM, and transport
  availability cannot retarget installer, unit, invocation, doctor, or example projections;
- doctor and invocation paths retain the explicit R3 prerequisite and perform no forwarding or
  lifecycle action;
- cleanup statements in `write_systemd_units` and `configure_guest` remain byte-identical;
- non-macOS installer behavior and all frozen compatibility behavior remain unchanged; and
- the example is only a typed compile/call-site update and performs no product proof.

No test may run `scripts/mac/smoke.sh`, start a forwarder, create/unlink the managed socket,
exercise SSH-side unlink/timeout/handle drop, delete/rebuild a VM, remove/replace units, or claim
native macOS evidence. Native mapping-only evidence remains R2-3Z-owned.

REQUIRED VERIFICATION

Run:

- `bash -n` over every changed shell script and shell fixture;
- `shellcheck` over every changed shell script and shell fixture when the pinned tool is available;
- `cargo fmt --all -- --check`;
- `cargo check --locked -p substrate-shell`;
- focused colocated shell tests for every edited named Rust symbol;
- `cargo test --locked -p substrate-shell` with the narrowest filters that cover the edited
  invocation/macOS doctor surfaces, plus any broader package test required by current source truth;
- `cargo clippy --locked -p substrate-shell --all-targets -- -D warnings`;
- `cargo check --locked -p world-mac-lima --example mac_backend_smoke` where cfg/toolchain permits;
- static target-macOS check/test compilation for an already-installed supported Apple Rust target
  when available, recording exact target/SDK/linker availability without installing or simulating
  a missing native toolchain;
- focused `tests/mac/installer_parity_fixture.sh`, `tests/mac/lima_doctor_fixture.sh`, and
  `tests/mac/prefix_mapping_r2_3.sh`;
- `git diff --check` and staged `git diff --cached --check`;
- exact production/test unstaged and staged allowlist/status checks;
- confirmation that every manifest, `Cargo.lock`, non-macOS installer section, base profile,
  stop/smoke runner, every non-projection `lima-doctor.sh` statement,
  forwarding/lifecycle source, factory/shim/replay/forwarder/Windows file, control document, and
  generated artifact is unchanged;
- exact source/call-graph proof that no typed path activates forwarding or bypasses the R3
  prerequisite;
- exact frozen-byte comparison for the named cleanup statements and all touched files' excluded
  sections;
- post-change source/caller closure for every edited existing symbol; and
- `gitnexus_detect_changes()` before commit, with every affected flow inspected.

If native Apple compilation is unavailable because this host lacks an Apple SDK/toolchain,
preserve the raw failure and report it as unavailable static compilation, not a pass or native
evidence. If an exact required command exposes only pre-existing failures in untouched forbidden
files, preserve the raw failure and run a narrowly scoped subject supplement without weakening
the required command.

SUBJECT FINGERPRINT

After deterministic formatting/checks and before discovery review:

1. record the pre-edit base commit;
2. build a sorted manifest containing that commit plus every exact changed subject path, its Git
   mode (or `NEW`), and `git hash-object --no-filters` blob ID (or `MISSING`);
3. include only paths inside the exact M4 production/test allowlist; and
4. SHA-256 the manifest and use `sha256:<digest>` as the review subject fingerprint.

BOUNDED REVIEW

Open and validate one V1 review record with packet ID `A1.1d-5R2-3M4`. Use fresh read-only lenses
for:

1. installer boundary-carrier/IH/PM propagation, strict mismatch rejection, and ambient
   non-authority;
2. warm/doctor unit-parity, socket/doctor/invocation exact projection, explicit R3 prerequisite,
   and no forwarding/lifecycle activation; and
3. exact file/symbol/test allowlist, frozen cleanup/non-macOS behavior, static-proof honesty, and
   regression sufficiency.

Apply the common causal bounded-review contract. P1/P2 block. Valid unfixed P3/P4 must be
deduplicated/inventoried before completion; if that requires a file outside the exact M4
allowlist, stop with `BLOCKED_SCOPE_EXPANSION`. Validate the record after every returned cycle. A
CLEAN cycle is terminal; do not launch another review after CLEAN.

PUBLICATION

Before publication require:

- exact staged allowlist and frozen-section proof;
- every required check and change-detection result;
- validated CLEAN review with zero P1/P2 and complete P3/P4 disposition;
- live target still equal to the expected base;
- one atomic Conventional Commit;
- normal fast-forward push only; and
- post-push live equality, clean checkout, current GitNexus index, and 0 ahead/0 behind.

The successful completion claim must say that M4 completes only PI-009, PI-022, and PI-040 by
carrying the verified macOS IH/PM through installer, unit/socket, invocation, doctor, and harness
projections. It must explicitly disclaim forwarding activation, PI-055/PI-075 completion, native
macOS evidence, macOS product transport, R3 lifecycle ownership, later caller migration, and
R2-3 parent completion.

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

Do not generate the next increment prompt and do not begin R2-3W1. The meta
orchestrator owns independent verification and subsequent dispatch.
