**Kind:** evidence and regression record
**Stable ID:** `A1.2-earlier-histories-family`
**Status:** canonical historical/completed-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Supersedes:** canonical ownership of the extracted source bodies; source headings/rows remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)
**Canonical for:** A1.2 and earlier A0/A1.1d/A1.1e completion, proof, and regression history only
**Source span:** composite of the six preserved root compatibility spans listed in the extraction ledger

# A1.2 and earlier packet-histories evidence and regression record

## A1.2b recorded result

At the bound Tuesday, August 4, 2026 source candidate, A1.2b completes the bounded internal
successor/post-turn packet without public CLI/helper/REPL/auto-attach adoption and without
beginning R3 or A1.3. HostSessionAuthority now publishes only one strict V2-to-V3 root extension
that preserves every strict V2 Start intent, both R0 retained-registration maps, existing
application-proof bytes, and the exact current-authority/R0 lineage corridor without widening
older roots or down-converting V3. Exact current-authority resolution is closed-version aware, the
retained-admission corridor consumes the preserved V2 Start view under V3, and startup acceptance
continues to authenticate the original Start application revision only through one unique
contiguous R0 registration-proof chain to exact current authority.

Successor `Attach`/`ResumeOneTurn` issuance, claim/application, input acceptance, startup/post-turn
reconciliation, `AwaitingObligationCut`, unchanged C1 `Pending`/`Complete` consumption, immutable
journals/results, exact retry, transport reprojection, and `ReleaseEligible` handoff now remain
entirely inside HostSessionAuthority without classifying or rewriting ledger truth. Complete
snapshots use the unchanged C1 disposition directly; pending, incomplete, mismatched, or
substituted cuts remain pending or fail closed. Exact result join survives a separately validated
`Released` transport state through durable reprojection, but this packet does not claim the
destructive payload-deletion/root-advance step itself.

Focused host-session-authority strict codec/store/transition/reconciliation tests and retained
admission compatibility tests are green. Both `make shell-lib-wall` and
`make shell-lib-wall-serial` retain the exact 48-failure inventory and the accepted hashes
`c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9` /
`e53ffb35dbd4fe32ea60ad8da88efe449edc510a5bd0b3fe446e8a368d5beb40` while growing broad proof to
`1351 discovered / 1303 passed / 48 failed / 0 ignored`, exactly twenty-one new passing shell
tests, and zero retained failure-name/message/assertion/test-identity/behavior drift. The accepted
differential is recorded in
[`review-control/a1-2b-differential-evidence.json`](../review-control/a1-2b-differential-evidence.json).
A1.2b is complete only as the bounded internal durable successor/post-turn protocol; no seam is
promoted, public consumer adoption remains outside A1.2b and now belongs to the active
[A1.3-P1 Linux-first atomic public-adoption packet](../linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md),
while the held A1.3-P0 and A1.3 records remain preserved as non-executable historical fences. The
then-current historical gate was `AUTHORITY_REQUIRED:R3_RESUME`.

## A1.2a, A1.2a-WB, and A1.2a-S recorded result

A1.2a is landed and published through `b5f2b4f8dd7d9f650c462cd4626a562cacc1d27f`. Its bounded
commit chain is strict V2 greenfield upgrade `ab2af5a4`, strict V2 dispatch `91d7e491`, atomic Start
issuance `eba02f17`, atomic claim/application `e25896a9`, terminal-handoff hash-input documentation
`a66096be`, and terminal/current-authority read completion `b5f2b4f8`. Independent review is clean.
The exact shell-library baseline at that commit is `876 passed / 171 failed / 0 ignored`; the 171
failures are the inherited legacy-writer/preflight set.

The newly bounded prerequisite order is **A1.2a → A1.2a-WB → A1.2a-S → B1/B2.1-R0**.
A1.2a-WB owns only the Host/world-binding write/read correction in `transition.rs`, its colocated
`transition_tests.rs`, and `facade.rs::HostSessionAuthority::resolve_current_exact`. The reader must
accept and return the exact authority persisted under the same four-case matrix; all other facade
behavior remains outside scope. The packet does not change schemas, canonical JSON bytes, golden
vectors, V1/V2 fixtures, migrations, compatibility behavior, already-persisted objects, authority
fields, or world capability/policy enforcement. Its docs commits are `09b7ba6d` and `7ab20f1c`;
runtime commit `275f9fa2` is independently review-clean.

A1.2a-S runtime commit `2f2fecb3` is independently review-clean. It changes only
`crates/shell/src/repl/async_repl.rs`: the ordinary internal greenfield host path now creates an
identity-free unpersisted proposal, obtains the exact optional world binding, applies or exact-joins
Start before transport or legacy persistence, materializes `PreparedAgentRuntime` only from the
applied authority, carries `RuntimeAuthorityContext::Bound` to the internal toolbox, performs zero
activated legacy session/participant/snapshot writes, and leaves startup ownership Pending. Exact
`Host + Some` and `Host + None` tests, the six host runtime lifecycle tests, and all 132
HostSessionAuthority tests pass; formatting, focused Clippy with warnings denied, and diff checks
pass. The post-review serial shell wall is `898 passed / 161 failed / 0 ignored` against the exact
starting `876 / 171 / 0`: 10 exact `FailToPass`, zero `PassToFail`, zero `NewFail`, and no removed or
renamed test. The 10 improvements are the six existing startup/shutdown proofs now exercising
Pending authority with zero activated legacy writes and four existing dispatch-validation proofs
that now reach their unchanged assertions after canonical Start application. The remaining 161 are
all inherited failure names, but they are not all the same normalized failure: a first-panic
signature audit classifies 137 as `FailToSameFailure` and 24 as `FailToChangedFailure`. Those 24
advance past the removed early Start preflight to separately gated seams: 10 reach the canonical
live-orchestrator-parent requirement, four reach the still-missing canonical orchestration-session
consumer, eight reach a later legacy-writer boundary, and two reach their unchanged downstream
assertions. No new failure name, assertion change, fixture weakening, or test substitution is
involved. These progressed failures remain explicit inputs to their later owning packets and are
not counted as A1.2a-S closure of the B1/B2.1 joint differential gate. Reviewer
`/root/a12a_s_runtime_review_1` completed read-only with verdict CLEAN. No seam is promoted;
B1/B2.1-R0 is independently review-clean through `bb3eefba`, and B3.2a plus B3.2a-WA are
independently review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`.

## A1.1e explicit-home policy proof requirement

A1.1e must route accepted-home policy input through the canonical broker resolver; shell-local
parsing, layering, validation, finalization, or explanation is a blocking authority error. Focused
differential proof must show that ambient and explicit APIs converge for identical defaults,
global-only, workspace-only, global-plus-workspace, replacement/merge, and explain/no-explain
inputs, including unchanged source-layer classification, provenance paths, derived legacy and V3
world-filesystem fields, network/backend/dispatch values, validation errors, and finalization.
Conflicting explicit home A and ambient home B plus CWD changes after acceptance must show that the
explicit result reads neither ambient home nor a newly selected home. Missing global policy,
malformed global/workspace policy, unsafe input, and finalization failure must fail consistently.

The same wall must prove that config retains its existing conditional policy-parsing behavior and
that absent descriptor entries still perform complete expected store/workspace/session/reference
and policy/snapshot identity validation. No global environment mutation may stand in for explicit
resolution. Passing these focused gates does not close `RG-AUTH-03`, `RG-HOME-01`, `RG-POLICY-03`,
or `RG-BASE-01`; dispatch narrowing, immutable active-work policy, A1.1d integrated Linux closeout,
and A1.1d cross-platform proof remain open. At the time of this A1.1e proof, A1.2 was unstarted;
that sentence is historical evidence, not current sequencing authorization.

### A1.1e recorded result

The docs-only allowlist correction is `69db6b29`. P1 is `8826270a` plus test remediation
`d2aac84f`; it adds the exact `HostSessionAuthority` resolution facade, exact root/authority
revision and commitment observations, full-store exact-current validation, and stale-observation
rejection without adding production `ExpectedAbsent` or transition issuance. P2 is `49876412` plus
remediations `80b0f9e0`, `839f78b4`, `182dd4af`, `1e356654`, and `cd676614`. The broker API
`resolve_effective_policy_with_explain_from_global_source` accepts explicit broker-appropriate
source path/bytes and enters the same private canonical core as the ambient API; shell code does not
parse, layer, validate, finalize, or explain a parallel policy. The accepted bootstrap home also
owns descriptor-backed config/policy/inventory reads and a distinct bound StateStore capability;
inventory provenance uses the accepted physical path and the bound StateStore capability exposes
no descendant-path API.

Recorded proof is broker differential `4/4`, broker `75/75`, config `20/20`, inventory `16/16`,
policy `13/13`, policy snapshot `9/9`, HostSessionAuthority `95/95`, StateStore `188/188`, and the
explicit-home group `19/19`. These cover defaults/global/workspace precedence and replacement,
explain provenance, malformed and finalization errors, conflicting ambient/explicit homes,
post-acceptance CWD input, conditional config policy parsing, absent and unsafe descriptors, exact
physical inventory provenance, stale observation, exact current-root identity, and bound
StateStore replacement rejection. Format, broker/shell check, Clippy with warnings denied, diff
check, and scoped GitNexus change detection passed; GitNexus classifies the complete authority and
policy range as critical breadth (`118` changed symbols, `67` affected processes), consistent with
the focused proof wall and without authorizing sibling work.

Exact focused command mapping:

- broker differential: `cargo test -p substrate-broker --lib a11e_explicit_global_source`;
- broker full: `cargo test -p substrate-broker --lib`;
- config/inventory/policy/snapshot: `cargo test -p shell --lib execution::config_model::tests`,
  `execution::agent_inventory::tests`, `execution::policy_model::tests`, and
  `execution::policy_snapshot::tests` respectively;
- authority/StateStore: `cargo test -p shell --lib execution::agent_runtime::host_session_authority`
  and `cargo test -p shell --lib execution::agent_runtime::state_store::tests`;
- explicit-home group: `cargo test -p shell --lib explicit_`;
- static wall: `cargo fmt --all -- --check`, `cargo check -p substrate-broker -p shell`,
  `cargo clippy -p substrate-broker -p shell --lib -- -D warnings`, and `git diff --check`.

The broad shell library wall remains non-green at `843 passed / 171 failed`; the sorted 171-test
failure set is exactly unchanged from the accepted `842/171` deferred legacy-writer/lifecycle
baseline at `49876412`, both from `cargo test -p shell --lib -- --nocapture`. Task evidence captured
the baseline log as `/tmp/substrate-a11e-shell-lib.log` with SHA-256
`ca3da7f69e4b53e8009900927c96abf20152127b2dab14e37b90caa3cb7416ad` and the final log as
`/tmp/substrate-a1-1e-shell-lib-combined-final.log` with SHA-256
`d9abaedc11fc8960a7fa53ac4c5d8cba74d842fe7fe8612b533e0ea56a47c6d7`; sorted failure-name lists
were compared with `diff -u` and produced no output. These task-local paths are evidence provenance,
not durable product artifacts. No unit/component result is promoted to integrated product proof. A1.1d integrated Linux
closeout remains open, native macOS closeout remains pending, `RG-AUTH-03`, `RG-HOME-01`,
`RG-POLICY-03`, and `RG-BASE-01` remain open, and no seam is promoted. Real CLI/helper/REPL
adoption, Start reservation, transition-intent issuance/claim/application, parked-successor repair,
dispatch narrowing/enforcement, auto-attach adoption, and all A1.2 work were deliberately excluded
from that A1.1e closeout. A1.2 work exposed the cycle and remains preserved out of the source
branch. This paragraph records the A1.1e closeout conclusion at that time; its old next-packet/order
statement is superseded by the
[Case B production-ingress audit](../b1-b2-1/evidence-regression.md#b1-production-path-sequencing-blocker-and-disposition).
Do not restore or modify the
broad A1.2 checkpoint, and do not begin A1.2b before the joint closeout → B3.1 → C1 corridor lands.
That historical next-packet statement is now superseded: bounded A1.2a, A1.2a-WB, and A1.2a-S are
landed and independently review-clean. B1/B2.1-R0 is also landed and independently review-clean
through `bb3eefba`; B3.2a plus B3.2a-WA are independently review-clean through
`d0a70727c2bec2b2d6fe0754ea469c4682684dda`. The recovered B1/B2.1 cores and B1/B2.1-0 are
review-clean through `6436289f`, `c519024b`, `de727091`, `717579b0`, and `83101dcb`, and their
later joint production integration closeout is recorded against the bound 2026-08-03 source
snapshot without additional product/test edits. B3.1 is dependency-ready, the held A1.3-P0 and
A1.3 records remain preserved as historical fences, A1.3-P1 is the active next public-adoption
packet, and A1 as a whole remains incomplete and non-landable.

## A1.1d-5I installer/bootstrap audit record

**Packet/status:** A1.1d-5I is a completed investigation beneath A1.1d-5 at baseline
`b29897e00194e7d3c27a43dc749fe29731d96f78`. Capital `I` means investigation. It is not
A1.1d-6, authorizes no implementation, closes no gate, promotes no seam, and changes no broader
sequence. The source branch, upstream, and remote all matched the baseline with a clean index and
worktree before evidence collection. The host was Manjaro Linux, kernel
`6.16.8-1-MANJARO`, effective UID `1000` (`spenser`), no sudo intended-user variables, umask
`0077`, and `getfacl`/`setfacl` `2.3.2`. Passwordless sudo was unavailable, so full provisioning
and Codex runtime installation were deliberately not run; the absent/inactive world service,
socket units, `/run/substrate.sock`, and `/run/substrate` were snapshotted and left unchanged.

**Focused context capsule:** `PrivateSubstrateHomeV1` in HostSessionAuthority owns final-root and
ancestor acceptance. The selected home then supplies StateStore, config, policy, and inventory;
those consumers cannot select a fallback. Dev/release installers bootstrap the root, write
installation projections, deploy shims, optionally provision platform services/world dependencies,
and optionally provision the Codex runtime. In host V1 the declared prefix, `SUBSTRATE_HOME`, and
`SUBSTRATE_ROOT` identify the same host root, and privileged paths must also carry the intended
platform principal. A Lima/WSL/native realization may use a distinct platform path/principal only
through an explicit mapping bound to that host-context commitment and platform instance.
The final root remains exact owner, directory, exact `0700`, rejects every observable access/default
ACL, and retains no-follow/stable physical identity; qualified `NoData` is accepted only under that
descriptor and mode authority and is not physical-absence proof. Ancestors must exclude
other-principal replacement
authority and uncertainty. Malicious root and malicious same-UID substitution before the first
accepted descriptor, shared multi-principal homes, separate install-root design, world/policy
changes, native platform proof, and remediation implementation are non-goals. `RG-HOME-01`,
`RG-INSTALL-01`, integrated Linux product proof, and native macOS proof remain open.

### Static stage and authority inventory

| Stage | Product path and boundary | Selected-context propagation and partial-failure truth | Coverage/evidence |
|---|---|---|---|
| Private-home bootstrap | Dev and release Unix installers invoke the built/extracted `substrate --version`; Linux provisioning repeats it for the invoking user. HostSessionAuthority is the acceptance owner. | Dev and release bootstrap pass `SUBSTRATE_HOME=<prefix>`; release/Linux also pass the resolved intended user where privileged. An invalid existing root is not repaired. A default-ACL-created candidate can nevertheless remain after post-create rejection. | Focused private-home tests cover wrong owner, mode, symlink, replacement, malformed ACL, concurrency, and effective-zero named ACL, but not non-writing effective access ACLs or inherited default ACL cleanup. |
| Shim deploy/remove | Dev install/uninstall, release install, and Windows install/uninstall paths launch or remove shim state. This child boundary is not home authority. | Unix dev deploy/remove and Windows dev deploy/remove set root but omit home; Unix release deploy passes neither explicitly. Windows release normally relies on profile sourcing, but `-NoAutoSource` still deploys without explicit context. Unix/Windows release uninstall has no selected-context shim-remove boundary before deletion. | `installer_env_wcu4` checks selected post-install doctor/sync strings and generated Windows profile content, not these child boundaries or lifecycle symmetry. |
| World dependency add/remove/current sync/rollback | Dev/release installer invokes the world-deps CLI; the runtime-family adapter consumes the selection. | Unix dev and release paths pass both root and home. Dev sync failure removes a global enable only when the installer added it; guest-side changes are explicitly not rolled back. | Static source confirmation only in 5I; no world mutation was authorized on this host. |
| World provisioning/enable/disable | Dev Linux calls `world-provision.sh`; release installs units; macOS maps the host selection to the Lima guest user's private home; Windows delegates to WSL. | Linux dev passes home, resolves intended user, validates the root, and embeds it in the unit. Unix release embeds the selected home but derives `ReadWritePaths` from ambient `$HOME`, so a custom prefix outside that allowlist can be denied by `ProtectSystem=strict`. Platform mappings cross distinct path/principal domains. | Full live provisioning requires a dedicated sudo-capable host and post-state restoration proof; static macOS/Windows comparison is not native proof. |
| Service install/restart | Linux provisioner installs world-service/gateway/ACL helper, units, socket drop-in, group/ACL bridge, and runtime/state directories, then reloads/restarts. | Dev uninstall can remove world-service/unit basics only when opted in, but omits installed gateway, ACL helper, and socket drop-in; release uninstall also omits the installed gateway. Failure/uninstall can therefore leave exact system-managed artifacts. | Static source and unchanged pre/post service snapshot only; R3 requires exact manifest and product parity. |
| Intended principal, group, and linger state | Linux provisioning resolves the invoking account across sudo, creates/uses the `substrate` group, may add membership/ACL bridge state, and reports linger requirements; release install-state can record selected host changes. | Dev uninstall gives manual group guidance; release automatic host-state cleanup is opt-in and acts from recorded metadata. Neither path may infer a principal from an unrelated ambient home or remove pre-existing membership/linger state. | Static inventory only; dedicated-host R2/R3 smoke must compare recorded intended principal and exact pre/post group, ACL-bridge, and linger state. |
| Codex runtime provisioning | Dev/release paths enable `codex-runtime` and run current sync only with world enabled. | Both root and home are passed. If a newly added global enable cannot sync, removal is attempted; any guest-side effects may remain. | Static source only; not native product proof and not evidence of runtime capability loss. |
| Generated projections/install state | Dev/release writers produce `env.sh`, manager environment, configuration, dev shim helper, versions, dependency scaffold, and install-state metadata. | `env.sh` and the dev shim helper encode selected values; manager scripts can self-derive install location; other artifacts may only be located beneath the prefix. Both manager variants prefer conflicting ambient `SUBSTRATE_HOME` over self-location. Shim doctor also projects the trace log from default home. | File-content/location/mode inspection plus world/shim/health doctor; R2 needs custom A versus ambient B consumer proof. |
| Uninstall/rerun/reinstall | Dev uninstaller removes selected artifacts and optionally services; Unix release wrapper locates by root but its child deletes from ambient home; Windows release accepts `-Prefix`. | Dev shim removal loses home without the override and leaves two gateway symlinks. Unix release can miss the selected tree while targeting default-home state. Windows release uses a prefix-independent forwarder PID path, recursively deletes every `$USERPROFILE/.substrate*` match, and has no selected shim-remove boundary. macOS dev host-socket cleanup uses `${HOME}/.substrate`, not the selected prefix. | Live dev custom-prefix matrix; Unix/macOS/Windows destructive paths are static evidence only and were not executed against real/native homes. |

### Isolated ACL matrix

All mutable cases used the recorded audit-owned root
`/run/user/1000/substrate-a1d5i-audit-019f6d60`; `/`, `/home`, and `/home/spenser` were read
only. The real home was owner `1000`, mode `0710`, with a named `libvirt-qemu:--x` access entry and
mask `--x`; its default-ACL read returned `NoData`, which is not proof of physical xattr absence.
The real `/home/spenser/.substrate` did not exist and was never
created, repaired, moved, or removed. Linux ACL semantics were checked against
[`acl(5)`](https://man7.org/linux/man-pages/man5/acl.5.html),
[`path_resolution(7)`](https://man7.org/linux/man-pages/man7/path_resolution.7.html),
[`mkdir(2)`](https://man7.org/linux/man-pages/man2/mkdir.2.html),
[`unlink(2)`](https://man7.org/linux/man-pages/man2/unlink.2.html), and
[`rename(2)`](https://man7.org/linux/man-pages/man2/rename.2.html): the ACL mask limits named
entries, search permits traversal, directory write plus search authorizes creation/removal/rename,
and a default ACL is inherited separately from the access ACL.

| Case | Expected security semantics | Actual at `b29897e0` |
|---|---|---|
| Ancestor named access ACL, effective `--x` | Traversal only; no create/delete/rename/replacement authority. Supportable only after explicit R1 contract/proof. | Rejected exit `5` as `foreign-acl`; final child not created. |
| Ancestor named access ACL, effective `r-x` | Read/search but no write/replacement authority; disclosure posture must be explicit, not conflated with write. | Rejected exit `5` as `foreign-acl`; final child not created. |
| Ancestor named access ACL, effective `rwx` | Write plus search is unsafe replacement authority and must fail closed. | Rejected before child creation (`wrong-mode` from ACL-correlated mode bits); security outcome correct. |
| Ancestor default ACL with effective `--x` | Separate inheritance surface; reject before candidate creation in V1. Later support requires a separately approved contract. | Rejected before child creation as `foreign-acl`. |
| Ancestor default named entry masked to `---` | Still inheritable; must not be accepted merely because current effective access is zero. | Ancestor passed; child inherited access/default ACLs; final validation failed `foreign-acl`; invalid candidate remained and identical rerun failed again. |
| Final root exact owner, `0700`, no observable access/default ACL (`NoData` under mode authority) | Accept and idempotently scaffold without claiming physical absence. | Accepted; `substrate --version` succeeded. |
| Final root with named access ACL | Reject every extended access entry, even masked ineffective. | Rejected `foreign-acl`. |
| Final root with default ACL | Reject every default ACL. | Rejected `foreign-acl`. |
| Wrong owner | Reject exact owner mismatch. | Focused intended-owner test passed; no privileged live mutation was attempted. |
| Wrong mode (`0750`) | Reject without repair. | Rejected `wrong-mode`. |
| Symlink candidate | Reject no-follow. | Rejected `symlink`. |
| Replacement/identity drift | Fail closed and do not write the replacement. | Focused byte-exact rename and facade-handoff replacement tests passed. |
| Malformed/unreadable ACL | Fail closed as malformed or validation unavailable. | Malformed parser test passed; source maps unreadable/unexpected xattr state to fail-closed validation. No kernel ACL corruption was manufactured. |

### Product installer matrix

`<audit>` below means the exact audit-owned root above. No wildcard deletion or real-home ACL
mutation was used. After evidence capture, that exact root was removed with a same-filesystem,
owner-checked traversal; it no longer exists. The real default home remained absent and the world
service/socket/runtime paths matched the initial absent/inactive snapshot.

| Exercise | Result |
|---|---|
| `cargo build -p substrate --bin substrate` | Passed. |
| Fresh default `dev-install-substrate.sh --profile debug --no-world` | Failed at private-home bootstrap on the real non-writing ancestor access ACL; real default root remained absent. |
| Custom `--prefix <audit>/product-prefix-only`, no outer home | Private bootstrap and generated projections used the prefix; shim deploy fell back to the default home and failed. Rerun failed at the same stage. |
| Same custom prefix with outer `SUBSTRATE_HOME=<prefix>` | Diagnostic workaround succeeded; repeat install succeeded and shim deploy reported up-to-date. This is not product-gate proof. |
| Shim status/deploy/remove with explicit selected context | Selected shim directory and 25 commands were reported; repeat deploy joined; removal succeeded. |
| Generated files | `env.sh` and the dev shim helper encoded the selected prefix; manager environment could self-locate; configuration/install state/version/dependency artifacts were inspected for prefix-relative placement and modes rather than falsely treated as encoded references. Root/shim directories were `0700`; the sensitive helper was `0600`. Conflicting ambient-home consumption remains unproven and defective by source. |
| Doctors after diagnostic install | `world doctor --json` exited `4` because world was intentionally disabled/unprovisioned; `shim doctor --json` exited `0` but projected `trace_log` from `/home/spenser/.substrate`; `health --json` exited `0` with world disabled. |
| Uninstall same prefix, no outer home | Shim removal fell back to default home and warned; manual selected-prefix cleanup returned success but left managed host and Linux `substrate-gateway` symlinks. |
| Uninstall with outer home | Shim removal used the selected prefix, but the same two gateway symlinks remained. |
| Uninstall followed by reinstall | Reinstall with the diagnostic override succeeded, proving remaining artifacts are currently tolerated rather than proving correct cleanup; final diagnostic uninstall reproduced the leftover symlinks. |
| Full world/Codex provisioning | Not run: passwordless sudo was unavailable and this was not established as a dedicated mutable service host. Static review proved the Unix release selected-home/`ReadWritePaths` mismatch and incomplete system-artifact cleanup; live product proof remains mandatory. |
| `cargo test -p shell --lib private_home_ -- --nocapture` and focused ACL/owner/replacement tests | Passed. |
| `cargo test -p shell --test installer_env_wcu4 -- --nocapture` | Four static tests passed; `config_current_show_is_not_affected_without_override_inputs` failed before its assertion because its private-home fixture is hard-coded below other-writable `/tmp`. This is a stale harness/missing-regression result, not positive installer proof. |

### Stable findings and bounded next proof

| Finding ID | Classification | Expected versus actual / security and product impact | Owner and smallest remediation | Required regression/product smoke | Platform / closeout block |
|---|---|---|---|---|---|
| **A1D5I-HOME-01** | `ContractGap` | Ancestor access and default ACLs are not separated by effective authority. A masked `--x`/`r-x` access entry lacks write/replacement authority, while a default ACL can alter a child. | HostSessionAuthority; R1 defines masked access semantics, rejects any effective write bit and every ancestor default ACL in V1, and preserves fail-closed uncertainty without weakening the final root. | Masked `---`/`--x`/`r-x`/write-only/combined-write matrix plus default ACL and malformed/unreadable cases; normal Linux install on the observed host. | Linux proven; static Unix applicability; both A1.1d Linux and B1/B2.1 Linux product smoke blocked. |
| **A1D5I-HOME-02** | `ImplementationBug` | `parent_acl_grants_named_principal` rejects every nonzero masked permission, so supportable non-writing traversal blocks default install before the approved Case A decision can be represented. | HostSessionAuthority; R1 implements strictly parsed masked non-writing Linux POSIX access-ACL support and rejects effective write or uncertainty. | Unit/property tests over ACL mask/effective rights and live bootstrap under each approved/rejected access-ACL case. | Linux proven; macOS requires native ACL mapping/proof; both blocked. |
| **A1D5I-HOME-03** | `DiagnosticBug` | An ancestor failure is attributed to the final requested home as generic `foreign-acl`, without path, role, ACL kind, or effective authority; it can also say “Existing roots” when no final root exists. | HostSessionAuthority error boundary; R1 structured diagnostic with sensitive-principal suppression. | Exact diagnostic assertions for ancestor/final/access/default/unavailable/new-candidate cases. | Unix-facing; both blocked because diagnostic/product proof is required. |
| **A1D5I-HOME-04** | `ImplementationBug` | A zero-effective default ACL passes ancestor validation, is inherited, fails final validation, and leaves a non-convergent current-attempt candidate. | HostSessionAuthority R1 rejects every ancestor default ACL before creation; R3 may remove only an exact descriptor-rejoined empty candidate created by the current attempt, never recursively or from pre-existing provenance. | Inherited-default pre-create rejection and synchronous current-attempt empty-candidate rollback/rerun tests; replacement/nonempty/ambiguous/`AlreadyExists` candidates and pre-existing invalid roots remain byte/metadata unchanged. | Linux proven; both blocked. |
| **A1D5I-HOME-05** | `ContractGap` | After an abrupt interruption, an unaccepted invalid candidate has no trustworthy current-attempt provenance on rerun. The no-repair rule forbids treating `AlreadyExists` as deletion authority, so arbitrary crash-window convergence cannot be promised. | R3 proves every reachable post-R1 crash residue is either already valid and exact-joinable or stays fail-closed. If invalid residue is still reachable and product convergence is required, stop for a separately approved provenance/publication mechanism; never infer provenance from the name. | Kill-point matrix around parent validation, creation, first open, validation, cleanup, and acceptance; valid residues join, exact synchronous failures clean safely, invalid unknown-provenance residues remain unchanged/fail-closed. | Unix contract; both gates remain blocked until R3 resolves or explicitly bounds every reachable state. |
| **A1D5I-INSTALL-01** | `ImplementationBug` | Selected custom prefix is lost at shim deploy/remove. Outer `SUBSTRATE_HOME` makes the same install work, proving propagation—not final-root validity—is the failure. Unix release deploy and Windows dev deploy/remove have the same static omission. | Product child adapter; R2 carries the host `InstallBootstrapContextV1` explicitly everywhere. | Dev/release custom-prefix install/uninstall without outer overrides; shim status/deploy/remove and encoded/self-derived/prefix-relative generated-artifact assertions; native platform runs where supported. | Linux live; Unix/Windows static; both blocked. |
| **A1D5I-INSTALL-02** | `ImplementationBug` | Unix release uninstall locates by `SUBSTRATE_ROOT` but `uninstall-substrate.sh` deletes `.substrate` relative to ambient `HOME`; a custom-prefix uninstall can miss the selected tree and target unrelated default-home state. | Release uninstall boundary; R2 selects exact matching context and R3 limits deletion to recorded managed state. | Hermetic two-home negative test proving selected-only removal and zero ambient-home mutation; release install→uninstall→reinstall product smoke. | Unix static high-impact path; both blocked. It was not executed against real home. |
| **A1D5I-INSTALL-03** | `ImplementationBug` | Dev uninstall omits managed host and Linux `substrate-gateway` symlinks while removing sibling managed links. | Dev lifecycle cleanup; R3 exact managed-artifact inventory and convergence. | Install→uninstall exact-artifact diff, repeat uninstall, reinstall, and unrelated-symlink preservation. | Linux live; both blocked. |
| **A1D5I-INSTALL-04** | `DiagnosticBug` | With selected custom home, shim doctor reports the trace log under default `$HOME/.substrate`; focused bootstrap success can therefore misstate diagnostic authority. | Shim diagnostic projection; R2 consumes selected context consistently. | Custom-home shim/trace doctor assertions and emitted trace-location product check. | Linux live; cross-platform static review required; both blocked. |
| **A1D5I-INSTALL-05** | `ImplementationBug` | Unix release unit sets `SUBSTRATE_HOME` to the selected prefix but derives `ReadWritePaths` from ambient `$HOME`; `ProtectSystem=strict` can deny a valid custom home outside that allowlist. | Release Linux service projection; R2 derives the sandbox allowlist from the selected host context. | Full-world release smoke with a custom prefix outside ambient home, service write/bootstrap proof, and exact unit assertions. | Linux static; both blocked. |
| **A1D5I-INSTALL-06** | `ImplementationBug` | Linux provisioning installs the gateway, ACL helper, and socket drop-in, but dev removal omits them; release removal also omits the installed gateway. Prefix-local success cannot stand in for system lifecycle convergence. | Linux managed-system lifecycle; R3 records exact pre-state/managed manifest and restores binaries, helpers, units/drop-ins, sockets, runtime/state paths, and installer-created account-state changes without broad deletion or removing pre-existing state. | Dedicated-host install/uninstall pre/post diff, repeat uninstall, accepted-home partial provisioning/rerun, exact group/ACL-bridge/linger accounting, and preservation of pre-existing system artifacts. | Linux static; both blocked. |
| **A1D5I-INSTALL-07** | `ImplementationBug` | Windows release uninstall recursively deletes every `$USERPROFILE/.substrate*` match independent of `-Prefix`, so unrelated backups/state can be destroyed. | Windows uninstall boundary; R2 selects exact context and R3 forbids wildcard cleanup in favor of exact managed identity. | Hermetic native Windows negative test with selected prefix plus unrelated `.substrate*` siblings; only selected recorded artifacts may change. | Windows static only; native Windows product proof pending; neither named Linux closeout gate is blocked by this row alone. |
| **A1D5I-INSTALL-08** | `ImplementationBug` | Windows release `-NoAutoSource` deploys shims without explicit selected context; macOS dev cleanup targets `${HOME}/.substrate/sock/agent.sock` instead of the selected host prefix. | Platform lifecycle adapters; R2 carries host context and exact platform mapping through deploy and transports the selected macOS socket target; R3 alone performs socket cleanup and preservation/convergence. | Native Windows `-NoAutoSource` custom-prefix shim proof, R2 native macOS selected-socket mapping proof, and R3 native macOS cleanup/preservation proof. | Static Windows/macOS only; native macOS/product proof remains open; neither named Linux closeout gate is blocked by this row alone. |
| **A1D5I-INSTALL-09** | `ImplementationBug` | Generated release/dev manager scripts prefer ambient `SUBSTRATE_HOME=B` over their installed self-location A, so a custom-prefix projection can consume a different authority home. | Generated projection consumer; R2 binds encoded/self-derived location to the selected host context and rejects conflicting ambient selection. | Install at A, source under ambient B, and prove only A is consumed; include missing/malformed A and B negative cases. | Unix static; both blocked. |
| **A1D5I-INSTALL-10** | `ContractGap` | Windows release uses a prefix-independent forwarder PID path and removes the prefix tree without a selected-context shim-remove boundary. The current pack does not define which artifacts are per-prefix versus intentionally per-user shared. | R2 classifies and transports exact prefix-scoped versus SID+platform-instance+pipe-scoped context without deletion authority; R3 alone defines any managed-artifact ownership manifest used for removal and removes only exact matching provenance. | Two-prefix native Windows lifecycle matrix covering shared/per-prefix forwarder and shim state, conflict, partial install, uninstall order, and unrelated-state preservation. | Windows static only; native Windows product proof pending; neither named Linux closeout gate is blocked by this row alone. |
| **A1D5I-REG-01** | `MissingRegression` | Existing installer tests check selected strings but not real child propagation/lifecycle; one fixture uses `/tmp`, which is invalid under the current ancestor contract and masks its intended assertion. | R1/R2/R3 regression suites. | Secure audit-owned fixture roots; default/custom, accepted-home partial/rerun, synchronous rollback, crash-window fail-closed, uninstall/reinstall, and negative two-home matrices. | Linux proven; both blocked. |
| **A1D5I-ENV-01** | `EnvironmentUnsupported` | This audit host could not safely run privileged service/Codex provisioning without a sudo prompt/dedicated-host restoration contract. This is an evidence limitation, not a product defect or capability regression. | Later R2/R3 product-smoke environment. | Snapshot, provision, doctor/smoke, runtime sync, uninstall/restore, and post-snapshot parity on a dedicated supported Linux host. | Linux proof still required; gate stays open, but classification itself blocks neither architecture core. |
| **A1D5I-FP-01** | `FalsePositive` | Install/bootstrap unavailability does not prove loss of world filesystem/network capability, changed policy/enforcement, or regression of review-clean B1 receipt/B2.1 supervisor semantics. | No remediation outside R1/R2/R3. | Preserve the differential world/policy wall and later product smoke; do not reopen B1/B2.1-0. | All platforms; neither semantic core is regressed. |

### A1.1d-5R2-0 propagation planning record

**Decision/status:** the docs/evidence-only planning packet starts at
`6ab2a515e13946324d0aac25b144e1c3408cb2c1` and freezes 118 live propagation edges as
PI-001–PI-118 in `02-seam-crosswalk.md`. It changes only the existing six control-pack files,
implements no runtime behavior, performs no privileged/platform mutation, promotes no seam, and
claims no native macOS or Windows proof. At that planning point the corrected R2 implementation
order was R2-1 -> R2-2 Routes A-D -> R2-2E -> R2-2F -> R2-2 integration closeout -> R2-3 -> R2-4;
F0's later evidence-gate insertion supersedes that outgoing sequence without changing inventory
ownership. R3 follows and remains unimplemented.

Every A1.1d-5I finding has an explicit terminal owner:

| A1.1d-5I finding | Frozen owner/packet | Closure rule |
|---|---|---|
| HOME-01, HOME-02, HOME-03 | R1, already review-clean | Preserve effective-authority and diagnostic contract; R2/R3 do not reopen it. |
| HOME-04 | R1 pre-create rejection; R3 synchronous exact-candidate cleanup | R2 may carry context only; it cannot remove the candidate. |
| HOME-05 | R3 crash-window reachability/convergence decision | Unknown-provenance invalid residues remain unchanged and fail closed absent a separately approved boundary. |
| INSTALL-01 | R2-1 Unix dev/shim; R2-2 Unix release; R2-3 Windows/platform | Complete context reaches deploy/remove/status/doctor without outer override. |
| INSTALL-02 | R2-2 exact release-uninstall selection; R3 deletion/convergence | Selection and deletion authority stay separate. |
| INSTALL-03 | R3 | Exact managed gateway/symlink cleanup only. |
| INSTALL-04 | R2-1, joined in R2-4 | Shim/trace diagnostic reports selected A. |
| INSTALL-05 | R2-2, proven in R2-4 | Linux `ReadWritePaths` is derived from selected context. |
| INSTALL-06 | R3 | System helper/gateway/unit/drop-in/socket/account-state cleanup. |
| INSTALL-07 | R2-3 exact Windows selection; R3 deletion | Wildcard removal is never R2 authority. |
| INSTALL-08 | R2-3 context/mapping/target transport; R3 cleanup action | Windows `-NoAutoSource` and macOS selected-prefix socket mapping are R2; socket removal/preservation/convergence is R3; native evidence remains assigned. |
| INSTALL-09 | R2-1/R2-2, joined in R2-4 | Generated manager binds A and cannot accept ambient B as authority. |
| INSTALL-10 | R2-3 scope classification/context transport; R3 deletion manifest/action | R2 records prefix versus SID+instance+pipe scope without inventing a removal manifest. |
| REG-01 | R1 preserved; R2-1/R2-2/R2-3 tests; R2-4 join; R3 lifecycle suite | Secure fixture roots and the named two-home/lifecycle matrices are mandatory. |
| ENV-01 | R2-4 Linux product host; native platform assignments remain separate | Evidence limitation is not positive product proof. |
| FP-01 | Out of scope; preserve review-clean behavior | No world capability, policy, B1 receipt, or B2.1 supervisor reopening. |

The R2 regression gates are exact labels used by the inventory and packet allowlists:

| Gate | Required proof before the owning packet can exit |
|---|---|
| **R2-UDEV-01** | Unix dev install, uninstall, `dev-shim-bootstrap.sh`, and successful install-sensitive standalone CLI modes each construct exactly one equal A/H/R context from the declared/installed-witness/account-default source before explicit-context home bootstrap; parse failures, help, `--version`, and `--version-json` mutate no scaffold; installers and focused R1 private-home tests bootstrap only through hidden `--install-bootstrap-home-v1` plus the authenticated argv carrier, current account+UID equality, and checked H/R, after which the process returns without normal dispatch; environment alone cannot select the action; custom A/bin dev symlink self-derives A without an outer override; direct repo and zero/multiple witnesses fail; conflicting ambient B/dev-prefix cannot retarget bootstrap, children, preexec, or uninstall selection; malformed/tampered/forged-principal carriers fail before mutation. |
| **R2-UREL-01** | Unix release wrapper, direct installer/uninstaller, and installed child distinguish constructor/child modes and preserve the same context across install/uninstall; Unix release A/bin witness self-derives A; every child binds the committed principal to current Unix identity/sudo origin and cannot reinterpret prefix; repeat install preserves commitment; no removal/convergence claim. |
| **R2-SHIM-01** | Installer-managed, automatic CLI, standalone declared/self-derived deploy/remove/status/doctor/repair, and physical shim paths receive one complete context or verified mapping projection; a bare physical shim recovers exactly one no-follow invocation witness without PATH precedence, resolves the canonical current Unix account+UID or Windows account+SID through its exact OS observation surface, and rejects zero/multiple witnesses or a forged principal before dispatch; repair target derives from the committed principal; telemetry and manager-hint manifest/overlay consume custom A under ambient B, with no compiled-repo or manifest-environment fallback in normal product mode. R2-1 proves only shell `ExplicitProduct`: trace output `A/trace.jsonl`, policy Git A, repeated A reuse, conflicting-path rejection, missing-metadata no-fallback, and zero B access while the setter/default physical-shim/replay/platform behavior remains unchanged and unpromoted. R2-3 migrates physical shim and already-frozen replay/platform callers, removes/makes unreachable `LegacyAmbientCompatibility`, and proves the final global unbound-init failure rule. Legacy H/R/carriers remain consistency checks. Replacement/migration/recursive removal actions and trace rotation/retention semantics remain byte-frozen in R2. |
| **R2-GEN-01** | Dev/release `env.sh`, manager, Bash preexec, helper, generated `A/manager_hooks.yaml`, configuration, version, install-state, dependency, service, intended-principal PATH, and Lima known-hosts projections are prefix-relative or self-derived, encode/project the same context where consumed, and never treat a conflicting ambient B, repository path, or generated value as selection authority. |
| **R2-RUNTIME-01** | Before world-enable dispatch, nested `--home` or global `--install-prefix` selects A, equal normalized selectors join one context, conflicting selectors fail before context construction or mutation, absence of both selectors permits only a verified installed-product witness, and every declared selector on an authenticated internal carrier must match it; ambient B and runner-local state never select A. Typed IH crosses `run_shell_with_cli` → the existing Unix `ShellConfig::from_cli` world branch → `handle_world_command` → `run_enable` by explicit argument at every hop; H/R/carrier environment values are checked projections only, and the runner performs no reconstruction. At the child boundary, `run_enable` or its existing provision-deps path canonically encodes that IH and `run_helper_script` transports the exact authenticated carrier on hidden argv without interpreting or logging it; `world-enable.sh` discriminates child mode by argv presence, validates the carrier and current principal/exact sudo origin, requires normalized `--home` equality, installs checked projections, and only then dispatches. Missing, malformed, duplicate, tampered, reordered, forged-principal, or conflicting input fails before action and never falls into public mode; environment-only state cannot recover authority. `run_enable`, including its existing provision-deps branch, derives A from typed IH and passes explicit A to `update_manager_env_exports`; conflicting ambient B cannot retarget the generated manager export. Routes A–D preserve their reviewed behavior but do not prove non-enable Gateway/Deps or `run_sync_after_provisioning`: R2-2E owns authenticated gateway projection and R2-2F owns normal world-deps/provision/post-sync propagation. E has an authenticated Linux Gateway route through fixed `/run/substrate.sock`; because source closure found no authenticated macOS platform-endpoint source in E, macOS must fail before client construction or `auto_select`, and Windows/other contextless Gateway cfgs must fail before config, policy, inventory, disabled-response, or client selection. Their ambient clients remain frozen R2-3 compatibility. F's authenticated world-deps and doctor guarantee is likewise Unix/Linux only; macOS, Windows, fallback, and other non-Unix adapters remain explicit unproven R2-3 compatibility or unavailable/fail-closed paths. Host/Health/Config/Policy same-process branches receive exact typed IH; Unix Health reaches the existing `report::collect_report_for_context` only through a crate-private `shim_doctor::collect_report_for_context` name exposure, then carries typed IH through the snapshot path. The context-aware export changes visibility only. The existing `collect_report` compatibility re-export and collector may receive only item-level Unix `unused_imports`/`dead_code` annotations whose reasons name temporary R2-3 compatibility ownership; they have no runtime or non-Unix effect, and no broader lint allowance is permitted. Named checked-projection `collect_report` remains behavior-frozen, cannot be selected by typed Health, and is barred from product proof pending R2-3 migration/removal. Route A production proof remains patch-bound to immutable SHA-256 `ea4abf43043013994e698d54f21c04bba58833b3bb3247bf6d5d3b454f51b943` and its exact eleven-file fingerprints/source-level diff. Its sole successor may add item-level Unix cfg only to the two exact tests and their sole test-only `HostSessionAuthority` imports, plus focused Health conflicting-A/B and Host installed-witness regressions specified in `03`/`04`; completion requires a new exact candidate hash/manifest/fingerprints, deterministic base-to-successor hunk map, unchanged production bytes, complete fresh-label attribution mapping, and CLEAN review. Historical GitNexus observations are 33/18, reverse 11/18, 37/18, 39/18, and current base CRITICAL 24/31; counts and generated labels remain required evidence but are not raw completion gates. Stale-index pinning is forbidden, and every authorized test edit receives normal pre-edit impact. Native non-Linux proof is unavailable here when MSVC `lib.exe` or the Apple SDK `TargetConditionals.h` is missing; that is neither success nor a product regression. Completion requires E to bind gateway config/effective-policy/network/runtime-family/Codex projections to A and F to bind every current/global/workspace/runtime/provision/post-sync path plus doctor constituent identity to the same A. PI-105 derives process-scoped `PlatformPrincipalV1` only from validated IH, carries it through normal REPL/hidden owner-helper and prepared member dispatch without durable authority changes, requires Unix account+UID database round-trip, and targets only that account's `.codex`; ambient/root home and A cannot retarget it. World-service host diagnostic fields remain absent while the host shell attaches optional selected-prefix/commitment fields from typed IH. Mixed-authority diagnostics cannot report success. Rollback and synthetic-auth deletion remain R3. Runtime, policy, gateway, provider, orchestration, receipt, supervisor, retained-worker, and world capability semantics do not change. |
| **R2-LINUX-01** | Unix account+UID and context survive every release, dev-install, and provision sudo boundary: context-aware Substrate helpers validate the full argv carrier; arbitrary tools receive only exact context-derived argv after parent revalidation and no preserved environment; the ACL bridge receives no context and rejects every tuple except its exact three fixed mode/target/`substrate` combinations while preserving group-member enumeration. Linux unit environment carries H=A, R=A, commitment, and intended-principal projection; socket/drop-in and `ReadWritePaths` derive from A; only the fixed same-attempt Linux socket-restart unlink is exercised; focused/static proof passes in R2-2 and dedicated-host service/world/Codex product proof passes in R2-4. |
| **R2-DIAG-01** | Trace, shim, repair, world, host, health, world-deps, config/policy proof, gateway, Lima/WSL, and pipe diagnostics report selected host commitment, verified platform mapping/transport, and host platform-control root where applicable; direct mode constructs from declared/principal/OS Known Folder inputs, internal mode validates hidden argv and matching scrubbed projections, and no default-home/guest/pipe/`LIMA_HOME`/`LOCALAPPDATA` reconstruction is represented as authority. `WorldDoctorReportV1` host fields are optional/defaulted and omitted by the world-service producer; only the host shell enriches them from typed IH, preserving old wire JSON. Health/shim snapshots receive typed IH/A-derived paths. The world-deps fixture remains A-rooted and F5-owned. Under F5-PD, authenticated Linux production never selects the World Doctor fixture and instead runs the hidden passive child with the same carrier plus checked A projections; that fixture is Linux test evidence only. Existing non-Linux fixture/public-child compatibility remains behavior-frozen, unproven, and unable to satisfy F5-PD/F5 authority or native proof. R2-2F additionally requires exact non-secret prefix/commitment equality for every Linux world/config/policy/inventory/dependency/runtime constituent: missing `ok`, missing identity, or mixed A/B becomes unavailable/incoherent with `ok=false`, never healthy A-bound truth. Host/World doctor consumes A-derived world-fs policy rather than ambient broker state. The named checked-projection shim compatibility caller remains behavior-frozen and cannot satisfy R2-2 proof. Migrated shell live trace/policy production uses only entry-bound A; `LegacyAmbientCompatibility` remains named, temporary, behavior-equivalent, and non-promotable until R2-3 migration/removal. `SHIM_TRACE_LOG` cannot satisfy R2-1 product proof. |
| **R2-MAP-MAC-01** | IH plus the VM selector and host account-database-derived `/.lima` control root performs only Stage-1 declared-instance realization; parents scrub/overwrite child `HOME`/`LIMA_HOME`, direct internal mismatch rejects, and PM is finalized from running-guest machine ID/account+UID+home before R2 guest projection. R2 fixes the future V1 SSH-UDS target from `A/sock/agent.sock` to `/run/substrate.sock`; backend auto-selection, `vsock-proxy`, TCP, ambient endpoints, and ambient Lima store cannot replace it. The validated R2 path stops before forwarder launch with an explicit R3 prerequisite, so socket unlink, `StreamLocalBindUnlink`, timeout kill/wait, handle-drop teardown, retry, and convergence remain exclusively R3 and are not exercised as R2 proof. Host commitment also binds the guest unit and future A-scoped known-hosts projection; shell/shim/replay factory callers are explicit; no path/principal equality. Static proof is labeled static and a native pre-existing-Lima no-forwarder A/B mapping-only run is assigned separately; macOS product transport remains pending R3. |
| **R2-MAP-WIN-01** | Windows account+SID host context survives dev/release/uninstall and `-NoAutoSource`; the release A/bin copy self-derives A under ambient B and every internal carrier is bound to the current account+SID, with forged-principal rejection. It binds one WSL instance, WSL-native account+UID+home, one normalized public-selected/default pipe, and an OS Known Folder control root. A canonical SID+registered-distro+machine-ID+pipe digest scopes the shared PID root; config/logs are under A and all paths are explicit, so `LOCALAPPDATA`/`USERPROFILE` cannot select them. Warm/forwarder/backend/status/doctor plus shell/shim/replay consume the same mapping. The forwarder validates PM/current token before `wsl -d`; its WSL child receives only PM-derived distro, normal-product guest target, and commitment despite conflicting ambient config/target/`WSLENV`. The fail-closed WSL guards remain byte-identical; creation, replacement, timeout kill, stop, PID deletion, and convergence remain R3; static proof is labeled static and native existing-instance A/B mapping-only proof is assigned without claiming provisioning. |
| **R3-LIFE-01** | R3-only candidate, rollback, manifest, managed-system cleanup, account-state restoration, crash-window, uninstall/reinstall, shim/payload/bin/cache/helper/unit/socket/platform-staging/forwarder unlink/drop/timeout/synthetic-auth cleanup, and unrelated-state preservation matrix. Referencing this gate from R2 transports a target only. |
| **R3-WIN-01** | R3-only native Windows two-prefix cleanup matrix: no wildcard removal; exact per-prefix versus shared state; version/bin/profile replacement, timeout rollback, uninstall order, WSL/forwarder/shim cleanup, partial install, and unrelated-state preservation. |

For Route B, the immutable WIP is exact patch SHA-256
`f8f845ef6aa6688fdf60be6ed983bb119971d65895a0f38c25fc1dad7643a77f` across the six PI-105
production files at preservation commit `a6e29a10ea3dc9cb673a22912002efb83658e7b2`. Its refreshed
GitNexus observation is CRITICAL, 17 attributed symbols, and 25 existing process labels; every
label maps in `03-phase-slice-map.md` to the shell-root principal extraction, hidden owner-helper
transport, or prepared member-request/seed-resolution root, with downstream public Agent,
policy/filesystem/network, validation, carrier, and path labels classified as unchanged
attribution. The final candidate may add only one item-level `dead_code` annotation to the
principal-less compatibility entrypoint, with a reason stating intentional non-use and temporary
R2-3 ownership. That entrypoint remains uncalled by typed product routes and fail-closed before
allowlisted Codex credential projection. Completion requires exact six-file/hunk containment,
fresh semantic review, and no new module, execution family, authority source, schema, secret
surface, capability, lifecycle, Route C, Route D, or R2-3 behavior.

Route B's exact post-lint, pre-format candidate is preserved at ordinary/binary SHA-256
`e9da85eb206be522645120dfee459ee79ce48793bc61161487d4b5c4d2fda243`, commit
`07f3117aa0d2f3d57279d20fec204757bba8383a`, with the same six-file manifest. Its only permitted
successor is repository-default `cargo fmt --all` output in `agents_cmd.rs`,
`orchestrator_world_dispatch.rs`, `world_ops.rs`, and `async_repl.rs`; `invocation/plan.rs` and
`routing.rs` remain byte-identical. A new candidate hash and fingerprints are recorded after
proving every successor-only hunk is whitespace/layout, the whitespace-ignored source diff is
empty, format check passes, and every refreshed GitNexus label remains an existing Route B root or
formatting attribution. Raw GitNexus count drift does not authorize semantics. A seventh file,
non-formatting hunk, changed unaffected-file fingerprint, new module/family/path, or remediation
beyond canonical formatting stops before runtime review or commit. Fresh runtime review remains
mandatory; this exception imports no Route C, Route D, R2-3, capability, cleanup, or lifecycle work.

The formatted Route B candidate is preserved at ordinary/binary SHA-256
`ea9cf3e582650007083812ad70e0bf3198405e73d0cde0fb8ecfbedeb49884a2`, commit
`57e13d291abf1239aacee0020ac444ec05e11d56`, with the same six files. Security review found three
valid R2-2 defects: an ambient reserved seed value could survive injector early returns;
principal-aware direct/prepared Spawn dropped the typed principal before member-request
construction; and `cfg(any(target_os = "linux", test))` exposed Unix account calls in Windows test
builds. The only authorized successor clears the reserved key before every decision, explicitly
passes `Some(exact principal)` through both live principal-aware Spawn routes while compatibility
passes `None`, and narrows the account resolver/helper plus mechanically required test/import cfgs
to Linux production or Unix tests. Poisoned-input, principal-aware direct/prepared Spawn,
compatibility fail-closed, and static cfg regressions are mandatory. The six-file manifest is fixed;
no durable authority/schema change, ambient recovery, policy/capability change, Route C/D, R2-3,
cleanup, or lifecycle work is imported. GitNexus raw counts remain evidence, while the exact patch,
fingerprints, hunk map, complete manual under-resolved Spawn closure, and CLEAN reviews are the
semantic containment boundary.

### Historical Route C authorization checkpoint

Route B is preserved exactly and remains review-clean at
`6cee990f0370013c8b05a5495301db7aea642cd5`, ordinary/binary patch SHA-256
`28e6b9da35f96b5f93c49369cbde0eda77e9a145b54f9c412bae8e5a83871674`, with its reviewed
six-file semantics unchanged. At that checkpoint, the seven unpushed runtime commits from published
baseline `dab71d816f8e3a51d841c2293b646463e2d28cfc` were replay inputs only: the docs correction had
to land on that published baseline first, and each runtime commit had to replay with identical
ordinary/binary patch and file scope before Route C proof resumed. That historical replay completed
and is not renewed-closeout authority.

The preserved Route C candidate is commit `41b82327e23798719ed9a0b4cae1f557fb593670`, parented by
the exact Route B tree, with canonical full-index binary SHA-256
`5b436d5dfeaf6e513cbaa306de65848cbe3d5b003fa8c00589be09e54b630277`, 182 insertions/five
deletions, and the exact five-file manifest/fingerprints in `04-contracts-and-gates.md`. It is
authorized only as an authenticated, non-authoritative Host/World doctor projection. The four
existing diagnostic roots are `handle_host_command`, `host_doctor_main`, `handle_world_command`,
and `world_doctor_main`; no world-service, representation, helper, or test symbol becomes another
execution root or authority owner. The preservation-time GitNexus result was CRITICAL 25 symbols/32
labels/five files; a byte-identical refreshed index reported CRITICAL 19/32/five. Review must decide
semantic containment from the patch, fingerprints, exact manifest, source mapping, owner/family
set, and authority behavior—not a brittle numerical ceiling—and must explicitly confirm any
attribution-only drift.

Route C cannot establish context, recover it from ambient or generated projections, expose hidden
carrier/credential/request/secret bytes or sensitive principal material, or mutate installation,
service, world, policy, capability, filesystem/network enforcement, placement, caging, credential,
receipt, supervisor, retained-worker, cleanup, lifecycle, or execution semantics. Route D and R2-3
remain unstarted and no seam is promoted. A sixth file, actual new semantic path, new module owner
or execution family, or broader authority owner is respectively `ImpactDecisionRequired` or
`CrossDocumentChangeRequired`. Review remediation may stay within the exact five-file semantic
envelope only after recording a new patch/fingerprint set and rerunning impact, proof,
differential, and fresh reviews.

The Route C regression ledger requires exact reproduction of the inherited disabled-world doctor
failure on the clean replayed Route B baseline; transport API and world-service proof; installed-
witness Host A and World typed-A JSON under ambient B; applicable malformed/missing-context
fail-closed proof; warnings-denied touched-crate Clippy; shell/workspace all-target compilation;
format/diff checks; and a broad shell differential with zero `PassToFail`, `NewFail`, removed,
renamed, substituted, weakened, or newly ignored tests. Retained failures keep normalized
signatures, and each `FailToPass` receives a no-bypass causal audit. Three fresh read-only review
perspectives—authority/security, call-path/impact, and cross-platform/regression—must be CLEAN
before one local Route C commit and its remote preservation ref. Source runtime commits remain
unpublished until complete R2-2 closeout authorizes publication.

### Historical Route D docs-first source-closure correction

Route D started from replayed Route C commit `fe288d233b5a198e19c2afba857d02e982ef6e1b`, tree
`1703a0e0f57c8c994a3fd59f276c0f1a28dfbb50`. Routes A, B, and C were frozen. The eight then-unpushed
runtime commits above published docs commit `f2131f13424906a167f58795f8eb6175cc384f14` were replay
inputs only after that correction published first; their per-commit and aggregate ordinary/full-
index binary patch identities and file manifests remained exact. That Route D replay completed and
is packet-scoped historical evidence, not renewed-closeout authority.

The read-only Route D audit found two ambient pre-carrier branches in the existing
`crates/shell/src/builtins/shim_doctor/report.rs` family. `try_load_health_fixture` and
`health_fixture_path` could select `B/health/world_deps.json` or
`B/health/world_doctor.json` before typed collection, while `gather_world_doctor_snapshot` and
`run_json_subcommand` could launch the current repository binary as `world doctor --json` without
the authenticated witness. A malicious or malformed B fixture could therefore replace A's report
or disclose B's absolute path, and the contextless child could fail or execute against B.

At that checkpoint the correction authorized only the Route D carrier mechanics inside that
already-allowed report file: `build_report` passed the typed carrier to both gather branches, both
fixture lookups derived only from A, and the existing Unix world-doctor child received the same
canonical hidden argv carrier plus context-derived checked child projections. F5-PD later
supersedes the authenticated Linux World Doctor portion: its production fixture is removed and its
child becomes passive. Non-Linux compatibility remains unchanged and cannot satisfy proof. The
existing Unix `collect_report` compatibility
body, visibility, cfg, re-export, caller, output, and behavior remain frozen until R2-3.

GitNexus reports LOW upstream impact for `gather_world_doctor_snapshot` (one direct/five impacted),
`try_load_health_fixture` (two/four), `health_fixture_path` (one/four), and
`run_json_subcommand` (one/three). Each maps to the existing `health::run`/shim-doctor family; raw
counts are observed evidence, not a brittle ceiling. Completion requires the exact approved
source/test manifest, source-level caller/cfg mapping, no new owner/module/execution family, final
change detection, and fresh semantic containment review.

Historical Route D regressions create conflicting B world/deps fixtures under typed A, prove neither
fixture can win or leak, and prove the nested world snapshot receives A's authenticated witness.
F5-PD replaces the Linux World Doctor fixture/success expectation with test-only fixture and
truthful passive unavailable proof. The combined gates scan JSON, human output, errors, logs,
traces, snapshots, and fixtures for carrier, credential,
request, non-public commitment, and sensitive-principal bytes. Missing/malformed/tampered/contextless
input fails closed; success and failure preserve complete A/B trees and the parent environment;
environment-only input cannot invoke the typed route; compatibility collection remains unchanged;
non-Unix cfg compilation remains unchanged. Focused diagnostic suites, warnings-denied Clippy,
shell/workspace all-target checks, format/diff checks, final GitNexus mapping, and the broad shell
differential remain mandatory. Any new resolver/schema/authority owner, ambient fallback, physical
shim/replay/platform migration, world/policy/capability change, or installation, cleanup, service,
credential, receipt, supervisor, retained-worker, or lifecycle change is
`CrossDocumentChangeRequired`.

The optional named checkpoint and cross-document verification skills are unavailable for this
increment. Their explicit substitutes are immutable commit/tree/patch/file checkpoints at every
phase and a six-file structure, relative-link, table, fence, gate, sequence, and stale-status
cross-check. The security skill's missing supplemental checklist is replaced by the complete
embedded security checklist. These substitutions do not weaken any gate.

### R2-2 historical failed integration closeout and remaining-seam correction

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#r2-2-historical-failed-integration-closeout-and-remaining-seam-correction`](../a1.1d-5r2-2f/evidence-regression.md#r2-2-historical-failed-integration-closeout-and-remaining-seam-correction).

### A1.1d-5R2-2E authenticated gateway closeout

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2e-authenticated-gateway-closeout`](../a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2e-authenticated-gateway-closeout).

### A1.1d-5R2-2F0 deterministic world-socket test isolation authorization

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0-deterministic-world-socket-test-isolation-authorization`](../a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0-deterministic-world-socket-test-isolation-authorization).

### A1.1d-5R2-2F0a — SUBSTRATE_HOME test isolation

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0a--substrate_home-test-isolation`](../a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0a--substrate_home-test-isolation).

### A1.1d-5R2-2F0b — deterministic renderer-output test isolation

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0b--deterministic-renderer-output-test-isolation`](../a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0b--deterministic-renderer-output-test-isolation).
### A1.1d-5R1 Linux effective-authority contract correction

**Decision/status:** Case A is implemented and review-clean for the bounded Linux R1 scope. This is a
security-contract correction from **physically ACL-free** to **no effective other-principal
authority**. R2-0 planning is complete; R2 implementation and R3 remain separately sequenced and
unstarted. No seam is added or promoted.

The bounded primary-source proof is:

1. [`getxattr(2)`](https://man7.org/linux/man-pages/man2/getxattr.2.html) defines `ENODATA` for
   either a nonexistent named attribute or lack of process access. R1 therefore records exact
   `ENODATA` as `NoData`—only “the kernel returned no ACL data”—and never as physical absence.
2. [`acl(5)`](https://man7.org/linux/man-pages/man5/acl.5.html) maps `ACL_MASK` to group-class mode
   bits and applies the mask to named-user, group-object, and named-group access. Linux
   [`fs/posix_acl.c`](https://github.com/torvalds/linux/blob/master/fs/posix_acl.c) performs the same
   mask intersection and mode mapping. Thus, for supported Linux POSIX access ACLs, a named
   principal cannot retain effective write while the descriptor's authoritative group-write mode
   bit is clear; raw write fully removed by the mask is not effective authority.
3. `acl(5)` distinguishes an access ACL, which governs the current object, from a default ACL,
   which initializes a created child's access ACL. A default ACL does not grant access to its
   directory. Exact final-root `0700` prevents other-principal traversal, and exact owner-only
   descendant `0700`/`0600` modes prevent inherited Linux POSIX entries from granting effective
   other-principal authority.
4. Linux [`security/security.c`](https://github.com/torvalds/linux/blob/master/security/security.c)
   mediates POSIX-ACL and xattr reads. `ENOTSUP`, malformed/unsupported bytes or model, and every
   distinguishable non-`ENODATA` read failure therefore remain `Failed` and fail closed.
5. The [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/)
   continues to govern the existing user-specific placement/default inputs; it supplies no ACL or
   authority attestation. Physical ACL-xattr absence, if ever required, needs a separately approved
   privileged platform-attestation boundary. R1 neither designs nor implies one.

The canonical observation is `Present(bytes) | NoData | Failed(error_class)`. Ancestor access
`Present` is strictly parsed and mask-evaluated: non-writing `--x`/`r-x` is accepted, any effective
write rejects, and all base/mask/tag/order/uniqueness/version/length invariants are mandatory.
Ancestor default `Present` rejects before creation. Final-root access or default `Present` rejects;
qualified `NoData` requires exact owner, directory type, exact `0700`, stable descriptor identity,
no-follow traversal, no replacement, and authoritative safe mode. Existing invalid roots remain
unchanged and no ACL cleanup/repair is permitted.

Diagnostics use at least `PresentAcceptedNoEffectiveWrite` (ancestor access only),
`PresentRejected`, `NoDataAcceptedUnderModeAuthority`, and `FailedOrUnavailable`. They carry the
requested path, actual offending path, path role, ACL kind, reason class, and candidate-creation
state (`yes`, `no`, or `unknown`). They never call `NoData` absent/ACL-free, disclose ACL principals
or authority payloads, imply repair, or attribute an ancestor failure to the final root.

**R1 regression gate:** the authorized two-file runtime change must prove all of the following
without removing, renaming, substituting, or weakening inherited tests:

1. safe ancestor `ENODATA` accepts as `NoData`, never `Absent`;
2. ancestor `ENODATA` with group/world write rejects by mode authority;
3. effective-write ancestor access `Present` rejects;
4. effective `--x` access `Present` accepts;
5. effective `r-x` access `Present` accepts;
6. raw write fully removed by `ACL_MASK` accepts;
7. multiple named entries are mask-evaluated correctly;
8. final-root access `Present` rejects;
9. ancestor default `Present` rejects before candidate creation;
10. final-root default `Present` rejects;
11. non-`ENODATA` retrieval failure rejects;
12. unsupported ACL state rejects;
13. malformed version, length, order, duplicate, missing base/mask, and unsupported tag reject;
14. diagnostics never claim `NoData` proves absence;
15. diagnostics identify the actual offending ancestor and path role;
16. final-root owner and `0700` remain exact under umasks `000`, `022`, `027`, `077`, and `777`;
17. final-root traversal remains no-follow and identity-stable;
18. replacement and identity drift reject;
19. invalid existing roots retain exact metadata and contents;
20. no ACL cleanup or repair occurs;
21. a real `libvirt-qemu:--x` ancestor fixture passes; and
22. world, policy, receipt, supervisor, and retained-runtime behavior remains untouched.

The proof wall additionally requires focused trusted-filesystem and home-bootstrap tests, complete
HostSessionAuthority tests, product-path bootstrap scaffold tests, relevant installer-environment
regressions, formatting, warnings-denied Clippy for touched targets, workspace all-target check,
`git diff --check`, and GitNexus change detection. The shell differential must preserve the
inherited `1054 passed / 149 failed` baseline with `PassToFail = 0`, `NewFail = 0`, `Removed = 0`,
`RenamedOrSubstituted = 0`, unchanged retained failure names/signatures, audited `FailToPass`, and
passing new R1 tests. This does not claim the later privileged Linux installer/product wall, R2
prefix propagation, R3 cleanup/idempotency, native macOS proof, or cross-platform completion.

**R1 recorded result (2026-07-17):** the source began at
`ebbc5d5649be5ac0c006aac5806e60701e9903da`. The original two-file WIP is preserved on
`feat/preserve-a1-1d-5r1-linux-acl-wip-20260717` at
`395705a5b893aa7704e3a424904a8cd42f247dd3` with exact parent `ebbc5d56`. Its binary and ordinary
patch SHA-256 are both `e852cb8da883e2bffaecde3b46502ee374010a74d56584f1846707c94749719f`;
the original file SHA-256 values are `c0655e3fd0a7b5d5b31cdbdc9219265c52fbc5be9fbde084c66cd171dba1767e`
for `trusted_fs.rs` and `0aad8f5bc01a41b98de174842e92ec2910f12b9a946ad965788d33bab10ed391`
for `home_bootstrap.rs`. The docs-only contract commit is
`17ea3a839345cd47a5b2409cde0d4facdde09446`; the separate two-file runtime commit is
`4d0acff68e20d86b97fe5367b8a4617554f33ef4`. Its final per-file SHA-256 values are
`eeb55bef5c4ef74cf2cb6d0ff5ca723bc8b35417a4d8b12cc0f1e0f660edc994` and
`861e5c5e71e1cde9480ea9d502565edac217eb8db10866e98cffcc51fff8eb9a`, respectively; its binary
diff SHA-256 is `1c9be9810e7f0048d8265b8fb79e7fd3920a0b6e0b34e2e05d4d377f088b7c99`.

The exact proof commands/results were:
`cargo test -p shell --lib execution::agent_runtime::host_session_authority::trusted_fs::platform::tests`
**48 passed**; `cargo test -p shell --lib execution::home_bootstrap::tests` **9 passed**;
`cargo test -p shell --lib execution::agent_runtime::host_session_authority` **165 passed**;
`cargo test -p shell --test world_deps_home_scaffold_wdh3` **14 passed**;
`cargo test -p shell --test installer_env_wcu4` **4 passed / 1 inherited hard-coded `/tmp`
failure unchanged**; `cargo test -p shell --test world_deps_scaffold_wdh3` **0 passed / 2 inherited
hard-coded `/tmp` failures unchanged**; `cargo clippy -p shell --lib --tests -- -D warnings`,
`cargo check --workspace --all-targets`, `cargo fmt --all -- --check`, and `git diff --check` all
passed. The live shell comparator moved from inherited **1054 passed / 149 failed / 1203 total** to
**1080 passed / 149 failed / 1229 total**: `PassToFail = 0`, `FailToPass = 0`, `NewFail = 0`,
`Removed = 0`, and `RenamedOrSubstituted = 0`; all 149 retained names and normalized failure
signatures were unchanged, and all 26 added R1 tests passed.

R2-1 differential comparison uses that 1080/149/1229 result with one explicit accounting
exception: exactly the canonical old/new trigger-name mapping above is accepted, and one new
`test_version_is_non_mutating` must pass. `PassToFail = 0`, `NewFail = 0`, retained failure names and
normalized signatures remain unchanged, no test is removed or substituted, and no other rename is
accepted. The mapped test preserves the R1 scaffold fixture/assertions and changes only its
authenticated bootstrap trigger.

GitNexus change detection mapped the two authorized runtime files to 132 changed symbols and 24
existing downstream flows, with aggregate CRITICAL risk from the already authorized
HostSessionAuthority revalidation surface; it found no new world/policy/gateway product-flow owner.
The docs proof reviewers `/root/docs_acl_proof_review_2` and
`/root/docs_security_boundary_review_2` returned CLEAN after one stale phrase was corrected. Initial
runtime reviewers `/root/runtime_acl_vfs_review_1`, `/root/runtime_security_race_review_1`, and
`/root/runtime_diagnostics_compat_review_1` identified final-snapshot/mask/race and candidate-
provenance gaps; those were remediated with red/green tests. Fresh reviewers
`/root/runtime_acl_vfs_review_2`, `/root/runtime_security_race_review_2`,
`/root/runtime_diagnostics_compat_review_2`, and `/root/runtime_integrated_r1_review_1` then returned
CLEAN against the frozen runtime hashes. No privileged installer/product wall, native macOS proof,
or cross-platform completion is claimed.

### A1.1d-5R2-1 host-context and Unix-dev closeout

**Decision/status:** A1.1d-5R2-1 is implementation- and review-complete through runtime commit
`2653c2ef20ae2e119a444811e6fb46e86d1a6ec6`. The separately published contract corrections are
`e42b1a1ead9dbbebd399e6ba6ab056e342a3b4db` (authenticated hidden home-bootstrap action) and
`1565c2ac0af358b25ef8499cddee1daea051c7b0` (staged trace binding). Runtime commits are
`1acc8c70`, `d9d991b6`, `aea192f6`, `85798bfe`, `fee75ff0`, `af85adf0`, and `2653c2ef`.

The shared transport codec proves exact IH line framing/base64url/SHA-256 commitment and strict
missing, duplicate, unknown, malformed, reordered, path, commitment, and principal rejection. Unix
dev install/uninstall/dev-shim and standalone witnesses construct or validate one context; hidden
argv alone selects internal mode; the authenticated `--install-bootstrap-home-v1` action binds the
current account+UID and checked H/R before the unchanged explicit-context R1 bootstrap. The
unprivileged A/B product matrix proves custom A remains authoritative for bootstrap, shim
deploy/remove/status/doctor/repair, generated env/manager/preexec/helper projections, uninstall,
shell `A/trace.jsonl`, and policy Git A, with no product fallback/access under B. Parse/help/version
exits are non-mutating. The one scaffold-trigger rename preserves its fixture/assertions, and the
new version-nonmutation test passes.

Focused transport, IH, R1 private-home/HostSessionAuthority, installer, script, shim
deploy/status/doctor/health, trace, replay, and A/B suites passed. `cargo check --workspace
--all-targets`, shell all-target warnings-denied Clippy, `cargo fmt --all -- --check`, Bash syntax,
available ShellCheck, `git diff --check`, and GitNexus detection passed. The final broad shell run,
which became the **R2-2 historical starting baseline**, is **1089 passed / 149 failed / 1238 total**
versus inherited **1080/149/1229**: `PassToFail = 0`,
`NewFail = 0`, no removed/substituted test, and all 149 retained normalized failure signatures are
byte-identical after the already-audited dynamic orchestration-ID normalization
(`c818c4f2c4acce52ea5cb057020419bb10dd871148bafcc38112f13437aba6e9`). Windows source-only
comparison removes all 44 R2-1-introduced errors and adds none; the 47 retained errors are inherited,
so no Windows IH or platform capability is claimed.

Fresh read-only reviewers `/root/uid_lint_remediation_rereview`,
`/root/windows_cfg_remediation_review`, and `/root/final_integrated_r2_1_rereview` returned CLEAN.
The neutral trace setter, default compatibility callers, physical-shim/replay production files,
world-deps production, trace lifecycle semantics, shim replacement/migration/removal predicates,
R1 private-home security behavior, and every release/sudo/service/world/platform/lifecycle owner are
unchanged. PI-118 and compatibility removal remain R2-3-owned. `RG-HOME-01`, `RG-INSTALL-01`,
A1.1d, A1, the B1/B2.1 joint closeout, and B3.1 remain open; no seam is promoted.

At R2-1 closeout, the historical next packet was **A1.1d-5R2-2 — Unix release, sudo, Linux service,
and runtime propagation**. Routes A–D have since become individually review-clean, their integration
closeout failed source closure, and R2-2E has since become review-clean. F0/F0a/F0b/F0-HC were then
review-clean, canonically closed out, and preserved. At that historical checkpoint R2-2F was the
next packet. After F and the renewed closeout, the plan then required the complete Linux
regression and normal product lifecycle smoke without outer overrides. That proof can unblock
A1.1d Linux closeout and the Linux
product-smoke portion of the B1/B2.1 joint closeout. Native macOS proof remains separately required
to close A1.1d, but is not a prerequisite for the B1/B2.1 architectural corridor. B3.1 remains
blocked on the joint closeout, and no A1.1d-5I evidence authorizes B3.1 or remediation work.

## A1.1d mandatory-review state

A1.1d-1 through A1.1d-4 are Linux implementation/review clean and preserved through
`ca9429e5`; the corrected A1.1d-5 private-home implementation is focused-test and implementation-
review clean through `faabed16`. A1.1d remains incomplete because the positive `RG-BASE-01` product
wall exposes a preexisting owner-transition handoff defect. A1.1d integrated Linux closeout remains
open and cross-platform closeout remains pending. The exact cross-packet hold makes A1.1e
implementation-ready without declaring A1.1d complete; A1.1e is now focused-proof and review clean
through `cd676614`. This does not close A1.1d, its integrated/cross-platform proof, or any
cross-document seam.

The later A1.1d-5R1 Case A contract is recorded by
`17ea3a839345cd47a5b2409cde0d4facdde09446` and its bounded Linux runtime is review-clean through
`4d0acff68e20d86b97fe5367b8a4617554f33ef4`. This corrects only effective-authority semantics and
diagnostics; it does not supersede the preserved [A1.1d](#a11d-mandatory-review-state) and
[A0](#a0-closeout-evidence) historical checkpoints, close A1.1d/A1, or start R2/R3.

The rejected A1.1d-6 heartbeat hypothesis is recorded as `PreexistingExposedByA1d`. With the same
hermetic lifecycle fixture, pre-A1.1d `c800436d` and pre-rejection `f73e8a81` pass three of three
runs, while stale-rejection commit `bdb1796d` and later `ca9429e5`, `c5117b51`, and `faabed16` fail
three of three. A focused observation-only heartbeat operation passed five StateStore tests, but the
public lifecycle still failed unchanged, proving heartbeat persistence is not the blocker. The
heartbeat remediation was withdrawn; no runtime heartbeat change remains and no A1.1d-6 is implied.

Bounded successor diagnostics show exact incompatible authority records. Current durable truth is
`Active`/`ParkedResumable`, owner PID zero, the prior authoritative participant, and a completed
startup prompt. The successor reconstructs and supplies `Allocating`/`ActiveAttached`, a new owner
PID and participant, no active handle, and a pending prompt. `persist_orchestration_session`
correctly refuses that lifecycle regression; `current_world_binding_session` then correctly rejects
the supplied whole-session record as `stale_world_binding_session_snapshot`. Pre-A1.1d `c800436d`
and pre-rejection `f73e8a81` passed only because stale overwrite remained possible. From
`bdb1796d` onward, stale-rejection and later revisions fail closed; that does not make the old
passing behavior correct.

Correct closure requires a revision-bound HostSessionAuthority transition that adopts exact current
parked truth into the successor episode or rejects/reconciles it idempotently. It cannot be achieved
by heartbeat-only writes, timeout inflation, retrying the stale snapshot, last-writer-wins, or
weakening stale checks. That is a broader owner seam than the proposed A1.1d-6 allowed scope, so the
current stop classification is `ArchitecturalOwnerChangeRequired`. A1.2 owns the durable
transition-protocol closure, including the rule that every applied Resume terminal outcome carries
an exact completion/post-turn-application pair. The active A1.3-P1 packet owns the real CLI/helper/REPL adoption, the bounded transport of exact
startup ownership acknowledgement or typed pre-ownership rejection/failure (never readiness,
PID/helper/socket posture, timeout, EOF, or local-error inference), and `RG-BASE-01` closure; the
older A1.3 and A1.3-P0 records remain held as historical fences only.

Unsafe ACL rejection is a negative security success, not positive product smoke. The positive
`RG-BASE-01` smoke must run separately against a valid owner-only private bootstrap home with mode
`0700`, no observable POSIX access/default ACL, and qualified mode-authority observations. A
Linux `ENODATA` result in that proof means only that the kernel returned no ACL data, never that an
ACL xattr is physically absent. A1.1d-5I captured the blocked real default path and isolated
effective/default ACL matrix: the supported host ancestor carried only a masked non-writing access
entry, while the final root was absent; a separate isolated default-ACL case proved inheritance and
candidate residue. This is now a product compatibility decision, not fixture-only or environmental
contamination. R1 Case A approves only strictly parsed masked non-writing Linux POSIX ancestor
access ACLs; every default ACL, effective write, unsupported model, and uncertain ACL state remains
rejected, and no enforcement is weakened by the audit.

The corrected A1 V1 identity boundary starts at the first successful no-follow child open beneath
the retained, validated parent. Candidate creation or `AlreadyExists` convergence precedes that
boundary; all later operations remain descriptor-relative and later replacement fails closed.
Neither malicious root nor malicious same-UID substitution before the first open is in scope, and
no portable atomic create-and-bind or privileged-broker claim is part of `RG-HOME-01`.

## A0 closeout evidence

A0 changed documentation only. It moved no authority decision, changed no production call path or enforcement point, added no runtime primitive, and preserved all compatibility behavior. The inventory in `02-seam-crosswalk.md` is therefore evidence of current ownership and coverage, not evidence that HostSessionAuthority or HostExecutionEpisode has landed.

The following 49 focused test commands passed on Linux on 2026-07-10; every command reported `1 passed; 0 failed`:

```text
cargo test -p shell public_start_persists_detached_session_when_hidden_owner_helper_exits -- --nocapture
cargo test -p shell --lib list_live_participants_filters_dead_owner_pid_rows -- --nocapture
cargo test -p shell --lib resolve_internal_continue_world_dispatch_target_accepts_retained_worker_after_owner_pid_exit -- --nocapture
cargo test -p shell --lib resolve_internal_stop_world_dispatch_target_accepts_non_authoritative_live_worker -- --nocapture
cargo test -p shell --lib dispatch_contract_stop_world_worker_spec64_recovery_harness_drives_the_real_refreshed_transport_failure_branch -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 c3_internal_toolbox_stop_world_worker_treats_disappearing_private_stop_delivery_as_fail_closed -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 c3_internal_toolbox_stop_world_worker_keeps_refused_transport_text_distinct_from_missing_transport -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 c3_startup_fail_closed_when_persistent_session_cannot_reach_ready -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 public_stop_cleanly_closes_same_durable_session_after_reattach -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 public_stop_fails_closed_without_same_episode_terminal_proof_even_if_later_state_reads_stopped -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 public_stop_refused_transport_stays_on_existing_connect_failure_surface -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 public_stop_timeout_wording_stays_distinct_from_missing_transport_and_stale_authority -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 public_reattach_and_fork_preserve_exact_session_and_lineage_contracts -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 public_turn_uses_persisted_attach_continuity_selector_when_recovering_detached_host_turns -- --nocapture
cargo test -p shell --lib helper_readiness_accepts_resume_one_turn_after_fast_detach_once_prompt_is_terminal -- --nocapture
cargo test -p shell --lib persist_orchestration_session_rejects_stale_active_snapshot_after_terminal_snapshot -- --nocapture
cargo test -p shell --lib dispatch_contract_fork_world_worker_times_out_when_stop_transport_publication_never_completes -- --nocapture
cargo test -p shell --lib wait_for_fork_child_durable_publication_allows_late_child_visibility_without_extending_stop_transport_budget -- --nocapture
cargo test -p shell --lib manual_reattach_verification_failure_releases_claimed_auto_attach_obligation -- --nocapture
cargo test -p shell --lib finalize_session_auto_attach_after_launch_returns_failed_closed_when_restore_check_fails -- --nocapture
cargo test -p shell --lib synchronize_repl_authoritative_world_binding_repairs_from_persisted_session_truth -- --nocapture
cargo test -p shell --lib synchronize_repl_authoritative_world_binding_prefers_shared_world_metadata_over_stale_session_truth -- --nocapture
cargo test -p shell --lib synchronize_repl_authoritative_world_binding_fails_closed_when_metadata_is_unreadable_even_if_live_member_truth_exists -- --nocapture
cargo test -p shell --lib start_member_runtime_reuses_parent_session_and_persists_world_binding -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 public_root_start_world_scope_starts_attached_host_session_with_world_binding_truth -- --nocapture
cargo test -p world-service bootstrap_completion_without_session_handle_emits_only_exit -- --nocapture
cargo test -p world-service bootstrap_completion_with_session_handle_emits_registered_then_exit -- --nocapture
cargo test -p world-service finish_bootstrap_preserves_retained_slot_when_session_handle_exists -- --nocapture
cargo test -p world-service register_member_replaces_stale_fork_child_in_same_slot -- --nocapture
cargo test -p shell --lib ensure_router_auto_attach_targets_exact_local_host_allows_untargeted_obligations -- --nocapture
cargo test -p shell --lib ensure_router_auto_attach_targets_exact_local_host_allows_same_host_targets -- --nocapture
cargo test -p shell --lib ensure_router_auto_attach_targets_exact_local_host_rejects_foreign_targets -- --nocapture
cargo test -p shell --lib dispatch_contract_persist_continue_world_worker_obligation_projects_supported_events_into_canonical_state -- --nocapture
cargo test -p shell --lib dispatch_contract_steering_policy_rejects_ephemeral_concurrency_cap_exceeded -- --nocapture
cargo test -p shell --lib active_ephemeral_terminal_truth_guard_publishes_failed_terminal_truth_on_drop_after_start -- --nocapture
cargo test -p shell --lib active_ephemeral_world_task_registry_round_trips_live_task_identity -- --nocapture
cargo test -p shell --lib prepare_member_replacement_runtime_preserves_resumed_from_lineage -- --nocapture
cargo test -p shell --lib terminalize_startup_prompt_failure_marks_accepted_prompt_failed -- --nocapture
cargo test -p shell --lib private_prompt_bridge_emits_terminal_failed_after_accepted_owner_drop -- --nocapture
cargo test -p world-service --test member_runtime_retained_lifecycle_v1 member_runtime_non_zero_submitted_turn_exit_cleans_active_turn_slot_without_deleting_retained_worker -- --nocapture
cargo test -p shell --lib inflight_attach_join_returns_authoritative_receipt_when_ready_state_already_exists -- --nocapture
cargo test -p shell --lib inflight_attach_join_does_not_accept_detached_live_owner_as_attached_success -- --nocapture
cargo test -p shell --lib classify_prompt_worker_error_treats_common_terminal_loss_errors_as_abnormal -- --nocapture
cargo test -p shell --lib shutdown_disposition_tracks_termination_cause -- --nocapture
cargo test -p shell --lib hidden_owner_private_stop_fails_closed_when_completion_never_resolves -- --nocapture
cargo test -p shell --lib shutdown_host_orchestrator_runtime_parks_resumable_host_session_on_detach -- --nocapture
cargo test -p shell --lib can_park_host_runtime_after_detach_accepts_completed_one_turn_when_store_lags -- --nocapture
cargo test -p shell --lib host_inbox_state_store_materialization_is_serialized_for_concurrent_repeat_runs -- --nocapture
cargo test -p substrate-common test_substrate_home -- --nocapture
```

These exercise selected current behaviors relevant to `RG-AUTH-01`, expose selected still-current `RG-AUTH-02` dead-PID, PID-zero sentinel, and socket-gated startup/fork/public-stop behavior without normalizing it, and cover selected positive and negative `RG-CLOSE-01` baseline cases. They do not satisfy any complete regression gate or promote a seam. The session-snapshot test proves the narrow terminal-state guard only; no test submits conflicting `Active` snapshots and then a later heartbeat from the stale episode, so same-state revision safety remains an explicit A1 proof gap.

No production-path test proves revision-safe, idempotent binding establishment and failure clearing at startup, covers every effective-UID/XDG/`/run/user`/`/tmp` metadata namespace, or crashes/fails between world replacement, binding persistence, and older-generation worker invalidation, so binding reconciliation remains an explicit A1 proof gap. No test fails after initial `ActiveAttached` session persistence but before hidden-helper gateway/ownership establishment and then restarts to prove reconciliation without phantom attachment or stale rewrite. Arbitrary-path, stale/replayed helper plan, mismatched identity/binding, restart, and stale-revision rejection are also unproven. No focused test covers startup-prompt connect/EOF/duplicate/stale/restart phases, process-local startup-signal loss/timeout/duplicate/stale delivery, attach leader process loss/duplicate launch, or initial prompt/stop/cancel/toolbox registration failure while proving availability stays an episode observation; terminal-loss classification tests also do not prove revision-safe park/stop across restart, so those remain A1/A2 proof gaps. The selected private-stop tests cover individual helper failure, ordinary detach, completed one-turn, and public missing/refused/timeout branches, but no Start/Attach/ResumeOneTurn by host/member-role matrix combines stale `Running`, event/completion ordering, signal/helper/transport loss, stop, restart, stale revision, and repeated closeout while proving revision-safe outcomes. Lost/disconnected/duplicate/delayed auto-park delivery and completion ordering are likewise unproven, so helper intent/plan, parent-role gating, auto-park, and private-stop ownership/delivery remain explicit A1/A2/B3.2 proof gaps.

Remote member client-build/stream-open and post-accept ownership-persistence failure are not proven to retry/restart without a stranded local row, daemon slot, or duplicate; process-local dispatch caps, ephemeral identity/terminal wait, submitted-turn maps, and retained member admission/routing likewise lack multi-process/daemon-restart survival, so receipt/supervisor/runtime ownership remains a B0/B1/B2.1/B2.2/B3.1/B3.2/B4 gap. Existing member-runtime tests cover selected clean-exit/session-handle and stale-child cases; retained/ephemeral action-versus-`ash_` prefix mismatch, malformed identity, daemon restart followed by exact continue, duplicate participant/retained-key retry, the full clean/nonzero/cancel/missing-identity/stale-observer matrix, and restart between durable invalidation and replacement remain B3.2 proof gaps. No exact runtime-toolbox cancel test proves owner-PID loss reaches receipt-targeted cancel with distinct no-active, unavailable-owner, and terminal outcomes, so the pre-resolver PID gate remains a B4 gap. The fork tests prove current missing/late stop-socket handling, not that a durable child's lineage is independent from transport publication. Lossy fragments and the 100-byte shared-`/tmp` fallback lack colliding-ID, two-long-home, cross-UID/store, stale-path/symlink, permission, and cross-route negative proof; the session toolbox also lacks competing/stale-episode rebind proof, so exact plan/attach/startup/prompt/stop/cancel/toolbox addressing remains an A2/B3.2/B4 gap.

Existing host-target tests exercise the current untargeted/same-host/wrong-host behavior, but hostname absence/change/collision across restart is unproven and remains a C2/C3 gap. The common-path unit proves only the ordinary absolute `SUBSTRATE_HOME` result; no test creates conflicting state/config/policy/agent inventory under two homes or global/workspace precedence and crosses create/control, supplies a relative home across CWD changes, exercises empty/unset with missing `dirs::home_dir()`, forces CWD lookup failure/`.` fallback, or changes CWD between contract resolution, workspace persistence, and existing-session launch. Normalized store/workspace/config/policy/inventory identity and cross-home/cross-CWD fail-closed behavior therefore remain explicit A1/A3/E2/E3 proof gaps. The `SUBSTRATE_OVERRIDE_*` effective-config family also lacks changed/invalid/empty/removed create-control-restart proof. Anchor projection lacks invalid/empty mode/path, Project/FollowCwd/Custom, explicit project override, fallback, and CWD-change coverage. The concurrent inbox materialization test exercises threads within one process only; no proof runs two independent StateStore writers against one snapshot and demonstrates that conflicting lifecycle, binding, claim, inbox, or obligation updates cannot overwrite each other, so cross-process atomicity and revision safety remain explicit A1/A3 gaps. `SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE` and `SUBSTRATE_WORLD_EXEC_FORCE_DIRECT` have no production-ingress negative matrix or live transport/isolation smoke here; A0 records their actual roles without treating that missing B/D/E proof as success. `cargo fmt --all -- --check` and `git diff --check` passed. No external live-world doctor/smoke/e2e, full workspace test, or clippy was run because A0 changes no runtime behavior; those gates remain unavailable, and no focused unit/integration result is promoted to whole-seam proof. Remaining gaps are the unchanged A1/A2/A3 ownership moves and later retained-runtime/config/receipt work named by the inventory.
