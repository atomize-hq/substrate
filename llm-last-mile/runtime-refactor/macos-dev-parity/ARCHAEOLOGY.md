# macOS Developer Parity Phase 2A Archaeology

## Status, scope, and evidence rule

- **Packet:** Phase 2A, documentation only — installer/Lima archaeology and reconstruction
  decision.
- **Bound source:** `ac496a8ab18291d5294958de83842e995ed821dc` (tree
  `36b89af3cf3fa3618e6719df74a181eea70fc23b`), whose exact parent is product baseline
  `40015a6cfa508c112e6444086341d6df8473e875`.
- **No effects:** this packet did not run an installer, uninstaller, Lima, `limactl`, `security`,
  `launchctl`, `sudo`, lifecycle binaries, finalizer, E03, or an Attempt 4 overlap check.
- **Classification rule:** each row below uses exactly one of `RETAIN_UNCHANGED`,
  `RETAIN_INTENT_SIMPLIFY_IMPLEMENTATION`, `DROP_PROTECTED_LIFECYCLE_COUPLING`, or
  `DEFERRED_UNCERTAIN_WITH_REQUIRED_PROOF`.
- **Provenance rule:** conclusions below were checked against file diffs or blobs, not inferred
  from commit subjects.

## Startup rebind

| Item | Verified result |
|---|---|
| Canonical checkout | Branch `feat/internal-host-orchestrator-world-dispatch-bootstrap`, HEAD `ac496a8ab`, tree `36b89af3c`, upstream `origin/feat/internal-host-orchestrator-world-dispatch-bootstrap` |
| Canonical status | Clean index, clean tracked worktree, no untracked files; the prompt's expected dirty condition was not present and was not manufactured |
| Phase 1 reachability | Local branch contains `ac496a8ab`; no remote-tracking branch contains it. The upstream is one commit behind. Phase 1 is **local-only in the locally available ref set** and was not pushed |
| Phase 2A checkout | Linked worktree `/Users/spensermcconnell/.codex/worktrees/macos-dev-parity-phase2a/substrate`, branch `feat/macos-dev-parity-phase2-archaeology`, exact HEAD/tree `ac496a8ab` / `36b89af3c`, no upstream, clean before documentation edits |
| Protected archive | Local and `origin/` refs `feat/archive-r3-macos-protected-lifecycle-20260819` both resolve to `ff48da180db4515147486f8b95f05626ca38e89b` |
| Donor archive | Local and `origin/` refs `feat/archive-r3-macos-donor-20260814` both resolve to `c9b67286193b41f942e41c52af66331d5ad2d702` |

The clean canonical status is a factual variance, not an authority contradiction. A separate
Phase 2A branch/worktree was still required by the task and prevents the accepted Phase 1 branch
from being reused as an editing surface.

## Verified history and baseline comparisons

### Product ancestry and the simple behavioral reference

1. `origin/main` resolves locally to `7028089742ea6a936d9dd5cd4a4582f8715fd6dd`
   (`chore: release 0.2.8`, 2026-05-19).
2. `7028089742` is an ancestor of `40015a6cfa`; their merge base is exactly `7028089742`.
3. `40015a6cfa` is the exact parent of `ac496a8ab`; therefore the active product line stays on
   `40015a6cfa` plus the Phase 1 documentation commit. `origin/main` is a known-working behavioral
   reference, not a reset/cherry-pick target.
4. The reason not to reset is source-backed: the interval from `origin/main` to `40015a6cfa`
   contains current prefix propagation, private-home/bootstrap projection, runtime provisioning,
   Linux service behavior, typed Lima mapping, current runtime APIs, and the uninstaller changes
   described below. Resetting would discard product work unrelated to protected macOS lifecycle.

### Installer history

`git rev-list --count origin/main..40015a6c -- scripts/substrate/dev-install-substrate.sh`
returns **37**. The last pre-protected-lifecycle installer checkpoint is
`bb098b6e265a4b6cadc4b7ca908c8b6ccf59f2b9`; its immediate successor is
`fc150213826b8b423f042b4591d6b8b1cb3ace20`.

The actual `bb098b6e2..fc1502138` diff adds 76 installer lines: immutable copies and a managed
manifest for `substrate-lifecycle-control` and `substrate-lifecycle-macos`, adds those binaries to
the macOS Cargo build, and stages them under the prefix. That is the first installer integration
with the protected control/publisher lane. Later commits add the root-owned provenance record,
LaunchDaemon/publisher service, Stage-1 lifecycle request, fixed evidence build, and pairing.

Post-`bb098b6e2` installer commits were inspected individually:

| Commit | Actual installer contribution | Phase 2A disposition |
|---|---|---|
| `fc1502138` | Managed immutable copies of protected lifecycle control/executor and their manifest | Drop with protected lane |
| `a9626bc10` | Root-owned publisher provenance, privileged helper/plist, code identity, fixed review record | Drop with protected lane |
| `a06b3fd26` | Root-controlled `/var/empty` for the privileged provenance probe | Drop; only exists inside publisher provenance |
| `2511eea6e` | Fixed four-artifact AArch64 host cross-build and protected install-recovery receipt structure | Do not forward-port wholesale; build-route decision is separate below |
| `731938cfe` | Removes duplicate `--target=aarch64-unknown-linux-gnu` from the Zig linker wrapper | Correct and reusable **only if** host Zig cross-build is later selected; the selected route is guest build, so no immediate forward-port |
| `3bd997948` | Runs the provenance-bound fixed `limactl --version` image under the installer UID from a privileged publisher block | Drop; this is publisher provenance, not the ordinary user-level `limactl` execution path |
| `cc2bf70ec`, `e7b62a140`, `c6641a03d`, `cf1ad2de7`, `68de6d936`, `feeeb914c`, `b032dcdf` | CDHash requirements, FD3/bootstrap response, signed control image, launchd registration, Stage-1 template/image digests, root artifact adoption and pairing handoff | Drop with protected lane |

**Installer reconstruction baseline:** use the `bb098b6e2` installer as the primary behavioral
reference while editing the current `40015a6cfa` file hunk-by-hunk. Do not replace the current file
with the old blob. No post-`bb098b6e2` installer correction is mandatory for the selected
inside-Lima build route. `731938cfe` becomes mandatory only if a later decision changes the build
route to Zig host cross-build.

### Uninstaller history

Exactly three commits touch `scripts/substrate/dev-uninstall-substrate.sh` between `origin/main`
and `40015a6cfa`: `8f8db02cd`, `1c7cfa918`, and `d9d991b60`.

The blob at `d9d991b60`, `bb098b6e2`, and `40015a6cfa` is identically
`f8b22525dacf1e994f631f9b6730c48f2943acd0`; the `origin/main` blob is instead
`2a66651bb861bafee7ca973eea4c45f9a77e924b`. The current uninstaller therefore retains useful
selected-prefix/install-context propagation and other cleanup behavior and must not be replaced
with `origin/main`.

The unchanged blob also exposes a real reconstruction gap: protected installer commits added
new macOS managed roles but never advanced the uninstaller. It removes only selected cached Linux
roles (`substrate`, `world-service`), uses a hard-coded `substrate` instance for optional guest
cleanup, and does not own the protected lifecycle binaries. Phase 2 must extend the manifest model
for the ordinary corridor rather than borrowing protected cleanup.

### Lima and runtime checkpoints

| Commit | Diff-confirmed contribution |
|---|---|
| `1b182a705` | Hardened account-database host context, selected prefix, explicit `HOME`/`LIMA_HOME`, stable guest machine ID/account/UID/home mapping, and fixture coverage in `lima-warm.sh` and `lima-doctor.sh` |
| `8296a7fce` | Typed `MacLimaBackend`, `LimaVM` command context, control-root validation, canonical host/guest socket projection, and typed primitive tests |
| `4325e7aa0` | Typed SSH-UDS forwarding gate and product requirement that the mapped transport be used rather than an ambient fallback |
| `48216a786` | Carries the selected prefix/mapping into rendered guest unit environment and the installer/warm/doctor fixtures |
| `c583c5f29` | Removes ambient `MacLimaBackend` constructors; production callers must supply an authenticated host carrier and Lima mapping |
| `d8a65fc88` | Replaces ordinary warm/stop with protected lifecycle dispatch, leaves the typed VM-lifecycle gate introduced by `8296a7fce` in place while enabling mapped forwarding around it, and adds useful exact current-attempt SSH socket/known-host tracking |

The important split inside `d8a65fc88` is behavioral rather than commit-wide: protected
publisher/lifecycle dispatch is dropped, while its current-attempt forwarding identity and
before-state discipline is retained in simplified user-level form.

## Current execution surfaces

### Developer installer

- `resolve_install_bootstrap_context` (principally `d9d991b60`) canonicalizes the prefix, binds the
  non-root account/UID, creates or validates a carrier, and rejects conflicting environment
  projections.
- The current macOS branch still builds `substrate-lifecycle-control` and
  `substrate-lifecycle-macos`, publishes protected provenance, installs the publisher service,
  obtains Stage-1 authority, and calls the protected warm path. Those calls are the ordinary-dev
  blocker.
- The Linux branch and shared configuration/shim staging contain post-main product behavior and
  are preservation boundaries, not reconstruction sources.

### Developer uninstaller

- The selected-prefix carrier and `--no-world --shim-remove` call are symmetric with install.
- Managed repo symlinks, config, version directory, trace file, runtime scripts, and cached Linux
  binaries are removed conservatively; unrecognized targets are preserved and reported with exit
  5.
- `RUN_DIR` is deleted wholesale; this intent is valid but the implementation is too broad for a
  state-preserving corridor.
- `--remove-world-service` on macOS uses ambient `HOME`, a hard-coded `substrate` instance, direct
  guest commands, and unconditionally named paths. It needs selected mapping and ownership
  records before it can remain supported.
- `--kill-live-processes` matches `ps` command text and PID only. PID reuse, argv differences,
  wrappers, and same-text foreign processes make that insufficiently exact.

### Lima provisioning and guest service

- The pre-coupling `48216a786` warm path stages a bounded workspace, prefers valid Linux guest
  binaries supplied under `bin/linux`, builds missing mandatory components inside Lima, installs
  `/usr/local/bin/substrate-world-service` and `substrate-gateway`, renders the canonical service
  and socket units, enables them, and verifies readiness.
- The current warm path keeps the typed mapping validators but routes mutation through
  `lima-lifecycle.sh` and a signed Stage-1 authorization. That coupling is deliberately removed.
- The current unit names and guest socket are `substrate-world-service.service`,
  `substrate-world-service.socket`, and `/run/substrate.sock`. Linux privilege inside the guest is
  retained; it creates no privileged macOS host requirement.

### Typed runtime and forwarding

- `MacLimaBackend::new_with_mapping` is the only production constructor after `c583c5f29`.
- `validate_typed_lima_backend_mapping` verifies the carrier, mapping commitment, account-database
  home/control root, Lima instance, non-root guest principal, and selected-prefix host socket.
- `ensure_vm_running` currently fails intentionally for typed command context with
  `typed Lima backend lifecycle remains gated on the R3 forwarding lifecycle prerequisite`.
- `create_mapped_ssh_uds_forwarding_v1` is the current typed product route. It refuses a
  pre-existing host socket, uses the mapped Lima SSH config, a prefix-local known-hosts file,
  `StreamLocalBindUnlink=no`, and a real capabilities probe.
- `ForwardingHandle::drop` reaps its child and validates identities. It deliberately preserves a
  leftover socket or created known-hosts path rather than racing a path unlink. The safety intent
  is retained, but a private attempt directory/receipt is needed to make successful ordinary
  teardown both exact and residue-free.
- Compatibility `auto_select` (VSock, then SSH-UDS) is diagnostic only. Typed product operation
  must not fall back to ambient VSock/TCP selection.

### GitNexus observation

The repository instruction names index `substrate-current`, but the MCP registry exposes multiple
same-named `substrate` indexes rather than that name. Exploration explicitly selected the canonical
path `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`. That index reports commit
`40015a6c`, one commit behind `ac496a8`; the intervening Phase 1 commit is documentation-only, so
the listed product-symbol relationships are not expected to drift, but they are not treated as a
fresh implementation impact report. Concept queries returned no processes and warned that FTS
indexes are missing. No index repair was run because Phase 2A authorizes read-only GitNexus
exploration only. Exact symbol context still confirmed:

- callers of `MacLimaBackend::new_with_mapping` include macOS platform detection and the typed
  backend factory;
- `validate_typed_lima_backend_mapping` feeds the mapped control-root/socket helpers;
- `managed_host_socket_path_for_mapping` is used by platform detection and backend validation;
- `create_mapped_ssh_uds_forwarding_v1` owns the exact attempt cleanup helpers; and
- `command_for_control_root` is reached from `LimaVM` command construction.

## Classification matrix

`Owner` names the later implementation slice from `PLAN.md`; no row authorizes that work.

| Surface / classification | Commit and file/symbol or region | Behavior supplied and parity rationale | Linux / Windows implications | Owner and focused acceptance proof |
|---|---|---|---|---|
| Installer carrier — `RETAIN_UNCHANGED` | `d9d991b60`; `dev-install-substrate.sh::resolve_install_bootstrap_context` | Canonical selected prefix, non-root principal, commitment and conflict rejection are required and not protected lifecycle | Shared Unix code: Linux fixture parity mandatory; Windows scripts untouched | A; existing prefix propagation and Bash 3.2 carrier fixtures |
| Installer shared staging — `RETAIN_UNCHANGED` | current `40015a6c`; config/shim/metadata and Linux provisioning regions | Current product behavior is newer than `origin/main`; reconstruction must be a macOS conditional edit | Byte/behavior preserve Linux; no Windows paths | A; Linux installer fixtures plus macOS `--no-world` fixture |
| Protected control/executor build and copies — `DROP_PROTECTED_LIFECYCLE_COUPLING` | `fc1502138`; `stage_managed_mac_control_binary_copy`, macOS build flags/call site | Exists only to feed protected publisher/bootstrap | No Linux change; Windows zero change | A; source-shape test proves no protected binary/function/call remains in ordinary mac branch |
| Publisher provenance/LaunchDaemon/Stage-1/pairing — `DROP_PROTECTED_LIFECYCLE_COUPLING` | `a9626bc10` through `b032dcdf`; `publish_mac_publisher_install_provenance_v1` and macOS world block | Root host state, code-signing, publisher and signed lifecycle authority are explicitly outside parity | No Linux change; Windows zero change | A/B; negative fixture asserts no protected command/path/string is invoked |
| Host Zig evidence build — `DEFERRED_UNCERTAIN_WITH_REQUIRED_PROOF` | `2511eea6e`, corrected by `731938cfe`; `build_and_stage_mac_aarch64_lima_artifacts_v1` | Useful cross-build technique, but brings fixed Zig/Rust/offline toolchain and protected evidence assumptions. Not selected | Linux unaffected; Windows untouched | B; only reconsider by separate route decision and then require `731938cfe` target-filter fixture |
| Uninstaller carrier and repeated removal — `RETAIN_UNCHANGED` | `d9d991b60`; `dev-uninstall-substrate.sh::resolve_install_bootstrap_context` and absent-path checks | Exact install/uninstall prefix symmetry and naturally idempotent absent checks are required | Linux carrier must remain identical; Windows untouched | A; repeat uninstall twice, ambient-prefix untouched |
| Managed binary/symlink cleanup — `RETAIN_INTENT_SIMPLIFY_IMPLEMENTATION` | current `40015a6c`; `remove_managed_symlink`, `remove_managed_prefix_linux_binary_copies` | Keep manifest-only cleanup and exit-5 preservation, extend roles to current corridor and validate exact type/identity | Preserve Linux manifests and names; Windows untouched | A/B; managed removed, foreign preserved, partial manifest stops |
| Run-directory cleanup — `RETAIN_INTENT_SIMPLIFY_IMPLEMENTATION` | current uninstaller `RUN_DIR` block | Remove recorded attempt subtrees, not blanket `rm -rf <prefix>/run`; unrelated run state must survive | Linux regression required; Windows untouched | A/C; seeded unrelated entry survives uninstall |
| Guest service name and optional cleanup — `RETAIN_INTENT_SIMPLIFY_IMPLEMENTATION` | current macOS `--remove-world-service` block | Keep service/socket/binary names, but act only through selected mapping and a user-level ownership receipt; never hard-code ambient instance | Linux service path unchanged; Windows untouched | B; pre-existing service preserved, current-attempt service removed, repeat no-op |
| `--kill-live-processes` — `DEFERRED_UNCERTAIN_WITH_REQUIRED_PROOF` | `1c7cfa918`; `kill_live_dev_owner_helpers` | Operator need remains, but `ps` argv/PID matching is not exact enough to signal a process | Linux behavior cannot silently change; Windows untouched | C; replace with receipt-bound corridor attempt/transaction + executable/argv plan digest + PID/start/image identity or keep fail-closed; PID-reuse/foreign-process fixture |
| Warm typed host/guest mapping — `RETAIN_UNCHANGED` | `1b182a705`; warm/doctor context/control-root/observe/verify functions | Prevents ambient prefix, instance and account drift | macOS only; no Linux/Windows edits | B; `prefix_mapping_r2_3.sh`, doctor fixture, non-ambient command env checks |
| Pre-coupling VM create/start/readiness — `RETAIN_INTENT_SIMPLIFY_IMPLEMENTATION` | behavior at `48216a786`; `vm_status`, `create_vm`, `start_vm`, `wait_for_running`, `configure_guest` | Ordinary developer provisioning is needed, but restore hunk-by-hunk and add ownership receipts rather than reverting a whole script | Guest Linux only; host Linux untouched; Windows untouched | B; absent/existing VM fixtures, stable machine ID, retry after injected failure |
| Protected warm/stop dispatch — `DROP_PROTECTED_LIFECYCLE_COUPLING` | `d8a65fc88`; current `ensure_vm_ready`, `lima-lifecycle.sh` call, current `lima-stop.sh` | Protected role/action/publisher authority is not needed for ordinary user-owned Lima | No Linux change; Windows untouched | B; negative source/invocation fixture and no lifecycle arguments |
| Guest binary install and units — `RETAIN_INTENT_SIMPLIFY_IMPLEMENTATION` | pre-coupling warm through `48216a786`; current unit templates | Install valid ELF roles inside selected guest and render canonical units from typed mapping; use guest build as normal fallback | Linux service semantics/name preserved inside guest; Windows untouched | B; fixture copy/build alternatives, rendered-unit parity, guest socket readiness |
| Typed backend constructor/mapping — `RETAIN_UNCHANGED` | `8296a7fce`, `c583c5f29`; `new_with_mapping`, `validate_typed_lima_backend_mapping`, account helpers | Required anti-ambient boundary; never reintroduce default constructor | macOS cfg only; Windows factory unchanged | C; existing Rust mapping mismatch tests and factory tests |
| Typed VM lifecycle gate — `RETAIN_INTENT_SIMPLIFY_IMPLEMENTATION` | gate from `8296a7fce`; later forwarding relation in `4325e7aa0`/`d8a65fc88`; `ensure_vm_running`, `ensure_session_setup`, `LimaVM` typed command context | Replace the deliberate stop with typed `LimaVM` status/start/wait using validated home/control root; do not bypass mapping | macOS cfg only; Windows zero change | C; typed command env, ambiguous JSON, start/wait and readiness unit/fixture tests |
| Mapped SSH-UDS route — `RETAIN_UNCHANGED` | `4325e7aa0` plus current `create_mapped_ssh_uds_forwarding_v1` | This is the ordinary supported route: selected prefix, selected instance SSH config, canonical guest UDS and real health probe | macOS only; no Linux/Windows host behavior | C; Rust mismatch, pre-existing socket refusal and capabilities probe tests |
| Exact forwarding teardown intent — `RETAIN_INTENT_SIMPLIFY_IMPLEMENTATION` | `d8a65fc88`; `MappedSshUdsAttemptV1`, cleanup helpers, `ForwardingHandle::drop` | Keep child reaping, before-state and identity checks; use a private attempt record/directory so owned residue can be removed without touching replacements | macOS only; Windows untouched | C; early exit, timeout, replacement race, known-host restoration, repeated attempt tests |
| Ambient VSock/TCP auto-selection — `DEFERRED_UNCERTAIN_WITH_REQUIRED_PROOF` | compatibility code predating typed route; `auto_select`, `Transport::auto_select` | Diagnostic compatibility may remain, but is not an accepted product fallback until it has equally typed identity/teardown proof | No Linux change; Windows transport separate and untouched | C; product path test must prove it cannot reach ambient fallback |
| Doctor routed-first proof — `RETAIN_INTENT_SIMPLIFY_IMPLEMENTATION` | `1b182a705`/`48216a786`; current doctor mapping and routed checks | Keep routed doctor and rendered-unit parity; replace obsolete R3-prerequisite wording with developer-corridor remediation only | macOS only; Windows untouched | B/C/D; fixture then native doctor JSON |
| Current focused fixtures — `RETAIN_INTENT_SIMPLIFY_IMPLEMENTATION` | installer parity, prefix mapping, Lima doctor and Rust unit tests | Retain ordinary mapping/provision/teardown assertions; delete or quarantine assertions whose only subject is publisher, Keychain, Stage-1, FD3 or evidence build | Linux tests preserved; Windows test tree byte-identical | A-C; focused commands enumerated in `TASKS.md` |

## Build-route decision

| Route | Complexity and toolchain | Retry and proof | Decision |
|---|---|---|---|
| Build inside Lima | Uses the selected Linux guest's normal Cargo/Rust environment; the pre-coupling warm path already stages a bounded workspace and builds missing mandatory roles | Slower first build, but failures and outputs remain inside the selected guest; can retry from staged manifest and verify ELF/install/readiness directly | **Selected ordinary source-checkout route** |
| Host Zig cross-build | Requires fixed Zig, Rust target, linker wrapper, target libraries, offline cache assumptions, and the `731938cfe` duplicate-target correction | Fast when warm but proof in current tree is entangled with protected evidence/provenance and immutable artifact adoption | Not selected; deferred |
| Prebuilt current Linux bundle | Lowest build cost when valid `bin/linux` artifacts already exist | Verify architecture/ELF/roles/digests before copy; no fallback to unverified host binary | Retained as a preferred input, with inside-Lima build as fallback |

This is not the protected evidence-build model: no root adoption, provenance record, code-signing,
fixed review digest, publisher, or Keychain is part of the selected route.

## Contradictions, gaps, and unresolved proof

1. **Canonical cleanliness:** the canonical checkout was clean, contrary to the prompt's expected
   dirty condition. No state was altered to manufacture the expectation.
2. **Typed lifecycle dead end:** current typed production construction succeeds, but session
   readiness stops before typed VM lifecycle. This is the exact coupling Phase 2 must replace.
3. **Uninstaller lag:** the current uninstaller predates every protected installer change. That is
   evidence for hunk-level reconstruction, not permission to use protected cleanup.
4. **Teardown tension:** current mapped forwarding safely preserves ambiguous residue. Native
   parity needs a private attempt boundary that can prove safe removal; until tested, exact
   residue-free teardown is a required proof, not an assumption.
5. **Process signaling:** command-text/PID discovery is not accepted. The operator need is open
   until a receipt-bound alternative passes negative tests.
6. **Pre-existing guest service:** a matching service may be older developer state or unrelated
   state. User-level records can authorize later updates/removal only when their prefix,
   instance/machine ID, unit hashes and installed binary identities all match. Otherwise stop.
7. **Attempt 4:** overlap has not been checked. The design is frozen in `PLAN.md`; executing it is
   separately authorized. Any need to query Keychain makes the check invalid and stops the lane.
8. **GitNexus FTS:** concept-query coverage is degraded. Exact contexts were available; a later
   implementation session must refresh/repair the index only if authorized and then run upstream
   impact analysis before each symbol edit.

No unresolved item in this section authorizes implementation or native evidence.
