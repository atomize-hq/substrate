**Kind:** gate
**Status:** canonical lane-local authority-required gate
**Canonical for:** `AUTHORITY_REQUIRED:MACOS_DEV_PARITY`

## `AUTHORITY_REQUIRED:MACOS_DEV_PARITY` contract (2026-08-19; macOS lane)

This gate supersedes the archived R3 macOS/recovery/retirement/finalizer/E03 and Windows-predecessor
contracts above **for macOS-lane scheduling only**. It is not the global product-work predecessor,
is not self-authorizing, and Phase 1 grants no Phase 2 implementation or native-operation authority.

### Admission

A later parity task must freshly bind live Git/source truth, exact edit and symbol fences, current
installer/runtime behavior, the selected user prefix, selected Lima instance, forwarding endpoint,
Linux-preservation obligations for shared scripts, and native action/restoration rules. Before any
effect it must complete an exact read-only overlap check, without a Keychain query, showing that
those declared developer resources are disjoint from Attempt 4. Any actual path/resource collision
requires a separately authorized disposition and leaves this gate open.

### Allowed completion claim

The gate may close only after the current product proves user-owned-prefix install/uninstall,
current shims/configuration/binary staging, current Lima/`world-service` provisioning, typed
selected-prefix and Lima-instance mapping, safe host-to-guest forwarding, and native install →
exercise world → uninstall → verify → reinstall behavior. Exact intended removal and preservation
of unrelated/pre-existing state are part of acceptance. Static checks alone are insufficient.

### Prohibited ownership and actions

The parity corridor owns no System-Keychain record, protected publisher, macOS lifecycle
LaunchDaemon/privileged host helper, terminal-retirement/finalizer state, E03/freeze/identity-
rotation/assurance evidence, or Attempt 4 artifact. It must not inspect or mutate Attempt 4
Keychain records; retire, migrate, overwrite, adopt, or clean its fixed privileged artifacts; use
unfinished lifecycle machinery as cleanup; or revive the archived architecture.

Landed Linux R3 facts remain historical facts and receive no new implementation/evidence claim;
later shared-script changes must preserve Linux behavior. Windows remains untouched and incomplete
where applicable, outside this lane, and deferred until a separately authorized post-runtime-refactor
scheduling decision.

### Exit and continuation

Any ambiguity, overlap, protected-lifecycle dependency, unrelated-state change, native failure, or
inexact uninstall/restoration keeps `AUTHORITY_REQUIRED:MACOS_DEV_PARITY` open. Its closure neither
blocks nor authorizes the Linux-first runtime-refactor sequence; the global reentry gate separately
requires a fresh live-repository bind and exact packet selection. This contract does not dispatch
A1.3, A1.4, Windows, E03, or any successor. Protected machinery can return only through a separate
production threat-model decision.

The complete decision record is
[`macos-dev-parity/DECISION.md`](../macos-dev-parity/DECISION.md).

**Source provenance:** extracted from [`04-contracts-and-gates.md#authority_requiredmacos_dev_parity-contract-2026-08-19-macos-lane`](../04-contracts-and-gates.md#authority_requiredmacos_dev_parity-contract-2026-08-19-macos-lane), baseline lines 10268–10316; the one repository-relative Markdown target for `macos-dev-parity/DECISION.md` was rebased by one parent directory to preserve its original root-level target after relocation under `gates/`
**Baseline span SHA-256:** `57be72cdbea67da769353346466e1316f25e283d029f16b956fde9600ae4b23e`
