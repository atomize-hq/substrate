# Claude Code Live Session Smoke Procedure And Evidence Manifest

## Purpose

This companion note supports `C-10` by giving operators one bounded procedure for the live smoke path and one bounded evidence manifest for review and later closeout preparation.

It is intentionally narrow:

- it follows the published `C-09` bootstrap path and the landed `C-10` live smoke contract
- it keeps provider, planner, executor, and deployment identity internal or support-facing
- it prepares closeout inputs for `S3` without claiming live execution has already been published

It does not define:

- runtime transport behavior
- smoke closeout accounting
- promotion readiness
- downstream troubleshooting ownership

## Canonical Sources

This note is grounded in:

- `docs/foundation/claude-code-c10-live-session-smoke-verification-contract.md`
- `docs/foundation/claude-code-c09-operator-bootstrap-contract.md`
- `crates/gateway/docs/project_management/packs/active/claude-code-live-integration-smoke/threaded-seams/seam-2-live-session-smoke-verification/seam.md`
- `crates/gateway/docs/project_management/packs/active/claude-code-live-integration-smoke/threaded-seams/seam-2-live-session-smoke-verification/review.md`
- `crates/gateway/docs/project_management/packs/active/claude-code-live-integration-smoke/threading.md`
- `gateway/README.md`
- `gateway/src/router/mod.rs`
- `gateway/src/server/mod.rs`

If this note and those anchors disagree, the note or upstream evidence must be revalidated before downstream execution proceeds.

## Operator Procedure

1. Complete the bootstrap steps in `gateway/README.md`.
2. Ensure the Claude Code statusline script is installed before smoke.
3. Decide whether optional tracing is needed for redacted debugging evidence.
4. Launch Claude Code against `ANTHROPIC_BASE_URL="http://127.0.0.1:13456"`.
5. Exercise the normal execution branch through the public `/v1/messages` path.
6. Exercise the think / planner branch through the public `/v1/messages` path with thinking enabled.
7. Exercise the tool-loop continuation branch through the public `/v1/messages` path with a plan-mode follow-up that contains only tool results.
8. Record the branch identity, routing evidence, and redacted session outcome for each run.

## Evidence Manifest

### Required Evidence

- `~/.substrate-gateway/last_routing.json`
- statusline output from the installed Claude Code statusline script
- the branch identity exercised for each run: normal execution, think / planner, or tool-loop continuation
- the redacted session outcome for each run: pass or fail

### Optional Evidence

- `~/.substrate-gateway/trace.jsonl` when additional redacted debugging evidence is needed

### Redaction Rules

- redact credentials, bearer tokens, and API keys
- redact raw deployment identifiers if they would expose internal provider mapping as public truth
- redact host or config details when they are only needed as environment identifiers
- keep enough surrounding context to classify the failure without reading runtime code

## Closeout-Input Checklist

Use this checklist to prepare `S3` inputs without claiming closeout has already been published.

- `C-10` is the source of truth for the three required branches and evidence posture.
- The README procedure and this note agree on the normal, think / planner, and tool-loop continuation branches.
- `last_routing.json` and the statusline output are available for each branch run.
- The branch identity and pass or fail outcome are recorded for each branch run.
- Optional `trace.jsonl` was captured only when needed for redacted debugging evidence.
- The recorded evidence is sufficient for downstream closeout accounting without exposing provider or deployment identity as public truth.
- No `S3` closeout, promotion, or troubleshooting text is claimed by this note.

## Review Boundaries

This note supports `S2` delivery work only.

- It does not publish `THR-09`.
- It does not update the seam closeout.
- It does not change runtime code.
- It does not expand the `C-10` contract.
