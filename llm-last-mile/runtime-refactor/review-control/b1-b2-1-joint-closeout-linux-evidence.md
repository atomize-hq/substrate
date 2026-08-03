# B1/B2.1 joint closeout Linux evidence

**Status:** supported Linux doctor plus installed-product smoke for `B1_B2_1_JOINT_CLOSEOUT`,
recorded against the bound 2026-08-03 source metadata; no product bytes changed.

## Bound source and artifact root

- Checkout: `/home/spenser/.codex/worktrees/9469/substrate`
- Bound commit: `f37943eb917285a044c5e12a05b481572c8d0a09`
- Bound tree: `54ae7b2a2d467a568b575665b99dcb94ef2893a2`
- Artifact root:
  `/home/spenser/.cache/substrate-runtime-refactor/b1-b2-1-joint-closeout-retry3-preflight-20260803T193755Z/linux-smoke`

## Commands

```bash
substrate world doctor --json > world-doctor-before.json
substrate --world --shim-skip -c true > world-command.stdout 2> world-command.stderr
substrate world doctor --json > world-doctor-after.json
```

## Runtime identity

- Observed UTC from `metadata.txt`: `2026-08-03T20:00:12Z`
- Product binary: `/home/spenser/.substrate/bin/substrate`
- Product version: `substrate 0.2.8`
- This packet preserved and exercised the existing installed product boundary; it did not rebuild,
  digest-bind, or otherwise source-prove that installed binary against the bound commit/tree.

## Doctor before

`world-doctor-before.json` reported:

- top-level `ok=true`
- `platform="linux"`
- `world_enabled=true`
- `world.ok=true`
- `world.status="ok"`
- `world.selected_host_prefix="/home/spenser/.substrate"`
- `world.host_context_commitment="3026bf26c312b93b6710de004a14d9d7286ef75d328c28319b45a8aa4da45b04"`
- `world.world_fs_strategy.primary="overlay"`
- `world.world_fs_strategy.fallback="fuse"`
- `world.world_fs_strategy.probe.result="pass"`
- `world.landlock.supported=true`
- `world.landlock.abi=7`
- `host.world_socket.mode="socket_activation"`
- `host.world_socket.socket_path="/run/substrate.sock"`
- `host.world_socket.socket_exists=true`
- `host.world_socket.socket_acl.owner_user="root"`
- `host.world_socket.socket_acl.group_name="substrate"`
- `host.world_socket.socket_acl.mode_octal="0660"`
- `host.world_socket.access.current_user="spenser"`
- `host.world_socket.access.active_process_has_socket_group=true`
- `host.world_socket.access.account_is_in_socket_group=true`
- `host.world_socket.access.named_user_acl_grants_rw=true`
- `host.world_socket.access.authorization_source="active-group"`
- `host.world_socket.access.status="ok.active_group"`
- `host.world_socket.access.contract_ok=true`
- `host.world_socket.probe_ok=true`
- `host.world_socket.systemd_socket.name="substrate-world-service.socket"`
- `host.world_socket.systemd_socket.active_state="active"`
- `host.world_socket.systemd_socket.unit_file_state="enabled"`
- `host.world_socket.systemd_service.name="substrate-world-service.service"`
- `host.world_socket.systemd_service.active_state="active"`
- `host.world_socket.systemd_service.unit_file_state="enabled"`

## Product smoke

`substrate --world --shim-skip -c true` exited `0`.

- `world-command.stdout`: `0` bytes
- `world-command.stderr`: `0` bytes

This is the bounded supported-Linux doctor plus installed-product smoke required by the joint
closeout row. It proves the existing world-backed command path remained available on the supported
Linux host while preserving the current product installation and socket-activation boundary. It is
not used as commit/tree provenance for the installed binary.

## Doctor after and restoration boundary

`world-doctor-after.json` remained green:

- top-level `ok=true`
- `world.ok=true`
- `world.status="ok"`
- `host.world_socket.probe_ok=true`
- systemd socket/service still `active` and `enabled`
- socket path, ACL owner/group/mode, and access status unchanged

The only observed semantic deltas between `world-doctor-before.json` and
`world-doctor-after.json` were:

- `world.collected_at_utc`: `2026-08-03T20:00:12Z` -> `2026-08-03T20:00:13Z`
- `world.policy_resolution_mode`: `null` -> `"snapshot_v3"`

No restoration action was required inside the tracked checkout. The smoke preserved the existing
product install, socket activation, ACL boundary, and supported Linux world-service posture.
