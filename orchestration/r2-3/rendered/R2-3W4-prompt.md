Use $orchestrate-top-level-tasks and every skill required by the increment contract.

ROLE

You are the fresh top-level increment orchestrator for R2-3W4. You own this increment only.
You may use subagents for GitNexus/source analysis, bounded implementation, test/proof execution,
and independent review. You remain responsible for scope, shared-worktree integration,
verification, publication, and the terminal receipt.

DISPATCH IDENTITY

- orchestration_id: substrate-r2-3
- dispatch_nonce: a5c9cb16b2b67800ca4f3ba15d1eec1930e77ba5896184fe088ab3a1e8ee4a24
- meta_thread_id: 019fa3f7-c447-7132-9126-82e2cf38bd9d
- meta_host_id: remote-ssh-discovered:spenser-linux-codex
- increment: R2-3W4
- packet_id: A1.1d-5R2-3W4
- next_increment: R2-3W5

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
- expected base commit: 657f1a9530f9c86cd2dc88d33f5704b2bf97743e
- expected base tree: fb5933e50bfcc7dd8807ae799e4adf7d62c891e5
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

CURRENT AUTHORIZED INCREMENT: A1.1d-5R2-3W4

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
subagent must use GPT-5.4 with Extra High reasoning. Ground Windows token/Known Folder, named-pipe,
PowerShell/process, path, CLI/configuration, and Rust boundary decisions in current repository
truth, official Microsoft documentation, and the published runtime-refactor control pack. Do not
reopen R2-2 or R2-3A through R2-3W3 without a concrete contradiction.

Load only:

- current status in `llm-last-mile/runtime-refactor/00-README.md`;
- the parent A1.1d-5R2-3 row, common child-subdivision rules, and R2-3W4 section in
  `llm-last-mile/runtime-refactor/03-phase-slice-map.md`;
- PI-049, PI-060, PI-079, and PI-115 current-state and required-action rows in
  `llm-last-mile/runtime-refactor/02-seam-crosswalk.md`;
- the host-context path/principal/framing, boundary-carrier, `PlatformBootstrapMappingV1`
  construction/verification, Windows scope/control-root, forwarder internal-boundary,
  diagnostics, and R3-exclusive lifecycle contracts in
  `llm-last-mile/runtime-refactor/04-contracts-and-gates.md`;
- R2-MAP-WIN-01, R2-GEN-01, and R2-DIAG-01 in
  `llm-last-mile/runtime-refactor/05-debug-regression-ledger.md`; and
- `06-review-finding-inventory.md` only for actual P3/P4 deduplication.

SELECTED OUTCOME

Make the Windows forwarder's internal Rust entry require the exact authenticated W1 host carrier,
W2 `PlatformBootstrapMappingV1`, and explicit A-scoped config/log paths already validated by
`start-forwarder.ps1`. Revalidate the current Windows token, mapping commitment, registered distro,
normalized named pipe, exact guest target, and explicit path projections before logging, listener
construction, bridge construction, or another process boundary. Transport the shared PID target
as an identity projection only; never delete, replace, terminate, or claim ownership of it.

EXACT COMPLETION CLAIM

R2-3W4 completes only PI-049 and the forwarder-config consumption slice of PI-079. It supports
PI-060 and PI-115. It does not implement the named-pipe listener or WSL child leaf, migrate later
factory/shell/shim/replay callers, activate provisioning, replace/stop/kill a process, remove a PID
or artifact, define a deletion manifest, claim native Windows evidence, complete any R3 row, or
complete the R2-3 parent packet.

EXACT ALLOWLIST

Production changes are limited to:

- `scripts/windows/start-forwarder.ps1`
- `crates/forwarder/src/config.rs`
- `crates/forwarder/src/logging.rs`
- `crates/forwarder/src/windows.rs`
- `crates/forwarder/src/main.rs`

Test changes are limited to:

- colocated `#[cfg(test)]` modules inside those Rust files; and
- mapping-only W4 assertions in `scripts/windows/prefix-mapping-r2-3.Tests.ps1`.

Do not edit `crates/forwarder/src/pipe.rs`, `bridge.rs`, `wsl.rs`, `tcp.rs`, any manifest,
`Cargo.lock`, another crate/file, installer/uninstaller, other PowerShell script, Unix/macOS file,
control document, generated analyzer file, `AGENTS.md`, or `CLAUDE.md`. If another production file,
dependency, feature, or test file is required, stop with `BLOCKED_SCOPE_EXPANSION`.

EXACT SYMBOL SCOPE

Only these existing Rust items/functions may change:

- private `Cli` in `crates/forwarder/src/windows.rs`, including only the minimum fields/attributes
  required for the internal authenticated argv surface;
- `Cli::resolve_log_dir`;
- `windows::run`;
- `ForwarderConfig::load`;
- `default_config_path`;
- `resolve_target`;
- `logging::init`; and
- the Windows and non-Windows cfg `main` entries in `crates/forwarder/src/main.rs`.

Only these new production symbols may be added:

- `ForwarderConfig::load_internal`
- `ForwarderConfig::validate_mapping`

Private test seams may be added only inside the allowlisted files and must prevent real process,
listener, pipe, PID, log, or lifecycle action. Do not add another public host context, platform
mapping, transport, path, target, configuration, or lifecycle type. Reuse
`InstallBootstrapContextCarrierV1`, `PlatformBootstrapMappingV1`,
`WindowsForwarderScopeV1`, and existing transport-api-types validation.

Every other production symbol is byte-frozen except minimum import/field/call plumbing needed for
the named symbols. `BridgeTarget` representation/display, config deserialization shape, listener
and stream construction, bridge copy/wait/shutdown, Ctrl-C handling, task join/cancellation, and
process behavior are frozen unless the named functions require exact typed argument plumbing
without semantic change.

PRE-EDIT SOURCE CLOSURE AND IMPACT

Before editing, perform exact source/caller closure across the five production files, their
colocated/shared tests, W1/W2 carrier/mapping surfaces, the W3 typed backend, and the frozen W5
listener/bridge leaves for:

- every forwarder public/internal invocation and every `Cli` field/default;
- every config/log/PID/pipe/distro/target/commitment path or value;
- every read of `LOCALAPPDATA`, `USERPROFILE`, `SUBSTRATE_HOME`, `SUBSTRATE_ROOT`,
  `SUBSTRATE_INSTALL_*`, `SUBSTRATE_FORWARDER_*`, `WSLENV`, CWD, executable/repository location,
  config contents, or hard-coded default;
- every `ForwarderConfig` constructor/load/target caller;
- every logging initialization and listener/bridge handoff;
- every current Windows account/SID and token Known Folder observation;
- every process start, wait, kill, stop, terminate, PID read/write/remove, timeout, retry,
  readiness, stream, cancellation, or convergence boundary; and
- every normal-product UDS target versus explicitly diagnostic TCP path.

Run file-qualified upstream GitNexus impact with tests included for every named existing Rust
symbol. Record direct callers, affected processes/modules, and risk. Treat `Cli`,
`windows::run`, `ForwarderConfig::load`, target selection, explicit config/log paths, shared PID
projection, logging initialization, and both main entries as HIGH even when Windows cfg indexing
under-reports them. Warn before proceeding. Any unexpected caller, lifecycle reachability,
authority root, or need to edit a W5 leaf stops W4.

AUTHENTICATED INTERNAL ENTRY

`scripts/windows/start-forwarder.ps1` remains the public or installer-managed W2 boundary. After
its existing canonical IH/PM/current-token validation, its child launch must pass on explicit argv:

- the exact canonical encoded W1 host carrier;
- the exact canonical encoded W2 platform mapping;
- exact registered distro spelling;
- exact normalized PM named pipe;
- exact `A\forwarder\forwarder.toml` config path;
- exact `A\forwarder\logs` log directory; and
- only an explicitly selected diagnostic TCP bridge when the existing public diagnostic option
  was supplied and PM validation already succeeded.

The child process environment may carry only checked non-authoritative projections required by
the already-reviewed boundary. Missing, duplicate, malformed, noncanonical, tampered, or
conflicting argv fails closed in Rust. `AdditionalArgs` may not override any authenticated
selector, carrier, config/log path, mode discriminator, or mapping field.

The existing readiness loop and `-WaitForExit` behavior remain operational compatibility only.
The timeout kill/wait block, PID removal/ownership behavior, process termination, pipe probing,
retry timing, and every statement implementing those actions are byte-frozen. W4 may change only
the authenticated child argv/projection and the minimum exact tests around it.

CLI, CONFIG, AND LOG PATH RULES

Internal mode is selected only by the complete authenticated argv surface. It requires explicit
`--config` and `--log-dir`; both must be absolute normalized Windows paths under the selected
prefix A and must equal the paths recomputed from the validated carrier. No internal default or
environment fallback exists.

`Cli::resolve_log_dir` returns the explicit validated internal log directory. In any preserved
diagnostic compatibility mode, ambient/default behavior must remain explicitly labeled and unable
to satisfy W4 product proof. `LOCALAPPDATA`, `USERPROFILE`, CWD, executable/repository location,
and config-file location cannot select internal config or log paths.

`ForwarderConfig::load_internal` receives the validated carrier/mapping plus the explicit CLI
projections and config path. It:

- validates and canonically re-encodes the host carrier;
- decodes/re-encodes/revalidates PM against that exact carrier;
- requires `platform_kind=wsl` and `transport_kind=wsl`;
- validates the current Windows account+SID against IH;
- resolves the current token's `FOLDERID_LocalAppData`, recomputes
  `WindowsForwarderScopeV1`, and requires the exact PM control root/shared PID target;
- requires CLI and config distro to equal PM's registered distro;
- requires CLI and config pipe to equal PM's normalized named pipe;
- requires normal-product `BridgeTarget::Uds` to equal PM's exact `/run/substrate.sock`;
- requires config/log paths to equal their A-scoped projections; and
- finishes all validation before logging, listener construction, bridge construction, or process
  action.

`ForwarderConfig::validate_mapping` revalidates the complete carrier/mapping/current-token and the
stored distro, pipe, target, commitment, and path projections. It may not construct a replacement
IH/PM, accept a self-consistent mapping for another principal/instance, read ambient selectors, or
promote a config/environment value to authority.

`ForwarderConfig::load`, `default_config_path`, and `resolve_target` may preserve explicitly
labeled diagnostic compatibility, but normal internal mode must never call an ambient/default
selection path. A config file, `LOCALAPPDATA`, `USERPROFILE`, `SUBSTRATE_FORWARDER_TARGET`,
`SUBSTRATE_FORWARDER_TARGET_*`, inherited `WSLENV`, or a hard-coded default is match-only
diagnostic input after PM verification and cannot replace the verified mapping.

TRANSPORT AND SHARED-STATE BOUNDARY

Normal product transport is PM's normalized named pipe with exact PM registered distro and guest
socket. The W4 configuration passed to frozen W5 leaves must carry that identity without
reselection. An explicitly selected TCP compatibility target remains a labeled diagnostic only
after complete PM validation; it cannot select distro, pipe, host commitment, guest identity/home,
config/log paths, or satisfy R2-MAP-WIN-01 product proof.

The shared PID path is exactly under PM's token-Known-Folder-derived Windows forwarder control
root for the canonical `(SID, registered distro, machine ID, normalized pipe)` scope. W4 may
validate and transport that path but may not create deletion ownership, remove a PID, stop or
replace a process, kill on timeout, or claim convergence. Prefix config/log projections stay under
A and must reject a conflicting host commitment.

MAIN AND FAILURE ORDERING

The Windows main entry must surface sanitized validation failure and exit before logging/listener/
bridge/process action. The non-Windows main remains an unsupported-platform compatibility surface
and may change only for exact shared parsing/test plumbing required by the W4 static harness; it
must not become a working forwarder.

`windows::run` parses, validates, and constructs the complete internal configuration before
calling `logging::init` or spawning listener tasks. `logging::init` receives only the explicit
validated A-scoped directory and may not derive a path. Existing log formatting/rotation/guard
semantics are frozen.

TEST-DRIVEN PROOF

Use deterministic mocks/fakes and temporary sandboxes. Tests must never mutate the real Known
Folder, registry, WSL distro, forwarder, named pipe, PID/config/log state, process table, profile,
installed prefix, or protected checkout. Cover at least:

- exact canonical host-carrier/PM round trip and current account/SID binding;
- exact token Known Folder, scope digest, shared PID path, A-scoped config path, and A-scoped log
  directory;
- internal CLI requires both carriers and explicit config/log paths;
- missing, duplicate, malformed, noncanonical, tampered, forged-principal, wrong-commitment,
  wrong-distro, wrong-pipe, wrong-control-root, wrong-PID-root, wrong-config/log, and wrong-guest-
  socket rejection;
- internal config and CLI equality with PM and failure before logging/listener/bridge action;
- ambient `LOCALAPPDATA`, `USERPROFILE`, target, pipe, distro, config, log, home/root, CWD,
  executable/repository, and `WSLENV` values never select or replace an internal value;
- config-file/environment conflicts reject or remain explicitly diagnostic after PM validation;
- normal product target is exactly PM UDS and diagnostic TCP cannot satisfy mapping proof;
- `start-forwarder.ps1` passes the exact authenticated argv/config/log projections and prevents
  `AdditionalArgs` override;
- timeout kill/wait, PID removal, termination, readiness retry/probe, Ctrl-C, task-join, listener,
  stream, and bridge-leaf bytes/semantics remain unchanged; and
- W4 never exercises deletion, replacement, stop/kill, listener binding, bridge spawn, or
  convergence in tests.

The shared PowerShell test may add only mapping/internal-boundary W4 static assertions; it may not
execute Windows lifecycle. Colocated Rust tests own CLI/config/log validation behavior.

REQUIRED VERIFICATION

Run:

- `cargo fmt --all -- --check`;
- `cargo check --locked -p substrate-forwarder`;
- `cargo test --locked -p substrate-forwarder -- --nocapture`;
- `cargo clippy --locked -p substrate-forwarder --all-targets -- -D warnings`;
- an external temporary harness that copies only the allowlisted W4 Rust sources and the minimum
  unchanged dependency surfaces, removes only Windows cfg needed to expose the internal
  CLI/config/log path on Linux, injects read-only token/Known-Folder seams, stubs every
  listener/bridge/process action, and executes the complete focused Rust proof without changing
  tracked files;
- honest Windows-target `cargo check`/`cargo test --no-run` attempts when the installed target and
  cross-toolchain permit them, recording linker/toolchain unavailability rather than treating it
  as product regression or success;
- the narrowest supported W4-only/static mode of
  `scripts/windows/prefix-mapping-r2-3.Tests.ps1`, plus the full shared file, only if a supported
  PowerShell runtime is present; otherwise preserve exact unavailability and use static assertions
  without editing production PowerShell outside the allowlisted launch boundary;
- exact source assertions proving internal mode requires carriers and explicit A-scoped config/log
  paths, PM supplies product distro/pipe/target/commitment, ambient/default selectors cannot
  satisfy product construction, and the shared PID target is never deleted;
- exact frozen-block hashes for the `start-forwarder.ps1` timeout kill/wait and readiness loop plus
  Rust Ctrl-C/task-join/listener/bridge call semantics outside the named plumbing;
- `git diff --check` and staged `git diff --cached --check`;
- exact unstaged and staged production/test allowlist and symbol containment;
- confirmation that every manifest, lockfile, W5 leaf, other crate/script, control document, and
  generated artifact is unchanged; and
- `gitnexus_detect_changes()` before commit, with every reported flow inspected.

Linux native cargo commands that compile out Windows code are baseline checks only and cannot
satisfy focused W4 proof by themselves. The external harness and honest cross-target/static checks
must cover the subject. Formal native Windows product evidence remains owned by R2-3Z and must not
be claimed here. If an exact required command exposes only a pre-existing failure in untouched
forbidden code, preserve it and run a narrowly scoped subject supplement without weakening the
required command.

SUBJECT FINGERPRINT

After deterministic formatting/checks and before discovery review:

1. record the pre-edit base commit;
2. build a sorted manifest containing that commit plus every exact changed subject path, its Git
   mode (or `NEW`), and `git hash-object --no-filters` blob ID (or `MISSING`);
3. include only paths inside the exact W4 production/test allowlist; and
4. SHA-256 the manifest and use `sha256:<digest>` as the review subject fingerprint.

BOUNDED REVIEW

Persist and validate one actual V1 review-cycle JSON record with packet ID `A1.1d-5R2-3W4` using
`llm-last-mile/runtime-refactor/review-control/validate_review_cycle.py`. The record path in the
terminal receipt must point to that exact JSON file and its SHA-256 must match. Use fresh read-only
lenses for:

1. carrier/PM canonical validation, current token/Known Folder binding, scope/PID root, and A-scoped
   config/log paths;
2. CLI/config/target/logging propagation, product UDS and named-pipe identity, diagnostic TCP
   isolation, and failure ordering before logging/listener/bridge action; and
3. PowerShell child argv, exact file/symbol/test allowlist, frozen timeout/PID/listener/stream/
   lifecycle semantics, static/native-proof honesty, and regression sufficiency.

Apply the common causal bounded-review contract. P1/P2 block. Valid unfixed P3/P4 must be
deduplicated or added to `06` only through the separately authorized inventory path. The final
record must validate, end CLEAN, have zero open P1/P2, and completely dispose P3/P4. CLEAN is
terminal; do not launch another review after CLEAN.

LANDING GATES

Before commit, require:

- exact W4 production/test allowlist and symbol containment;
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
completion/non-claim boundary, and `next_increment=R2-3W5`.

For failure, send the exact blocked status, evidence, required authority or platform, and a
copy-ready handoff prompt. Do not begin, render, or dispatch R2-3W5.

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

Do not generate the next increment prompt and do not begin R2-3W5. The meta
orchestrator owns independent verification and subsequent dispatch.
