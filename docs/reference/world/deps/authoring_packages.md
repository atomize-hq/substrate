# Authoring World-Deps Packages (Operator Guide)

This guide focuses on authoring *operator-defined* packages under `$SUBSTRATE_HOME/deps/` or
`<workspace_root>/.substrate/deps/`.

For copy/pasteable examples, see: `docs/reference/world/deps/examples/README.md`.

## Package file layout

Packages live under an inventory directory:

```text
$SUBSTRATE_HOME/deps/
  packages/
    <name>.yaml
  scripts/
    <anything>.sh
```

Rules:
- Filename must match `name` (e.g. `packages/nvm.yaml` must contain `name: nvm`).
- `install.script_path` relative paths are resolved relative to the package YAML file.

## Writable-path contract (script installs)

In hardened worlds, script installs MUST restrict all writes to:
- `/var/lib/substrate/world-deps` (install prefix)
- `/tmp` (scratch)

Do not write to `$HOME`, `/usr`, `/etc`, `/var/lib`, etc.

## Recommended prefix conventions

Use a stable root per package:
- Root: `/var/lib/substrate/world-deps/<package>`
- Binaries / entrypoints: `/var/lib/substrate/world-deps/bin`

When an upstream tool expects “its home dir” under `$HOME`, set its tool-specific environment variable(s) to the prefix.

Example pattern:
```sh
TOOL_HOME="/var/lib/substrate/world-deps/tool"
export TOOL_HOME
mkdir -p "$TOOL_HOME"
```

## Runnable packages and entrypoints

If `runnable: true`, you must provide `entrypoints: [...]`.

**Important:** for runnable behavior to be deterministic, the world expects entrypoints to resolve via the world-deps bin
prefix (`/var/lib/substrate/world-deps/bin`).

Ways to satisfy this:
- Install script creates symlinks/wrappers under `/var/lib/substrate/world-deps/bin`, and/or
- Use `wrappers:` to have Substrate generate wrapper scripts under `/var/lib/substrate/world-deps/bin`.

## Wrapper gotchas (read this)

Wrapper fields like `wrappers[].kind.bash_function.bash_source` are not a shell; they are treated as literal strings.

That means:
- Do **not** use `${VARS}` in wrapper fields.
- Prefer absolute paths inside the world, e.g. `/var/lib/substrate/world-deps/nvm/nvm.sh`.

## Script authoring guidelines

Your script should be:
- Idempotent (safe to re-run).
- Explicit about dependencies (e.g. require `bash`, `curl` or `wget` if you need them).
- Careful to not edit shell RC/profile files.

If you call third-party install scripts, set `PROFILE=/dev/null` (or equivalent) to prevent profile mutation when the
installer supports it.

## Schema sketch (version 1)

Minimal script-installed runnable package:

```yaml
version: 1
name: hello
description: Example user-space CLI.
runnable: true
entrypoints: ["hello"]
install:
  method: script
  script_path: ../scripts/hello.sh
probe:
  command: "hello --version"
```

See `docs/reference/world/deps/examples/` for complete working examples (YAML + scripts).

