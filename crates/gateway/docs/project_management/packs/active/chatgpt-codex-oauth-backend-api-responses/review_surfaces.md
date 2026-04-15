# Review Surfaces - ChatGPT Codex OAuth Backend-API Responses

These diagrams orient the pack. They show the actual product/work shape that is expected to land.
They do not, by themselves, satisfy seam-local pre-exec review.
Active and next seams still require seam-local `review.md` artifacts later.

## R1 - End-to-end routed Codex OAuth turn

```mermaid
flowchart LR
  CALLER["Client via /v1/messages
  /v1/chat/completions
  or /v1/responses"] --> INGRESS["Public gateway ingress
  stays thin"]
  INGRESS --> CORE["Normalized GatewayRequest
  + stream model"]
  CORE --> ROUTE["Codex OAuth route selector
  provider-side only"]
  ROUTE --> UP["ChatGPT backend-api
  /codex/responses
  stream=true"]
  UP --> SSE["Semantic SSE event flow"]
  SSE --> SYNC["Sync drain assembly"]
  SSE --> STREAM["Streaming transform"]
  SYNC --> CALLER
  STREAM --> CALLER
```

## R2 - Route compatibility and stream assembly boundary

```mermaid
flowchart TB
  REQ["Normalized request controls"] --> MATRIX["Route-local matrix
  pass | translate | force | reject"]
  MATRIX --> SERIAL["Codex serializer
  typed message items
  flat tools
  flat tool_choice"]
  SERIAL --> HEADERS["Minimal header contract
  Authorization
  ChatGPT-Account-ID
  Content-Type"]
  HEADERS --> UP["backend-api/codex/responses"]
  UP --> EVENTS["response.created
  response.output_item.*
  response.output_text.*
  response.function_call_arguments.*
  response.completed"]
  EVENTS --> ASSEMBLY["Semantic output + tool assembly"]
  ASSEMBLY --> OUT["GatewayResponse
  or normalized stream"]
```

## R3 - Integrated auth-handoff owner line

```mermaid
flowchart LR
  HOST["Host-side Substrate
  credential preflight"] --> BUNDLE["Secret-channel auth bundle
  cli:codex fields"]
  BUNDLE --> WORLD["In-world gateway runtime"]
  WORLD --> AUTHCTX["Resolved auth context
  explicit account_id first"]
  AUTHCTX --> PROVIDER["Provider request builder"]
  PROVIDER --> HEADER["ChatGPT-Account-ID
  injection"]
  HEADER --> UP["backend-api/codex/responses"]
  LOCAL["Standalone local auth state
  bounded fallback only"] --> AUTHCTX
```

## Review intent

- `R1` makes the delivery target explicit: public ingress stays thin while the routed provider path becomes a dedicated ChatGPT Codex transport
- `R2` highlights the seam-1 review surface that must become canonical before implementation: compatibility classification, serializer shape, minimal headers, and semantic event assembly
- `R3` makes the seam-2 trust boundary explicit so integrated auth delivery and standalone fallback cannot blur into one implicit owner line
