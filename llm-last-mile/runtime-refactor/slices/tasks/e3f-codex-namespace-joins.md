# E3-F same-owner Codex namespace joins

**Kind:** task evidence in the existing E3-F control plane.
**Status:** component implementation and local proof recorded; owner inspection and fresh material
review remain pending. This is not E3-F acceptance, production adoption or closure.

## Frozen component specification and scope

Implement only `E3GatewayRuntimeAuthorityV1::install_codex_user_namespace_v1` and
`release_codex_user_namespace_v1`, their necessary private binding validation, and directly
necessary colocated tests in `crates/world-service/src/gateway_runtime.rs`. Both operations use
the exact cataloged signatures and `pub(super)` visibility. The existing lease helper in
`e3_child_security.rs` is consumed unchanged. This task evidence and a narrow E3 slice note are
the only additional product files in the packet; runtime notes and review records remain owner-owned.

**Exact completion claim:** given an existing gateway attempt and its original exclusion lease,
the two operations validate exact attempt/member registration bindings, delegate setup and terminal
namespace release through that same lease, preserve partial progress, reject foreign/duplicate
ownership, and support exact release retry while the registered cgroup exists.

Authority remains the [retained callable joins](../../contracts/agent-config-projection-v1.md#e3-f-retained-ownership-and-callable-joins),
[I1/I2 reconciliation](../../contracts/agent-config-projection-v1.md#e3-f-i1i2-ownership-and-accounting-reconciliation),
[future fence](../../contracts/agent-config-projection-v1.md#e3-f-future-symbol-fence-and-proof-accounting),
[gateway cleanup](../../contracts/managed-gateway-adoption-v1.md#e3-f-retained-successor-and-cleanup-joins),
and [development review contract](../../contracts/development-review-and-remediation-contract.md).
The task owner explicitly selected this one packet after a fresh independent pre-edit observation;
implementation followed successful bounded qualification of the existing namespace helper.

Baseline: clean `c4b22d499cd29b0df5362c909e7b9e9aaf31e037`, tree
`591138ddd812af93e9ee23a9cc1a63bd68b963dd`, Linux x86_64 and Rust/Cargo 1.89.0.
The preserved `eeb17d952` ancestry/checkpoint delta and native prerequisite remain intact.
Native `lib.rs`, `registry.rs` and `service.rs` hashes still equal the retained native CLEAN subject;
its `P2-CUSTODY-1` and `P2-PARENT-1` findings/resolution evidence are not rewritten or reopened.

## Behavior and ownership decisions

A shared private validator joins the child to the retained gateway launch, original gateway/member
cgroup registrations and intents, store/series/fence, preparation, service and current boot identity.
It checks complete member registration and intent hashes, the unique original member group,
held parent/child descriptors, exact named-object identities and root-owned non-writable cgroup-v2
objects. Caller hashes alone do not establish ownership. Directory validation completes before the
namespace handshake; no filesystem-authority transaction or registry lock spans a process wait.

Install additionally requires open, nonterminal admission, exact original identity and namespace
mapping requirements, and the actual live PID/start/cgroup before entering setup. The consumed lease
repeats process observations after the handshake and retains its existing partial slot before effects.
An error may already own namespace custody; duplicate setup is rejected, never replayed.

Release uses the same original lease and registration, without requiring a live child, open gates,
a setup attestation or an unexpired activation deadline. It remains usable during terminal cleanup.
`NeverRetained` means namespace nonownership only: it does not prove child terminality or cgroup
emptiness. A held namespace releases only after actual child termination and current empty-tree
proof. Exact retry uses the existing Released slot; the original cgroup must still exist.
Later Revoking/accounting callers retain their separate obligation to establish early denial before
late cleanup effects. These joins neither grant that permission nor implement whole-attempt cleanup.

| Mechanism | Required capability / existing owner | Purpose and disposition |
|---|---|---|
| Original gateway `_exclusion` | E3-D/gateway security custody | Retain sole ownership; borrow in place, never expose, clone or reacquire |
| Existing namespace slots and dedicated setup thread | E3-D mapping, partial progress and exact release | Consume unchanged; no parallel namespace ledger or lifecycle owner |
| Original cgroup descriptors/registrations/intents | Same-attempt containment | Revalidate objects and preparation; namespace release before group removal |
| Native publication/inputs and original Held lease | Projection service/native component | Preserve unchanged; no reconstructed capability or new resource transfer |
| V1/UAA launchers, copied host configuration/auth and fallback | Runtime-family/E3 adoption owners | Not used or newly retained here; existing replacement/removal debt remains separately owned |

## Dependency and semantic-status capsule

The adapter's own absent implementation is candidate-owned work, not a readiness veto. Its external
namespace, wrapper and gateway-accounting dependencies are not yet all implemented; this packet
selects only the exact missing namespace prerequisite.

| Join | Present source / production status | Remaining obligation |
|---|---|---|
| Configured authority | Service/preparation authority exists; member manager lacks paired service/registry injection | Fail-closed paired injection before transfer |
| `take_for_v2` | Implemented; no production caller | Consume unchanged once; immediately retain provisional cleanup custody |
| Native inputs | Retained native component exists with its original bounded proof | Real artifact/installed/runtime adoption remains unproved |
| Same-owner Codex namespaces | Two component operations added; real kernel seam exercised | No adapter/production caller added |
| Wrapper/loader/final gate | Incomplete Codex branch and placeholder fingerprint remain | Exact source/realization/mounted-object joins and barrier rechecks |
| Gateway accounting/denial | I2 Codex history, ACK and early-denial joins remain absent | Complete history, exact custody/readback, denial-before-late-effects |
| Service Active/resume | Registry original-lease/Active guards exist | Service release/resume operations and genuine release ordering |
| I1 ownership/admission | Common non-E3 fields remain; no `launch_v2` | Complete tagged NonE3/E3 variants and lifecycle routing |
| Production/bridge/F14 | V2 remains rejected | Authentic launch/cleanup and later separately selected producer/integrated proof |

Ordered dependencies: native prerequisite -> this namespace component -> remaining wrapper/accounting
prerequisites -> adapter/service release integration -> tagged ownership and authentic admission ->
production proof -> separately selected bridge/F14 work. This is not successor authorization.
No new schema, dependency, lease accessor, placeholder product owner or generic callback was added.
No source under shell/REPL/bridge, B01/B03, E2, native projection, existing gateway/probe/revoke paths,
other platforms or persistent services was changed. E3-E stays closed; enclosing E3 remains incomplete.

## Actual tests and failure-to-fix evidence

Raw evidence root:
`/home/spenser/__Active_code/review-evidence/e3f-codex-namespace-joins-20261022/`.
Every `capture-check.py` command record preserves argv, cwd, source-before/after hashes and diff
identity, exit, elapsed time and separate raw stdout/stderr. The scripts are external one-shot raw
proof support, not product supervisors, new acceptance authority or runtime records.

Actual new selectors in `gateway_runtime::tests`:

- `test_e3f_codex_namespace_missing_attempt_rejects_without_setup` — unprivileged rejection and
  exclusion retention without a launch/setup slot. `red-missing-join-exact` failed specifically on
  the absent method; `green-missing-attempt` passed after implementation. The earlier first RED
  also exposed a test import error; it is retained separately, not counted as a behavioral baseline.
- `privileged_e3f_codex_namespace_owner_lifecycle` — explicitly ignored in ordinary library runs,
  separately executed with `--exact --ignored --nocapture --test-threads=1` and the existing explicit
  `SUBSTRATE_E3D_*` controls. Its assertion matrix covers both gateway methods: original setup;
  foreign store/series/fence/role/service/boot/registration/cgroup and identity; same-world siblings;
  substituted held parent/child descriptors; wrong preparation; closed installation; duplicate
  setup; malformed setup before retention; mapped-reply loss after retention; live child and
  populated descendant rejection; terminal/deadline-independent release; exact outer-ACK retry;
  namespace-FD closure before group removal and exclusion release last.

The privileged fixture has real children, PID/start/pidfd, separate user namespaces, actual UID/GID
maps and actual cgroups/descriptors. Its retained launch metadata and inert artifact descriptors
are nonsecret equality fixtures: they are not gateway startup, authenticated registry preparation,
artifact execution or production evidence. No executable impersonates Codex or a gateway.
The ordinary NeverRetained case proves namespace nonownership while the actual child remains live.
Held cases prove exactly one matching namespace FD remains after failed release and zero after
successful release, while the original group still exists. Three namespace setup outcomes share
one original member group/lease; this does not claim Codex session/resume behavior.

The first gateway seam run (`seam-01`) timed out at a self-consistent wrong-start negative. It was
not a pass. Exact pidfd kill/reap and empty-group cleanup completed. The cause was entering the
lease socket wait before observing the live start identity. Install now checks PID/start/cgroup
before slot/setup entry; release retains its post-reap eligibility. The same matrix passed in
`seam-02`; added descriptor/closed-state/namespace-FD assertions and final formatting were verified
again in `seam-final`. No assertion or failing path was disabled.

## Verification and limits

All Cargo commands below use `+1.89.0`, `--locked`, and the explicit implementation checkout cwd.

| Command / evidence label | Actual result |
|---|---|
| `cargo test -p world-service --lib --no-run --message-format=json` / `build-seam-final` | Exit 0; emitted executable bound by path/SHA-256 in `seam-binary-final.json` |
| Explicit root gateway selector / `seam-final-invocation` | Exit 0; 1 passed, 0 failed, 0 ignored; bounded to 20s |
| `cargo check -p world-service` / `verify-world-check` | Exit 0; inherited shell warnings retained |
| `cargo test -p world-service --lib` / `verify-world-lib` | Exit 0; 223 passed, 8 ignored, 0 failed; includes unchanged V2 pre-effect rejection |
| `cargo test -p config-projection --lib` / `verify-config-lib` | Exit 0; 65 passed, 6 ignored, 0 failed; existing child subprocess result separately retained |
| `cargo test -p config-projection --test agent_config_projection_v1` / `verify-config-integration` | Exit 0; 6 passed |
| `cargo test -p config-projection --doc` / `verify-config-doc` | Exit 0; 1 passed |
| `cargo clippy -p world-service --all-targets --no-deps -- -D warnings` / `verify-world-clippy` | Exit 0; no package warning suppression added; inherited dependency warnings remain distinct |
| Scoped `rustfmt --edition 2021 --check` / `verify-rustfmt`; `git diff --check` | Exit 0 |

The existing helper facility was first qualified separately against its baseline source/binary.
Meta interpreted current canonical contracts plus that explicit test as operative guidance; the
missing `docs/HOWTO_PRIVILEGED_TESTS.md` remains unavailable, and unseen requirements are not claimed
satisfied. A disposable parent/child timeout cleanup was exercised before the helper and each seam
run. The external controller binds pidfds/start/boot, confines only its test processes, acts as a
local subreaper, and removes only captured exact empty disposable groups after reap. Final raw
source/binary/identity/mapping/cleanup output is in `seam-final-*`; the postcheck found all seven
observed test/preflight processes and all eight captured groups absent. No broad kill, service
namespace/policy change, startup/restart, provisioning or installation occurred.

The new privileged test's ignore is an explicit execution boundary, not a pass in the library
count. Other ignored cases remain unavailable unless covered by their original separately retained
proof. No activation test, full F14, installed campaign or baseline differential is claimed here.

Required installed diagnostic availability was inspected before invocation. The configured
`/home/spenser/.substrate/bin/substrate` symlink points to an absent executable; PATH lookup finds
no available `substrate`. Therefore `world doctor --json`, `shim doctor --json` and `health --json`
were not executed. Source inspection also identifies public startup home-scaffold/trace effects
and potential world-service connection/execute behavior. No diagnostic workaround, repair or service
activation was attempted. `installed-diagnostics-availability.json` retains exact intended commands,
paths and source observations. Installed artifact/source correspondence and runtime health remain
unavailable, separate from this candidate's component proof.

Pre-edit GitNexus on the exact checkout index reported LOW: owner struct/impl zero indexed callers
or flows; consumed installer two callers/one test flow; consumed release four callers/one flow.
Source inspection finds omitted gateway/probe callers, so these counts are incomplete coverage,
not a safety guarantee. `pre-edit-impl-impact.json` binds the refreshed original owner-container
analysis. New methods have no prior graph symbols. No existing method body or test assertion was
removed. Owner-side change detection remains required before the authorized eventual commit.

The final owner-inspection source manifest includes all three changed/new product files with modes
and hashes; tests are contained in the gateway source. Material discovery, any consolidated causal
repair/fresh review and owner acceptance/publication remain pending. A local pass is not CLEAN.
All I1/I2, complete historical accounting/ACK/denial, authentic adapter, Active/Registered,
production V2, installed and bridge/F1–F14 obligations outside this component remain with E3-F.
