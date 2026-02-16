# Architecture and core flows
## Architecture diagram (conceptual)
```
+------------------------------- macOS host --------------------------------+
|                                                                           |
|  Substrate CLI / Manager                                                  |
|   - selects backend (VF vs Lima)                                          |
|   - compiles policy (cmd + fs + net)                                      |
|   - launches world                                                       |
|                                                                           |
|  VF backend (new)                                                         |
|   - VM lifecycle (start/stop/snapshot)                                    |
|   - virtiofs device(s)                                                    |
|   - vsock control plane                                                   |
|   - NAT/bridged networking attachment                                     |
|                                                                           |
|  Policy mount builder (new)                                                |
|   - creates per-world staging trees                                       |
|   - enforces read/write/discover by construction                          |
|                                                                           |
+----------------------------------|----------------------------------------+
                                   | virtio (block/fs/vsock/net)
+----------------------------------|----------------------------------------+
|                                Guest VM                                   |
|                                                                           |
|  Guest OS: Linux OR macOS                                                 |
|                                                                           |
|  World agent                                                              |
|   - command dispatch boundary                                             |
|   - enforces cmd allow/deny                                               |
|   - mounts shares at boot/login                                           |
|   - optional net proxy integration                                        |
|                                                                           |
|  Workdir(s)                                                               |
|   - guest root disk (ephemeral)                                           |
|   - mounted policy shares                                                 |
|                                                                           |
+---------------------------------------------------------------------------+
```
## Flow: create a world (VF-Linux or VF-macOS)
1. User requests world creation (`substrate world create...`).
2. Manager selects VF backend (Apple Silicon default; else per config).
3. `world_image_manager`:
   - creates an ephemeral copy-on-write disk from base image
   - creates unique world ID and storage path
4. `policy_mount_builder` compiles initial filesystem policy mounts (empty or default).
5. VM config built:
   - root disk attached
   - vsock device attached
   - virtiofs devices attached
   - network device attached (or not) per policy
6. VM boots; guest agent starts and registers with host via vsock.
## Flow: run a command in the world
1. User asks to run `cmd` (interactive shell or single command execution).
2. Manager sends request to world agent (over vsock).
3. World agent validates:
   - command allow/deny rules (policy)
   - optional resource constraints (timeouts, cpu/mem caps if supported)
4. World agent executes and streams:
   - stdout/stderr
   - exit code
   - structured command telemetry
5. Host returns result to user.
## Flow: filesystem policy updates (read/write/discover)
When policy changes (or a new command needs a new set of visible paths):
1. Manager compiles policy to a manifest of host paths with modes:
   - RW
   - RO
   - DISCOVER
2. `policy_mount_builder` produces per-world staging directories:
   - `share_rw/` (real content, writeable)
   - `share_ro/` (real content, read-only)
   - `share_discover/` (tree structure + placeholder files only)
3. VM sees mounts under stable mount points, e.g.:
   - `/mnt/substrate/rw`
   - `/mnt/substrate/ro`
   - `/mnt/substrate/discover`
> Note: mount commands differ by guest OS:
> - macOS guest uses `mount_virtiofs tag directory`
> - Linux guest uses `mount -t virtiofs tag directory`
## Flow: network policy updates
Baseline control:
- “Network disabled” = do not attach a NIC (guest has no interface)
- “Network enabled” = attach NAT NIC (default)
Stronger control (roadmap):
- Policy-driven proxy for common tools
- Host-level filtering via PF anchor + privileged helper (if feasible)
- Bridged networking only behind config flag + extra entitlement


## References

- Shared directories + mount commands summary (based on Apple docs, reproduced here): https://github-wiki-see.page/m/Code-Hex/vz/wiki/Shared-Directories
