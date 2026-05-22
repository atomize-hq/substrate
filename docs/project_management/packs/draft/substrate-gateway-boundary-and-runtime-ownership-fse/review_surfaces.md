# Review Surfaces - substrate-gateway-boundary-and-runtime-ownership

These diagrams orient the pack. They show the actual operator workflow, status/policy boundary, typed runtime path, and documentation lock-in shape that are expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.
`SEAM-1` and `SEAM-2` still require seam-local `review.md` artifacts later.

## R1 - Operator workflow and ownership boundary

```mermaid
flowchart LR
  OP["Operator"] --> CMD["substrate world gateway sync | status | restart"]
  CMD --> SUB["Substrate-owned boundary<br/>policy / placement / lifecycle / secret delivery / operator UX / tracing"]
  SUB --> WA["world-service lifecycle + status surface"]
  WA --> GW["substrate-gateway inside the world"]
  GW --> P["provider / planner / executor internals"]
```

## R2 - Machine-readable status and wiring authority

```mermaid
flowchart LR
  CMD["substrate world gateway status --json"] --> WA["typed world-service status surface"]
  WA --> ST["Substrate-owned status object"]
  ST --> CW["client_wiring.*"]
  ST --> AV["gateway availability + absence semantics"]
  ST --> PP["policy posture"]
  CW --> ENV["SUBSTRATE_LLM_OPENAI_BASE_URL<br/>SUBSTRATE_LLM_ANTHROPIC_BASE_URL"]
  ST --> ADR42["ADR-0042 additive metadata boundary<br/>(outside client_wiring.*)"]
```

## R3 - Policy, placement, and secret-delivery flow

```mermaid
flowchart TB
  CFG["ADR-0027 config + policy inputs"] --> DEC{"Gateway allowed and in-world required?"}
  DEC -->|"No: invalid config / invalid integration"| EC2["Exit 2"]
  DEC -->|"No: policy or safety denial"| EC5["Exit 5"]
  DEC -->|"No: dependency unavailable"| EC4["Exit 4"]
  DEC -->|"Yes"| SD["Substrate-owned host-to-world secret delivery"]
  SD --> WA["typed world-service lifecycle orchestration"]
  WA --> GW["substrate-gateway in-world runtime"]
  GW --> RS["status / sync / restart result"]
```

## R4 - Cross-doc and quality-gate lock-in

```mermaid
flowchart LR
  C1["SEAM-1 contract"] --> PLAY["manual_testing_playbook.md"]
  C2["SEAM-2 schema + policy"] --> PLAY
  C3["SEAM-3 runtime + parity"] --> PLAY
  PLAY --> DOCS["docs/CONFIGURATION.md<br/>docs/USAGE.md<br/>docs/WORLD.md<br/>docs/TRACE.md"]
  PLAY --> TASKS["plan.md + tasks.json + quality_gate_report.md"]
  DOCS --> QG["one-owner-per-surface quality gate"]
  TASKS --> QG
```
