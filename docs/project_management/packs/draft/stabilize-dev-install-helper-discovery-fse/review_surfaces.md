# Review Surfaces - stabilize-dev-install-helper-discovery

These diagrams orient the pack. They show the actual product/work shape that is expected to land.

They do not, by themselves, satisfy seam-local pre-exec review.

Active and next seams still require seam-local `review.md` artifacts later.

## R1 - Install → enable-later → uninstall lifecycle

```mermaid
flowchart LR
  OP[Operator] --> DI[dev-install-substrate.sh --prefix <home>]
  DI --> HOME[$SUBSTRATE_HOME]
  HOME --> HELPER[scripts/substrate/world-enable.sh]
  HOME --> INSTALL[scripts/substrate/install-substrate.sh]
  HOME --> DEPS[world-deps.yaml + macOS Lima files + bin/linux/*]
  OP --> WE[substrate world enable]
  WE --> ENV{SUBSTRATE_WORLD_ENABLE_SCRIPT set?}
  ENV -->|yes| OVERRIDE[Use override helper]
  ENV -->|no| PREFIX{Prefix helper exists?}
  PREFIX -->|yes| HELPER
  PREFIX -->|no| VERSION{Version-dir helper exists?}
  VERSION -->|yes| VHELPER[<inferred version dir>/scripts/substrate/world-enable.sh]
  VERSION -->|no| FAIL[Exit 4 fail-closed]
  OP --> DU[dev-uninstall-substrate.sh --prefix <home>]
  DU --> MANAGED{Repo-managed symlink or manifest-tracked copy?}
  MANAGED -->|yes| REMOVE[Remove staged asset]
  MANAGED -->|no| PRESERVE[Preserve path + exit 5]
```

Why this matters:

- The durable helper bundle and the managed cleanup rule are two halves of the same fixed-path contract.
- The operator-visible failure classes are part of the landed product shape, not just implementation details.

## R2 - Runtime resolution and release-root coupling

```mermaid
flowchart TB
  CLI[substrate world enable] --> RUNNER[paths.rs helper discovery]
  RUNNER --> OVR[env override candidate]
  RUNNER --> PFX[$SUBSTRATE_HOME/scripts/substrate/world-enable.sh]
  RUNNER --> VDIR[<inferred version dir>/scripts/substrate/world-enable.sh]
  PFX --> SCRIPT[world-enable.sh]
  VDIR --> SCRIPT
  SCRIPT --> ROOT[derived RELEASE_ROOT]
  ROOT --> MAC[scripts/mac/lima-*]
  ROOT --> DEPS[world-deps.yaml]
  ROOT --> LINUX[bin/linux/*]
```

Orientation notes:

- The helper is discovered from `$SUBSTRATE_HOME`, but the script still derives its release-root behavior from where it runs.
- On macOS, helper discovery can be correct even when full provisioning would still need additional release-root assets; the pack explicitly keeps that broader parity out of scope.

## R3 - Landed touch surface and evidence map

```mermaid
flowchart LR
  INSTALL_SCRIPT[scripts/substrate/dev-install-substrate.sh] --> BUNDLE[$SUBSTRATE_HOME bundle tree]
  UNINSTALL_SCRIPT[scripts/substrate/dev-uninstall-substrate.sh] --> BUNDLE
  PATHS_RS[crates/shell/.../paths.rs] --> RESOLVE[helper lookup order]
  TESTS_RS[crates/shell/tests/world_enable.rs] --> RESOLVE
  BUNDLE --> LSMOKE[smoke/linux-smoke.sh]
  BUNDLE --> MSMOKE[smoke/macos-smoke.sh]
  RESOLVE --> LSMOKE
  RESOLVE --> MSMOKE
  RESOLVE --> WSMOKE[smoke/windows-smoke.ps1]
  LSMOKE --> EVIDENCE[manual playbook + checkpoint evidence]
  MSMOKE --> EVIDENCE
  WSMOKE --> EVIDENCE
```

Orientation notes:

- The landed code touch set stays narrow even though the pack carries rich validation scaffolding.
- `SEAM-3` must eventually compare these evidence surfaces against upstream closeouts, not against provisional assumptions alone.
