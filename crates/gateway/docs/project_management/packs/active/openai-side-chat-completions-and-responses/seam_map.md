# Seam Map - OpenAI-Side Chat Completions and Responses

This seam map extracts executable feature seams from ADR 0008’s OpenAI-side public-ingress contract. It does not mirror code module boundaries; the seams are organized around externally verifiable public behavior and the cross-cutting invariants that keep adapters thin.

Constraint posture:

- ADR 0008 is the normative contract and must be implemented as written (compatibility-first via `chat/completions`, modern capability-first via `responses`).
- `IMPORTANT_SUBSTRATE_ALIGNMENT.md` constrains boundary discipline: both OpenAI-facing endpoints must remain thin adapters over the same normalized core, and `/v1/messages` remains primary unless changed by a later ADR.
- The gateway supports function tools only for OpenAI-side public ingress; built-in tools remain out of scope until explicitly added by a later ADR.

## Horizon summary

- **Active seam**: none remaining in this pack
- **Next seam**: none remaining in this pack
- **Future seams**: none remaining in this pack

All seams in this pack have landed and are now maintained through their published closeouts and contracts.

## Seam roster

| Seam | Horizon / state | Type | Why this is a seam | Likely value | Touch surface | Verification path |
| --- | --- | --- | --- | --- | --- | --- |
| `SEAM-1` `openai-chat-completions-surface` | `landed` / `closed` | `capability` | a bounded compatibility surface that many SDKs target, with a clear tool-loop + streaming contract and strong adapter-boundary constraints | drop-in compatibility for OpenAI-shaped clients via `chat/completions` without adopting `/v1/messages` | `gateway/src/server/mod.rs`, `gateway/src/server/openai_compat.rs`, internal tool representation and streaming transforms | request/response golden tests (non-stream + stream), tool-loop fixtures, negative tests for rejected fields, and at least one SDK-level smoke (if available) |
| `SEAM-2` `openai-responses-surface` | `landed` / `closed` | `capability` | the modern OpenAI-shaped surface better aligned with the gateway’s normalized events, with its own streaming event set and tool-loop contract | first-class modern ingress for tools + streaming via `/v1/responses` while remaining thin over the same core | new `/v1/responses` route + adapter module(s), shared core transforms, existing provider `/v1/responses` support | response-object and SSE-event golden tests, tool-loop fixtures (`function_call_output`), and cross-checks against the shared adapter invariants |
| `SEAM-3` `openai-side-conformance-and-drift-guards` | `landed` / `closed` | `conformance` | locks the compatibility subset into regression coverage so future provider or core changes don’t silently break OpenAI SDK behavior | durable confidence: contract drift guards, negative-case enforcement, and documentation/test evidence that both endpoints remain thin adapters | `gateway/tests/`, docs updates tied to the contracted subset, any CI wiring needed for the conformance suite | deterministic tests for: error envelope, model echo, `X-Provider` forcing, tool-call mapping, SSE termination, and rejection/ignore posture |

## Ordering rationale

1. `SEAM-1` is `landed` because the compatibility surface, including request/response mapping and streaming, has already been published and closed out.
2. `SEAM-2` is `landed` because the Responses surface and its tool-loop and streaming contract have already been published and closed out.
3. `SEAM-3` is `landed` because the deterministic conformance and drift-guard suite has already been published and closed out.

## Non-seams and pruned candidates

- A “shared adapter refactor” seam was rejected as a standalone seam because it is not independently verifiable without landing a public behavior change; shared work remains owned by `SEAM-1`/`SEAM-2` and is validated through public contract tests.
- A “provider streaming” seam was rejected because provider normalization is explicitly out of scope for the public adapter seams; the public adapters must remain pure transforms over provider-normalized streams.
- A “built-in tools support” seam was rejected because ADR 0008 explicitly scopes OpenAI-side ingress to function tools only.
