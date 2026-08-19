# macOS Developer Parity Reconstruction Plan

## Decision status

This is the Phase 2A planning decision under `AUTHORITY_REQUIRED:MACOS_DEV_PARITY`. It selects a
reconstruction architecture but authorizes no product edit, native action, overlap check, commit
of implementation bytes, push, or successor work.

The implementation base remains `40015a6cfa508c112e6444086341d6df8473e875` plus accepted Phase 1
commit `ac496a8ab18291d5294958de83842e995ed821dc`. `origin/main` at `7028089742` is a
behavioral comparison only.

## Required decision register

1. **Product base:** remain on `40015a6cfa` / `ac496a8ab`; do not reset to `origin/main`.
2. **Installer reference:** use `bb098b6e2` as the primary pre-coupling behavioral reference and
   edit the current file hunk-by-hunk. No later installer commit is required for the selected guest
   build; `731938cfe` is mandatory only if host Zig cross-build is separately selected.
3. **Uninstaller:** retain selected-prefix/carrier propagation, canonical world-service names,
   managed binary/symlink cleanup, exact run-state cleanup, optional receipt-bound guest-service
   cleanup, and repeated-uninstall success. Simplify broad cleanup into record-driven cleanup.
4. **Live process flag:** preserve the operator need but do not approve current `ps`/argv/PID
   matching. Require receipt-bound PID/start/image/plan identity or fail closed.
5. **Typed protections:** retain selected instance/prefix, control root, socket, carrier/mapping
   commitment, account-database and no-ambient-constructor protections.
6. **R3 gate replacement:** use typed single-instance status/start/wait, fresh guest identity
   equality, mapped SSH-UDS and capabilities readiness; never bypass the gate with ambient
   selectors.
7. **Guest artifacts:** prefer a verified current `bin/linux` bundle, otherwise build missing
   mandatory roles inside Lima. Host Zig/evidence build is deferred.
8. **Forwarding:** typed SSH-UDS is the ordinary route. A private current-attempt receipt/directory
   owns exact child, socket and known-host teardown; VSock/TCP are not product fallbacks.
9. **Pre-existing state:** classify prefix, instance, service, socket and process state in one
   user-level record; mutate/remove only exact matching current/prior corridor state and stop on
   foreign or ambiguous state.
10. **Platform fence:** preserve Linux behavior through macOS conditionals plus focused Linux
    regression proof. Windows paths and cfg branches remain byte-for-byte unchanged.
11. **Attempt 4:** execute only the separately authorized declaration-based, exact read-only,
    non-Keychain overlap check defined below; any overlap, missing declaration or need for a
    Keychain query stops without alternate selection or repair.

## Selected per-file baselines

| Current surface | Reconstruction baseline | Decision |
|---|---|---|
| `scripts/substrate/dev-install-substrate.sh` | Current file, with `bb098b6e2` as the primary pre-coupling behavioral reference | Remove protected macOS hunks only; preserve current shared/Linux product behavior. No post-`bb098` forward-port is required for the selected guest-build route |
| `scripts/substrate/dev-uninstall-substrate.sh` | Current `40015a6c` blob `f8b22525…` | Retain; extend exact user-level manifest/receipt cleanup. Never replace with `origin/main` |
| `scripts/mac/lima-warm.sh` | Current mapping validators plus pre-coupling behavior at `48216a786` | Reconstruct hunk-by-hunk: typed mapping + ordinary create/start/stage/build/install/readiness; no lifecycle dispatch |
| `scripts/mac/lima-doctor.sh` | Current file, mapping/unit behavior rooted in `1b182a705` and `48216a786` | Retain routed-first proof; remove obsolete R3 dependency wording only when implementation lands |
| `scripts/mac/lima-stop.sh` | Typed selected-instance stop to be reconstructed; pre-`d8a65fc88` is behavioral evidence only | Do not restore the ambient hard-coded script; accept only explicit mapping + user receipt |
| `scripts/mac/lima/**` | Current profiles and canonical unit templates | Retain names/socket/layout inputs; update only if a focused implementation requirement proves necessary |
| `crates/world-mac-lima/src/lib.rs` | Current typed constructor and mapping validation | Preserve typed boundary; replace only the deliberate lifecycle gate and adjacent typed readiness orchestration |
| `forwarding.rs`, `limactl.rs`, `transport.rs`, `vm.rs` | Current typed primitives | Keep SSH-UDS and control-root checks; simplify exact attempt teardown without ambient fallback |
| Focused tests | Current ordinary mapping/provisioning assertions, with pre-coupling fixture behavior as evidence | Rewrite only protected-specific fixture portions; retain Linux fixtures and Windows tree unchanged |

## Architecture decision

### 1. One user-owned corridor record

The later installer creates one atomic, prefix-local record:

```text
<selected-prefix>/.dev-install-managed/macos-corridor-v1.json
```

Task A2 freezes the complete schema before its first code edit. It creates the parser and atomic
writer once; B may populate already-defined nullable sections but may not add fields ad hoc. The
record has a monotonic phase (`host_prestate`, `host_ready`, `guest_prestate`, `guest_ready`,
`installed`) plus a last-completed transaction ID. Fields that do not apply to a world-disabled
install are explicitly `not_applicable`; fields awaiting an authorized later checkpoint are
explicitly `pending`, never omitted. Its minimum fields are:

- schema owner/version and one random attempt ID;
- canonical install-bootstrap carrier commitment, selected prefix, host account and UID;
- selected Lima instance name, account-database-derived control root, observed guest machine ID,
  account, UID, home and realized guest Substrate home;
- canonical host socket `<prefix>/sock/agent.sock`, guest socket `/run/substrate.sock`, and
  prefix-local known-hosts path or private forwarding-attempt directory;
- every managed host path with type and creation/update disposition;
- every managed guest binary/unit path with content digest and the selected instance/machine ID;
- for VM, guest service, socket and prefix: `pre_existing`, `created_by_attempt`, or
  `previous_matching_corridor_record`; and
- the relative path and digest of a private forwarding-attempt receipt when forwarding is active.

The forwarding receipt is a second, ephemeral record under the prefix-local mode-0700 attempt
directory. Task C3 owns its fixed schema and atomic write/remove lifecycle. It contains the
corridor attempt ID and current transaction ID, a digest of the canonical executable path/digest
plus expected argv plan, the exact PID plus platform start/image identity, socket/known-hosts
before-state and attempt disposition. That tuple is the required **plan identity**; it never relies
on a command-text search. The durable corridor record points to that receipt only while it is live
and is atomically cleared after verified teardown. This is not a second source of mapping or
ownership authority.

Writes are temp-file + fsync + atomic rename + parent-directory fsync. A malformed, partial,
different-prefix, different-principal, different-instance, different-machine-ID, or unknown-version
record stops without mutation. The record is user-level bookkeeping, not protected authority.

### 2. Pre-existing versus current-attempt state

| Resource | Pre-existing rule | Current-attempt rule | Uninstall rule |
|---|---|---|---|
| Prefix | Existing directory is allowed; each target must be absent, a repo-managed symlink, or an exact matching prior-corridor entry | Record every created/replaced managed target before completion | Remove exact recorded targets; preserve unknown entries; `rmdir` only when empty |
| Lima instance | Existing selected name is allowed only after stable single-record status plus machine-ID/account round-trip | If absent, create the explicitly selected instance and record it as created; no ambient enumeration | Ordinary uninstall does **not delete the VM**. Optional stop/service cleanup uses the record and preserves pre-existing instance state |
| Guest service/binaries | Exact matching prior-corridor unit/binary identities may be upgraded only with a captured before-state; unrecorded or mismatched state stops | Install verified roles and canonical units; record hashes and whether each was created | `--remove-world-service` removes/restores only exact current/prior matching corridor entries; otherwise preserve and stop |
| Host socket/known-hosts | Socket must be absent. Known-hosts/attempt directory must be absent or an exact matching corridor record | Create in a private mode-0700 attempt boundary and record child/start/path identities | Reap exact child, verify attempt identities, remove only attempt-owned entries; preserve replacements |
| Run state | Unrelated run entries are allowed | Record only corridor-owned subtrees/files | Remove recorded subtree entries; never blanket-delete `<prefix>/run` |

Repeated install with the same carrier, instance and matching record is an idempotent
reconciliation. Repeated uninstall after successful cleanup is success with an explicit no-op
receipt. Absence never grants ownership of another resource.

### 3. Installer and uninstaller boundary

The installer keeps current shared prefix/config/shim/runtime behavior and the Linux branch. The
macOS conditional performs only:

1. validate selected prefix/principal/carrier and an explicit selected Lima instance (default may
   remain `substrate`, but it is persisted and never rediscovered ambiently);
2. create the full-schema user-level corridor record/prestate with guest sections `pending` or
   `not_applicable` as selected;
3. call the reconstructed typed warm helper;
4. accept one canonical mapping/readiness result;
5. stage/record only ordinary current binaries and verified guest roles; and
6. populate the already-defined guest sections and finalize the record.

It does not build/stage lifecycle-control binaries, publish provenance, use host privilege,
install a LaunchDaemon, request Stage-1 authority, perform pairing, inspect Keychain, or touch
Attempt 4.

The uninstaller keeps selected-prefix propagation, `--no-world --shim-remove`, world-service
naming, managed symlink/binary cleanup, optional guest-service cleanup, exit-5 preservation, and
repeat behavior. It changes broad cleanup into record-driven cleanup. `--kill-live-processes`
remains an operator requirement but is not approved in its current `ps`/argv/PID form; the flag
must either use exact recorded process identity or fail closed with guidance.

### 4. Lima provisioning and Linux guest artifacts

The selected ordinary source-checkout route is:

1. prefer already-current, verified Linux ELF roles from a supplied `bin/linux` bundle;
2. otherwise stage the bounded workspace and build missing mandatory roles **inside the selected
   Lima guest**;
3. install `substrate-world-service` and `substrate-gateway` inside the guest; install the guest
   CLI only when available/required for diagnostics;
4. render/install `substrate-world-service.service` and `.socket` with typed mapping values;
5. enable/start and verify `/run/substrate.sock` plus `/v1/capabilities`; and
6. return the canonical mapping and ownership receipt to the installer.

This route needs the guest Rust/Cargo toolchain and may have a slow first build, but it avoids a
mandatory host Zig/target/sysroot setup and proves the same Linux artifacts in the environment
where they run. Retry is bounded by the staged-workspace manifest and record state. The host Zig
route is deferred; if selected later, `731938cfe` is a mandatory correction and the route needs a
fresh decision independent of protected evidence-build provenance.

Guest `sudo`/systemd privilege is inside the Linux VM and remains required. It does not authorize
host `sudo`, publisher, Keychain, finalizer or E03.

### 5. Typed mapping and VM readiness

Retain all of these invariants:

- `MacLimaBackend::new_with_mapping` remains the only production constructor;
- carrier/mapping validation and commitment match precede every runtime action;
- selected prefix derives the host socket; selected host principal derives `HOME` and
  `<home>/.lima`; selected instance and machine ID are explicit and stable;
- guest principal round-trips by name and UID and is non-root;
- guest socket is exactly `/run/substrate.sock`; and
- no product path consults ambient `SUBSTRATE_WORLD_SOCKET`, ambient `HOME`/`LIMA_HOME`, or
  compatibility transport selection.

Replace the deliberate R3 prerequisite as follows:

```text
checked install carrier + persisted selected instance
  -> explicit HOME/control-root command context
  -> exact single-instance status
  -> typed start/wait if stopped
  -> fresh guest machine/principal observation
  -> mapping equality with persisted corridor record
  -> mapped SSH-UDS forwarding
  -> /v1/capabilities readiness
```

`ensure_vm_running` must use the typed `LimaVM` command context (or an equivalently explicit
command builder), not the ambient `limactl::command()`. `platform_world::detect` must stop
temporarily mutating process-global `HOME`/`LIMA_HOME`; it must construct a command with explicit
per-process environment and validate the persisted instance/mapping.

### 6. Forwarding and exact teardown

**Supported ordinary route:** typed SSH local Unix-domain-socket forwarding from the selected
prefix socket to `/run/substrate.sock`, using the selected instance's exact `ssh.config`. VSock/TCP
remain compatibility/diagnostic paths and are not product fallbacks.

Retain `BatchMode=yes`, `ExitOnForwardFailure=yes`, `ControlMaster=no`, `ControlPath=none`,
`StreamLocalBindUnlink=no`, a Substrate-scoped `UserKnownHostsFile`, process-group isolation, and a
real capabilities probe.

For teardown, create one private current-attempt directory under the selected prefix, record its
directory identity and child process identity, and keep all forwarding-local state inside it where
practical. On failure/drop/uninstall:

1. validate the receipt's corridor attempt/transaction/plan digest and PID start/image identity,
   then terminate and reap only that child;
2. validate socket/known-host identities and before-state;
3. remove or restore only entries owned by the current attempt;
4. preserve any replacement, failed child, or identity mismatch and return a non-success stop; and
5. prove a second attempt can start without broad cleanup.

The implementation may use a private directory to make same-user ordinary-dev cleanup tractable;
it does not claim protection against a hostile same-user process. Any stronger threat model belongs
to a separate production decision.

### 7. Linux preservation and Windows zero-change

Shared installer/uninstaller edits must be confined to macOS conditionals or demonstrably
platform-neutral manifest helpers. Acceptance requires:

- byte/diff review of the Linux provisioning branch;
- existing Linux install/uninstall, prefix propagation, runtime provisioning, group/linger and
  socket-ACL fixtures unchanged and passing;
- no new Linux implementation/evidence claim; and
- no changed file under Windows-specific scripts/crates/tests, no changed Windows cfg branch, and
  a path-diff gate that reports zero Windows paths.

## Exact Attempt 4 overlap-check design — not executed

### Authority and inputs

The check is a separately authorized read-only task that receives **two closed declarations**:

1. **Developer corridor declaration:** selected prefix; all proposed host paths; selected Lima
   instance name/control root; expected or observed machine ID if the instance already exists;
   host socket/known-hosts/attempt directory; guest service/socket/binary names and paths; planned
   process executable/argv identity; and the proposed user-level record path.
2. **Attempt 4 declaration:** the already-frozen resource identities from its authority/receipt,
   not newly discovered state. It includes its declared prefix, Lima instance/machine identity,
   host/guest sockets, fixed filesystem paths, launchd/service labels, helper/control image paths,
   process endpoints and other non-Keychain identities. Keychain item identities are represented
   only as the class `PROHIBITED_NAMESPACE`; they are never queried.

The declarations are compared as canonical tuples:

```text
(resource-kind, namespace, canonical-name-or-path, owning-principal/domain,
 selected-instance-or-service, optional machine/device/inode/digest)
```

### Allowed read-only observations

Only exact declared developer paths/resources may be observed, with no broad enumeration:

- `lstat`/metadata/hash of an exact declared user path;
- one exact `limactl list <selected-instance> --json` and exact guest identity/readiness probes,
  only if native authority explicitly permits those commands;
- exact process observation for a declared PID/image, if supplied; and
- comparison with the frozen Attempt 4 declaration bytes.

No Keychain API/CLI, `security`, `launchctl`, `sudo`, lifecycle binary, finalizer, E03, broad `ps`,
directory walk, cleanup, migration, repair, adoption, or mutation is allowed.

### Pass and stop rules

The check passes only when every developer tuple is disjoint from every Attempt 4 tuple and every
observed pre-existing developer resource is classifiable without inspecting Attempt 4. It stops
without effects if:

- canonical paths are equal, ancestor/descendant ownership would make cleanup overlap, instance
  names or machine IDs overlap, socket endpoints/service labels/process endpoints overlap, or an
  identity is ambiguous;
- a developer resource would require protected publisher/Keychain/finalizer ownership;
- the Attempt 4 declaration is missing a namespace needed for comparison;
- determining ownership would require a Keychain query or broad native enumeration; or
- observed exact state contradicts either declaration.

A collision is not repaired by choosing a new path during the check. It requires a separate
disposition or a new planning decision. Phase 2A did not execute this design.

## Dependency graph and vertical checkpoints

```text
Phase 2A decision (this packet)
  -> separate read-only Attempt 4 overlap authorization/check
  -> A: host-only --no-world install/uninstall -> review + separately authorized local A commit
  -> B: Lima provisioning + guest world-service readiness -> review + separately authorized local B commit
  -> C: typed runtime mapping + safe SSH-UDS forwarding -> review + separately authorized local C commit
  -> separate native-action authorization
  -> D: install -> exercise -> uninstall -> verify -> reinstall proof
  -> separate documentation-closeout authorization
  -> E: control-pack closeout -> optional docs-only local closeout commit
  -> separate push authorization
  -> fresh product-behavior authority
```

No edge grants its successor. B does not include runtime forwarding; C does not include native
proof. A, B and C are independently reviewed source commits before D; D does not authorize any
additional source edit or commit. E may commit only its separately authorized documentation
closeout and does not dispatch product work or push.

## Later implementation slices

### A. Host-only `--no-world` install/uninstall

Remove protected macOS installer dependencies while preserving current host binary/config/shim
behavior. Add the user-level corridor record in a world-disabled form. Make uninstall exact and
repeatable, including foreign-path preservation. Do not invoke `limactl`.

### B. Lima provisioning and guest world-service readiness

Reconstruct typed warm/create/start/stage/build/install behavior from the selected baseline. Produce
and consume the user-level mapping/ownership record. Verify canonical units and direct guest
readiness in fixtures only until native authority is granted.

### C. Typed runtime mapping and safe forwarding

Replace the deliberate runtime gate with typed VM readiness, eliminate process-global environment
mutation, use SSH-UDS only, and close exact current-attempt teardown. No native execution in the
code-edit task unless separately authorized.

### D. Native proof

Under a fresh action packet, run one declared attempt: install, doctor, exercise a real world,
uninstall, verify exact intended absence plus pre-existing preservation, reinstall, and repeat
doctor/world exercise. Capture restoration even on failure. Attempt 4 remains untouched.

### E. Control-pack closeout and product return

Record reviewed source commit, native receipts, exact preservation results and remaining limits.
Only a later fresh authority may select product behavior.

## Risk and rollback strategy

| Risk | Required mitigation / rollback |
|---|---|
| Shared-script Linux regression | Mac-only branches, before/after Linux fixture run, immediate revert of the slice commit if drift occurs |
| Foreign prefix/VM/service adoption | Prestate record plus exact identity; ambiguity stops before replacement |
| Partial guest install | Capture before-state, stage temp files, install roles/units as a bounded transaction where possible; on failure restore only captured exact prior state |
| Stale/replaced forwarding socket | Pre-spawn absence, private attempt directory, child/path identities, preserve on mismatch |
| Guest build failure | Preserve VM and staged manifest; report incomplete attempt; retry same record without claiming readiness |
| Uninstaller overreach | Manifest allowlist, type/digest checks, no whole-run-dir delete, foreign paths produce exit 5 |
| Native failure | Stop, capture read-only state allowed by the action packet, perform only predeclared current-attempt restoration, no retry without fresh authority |
| Protected-lane collision | Overlap check stop; no cleanup, migration, alternate selector, or Keychain query |

Rollback is slice-level Git revert before native work. Native rollback is not `git revert`: it is the
predeclared current-attempt resource restoration in D. Neither rollback path owns Attempt 4.

## Native proof sequence

The later native packet must bind exact prefix, instance, prestate, commands, timeouts, log/receipt
locations, restoration steps and stop rules before effects. The minimum order is:

1. exact read-only overlap check has separately passed;
2. snapshot declared developer prestate only;
3. install current source into the selected user prefix;
4. verify shims/config/binaries and typed mapping;
5. provision/start selected Lima and guest service, then routed doctor;
6. establish forwarding and execute one real world operation;
7. stop/reap forwarding and uninstall only corridor-owned resources;
8. verify exact intended absence and preservation of unrelated/pre-existing prefix, VM and guest
   state;
9. reinstall and repeat routed doctor plus real world operation; and
10. publish a local receipt for review, without push or successor dispatch.

Any failure ends the attempt after only the authorized current-attempt restoration. Static tests do
not close native acceptance.

## GitNexus upstream impact targets for the later edit session

The later implementation session must refresh live source/index truth and run
`gitnexus_impact(direction: "upstream")` **before editing each listed symbol**. High/critical risk
must be reported and separately accepted before the edit.

| File | Required impact targets |
|---|---|
| `crates/world-mac-lima/src/lib.rs` | `MacLimaBackend::new_with_mapping`, `ensure_vm_running`, `ensure_session_setup`, `ensure_forwarding`, `validate_typed_lima_backend_mapping`, `ensure_persistent_session_ready_async` |
| `crates/world-mac-lima/src/forwarding.rs` | `create_mapped_ssh_uds_forwarding_v1`, `cleanup_failed_mapped_ssh_attempt_v1`, `ForwardingHandle::drop`, `record_mapped_known_hosts_entry_v1`, `restore_exact_mapped_known_hosts_entry_v1` |
| `crates/world-mac-lima/src/limactl.rs` | `command_for_control_root`, `configure_command_for_control_root` |
| `crates/world-mac-lima/src/transport.rs` | `managed_host_socket_path_for_mapping` |
| `crates/world-mac-lima/src/vm.rs` | `LimaVM::new_with_command_context`, `status`, `ensure_running`, `start`, `wait_for_running`, `build_limactl_command`, `decode_matching_instance` |
| `crates/shell/src/execution/platform_world/mod.rs` | macOS `detect` |

Shell functions are not reliably indexed as symbols; the implementation task must still bind and
review `resolve_install_bootstrap_context`, the macOS installer branch, warm mapping/readiness
functions, uninstaller cleanup helpers and `kill_live_dev_owner_helpers` before edits.

## Completion boundary

Phase 2A is complete when `ARCHAEOLOGY.md`, this plan and `TASKS.md` pass documentation review and
are committed locally. Implementation remains **NOT STARTED**, native state and Attempt 4 remain
**UNTOUCHED**, and push remains **NOT AUTHORIZED**.
