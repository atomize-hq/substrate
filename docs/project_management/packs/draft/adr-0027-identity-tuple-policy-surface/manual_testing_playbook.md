# adr-0027-identity-tuple-policy-surface — manual testing playbook

This pack is documentation-driven. Validation is a deterministic contract and wording review against the current CLI/test surface plus the authored specs in this feature directory.

## Contracts consumed

- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/policy-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/telemetry-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/compatibility-spec.md`
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/contracts/gateway/policy-evaluation.md`
- `crates/broker/src/tests.rs`
- `crates/shell/src/builtins/world_gateway.rs`
- `crates/shell/tests/world_gateway.rs`

## Review setup

Run all commands from the repository root. Use a scratch `SUBSTRATE_HOME` and a workspace fixture so policy and config views are deterministic.

Recommended shell setup:

```bash
export SUBSTRATE_HOME="$(mktemp -d)"
mkdir -p "$SUBSTRATE_HOME"
mkdir -p .substrate
```

## Smoke entrypoints

Use these scripts for repeatable platform smoke before drilling into the manual matrix.

Behavior-platform smoke coverage:
- Linux and macOS smoke automate the minimal contract subset from sections 1 through 5:
  - authoritative `substrate policy current show --json --explain` output and explain provenance
  - schema-invalid tuple-policy rejection
  - machine-readable `substrate world gateway status --json` tuple publication shape
  - router and provider mismatch deny wording
- Windows smoke remains optional manual evidence for this pack and automates only the policy inspection plus schema-invalid checks because Windows is compile-parity only in `tasks.json`.

- Linux:

```bash
bash docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/smoke/linux-smoke.sh
```

Expected results:
- exit code `0`
- `stdout` contains `OK: adr-0027-identity-tuple-policy-surface linux smoke passed`

- macOS:

```bash
bash docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/smoke/macos-smoke.sh
```

Expected results:
- exit code `0`
- `stdout` contains `OK: adr-0027-identity-tuple-policy-surface macos smoke passed`

- Windows:

```powershell
pwsh -File docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/smoke/windows-smoke.ps1
```

Expected results:
- exit code `0`
- `stdout` contains `OK: adr-0027-identity-tuple-policy-surface windows smoke passed`

Use sections 1 through 8 below as the debugging path when a smoke script fails. Smoke covers the minimum runnable subset above; sections 6 through 8 remain the manual extension path for protocol mismatch, auth-authority mismatch, and validation-only auth-file review.

## Manual review matrix

### 1) Effective merged policy view

Goal:
- confirm `substrate policy current show --explain` is the authoritative merged view for `llm.constraints.*`

Command:

```bash
substrate policy current show --json --explain
```

Expected results:

- exit code `0`
- `stdout` is one JSON object containing the effective merged policy
- `stderr` is one JSON object with `kind = "substrate.policy.explain.v1"`
- `stdout` contains these policy paths when configured:
  - `/llm/constraints/routers`
  - `/llm/constraints/providers`
  - `/llm/constraints/protocols`
  - `/llm/constraints/auth_authorities`
- `stderr` provenance for each configured key appears under `/keys/<policy_key>/sources`

Fail if:

- the authoritative merged view is assigned to `substrate config show --explain`
- `stderr` is not a single explain JSON object
- any tuple-policy key is documented without a provenance path

### 2) Schema-invalid tuple-policy input

Goal:
- confirm invalid tuple-policy values fail hard as exit code `2`

Command examples:

```bash
substrate policy global set --json 'llm.constraints.providers=["OpenAI"]'
substrate policy global set --json 'llm.constraints.protocols=["openai"]'
```

Expected results:

- exit code `2`
- `stderr` mentions the invalid policy key or invalid token grammar
- the effective policy is not updated with a normalized fallback value

Fail if:

- invalid values degrade silently
- invalid values are rewritten instead of rejected

### 3) Gateway status tuple publication

Goal:
- confirm tuple-aware publication reuses `identity_tuple` and `placement_posture`

Commands:

```bash
substrate world gateway status --json
substrate world gateway status
```

Expected `--json` results:

- exit code `0` or `4` depending on runtime availability
- `identity_tuple` is a top-level sibling of `status`
- `placement_posture` is a top-level sibling of `status`
- `identity_tuple` never appears under `client_wiring`
- `placement_posture` never appears under `client_wiring`

Expected human-readable results:

- labels appear in this order when available:
  - `originating client`
  - `routing authority`
  - `fulfillment provider`
  - `auth authority`
  - `protocol`
  - `deployment posture`
  - `bridge transport`
- missing optional fields are omitted without placeholders

Fail if:

- `backend:` appears as a tuple label
- `unknown`, `n/a`, or empty placeholder lines are used for omitted fields

### 4) Router mismatch denial

Goal:
- confirm router-axis denial wording matches the locked contract

Fixture policy:

```yaml
llm:
  allowed_backends:
    - api:openai
  constraints:
    routers:
      - direct_provider_path
```

Command:

```bash
substrate world gateway status
```

Expected results:

- exit code `5`
- `stderr` contains `substrate world gateway status: policy or safety failure`
- `stderr` contains `substrate world gateway: gateway_policy_blocked: effective gateway routing authority 'substrate_gateway' is not allowlisted by llm.constraints.routers`

### 5) Provider mismatch denial

Goal:
- confirm provider-axis denial wording matches the locked contract

Fixture policy:

```yaml
llm:
  allowed_backends:
    - api:openai
  constraints:
    providers:
      - anthropic
```

Command:

```bash
substrate world gateway status
```

Expected results:

- exit code `5`
- `stderr` contains `effective gateway provider 'openai' is not allowlisted by llm.constraints.providers`

### 6) Protocol mismatch denial

Goal:
- confirm protocol-axis denial wording matches the locked contract

Fixture policy:

```yaml
llm:
  allowed_backends:
    - api:openai
  constraints:
    protocols:
      - anthropic.messages
```

Command:

```bash
substrate world gateway status
```

Expected results:

- exit code `5`
- `stderr` contains `effective gateway protocol 'openai.responses' is not allowlisted by llm.constraints.protocols`

### 7) Auth-authority mismatch or unresolved denial

Goal:
- confirm auth-authority denial keeps the auth-source class distinct from provider

Fixture policy:

```yaml
llm:
  allowed_backends:
    - api:openai
  constraints:
    auth_authorities:
      - anthropic_api_key
```

Command:

```bash
substrate world gateway status
```

Expected results:

- exit code `5`
- `stderr` contains one of:
  - `effective gateway auth authority 'codex_subscription' is not allowlisted by llm.constraints.auth_authorities`
  - `effective gateway auth authority is unresolved while llm.constraints.auth_authorities is constrained`

Fail if:

- the deny wording rewrites auth authority into provider identity
- the deny wording hides the denying policy key

### 8) Validation-only Codex auth example

Goal:
- confirm `~/.codex/auth.json` remains a validation example rather than a new Substrate-owned path contract

Review targets:

```bash
rg -n '~/.codex/auth.json|codex_subscription|openai_api_key' \
  docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface \
  docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md
```

Expected results:

- `~/.codex/auth.json` appears only as illustrative validation input
- the docs describe it as auth-authority evidence, not as a new policy or config root
- no owned doc promotes the path into a persistent Substrate contract

## One-owner-per-surface checklist

Pass every item before promoting this pack:

- `tuple-policy-schema-spec.md` is the only owner of `llm.constraints.*` key grammar, defaults, and empty-list semantics.
- `policy-spec.md` is the only owner of tuple-axis evaluation ordering and deny taxonomy.
- `telemetry-spec.md` is the only owner of tuple-aware allow and deny publication rules.
- `compatibility-spec.md` is the only owner of additive rollout and promotion invariants.
- ADR-0042 remains the semantic owner of `client`, `router`, `provider`, `auth_authority`, `protocol`, `identity_tuple`, and `placement_posture`.
- ADR-0028 remains the owner of the base trace envelope and correlation-key vocabulary.

## Stale-reference checks

Run these searches and resolve any active-surface mismatch.

1. Inspection-surface drift:

```bash
rg -n 'config show --explain|policy current show --explain' \
  docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface \
  docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md
```

Accept only if `substrate policy current show --explain` is the authoritative merged view for `llm.constraints.*`.

2. Backend-surrogate drift:

```bash
rg -n 'backend_id.*(provider|protocol|auth_authority|router)|backend:' \
  docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface \
  docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md \
  docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md
```

Accept only if remaining matches refer to adapter selection or historical evidence.

3. Secret-path drift:

```bash
rg -n '~/.codex/auth.json|API key|token|cookie' \
  docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface \
  docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md
```

Accept only if illustrative credential paths remain examples and secret material remains redacted or omitted.

## Manual pass criteria

Pass when:

- `substrate policy current show --explain` is the only authoritative merged view for tuple-policy keys
- invalid tuple-policy values fail as exit code `2`
- router, provider, protocol, and auth-authority denials use the exact locked wording family and exit code `5`
- tuple-aware telemetry reuses `identity_tuple` and `placement_posture`
- `~/.codex/auth.json` remains validation input only

Fail when:

- config and policy ownership are conflated
- any deny path hides the denying policy key
- tuple-aware publication invents a second tuple schema
- any owned doc promotes secret-bearing auth sources or illustrative credential paths into a new product contract
