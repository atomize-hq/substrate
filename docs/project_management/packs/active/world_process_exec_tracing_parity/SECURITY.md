# Security — World process exec/exit telemetry (ADR-0028) (Authoritative)

This document defines the security posture for ADR-0028: process tree exec/exit telemetry for in-world executions.

## 1) Safe-by-default invariants (non-negotiable)
- Canonical trace (`~/.substrate/trace.jsonl`) MUST NOT contain raw secrets introduced by this feature.
- Process telemetry MUST apply data minimization:
  - argv MUST be redacted.
  - env MUST be allowlist-only and redacted.
  - payload MUST be capped and truncation MUST be explicit.
- Preexec/builtin tracing MUST NOT write raw command bodies into canonical trace.

## 2) Redaction model

### 2.1 argv redaction (required)
For process telemetry (`world_process_*`):
- Redaction MUST handle:
  - `--flag value` (flag consumes next arg)
  - `--flag=value`
  - `-H 'Header: Bearer <token>'` style paired-arg secrets
- Redaction MUST be applied in-world before returning telemetry to the host.

### 2.2 env capture (allowlist-only)
- World-agent MUST only include env keys explicitly allowlisted for telemetry.
- Any env key not in the allowlist MUST be omitted.
- Redaction MUST be applied to env values before they are returned to the host.

### 2.3 Debug-only raw logging posture (preexec)
- Canonical trace `event_type: builtin_command` records MUST omit the command body and MUST include `command_omitted: true`.
- If raw capture is needed for debugging, it MUST be opt-in via an explicit file path env var:
  - `SUBSTRATE_PREEXEC_RAW_LOG=<path>`
- Raw capture MUST be written to that file only (never to `trace.jsonl`) and each raw record MUST include:
  - `may_contain_secrets: true`

## 3) Degrade vs fail-closed rules

### 3.1 Execution correctness
- Command execution inside the world MUST proceed even if process telemetry capture is unavailable.

### 3.2 Observability correctness (no silent omission)
- If process telemetry is unavailable, world-service MUST return explicit diagnostics:
  - `process_events_status: "unavailable"`
  - `process_events_reason: "<stable reason code>"`

### 3.3 Truncation correctness
- When event caps trigger truncation, world-service MUST:
  - return `process_events_status: "truncated"`, and
  - include a deterministic dropped-count summary (`process_events_dropped`).

## 4) Non-triggerability and audit safety
- The canonical trace MUST remain suitable for downstream routing (ADR-0029) without enabling new triggerable secret exfiltration surfaces.
- High-volume debug telemetry (preexec raw command bodies) MUST remain opt-in and MUST be clearly marked as sensitive.

## 5) Backlog: redaction hardening prerequisite for “include bodies”
The optional future posture “include builtin command bodies in canonical trace” is prohibited until:
- codebase-wide redaction is hardened and validated for bash command bodies and common secret formats, and
- the new posture is explicitly selected in a dedicated ADR/Decision Register entry.

