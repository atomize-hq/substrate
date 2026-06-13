# Spec: Slice 11 Substrate-Owned Lifecycle and Diagnostics Contract

Source phase authority:
- [`../phase-3-substrate-owned-operations/README.md`](../phase-3-substrate-owned-operations/README.md)
- [`../phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md`](../phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md)

Source execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`](./design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)
- [`design/DESIGN-macos-ingress-and-mount-contract.md`](./design/DESIGN-macos-ingress-and-mount-contract.md)
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)
- [`design/DESIGN-macos-guest-unit-source-of-truth.md`](./design/DESIGN-macos-guest-unit-source-of-truth.md)

Neighboring slice authority:
- [`SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md`](./SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md)
- [`PLAN-10.md`](./PLAN-10.md)
- [`TASKS-10.md`](./TASKS-10.md)
- [`../phase-3-substrate-owned-operations/milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md`](../phase-3-substrate-owned-operations/milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md)

Required official source set for this slice:
- [Lima `limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
- [Lima SSH usage](https://lima-vm.io/docs/usage/ssh/)
- [Lima Filesystem mounts](https://lima-vm.io/docs/config/mount/)

Phase: `SPECIFY`
Status: draft slice authority
Slice focus: freeze the owned macOS operator contract around Substrate-first
lifecycle, diagnostics, gateway, and normal ingress/sync surfaces after Slice
`10`, without widening into the broad Slice `12` breakglass/docs cutover.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. The latest landed planning and closeout authority is Slice `10`, and Slice
   `11` is now the next honest dependency-ordered seam in the feature-local
   sequence.
2. Per [`../ROADMAP.md`](../ROADMAP.md), Slice `11` owns milestone `3.1`
   Substrate-managed lifecycle/diagnostics/sync productization, while Slice
   `12` still owns the broader breakglass reclassification and full docs
   cutover.
3. Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md), this slice requires
   `spec-driven-development` and a targeted `source-driven-development` pass,
   because the operator classification for `limactl shell`, direct SSH, and
   mount-backed guest access depends partly on official Lima semantics rather
   than repo wording alone.
4. Live repo truth on 2026-06-13 still shows the owned surfaces already landed
   in code and docs:
   - `docs/USAGE.md` already teaches `substrate host doctor [--json]`,
     `substrate world doctor [--json]`, `substrate world gateway
     sync|status|restart`, `substrate world enable`, and
     `substrate world deps current sync`,
   - `crates/shell/src/execution/workspace_cmd.rs` already exposes
     `substrate workspace sync` and explicitly requires world enablement plus
     `substrate world doctor`,
   - `scripts/mac/lima-doctor.sh` and `scripts/mac/smoke.sh` explicitly ignore
     `SUBSTRATE_WORLD_SOCKET` during routed proof and keep doctor/gateway-first
     readiness evidence central,
   - `crates/shell/src/builtins/world_enable/` already provides a CLI-owned
     provisioning surface even though it still routes through helper logic.
5. Live repo truth on 2026-06-13 still shows the operator story is not yet
   fully consolidated:
   - `scripts/mac/lima-warm.sh` remains the primary owner for create/start/
     repair automation and stages the requested checkout into
     `/var/lib/substrate/staged-workspace/current` via `limactl copy`,
   - `docs/reference/world/platforms/macos-lima-setup.md` still teaches direct
     `limactl shell substrate ...`, direct guest build/install, guest
     `systemctl`, guest `curl`, and guest `journalctl`,
   - `docs/WORLD.md` already centers doctor/gateway surfaces but still retains
     breakglass guest commands and `SUBSTRATE_WORLD_SOCKET` bypass guidance.
6. Slice `10` already unified the guest service/socket contract, so Slice `11`
   should consume the landed unit parity and routed readiness proof instead of
   reopening guest-unit or sandbox design.
7. Slice `09` already narrowed ingress toward the guest-local staged workspace,
   so Slice `11` should define the normal operator-facing sync/copy path on top
   of that result rather than reopen broad-mount design.
8. The smallest honest implementation is a contract/productization slice, not a
   broad docs rewrite: it may update operator-facing commands, helper
   messaging, validation flow, and narrow contract docs, but it should leave
   the full breakglass classification sweep and full setup/troubleshooting
   rewrite to Slice `12`.

If any of these are wrong, correct them before execution.

## Objective

Make the normal macOS same-user Lima operator path legible from Substrate-owned
commands first instead of from helper scripts and raw guest administration.

This slice is complete only when a reviewer can answer, without guessing:

1. which macOS lifecycle, diagnostics, gateway, and normal ingress/sync
   actions are already `supported`,
2. which remaining wrappers or helper-backed flows are
   `degraded-but-supported`,
3. which raw guest or bypass flows remain `breakglass`,
4. which command surfaces are the primary evidence wall for readiness, gateway
   lifecycle, repair, and sync,
5. whether create/warm/repair and normal sync behavior are taught through
   Substrate-owned entrypoints rather than raw `limactl shell` recipes,
6. what operator-facing work still remains explicitly deferred to Slice `12`.

## Commands and evidence surfaces

Use these as the default build, validation, and operator-evidence commands for
this slice:

```bash
cargo build --workspace
cargo test -p shell
cargo test -p world-mac-lima
target/debug/substrate host doctor --json | jq .
target/debug/substrate world doctor --json | jq .
scripts/mac/smoke.sh --gateway-conformance
tmp="$(mktemp -d)"; mkdir -p "$tmp/substrate-home/scripts/substrate"; cp scripts/substrate/world-enable.sh "$tmp/substrate-home/scripts/substrate/world-enable.sh"; chmod +x "$tmp/substrate-home/scripts/substrate/world-enable.sh"; target/debug/substrate world enable --home "$tmp/substrate-home" --dry-run; rc=$?; rm -rf "$tmp"; exit $rc
target/debug/substrate world deps current sync --dry-run --verbose
bin="$(pwd)/target/debug/substrate"; tmp="$(mktemp -d)"; ws="$tmp/ws"; mkdir -p "$ws"; "$bin" workspace init "$ws" >/dev/null && (cd "$ws" && "$bin" workspace sync --dry-run); rc=$?; rm -rf "$tmp"; exit $rc
scripts/mac/lima-doctor.sh
scripts/mac/smoke.sh
scripts/mac/orchestration-smoke.sh
```

If the implementation does not touch a given command family, use the smallest
relevant subset rather than mechanically running the entire wall.

## Repo surface map

Primary repo surfaces for this slice:

1. `crates/shell/src/execution/platform/macos.rs`
   - current host/world doctor ownership and routed readiness evidence
2. `crates/shell/src/builtins/world_gateway.rs`
   - current gateway lifecycle/status contract
3. `crates/shell/src/builtins/world_enable/`
   - current CLI-owned provisioning/enablement surface
4. `crates/shell/src/execution/workspace_cmd.rs`
   - current Substrate-owned workspace sync surface and gating
5. `scripts/mac/lima-warm.sh`
   - current create/warm/repair implementation owner
6. `scripts/mac/lima-doctor.sh`
   - current helper diagnostic wrapper
7. `scripts/mac/smoke.sh`
   - current routed readiness and gateway lifecycle smoke proof
8. `scripts/mac/orchestration-smoke.sh`
   - current shared-owner/orchestration proof surface
9. `docs/USAGE.md`
   - current top-level CLI command inventory
10. `docs/contracts/gateway/operator-contract.md`
    - current owned gateway operator contract
11. `docs/WORLD.md`
    - current runtime/operator contract narrative
12. `docs/reference/world/platforms/macos-lima-setup.md`
    - current macOS setup + breakglass-heavy workflow narrative

## Contract and wording conventions

This slice should preserve repo-native command and contract style:

1. reuse the exact taxonomy labels `supported`, `degraded-but-supported`, and
   `breakglass`,
2. present Substrate-owned commands before helper or guest-admin flows,
3. keep machine-readable surfaces authoritative where they already exist,
4. avoid inventing new public macOS lifecycle vocabulary when an existing
   command already covers the function,
5. keep `SUBSTRATE_WORLD_SOCKET` on macOS classified as
   advanced/test/breakglass rather than as the default Lima operator path.

Representative style for operator messaging:

```text
Primary path: run `substrate world doctor --json` and
`substrate world gateway status --json` first.
Breakglass only after routed checks fail.
```

## Testing strategy

This is a contract/productization slice, so verification should emphasize:

1. command-surface truth
   - the owned command matrix matches live CLI behavior
2. routed evidence first
   - doctor JSON and gateway status JSON remain the primary health and wiring
     artifacts
3. helper/wrapper truth
   - helper scripts do not claim broader ownership than the CLI actually
     exposes
4. shared-owner continuity
   - orchestration/shared-world proof remains supported where already landed
5. normal-path sync/lifecycle truth
   - the slice either teaches a real owned or degraded-but-supported sync/
     lifecycle path, or explicitly defers anything not yet honest

## Boundaries

### Always

1. Start from live repo truth, not stale docs or chat memory.
2. Preserve Slice `10` unit/source-of-truth and Slice `09` staged-ingress
   results.
3. Keep the supported path Substrate-first and the breakglass path explicit.
4. Tie every operator claim to a real command, script behavior, or contract doc.

### Ask first

1. Renaming public CLI commands.
2. Introducing a new public lifecycle command family instead of consolidating
   around existing surfaces.
3. Reopening the staged-workspace / mount-minimization design from Slice `09`.
4. Broadening into cross-platform lifecycle redesign outside the macOS seam.

### Never

1. Treat raw `limactl shell`, direct guest `systemctl`, guest socket `curl`, or
   host-side `SUBSTRATE_WORLD_SOCKET` override use as the normal macOS happy
   path.
2. Claim Linux-equivalent host ownership isolation for same-user Lima.
3. Silently let Slice `11` absorb the full Slice `12` docs/breakglass cutover.
4. Reopen Slice `07` listener narrowing or Slice `10` unit-unification unless a
   minimal correction is proven mandatory.

## Frozen in this slice

This slice freezes only:

1. the owned command matrix for macOS lifecycle, diagnostics, gateway status/
   lifecycle, and normal ingress/sync actions,
2. the classification of each relevant surface as `supported`,
   `degraded-but-supported`, or `breakglass`,
3. the minimum command/help/contract updates required so the operator story is
   honest,
4. the primary evidence wall for normal macOS operator validation,
5. the precise handoff boundary into Slice `12`.

## Deferred by design

This slice intentionally does **not** freeze:

1. the full setup/troubleshooting doc rewrite across all macOS docs,
2. the final broad breakglass reclassification sweep,
3. a new macOS-specific lifecycle command family if existing commands plus
   bounded wrappers are sufficient,
4. a reopened ingress redesign or mount-policy rethink after Slice `09`,
5. unrelated Linux or WSL operator-surface work.

## Why this slice exists

Slice `10` closed the guest unit parity seam, but it did not finish the
operator contract.

Live repo truth now makes Slice `11` the next honest seam:

1. the CLI already has real owned surfaces for doctor, gateway lifecycle, and
   enablement,
2. `substrate workspace sync` already exists in code but is not yet clearly
   integrated into the macOS operator story,
3. helper scripts and direct guest flows still define too much of the practical
   happy path,
4. without an explicit owned-command matrix, Slice `12` would be forced to do a
   broad docs cutover against an unstable operational contract.

## Packet 1 resolved source gate and operator-matrix freeze

Packet `1` freezes the source-gated operator direction for this slice.

Official-source-backed conclusions:

1. Lima documents `limactl shell` as an SSH-backed host command into the guest,
   so raw `limactl shell` access remains guest access rather than a
   Substrate-owned operator surface.
2. Lima documents plain SSH as an alternative interoperability path instead of
   `limactl shell`, so raw SSH access remains the same guest-access class for
   Slice `11`.
3. Lima documents host↔guest visibility as explicit filesystem mount
   configuration, so Slice `11` should consume the Slice `09` staged-ingress
   result rather than imply that mount-backed guest access is itself the normal
   operator sync contract.

Frozen Packet `1` operator matrix:

| Surface / workflow | Classification | Why this is the honest Packet `1` freeze | Operator direction |
| --- | --- | --- | --- |
| `substrate host doctor [--json]` | `supported` | Already documented in `docs/USAGE.md` and centered in `docs/WORLD.md` as the first routed readiness check. | Start normal routed readiness here. |
| `substrate world doctor [--json]` | `supported` | Already landed as the primary world-readiness report and preserves machine-readable routed evidence. | Use before guest-direct diagnosis. |
| `substrate world gateway sync|status|restart` (`status --json` authoritative) | `supported` | Already-landed CLI-owned gateway lifecycle/status family and the stable machine-readable gateway posture surface. | Keep gateway lifecycle and availability on the owned path. |
| `substrate world enable` | `supported` | CLI-owned provisioning/enablement surface already exists even though the current macOS implementation still routes through helper logic underneath. | Treat this as the owned provisioning entrypoint when provisioning is needed. |
| `substrate world deps current sync` | `supported` | Already-landed CLI-owned dependency-application surface documented in `docs/USAGE.md` and `docs/WORLD.md`; it applies the effective enabled deps list into the world without claiming to sync workspace source contents. | Use when enabled world deps change, but do not confuse it with the separate normal workspace sync/copy story. |
| `scripts/mac/lima-doctor.sh` after routed proof failure | `degraded-but-supported` | Repo truth already preserves it as a deeper troubleshooting helper, but only after doctor/gateway-first routed proof. | Keep it behind the routed CLI checks rather than as the first command. |
| Direct `scripts/mac/lima-warm.sh` create/start/repair plus its guest-local staged-workspace copy into `/var/lib/substrate/staged-workspace/current` | `degraded-but-supported` | This is still the live macOS lifecycle + normal workspace-ingress implementation owner, but it is helper-backed and not yet the final owned operator contract. | Freeze this as the truthful interim lifecycle/sync contract for Packet `1` while later packets decide how much to productize. |
| `substrate workspace sync` | deferred candidate, not the frozen normal path yet | The command exists and is world-gated, but Packet `1` evidence does not yet prove it is the current honest macOS default for lifecycle-adjacent workspace ingress. | Do not claim it as the normal macOS sync/copy story yet. |
| Raw `limactl shell` access, including ad hoc guest shelling | `breakglass` | Official Lima docs make it an SSH-backed guest-access primitive, not a Substrate-owned control-plane surface. | Reserve for recovery, deep debugging, or bounded post-failure diagnosis. |
| Plain SSH guest access | `breakglass` | Lima documents it as an interoperability alternative to `limactl shell`, so it stays in the same guest-access class. | Do not present as the default operator path. |
| Direct guest `systemctl`, guest socket `curl`, and guest `journalctl` | `breakglass` | These probe or mutate guest implementation details directly instead of going through doctor/gateway-first routed proof. | Use only after owned surfaces fail or when explicitly doing deep diagnosis. |
| Host-side `SUBSTRATE_WORLD_SOCKET=<path>` override use on macOS | `breakglass` | Routed helpers already suppress it during readiness proof and `docs/WORLD.md` treats it as advanced/test/bypass behavior. | Keep it outside the normal Lima-backed operator story. |

Frozen normal sync/copy direction for Packet `1`:

1. Slice `11` does **not** yet freeze `substrate workspace sync` as the normal
   macOS operator sync/copy path.
2. The honest current normal-path contract is the explicit guest-local staged
   workspace flow already owned by `scripts/mac/lima-warm.sh`, which therefore
   remains a bounded `degraded-but-supported` interim sync/lifecycle contract.
3. `substrate world deps current sync` is already a `supported` owned surface
   for dependency reconciliation into the world, but it is not the same thing
   as syncing workspace source contents into the guest.
4. Later packets may promote a more owned CLI-facing sync story, but Packet `1`
   must not pretend that productization is already finished.

## Validation command wall

Use these commands as the default evidence wall for this slice when they become
relevant during implementation:

```bash
cargo test -p shell
cargo test -p world-mac-lima
target/debug/substrate host doctor --json | jq .
target/debug/substrate world doctor --json | jq .
scripts/mac/smoke.sh --gateway-conformance
tmp="$(mktemp -d)"; mkdir -p "$tmp/substrate-home/scripts/substrate"; cp scripts/substrate/world-enable.sh "$tmp/substrate-home/scripts/substrate/world-enable.sh"; chmod +x "$tmp/substrate-home/scripts/substrate/world-enable.sh"; target/debug/substrate world enable --home "$tmp/substrate-home" --dry-run; rc=$?; rm -rf "$tmp"; exit $rc
target/debug/substrate world deps current sync --dry-run --verbose
bin="$(pwd)/target/debug/substrate"; tmp="$(mktemp -d)"; ws="$tmp/ws"; mkdir -p "$ws"; "$bin" workspace init "$ws" >/dev/null && (cd "$ws" && "$bin" workspace sync --dry-run); rc=$?; rm -rf "$tmp"; exit $rc
scripts/mac/lima-doctor.sh
scripts/mac/smoke.sh
scripts/mac/orchestration-smoke.sh
```

If execution widens into a specific CLI builtin or helper script, add the
smallest targeted test coverage that proves the widened surface explicitly.

## Success criteria

1. One reviewable doc section or contract matrix lists the normal macOS
   lifecycle, diagnostics, gateway, and sync surfaces without requiring guest
   admin recipes to understand the happy path.
2. `substrate host doctor`, `substrate world doctor`, and
   `substrate world gateway sync|status|restart` remain explicitly central to
   the supported path.
3. The slice classifies `substrate world enable`, helper-backed lifecycle
   flows, and the chosen normal sync/copy path honestly as `supported` or
   `degraded-but-supported` rather than by implication.
4. Raw `limactl shell`, direct SSH guest access, direct guest `systemctl`,
   direct guest socket probing, and host-side `SUBSTRATE_WORLD_SOCKET` override
   use remain clearly outside the default supported path.
5. Validation and smoke evidence can be gathered through the owned path before
   escalating to breakglass checks.
6. Slice `12` remains clearly deferred for the broad docs/breakglass cutover.
