# World Deps (Internals)

This document captures implementation details and “gotchas” for the `substrate world deps` system.

Operator-facing docs live under:
- `docs/reference/world/deps/README.md`

## High-level flow

The shell:
1) Resolves the effective inventory view (built-ins + global inventory + workspace inventory chain).
2) Resolves the effective enabled list (global + workspace patches).
3) Computes the in-scope package set and normalized APT requirement set.
4) If APT or pacman requirements exist, probes them read-only with `dpkg-query` and `pacman -Q`
   inside the world.
5) If any system-package requirement is unsatisfied, exits `4` with remediation that points
   operators to `substrate world enable --provision-deps`.
6) If system-package requirements are already satisfied, treats system-package items as no-op and
   proceeds with:
   - `script` packages under the Substrate-managed prefix
   - `manual` packages as blocked/manual-only items

Code pointers:
- Shell engine: `crates/shell/src/builtins/world_deps/surfaces.rs`
- Inventory parsing + merge: `crates/shell/src/builtins/world_deps/inventory.rs`

## Why `$HOME` installs are ephemeral in hardened worlds

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
- Mount-namespace setup (workspace/full isolation): `crates/world/src/exec.rs`

Implications:
- The base root filesystem may be effectively read-only for world executions, but Substrate will still ensure `/root` is
  writable (often via a tmpfs mount) so common tooling (npm/pip/git, etc.) can run.
- `/root` should be treated as **scratch**:
  - In non-PTY runs (`substrate -c ...`), each command may execute in a fresh mount namespace, so `$HOME` state can be
    lost between invocations.
  - In the interactive REPL, `$HOME` state may persist only for the lifetime of that REPL session.

Practical guidance:
- Prefer project-local installs when possible:
  - Example: `npm i <pkg>` then run `npx <tool>` / `npm exec <tool>` (or `./node_modules/.bin/<tool>`).
- For “world-global” CLIs that must resolve on the deterministic world PATH, create a world-deps `package`/`bundle` that
  installs into `/var/lib/substrate/world-deps/…` and exposes an entrypoint under `/var/lib/substrate/world-deps/bin`.

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

## System-package runtime fail-early

Runtime world-deps application is now probe-only for APT-backed and pacman-backed items:
- `substrate world deps current sync`
- `substrate world deps current install <item...>`

These commands never invoke `apt`, `apt-get`, mutating `dpkg`, or `pacman` at runtime. They only
run read-only `dpkg-query` and `pacman -Q` probes inside the selected world, then:
- fail early with exit `4` and remediation if required packages are missing, or
- continue with non-system-package work if the packages are already present.

System-package provisioning is owned by:
- `substrate world enable --provision-deps`

On guest Linux agents, the `world-deps-provision` request profile is executed through a transient
`systemd-run` unit. This keeps the long-lived `substrate-world-agent.service` sandbox hardened for
normal runtime execution while still allowing explicit provisioning-time OS mutation (`apt`,
package postinst scripts, privilege drops to `_apt`) to touch the guest system paths it needs.
Those internal world-deps profiles are reserved for Substrate’s built-in world-deps flows; generic
`SUBSTRATE_WORLD_REQUEST_PROFILE` overrides do not select them.

Contract source:
- `docs/reference/world/deps/README.md`
- `docs/project_management/packs/implemented/world-deps-apt-provisioning/contract.md`
- Historical draft-pack path:
  `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`

Implementation lives in:
- runtime preflight/probe: `crates/shell/src/builtins/world_deps/surfaces.rs`
- provisioning-time system-package runner: `crates/shell/src/builtins/world_enable/runner/provision_deps.rs`

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
