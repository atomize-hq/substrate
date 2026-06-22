# Handoff: World-Deps ACL Bridge And Agent Doctor Cleanup

## Session Metadata
- Created: 2026-06-20 16:35:52
- Project: /home/spenser/__Active_code/substrate
- Branch: feat/internal-host-orchestrator-world-dispatch-bootstrap
- Session duration: ~2.5 hours

### Recent Commits (for context)
  - fec0398c patch update installer and gateway smoke scripts
  - c1ec6f0e Wrap session root read error handling
  - be14fc3a patch: harden session recovery test
  - a41c4ddd patch: dns world resolution timeout
  - f2727d25 patch: update codex-runtime guest install itself

## Handoff Chain

- **Continues from**: [2026-06-15-224054-substrate-codex-toolbox-uds-investigation.md](./2026-06-15-224054-substrate-codex-toolbox-uds-investigation.md)
  - Previous title: Substrate Codex Toolbox UDS Investigation
- **Supersedes**: None

> Review the previous handoff only if you need earlier Linux socket/bootstrap context. This handoff is self-contained for the current bug and next implementation pass.

## Current State Summary

The session started with a post-SPEC regression in `scripts/substrate/dev-fresh-install-gateway-smoke.sh`: `substrate world gateway status` and `sync` were failing with `gateway_invalid_integration` because placement-aware exact backend ids like `cli:codex-host` were still being validated against legacy exact backend assumptions. That gateway issue was fixed in code and verified against a fresh private `world-service` socket. After a full reinstall, the remaining failure moved to `substrate agent doctor --json`: `member_selection` still reports `codex-world` as not runtime-realizable because `/var/lib/substrate/world-deps/bin/codex` is "unavailable". Investigation proved that the runtime is actually present and applied, but the current host shell cannot traverse `/var/lib/substrate/world-deps` because it lacks active `substrate` group membership. The recent Linux ACL bridge fix only covers `/run/substrate.sock`, not the world-deps tree. The agreed next change is to extend the immediate first-run ACL bridge model to the world-deps tree and also clean up doctor diagnostics so permission-denied is not misreported as missing/unavailable.

## Codebase Understanding

### Architecture Overview

- There are now two distinct Linux first-run access surfaces:
  - `/run/substrate.sock` for host-to-world control, already bridged by named-user ACLs via systemd `ExecStartPost`.
  - `/var/lib/substrate/world-deps/...` for host-side validation/probing of world runtime assets, currently still gated by live group membership and normal filesystem traversal.
- `substrate host doctor --json` only reasons about socket access. It can report `ok.named_user_acl` even when the current shell does not have active `substrate` supplementary groups.
- `substrate agent doctor --json` reaches `member_selection` via `validate_member_selection()` and world-scoped Codex realizability via `resolve_world_scoped_codex_binary_path()` in `crates/shell/src/execution/agent_runtime/validator.rs`.
- That realizability path does a direct host `std::fs::metadata()` probe on `/var/lib/substrate/world-deps/bin/codex` (or the `SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR` override), so it fails on `EACCES` before it can distinguish presence from absence.
- `substrate world deps current list applied --json` is not the source of truth for the realizability check today. It can say `codex-runtime` is `present` while `agent doctor` still fails because the host process cannot traverse the path.
- The gateway regression fixed earlier in the session had two layers:
  - shared transport auth validation needed to accept realized backend ids (`cli:codex-host/world`, `cli:claude_code-host/world`)
  - world-service gateway runtime binding needed to understand realized backend ids while still emitting canonical auth-bundle backend ids (`cli:codex`, `cli:claude_code`, `api:openai`) to `substrate-gateway`

### Critical Files

| File | Purpose | Relevance |
|------|---------|-----------|
| [crates/shell/src/execution/agent_runtime/validator.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/agent_runtime/validator.rs) | Host-side runtime realizability and member selection checks | Current bug source: world-scoped Codex path probe treats permission denied as unavailable |
| [scripts/linux/world-provision.sh](/home/spenser/__Active_code/substrate/scripts/linux/world-provision.sh) | Linux install/provision path for systemd units, ACL helper install, socket restart | Existing socket ACL bridge pattern likely needs extension for world-deps ACL projection |
| [scripts/linux/substrate-apply-socket-acl.sh](/home/spenser/__Active_code/substrate/scripts/linux/substrate-apply-socket-acl.sh) | Named-user ACL projection helper for authorized `substrate` users | Candidate to generalize or clone for world-deps tree ACL projection |
| [scripts/substrate/dev-install-substrate.sh](/home/spenser/__Active_code/substrate/scripts/substrate/dev-install-substrate.sh) | Dev install workflow and post-provision messages | Relevant because it currently warns only about socket bridge behavior |
| [scripts/substrate/dev-fresh-install-gateway-smoke.sh](/home/spenser/__Active_code/substrate/scripts/substrate/dev-fresh-install-gateway-smoke.sh) | Fresh-install gateway smoke helper | Already adjusted so pre-sync `world gateway status` exit `4` is treated as expected on a fresh daemon |
| [crates/transport-api-types/src/lib.rs](/home/spenser/__Active_code/substrate/crates/transport-api-types/src/lib.rs) | Shared gateway request/auth validation | Fixed earlier in the session for placement-aware realized backend ids |
| [crates/world-service/src/gateway_runtime.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/gateway_runtime.rs) | Gateway runtime backend binding and auth handoff | Fixed earlier in the session for realized host/world backend ids with canonical bundle backend ids |
| [crates/world-service/src/service.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/service.rs) | World-service request preparation and gateway runtime binding tests | Contains regression coverage for `cli:codex-host` request preparation |
| [docs/contracts/gateway/operator-contract.md](/home/spenser/__Active_code/substrate/docs/contracts/gateway/operator-contract.md) | Operator contract for gateway lifecycle/status semantics | Important because `status` is allowed to be `unavailable` before first successful `sync`/`restart` |

### Key Patterns Discovered

- Linux first-run access bridging is intentionally durable-plus-immediate:
  - durable access via `root:substrate` ownership/mode and account-group membership
  - immediate first-run access via named-user ACL projection for authorized users whose shell groups are stale
- The ACL helper pattern is fail-open for observability and fail-closed nowhere:
  - `ExecStartPost=-...`
  - helper logs degraded cases and exits `0`
  - service/socket stay up even if ACL projection is degraded
- The helper discovers authorized users from:
  - explicit `getent group substrate` members
  - `getent passwd` entries whose primary GID equals the `substrate` GID
- Gateway runtime backend selection now has a useful split:
  - exact placement-aware backend ids for selection/routing (`cli:codex-host`, `cli:codex-world`, etc.)
  - canonical backend ids for the auth bundle consumed by `substrate-gateway` (`cli:codex`, `cli:claude_code`, `api:openai`)
- The doctor/validator path currently collapses host `EACCES` into the same "unavailable" message as a truly missing guest runtime, which is the diagnostic cleanup requested by the user.

## Work Completed

### Tasks Finished

- [x] Traced the original `world gateway status` / `sync` `gateway_invalid_integration` regression to legacy exact-backend auth-facet checks after placement-aware backend cutover.
- [x] Ran GitNexus impact/context on the gateway integration edit points before patching.
- [x] Fixed transport-layer integrated-auth validation to accept realized exact backend ids for Codex and Claude Code placements.
- [x] Fixed world-service gateway runtime binding/auth handoff to accept realized host/world exact backend ids while preserving canonical auth-bundle backend ids for `substrate-gateway`.
- [x] Added targeted regression tests covering realized Codex/Claude backend ids and world-service request preparation/binding lookup/auth handoff.
- [x] Updated both fresh-install gateway smoke helpers so pre-sync `world gateway status` exit `4` is treated as an expected fresh-daemon state and the helper continues to `world gateway sync`.
- [x] Reproduced and explained the remaining `substrate agent doctor --json` failure after reinstall.
- [x] Proved the remaining failure is not a missing runtime, but a host-shell permission/traversal problem on `/var/lib/substrate/world-deps/...`.
- [x] Reached decision with user: implement ACL bridge extension for world-deps tree (`#1`) and include doctor diagnostic cleanup (`#2`) in the same pass.
- [x] Reached decision with user: this is small/contained enough to land in a single coding session without a full spec/plan/tasks planning pack.

### Files Modified

| File | Changes | Rationale |
|------|---------|-----------|
| [crates/transport-api-types/src/lib.rs](/home/spenser/__Active_code/substrate/crates/transport-api-types/src/lib.rs) | Accept realized exact backend ids for integrated auth facet validation; add regression tests | Fixes post-SPEC placement-aware gateway auth validation regression |
| [crates/world-service/src/gateway_runtime.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/gateway_runtime.rs) | Add realized host/world backend bindings; preserve canonical auth-bundle backend ids; add tests | Fixes world-service side of placement-aware gateway routing/auth handoff |
| [crates/world-service/src/service.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/service.rs) | Add `cli:codex-host` request-preparation regression test | Guards the transport/request-preparation layer |
| [scripts/substrate/dev-fresh-install-gateway-smoke.sh](/home/spenser/__Active_code/substrate/scripts/substrate/dev-fresh-install-gateway-smoke.sh) | Treat pre-sync status exit `4` as expected and continue to sync | Align helper with gateway operator contract |
| [scripts/substrate/dev-fresh-install-gateway-smoke-claude-code.sh](/home/spenser/__Active_code/substrate/scripts/substrate/dev-fresh-install-gateway-smoke-claude-code.sh) | Same helper ordering fix for Claude Code variant | Keep both smoke helpers consistent |
| [handoffs/2026-06-20-163552-world-deps-acl-bridge-and-agent-doctor.md](/home/spenser/__Active_code/substrate/handoffs/2026-06-20-163552-world-deps-acl-bridge-and-agent-doctor.md) | Session handoff | Preserve context for next session |

Note: by the end of the session the working tree was clean except `handoffs/`, so the gateway-related code changes appear to already be reflected in the branch state rather than remaining as uncommitted diffs. The key remaining implementation work for the next session is the world-deps ACL bridge and diagnostic cleanup, which has not been coded yet.

### Decisions Made

| Decision | Options Considered | Rationale |
|----------|-------------------|-----------|
| Treat pre-sync `substrate world gateway status` exit `4` as expected in fresh-install smoke | Keep `status` before `sync` and fail hard; reorder helper to `sync` first; tolerate exit `4` and continue | Operator contract explicitly allows `status` to be `unavailable` before first successful `sync`/`restart`, so the helper should not abort on that state |
| Support placement-aware realized exact backend ids in gateway validation/runtime binding | Preserve legacy exact ids only; add realized ids end-to-end | Required by the SPEC 58/59/60 landing; direct cause of the gateway integration regression |
| Preserve canonical gateway auth-bundle backend ids even when routing with realized exact backend ids | Emit realized exact backend ids into auth bundle; map realized ids back to canonical bundle ids | `substrate-gateway` already consumes canonical bundle backend ids; changing that would widen blast radius unnecessarily |
| Choose world-deps ACL bridge as the primary fix for the remaining doctor failure | `#1` ACL bridge for world-deps tree; `#2` only doctor workaround via applied-deps or special-cased probing | `#1` fixes the real resource-access problem consistently for any host-side probe/operator workflow, not just `agent doctor` |
| Include doctor diagnostic cleanup in the same pass | Defer diagnostic cleanup | Current message says "unavailable" even when the file exists but host traversal is denied; this is misleading and cheap enough to improve in the same session |
| Do not stop for a full spec/plan/tasks pack | Write full planning docs first | Scope is contained and well-defined; it extends an existing Linux ACL bridge pattern rather than creating a new product surface |

## Pending Work

### Immediate Next Steps

1. Implement Linux immediate ACL bridging for `/var/lib/substrate/world-deps` so authorized `substrate` users do not need `newgrp`/fresh shell to traverse the tree from host processes.
2. Update `crates/shell/src/execution/agent_runtime/validator.rs` so the world-scoped Codex realizability path distinguishes permission denied from genuinely missing runtime content, and reports actionable wording.
3. Add targeted tests/docs for the new world-deps ACL bridge behavior and rerun `substrate agent doctor --json` on a stale-group shell to verify the failure is gone.

### Blockers/Open Questions

- [ ] Decide whether to generalize `scripts/linux/substrate-apply-socket-acl.sh` into a path-ACL helper or add a sibling helper dedicated to recursive world-deps ACL projection.
- [ ] Decide where ACL reapplication must happen beyond install-time provisioning. Likely places: Linux provisioning, world-deps sync/install/provision surfaces after mutations, possibly any helper that rewrites wrappers/symlinks under `/var/lib/substrate/world-deps`.
- [ ] Decide exact ACL semantics for world-deps tree projection. The next session should verify the minimum needed permissions on directories and files/symlinks to support `metadata()` probes and operator inspection without over-broadening access.

### Deferred Items

- No further gateway placement-aware work is pending from this session. The fresh-socket `world gateway sync`/`status` path is green.
- Full product/general design docs for the world-deps ACL bridge were deferred because the user agreed the change is contained enough to land directly.

## Context for Resuming Agent

### Important Context

The current user-visible failure is **not** another gateway integration bug. The gateway regression from earlier in the session was fixed and verified. The remaining issue is a host permission mismatch on the world-deps tree:

- `~/.substrate/bin/substrate world gateway sync` is now succeeding.
- `~/.substrate/bin/substrate agent doctor --json` is still failing `member_selection` with:
  - `selected runtime 'codex-world' is not runtime-realizable in world scope because guest entrypoint '/var/lib/substrate/world-deps/bin/codex' is unavailable; install the world runtime and rerun 'substrate world deps current sync'`
- That message is misleading. The runtime is actually present:
  - `~/.substrate/bin/substrate world deps current list applied --json` reports:
    - `codex-runtime`
    - `enabled: true`
    - `world: "present"`
  - `~/.substrate/bin/substrate --world -c 'ls -l /var/lib/substrate/world-deps/bin/codex'` succeeds and shows:
    - `/var/lib/substrate/world-deps/bin/codex -> /var/lib/substrate/world-deps/codex-runtime/bin/codex`
- From the host shell, the same path is not traversable:
  - `ls -l /var/lib/substrate/world-deps/bin/codex` -> `Permission denied`
  - `stat /var/lib/substrate/world-deps/bin/codex` -> `Permission denied`
  - `namei -l /var/lib/substrate/world-deps/bin/codex` shows `/var/lib/substrate` is `root:substrate` with mode `750`, and traversal fails at `world-deps`
- The live shell still lacks active supplementary groups:
  - `getent group substrate` includes `spenser`
  - `id -nG` does **not** include `substrate`
- `substrate host doctor --json` reports socket access is healthy via named-user ACL:
  - `active_process_has_socket_group: false`
  - `account_is_in_socket_group: true`
  - `named_user_acl_grants_rw: true`
  - `authorization_source: "named-user-acl"`
  - `status: "ok.named_user_acl"`

Conclusion:

- The recent "no shell reload required" Linux fix is working for `/run/substrate.sock`.
- It does **not** cover `/var/lib/substrate/world-deps`.
- `agent doctor` currently probes the host path directly and collapses `EACCES` into "unavailable".
- The next implementation should extend the immediate named-user ACL bridge model to the world-deps tree and improve doctor wording so permission problems do not masquerade as missing runtime content.

### Assumptions Made

- Assumed the user wants the robust product fix, not just a one-off workaround like `newgrp substrate`.
- Assumed the best fix is to mirror the existing Linux socket ACL bridge model onto world-deps rather than only teaching `agent doctor` to special-case this one probe.
- Assumed the contained scope does not require a new spec pack because the design direction is already constrained by the existing socket ACL bridge model.
- Assumed the world-deps ACL bridge should preserve the same degraded-but-non-disruptive operational posture as the socket ACL helper: log warnings, avoid tearing down services, and keep the platform usable even if ACL application partially degrades.

### Potential Gotchas

- Do not confuse socket access health with world-deps path access health. They are separate surfaces.
- `host doctor` being green does not imply `agent doctor` will be green from a stale-group shell.
- The world-scoped Codex probe follows a symlink into `/var/lib/substrate/world-deps/codex-runtime/bin/codex`, so ACL/traversal requirements likely apply to more than just `/var/lib/substrate/world-deps/bin`.
- If you only fix doctor wording (`#2`) without implementing world-deps ACL bridging (`#1`), the host shell will still be unable to traverse the runtime tree. The user explicitly prefers `#1` as the robust fix, with `#2` bundled in.
- The repo root contains a file named `.codex`, not a directory. The handoff skill had to be run with `--dir handoffs` because the default `.codex/handoffs` location is invalid here.

## Environment State

### Tools/Services Used

- GitNexus MCP:
  - used for impact/context on gateway edit points and runtime binding/auth handoff surfaces
- Local commands used heavily:
  - `rg`
  - `cargo test`
  - `cargo build`
  - `substrate host doctor --json`
  - `substrate world deps current list ... --json`
  - `substrate agent doctor --json`
  - `substrate --world -c ...`
  - `namei -l`
  - `getent group`
  - `id -nG`
- System state observed:
  - installed systemd daemon at `/usr/local/bin/substrate-world-service`
  - `/run/substrate.sock` access healthy via named-user ACL bridge
  - current host shell still missing active `substrate` supplementary group

### Active Processes

- No private debug `world-service` processes were left running at handoff time.
- Systemd-managed installed `substrate-world-service` may still be running normally via socket activation; nothing special was left attached to it in this session.

### Environment Variables

- `SUBSTRATE_WORLD_SOCKET`
- `SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR`
- `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID`
- `SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN`
- `CODEX_HANDOFF_DIR`

## Related Resources

- [docs/contracts/gateway/operator-contract.md](/home/spenser/__Active_code/substrate/docs/contracts/gateway/operator-contract.md)
- [docs/USAGE.md](/home/spenser/__Active_code/substrate/docs/USAGE.md)
- [docs/WORLD.md](/home/spenser/__Active_code/substrate/docs/WORLD.md)
- [scripts/linux/world-provision.sh](/home/spenser/__Active_code/substrate/scripts/linux/world-provision.sh)
- [scripts/linux/substrate-apply-socket-acl.sh](/home/spenser/__Active_code/substrate/scripts/linux/substrate-apply-socket-acl.sh)
- [scripts/substrate/dev-install-substrate.sh](/home/spenser/__Active_code/substrate/scripts/substrate/dev-install-substrate.sh)
- [scripts/substrate/dev-fresh-install-gateway-smoke.sh](/home/spenser/__Active_code/substrate/scripts/substrate/dev-fresh-install-gateway-smoke.sh)
- [crates/shell/src/execution/agent_runtime/validator.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
- [crates/transport-api-types/src/lib.rs](/home/spenser/__Active_code/substrate/crates/transport-api-types/src/lib.rs)
- [crates/world-service/src/gateway_runtime.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/gateway_runtime.rs)
- [crates/world-service/src/service.rs](/home/spenser/__Active_code/substrate/crates/world-service/src/service.rs)
- [handoffs/2026-06-15-224054-substrate-codex-toolbox-uds-investigation.md](/home/spenser/__Active_code/substrate/handoffs/2026-06-15-224054-substrate-codex-toolbox-uds-investigation.md)

---

**Security Reminder**: Before finalizing, run `validate_handoff.py` to check for accidental secret exposure.
