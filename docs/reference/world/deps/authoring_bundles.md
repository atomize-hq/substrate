# Authoring World-Deps Bundles (Operator Guide)

A bundle is a named list of packages:
- Bundles are never invoked directly.
- A bundle is `present` iff all of its packages are `present`.

## Bundle file layout

Bundles live under an inventory directory:

```text
$SUBSTRATE_HOME/deps/
  bundles/
    <name>.yaml
```

Rules:
- Filename must match `name` (e.g. `bundles/node-tooling.yaml` must contain `name: node-tooling`).
- `packages: [...]` must reference package names visible in the *effective* inventory view.

## Schema sketch (version 1)

```yaml
version: 1
name: node-tooling
description: Node manager + baseline tooling
packages: ["nvm"]
```

For examples, see: `docs/reference/world/deps/examples/README.md`.

