**Kind:** slice/task
**Stable ID:** `A1.1d-5R2-3-family`
**Canonical for:** A1.1d-5R2-3 authoritative slice/task record
**Status:** canonical
**Authority scope:** exact extracted family-local source bodies only
**Source span:** [`03-phase-slice-map.md#a11d-5r2-3--platform-native-mapping-adapters`](../03-phase-slice-map.md#a11d-5r2-3--platform-native-mapping-adapters) lines 693–1339
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R2-3 authoritative slice/task record

###### A1.1d-5R2-3 — Platform-native mapping adapters

| Packet field | Frozen requirement |
|---|---|
| Goal | Implement and verify `PlatformBootstrapMappingV1` for macOS host-to-Lima and Windows host-to-WSL, including two-stage Lima realization/mapping, the fixed future V1 SSH-UDS target plus an R3 activation prerequisite before any macOS forwarding, one mapped pipe/forwarder scope, direct platform-helper construction, every backend-factory caller (shell, shim telemetry, replay), physical-shim trace/policy projections, Windows `-NoAutoSource`, and byte-identical WSL capability guards, without host/guest path or principal equality. |
| Prerequisites | Final R2-2; stable encoded host commitment and sudo/child contract; native-proof assignments booked even if unavailable during implementation. |
| Must read | R2-1/R2-2 closeouts; `02-seam-crosswalk.md` PI-009, PI-022, PI-039–PI-060, PI-068–PI-070, PI-075–PI-081, PI-090–PI-102, PI-112, PI-115–PI-116, PI-118, and frozen PI-113–PI-114; `04-contracts-and-gates.md` two-stage platform instance/transport identity, mapping/factory verification, physical-shim manifest/trace, WSL child, WSL guard, and shared-state scope; R2-MAP-MAC-01/R2-MAP-WIN-01/R2-GEN-01/R2-DIAG-01. |
| Sibling context | Host A remains authoritative; adapters realize platform-native paths/principals. R3 alone removes Lima/WSL/forwarder/shim/socket state. |
| Exact production-file allowlist | `crates/transport-api-types/src/lib.rs` mapping/framing section; `crates/shell/Cargo.toml` only for the frozen `windows-sys` feature additions named below; `crates/shell/src/execution/install_bootstrap.rs` mapping construction/validation only; `scripts/mac/{lima-warm.sh,lima-doctor.sh}`; `scripts/mac/lima/units/{substrate-world-service.service.tmpl,substrate-world-service.socket}`; `scripts/substrate/{dev-install-substrate.sh,install-substrate.sh}` macOS call sections only; `crates/world-mac-lima/src/{lib.rs,forwarding.rs,transport.rs,limactl.rs,vm.rs}`; `crates/world-backend-factory/{Cargo.toml,src/lib.rs}`; `crates/forwarder/Cargo.toml`; `crates/shim/Cargo.toml`; `Cargo.lock` only for the exact package dependency-list additions frozen below (the three internal `transport-api-types` edges plus existing `windows-sys 0.52.0` in the shim list; no package/version/checksum change); `crates/shell/src/execution/{invocation/plan.rs,platform/macos.rs,platform/windows.rs,platform_world/mod.rs,platform_world/windows.rs,routing/replay.rs}` exact platform/context-call sections; `crates/shell/src/builtins/world_gateway.rs` platform transport selector only; `crates/shim/src/{context.rs,logger.rs,exec/mod.rs,exec/logging.rs,exec/policy.rs}` only for invocation/factory projection, PI-116 manager-manifest intake, and PI-118 trace binding/reuse; `crates/replay/src/{lib.rs,replay/mod.rs,replay/planner.rs,replay/executor.rs}` factory-context parameter plumbing only; `crates/trace/src/{context.rs,util.rs}` only for PI-118 compatibility-posture removal/final unbound-init rule after owned shim/replay/platform callers migrate; `scripts/windows/{dev-install-substrate.ps1,dev-uninstall-substrate.ps1,install-substrate.ps1,uninstall-substrate.ps1,start-forwarder.ps1,pipe-status.ps1,wsl-warm.ps1,wsl-doctor.ps1}`; `crates/world-windows-wsl/src/{backend.rs,lib.rs,paths.rs,transport.rs,warm.rs}`; `crates/forwarder/src/{bridge.rs,config.rs,logging.rs,windows.rs,main.rs,pipe.rs,wsl.rs}`. Lima base profiles, `scripts/mac/lima-stop.sh`, `scripts/windows/wsl-stop.ps1`, and `scripts/wsl/provision.sh` are explicitly excluded for world/R3/capability ownership. No other production file. |
| Exact production sections/symbols | Mapping wire model/framing, including `host_platform_control_root` and `WindowsForwarderScopeV1`; Windows release-copy installed-product witness and current account/SID binding in `install_bootstrap.rs`; the existing target-Windows `windows-sys = "0.52"` dependency in `crates/shell/Cargo.toml` may add exactly `Win32_Security`, `Win32_Storage_FileSystem`, `Win32_System_Threading`, `Win32_System_Com`, and `Win32_UI_Shell` to its existing `Win32_Foundation` and `Win32_System_Console` features for token/SID and file identity plus `SHGetKnownFolderPath(FOLDERID_LocalAppData)`/`CoTaskMemFree`. No dependency version, new dependency, target scope, or other feature may change; this feature-only edit must not change the package graph, and any resulting `Cargo.lock` change is an unexpected stop-and-review condition rather than shell-manifest authority. Context/mapping intake; host commitment/instance/transport/control-root verification; platform account/home resolution; unit/env/socket/pipe/forwarder projection; doctor/pipe-status projection; backend factory/constructor parameters; gateway/platform-world call sites; shim unique invocation-witness construction, token-bound Known Folder projection, `ManagerHintEngine::new`/`manifest_paths` typed-IH intake and exact A-base/A-overlay selection with ambient/repo fallback removed from normal product mode, `collect_world_telemetry` explicit factory projection, and pre-manager/policy construction of the PI-117 explicit product posture from exact `A/trace.jsonl` plus policy Git directory A; `evaluate_policy` and `write_log_entry` may only reuse that already-bound trace and cannot initialize from ambient state; already-frozen replay/platform callers migrate to explicit binding; only after those owned migrations may `LegacyAmbientCompatibility` be removed/made unreachable and the final global unbound-`init_trace(None)` rule land in `trace/context.rs`, with legacy policy lookup migration/removal in `trace/util.rs`. `set_global_trace_context` remains neutral and no caller-identity table is allowed. Windows dev/release projection of the already-selected manager manifest to `A/manager_hooks.yaml` after IH construction; replay public bootstrap config and shell route through planner/executor factory argument only; shim invocation; `Transport::auto_select`, `forwarding::auto_select`, and `MacLimaBackend::{new,ensure_forwarding,get_agent_endpoint}` only to remove auto-selection from the validated normal path, consume the PM-fixed future SSH-UDS identity, and fail before forwarder launch with an explicit R3 lifecycle prerequisite; `lima_home_dir`/`lima_ssh_config_path` become typed-control-root consumers and every `limactl`/SSH-config child receives scrubbed/overwritten `HOME` and `LIMA_HOME`; `create_ssh_uds_forwarding` may receive future mapping/known-hosts parameters but is not called by the R2 validated path; `start-forwarder.ps1` IH/PM child carrier, current-token validation, exact A-scoped `--config`/`--log-dir`, and typed shared PID-root projection; `Cli::resolve_log_dir` and `ForwarderConfig::{load,default_config_path}` require the explicit PM/IH-derived paths in internal mode and may not use `LOCALAPPDATA`; `ForwarderConfig` carrier/PM/distro/pipe/target validation; `spawn_bridge` and `wsl::spawn` exact typed distro/target/commitment projection with overwritten child environment/`WSLENV`; and `-NoAutoSource` carrier. The PI-117 additive representation and all trace writer/rotation/retention/span/policy semantics are frozen in R2-3 except the exact PI-118 compatibility closure just named. Lima VSock, SSH UDS, and SSH-TCP constructors remain direct diagnostic/test-only compatibility paths in R2 and cannot satisfy PM/product proof. R3 alone may activate PM-bound SSH UDS after exact PI-101/PI-113/PI-114 lifecycle semantics land; R2 may not launch a forwarder, create/unlink a socket, set `StreamLocalBindUnlink`, kill/wait a forwarding child, or exercise handle-drop cleanup as proof. Existing explicit Windows TCP target mode likewise may remain only as labeled diagnostic input after PM validation and cannot satisfy normal mapping proof; target/config/environment cannot select normal product mapping. Lima helper construction follows PI-081: IH plus instance selector, derived committed control root, and fixed transport target before Stage 1, PM only after a running instance is observable. Stage 1 permits only status, fixed-profile render, absent-instance create or same-instance start, and wait; Stage 2 must finalize PM before R2-owned guest projection. `destroy_vm`, mismatch delete/rebuild, staged-tree/temp/unit/socket cleanup, explicit/SSH-side forwarding unlink, `ForwardingHandle::drop`, and SSH timeout kill are byte-frozen R3 sections. `start-forwarder.ps1` timeout kill is byte-frozen. Forwarder stream/wait behavior and replay command/state/origin/policy/timeout/strategy semantics are byte-frozen. In uninstallers, only context intake, non-mutating classification, current-token binding, and carrier arguments on an existing shim-remove invocation are editable. In `wsl-warm.ps1`, only parameter/context/control-root validation before the guard is editable; the guard and unreachable code are byte-frozen. `scripts/wsl/provision.sh` remains byte-for-byte fail-closed. No WSL capability, base Lima profile, or transport-protocol redesign. |
| Allowed tests | Colocated tests in each allowed Rust file, including `crates/trace/src/tests.rs` only for PI-118 compatibility closure; `crates/world-windows-wsl/src/tests.rs`; `crates/world-mac-lima/examples/mac_backend_smoke.rs` call-site update only; `crates/shim/tests/integration.rs`; `crates/replay/tests/{integration.rs,planner_executor.rs}`; `crates/shell/tests/replay_world.rs` factory-context cases only; `tests/mac/lima_doctor_fixture.sh`; mapping/context-only cases in `tests/mac/installer_parity_fixture.sh` with lifecycle expectations frozen; new `tests/mac/prefix_mapping_r2_3.sh`; new `scripts/windows/prefix-mapping-r2-3.Tests.ps1`. Neither `scripts/mac/smoke.sh` nor `scripts/windows/wsl-smoke.ps1` is an R2-3 runner: both execute R3 lifecycle actions, and Windows smoke also reaches the frozen warm guard. Native R2-3 uses only the focused non-mutating existing-instance mapping tests. No tests in excluded stop/provision/base-profile files. |
| Explicit non-goals | Host/guest equality; backend home selection; cross-platform common-home side table; WSL/Lima lifecycle deletion; wildcard removal; forwarder ownership manifest; activating macOS forwarding before R3; deleting VSock/TCP support or admitting either into V1 normal-product proof; new transport protocol; policy/world capability semantics; claiming static review as native proof. |
| Exit gate | Verified host commitment binds one named platform instance, OS-resolved host platform-control root, native account/home/principal, and the future normalized SSH-UDS transport target. Lima control root derives only from the committed host principal's account-database home; public parents scrub and overwrite child `HOME`/`LIMA_HOME`, internal conflicts reject, and no Lima command reads ambient selection. Lima two-stage order is proven without requiring PM before Stage 1 or executing delete/rebuild/staging/forwarding cleanup; mapping mismatch fails closed; the validated Lima path never auto-selects VSock/TCP and stops before forwarder launch with the explicit R3 lifecycle prerequisite; no socket/process lifecycle action becomes newly reachable. The future mac host socket and SSH known-hosts projection are A-scoped; unique shim invocation-witness recovery works without PATH precedence and fails on zero/multiple candidates; physical shim manager hints read only A-base/A-overlay under ambient B and never repo fallback; physical-shim spans/execution logs write only `A/trace.jsonl`, policy-commit lookup reads only A, and neither repeated init nor missing Git metadata touches B; Windows release-copy standalone self-derivation selects A under ambient B and rejects a forged account/SID carrier; token Known Folder plus exact SID/instance/pipe scope fixes the shared PID root, while config/logs are exactly A-scoped and every internal forwarder path is explicit; `LOCALAPPDATA`/`USERPROFILE` cannot select. Shell/shim/replay factory calls are explicit and no contextless platform factory remains; Windows `-NoAutoSource` propagates context; pipe producer/consumer/doctor/backend share one selector; forwarder validates PM/current token and its WSL child receives the exact PM distro/target/commitment despite conflicting ambient target/`WSLENV`; shared Windows artifacts are classified without deletion authority; WSL guards remain byte-identical; all static/focused tests clean. |
| Regression gates | R2-1/R2-2 gates; R2-MAP-MAC-01/R2-MAP-WIN-01/R2-GEN-01/R2-DIAG-01; physical-shim A-versus-B manager-hint loading, no-repo-fallback, trace-output, policy-Git-source, missing-metadata, and unbound-init fail-closed cases, including zero filesystem access under B; Lima tests prove account-database control-root derivation, scrubbed/overwritten child `HOME`/`LIMA_HOME`, rejection of a directly injected conflicting internal projection, A-scoped future SSH UDS despite `vsock-proxy`/TCP availability, the R3 prerequisite before command spawn/socket mutation, and diagnostic transport output excluded from PM/product proof; Windows tests prove token Known Folder and canonical scope hash select the PID root, exact A selects config/logs, conflicting `LOCALAPPDATA`/`USERPROFILE` cannot retarget, and PM-derived WSL argv/environment wins over ambient target/`WSLENV`; existing mapping/backend/forwarder tests do not exercise macOS forwarding lifecycle; native evidence records exact OS/backend/tool versions, A/B setup, commitment, control root, mapping output, and the unactivated target. Static-only results remain explicitly pending. |
| GitNexus posture | HIGH/CRITICAL is expected for `managed_host_socket_path`, platform backend constructors, and shared transport state. Warn before editing; unrelated execution-flow impact, side-table selection, or a new backend authority stops the packet. |
| Independent reviews | Host-context/mapping security; cross-platform lifecycle R2/R3 ownership; allowlist, native-proof honesty, and regression sufficiency. All CLEAN. |
| Platform evidence | Static review on any host; separately assigned native supported macOS+pre-existing-Lima no-forwarder mapping-only run and native supported Windows+pre-existing-WSL mapping-only run. The macOS record may read the existing instance and must show the future A-scoped Unix socket, `/run/substrate.sock`, ambient transport availability non-authority, and the explicit R3 prerequisite; it must not start a forwarder, create/unlink a socket, kill/wait a child, or exercise handle drop. Full warm/smoke/lifecycle and macOS product transport are not R2-3 evidence. A missing native runner leaves the native assignment pending and never converts static evidence into native evidence. |
| Stop conditions | Platform instance/transport identity cannot be observed coherently; guest home/principal must be guessed; shared/per-prefix classification would require deletion semantics; a host path/UID is imposed in the guest; either WSL fail-closed guard would move or unreachable provisioning would activate; native evidence is mislabeled; allowlist expands. Architecture ambiguity/capability activation is `ArchitecturalBoundaryDecisionRequired`. |
| Next packet | A1.1d-5R2-4 — R2 integration and closeout. |

The R2-3 dependency surface is also closed. `crates/world-backend-factory/Cargo.toml`,
`crates/forwarder/Cargo.toml`, and `crates/shim/Cargo.toml` may each add exactly
`transport-api-types = { version = "0.2.8", path = "../transport-api-types" }`, respectively so
the factory accepts the shared typed mapping, the forwarder authenticates shared IH/PM carriers,
and the physical shim receives shared IH without a local duplicate. `Cargo.lock` may add only
`transport-api-types` to the existing dependency lists for `world-backend-factory` and
`substrate-forwarder`, and only `transport-api-types` plus `windows-sys 0.52.0` to
`substrate-shim`; no package/version/checksum or other package dependency list may change. The
shell `windows-sys` feature-only edit does not authorize a lock hunk. Any other dependency,
feature, target section, version, or lock change stops the packet.

The physical shim is the only additional OS-observation owner: within `context.rs`, the exact
invocation-witness constructor also resolves the current Unix account+UID or Windows account+SID
and performs the same no-follow Windows file-identity checks before constructing IH through the
shared type. `crates/shim/Cargo.toml` may add only `user` to its existing Unix
`nix = { version = "0.29", features = ["fs", "process"] }` feature list, and may add only
`windows-sys = { version = "0.52", features = ["Win32_Foundation", "Win32_Security",
"Win32_Storage_FileSystem", "Win32_System_Threading", "Win32_System_Com",
"Win32_UI_Shell"] }` under target Windows. The last two features are limited to token-bound
`FOLDERID_LocalAppData` resolution and freeing its returned buffer. The feature-only
`nix` change creates no lock hunk; the `windows-sys 0.52.0` shim lock entry above is its sole lock
effect. Native Unix tests cover canonical account/UID and
forged-principal rejection; static Windows compilation plus assigned native Windows tests cover
account/SID, no-follow file identity, and forged-principal rejection. No raw FFI, shim-local
principal type, ambient account variable, or other feature/dependency is authorized.

###### A1.1d-5R2-3 bounded docs-first subdivision

R2-3 is too broad for one implementation/review subject. The binding order is:

```text
R2-3D
  -> R2-3A -> R2-3B -> R2-3C
  -> R2-3M1 -> R2-3M2 -> R2-3M3 -> R2-3M4
  -> R2-3W1 -> R2-3W2 -> R2-3W3 -> R2-3W4 -> R2-3W5
  -> R2-3F -> R2-3R -> R2-3S1 -> R2-3S2 -> R2-3T
  -> R2-3ZD1 -> R2-3ZT1 -> R2-3ZH1 -> R2-3ZM5
  -> fresh native macOS/Windows evidence -> R2-3Z
```

This order is sequential. A later increment may be re-subdivided before implementation if its
pre-edit impact or source-closure evidence exceeds the frozen risk ceiling; it may not absorb an
earlier incomplete outcome. `R2-3D` alone was authorized by the initiating docs-first session. It
completed only the subdivision below and did not authorize or complete `R2-3A` by itself.

At that checkpoint, the published tip included four frozen prerequisites after the historical R2-2 publication:
`ebaf8941` keeps installer configuration private, `a009959b` stages gateway-smoke manifests as
trusted files, `343844c9` preserves install context in agent doctor, and `43e528af` calibrates the
bounded-review process. They remain prior work, not R2-3 implementation or proof. Reopening any of
them requires a concrete contradiction against current repository truth.

The parent R2-3 row above remains controlling. In the child records below, **owns** means the named
increment is the primary implementation/proof owner for that PI row; **supports** means the
increment supplies a prerequisite but may not claim that PI complete. PI-059 is a harness-only
call-site update; PI-077 and PI-078 are frozen capability guards; PI-050 remains an R2-4 guardrail;
PI-080 is already satisfied by R2-2. PI-047, PI-051, PI-053, PI-092–PI-102, and PI-113–PI-114
remain R3-exclusive. No child may claim the parent packet complete before R2-3Z.

`R2-3ZP3` is permanently deferred from the blocking path. Its unpublished commits
`2cb796ffef68c2b049376984a90ce0382e5f3980`, `9fa3fe0d4ed2933521dfcd67919d91aa9da6a499`,
`50948dbeb921582515a34bb6b7be21c46f29d008`, and
`868994efaf132bb04c6cdd7c82da9433333c94e5` are preserved only as diagnostic proof-infrastructure
evidence, are not accepted product source, and must not be cherry-picked, pushed, or represented
as landed. `R2-3ZD1` was the closeout-policy amendment only and completed no product proof by
itself.

Every child uses this same fingerprint/review method:

1. freeze the pre-edit commit and exact child file/test allowlist;
2. before its discovery fingerprint, run format or parser checks, focused tests, `git diff --check`,
   and an allowlist/status check;
3. hash a sorted manifest containing the pre-edit commit plus, for every exact subject path, its
   repository-relative path, Git mode (or `NEW`), and `git hash-object --no-filters` blob ID (or
   `MISSING`); SHA-256 of that manifest is the `subject_fingerprint`;
4. open a new V1 review-cycle record with packet ID equal to the child ID and validate it with
   `review-control/validate_review_cycle.py` after every returned cycle;
5. permit one complete-subject discovery burst, one consolidated P1/P2 remediation, one
   different-fresh closure, and at most two immediately causal supplemental cycles; demonstrated
   P1/P2 block, valid unfixed P3/P4 are added to or deduplicated in `06`, and CLEAN is terminal; and
6. run `gitnexus_detect_changes()` before any child commit. An unexpected production symbol,
   execution flow, file, dependency, or generated artifact stops the child.

Before editing any named existing symbol, run upstream GitNexus impact with tests included for that
exact symbol/file disambiguation and record direct callers, affected processes/modules, and risk.
New symbols record `N/A (new)` plus impact on the existing constructor/caller they replace or feed.
Shell/PowerShell functions that GitNexus does not index require the same pre-edit source-caller
closure by exact name. HIGH or CRITICAL is a warning-and-confirm checkpoint; an unreviewed
authority root, side table, lifecycle edge, or unrelated flow is a stop. `managed_host_socket_path`,
platform backend constructors, factory migration, shared pipe state, and forwarding boundaries are
presumptively HIGH/CRITICAL even if an index under-reports cfg-specific callers.

Across every child, the following remain frozen: host/guest path or principal equality; ambient
home/control-root/pipe selection; backend-selected home; WSL or Lima capability enablement; base
Lima profiles; VSock/TCP admission into V1 normal-product proof; policy/world/replay semantics not
named by the parent row; and all R3 deletion, replacement, rollback, teardown, timeout-kill,
wildcard, recursive, manifest, retry, and convergence authority. The exact WSL warm guard and
`scripts/wsl/provision.sh` exit-4 body remain byte-identical. Mac forwarding activation stops before
child launch until R3 lands PI-101/PI-113/PI-114 together. A frozen-byte change, allowlist
expansion, false native claim, or need to guess guest identity is
`ArchitecturalBoundaryDecisionRequired`.

**R2-3D — docs-only subdivision.**

- **Outcome/claim:** make the published current status truthful and freeze this exact child
  sequence. It completes planning only.
- **PI ownership:** owns no PI and completes no product gate.
- **Files/symbols/tests:** only `00-README.md` current-status text and this R2-3 subdivision in
  `03-phase-slice-map.md`; no production symbol, test, manifest, lockfile, installer, or script.
- **Dependencies/evidence:** requires published R2-2 plus the four frozen post-publication fixes.
  Markdown/diff/allowlist checks and GitNexus change detection are the complete D gate; no R2
  product proof gate is claimable and native evidence is not applicable.
- **Impact/review/stop:** no symbol impact call is required. Fingerprint exactly the two docs and
  use review record packet ID `A1.1d-5R2-3D`. Any product/test byte, PI completion claim, edit to
  `02`/`04`/`05`/`06` without a concrete contradiction, or authorization of R2-3A stops D.

**R2-3A — shared PM wire model.**

- **Outcome/claim:** add only the canonical, strict mapping/instance/transport/scope value model;
  no runtime consumer or platform mutation. This is the narrowest correct first product increment.
- **PI ownership:** supports PI-039–PI-041, PI-048–PI-049, PI-052, PI-054–PI-060, PI-075–PI-076,
  PI-079, PI-081, PI-090–PI-091, PI-112, and PI-115; completes none of those consumer rows.
- **Files:** only `crates/transport-api-types/src/lib.rs`.
- **Existing/new symbols:** existing IH types, path normalizers, and their behavior remain
  unchanged. Add exactly `PlatformInstanceIdentityV1`, `PlatformTransportIdentityV1`,
  `PlatformBootstrapMappingV1`, `WindowsForwarderScopeV1`, mapping member operations
  `new_lima`, `new_wsl`, `validate`, `encode`, and `decode`, scope member `derive`, and
  `normalize_windows_pipe_path`. No other public symbol.
- **Tests/proof:** colocated tests only:
  `platform_bootstrap_mapping_lima_golden_vector_is_exact`,
  `platform_bootstrap_mapping_wsl_golden_vector_is_exact`,
  `platform_bootstrap_mapping_rejects_noncanonical_or_tampered_records`, and
  `windows_forwarder_scope_and_pipe_normalization_are_canonical`; run the
  `transport-api-types` crate tests. These are the shared type prerequisites for
  R2-MAP-MAC-01/R2-MAP-WIN-01 and prove exact thirteen-line framing, outer unpadded base64url,
  commitment equality, canonical re-encoding, guest/host non-equality, pipe normalization, digest,
  and unknown/duplicate/reordered/tampered rejection.
- **Dependencies/native:** depends only on D and the existing IH model. Static wire proof only;
  native macOS/Windows mapping remains assigned to Z.
- **Impact/review/stop:** impact `normalize_windows_install_bootstrap_path`, `encode_inner_field`,
  `decode_inner_field`, `record_value`, `require_record_value`, and `is_lower_hex_digest` before
  reuse edits; new symbols are N/A and require context on `InstallBootstrapContextCarrierV1`.
  Packet ID `A1.1d-5R2-3A`. Any consumer/dependency/platform edit, alternate home selector, mapping
  side table, or lifecycle authority stops A.

**R2-3B — exact dependency edges.**

- **Outcome/claim:** make only the closed dependency/feature edges needed by later typed consumers;
  no runtime behavior.
- **PI ownership:** supports PI-054, PI-060, PI-090, PI-115, and PI-116; completes no PI.
- **Files/symbols:** only `crates/world-backend-factory/Cargo.toml`,
  `crates/forwarder/Cargo.toml`, `crates/shim/Cargo.toml`, and `Cargo.lock`; no Rust symbol.
  Add only the three exact `transport-api-types = { version = "0.2.8", path =
  "../transport-api-types" }` edges, shim `nix` feature `user`, shim target-Windows
  `windows-sys 0.52` features named by the parent, and the exact three lock dependency-list changes.
- **Tests/proof:** no test file and no product gate. Run the parent dependency-closure gate with
  locked metadata/build resolution and prove no package/version/checksum/target/other feature or
  dependency-list change.
- **Dependencies/native:** depends on A; native evidence is not applicable.
- **Impact/review/stop:** manifest-only, so no symbol impact call; inspect consumers
  `factory`, `PipeListener::new`, `spawn_bridge`, `ShimContext::from_current_exe`, and
  `ManagerHintEngine::new` before asserting need. Packet ID `A1.1d-5R2-3B`. Any shell manifest or
  unexpected lock hunk stops B.

**R2-3C — shell Windows principal and installed-witness observation.**

- **Outcome/claim:** provide the target-Windows OS observation needed to bind the current
  account+SID, token Known Folder, and installed release-copy witness; no installer or world
  consumer.
- **PI ownership:** supports PI-042–PI-046, PI-048–PI-049, PI-052, PI-068–PI-070, PI-076,
  PI-079, PI-081, and PI-090; completes none.
- **Files:** only `crates/shell/Cargo.toml` and
  `crates/shell/src/execution/install_bootstrap.rs`.
- **Existing/new symbols:** preserve all Unix functions. Add exact Windows counterparts
  `construct_windows_install_bootstrap_context`,
  `decode_and_bind_windows_install_bootstrap_context`,
  `bind_windows_install_bootstrap_context`,
  `current_windows_principal_and_known_folder`,
  `windows_known_folder_for_principal`,
  `resolve_windows_install_prefix_from_invocation`, and
  `require_same_windows_file_identity`; no raw FFI and no ambient account/folder variable.
  The shell manifest may add only the parent-row `windows-sys` features; no lock hunk.
- **Tests/proof:** the Windows-observation prerequisites of R2-MAP-WIN-01/R2-DIAG-01 use
  colocated target-Windows tests for canonical account/SID and token Known Folder, release-copy
  self-derivation under ambient B, no-follow exact file identity, forged principal, zero/multiple
  witness, and conflicting projection rejection; static Windows compilation where available.
  Native Windows use remains assigned to Z.
- **Dependencies/impact/review:** depends on A. Impact every existing Unix analogue before any
  shared-helper edit, especially `construct_unix_install_bootstrap_context`,
  `decode_and_bind_unix_install_bootstrap_context`,
  `bind_unix_install_bootstrap_context`, `current_unix_principal_and_home`,
  `resolve_unix_install_prefix_from_invocation`, and `require_same_file_identity`.
  Packet ID `A1.1d-5R2-3C`. A Unix behavior change, environment-selected principal/Known Folder,
  routing-file edit, new dependency/version, or lock hunk stops C.

**R2-3M1 — macOS helper mapping and two-stage observation.**

- **Outcome/claim:** make direct macOS helpers consume IH, derive the account-database Lima control
  root, perform only declared-instance Stage 1, and finalize/diagnose PM from an already-running
  guest before R2 projection.
- **PI ownership:** owns PI-039 and the macOS half of PI-081; supports PI-040–PI-041 and PI-075.
- **Files:** only `scripts/mac/lima-warm.sh` and `scripts/mac/lima-doctor.sh`.
- **Existing/new symbols:** edit `check_only_status`, `render_profile`, `vm_exists`, `vm_status`,
  `create_vm`, `start_vm`, `wait_for_running`, `ensure_vm_ready`, `configure_guest`, `diagnose`,
  `check_rendered_unit_parity`, and `run_breakglass_guest_checks`; add shell functions
  `resolve_install_bootstrap_context_v1`, `resolve_lima_control_root_v1`,
  `run_limactl_with_mapping_env_v1`, `observe_lima_mapping_v1`, and
  `verify_lima_mapping_v1`. `destroy_vm`, `stage_workspace`, cleanup bodies in
  `write_systemd_units`/`enable_socket_activation`, and all forwarding actions are frozen.
- **Tests/proof:** R2-MAP-MAC-01/R2-DIAG-01 use
  `tests/mac/lima_doctor_fixture.sh` plus new
  `tests/mac/prefix_mapping_r2_3.sh`; prove public B scrubbing, internal projection mismatch
  rejection, exact child `HOME`/`LIMA_HOME`, machine ID/account/UID/home observation, Stage order,
  and no delete/stage/unit/socket/forwarder action.
- **Dependencies/native:** follows A–C in the binding order but consumes only A and the existing IH
  conventions, not Windows-only C behavior. Static/focused tests only; Z owns the native
  pre-existing-Lima mapping-only record.
- **Impact/review/stop:** run GitNexus/file-qualified impact where indexed for every edited function
  and exact shell caller closure otherwise; `ensure_vm_ready` and `configure_guest` are
  HIGH-posture. Packet ID `A1.1d-5R2-3M1`. Need for guessed guest home, delete/rebuild, base-profile
  change, unit/socket cleanup, or forwarder launch stops M1.

**R2-3M2 — macOS typed backend primitives.**

- **Outcome/claim:** make Lima command, VM, socket, and backend construction consume typed
  control-root/mapping inputs without selecting ambient state; no forwarding activation.
- **PI ownership:** supports PI-041, PI-054–PI-056, PI-075, and PI-090–PI-091; completes no
  cross-caller PI.
- **Files:** only `crates/world-mac-lima/src/lib.rs`,
  `crates/world-mac-lima/src/transport.rs`, `crates/world-mac-lima/src/limactl.rs`, and
  `crates/world-mac-lima/src/vm.rs`.
- **Existing/new symbols:** edit `MacLimaBackend::new`, `MacLimaBackend::new_with_vm_name`,
  `ensure_vm_running`, `get_agent_endpoint`, `managed_host_socket_path`,
  `managed_host_socket_path_from`, `Transport::auto_select`, `limactl::path`,
  `limactl::command`, `LimaVM::new`, `LimaVM::status`, `LimaVM::ensure_running`,
  `LimaVM::start`, `LimaVM::wait_for_running`, `LimaVM::exec`, `LimaVM::info`, and
  `run_limactl`; add `MacLimaBackend::new_with_mapping`,
  `managed_host_socket_path_for_mapping`, and `limactl::command_for_control_root`.
  Existing diagnostic constructors may remain only explicitly labeled non-product.
- **Tests/proof:** the backend-primitive portion of R2-MAP-MAC-01 uses colocated tests only; prove
  A-scoped future socket, typed control root, scrubbed child environment, mapping mismatch
  rejection, and diagnostic transport non-authority without starting forwarding.
- **Dependencies/native:** depends on M1 and B. Static Rust proof only; Z owns native evidence.
- **Impact/review/stop:** impact every named existing Rust symbol; treat
  `managed_host_socket_path` and both backend constructors as CRITICAL regardless of cfg graph
  under-reporting. Packet ID `A1.1d-5R2-3M2`. An ambient fallback, backend-selected home,
  contextless normal constructor, or forwarding/lifecycle reachability stops M2.

**R2-3M3 — macOS forwarding boundary without activation.**

- **Outcome/claim:** fix the future PM-bound SSH-UDS target and A-scoped known-hosts projection,
  remove auto-selection from the validated path, and fail before forwarding child launch with the
  explicit R3 prerequisite.
- **PI ownership:** owns PI-041 and PI-112; PI-101/PI-113/PI-114 remain frozen R3 rows.
- **Files:** only `crates/world-mac-lima/src/forwarding.rs`,
  `crates/world-mac-lima/src/transport.rs`, and `crates/world-mac-lima/src/lib.rs`.
- **Existing/new symbols:** edit `forwarding::auto_select`, `create_ssh_uds_forwarding`,
  `lima_home_dir`, `lima_ssh_config_path`, `Transport::auto_select`,
  `MacLimaBackend::ensure_forwarding`, and `MacLimaBackend::get_agent_endpoint`; add
  `lima_home_dir_for_mapping`, `lima_ssh_config_path_for_mapping`, and
  `r3_forwarding_activation_required`. `ForwardingHandle::drop`, explicit/SSH-side unlink,
  `StreamLocalBindUnlink`, timeout kill/wait, VSock/TCP constructors, and retry/teardown bodies are
  byte-frozen.
- **Tests/proof:** R2-MAP-MAC-01/R2-GEN-01 use colocated tests only; prove fixed
  `A/sock/agent.sock` to `/run/substrate.sock`, exact `A/lima_known_hosts`, ambient
  VSock/TCP/SSH availability non-authority, and failure before command spawn or socket mutation.
  No current forwarding test may count as product proof.
- **Dependencies/native:** depends on M2. Static no-forwarder proof only; Z owns native mapping and
  macOS product transport remains pending R3.
- **Impact/review/stop:** impact every named function and `ForwardingHandle::drop` for frozen
  reachability; HIGH/CRITICAL is expected. Packet ID `A1.1d-5R2-3M3`. Any call to the current SSH
  constructor from the validated path or any lifecycle-byte change stops M3.

**R2-3M4 — macOS installer, unit, shell, and proof projection.**

- **Outcome/claim:** carry the already-selected IH/PM through dev/release macOS call sections,
  generated guest unit/socket projections, and host doctor/invocation surfaces without activating
  forwarding.
- **PI ownership:** owns PI-009, PI-022, and PI-040; supports PI-055 and PI-075, which F completes;
  PI-059 is harness-only.
- **Files:** only macOS call sections in
  `scripts/substrate/dev-install-substrate.sh` and
  `scripts/substrate/install-substrate.sh`,
  mapping/projection-only sections in `scripts/mac/lima-warm.sh`,
  `scripts/mac/lima/units/substrate-world-service.service.tmpl`,
  `scripts/mac/lima/units/substrate-world-service.socket`,
  `crates/shell/src/execution/invocation/plan.rs`,
  `crates/shell/src/execution/platform/macos.rs`, and call-site-only
  `crates/world-mac-lima/examples/mac_backend_smoke.rs`.
- **Existing/new symbols:** edit `provision_macos_world`, `install_macos`, the dev macOS
  `lima-warm.sh` call section, `write_systemd_units`, `configure_guest`,
  `ShellConfig::from_args`, `ShellConfig::from_cli`,
  `host_doctor_main`, `world_doctor_main`, `resolve_lima_vm_name`,
  `selected_host_visible_transports`, `try_bootstrap_host_visible_transport`,
  `collect_world_doctor_assessment`, and example `main`; add no new symbol. Cleanup statements
  inside `write_systemd_units`/`configure_guest` remain byte-frozen. Unit templates may project
  only the verified commitment/mapping/service/socket fields.
- **Tests/proof:** R2-MAP-MAC-01/R2-GEN-01/R2-DIAG-01 use mapping/context-only cases in
  `tests/mac/installer_parity_fixture.sh`,
  `tests/mac/lima_doctor_fixture.sh`, `tests/mac/prefix_mapping_r2_3.sh`, colocated shell tests, and
  example compile/call update. Prove no outer override, A/B isolation, unit/doctor commitment
  agreement, explicit R3 prerequisite, and no lifecycle action.
- **Dependencies/native:** depends on M1–M3. Static/focused proof only; Z owns native macOS record.
- **Impact/review/stop:** impact every named Rust symbol and exact shell caller closure; treat
  `ShellConfig::from_cli`, doctor transport selection, and backend construction as HIGH posture.
  Packet ID `A1.1d-5R2-3M4`. Non-mac installer edits, forwarding activation, unit/socket cleanup, or
  product-transport claim stops M4.

**R2-3W1 — Windows installer context and child propagation.**

- **Outcome/claim:** construct the same Windows IH for dev/release install and uninstall, carry it
  through profile/shim/doctor/guarded-WSL child calls including `-NoAutoSource`, and keep deletion
  semantics unchanged.
- **PI ownership:** owns PI-042–PI-046 and PI-068–PI-070.
- **Files:** only `scripts/windows/dev-install-substrate.ps1`,
  `scripts/windows/dev-uninstall-substrate.ps1`,
  `scripts/windows/install-substrate.ps1`, and
  `scripts/windows/uninstall-substrate.ps1`.
- **Existing/new symbols:** edit only prefix/context setup and existing shim/profile/doctor/WSL
  call sections; add PowerShell functions `Resolve-InstallBootstrapContextV1`,
  `Assert-InstallBootstrapContextV1`, and `Invoke-SubstrateWithInstallContextV1` where needed.
  Release-copy self-derivation must bind the installed binary and current token. Every recursive,
  wildcard, profile-removal, version/bin replacement, stop/unregister, and cleanup block is frozen.
- **Tests/proof:** R2-MAP-WIN-01/R2-GEN-01/R2-DIAG-01 use new
  `scripts/windows/prefix-mapping-r2-3.Tests.ps1`; prove default/custom A, ambient B, repeat
  propagation, `-NoAutoSource`, install/uninstall selection symmetry, current account+SID, forged
  carrier rejection, and zero B mutation. No lifecycle deletion is proof.
- **Dependencies/native:** depends on A–C. Static PowerShell proof only; Z owns native Windows.
- **Impact/review/stop:** exact source-caller closure for all edited top-level blocks and new
  functions; impact the shell Windows observation symbols they invoke. Packet ID
  `A1.1d-5R2-3W1`. A wildcard/recursive/action-predicate change, environment-selected Known Folder,
  guard movement, or deletion claim stops W1.

**R2-3W2 — Windows public helper and diagnostic mapping.**

- **Outcome/claim:** select/normalize the pipe once, validate IH/PM/current token at public helper
  boundaries, project the typed control root/scope, and preserve the WSL guard before mutation.
- **PI ownership:** owns PI-052 and the Windows half of PI-081; owns the selection edge of PI-079
  and supports PI-045, PI-049, PI-076, and PI-115.
- **Files:** only `scripts/windows/start-forwarder.ps1`,
  `scripts/windows/pipe-status.ps1`, `scripts/windows/wsl-warm.ps1`, and
  `scripts/windows/wsl-doctor.ps1`.
- **Existing/new symbols:** edit parameter/intake/diagnostic/child-argument sections and
  `Get-PipeNameFromPath`, `Normalize-WSLName`, `Get-InstalledWslDistros`,
  `Test-NamedPipe`, and `Get-ForwarderTargetInfo`; add
  `Resolve-PlatformBootstrapMappingV1`, `Assert-PlatformBootstrapMappingV1`,
  `Get-WindowsForwarderScopeV1`, and `Assert-CurrentWindowsPrincipalV1`.
  `start-forwarder.ps1` timeout kill and every statement from the `wsl-warm.ps1` guard onward are
  byte-frozen.
- **Tests/proof:** R2-MAP-WIN-01/R2-DIAG-01 use new
  `scripts/windows/prefix-mapping-r2-3.Tests.ps1`; prove canonical pipe, exact registered distro
  spelling/machine ID/guest identity, token Known Folder, scope digest, A-scoped future
  config/logs, shared PID target, ambient mismatch rejection, and exact guard bytes.
- **Dependencies/native:** depends on W1. Static/focused only; Z owns native existing-WSL evidence.
- **Impact/review/stop:** source-caller closure for each edited PowerShell function and impact
  `normalize_windows_pipe_path`/`WindowsForwarderScopeV1::derive`. Shared pipe/PID state is
  HIGH/CRITICAL posture. Packet ID `A1.1d-5R2-3W2`. Guard movement, provisioning activation,
  timeout kill, PID removal, or deletion-manifest semantics stops W2.

**R2-3W3 — Windows WSL typed backend.**

- **Outcome/claim:** make the backend/paths/transport/warm interfaces consume and revalidate the
  verified PM without environment/default reselection or enabling guarded provisioning.
- **PI ownership:** owns PI-048 and supports PI-057, PI-076, PI-079, PI-090–PI-091, and PI-115.
- **Files:** only `crates/world-windows-wsl/src/backend.rs`,
  `crates/world-windows-wsl/src/lib.rs`, `crates/world-windows-wsl/src/paths.rs`,
  `crates/world-windows-wsl/src/transport.rs`, and
  `crates/world-windows-wsl/src/warm.rs`.
- **Existing/new symbols:** edit `WindowsWslBackend::new`, `WindowsWslBackend::build`,
  `agent_transport`, `build_agent_client`, `ensure_agent_ready`, `ensure_ready`,
  `ensure_persistent_session_ready`, `WarmCmd::enabled`, `WarmCmd::run`,
  `detect_tcp_forwarder`, `to_wsl_path`, and `to_windows_display_path`; add
  `WindowsWslBackend::new_with_mapping`, `observe_wsl_mapping_v1`, and
  `validate_wsl_mapping_v1`. Existing TCP mode remains diagnostic-only.
- **Tests/proof:** the backend portion of R2-MAP-WIN-01/R2-DIAG-01 uses colocated tests and
  `crates/world-windows-wsl/src/tests.rs`, plus mapping-only cases in
  `scripts/windows/prefix-mapping-r2-3.Tests.ps1`; prove exact distro/guest
  machine/principal/home/pipe/commitment and failure before warm/provision on missing or conflicting
  PM.
- **Dependencies/native:** depends on W2 and B. Static backend proof only; Z owns native evidence.
- **Impact/review/stop:** impact every named Rust symbol; constructor and warm paths are
  HIGH/CRITICAL even when cfg indexing is incomplete. Packet ID `A1.1d-5R2-3W3`. Environment
  selection, guarded provisioning, guest-home guess, or transport redesign stops W3.

**R2-3W4 — Windows forwarder internal boundary.**

- **Outcome/claim:** require authenticated IH/PM and explicit A-scoped config/log paths in internal
  forwarder mode, validate token/distro/pipe/target/commitment, and transport but never delete the
  shared PID target.
- **PI ownership:** owns PI-049 and the forwarder-config consumption slice of PI-079; supports
  PI-060 and PI-115.
- **Files:** only `scripts/windows/start-forwarder.ps1`,
  `crates/forwarder/src/config.rs`, `crates/forwarder/src/logging.rs`,
  `crates/forwarder/src/windows.rs`, and `crates/forwarder/src/main.rs`.
- **Existing/new symbols:** edit `Cli`, `Cli::resolve_log_dir`, `windows::run`,
  `ForwarderConfig::load`, `default_config_path`, `resolve_target`, `logging::init`, and both cfg
  `main` entries; add `ForwarderConfig::load_internal` and
  `ForwarderConfig::validate_mapping`. Internal mode has no `LOCALAPPDATA`/`USERPROFILE` or target
  environment default. Timeout kill/PID removal/process termination remain frozen.
- **Tests/proof:** R2-MAP-WIN-01/R2-GEN-01 use colocated forwarder tests plus
  `scripts/windows/prefix-mapping-r2-3.Tests.ps1`; prove exact explicit config/log/PID paths,
  token/carrier/PM equality, environment conflict rejection, and diagnostic TCP non-promotion.
- **Dependencies/native:** depends on W2–W3 and B. Static proof only; Z owns native Windows.
- **Impact/review/stop:** impact all named Rust symbols and source-close the PowerShell launch.
  Shared-state and CLI default removal are HIGH posture. Packet ID `A1.1d-5R2-3W4`. Ambient
  selection, PID deletion/ownership manifest, timeout action, or stream behavior change stops W4.

**R2-3W5 — Windows pipe listener and PM-derived WSL leaf.**

- **Outcome/claim:** bind the named-pipe listener to the normalized PM pipe and make the WSL bridge
  child receive only PM-derived distro/target/commitment with overwritten target environment and
  `WSLENV`.
- **PI ownership:** owns PI-060 and PI-115 and completes the downstream-consumer slice of PI-079.
- **Files:** only `crates/forwarder/src/pipe.rs`, `crates/forwarder/src/bridge.rs`, and
  `crates/forwarder/src/wsl.rs`.
- **Existing/new symbols:** edit `PipeListener::new`, `normalize_path`, `serve`,
  `run_pipe_session`, `run_tcp_session`, `spawn_bridge`, and `wsl::spawn`; add no public symbol.
  `finalize_bridge`, `bridge_copy`, `WslStream` read/write/shutdown, and wait/stream semantics are
  frozen except exact argument plumbing required by the named callers.
- **Tests/proof:** R2-MAP-WIN-01 uses colocated tests plus
  `scripts/windows/prefix-mapping-r2-3.Tests.ps1`; prove commitment-bound pipe, conflict rejection,
  exact `wsl -d` registered spelling, exact UDS target/commitment, full overwrite of inherited
  target variables/`WSLENV`, and diagnostic TCP non-promotion.
- **Dependencies/native:** depends on W4. Static leaf proof only; Z owns native Windows/WSL.
- **Impact/review/stop:** impact every named Rust symbol; pipe listener and child spawn are
  HIGH/CRITICAL posture. Packet ID `A1.1d-5R2-3W5`. Pipe reselection, inherited target authority,
  provisioning, timeout kill, stop/unregister, or stream/wait semantic change stops W5.

**R2-3F — typed factory and shell platform callers.**

- **Outcome/claim:** require the explicit typed factory projection on macOS/Windows and migrate all
  shell platform, platform-world, gateway, and doctor callers; no contextless platform factory
  remains reachable.
- **PI ownership:** owns PI-054–PI-058 and PI-075–PI-076; supports PI-090–PI-091.
- **Files:** only `crates/world-backend-factory/src/lib.rs`,
  `crates/shell/src/execution/invocation/plan.rs`,
  `crates/shell/src/execution/platform/macos.rs`,
  `crates/shell/src/execution/platform/windows.rs`,
  `crates/shell/src/execution/platform_world/mod.rs`,
  `crates/shell/src/execution/platform_world/windows.rs`, and
  `crates/shell/src/builtins/world_gateway.rs`.
- **Existing/new symbols:** edit every cfg `factory`, `ShellConfig::from_args`,
  `ShellConfig::from_cli`, both platform `host_doctor_main`/`world_doctor_main`,
  `connect_transport_stream_ws`, platform `detect`, Windows `context`, `get_backend`,
  `build_agent_client`, `build_macos_gateway_client`, `resolve_macos_gateway_client_endpoint`,
  `resolve_macos_host_gateway_socket`, `macos_default_world_socket_path`, and
  `build_gateway_client`; add only the `PlatformWorldContext::bootstrap_mapping` field. Linux
  factory behavior stays platform-independent.
- **Tests/proof:** R2-MAP-MAC-01/R2-MAP-WIN-01/R2-DIAG-01 and the applicable unchanged
  R2-RUNTIME-01 differential use colocated tests in the named files, mapping-only macOS/Windows
  focused tests, and the unchanged gateway differential; prove explicit shell callers,
  contextless platform failure before client/backend selection, Linux parity, and no
  forwarding/provisioning action.
- **Dependencies/native:** depends on M4 and W5. Static call-graph proof only; Z owns native runs.
- **Impact/review/stop:** impact every named Rust symbol; factory's indexed LOW result does not
  waive manual cfg source closure, and platform constructors remain HIGH/CRITICAL posture. Packet
  ID `A1.1d-5R2-3F`. A missing caller, new backend authority, Linux semantic change, lifecycle
  reachability, or gateway contract change stops F.

**R2-3R — replay factory-context migration.**

- **Outcome/claim:** carry explicit bootstrap/factory input through shell replay, public replay
  config, planner, and executor only; macOS/Windows direct library world calls without it fail before
  factory construction.
- **PI ownership:** owns PI-091.
- **Files:** only `crates/shell/src/execution/routing/replay.rs`,
  `crates/replay/src/lib.rs`, `crates/replay/src/replay/mod.rs`,
  `crates/replay/src/replay/planner.rs`, and
  `crates/replay/src/replay/executor.rs`.
- **Existing/new symbols:** edit `handle_replay_command`, `ReplayConfig`, `replay_span`,
  `replay_batch`, `ExecutionState`, `execute_in_world`, `replay_sequence`,
  `execute_with_world_backends`, and `try_world_backend`; add only
  `ReplayConfig::platform_bootstrap_mapping` and its exact internal `ExecutionState` projection.
  Recorded command/environment/origin/policy/timeout/strategy and agent-fallback semantics are
  byte-behavior frozen.
- **Tests/proof:** the factory-caller portions of
  R2-MAP-MAC-01/R2-MAP-WIN-01/R2-SHIM-01 use `crates/replay/tests/integration.rs`,
  `crates/replay/tests/planner_executor.rs`, and factory-context-only cases in
  `crates/shell/tests/replay_world.rs`; prove explicit platform projection, pre-factory failure when
  absent, Linux parity, and unchanged replay differentials.
- **Dependencies/native:** depends on F. Static/focused replay proof; Z owns native platform
  mapping, not replay semantic reproof.
- **Impact/review/stop:** impact every named symbol, especially `ReplayConfig`,
  `execute_with_world_backends`, and `try_world_backend`. Packet ID `A1.1d-5R2-3R`. Any replay
  semantic/schema/timeout/strategy change or ambient factory fallback stops R.

**R2-3S1 — physical-shim IH, manager, and factory binding.**

- **Outcome/claim:** recover exactly one no-follow invocation witness for the physical shim, bind
  the current OS principal, and use that IH for A-only manager manifests and explicit telemetry
  factory projection before dispatch.
- **PI ownership:** owns PI-090 and PI-116; supports PI-118.
- **Files:** only `crates/shim/src/context.rs`, `crates/shim/src/exec/mod.rs`, and
  `crates/shim/src/exec/logging.rs`.
- **Existing/new symbols:** edit `ShimContext::from_current_exe`, `resolve_invoked_path`,
  both cfg `find_candidate_in_dir`, `check_candidate`, `run_shim`,
  `collect_world_telemetry`, `ManagerHintEngine::new`, `manifest_paths`,
  `manifest_overlay_path`, and `repo_manifest_path`; add
  `resolve_install_bootstrap_context_from_invocation` and
  `current_platform_principal_v1`. PATH/CWD may resolve a unique witness but never prefix
  precedence; repo/ambient/manifest-environment fallback is unreachable in normal product mode.
- **Tests/proof:** R2-SHIM-01/R2-GEN-01 and the shim factory portions of
  R2-MAP-MAC-01/R2-MAP-WIN-01 use colocated tests and
  `crates/shim/tests/integration.rs`; prove absolute/relative/bare invocation, zero/multiple
  candidates, exact current Unix account+UID or Windows account+SID, no-follow identity, forged
  principal rejection, A-base/A-overlay manager hints, no repo fallback, explicit telemetry
  mapping, and zero B access.
- **Dependencies/native:** depends on B, C, and F. Native Unix shim proof may be recorded here;
  native Windows mapping remains assigned to Z.
- **Impact/review/stop:** impact all named symbols; graph LOW for `run_shim` does not waive the
  physical process-root review. Packet ID `A1.1d-5R2-3S1`. Multiple-witness guessing, ambient/repo
  selection, manager semantic change, recursive shim action, or lifecycle change stops S1.

**R2-3S2 — physical-shim trace and policy projection.**

- **Outcome/claim:** bind explicit product trace/policy inputs from the S1 IH before manager,
  policy, telemetry, span, or execution logging and reuse that one binding throughout the physical
  shim.
- **PI ownership:** owns the physical-shim portion of PI-118; T remains the final compatibility
  owner.
- **Files:** only `crates/shim/src/exec/mod.rs`,
  `crates/shim/src/exec/policy.rs`, `crates/shim/src/logger.rs`, and exact invocation/factory
  projection in `crates/shim/src/exec/logging.rs`.
- **Existing/new symbols:** edit `run_shim`, `evaluate_policy`, `start_span`,
  `log_execution`, `write_log_entry`, and `collect_world_telemetry`; add no public symbol.
  `evaluate_policy` and logging reuse the already-bound context and cannot initialize or select
  ambient state.
- **Tests/proof:** R2-SHIM-01/R2-DIAG-01 use colocated tests and
  `crates/shim/tests/integration.rs`; prove only
  `A/trace.jsonl`, policy Git A, repeated A reuse, conflicting-path rejection, missing-metadata
  no-fallback, and zero B filesystem access. Rotation/retention/writer/span/policy semantics are
  frozen.
- **Dependencies/native:** depends on S1 and R. Native Unix physical-shim evidence may land here;
  Z owns final cross-platform accounting.
- **Impact/review/stop:** impact every named symbol and `TraceContext::explicit_product` as the
  consumed boundary. Packet ID `A1.1d-5R2-3S2`. Any default trace init, environment-selected policy
  directory, writer/rotation/retention change, or side table stops S2.

**R2-3T — final trace compatibility closure.**

- **Outcome/claim:** only after F/R/S1/S2 callers are migrated, remove or make unreachable
  `LegacyAmbientCompatibility`, make global unbound `init_trace(None)` fail, and remove legacy
  ambient policy lookup without changing the neutral setter.
- **PI ownership:** owns final completion of PI-118.
- **Files:** only `crates/trace/src/context.rs`, `crates/trace/src/util.rs`, and
  `crates/trace/src/tests.rs`.
- **Existing/new symbols:** edit `TraceContextBindingV1`, `TraceContext::default`,
  `TraceContext::init_trace`, global `init_trace`, `get_policy_git_hash`, and exact tests;
  `set_global_trace_context`, `TraceContext::explicit_product`,
  `get_policy_git_hash_at`, writer/rotation/retention/span/replay bodies, and signatures remain
  frozen. Add no public symbol.
- **Tests/proof:** final R2-SHIM-01/R2-DIAG-01 closure uses `crates/trace/src/tests.rs`; replace the
  legacy-default expectation with final unbound failure and retain explicit-product A/B, policy
  Git, repeated-init, conflict, symlink, missing-metadata, writer, rotation, and retention tests.
- **Dependencies/native:** depends on F, R, S1, and S2 plus source closure proving no owned legacy
  caller remains. Native evidence is not independently required beyond migrated caller proof.
- **Impact/review/stop:** impact every edited symbol and query all callers of global `init_trace`
  and `get_policy_git_hash` before edit. Packet ID `A1.1d-5R2-3T`. Any unmigrated owned caller,
  setter semantic change, caller-identity table, lifecycle/writer change, or unrelated trace schema
  change stops T.

**R2-3Z — integration evidence and closeout.**

- **Outcome/claim:** `R2-3ZT1`, `R2-3ZH1`, `R2-3ZM5`, and the refreshed native macOS/Windows
  evidence are complete. This closeout joins the completed child increments, proves the complete
  R2-3 parent exit gate through accepted direct product/static checks plus fresh native evidence,
  records the canonical-runner provenance limitation truthfully, and hands only later-owner R2-4
  work plus R3-owned lifecycle/forwarding/provisioning/cleanup/rollback/convergence work forward.
  No production repair is allowed.
- **PI ownership:** closes only after code+proof for PI-009, PI-022, PI-039–PI-046, PI-048–PI-049,
  PI-052, PI-054–PI-058, PI-060, PI-068–PI-070, PI-075–PI-076, PI-079, PI-081, PI-090–PI-091,
  PI-112, PI-115–PI-116, and PI-118 is present. PI-059 remains harness-only; PI-077/PI-078 remain
  byte-frozen fail-closed guards; PI-050 remains the R2-4 guardrail; PI-080 remains satisfied by earlier R2-2 Linux
  restart-scope work and is not reopened here; and every R3 lifecycle/forwarding/provisioning/
  cleanup/rollback/convergence row retains the ownership above.
- **Files/symbols:** no production file or symbol. Evidence transcription may touch only the five
  existing runtime-refactor control-pack Markdown files in this bounded closeout subject and only
  to record current proof; any
  product defect returns to its owning child under a new fingerprint/review sequence.
- **Canonical-runner limitation:** `R2-3ZP3` is permanently deferred from the blocking path.
  Unpublished commits `2cb796ffef68c2b049376984a90ce0382e5f3980`,
  `9fa3fe0d4ed2933521dfcd67919d91aa9da6a499`,
  `50948dbeb921582515a34bb6b7be21c46f29d008`, and
  `868994efaf132bb04c6cdd7c82da9433333c94e5` remain diagnostic evidence only. Honest
  authenticated wall results that still finalize ineligible with `evidence_write_failed` and
  `mount_teardown_failed` remain proof-infrastructure work outside the R2-3 gate.
- **Tests/proof:** the accepted direct shell-library result at final source
  `c583c5f293644fab75d8d42bd3bcad63f114d4fe` is
  `1322 discovered / 1274 passed / 48 failed / 0 ignored`, with failure-name SHA-256
  `c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9` and normalized-signature
  SHA-256 `2a0df9b340cc7e5e1b6e4f76e60e6f937b7442008142d78a7ae24bbcd2f60a90`. Count-only
  equivalence remains insufficient: the historical 45-failure inventory must remain present, and
  the only additional failures may be the three separately classified non-R2-3 world-deps/report
  expectations. `R2-3ZH1` owns only the host-inbox trusted-root test helper. `R2-3ZM5` owns only
  the macOS contextless-constructor removal and typed pre-R3 smoke contract. Their focused tests,
  package checks, formatting/Clippy/workspace checks proportionate to the touched crates, exact
  installer/script parser and frozen-byte checks, R2-SHIM-01, R2-GEN-01, R2-DIAG-01,
  R2-MAP-MAC-01, R2-MAP-WIN-01, applicable R2-RUNTIME-01 regression, allowlist/diff/GitNexus
  checks, and the parent independent review lenses remain mandatory.
- **Current broad-wall authority:** the only normative public entrypoints for this exact source are
  `make shell-lib-wall` and `make shell-lib-wall-serial`. The tracked Python runner remains
  historical diagnostic evidence only and its provenance-ineligible `1322 / 1277 / 45 / 0` result
  never overrides the accepted direct Make baseline.
- **Native macOS:** the refreshed source-bound macOS receipt
  `sha256:3b44f6387070b7aaea4306ae58d7f280b1cee3e163ee59219d4900ce3f53dfaf` is
  `EVIDENCE_CLEAN` for `R2-DIAG-01` and `R2-MAP-MAC-01` at source
  `c583c5f293644fab75d8d42bd3bcad63f114d4fe` / tree
  `a070f5f5787c27180f13dece9a1c3c3241728fda`, with artifact digest
  `sha256:64a726d45b8bb6724fe43f566903d7b5e5ed6ef16119f14d30cd5a03edcc7e50`. It records supported
  macOS/Lima/tool versions, the typed account-database-derived Lima control root, and the future
  `A/sock/agent.sock` to `/run/substrate.sock` target without starting a forwarder or claiming any
  socket/process lifecycle action.
- **Native Windows:** the refreshed source-bound Windows receipt
  `sha256:4ee942690655c1fac185244438d14e2561df52c306dea7e5428d556b530fd28c` is
  `EVIDENCE_CLEAN` for `R2-DIAG-01` and `R2-MAP-WIN-01` at that same source/tree, with artifact
  digest `sha256:2c36a8dae9f3bcfa1c240c62a1e44f93aee0798702ae78a0076d43778b93d46e`. It records the exact
  account+SID, registered distro/machine ID, normalized pipe/scope digest/shared PID root, and
  PM-derived WSL argv/environment under conflicting ambient values without provisioning, timeout
  kill, stop, PID deletion, unregister, or cleanup claims. Static proof never substitutes for a
  missing native assignment.
- **Impact/review/stop:** no pre-edit symbol impact unless a defect is returned to an owner.
  The accepted closeout subject uses packet ID `A1.1d-5R2-3Z` and a fresh bounded-review record
  over host-context/mapping-security, lifecycle/R2-versus-R3, and
  allowlist/native-honesty/regression lenses. Any production fix, false native claim, false
  canonical-runner-eligibility claim, incomplete PI/proof, changed guard/frozen byte, open P1/P2,
  or R3 authority leakage still prevents closeout. This row is complete only for R2-3; R2-4 and
  later R3 work remain next.

**Source provenance:**
- extracted from [`03-phase-slice-map.md#a11d-5r2-3--platform-native-mapping-adapters`](../03-phase-slice-map.md#a11d-5r2-3--platform-native-mapping-adapters), lines 693–1339; baseline span SHA-256 `29481d489a1624e2c0ca44d2c63f4c996496b772d96ba7381132a14be40d5305`
