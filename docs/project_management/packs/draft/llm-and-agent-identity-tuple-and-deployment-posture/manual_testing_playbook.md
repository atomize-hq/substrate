# llm-and-agent-identity-tuple-and-deployment-posture — manual testing playbook

This pack is semantic and planning-only. Validation is a deterministic cross-document review against the contract owners listed below.

## Contracts consumed

- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/platform-parity-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/compatibility-spec.md`
- `docs/contracts/substrate-gateway-status-schema.md`
- `docs/contracts/substrate-gateway-policy-evaluation.md`
- `docs/contracts/substrate-gateway-runtime-parity.md`
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`
- `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`

## One-owner-per-surface audit

Use this checklist to confirm each surfaced area has one owner and no second owner is implied elsewhere.

### 1) Tuple meanings and wording

Validate that operator-visible tuple meanings come from ADR-0042 and `contract.md`.

Check:

- `client`, `router`, `provider`, `auth_authority`, and `protocol` have the same meanings in ADR-0042 and `contract.md`.
- `in_world`, `host_only`, and `host_to_world_bridge` have the same meanings in ADR-0042 and `contract.md`.
- `direct_provider_path` is routing authority only and requires `host_only` when tuple and posture objects appear together.
- `host_to_world_bridge` is described only as transport and never as a router, gateway, or control plane.
- The human-readable labels in `contract.md` match the labels reused by `telemetry-spec.md`.

Pass condition:

- ADR-0042 and `contract.md` present one tuple vocabulary and one placement-posture vocabulary with no owner conflict.

### 2) Machine-readable schema ownership

Validate that the tuple and posture object shapes are owned only by `identity-tuple-schema-spec.md`.

Check:

- `identity_tuple` and `placement_posture` are the only machine-readable object names used by this pack.
- Required fields, optional fields, omission rules, and token grammar appear in `identity-tuple-schema-spec.md`.
- `telemetry-spec.md`, `platform-parity-spec.md`, and `compatibility-spec.md` consume those shapes without renaming fields or widening types.
- No doc uses `null`, empty strings, or placeholder text as the omission model for `provider` or `auth_authority`.

Pass condition:

- One schema owner exists and every consumer doc reuses its field names and omission rules unchanged.

### 3) Policy and telemetry owner lines

Validate that routing, status placement, and trace placement each keep the correct owner line.

Check:

- `policy-spec.md` owns routing-hint evaluation, direct-provider gating, and bridge transport-only policy rules.
- ADR-0043 owns tuple-axis policy keys under `llm.constraints.*` and does not restate tuple meanings as a competing owner.
- `telemetry-spec.md` owns additive placement of `identity_tuple` and `placement_posture` on status, diagnostics, and trace surfaces.
- `docs/contracts/substrate-gateway-status-schema.md` remains the owner of the top-level `status --json` envelope and `client_wiring.*`.
- ADR-0028 remains the owner of canonical trace correlation keys and tuple publication remains additive relative to those keys.

Pass condition:

- Policy, status placement, and trace placement each map to one owner and no doc blurs those boundaries.

### 4) Platform parity and compatibility

Validate that parity and compatibility statements stay bounded to the correct docs.

Check:

- `platform-parity-spec.md` states one tuple and placement-posture meaning across Linux, macOS, and Windows.
- `platform-parity-spec.md` states that bridge transport does not alter in-world `net_allowed` governance.
- `compatibility-spec.md` states that `backend_id` remains adapter selection only.
- ADR-0040, ADR-0041, and ADR-0046 are consumed as boundary anchors and are not rewritten locally as competing owners.
- Windows wording treats WSL as hidden transport detail only.

Pass condition:

- Parity and compatibility claims stay bounded to one owner each and do not reopen runtime-ownership or backend-selection ownership.

## Example review cases

### Case 1 — Claude Code pointed at `substrate_gateway`

Review surfaces:

- ADR-0042 example text
- `contract.md`
- `telemetry-spec.md`

Assertions:

- `client=claude_code`
- `router=substrate_gateway`
- `provider` remains independent from `client`
- `auth_authority` remains independent from `provider`
- `protocol` remains capability metadata rather than routing authority

Pass condition:

- Every reviewed surface preserves the same tuple meaning and does not collapse the example into one overloaded backend label.

### Case 2 — Codex using Responses API and `~/.codex/auth.json`

Review surfaces:

- ADR-0042 example text
- `contract.md`
- `policy-spec.md`
- `compatibility-spec.md`

Assertions:

- `client=codex`
- `protocol=openai.responses`
- `~/.codex/auth.json` is treated as an illustrative example path only
- `auth_authority` is not rewritten into `provider`
- `router=direct_provider_path` remains policy-gated and requires `host_only` when published with posture

Pass condition:

- The Codex example keeps client, protocol, auth authority, and router meaning distinct across all reviewed surfaces.

### Case 3 — Pre-provider-selection publication

Review surfaces:

- `identity-tuple-schema-spec.md`
- `policy-spec.md`
- `telemetry-spec.md`

Assertions:

- `provider` omits by field absence only
- `auth_authority` omits by field absence only
- no reviewed surface uses `null`, empty string, or placeholder text such as `unknown`
- `policy-spec.md` denies unresolved `provider` or `auth_authority` when the corresponding constraint list is non-empty

Pass condition:

- Omission semantics remain identical across schema, policy, and telemetry wording.

## Stale-reference checks

Run these searches during review and resolve every active-surface match that violates the pack boundary.

1. Search for overloaded backend wording:

```bash
rg -n 'backend_id.*(client|router|provider|auth_authority|protocol)|backend label' \
  docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture \
  docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md \
  docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md \
  docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md
```

2. Search for bridge wording that implies a second control plane:

```bash
rg -n 'host_to_world_bridge|second control plane|second gateway|host gateway' \
  docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture \
  docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md \
  docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md
```

3. Search for status-schema drift:

```bash
rg -n 'client_wiring|identity_tuple|placement_posture' \
  docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture \
  docs/contracts/substrate-gateway-status-schema.md
```

4. Search for stale active or backup references presented as current owners:

```bash
rg -n 'packs/(active|draft/.+-backup)' \
  docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture \
  docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md \
  docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md
```

Accept only if every remaining match is clearly historical evidence rather than current ownership.

## Manual pass/fail summary

Pass when:

- each surfaced area has one owner
- the Claude Code and Codex examples remain semantically stable across ADR-0042 and this pack
- omission rules remain identical across schema, policy, and telemetry docs
- platform parity and compatibility wording remain bounded and non-overlapping
- no active doc overloads `backend_id` into tuple meaning

Fail when:

- any surfaced area claims two owners
- any example rewrites `client`, `router`, `provider`, `auth_authority`, or `protocol`
- any doc treats `host_to_world_bridge` as more than transport
- any doc turns `backend_id` into semantic identity
- any active reference presents archived or backup wording as the current owner
