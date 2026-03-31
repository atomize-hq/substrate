# Review Surfaces - World Disabled Diagnostics

These diagrams orient the pack. They show the actual product/work shape that is expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.
Active and next seams still require seam-local `review.md` artifacts later before they can become `exec-ready`.

## R1 - High-level workflow

```mermaid
flowchart LR
  U[Operator] --> CMD[Run `substrate shim doctor` or `substrate health`]
  CMD --> CFG[Resolve effective config\n`world.enabled`]
  CFG --> ERR{Config resolution error?}
  ERR -->|yes| FAIL[stderr + exit 2\nno probes / no report]
  ERR -->|no| ENABLED{World enabled?}
  ENABLED -->|no| DISABLED[Disabled / skipped posture\nno world probes\nno world-deps probes]
  ENABLED -->|yes| PROBE[Backend probe + world-deps applied probe]
  DISABLED --> OUT[Text + JSON diagnostics output]
  PROBE --> OUT
```

## R2 - CLI / service / data flow

```mermaid
flowchart LR
  CLI[CLI entrypoint] --> RES[resolve_effective_config]
  RES --> SHIM[shim_doctor report builder]
  SHIM -->|enabled only| WB[world backend probe]
  SHIM -->|enabled only| WD[world deps applied probe]
  SHIM --> SHIMOUT[shim text + JSON]
  SHIMOUT --> HEALTH[health summary aggregation]
  HEALTH --> HOUT[health text + JSON]
  HOUT --> DOCS[operator docs / smoke expectations]
```

## R3 - Touch surface map

```mermaid
flowchart TB
  H[`crates/shell/src/builtins/health.rs`] --> CFG[`crates/shell/src/execution/config_model.rs`]
  SDR[`crates/shell/src/builtins/shim_doctor/report.rs`] --> CFG
  SDO[`crates/shell/src/builtins/shim_doctor/output.rs`] --> TXT[disabled/skipped copy]
  SDR --> JSON[status enums + omission rules]
  H --> SUM[summary.world_ok / failures / guidance suppression]
  SUM --> DOC[`docs/USAGE.md`]
  JSON --> T1[`crates/shell/tests/shim_doctor.rs`]
  SUM --> T2[`crates/shell/tests/shim_health.rs`]
  T1 --> SMOKE[manual playbook + smoke scripts]
  T2 --> SMOKE
```
