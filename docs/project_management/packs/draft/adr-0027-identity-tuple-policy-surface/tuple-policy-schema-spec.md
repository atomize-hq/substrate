# adr-0027-identity-tuple-policy-surface — tuple-policy schema spec

Authoritative inputs:
- ADR: `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- Semantic owner: `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- Existing schema root: `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- Policy patch parser/effective-policy validators:
  - `crates/shell/src/execution/policy_model.rs`
  - `crates/broker/src/effective_policy.rs`
  - `crates/broker/src/policy.rs`

## Schema ownership

This document owns only the additive tuple-axis schema under `llm.constraints.*`.

It does not own:
- backend id grammar
- `llm.allowed_backends`
- `agents.allowed_backends`
- `client` as a policy key
- trace field inventories

## Merge and default rules

Per-key merge posture for policy patches:
- workspace patch replaces the same global key
- omitted key inherits the next lower layer
- each of the four tuple-axis keys resolves independently

Effective default for every tuple-axis key:
- `[]`

Effective meaning of `[]`:
- unconstrained on that axis

This feature does not define list-merge across layers.

## Canonical keys

| Key path | Type | Effective default | Valid token grammar | Meaning when non-empty |
| --- | --- | --- | --- | --- |
| `llm.constraints.routers` | `[string]` | `[]` | lowercase snake_case id | effective `router` must match one listed value |
| `llm.constraints.providers` | `[string]` | `[]` | lowercase snake_case id | effective `provider` must match one listed value |
| `llm.constraints.protocols` | `[string]` | `[]` | lowercase dotted id | effective `protocol` must match one listed value |
| `llm.constraints.auth_authorities` | `[string]` | `[]` | lowercase snake_case id | effective `auth_authority` must match one listed value |

## Exact grammar

### Snake-case ids

Applies to:
- `llm.constraints.routers[*]`
- `llm.constraints.providers[*]`
- `llm.constraints.auth_authorities[*]`

Accepted grammar:
- first character is `a-z`
- remaining characters are `a-z`, `0-9`, or `_`
- `_` does not appear twice in a row
- the value does not end with `_`

Accepted examples:
- `substrate_gateway`
- `openai`
- `azure_openai`
- `codex_subscription`
- `openai_api_key`

Rejected examples:
- `Substrate_Gateway`
- `openai-responses`
- `_openai`
- `openai__api`
- `openai_`

Validation error family:
- `invalid <key> entry '<value>'; expected lowercase snake_case id`

### Dotted ids

Applies to:
- `llm.constraints.protocols[*]`

Accepted grammar:
- one or more snake_case segments separated by `.`
- at least one `.` is required
- every segment must satisfy the snake_case grammar above

Accepted examples:
- `openai.responses`
- `openai.chat_completions`
- `anthropic.messages`
- `uaa.agent_session`

Rejected examples:
- `openai`
- `OpenAI.responses`
- `openai..responses`
- `openai.responses_v1.`
- `openai.responses-v1`

Validation error family:
- `invalid llm.constraints.protocols entry '<value>'; expected lowercase dotted id`

## Shape rules

Accepted YAML shape:

```yaml
llm:
  constraints:
    routers:
      - substrate_gateway
    providers:
      - openai
      - anthropic
    protocols:
      - openai.responses
    auth_authorities:
      - codex_subscription
      - openai_api_key
```

Accepted minimal unconstrained shape:

```yaml
llm:
  constraints:
    routers: []
    providers: []
    protocols: []
    auth_authorities: []
```

Accepted omission posture:
- `llm.constraints` may be omitted
- any individual child key may be omitted
- omission resolves to the inherited value, then to `[]`

Rejected shapes:
- scalar instead of list
- mapping instead of list
- unknown child keys under `llm.constraints`
- invalid token grammar in any list entry

Representative rejected YAML:

```yaml
llm:
  constraints:
    routers: substrate_gateway
    providers:
      primary: openai
    protocols:
      - openai
    auth_authorities:
      - OpenAI_API_Key
```

## Constraint semantics

Axis behavior:
- empty list: no restriction on that axis
- non-empty list: exact-match allowlist on that axis

This schema layer does not widen any existing backend gate.
It narrows the already-selected effective gateway identity tuple.

## Reserved omission: `client`

`client` is not a standalone policy key in v1.

The following key family is out of scope and invalid for this feature:
- `llm.constraints.clients`

Tuple semantics still publish `client` through the tuple model owned by ADR-0042. This schema does not add a policy control for it.

## Illustrative effective-resolution examples

Global policy:

```yaml
llm:
  constraints:
    providers:
      - openai
      - anthropic
```

Workspace policy:

```yaml
llm:
  constraints:
    providers:
      - openai
```

Effective merged result:

```yaml
llm:
  constraints:
    providers:
      - openai
```

Reason:
- workspace replaces the entire `llm.constraints.providers` list

Cross-key independence example:
- a workspace patch that sets only `llm.constraints.protocols` does not overwrite `llm.constraints.providers`

## Error posture

Schema-invalid tuple-policy input is a hard error:
- parse/set/show workflows reject the invalid payload
- effective policy resolution does not silently normalize the value
- the command exits with code `2`

This schema introduces no degraded or best-effort mode.
