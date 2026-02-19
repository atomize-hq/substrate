# World Deps (Internals)

This document captures implementation details and “gotchas” for the `substrate world deps` system.

Operator-facing docs live under:
- `docs/reference/world/deps/README.md`

## High-level flow

The shell:
1) Resolves the effective inventory view (built-ins + global inventory + workspace inventory chain).
2) Resolves the effective enabled list (global + workspace patches).
3) Computes an install plan:
   - `apt` packages first (world image / OS mutation)
   - `script` packages second (Substrate-managed prefix)
   - `manual` packages never auto-executed (blocked)
4) Executes world-backed steps by calling world-agent (`/v1/execute`).

Code pointers:
- Shell engine: `crates/shell/src/builtins/world_deps/surfaces.rs`
- Inventory parsing + merge: `crates/shell/src/builtins/world_deps/inventory.rs`

## Why $HOME installs break in hardened worlds

Substrate’s hardened world-agent service runs with a restrictive systemd sandbox on guest Linux (Lima/WSL), including:
- `ProtectSystem=strict` (rootfs is effectively read-only)
- `ReadWritePaths=... /var/lib/substrate ... /tmp` (Substrate-owned writable surfaces)

On macOS (Lima), see:
- `scripts/mac/substrate-world-agent.service`

Additionally, Substrate’s world env contract sets:
- `HOME=/root`
- XDG dirs under `/root`

Code pointers:
- Env contract injection: `crates/world/src/guard.rs` (`wrap_with_world_env_contract`)

Implications:
- Any installer that tries to write to `$HOME` (e.g. `/root/.nvm`) will fail under hardening.
- Package scripts should install to `/var/lib/substrate/world-deps/<tool>` instead.

## Writable surfaces (effective behavior)

In the macOS Lima guest, you will typically observe:
- The VM root filesystem mounted read-write when you SSH into the VM.
- The filesystem seen by **Substrate world executions** (`substrate --world -c ...`) behaves like:
  - `/` is read-only
  - `/var/lib/substrate` is writable

This difference is expected: it comes from the world-agent service sandbox.

## Wrapper fields are literal strings (no ${VAR} expansion)

World-deps wrapper definitions are parsed as YAML and rendered into wrapper scripts.

Fields like:
- `wrappers[].bash_source`
- `wrappers[].function`
- `wrappers[].kind.*.exec`

are treated as literal strings, not a shell.

So this is a footgun:
```yaml
bash_source: "${NVM_DIR:-$HOME/.nvm}/nvm.sh"
```

It will not be expanded; the wrapper will try to `source` that *literal* path string.

Prefer absolute, in-world paths:
```yaml
bash_source: "/var/lib/substrate/world-deps/nvm/nvm.sh"
```

## Wrapper kinds (how they execute)

Wrapper scripts are emitted under the world-deps bin prefix (default: `/var/lib/substrate/world-deps/bin`).

Today’s wrapper kinds are:
- `bash_function`
  - Runs `bash -lc 'source <bash_source>; <function> "$@"'`.
  - Intended for tools like `nvm` that are bash functions rather than standalone binaries.
- `bash_source_exec`
  - Runs `bash -lc 'source <bash_source>; exec <exec> "$@"'`.
  - Intended for “activate then exec” patterns.
- `sh_env_exec`
  - Exports a fixed env map then runs `exec <exec> "$@"` under `sh`.
  - Intended for wrappers that only require environment setup (no bash sourcing).

Implementation lives in:
- Wrapper rendering: `crates/shell/src/builtins/world_deps/surfaces.rs` (`render_wrapper_file_v1`)

## Script install execution model

Script package installs run inside the world and:
- Always set `SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR` (used as the wrapper output dir).
- `mkdir -p` the bin dir.
- Execute the installer script under:
  - `bash -lc "<script>"` when `bash` exists, otherwise
  - `sh -c "<script>"`

This is designed to make installer behavior deterministic and avoid relying on user shell rcfiles.

Implementation lives in:
- Script install command builder: `crates/shell/src/builtins/world_deps/surfaces.rs`
  (`build_world_script_install_command_v1`)

## Wrapper reconciliation (sync behavior)

`substrate world deps current sync` additionally performs wrapper reconciliation:
- Computes the “keep set” of expected wrapper/entrypoint filenames for the effective enabled packages.
- Removes stale files in `$SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR` that are not in the keep set.
- Fails with exit `5` if a would-be wrapper path collides with a directory or symlink (fail-closed, do not mutate).

This keeps `PATH` resolution deterministic in host-visible worlds.

Implementation lives in:
- Keep set calculation: `crates/shell/src/builtins/world_deps/surfaces.rs`
  (`collect_world_deps_bin_keep_names_v1`)
- Reconcile command: `crates/shell/src/builtins/world_deps/surfaces.rs`
  (`build_world_deps_bin_reconcile_command_v1`)

## APT installs vs hardening

Current `apt` support uses a Substrate-managed APT state dir under:
- `/var/lib/substrate/world-deps/apt`

But `apt-get install` still invokes `dpkg`, which writes to system locations such as:
- `/var/lib/dpkg`
- `/var/log/apt`
- `/usr`
- `/etc/apt`

Under `ProtectSystem=strict`, these writes fail with “Read-only file system”.

Code pointers:
- APT command builder: `crates/shell/src/builtins/world_deps/surfaces.rs` (`build_world_apt_install_command_v1`)
- Hardening classification for errors: `crates/shell/src/builtins/world_deps/surfaces.rs` (`looks_like_*hardening_violation*`)

Planned/expected resolution directions (design space):
- Provisioning-time system packages (run outside the hardened runtime sandbox), or
- Prefer user-space installs for runnable toolchains in hardened worlds.

## Debugging checklist

From host:
- Inspect the effective inventory and enabled list:
  - `substrate world deps current list available`
  - `substrate world deps current list enabled`
- Inspect per-item details:
  - `substrate world deps current show <name> --explain`

From world execution:
- Check what is writable:
  - `substrate --world -c 'touch /var/lib/substrate/world-deps/_probe && echo ok'`
  - `substrate --world -c 'touch /root/_probe'` (expected to fail in hardened worlds)

If you need to validate the sandbox characteristics inside the guest:
- `substrate --world -c 'mount | head'` (you will often see `/` as `ro` for this execution path)
