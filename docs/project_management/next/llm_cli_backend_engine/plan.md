# plan — llm_cli_backend_engine

Goal: Make ADR-0024 execution-ready by defining the CLI backend engine contract (capability-gated) that fulfills canonical LLM requests via subscription-authenticated CLIs while remaining inside the world boundary.

Scope:
- Backend identity and registration via ADR-0027 agent inventory (`cli:<agent_id>` mapping)
- Session strategy (persistent vs per-request) and its implications
- Streaming behavior and fallback semantics when CLIs lack streaming
- Prompt/response translation boundaries (canonical request/response; adapter model)
- Policy gates: `llm.allowed_backends`, `llm.fail_closed.routing`, and `net_allowed`

Out of scope:
- Enumerating a canonical backend registry list (Phase 8 circle-back once ADR-0023/0024/0025 land)

Steps:
1. Lock engine session + streaming semantics decisions.
2. Define canonical request/response subset required for v1.
3. Define adapter interface and failure modes.
4. Define trace attribution requirements and error mapping.
5. Draft manual testing playbook + stub backend expectations (later; once code exists).

