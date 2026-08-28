**Kind:** crosswalk
**Stable ID:** `A1.1d-5R3-family`
**Canonical for:** R3 ownership/source-closure crosswalk bundle
**Status:** canonical
**Authority scope:** exact extracted 02 crosswalk source body only
**Source span:** [`02-seam-crosswalk.md#a11d-5r3-canonical-ownership-and-source-closure-crosswalk`](../02-seam-crosswalk.md#a11d-5r3-canonical-ownership-and-source-closure-crosswalk) lines 1509–1664
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R3 canonical ownership and source-closure crosswalk

## A1.1d-5R3 canonical ownership and source-closure crosswalk

This archived crosswalk superseded the old monolithic R3 hypothesis for the former R3 dispatch
plan. It is preserved engineering evidence and chronology, but is itself superseded for active
scheduling by [`linux-first-runtime-resumption/DECISION.md`](../linux-first-runtime-resumption/DECISION.md).
It is derived from source at `4ceecd50e20d822dda7cbd8f0e1bef4ccad65d8e`. GitNexus was exact at that
commit but under-resolved shell, PowerShell, cfg-specific, and implicit `Drop` flows; the caller
chains below therefore include manual source closure. Each former R3 row had exactly one owner.

### Historical `R3CleanupOnly` rows (superseded for active scheduling)

| PI | Current action and caller/consumer chain | Current authority and destructive effect | Present tests / platform | Sole future owner |
|---|---|---|---|---|
| PI-012 | `dev-uninstall-substrate.sh` entry -> helper/PID stop -> shim/version/run/prefix removal -> optional Linux or Lima cleanup | selected A plus paths, repo-target heuristics, PID/command text, literal system/VM names; kills processes and recursively removes prefix/platform state | prefix R2-1, install-state, mac parity static; Unix/Linux/macOS | `A1.1d-5R3-UNIX`; provider fences `R3-ACT-LNX-012` and `R3-ACT-MAC-012` must land first |
| PI-026 | `uninstall-substrate.sh` entry -> broad `pkill` -> profile/home/prefix removal -> systemd/Lima/socket cleanup | carrier selects A, but ambient home, names, matches, literal units/VM/socket authorize kill and recursive removal | R2-2 carrier and install-state stubs; Unix/Linux/macOS | `A1.1d-5R3-UNIX`; provider fences `R3-ACT-LNX-026` and `R3-ACT-MAC-026` must land first |
| PI-031 | release/dev provisioning -> `world-provision.sh` legacy stop/disable/removal and current replacement | selected A/principal plus literal system paths; replaces/removes units, drop-ins, socket, helpers, gateway, dirs, group/membership/ACL/linger state | world-provision static/stub fixtures; Linux privileged | `A1.1d-5R3-LINUX` |
| PI-047 | Windows release uninstall -> PID/process cleanup -> profile/A tree/wildcard cleanup -> WSL disable/terminate/unregister | A plus ambient LocalAppData/UserProfile, PID, process name, wildcard, default distro; kills and recursively removes prefix/shared/WSL state | R2-3 mapping static only; Windows | `A1.1d-5R3-WIN` |
| PI-051 | `lima-stop.sh` entry -> list/status/stop hard-coded instance | instance name only; stops possibly shared VM | none; macOS/Lima | `A1.1d-5R3-MAC` |
| PI-053 | `wsl-stop.ps1` entry -> PID/process stop -> PID deletion -> distro terminate | ambient PID, process name, pipe/default distro; kills shared process and terminates WSL | none; Windows | `A1.1d-5R3-WIN` |
| PI-064 | dev uninstall -> `dev-shim-bootstrap.sh::{uninstall_shims,remove_env_file}` -> CLI remove then recursive fallback | exact A but no artifact identity; removes shim tree/env projection | R2-1 context/basic removal; Unix | `A1.1d-5R3-UNIX` |
| PI-074 | release uninstall -> `remove_shell_path_snippets` -> `remove_path_snippet` for ambient-home profiles | ambient home and textual markers; rewrites files through temp/move | R2 selection only; Unix | `A1.1d-5R3-UNIX` |
| PI-092 | shell startup/CLI -> `ShimDeployer::ensure_deployed` -> `migrate_old_shims`/`deploy_shims` | A plus path/version/name; recursively removes/replaces shim trees and renames legacy state | shim unit/integration tests; cross-platform shell | `A1.1d-5R3-UNIX` |
| PI-093 | `ShellConfig::from_cli` shim-remove branch -> recursive A/shims removal | exact A only; removes whole tree | R2-1 target selection/basic removal; cross-platform shell | `A1.1d-5R3-UNIX` |
| PI-094 | dev installer prefix staging/cache/helper functions and matching dev-uninstall helpers | repo-target prefix and plaintext path list; removes/replaces cache, links, copied binaries, helper bridge | mac parity and installer static fixtures; Unix prefix-local | `A1.1d-5R3-UNIX` |
| PI-095 | release install -> `prepare_bundle_payload`/`link_binaries`/`deploy_shims`/`harden_shim_symlinks`/`provision_linux_world` | version/path/download/literal unit names; recursively replaces payload/version/bin/shims and invokes privileged replacement | install/prefix static fixtures; Unix/Linux | `A1.1d-5R3-UNIX`; provider fence `R3-ACT-LNX-095` must land first |
| PI-096 | Windows release installer version/bin block | exact A but no object identity; recursively replaces old version and clears A/bin | mapping/static only; Windows | `A1.1d-5R3-WIN` |
| PI-097 | Windows dev uninstaller -> typed shim remove -> recursive shim/profile removal | exact A only; removes possibly adopted tree/profile helper | mapping/static only; Windows | `A1.1d-5R3-WIN` |
| PI-098 | `lima-warm.sh::destroy_vm`; currently unreachable from layout-mismatch flow | declared VM name only; would stop/delete instance if activated | static fail-closed mismatch guard; macOS/Lima | `A1.1d-5R3-MAC` |
| PI-099 | verified mapping -> `configure_guest` -> `stage_workspace` | PM selects guest but paths/head text do not own state; recursively replaces guest staging/current tree | mac parity static; macOS/Lima guest | `A1.1d-5R3-MAC` |
| PI-100 | `configure_guest` -> `write_systemd_units`/`enable_socket_activation` | PM plus literal/generated paths; deletes temps/legacy units/socket and restarts services | mac parity static; macOS/Lima guest | `A1.1d-5R3-MAC` |
| PI-101 | compatibility `auto_select` -> `create_ssh_uds_forwarding`; typed product path currently stops before mutation | ambient/default compatibility or PM-fixed A socket; pre-unlinks path and asks SSH to unlink | source/probe freeze only; macOS | `A1.1d-5R3-MAC` |
| PI-102 | `start-forwarder.ps1` exact child launch -> wait-mode timeout kill/wait; readiness timeout currently leaves child | live current child handle only; kills current process but has no durable retry/publication join | R2-3 mapping/static only; Windows | `A1.1d-5R3-WIN` |
| PI-103 | `world-provision.sh::run_gateway_lifecycle_proof` -> `cleanup_gateway_smoke_auth` | same-run boolean plus ambient invoking home; deletes synthetic auth, with present test masking A/B mismatch | world-provision static test; Linux privileged | `A1.1d-5R3-LINUX` |
| PI-113 | implicit `ForwardingHandle::drop` -> child kill/wait -> stored-path unlink | live handle/path existence only; kills child and removes socket without exact identity or durable receipt | source-shape freeze only; macOS | `A1.1d-5R3-MAC` |
| PI-114 | `create_ssh_uds_forwarding` timeout -> child kill/wait/stderr drain | live current child handle; no exact socket restoration/retry state | source-shape freeze only; macOS | `A1.1d-5R3-MAC` |

The current set is exactly 22 rows: PI-012, PI-026, PI-031, PI-047, PI-051, PI-053, PI-064,
PI-074, PI-092 through PI-103, PI-113, and PI-114. No row may appear in another packet's owned-row
set.

### R3 action fences embedded in broader rows

These are action fences, not new PI rows and not duplicate ownership. They retain the owner of the
current PI row shown above while giving a provider packet a disjoint path/function fence:

| Action fence | Source action | Provider packet | Consumer/owner |
|---|---|---|---|
| `R3-ACT-LNX-012` | exact Linux system-state restore provider called by dev uninstall | `A1.1d-5R3-LINUX` | PI-012 / `UNIX` |
| `R3-ACT-MAC-012` | exact Lima/host-socket restore provider called by dev uninstall | `A1.1d-5R3-MAC` | PI-012 / `UNIX` |
| `R3-ACT-LNX-026` | exact Linux system-state restore provider called by release uninstall | `A1.1d-5R3-LINUX` | PI-026 / `UNIX` |
| `R3-ACT-MAC-026` | exact Lima/host-socket restore provider called by release uninstall | `A1.1d-5R3-MAC` | PI-026 / `UNIX` |
| `R3-ACT-LNX-095` | exact Linux privileged provisioning/replacement provider invoked by release install | `A1.1d-5R3-LINUX` | PI-095 / `UNIX` |
| `R3-ACT-007-RB` | current-attempt rollback after R2 dev context selection | `A1.1d-5R3-UNIX` | PI-007 remains R2-owned |
| `R3-ACT-019-RB` | current-attempt rollback after R2 release context selection | `A1.1d-5R3-UNIX` | PI-019 remains R2-owned |
| `R3-ACT-049-WIN` | Windows replacement/removal/termination after R2 mapping | `A1.1d-5R3-WIN` | PI-049 remains R2-owned |
| `R3-ACT-070-WIN` | Windows profile/removal after R2 target transport | `A1.1d-5R3-WIN` | PI-070 remains R2-owned |
| `R3-ACT-080-LNX` | legacy/uninstall cleanup excluding R2's fixed same-attempt socket restart | `A1.1d-5R3-LINUX` | PI-080 remains R2-owned |
| `R3-ACT-089-LNX` | cleanup for the synthetic-auth creation target; represented once by PI-103 | `A1.1d-5R3-LINUX` | PI-089 remains R2-owned |

Provider packets may add only their named provider symbols/files and focused tests. They may not
edit the PI-012, PI-026, or PI-095 orchestrator bodies. `UNIX` later integrates those already
landed providers and owns the row-level convergence claim.

### Finding and gate ownership

| Finding/gate | Exact R3 disposition | Sole owner or closer |
|---|---|---|
| `A1D5I-HOME-04` | synchronous current-attempt created/opened/exact/empty descriptor-relative rollback | `A1.1d-5R3-HOME` |
| `A1D5I-HOME-05` | unknown-provenance interruption residue remains unchanged/fail-closed; durable provenance need stops | `A1.1d-5R3-HOME` |
| `A1D5I-INSTALL-02` | exact selected release state deletion and convergence, never ambient home | `A1.1d-5R3-UNIX` |
| `A1D5I-INSTALL-03` | exact managed gateway/link cleanup; system gateway provider remains Linux-owned | `A1.1d-5R3-UNIX` |
| `A1D5I-INSTALL-06` | Linux helper/gateway/unit/drop-in/socket/runtime/state/account restoration | `A1.1d-5R3-LINUX` |
| `A1D5I-INSTALL-07` | replace Windows wildcard deletion with exact identity | `A1.1d-5R3-WIN` |
| `A1D5I-INSTALL-08` | R2 context is closed; R3 owns selected macOS socket lifecycle only | `A1.1d-5R3-MAC` |
| `A1D5I-INSTALL-10` | exact A-local versus SID+instance+machine-ID+pipe shared lifecycle | `A1.1d-5R3-WIN` |
| `A1D5I-REG-01` | join complete distributed lifecycle/kill-point regression coverage | `A1.1d-5R3-CLOSEOUT` |
| `A1D5I-ENV-01` | dedicated supported Linux evidence posture, not product remediation | `EVIDENCE:R3-LINUX-IMP-01`, closed by `A1.1d-5R3-LINUX-CLOSEOUT` |
| `RG-HOME-01` | HOME supplies implementation proof; closeout verifies final join | `A1.1d-5R3-CLOSEOUT` |
| `RG-INSTALL-01` | all lifecycle packets supply proofs; closeout verifies final join | `A1.1d-5R3-CLOSEOUT` |
| `R3-LIFE-01` | Unix/Linux/macOS/home/manifest lifecycle matrix | `A1.1d-5R3-CLOSEOUT` |
| `R3-WIN-01` | native Windows two-prefix/shared-state matrix | `A1.1d-5R3-CLOSEOUT` |

The R3 portions of the named findings are complete above. Current
`06-review-finding-inventory.md` entries RR-RF-0001 through RR-RF-0004 are review-process debt,
not R3 product findings; none changes owner.

### Reverse ownership and frozen boundaries

| Packet | Owned current PI rows |
|---|---|
| `HOME` | none; owns the two private-home findings |
| `MANIFEST` | none; owns the prerequisite contract/core and no destructive action |
| `LINUX` | PI-031, PI-103 |
| `MAC` | PI-051, PI-098, PI-099, PI-100, PI-101, PI-113, PI-114 |
| `WIN` | PI-047, PI-053, PI-096, PI-097, PI-102 |
| `UNIX` | PI-012, PI-026, PI-064, PI-074, PI-092, PI-093, PI-094, PI-095 |
| platform IMP evidence and closeout nodes | none; validate the exact published provider and land evidence/control only |
| final native evidence nodes | none; read-only proof of the same published UNIX checkpoint |
| `CLOSEOUT` | none; owns REG-01 and final gate joins, with no product mutation |

`HOME` freezes every installer/platform action. `MANIFEST` freezes every deletion and existing
IH/PM selector. `LINUX` freezes Unix orchestrators, Lima, Windows, and passive health.
`MAC` freezes Unix orchestrators, Linux system state, Windows, alternate instance/endpoint/
transport selection, and ambient `auto_select`. `WIN` freezes Unix/Linux/macOS and the existing R2
WSL mapping resolver/assertion/selector; it replaces only the post-mapping unconditional live guard,
supports only the already-registered PM-bound instance, and tombstones import/unregister/install-
tree deletion. It manifest-wraps guest provisioning and forwarder lifecycle without inventing a
missing-instance authority.
`UNIX` freezes provider internals, Windows, and
private-home rollback. `CLOSEOUT` freezes all production symbols.

### Source-closure and delivery refinements

These refinements are part of the singular ownership above and supersede any shorter descriptive
range in this section:

| Packet | Exact formerly escaping action fence | Closed disposition |
|---|---|---|
| LINUX | `world-provision.sh` from the first group-ensure call through the final service start, including lines currently performing enable/stop, directory recreation, socket unlink, ACL application, and restart, plus exact removal of `PartOf=substrate-world-service.service` from its `SOCKET_UNIT_CONTENT`; its existing Cargo argv/check sites | one `world-lifecycle.sh` call; socket-state owns its non-requestable coupled endpoint in one prepared transaction/receipt, service-state cannot propagate to it, and Linux executor build/check and root-publisher bootstrap are in the same packet |
| MAC | `lima-stop.sh` authority parameter/intake plus complete executable body; every normal-path `lima-warm.sh` mutator from instance realization through group/membership, staging, exact evidence-built guest binaries, private home, units/service state, sockets, layout sentinel and configuration; exact removal of `PartOf=substrate-world-service.service` from `scripts/mac/lima/units/substrate-world-service.socket`; host-publisher pairing ticket, one-use pre-effect guest-mutation grant, independent host/guest TTY pin, retained guest-bootstrap child channel/transcript, and protected-host proof plus external acknowledgement for both reserved and ticket-issued unused retirement; SSH-UDS handle/drop/known-hosts symbols | one exact IH/PM request per action; socket-state owns its non-requestable coupled endpoint in one prepared transaction/receipt and service-state cannot propagate to it; in-guest build/DNS/toolchain/package remediation is tombstoned, missing exact build-evidence artifact or interactive pairing stops, unused Keychain pairing-record change waits for the acknowledged proof, and no ambient instance selection remains |
| WIN | release install from `$tempRoot` creation through its closing `finally`, including handle-bound current-attempt registration/extraction/reverse rollback; dev install from Cargo prerequisite through completion; release uninstall through the legacy unregister range with unregister removed; dev uninstall including old shim-remove; `wsl-stop.ps1` authority intake and destructive body; `wsl-warm.ps1` post-mapping live-guard replacement, preserving absent-distro branch, complete provisioning through both health branches, and ambient-log-directory creation through PID/final probe; host-publisher pairing ticket, one-use pre-effect guest-mutation grant, independent host/guest TTY pin, retained guest-bootstrap child channel/transcript, and acknowledged unused retirement for both `Reserved` and `TicketIssued`; exact new `scripts/wsl/units/substrate-world-service.service.tmpl` and byte-exact `scripts/wsl/units/substrate-world-service.socket` sources plus the `scripts/wsl/provision.sh` PM-bound deterministic renderer/managed guest wrapper including exact `/run/substrate`, `substrate` group, PM-mapped guest membership, rendered unit bytes/state, and root:`substrate` `0660` `/run/substrate.sock` lifecycle; `start-forwarder.ps1` from PM-derived log-directory creation through launch/timeout | all creator/destructor paths route to `CurrentAttemptTempRollbackV1` or the Windows typed executor and LocalSystem publisher; exact `A\forwarder\logs` and each WSL directory/group/membership/unit/socket object have one role and before-state; noninteractive guest pairing is forbidden; the R2 mapping resolver/assertion/selector is frozen; import/install-tree deletion/unregister remain forbidden |
| UNIX | `dev-install-substrate.sh` from target/build selection through its last host-state write; both release install bodies from pre-manifest temp-root creation/registration/extraction through host-state write and retained handle-bound rollback, including exact removal of `PartOf=substrate-world-service.service` from `provision_linux_world`'s socket source; both uninstall bodies through final auto-cleanup; dev-shim bootstrap install/uninstall; shim runtime symbols | one closed manifest orchestration per accepted path plus `CurrentAttemptTempRollbackV1` before manifest availability; service-state cannot propagate to socket-state/endpoint; live hidden owner-helper enumeration/signal is tombstoned to `BLOCKED_SCOPE_EXPANSION`, so no legacy destructive fallthrough, process-name/PID authority, or path-only recursive temp cleanup remains |
| UNIX packaging | `Cargo.toml` exact cargo-dist binary list plus Unix/Windows dev build and staging lists already owned by their packets | all five lifecycle binaries are buildable; release archives carry them; wrong-platform executors reject before parsing/mutation; each installed executor is copied into its protected lifecycle capsule before first action |

Provider evidence delivery precedes ordinary distribution without creating a hidden producer. Each
MAC/WIN native evidence task builds its host executor and a Linux guest executor from the exact
remote-equal source/lock state in a harness-owned isolated build scope, records toolchain, target,
artifact SHA-256, file/code identity, and source checkpoint in `ExecutorBuildEvidenceV1`, removes
that scope before baseline, and bootstraps only those exact artifacts. Implementation fixtures are
non-executing mocks. The later UNIX packet remains the sole owner of ordinary cargo-dist archive
wiring; neither delivery path revives in-guest build or imports a prebuilt from an unproved source.
The MAC/WIN pairing ticket carries canonical host SPKI bytes, fixed P-256/P1363-low-S encoding,
and an anchor-joined fingerprint; the guest accepts it only after the independent full TTY pin.
Host System-Keychain/HKLM pairing records and the guest atomic root-only external intent bind the
seed/nonce before publisher-directory creation; the final-path inactive key is materialized only
after transcript durability. Completed post-intent states exact-join, a named-effect/identity-CAS
gap stops preserving, and a pre-intent retry must re-prompt.
Evidence-only pairing additionally carries a null-slot precommitted exhaustive guest-retirement
DAG; its receipt/acknowledgement and exact guest parity must complete before the host pairing record
and exhaustive host publisher DAG may retire. Product pairing keeps that field canonical null.

Implementation publication and native proof are distinct. LINUX, MAC, WIN, and UNIX each publish
their reviewed implementation before any native evidence task. Every IMP/final evidence task binds
a live-remote-equal commit/tree/ref and validates `codex.top-level-evidence-receipt.v1`; evidence
tasks never modify the repository. Platform closeout packets own only evidence/control ingestion.

### `A1.1d-5R3-MAC` implementation status

The MAC implementation packet materializes only the stated fixed-executor/request boundary:
typed mapping selects the already-fixed SSH-UDS path and records exact socket/known-host
before-state for timeout/Drop/retry restoration; mapped scripts carry prefix, bootstrap carrier,
platform mapping, and executor-build evidence to the executor rather than issuing Lima mutations.
The socket unit no longer allows service-to-socket propagation. Native publisher installation,
artifact build evidence, disposable Lima lifecycle, and human-pinned pairing remain exclusively
with `EVIDENCE:R3-MAC-IMP-01`.
**Source provenance:** extracted from [`../02-seam-crosswalk.md#a11d-5r3-canonical-ownership-and-source-closure-crosswalk`](../02-seam-crosswalk.md#a11d-5r3-canonical-ownership-and-source-closure-crosswalk), baseline lines 1509–1664
**Relocation note:** repository-relative Markdown targets were rebased as needed to preserve their original repository destinations after relocation.
**Baseline span SHA-256:** `1fe193616f39e360a3eda128e5afdd5883533d43fe52c73ea32b2321b1b8708d`
