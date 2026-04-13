# Claude Code C-11 Operator Troubleshooting Guide

## Purpose

This guide operationalizes `C-11` for operators. It does not redefine the contract, and it does not create a second source of truth for ownership or evidence semantics.

Use this guide when a live Claude Code integration run needs owner classification, redacted escalation, or a reproducible support flow that starts from the published bootstrap and smoke evidence.

Canonical inputs:

- `docs/foundation/claude-code-c11-troubleshooting-and-support-boundary-contract.md`
- `docs/foundation/claude-code-c09-operator-bootstrap-contract.md`
- `docs/foundation/claude-code-c10-live-session-smoke-verification-contract.md`
- `docs/foundation/claude-code-c10-live-session-smoke-procedure.md`
- `gateway/README.md`
- `gateway/src/router/mod.rs`
- `gateway/src/server/mod.rs`

## Support Flow

1. Confirm the bootstrap path from `C-09` is in place.
2. Confirm the live-smoke branch from `C-10` is the one being exercised.
3. Inspect the required evidence in order.
4. Classify the failure by owner using the runtime `error.class` taxonomy.
5. Escalate with the minimum redacted evidence needed for that owner.

This flow stays downstream of `C-11`: it consumes the contract and the runtime evidence surfaces, but it does not restate or revise the contract itself.

## Evidence Order

Review evidence in this order:

1. Bootstrap evidence from `C-09`.
2. Live-smoke branch identity and outcome from `C-10`.
3. `~/.substrate-gateway/last_routing.json`.
4. Claude Code statusline output.
5. Optional `~/.substrate-gateway/trace.jsonl` only when the redacted evidence above is not enough to classify the failure.

Required evidence for support classification:

- bootstrap readiness from the published `C-09` path
- the exercised branch identity: normal execution, think / planner, or tool-loop continuation
- the redacted pass or fail outcome
- `~/.substrate-gateway/last_routing.json`
- statusline output from the installed Claude Code statusline script

Optional evidence:

- `~/.substrate-gateway/trace.jsonl`
- additional redacted request or response context when it helps classify the failure

## Owner Classification Matrix

Use this matrix to decide who owns the next troubleshooting step. Runtime `error.class` values are inputs to classification, not the ownership matrix itself.

| Ownership branch | When to use it | Evidence reviewed first | Typical runtime inputs |
| --- | --- | --- | --- |
| Claude Code setup | The operator fails before a gateway request is classified, or before the gateway emits any `error.class` value | `C-09` bootstrap evidence, Claude Code launch environment, statusline install state | missing `ANTHROPIC_BASE_URL`, Claude Code launch failure, missing statusline, pre-runtime auth/setup issues |
| gateway runtime/config | The gateway receives the request but routing or local gateway behavior is wrong | `last_routing.json`, statusline output, the exercised branch from `C-10` | `route`, local config mismatch, wrong branch selection, missing or inconsistent routing evidence |
| Azure transport | The gateway path is in place, but provider-facing transport or deployment resolution is failing | `last_routing.json`, statusline output, optional `trace.jsonl`, the redacted session outcome | `auth`, `url`, `deployment`, provider response failures, outbound request mismatch |
| broader drift | The observed behavior no longer matches the landed contract or evidence posture, or the failure does not fit one branch cleanly | bootstrap evidence, smoke evidence, routing evidence, and optional `trace.jsonl` | evidence-hook drift, public/private boundary drift, unsupported surface drift, multi-branch ambiguity |

Classification notes:

- If the operator fails before the gateway can emit `error.class`, start with Claude Code setup.
- If the gateway emitted `error.class` but the branch is wrong, inspect gateway runtime/config first.
- If the gateway is on the expected branch but the provider interaction fails, inspect Azure transport first.
- If the evidence itself looks inconsistent with the landed contract, classify as broader drift.

## Failure Ownership

Use the runtime `error.class` surface and the documented evidence order to decide who owns the next step.

| `error.class` | What it means | First checks | Escalate with |
| --- | --- | --- | --- |
| `auth` | Azure credentials were missing, malformed, or rejected | auth mode, API key or bearer token wiring, and any secret-loading step in the published bootstrap | redacted auth failure summary and the minimal bootstrap context needed to prove the secret path |
| `url` | The request target, base URL, or host path is wrong | `ANTHROPIC_BASE_URL`, localhost/loopback targeting, and the request path shape | redacted request target details and the observed routing history |
| `deployment` | The routed model did not resolve to the intended internal Azure deployment | model mapping, `mapping.actual_model`, and the config that binds the public label to the internal deployment | redacted model label plus the relevant mapping evidence |
| `route` | The gateway selected the wrong internal route or could not resolve it | `router.think`, `router.default`, prompt-routing rules, and the branch exercised | the branch identity, `last_routing.json`, and the smallest reproducible request summary |
| `transport_drift` | Live behavior no longer matches the landed transport or evidence contract | request shape, response shape, evidence-hook behavior, and whether the README or runtime anchors have drifted | the smallest redacted reproduction that shows the mismatch |

Guidance:

- `route` means routing logic is wrong while the transport still exists.
- `transport_drift` means the operator-visible behavior no longer matches the landed contract or evidence posture.
- Do not collapse these branches into a generic cleanup bucket.

## Redaction Rules

Keep the support story capability-oriented.

Required redaction rules:

- do not publish credentials, bearer tokens, or API keys
- do not publish raw deployment identifiers unless they are already part of the redacted support evidence needed to explain the failure class
- do not expose planner/executor identity as public truth
- do not require provider-only bypasses to explain or reproduce the failure

Allowed evidence:

- the public `ANTHROPIC_BASE_URL` launch path
- the public `/v1/messages` path
- the routed model label needed to explain which branch was exercised
- the failure class and the minimal surrounding context needed to diagnose it

## Reproducible Escalation

When a support case needs escalation, record:

- the branch exercised
- the `error.class` classification
- the required evidence listed above
- whether optional trace output was captured
- the minimum next-owner summary in redacted form

Keep escalation bounded:

- if the failure is `auth`, hand off the bootstrap/auth evidence, not the full session transcript
- if the failure is `url`, hand off the target and routing evidence, not private host metadata
- if the failure is `deployment`, hand off the mapping evidence, not a provider identity narrative
- if the failure is `route`, hand off the branch evidence and routing history, not internal source code
- if the failure is `transport_drift`, hand off the smallest reproducible mismatch and flag it as a contract drift case

## Closeout Input For S3

This section is handoff-only. It exists so later `S3` work can consume landed support truth without reopening the support design.

Checklist:

- confirm the guide still matches the landed `C-11` contract and the current bootstrap and smoke evidence surfaces
- record which branches are covered: normal execution, think / planner, and tool-loop continuation
- record the required support evidence: `last_routing.json`, statusline output, branch identity, and pass or fail outcome
- record whether `trace.jsonl` was needed or remained optional
- record any redaction constraints that future support work must preserve
- record any drift note if the README or runtime evidence surfaces changed while this guide was being prepared

This checklist is not closeout publication, thread advancement, or seam-exit accounting.
