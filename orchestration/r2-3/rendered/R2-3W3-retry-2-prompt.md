Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3W3. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: 90718042c877775c62feeb1590f8b736cf98825d1d33444adacc1cbf81ae3d04
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3W3
- packet_id: A1.1d-5R2-3W3
- next_increment: R2-3W4

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
- expected base commit: 3970e6f1780408a752df1f54532d2ad9475bf284
- expected base tree: 83756158da904c5b5784b752c0bf1fcf0971c7bc
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

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3W3

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
subagent must use GPT-5.4 with Extra High reasoning. Ground Windows token/Known Folder, WSL
instance/account-database observation, named-pipe, PowerShell/process, path, and Rust boundary
decisions in current repository truth, official Microsoft documentation, and the published
runtime-refactor control pack. Do not reopen R2-2 or R2-3A through R2-3W2 without a concrete
contradiction.

Load only:

- current status in `llm-last-mile/runtime-refactor/00-README.md`;
- the parent A1.1d-5R2-3 row, common child-subdivision rules, and R2-3W3 section in
  `llm-last-mile/runtime-refactor/03-phase-slice-map.md`;
- PI-048, PI-057, PI-076, PI-079, PI-090, PI-091, and PI-115 current-state and required-action
  rows in `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`;
- the host-context path/principal/framing, boundary-carrier, `PlatformBootstrapMappingV1`
  construction/verification, Windows scope/control-root, backend-factory, WSL child, diagnostics,
  and R3-exclusive lifecycle contracts in `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`;
- R2-MAP-WIN-01 and R2-DIAG-01 in
  `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`; and
- `06-review-finding-inventory.md` only for actual P3/P4 deduplication.

SELECTED OUTCOME

Make the Windows WSL Rust backend, path conversion, transport selection, and warm interfaces
consume and revalidate the exact W1 host carrier and W2 `PlatformBootstrapMappingV1`. Remove
normal-product environment/default reselection, bind the backend to the exact registered running
distro, guest machine/account/UID/home, typed Windows control root, normalized PM pipe, and guest
socket, and fail before warm/provision or transport use on missing or conflicting mapping.

EXACT COMPLETION CLAIM

R2-3W3 completes only PI-048. It supports PI-057, PI-076, PI-079, PI-090, PI-091, and PI-115.
It does not migrate the backend factory or shell/shim/replay callers, implement forwarder Rust
intake or pipe-listener/WSL leaf changes, activate provisioning, replace/stop/kill a process,
remove a PID or artifact, claim native Windows evidence, complete any R3 row, or complete the
R2-3 parent packet.

EXACT ALLOWLIST

Production changes are limited to:

- `crates/world-windows-wsl/src/backend.rs`
- `crates/world-windows-wsl/src/lib.rs`
- `crates/world-windows-wsl/src/paths.rs`
- `crates/world-windows-wsl/src/transport.rs`
- `crates/world-windows-wsl/src/warm.rs`

Test changes are limited to:

- colocated `#[cfg(test)]` modules inside those five production files;
- `crates/world-windows-wsl/src/tests.rs`; and
- mapping-only W3 assertions in `scripts/windows/prefix-mapping-r2-3.Tests.ps1`.

Do not edit any manifest, `Cargo.lock`, other Rust crate/file, installer/uninstaller, other
PowerShell script, Unix/macOS file, control document, generated analyzer file, `AGENTS.md`, or
`CLAUDE.md`. If another production file, dependency, feature, or test file is required, stop with
`BLOCKED_SCOPE_EXPANSION`.

EXACT SYMBOL SCOPE

Only these existing Rust functions/methods may change:

- `WindowsWslBackend::new`
- `WindowsWslBackend::build`
- `WindowsWslBackend::agent_transport`
- `WindowsWslBackend::build_agent_client`
- `WindowsWslBackend::ensure_agent_ready`
- `WindowsWslBackend::ensure_ready`
- `WindowsWslBackend::ensure_persistent_session_ready`
- `WarmCmd::enabled`
- `WarmCmd::run`
- `detect_tcp_forwarder`
- `to_wsl_path`
- `to_windows_display_path`

Only these new production symbols may be added:

- `WindowsWslBackend::new_with_mapping`
- `observe_wsl_mapping_v1`
- `validate_wsl_mapping_v1`

Private test seams may be added only inside the allowlisted files to inject read-only Windows/WSL
observations and prevent real process or lifecycle action. Do not add another public context,
mapping, transport, path, or lifecycle type. Reuse `InstallBootstrapContextCarrierV1`,
`PlatformBootstrapMappingV1`, and existing transport-api-types validation.

Every other production symbol is byte-frozen except the minimum import/field/call plumbing needed
for the named symbols. `WorldBackend` execution semantics, request/response conversion, session
cache, trace fetching, timeout policy, `Drop`, stream behavior, and filesystem-diff behavior are
frozen.

PRE-EDIT SOURCE CLOSURE AND IMPACT

Before editing, perform exact source/caller closure across the five production files, their
colocated/shared tests, the current factory/platform callers, and W2 mapping surfaces for:

- every constructor/build call and every backend field initialized by them;
- every distro, project path, pipe, TCP endpoint, agent ID, timeout, and warm selector;
- every read of `SUBSTRATE_WSL_DISTRO`, `SUBSTRATE_PROJECT_PATH`,
  `SUBSTRATE_FORWARDER_PIPE`, `SUBSTRATE_FORWARDER_TCP*`, `LOCALAPPDATA`, `USERPROFILE`,
  `HOME`, `WSLENV`, CWD, executable/repository location, or hard-coded WSL/pipe default;
- every `wsl.exe`/`wsl`, `pwsh`/PowerShell, agent-client, warm, process-start, wait, kill, stop,
  terminate, unregister, PID, delete, retry, or convergence boundary;
- every path conversion and every conversion caller;
- current registered/running distro, machine-ID, guest account/UID/home observation; and
- every normal-product versus explicitly diagnostic TCP path.

Run file-qualified upstream GitNexus impact with tests included for every named existing Rust
symbol. Record direct callers, affected processes/modules, and risk. Treat the constructor/build,
agent transport, readiness/warm chain, and every process boundary as HIGH/CRITICAL even when cfg
indexing under-reports them. Warn before proceeding. Any unexpected caller, lifecycle reachability,
or authority root stops W3.

AUTHORITATIVE INPUT AND CONSTRUCTION

`WindowsWslBackend::new_with_mapping` receives the already-authenticated W1 host carrier and exact
W2 PM as explicit typed inputs, plus only the explicit non-authority project/workspace input
strictly required by existing request path conversion. It must:

- validate and canonically re-encode the host carrier;
- decode/re-encode/revalidate PM against that exact carrier;
- require `platform_kind=wsl` and `transport_kind=wsl`;
- require exact PM registered-distro spelling, 32-lowercase-hex machine ID, guest account/UID/home,
  typed Windows control root, normalized named pipe, and `/run/substrate.sock`;
- validate the current Windows account+SID against IH and resolve the current token's
  `FOLDERID_LocalAppData` without using ambient home variables;
- recompute `WindowsForwarderScopeV1` and exact
  `<KnownFolderLocalApplicationData>\\Substrate\\forwarder\\<scope-digest>` control root;
- observe only the declared exact already-running WSL distro and revalidate its machine ID and
  account-database account/UID/home; and
- finish all validation before constructing an agent client, invoking warm, or performing any
  other process/mutation boundary.

The contextless `WindowsWslBackend::new` may remain only as an explicit fail-closed compatibility
surface that returns a mapping-required error before environment/default/OS observation. It may
not construct or select IH, PM, distro, pipe, control root, guest identity, or project path from
environment, CWD, executable/repository location, or defaults. No normal-product caller may
receive a contextless working backend.

WSL OBSERVATION AND VALIDATION

`observe_wsl_mapping_v1` is read-only. It may:

- enumerate registered distros and running distros without starting/stopping/importing/installing
  anything;
- case-insensitively match the declared PM distro while preserving and requiring its exact
  registered spelling;
- require that exact distro is already running;
- query `/etc/machine-id`, `id -un`, `id -u`, and the exact `getpwnam`/`getpwuid` account-database
  round trip inside that distro; and
- normalize only the returned account-database absolute home and derive `<home>/.substrate`.

It must reject zero/multiple/case-ambiguous distro matches, stopped/missing distro, malformed or
changed machine ID, invalid account/UID, account-database mismatch, guessed `/home/<name>`, guest
`$HOME`, host-mounted paths, or any value that conflicts with PM. Host and guest principal/path
equality is never asserted.

`validate_wsl_mapping_v1` revalidates the complete mapping, host commitment, current token,
Windows scope/control root, observed WSL identity, normalized pipe, and guest socket. It may not
construct a replacement PM, accept a self-consistent mapping for another principal/instance, or
fall back to ambient/default state.

TRANSPORT AND PATH RULES

Normal product transport is exactly `Transport::NamedPipe` at PM's normalized
`transport_host`. `agent_transport`, `build_agent_client`, capabilities, execute, and trace use
that same stored validated path. `DEFAULT_AGENT_PIPE`, `SUBSTRATE_FORWARDER_PIPE`, config/PID
state, and ambient endpoint variables are not selectors.

Existing TCP mode remains explicitly diagnostic-only. `detect_tcp_forwarder` may parse a
deliberately explicit diagnostic/test input or report ambient compatibility posture, but no
normal-product constructor/build/readiness/client path may select TCP from
`SUBSTRATE_FORWARDER_TCP*`, promote it as product proof, or let it replace PM's pipe. A conflicting
ambient TCP or pipe projection must be ignored as outer public noise or rejected at an explicitly
internal consistency boundary; it can never select transport.

`to_wsl_path` and `to_windows_display_path` receive only explicit already-selected paths. They may
normalize/convert relative-to-explicit-project, drive, and UNC paths under their existing contract,
but may not read environment, CWD, profile/home, repository/executable location, distro state, or
PM and may not infer a host/guest home equality. Existing conversion behavior outside removal of
ambient/default reachability is preserved.

WARM AND CAPABILITY GUARD

`WarmCmd` carries the exact authenticated host carrier and PM projection required by the already
landed W2 `wsl-warm.ps1` intake. `WarmCmd::run` may pass only explicit matching carrier, prefix,
distro, pipe, and mapping arguments plus the existing explicit project path. It must scrub or
overwrite conflicting relevant child environment and restore any in-process override exactly.

At the starting commit, `scripts/windows/wsl-warm.ps1` fails closed before WSL mutation and
`scripts/wsl/provision.sh` is an unconditional exit-4 guard. W3 must not edit either guard or any
script except mapping-only tests. W3 may not interpret the guard failure as readiness success,
move/bypass the guard, or run provisioning in tests.

`ensure_agent_ready`, `ensure_ready`, and `ensure_persistent_session_ready` must validate the
stored mapping before capability or warm action. Missing/conflicting mapping fails before either.
If capabilities fail, the existing warm path may invoke only the authenticated W2 guard surface
and must propagate its failure honestly. It may not provision, start/stop WSL, replace a
forwarder, retry past the guard, or claim convergence. Preserve existing operational retry/timing
semantics only where they remain reachable after successful validated capability checks.

TEST-DRIVEN PROOF

Use deterministic mocks/fakes and temporary sandboxes. Tests must never mutate the real Known
Folder, registry, WSL distro, forwarder, pipe, PID/config/log state, process table, profile,
installed prefix, or protected checkout. Cover at least:

- exact typed host-carrier/PM acceptance and canonical round trip;
- current token account/SID and token Known Folder binding;
- exact scope/control-root recomputation;
- case-insensitive distro lookup preserving exact spelling and zero/multiple/ambiguous/stopped
  rejection;
- exact machine-ID and guest account/UID/home round trip;
- rejection of guessed guest home, forged principal, tampered/noncanonical carrier or PM, wrong
  commitment/distro/machine ID/control root/pipe/guest socket, and conflicting ambient projection;
- contextless `new` and missing/conflicting `new_with_mapping` fail before OS/process/client/warm;
- exact PM named-pipe transport across constructor, `agent_transport`, client, capability,
  execute, and trace paths;
- ambient/default pipe, distro, project, TCP, home/profile, CWD, executable/repository, config,
  PID, and target values never select a normal backend;
- diagnostic TCP remains explicit, labeled, and unable to satisfy product mapping proof;
- `WarmCmd` exact argv/environment carrier projection and guard-failure propagation;
- readiness paths cannot reach provisioning, WSL start/stop, process replacement, PID removal,
  deletion, or convergence; and
- explicit-path conversion preserves supported drive/UNC/relative behavior without ambient
  authority.

The shared PowerShell test may add only mapping-only W3 static assertions; it may not edit
production scripts or exercise WSL lifecycle. Colocated Rust tests own backend behavior.

REQUIRED VERIFICATION

This retry runs on the assigned Linux host because that host supplies the contract-mandated
GPT-5.4 Extra High subagents. W3 remains static backend proof; R2-3Z still owns formal native
Windows evidence. Run:

- `cargo fmt --all -- --check`;
- `cargo check --locked -p world-windows-wsl`;
- `cargo test --locked -p world-windows-wsl -- --nocapture`;
- `cargo clippy --locked -p world-windows-wsl --all-targets -- -D warnings`;
- an external temporary harness that copies only the allowlisted W3 crate sources, removes only the
  crate-level Windows cfg needed to expose them on Linux, uses the production dependency graph and
  injected read-only observation seams, and exercises the complete focused Rust proof without
  changing tracked files;
- honest Windows-target `cargo check`/`cargo test --no-run` attempts when an installed target and
  usable cross-toolchain exist, recording toolchain unavailability rather than treating it as a
  product regression or success;
- the narrowest supported W3-only/static mode of
  `scripts/windows/prefix-mapping-r2-3.Tests.ps1`, plus the full shared file, only if a supported
  PowerShell runtime is present; otherwise preserve exact unavailability and use static assertions
  without editing the production PowerShell scripts;
- exact source assertions proving normal product transport is PM named pipe only, current token
  and live WSL identity are revalidated, no environment/default selector remains in the named
  constructor/build/transport/readiness path, and warm cannot pass the guard;
- `git diff --check` and staged `git diff --cached --check`;
- exact unstaged and staged production/test allowlist and symbol containment;
- confirmation that every manifest, lockfile, other crate/script, control document, and generated
  artifact is unchanged; and
- `gitnexus_detect_changes()` before commit, with every reported flow inspected.

Linux native cargo commands that compile out Windows code are baseline checks only and cannot
satisfy the focused W3 proof by themselves. The external harness and honest cross-target/static
checks must cover the subject. Formal native Windows product evidence remains owned by R2-3Z and
must not be claimed here. If an exact required command exposes only a pre-existing failure in
untouched forbidden code, preserve it and run a narrowly scoped subject supplement without
weakening the required command.

SUBJECT FINGERPRINT

After deterministic formatting/checks and before discovery review:

1. record the pre-edit base commit;
2. build a sorted manifest containing that commit plus every exact changed subject path, its Git
   mode (or `NEW`), and `git hash-object --no-filters` blob ID (or `MISSING`);
3. include only paths inside the exact W3 production/test allowlist; and
4. SHA-256 the manifest and use `sha256:<digest>` as the review subject fingerprint.

BOUNDED REVIEW

Persist and validate one actual V1 review-cycle JSON record with packet ID `A1.1d-5R2-3W3` using
`llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py`. The record path in the
terminal receipt must point to that exact JSON file and its SHA-256 must match. Use fresh read-only
lenses for:

1. host-carrier/PM canonical validation, current token/Known Folder binding, scope/control-root,
   and live registered/running WSL guest identity;
2. constructor/build/path/transport/client/readiness/warm propagation, named-pipe-only product
   selection, diagnostic TCP isolation, explicit argv/environment, and failure ordering; and
3. exact file/symbol/test allowlist, frozen execution/lifecycle semantics, guard preservation,
   static/native-proof honesty, and regression sufficiency.

Apply the common causal bounded-review contract. P1/P2 block. Valid unfixed P3/P4 must be
deduplicated or added to `06` only through the separately authorized inventory path. The final
record must validate, end CLEAN, have zero open P1/P2, and completely dispose P3/P4. CLEAN is
terminal; do not launch another review after CLEAN.

LANDING GATES

Before commit, require:

- exact W3 production/test allowlist and symbol containment;
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
completion/non-claim boundary, and `next_increment=R2-3W4`.

For failure, send the exact blocked status, evidence, required authority or platform, and a
copy-ready handoff prompt. Do not begin, render, or dispatch R2-3W4.

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

Do not generate the next increment prompt and do not begin R2-3W4. The meta
orchestrator owns independent verification and subsequent dispatch.
