# Review Surfaces - Azure Kimi Claude Gateway

These diagrams orient the pack. They show the actual product/work shape that is expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.
Active and next seams still require seam-local `review.md` artifacts later.

## R1 - Runtime request and tool loop

```mermaid
flowchart LR
  CC["Claude Code"] --> AM["Anthropic Messages gateway surface"]
  AM --> SS["Gateway session state"]
  SS --> ORCH["Internal orchestration policy"]
  ORCH --> NORM["Normalized event core"]
  NORM --> AZ["Azure Kimi provider adapter"]
  AZ --> THINK["Kimi-K2-Thinking"]
  AZ --> EXEC["Kimi-K2.5"]
  THINK --> AZ
  EXEC --> AZ
  AZ --> NORM
  NORM --> TOOL["Tool action / final event stream"]
  TOOL --> AM
  AM --> CC
```

## R2 - Landed gateway component map

```mermaid
flowchart TB
  EXT["External clients"] --> API["Anthropic-compatible transport layer"]
  API --> CORE["Client-agnostic gateway core"]
  CORE --> POLICY["Planner/executor policy"]
  CORE --> EVENTS["Normalized structured events"]
  CORE --> PROVIDER["Azure Kimi normalization adapter"]
  PROVIDER --> RAW["Azure chat-completions responses"]
  EVENTS --> OBS["Observability / persistence hooks"]
  EVENTS --> DOWN["Future downstream integrations"]
```

## R3 - External boundary and future Substrate posture

```mermaid
flowchart LR
  SUB["Future Substrate consumer"] --> ID["One logical backend identity"]
  DEV["Host-local dev transport"] --> ID
  ID --> GW["Gateway service boundary"]
  GW --> AUTH["Replaceable auth / secret delivery layer"]
  GW --> EVT["Normalized structured event boundary"]
  AUTH --> AZURE["Azure Foundry Kimi upstream"]
  EVT --> HUB["Future shell / agent-hub consumers"]
```

## Review intent

- `R1` makes the landed runtime behavior explicit: Anthropic ingress, normalized core, internal orchestration, and Azure provider interaction
- `R2` highlights the component boundaries that should exist even if the implementation reuses large parts of `claude-code-mux`
- `R3` makes the non-negotiable external posture visible: one logical backend identity, replaceable deployment/auth boundary, and structured events instead of raw provider streams
