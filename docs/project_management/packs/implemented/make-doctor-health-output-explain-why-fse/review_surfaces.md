# Review Surfaces - make-doctor-health-output-explain-why

These diagrams orient the pack. They show the actual product/work shape that is expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.

Active and next seams still require seam-local `review.md` later before they can become `exec-ready` downstream.

## R1 - High-level workflow

```mermaid
flowchart LR
  O[Operator / automation] --> C[Run host doctor / world doctor / health]
  C --> R[Resolve effective world.enabled + provenance]
  R --> E{world.enabled?}
  E -->|true| N[Normal output; no disable-attribution fields]
  E -->|false| M[Map effective winner to disable reason]
  M --> T[Emit exact text attribution line]
  M --> J[Emit additive JSON reason/source fields when --json]
  T --> U[Operator changes the correct layer]
  J --> X[Tooling consumes stable structured attribution]
```

## R2 - API / service / data flow

```mermaid
flowchart LR
  CLI[CLI flags --world / --no-world] --> RES[resolve_effective_config_with_explain]
  WS[<workspace>/.substrate/workspace.yaml] --> RES
  ENV[SUBSTRATE_OVERRIDE_WORLD] --> RES
  GLB[$SUBSTRATE_HOME/config.yaml] --> RES
  DEF[Default config] --> RES
  RES --> ATTR[shared world_disable_attribution helper]
  ATTR --> DOC[platform doctor text / JSON renderers]
  ATTR --> HEALTH[health + shim_doctor reporting]
  DOC --> OUT1[doctor text + doctor JSON]
  HEALTH --> OUT2[health text + health JSON]
```

## R3 - Touch surface map

```mermaid
flowchart TB
  HD[substrate host doctor] --> P[execution/platform/mod.rs + platform/*]
  WD[substrate world doctor] --> P
  H[substrate health] --> HSRV[builtins/health.rs + shim_doctor/report.rs]
  P --> HELPER[world_disable_attribution.rs]
  HSRV --> HELPER
  HELPER --> CONTRACT[contract + decision register]
  HELPER --> SCHEMA[schema spec]
  CONTRACT --> VERIFY[manual playbook + smoke evidence]
  SCHEMA --> VERIFY
```
