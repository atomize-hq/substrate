# Review Surfaces - Azure Foundry Provider Transport

These diagrams orient the pack. They show the actual product/work shape that is expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.
Active and next seams still require seam-local `review.md` artifacts later.

## R1 - End-to-end Claude Code to Azure runtime flow

```mermaid
flowchart LR
  CC["Claude Code"] --> MSG["Gateway /v1/messages surface"]
  MSG --> ROUTER["Router policy
  think -> Kimi-K2-Thinking
  default -> Kimi-K2.5"]
  ROUTER --> REG["Provider registry + model mapping"]
  REG --> AZ["Azure Foundry transport resolver"]
  AZ --> THINK["Azure deployment
  Kimi-K2-Thinking"]
  AZ --> EXEC["Azure deployment
  Kimi-K2.5"]
  THINK --> AZ
  EXEC --> AZ
  AZ --> NORM["Landed normalized core"]
  NORM --> MSG
```

## R2 - Azure request construction surface

```mermaid
flowchart TB
  CFG["Provider config + model mappings"] --> AUTH["Azure auth/header resolver"]
  CFG --> TARGET["Deployment URL builder"]
  CFG --> VER["api-version injector"]
  AUTH --> REQ["Azure chat-completions request"]
  TARGET --> REQ
  VER --> REQ
  BODY["Landed normalized/public contracts"] --> REQ
  REQ --> HTTP["reqwest transport"]
  HTTP --> RESP["Azure runtime response"]
  RESP --> NORM["C-02 normalized event path"]
```

## R3 - Live smoke and troubleshooting loop

```mermaid
flowchart LR
  OP["Operator with Azure credentials"] --> SETUP["Gateway config and secret setup"]
  SETUP --> START["Start gateway"]
  START --> SMOKE1["Smoke request through /v1/messages
  think route"]
  START --> SMOKE2["Smoke request through /v1/messages
  default route"]
  SMOKE1 --> EVID["Redacted evidence + route confirmation"]
  SMOKE2 --> EVID
  EVID --> PASS["Operator-ready success path"]
  EVID --> FAIL["Troubleshooting matrix"]
  FAIL --> AUTH["Auth mismatch"]
  FAIL --> URL["Deployment URL mismatch"]
  FAIL --> API["api-version mismatch"]
  FAIL --> MAP["Model or deployment mapping mismatch"]
```

## Review intent

- `R1` makes the real delivery target explicit: Claude Code must reach Azure-hosted Kimi through the landed Anthropic-compatible surface and existing internal routing policy
- `R2` highlights the exact seam-1 control points that remain unresolved in the current generic OpenAI path
- `R3` makes live operator proof first-class and forces the pack to name success signals and troubleshooting surfaces instead of assuming runtime verification will be obvious later
