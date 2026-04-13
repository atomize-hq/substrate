# Review Surfaces - OpenAI-Side Chat Completions and Responses

These diagrams orient the pack. They show the actual product/work shape that is expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.

## R1 - High-level client workflows (tool loop)

```mermaid
flowchart LR
  C[OpenAI-shaped Client / SDK] -->|"POST /v1/chat/completions or /v1/responses"| G[Gateway]
  G -->|"assistant output OR tool request"| C
  C -->|"execute function tools"| T[Tool Runtime]
  T -->|"tool output"| C
  C -->|"follow-up request with tool results"| G
  G -->|"final assistant output"| C
```

## R2 - Gateway flow (thin adapters over shared core)

```mermaid
flowchart LR
  CC["/v1/chat/completions\n(compat surface)"] --> AD1[Adapter Transform]
  RSP["/v1/responses\n(modern surface)"] --> AD2[Adapter Transform]
  AD1 --> CORE["GatewayRequest\n(normalized core)"]
  AD2 --> CORE
  CORE --> RT["Router + model mapping\n(+ X-Provider override)"]
  RT --> PR["Provider adapter\n(e.g. OpenAI-family upstream)"]
  PR --> CORE2["GatewayResponse / stream\n(normalized output)"]
  CORE2 --> AD1
  CORE2 --> AD2
```

## R3 - Touch surface map (likely code anchors)

```mermaid
flowchart TB
  SRV["gateway/src/server/mod.rs\n(routes + handlers)"] --> OA["gateway/src/server/openai_compat.rs\n(chat/completions adapter)"]
  SRV --> OR["gateway/src/server/*responses*\n(responses adapter)"]
  OA --> CORE["gateway/src/core.rs\nGatewayRequest/Response"]
  OR --> CORE
  CORE --> PROVIDER["gateway/src/providers/openai.rs\n(upstream OpenAI-family)"]
  SRV --> ERR["gateway/src/server/*\n(error envelope + failure class)"]
  TST["gateway/tests/*\n(conformance suite)"] --> SRV
```

