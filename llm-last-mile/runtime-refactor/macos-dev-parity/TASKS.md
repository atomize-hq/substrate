# macOS Developer Parity Implementation Tasks

## Use and authority

These are approval-gated successor tasks derived by Phase 2A. They are not dispatched by this
document. Every task begins by rebinding live branch/HEAD/tree/ancestry, clean index/worktree,
archive refs, accepted predecessor commit and exact path fence. Any widened file or native action
requires a new authority rather than an in-task amendment.

The serial gates are distinct:

```text
planning acceptance
  != code-edit authority
  != native-action authority
  != code-commit authority
  != push authority
  != parity closeout
  != successor/product authority
```

Before any Rust symbol edit, run GitNexus upstream impact analysis named in `PLAN.md`. Before every
commit, run `gitnexus_detect_changes()` and confirm no unexpected execution flow. Windows paths are
a zero-change fence throughout.

## Gate 0 — accept Phase 2A and authorize only the overlap check

### Task 0.1: Execute the declared read-only Attempt 4 overlap check

**Purpose:** prove that one closed developer resource declaration is disjoint from the frozen
Attempt 4 declaration before any parity effect.

**Path/effect fence:** no repository edits. Exact read-only resources only, as defined in
`PLAN.md`. No Keychain query, `security`, `launchctl`, `sudo`, lifecycle binary, finalizer, E03,
broad enumeration, cleanup or mutation.

**Acceptance criteria:**

- [ ] Separate authority binds both declaration digests, exact allowed observations and report path.
- [ ] Every tuple compares disjoint and every developer pre-existing resource is classifiable.
- [ ] The report states `PASS_DISJOINT` or one exact stop; it never repairs or selects an alternate resource.

**Verification:** independently review the declarations, exact command transcript and zero-effect
receipt. A collision, missing identity or need for Keychain information is a stop.

**Approval gate:** a pass does not authorize code. Obtain fresh code-edit authority for A.

## Checkpoint A — host-only `--no-world` install/uninstall

### Task A1: Remove protected macOS installer admission

**Expected files (maximum 2):**

- `scripts/substrate/dev-install-substrate.sh`
- `tests/mac/installer_parity_fixture.sh`

**Expected regions:** macOS build flags/call site; `stage_managed_mac_control_binary_copy`;
`publish_mac_publisher_install_provenance_v1`; macOS world-provisioning branch. Shared carrier,
config/shim staging and Linux branch are read-only fences.

**Acceptance criteria:**

- [ ] macOS `--no-world` builds/stages current host product binaries and metadata without lifecycle-control, publisher, root host state, Stage-1 or pairing.
- [ ] Ordinary macOS install path contains no protected invocation or required protected artifact.
- [ ] Linux and Windows behavior is unchanged.

**Focused verification:**

- [ ] macOS installer fixture covers `--no-world` and rejects any protected stub invocation.
- [ ] `tests/installers/prefix_propagation_r2_1.sh` and `tests/installers/dev_install_bash32_fd_regression.sh` pass after protected-only expectations are split or removed without weakening carrier coverage.
- [ ] Diff review shows no Linux conditional change and zero Windows paths.

**Dependency:** Task 0.1 pass and fresh code authority.

### Task A2: Add the world-disabled user-level corridor record

**Expected files (maximum 3):**

- `scripts/substrate/dev-install-substrate.sh`
- `tests/mac/installer_parity_fixture.sh`
- one new focused fixture under `tests/mac/` if keeping the existing fixture reviewable requires it

**Expected regions:** the one full-schema atomic record writer/parser plus macOS completion call
site. Do not add protected schema fields or host privilege. This task owns every durable schema
field; later checkpoints may populate existing `pending` sections but may not expand the schema.

**Acceptance criteria:**

- [ ] Schema v1 binds attempt ID, carrier commitment, prefix, host principal, world-enabled flag,
      exact managed host paths, monotonic phase/transaction state, and typed nullable sections for
      selected instance/mapping, guest managed identities and the active forwarding-receipt pointer.
- [ ] World-disabled records mark guest/forwarding sections `not_applicable`; world-enabled
      prestate marks them `pending`, so B/C do not need an ad hoc schema expansion.
- [ ] Record write is atomic/durable; malformed, conflicting and partial prior records stop before replacement.
- [ ] Repeat same-prefix install reconciles; different prefix/principal cannot adopt the record.

**Focused verification:** fixture cases for absent, exact retry, malformed, different-prefix,
foreign target and injected pre/post-rename failure.

**Dependency:** A1.

### Task A3: Make host uninstall record-driven and repeatable

**Expected files (maximum 3):**

- `scripts/substrate/dev-uninstall-substrate.sh`
- `tests/mac/installer_parity_fixture.sh`
- `tests/installers/prefix_propagation_r2_1.sh`

**Expected regions:** managed symlink/binary cleanup, run-state cleanup, protected-path report and
record removal. The Linux `--remove-world-service` and host-state group/linger functions are
read-only fences.

**Acceptance criteria:**

- [ ] Selected-prefix/install-context propagation and service naming remain.
- [ ] Only exact recorded host targets/run entries are removed; unrelated entries survive and foreign conflicts return exit 5.
- [ ] First uninstall removes the corridor record last; second uninstall succeeds as an explicit no-op.

**Focused verification:** install/uninstall/reinstall fixture, unrelated prefix/run entries,
partial manifest, managed-versus-foreign symlink, repeated uninstall, and Linux prefix fixture.

**Dependency:** A2.

### Gate A-review

- [ ] Independent review resolves every P1/P2 finding.
- [ ] `git diff --check`, shell syntax checks and changed-path fence pass.
- [ ] No `limactl` or native operation has run.
- [ ] Separate local commit authority is obtained before committing A.
- [ ] Commit does not authorize B and is not pushed.

## Checkpoint B — Lima provisioning and guest world-service readiness

### Task B1: Reconstruct typed selected-instance create/start

**Expected files (maximum 4):**

- `scripts/substrate/dev-install-substrate.sh`
- `scripts/mac/lima-warm.sh`
- `scripts/mac/lima-stop.sh`
- `tests/mac/prefix_mapping_r2_3.sh`

**Expected regions:** current context/control-root/mapping functions; pre-coupling
`vm_status`/create/start/wait behavior reconstructed hunk-by-hunk, plus the installer call site
that atomically fills the already-defined selected-instance/mapping prestate. No wholesale
historical file replacement, schema expansion or `lima-lifecycle.sh` call.

**Acceptance criteria:**

- [ ] One explicit selected instance and account-derived control root are used for every command.
- [ ] Single-record status, stable machine ID and account/UID/home round-trip are required.
- [ ] Absent/current-attempt/pre-existing instance dispositions are recorded; stop never deletes a pre-existing VM.

**Focused verification:** public/internal carrier cases, named instance, ambiguous JSON, status
transitions, timeout, conflicting `HOME`/`LIMA_HOME`, no-lifecycle negative assertion.

**Dependency:** accepted A commit and fresh B code authority.

### Task B2: Restore bounded workspace staging and guest build/install

**Expected files (maximum 4):**

- `scripts/mac/lima-warm.sh`
- `tests/mac/installer_parity_fixture.sh`
- `scripts/mac/lima/substrate.yaml` only if a proved provisioning change is required
- `scripts/mac/lima/substrate-dev.yaml` only if the same proved change applies

**Expected regions:** staged-workspace manifest, verified `bin/linux` discovery, inside-guest Cargo
fallback, guest install. Host Zig evidence build remains absent.

**Acceptance criteria:**

- [ ] Verified current Linux ELF bundle is preferred; missing mandatory roles build inside the selected guest.
- [ ] Retry reuses only a matching staged manifest/attempt record and never treats a partial output as ready.
- [ ] `substrate-world-service` and `substrate-gateway` are installed; diagnostic CLI policy is explicit.

**Focused verification:** valid prebuilt, mandatory missing/in-guest build, wrong architecture,
partial bundle, injected build/install failure and same-attempt retry. No host Zig requirement.

**Dependency:** B1.

### Task B3: Install and verify canonical guest service/socket

**Expected files (maximum 5):**

- `scripts/substrate/dev-install-substrate.sh`
- `scripts/mac/lima-warm.sh`
- `scripts/mac/lima-doctor.sh`
- `scripts/mac/lima/units/substrate-world-service.service.tmpl`
- `tests/mac/lima_doctor_fixture.sh`

The socket unit is a read-only fence unless the implementation task explicitly names it after a
source-backed contradiction.

**Acceptance criteria:**

- [ ] Canonical mapping values render into `substrate-world-service.service`; service/socket names and `/run/substrate.sock` remain exact.
- [ ] Pre-existing matching corridor state is distinguished from foreign/mismatched state before update.
- [ ] Guest service/socket and `/v1/capabilities` readiness are proved; doctor has no R3 prerequisite text.
- [ ] The installer atomically fills the existing guest identity/hash/disposition section and
      advances the corridor record to `guest_ready`/`installed`; it does not add schema fields.

**Focused verification:** rendered-unit parity, netfilter flag variants, service/binary hash
mismatch, pre-existing preserve, current-attempt cleanup and routed-doctor fixture.

**Dependency:** B2.

### Task B4: Make optional guest-service uninstall ownership-exact

**Expected files (maximum 3):**

- `scripts/substrate/dev-uninstall-substrate.sh`
- `tests/mac/installer_parity_fixture.sh`
- one focused guest-cleanup fixture under `tests/mac/` if needed

**Expected region:** macOS `--remove-world-service` only.

**Acceptance criteria:**

- [ ] Reads selected prefix/instance/machine ID and managed guest identities from the corridor record.
- [ ] Removes/restores only service/socket/binaries created or exactly updated by that corridor; never hard-codes ambient `HOME`/instance.
- [ ] Pre-existing VM and unrelated guest state survive; repeat cleanup is a no-op.

**Focused verification:** created versus pre-existing service, machine-ID mismatch, unit/binary
replacement, absent state and repeated cleanup.

**Dependency:** B3.

### Gate B-review

- [ ] Independent review compares reconstructed behavior to `48216a786` and current typed mapping.
- [ ] No protected path/function/invocation remains in the ordinary flow.
- [ ] No native Lima command has run; all evidence is fixture/static.
- [ ] Separate commit authority is obtained; commit is local and does not authorize C.

## Checkpoint C — typed runtime mapping and safe forwarding

### Task C1: Replace the typed VM readiness gate

**Expected files (maximum 4):**

- `crates/world-mac-lima/src/lib.rs`
- `crates/world-mac-lima/src/vm.rs`
- `crates/world-mac-lima/src/limactl.rs`
- focused in-module tests in those files

**Required pre-edit impacts:** `ensure_vm_running`, `ensure_session_setup`,
`LimaVM::new_with_command_context`, `ensure_running`, `status`, `build_limactl_command`,
`command_for_control_root`, `decode_matching_instance`.

**Acceptance criteria:**

- [ ] Typed backend status/start/wait always uses validated host home/control root and exact instance.
- [ ] The R3 prerequisite error is gone without enabling an ambient constructor.
- [ ] Ambiguous instance/status, incomplete command context and mapping drift fail closed.

**Focused verification:** `cargo test -p world-mac-lima` on an applicable macOS build context or
the repository's established cfg-compatible target, plus exact unit test filters for command env,
ambiguous JSON and typed session setup. Do not invoke native Lima in the code task.

**Dependency:** accepted B commit and fresh C code authority.

### Task C2: Remove process-global mapping observation

**Expected files (maximum 3):**

- `crates/shell/src/execution/platform_world/mod.rs`
- `crates/world-mac-lima/src/vm.rs`
- focused tests adjacent to those modules

**Required pre-edit impacts:** macOS `detect`, typed VM observation helper(s),
`MacLimaBackend::new_with_mapping`, `managed_host_socket_path_for_mapping` if edited.

**Acceptance criteria:**

- [ ] `detect` uses checked prefix record/selected instance and per-command env; it never mutates process-global `HOME`/`LIMA_HOME`.
- [ ] Fresh guest observation matches persisted instance/machine/principal/mapping before backend construction.
- [ ] `SUBSTRATE_WORLD_SOCKET` and unrecorded instance overrides remain non-authoritative.

**Focused verification:** concurrent/ambient-env preservation test, mapping mismatch table,
backend-factory tests, and no Windows factory changes.

**Dependency:** C1.

### Task C3: Close exact SSH-UDS attempt teardown

**Expected files (maximum 3):**

- `crates/world-mac-lima/src/forwarding.rs`
- `crates/world-mac-lima/src/transport.rs` only if attempt path projection changes
- focused in-module tests

**Required pre-edit impacts:** `create_mapped_ssh_uds_forwarding_v1`,
`cleanup_failed_mapped_ssh_attempt_v1`, `ForwardingHandle::drop`, known-host helpers and
`managed_host_socket_path_for_mapping` if edited.

**Acceptance criteria:**

- [ ] Typed SSH-UDS is the only product route and uses a private current-attempt boundary.
- [ ] C3 owns one fixed ephemeral receipt schema in that boundary: corridor attempt/transaction,
      canonical executable path/digest plus expected argv plan digest, exact child PID/start/image,
      and socket/known-hosts before-state. That tuple is the plan identity. The durable corridor
      record only points to its path and digest while active, using the A2 field.
- [ ] Early exit, timeout and normal drop reap the exact child and remove/restore only exact attempt-owned state.
- [ ] Replacement or identity ambiguity is preserved with a non-success stop; a clean second attempt succeeds.

**Focused verification:** existing early-exit/pre-spawn replacement tests plus normal-drop,
timeout, PID/start-identity mismatch, socket replacement, known-host before-state and repeat tests.

**Dependency:** C2.

### Task C4: Resolve `--kill-live-processes`

**Expected files (maximum 3):**

- `scripts/substrate/dev-uninstall-substrate.sh`
- the corridor record producer in `scripts/substrate/dev-install-substrate.sh` only if required
- one focused test under `tests/mac/` or `tests/installers/`

**Expected region:** `kill_live_dev_owner_helpers` and exact forwarding-attempt receipt parsing
only. The durable schema remains the A2 schema.

**Acceptance criteria:**

- [ ] No signal is sent based solely on `ps` command text or PID.
- [ ] Exact recorded PID/start/image/plan identity is required, or the flag fails closed with guidance.
- [ ] Foreign same-text process and PID-reuse fixtures survive; exact current-attempt process terminates TERM then bounded KILL only if still identical.

**Dependency:** C3. If exact portable proof is unavailable, mark the flag unsupported and leave the
operator need deferred; do not silently delete it.

### Gate C-review

- [ ] GitNexus impact warnings were reported before edits and detect-changes shows only expected flows.
- [ ] Independent P1/P2 review is clean after fixes.
- [ ] Focused Rust/shell fixtures pass; Linux behavior and Windows paths remain unchanged.
- [ ] No native execution occurred.
- [ ] Separate commit authority is obtained; local commit is not pushed and does not authorize D.

## Checkpoint D — separately authorized native proof

### Task D1: Freeze native attempt packet

**Repository edits:** documentation/receipt path only if explicitly authorized; no product edits.

**Acceptance criteria:** exact prefix, selected instance, machine/prestate, commands, environment,
timeouts, expected resources, log locations, restoration actions, retry prohibition and stop rules
are reviewed before effects. The already-passed overlap report is rebound.

**Approval gate:** user explicitly authorizes one native attempt. Code approval is not native
approval.

### Task D2: Install and exercise

**Effects:** only those in the frozen D1 packet.

**Acceptance criteria:** current local candidate installs without protected machinery; host/world
doctor pass; typed SSH-UDS becomes ready; one real world operation succeeds; exact attempt receipt
is captured.

**Stop:** any prompt, protected invocation, overlap, unrelated mutation, identity change or failure
ends the attempt and permits only predeclared current-attempt restoration.

### Task D3: Uninstall, verify, reinstall

**Acceptance criteria:** exact corridor-owned host/forwarding/optional guest resources are absent or
restored as declared; unrelated and pre-existing prefix/VM/guest state is unchanged; repeated
uninstall is no-op; reinstall and second doctor/world operation succeed.

**Approval gate:** A/B/C source is already in separately reviewed local slice commits before D.
Native success does not authorize any additional source edit/commit, push or E.

## Checkpoint E — closeout only

### Task E1: Review the landed candidate and native receipt

Rebind the accepted A/B/C source commits and review the D receipt against the frozen native packet.
Run the changed-path allowlist, applicable validators/link checks and independent P1/P2 review on
the proposed closeout bytes. This task neither edits nor recommits product source.

### Task E2: Record and optionally commit the control-pack closeout

Under separate documentation edit and local-commit authority, record A/B/C commit trees, review
results, native receipt, Linux preservation, Windows zero-change, exact uninstall/reinstall outcome
and remaining limits. If bytes change, run `git diff --check`, documentation validators and
GitNexus detect-changes, then make one docs-only closeout commit. This task does not change or
recommit product bytes or archive refs. If no closeout bytes are authorized, it creates no commit.

### Gate push

Push requires a fresh explicit instruction after local commit/review. No task in this file grants
it.

### Gate successor

After parity is explicitly closed, rebind live repository truth and request a new product-behavior
task. Do not dispatch A1.3, A1.4, Windows, protected lifecycle, finalizer, E03 or any other
successor from this packet.
