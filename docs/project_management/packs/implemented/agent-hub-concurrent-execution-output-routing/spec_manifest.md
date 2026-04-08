# agent-hub-concurrent-execution-output-routing — spec manifest

This file enumerates every contract surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:

- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs

- Feature directory: `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- Alignment ADRs (delegated ownership; no pack takeover):
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
  - `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
  - `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
  - `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
  - `docs/project_management/adrs/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`

## Required spec documents (authoritative)

Spec templates:

- `docs/project_management/system/templates/spec/`

- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/spec_manifest.md` — spec selection + ownership map (this file)
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/impact_map.md` — touch set + cascading implications + cross-queue scan
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/plan.md` — execution runbook (high-level)
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/tasks.json` — triad task graph + acceptance criteria
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/contract.md` — operator-facing contract for output routing + config key + exit/error posture
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/agent-hub-event-envelope-schema-spec.md` — structured agent event envelope schema (authoritative)
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/telemetry-spec.md` — new/changed trace records for structured agent events + suppression warnings
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/platform-parity-spec.md` — platform guarantees + validation evidence requirements
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/decision_register.md` — A/B decisions and selections (already exists)
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/OR0-spec.md` — slice spec (event envelope + canonical trace persistence foundation)
- `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/OR1-spec.md` — slice spec (REPL output routing during PTY passthrough: buffer/drop + deterministic warnings)
- Validation artifacts (authoritative; required by ADR-0017):
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/manual_testing_playbook.md`
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/smoke/linux-smoke.sh`
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/smoke/macos-smoke.sh`
  - `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/smoke/windows-smoke.ps1`

## Coverage matrix (surface → authoritative doc)

| Surface                                                                  | Authoritative doc                                                                                                            | What is explicitly defined                                                                 |
| ------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| REPL output routing contract (PTY bytes vs structured events)            | `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/contract.md`                             | routing rules per mode (idle vs passthrough), non-injection invariants, drop behavior      |
| Structured agent event envelope                                          | `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/agent-hub-event-envelope-schema-spec.md` | required top-level fields, kind taxonomy, per-kind `data` schema, channel constraints      |
| Adapter-only `backend_id` + tuple-compatible optional metadata           | `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/agent-hub-event-envelope-schema-spec.md` | `backend_id` semantics, optional top-level tuple fields, and explicit delegation boundaries |
| Config key `repl.max_pty_buffered_lines`                                 | `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/contract.md`                             | default, precedence, bounds, invalid/out-of-range handling, warning emission rules         |
| Exit/error posture (config parse failures, warnings)                     | `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/contract.md`                             | exit code mapping, “warnings do not change exit code”, hard-error conditions               |
| Trace record types for structured agent events and suppression summaries | `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/telemetry-spec.md`                       | record `event_type`s, required correlation fields, stability/redaction rules               |
| Platform parity                                                          | `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/platform-parity-spec.md`                 | required behavior parity, permitted divergences, required validation evidence              |
| Manual validation                                                        | `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/manual_testing_playbook.md`              | deterministic manual cases and expected outcomes                                           |
| Automation smoke validation                                              | `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/smoke/*`                                 | smoke mirrors manual cases and asserts invariants (no PTY injection, no prompt corruption) |
| Slice acceptance (OR0)                                                   | `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/OR0-spec.md`                             | per-slice scope, acceptance criteria, regression protections                               |
| Slice acceptance (OR1)                                                   | `docs/project_management/packs/active/agent-hub-concurrent-execution-output-routing/OR1-spec.md`                             | per-slice scope, acceptance criteria, regression protections                               |

## Determinism checklist (must be satisfied before quality gate)

For the docs above, confirm they explicitly define:

- Inputs and precedence (PTY bytes vs structured events; config layering).
- Defaults and absence semantics (default cap; unset/missing fields; missing non-required correlation fields).
- Data model (schema) for every serialized boundary (event envelope, trace record payload).
- Error model and failure posture (hard-error vs clamp+warning; no PTY injection).
- Ordering/atomicity/concurrency rules (buffering, drop, flush timing).
- Security/redaction invariants (`channel` constraints; no secrets in structured events).
- Delegated ownership boundaries (`backend_id` stays adapter-only; tuple semantics stay in ADR-0042/0044/0045).
- Platform guarantees (Linux/macOS/Windows parity expectations; any permitted divergence is explicit).
