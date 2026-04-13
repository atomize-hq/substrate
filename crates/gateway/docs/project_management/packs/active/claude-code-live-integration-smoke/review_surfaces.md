# Review Surfaces - Claude Code Live Integration Smoke

These diagrams orient the pack. They show the actual product/work shape that is expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.
Active and next seams still require seam-local `review.md` artifacts later.

## R1 - Operator bootstrap flow

```mermaid
flowchart LR
  AZ["Azure Foundry credentials
  and deployments"] --> CFG["Gateway config
  provider + models + router"]
  CFG --> START["Gateway startup
  and config validation"]
  START --> CCENV["Claude Code env
  ANTHROPIC_BASE_URL + API key placeholder"]
  CCENV --> CLAUDE["Claude Code session"]
  START --> EHOOKS["Statusline / tracing / logs
  enabled for evidence"]
  EHOOKS --> CLAUDE
```

## R2 - Live request and routing path

```mermaid
flowchart LR
  CC["Claude Code"] --> MSG["Gateway /v1/messages"]
  MSG --> ROUTER["Router policy
  default / think / tool-result handoff"]
  ROUTER --> REG["Provider registry
  model mapping"]
  REG --> THINK["Azure deployment
  Kimi-K2-Thinking"]
  REG --> EXEC["Azure deployment
  Kimi-K2.5"]
  THINK --> NORM["Landed normalized core"]
  EXEC --> NORM
  NORM --> MSG
  MSG --> CC
```

## R3 - Smoke coverage map

```mermaid
flowchart TB
  BOOT["Bootstrap complete"] --> S1["Normal execution turn"]
  BOOT --> S2["Think / planner turn"]
  BOOT --> S3["Tool loop follow-up turn"]
  S1 --> EV1["Default-route evidence"]
  S2 --> EV2["Think-route evidence"]
  S3 --> EV3["Continuation-route evidence"]
  EV1 --> PASS["Redacted smoke manifest"]
  EV2 --> PASS
  EV3 --> PASS
```

## R4 - Troubleshooting and ownership branches

```mermaid
flowchart LR
  FAIL["Smoke or session failure"] --> CCF["Claude Code integration
  env / client setup / invocation"]
  FAIL --> GWF["Gateway runtime or config
  startup / route / trace"]
  FAIL --> AZF["Azure provider transport
  auth / URL / deployment"]
  FAIL --> DRIFT["Cross-layer drift
  contract or evidence mismatch"]
  CCE["Client-visible evidence"] --> FAIL
  TRACE["Statusline / tracing / logs"] --> FAIL
  RESP["Gateway response or error class"] --> FAIL
```

## Review intent

- `R1` forces the pack to show the exact operator path from Azure prerequisites into Claude Code rather than stopping at gateway-only setup
- `R2` keeps the live path grounded in the landed `/v1/messages` surface and internal routing policy instead of inventing a new public entrypoint
- `R3` makes the required smoke surface explicit: one normal turn, one think turn, and one tool-loop continuation path with named evidence
- `R4` makes failure ownership first-class so later seam planning cannot blur Claude Code, gateway, and Azure responsibilities together
