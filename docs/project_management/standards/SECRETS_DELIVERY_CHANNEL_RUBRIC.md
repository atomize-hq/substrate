# Secrets Delivery Channel Rubric (FD/pipe vs env vars)

This standard defines a reusable decision rubric for how Substrate delivers **secret values** (tokens, API keys, auth material) to Substrate-managed components without storing secrets in Substrate config/policy/inventory files.

It exists to prevent ad-hoc env var proliferation and to keep secret-handling decisions consistent across LLM gateway/engines, agent hub/toolbox, and future workflow/router components.

---

## Terms

- **Secret value**: the actual credential material (token/key). Must never be persisted by Substrate.
- **Secret reference**: a pointer to where a secret value is sourced (e.g., an env var name like `OPENAI_API_KEY`).
- **Delivery channel**: how Substrate transmits secret values to a spawned/managed process:
  - inherited one-time **FD/pipe** channel, or
  - **environment variable injection**.

---

## Non-negotiable invariants

- Substrate MUST NOT store secret values in:
  - `$SUBSTRATE_HOME/config.yaml`, `$SUBSTRATE_HOME/policy.yaml`
  - `$SUBSTRATE_HOME/agents/*.yaml`
  - any workspace equivalents.
- Substrate MAY store secret references (e.g., env var names) when explicitly required by a backend contract and always behind strict schema validation.
- Missing required secrets MUST fail closed with actionable errors that name the missing reference(s) (e.g., the env var name), never their values.
- Secret values MUST NOT be printed to stdout/stderr by default and MUST be redacted/capped in:
  - trace spans/events
  - structured errors
  - session logs.

---

## Decision rubric (v1)

### Prefer FD/pipe (default when Substrate spawns both endpoints)

Use an inherited one-time FD/pipe secret channel when all are true:
- Substrate is spawning the consumer process (child process lifecycle is Substrate-owned), AND
- the consumer can be made to read a token/secret from an FD/pipe without relying on external conventions, AND
- the platform transport can pass the FD/handle in the execution scope required.

Rationale: minimizes accidental disclosure risk (no env var, no file artifact) and keeps the secret bound to the intended process.

Required guardrails:
- The FD/pipe MUST be read-once and closed promptly.
- The FD/pipe MUST NOT be forwarded to child processes unless explicitly required and documented.
- Debug/inspection escape hatches (if any) MUST be explicit and MUST NOT be required for normal operation.
- If an env var is used to tell a process which FD to read (e.g., `*_TOKEN_FD: int`), that env var MUST carry only the FD number (non-secret) and MUST be safe to print.

### Use env var injection (interop-required cases)

Use environment variable injection when any are true:
- The consumer is a third-party tool/SDK that expects secrets in env vars, OR
- the delivery must traverse a boundary/transport where FD passing is not available in v1, OR
- the secret is already operator-provisioned via env vars and the contract requires a simple, portable v1 mechanism.

Rationale: maximizes compatibility and reduces per-backend custom integration surface, at the cost of higher accidental exposure risk.

Required guardrails:
- Injected secret env vars MUST use Substrate-owned names when possible (so redaction rules are uniform).
- Any env var family designated secret-bearing MUST be treated as sensitive everywhere (redact/cap; never print by default).
- Document the OS-level caveat: environment variables may be readable to same-user processes (e.g., via `/proc/<pid>/environ` on Linux); threat model assumptions must be explicit in the owning spec.

---

## Canonical examples (current decisions)

- **Orchestration toolbox auth token**:
  - Delivery: FD/pipe (Substrate-spawned orchestrator sessions).
  - Source of truth: `docs/project_management/next/orchestration_mcp_toolbox/decision_register.md` (DR-0009).

- **LLM gateway/engine auth for `api:*` backends**:
  - Delivery: env var injection into in-world gateway/engine spawn environment (no persistence).
  - Source of truth: `docs/project_management/next/llm_gateway_in_world/decision_register.md` (DR-0007) and `docs/project_management/next/llm_gateway_in_world/specs/env_injection.md`.

---

## Documentation requirement

Any ADR/decision register introducing a new secret delivery mechanism MUST:
- state which channel is used (FD/pipe vs env vars),
- justify it using this rubric,
- enumerate the redaction/caps rules and “missing secret” failure behavior,
- and link to this document as the shared rationale.
