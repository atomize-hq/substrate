# Review Surfaces - dev-install-world-agent-staging

These diagrams orient the pack. They show the actual product/work shape that is expected to land.

They do not, by themselves, satisfy seam-local pre-exec review.

Active and next seams still require seam-local `review.md` artifacts later.

## R1 - Linux enable-later workflow

```mermaid
flowchart LR
  DEV[Developer / operator] --> DI[dev-install-substrate.sh --no-world]
  DI --> CFG0[config.yaml world.enabled=false]
  DI --> BIN1[target/bin/world-agent]
  DI --> BIN2[target/bin/linux/world-agent]
  DEV --> WE[substrate world enable]
  WE --> OVR{SUBSTRATE_WORLD_ENABLE_SCRIPT set?}
  OVR -->|yes| OHELP[use override helper exactly]
  OVR -->|no| HOME[resolve home from --home -> SUBSTRATE_HOME -> ~/.substrate]
  HOME --> VDIR[derive standard version dir from <home>/bin/substrate]
  VDIR --> P1{bin/world-agent exists?}
  P1 -->|yes| RUN[launch helper / verify]
  P1 -->|no| P2{bin/linux/world-agent exists?}
  P2 -->|yes| RUN
  P2 -->|no| FAIL[exit 3 + remediation block]
  RUN --> OK[helper execution + health verification]
  OK --> CFG1[config.yaml world.enabled=true]
```

Why this matters:

- The landed workflow is coherent only when runtime preflight and dev-install staging agree on the same accepted path set.
- The operator-visible failure class is part of the product shape, not an implementation detail.

## R2 - Runtime path derivation and ordering controls

```mermaid
flowchart TB
  CLI[substrate world enable] --> RUNNER[runner.rs]
  RUNNER --> HOME[resolve state root]
  RUNNER --> OVERRIDE{override helper path?}
  OVERRIDE -->|yes| DIRECT[use override helper]
  OVERRIDE -->|no| SUBSTRATE[realpath <home>/bin/substrate]
  SUBSTRATE --> VERSION[standard version dir]
  VERSION --> ROOT[target/bin/world-agent]
  VERSION --> FALLBACK[target/bin/linux/world-agent]
  ROOT --> PREFLIGHT[missing-artifact preflight]
  FALLBACK --> PREFLIGHT
  PREFLIGHT -->|missing| STDERR[stderr remediation + exit 3]
  PREFLIGHT -->|present| HELPER[scripts/substrate/world-enable.sh]
  HELPER --> VERIFY[health verification]
  VERIFY --> WRITES[config/env writes only after success]
```

Orientation notes:

- The override path is an explicit carve-out and should not be mistaken for the standard version-dir guarantee.
- The no-write ordering and remediation visibility both belong to the runtime contract surface.

## R3 - Validation and checkpoint evidence flow

```mermaid
flowchart LR
  S1[SEAM-1 landed runtime contracts] --> PLAYBOOK[manual_testing_playbook.md]
  S2[SEAM-2 landed staging contracts] --> PLAYBOOK
  S1 --> LSMOKE[smoke/linux-smoke.sh]
  S2 --> LSMOKE
  S2 --> ISMOKE[tests/installers/install_smoke.sh]
  PLAYBOOK --> CP1[CP1-ci-checkpoint]
  LSMOKE --> CP1
  ISMOKE --> CP1
  PARITY[compile parity linux/macos/windows] --> CP1
  CP1 --> SESSION[session_log.md]
  CP1 --> CLOSEOUT[governance closeout + stale triggers]
```

Orientation notes:

- The checkpoint is a product-facing proof surface because it validates the entire enable-later workflow after both behavior seams land.
- Pack-level diagrams orient the evidence flow only; active and next seams still need seam-local review artifacts before execution.
