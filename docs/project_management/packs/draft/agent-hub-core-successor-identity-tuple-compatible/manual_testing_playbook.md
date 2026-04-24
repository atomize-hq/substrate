# agent-hub-core-successor-identity-tuple-compatible — manual testing playbook

This playbook is the authoritative manual validation checklist for the feature pack.

## Contracts consumed

- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/contract.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/agent-hub-session-protocol-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/policy-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/telemetry-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/platform-parity-spec.md`
- `docs/project_management/packs/draft/agent-hub-core-successor-identity-tuple-compatible/compatibility-spec.md`
- `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md`
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/project_management/adrs/draft/ADR-0044-agent-hub-core-successor-identity-tuple-compatible.md`
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- `crates/shell/tests/agents_validate.rs`
- `crates/shell/tests/agent_hub_trace_persistence.rs`
- `crates/shell/tests/repl_world_first_routing_v1.rs`

## Scope

This playbook proves five operator-visible outcomes:

- `substrate agent list` and `substrate agent status` publish the successor command namespace and keep `backend_id` as the adapter identifier only.
- the selected orchestrator is host-scoped and `substrate agent doctor` fails closed when that requirement is violated.
- world-scoped member rows publish `world_id` plus `world_generation`, while host-scoped rows omit both.
- pure-agent rows omit `provider` plus `auth_authority`, while nested gateway-backed records publish both on the nested record only.
- the feature pack keeps one owner per surface across contract, protocol, policy, telemetry, parity, and compatibility docs.

## Review setup

Run commands from the repository root.

Use a scratch Substrate home and workspace fixture so config, policy, and trace output are deterministic:

```bash
export SUBSTRATE_HOME="$(mktemp -d)"
workspace="$(mktemp -d)"
trace="$SUBSTRATE_HOME/trace.jsonl"
artifacts_dir="$SUBSTRATE_HOME/manual-playbook"
mkdir -p "$workspace"
mkdir -p "$artifacts_dir"
cd "$workspace"
```

Fixture requirements:

- the effective inventory contains one host-scoped orchestrator agent selected by `agents.hub.orchestrator_agent_id`
- the effective inventory contains at least one host-scoped member and one world-scoped member
- the effective policy allowlists the derived orchestrator and member `backend_id` values under `agents.allowed_backends`
- the fixture can trigger one nested gateway-backed request so `nested_llm_records` and correlated trace output are non-empty

Evidence capture requirements:

- save `stdout`, `stderr`, and exit code for every manual case
- save the resulting `trace.jsonl` fragment for every status or nested-record case
- record the platform under test as `linux`, `macos`, or `windows`

## Platform readiness entrypoints

Run the platform readiness path before the command matrix.

### Linux

Commands:

```bash
scripts/linux/world-provision.sh --profile release
substrate world doctor --json
```

Expected results:

- provisioning completes without error
- `substrate world doctor --json` exits `0`

### macOS

Commands:

```bash
bash scripts/mac/lima-warm.sh
bash scripts/mac/smoke.sh
substrate world doctor --json
```

Expected results:

- warm flow completes without error
- smoke completes without error
- `substrate world doctor --json` exits `0`

### Windows

Commands:

```powershell
pwsh -File scripts/windows/wsl-warm.ps1 -DistroName substrate-wsl -ProjectPath (Resolve-Path .)
pwsh -File scripts/windows/wsl-smoke.ps1
substrate world doctor --json
```

Expected results:

- warm flow completes without error
- smoke completes without error
- `substrate world doctor --json` exits `0`

## Command matrix

### Case 1 — `substrate agent list --json` keeps adapter identity and omission rules

Command:

```bash
substrate agent list --json | tee "$artifacts_dir/agent-list.json"
```

Assertions:

```bash
jq -e '
  .disabled == false and
  (.agents | type == "array") and
  (all(.agents[];
    (.agent_id | type == "string") and
    (.backend_id | test("^[a-z0-9_]+:[A-Za-z0-9_.-]+$")) and
    (.execution.scope == "host" or .execution.scope == "world") and
    (.protocol == "uaa.agent.session") and
    (has("provider") | not) and
    (has("auth_authority") | not) and
    (has("world_id") | not) and
    (has("world_generation") | not)
  ))
' "$artifacts_dir/agent-list.json"
```

Expected results:

- command exits `0`
- the orchestrator row uses `role = "orchestrator"`
- every `backend_id` remains derived adapter identity only
- no list row includes `provider`, `auth_authority`, `world_id`, or `world_generation`

### Case 2 — `substrate agent status --json` proves a host-scoped orchestrator

Command:

```bash
substrate agent status --json | tee "$artifacts_dir/agent-status.json"
```

Assertions:

```bash
jq -e '
  .orchestrator_agent_id as $orchestrator |
  ($orchestrator | type == "string") and
  any(.sessions[];
    .agent_id == $orchestrator and
    .role == "orchestrator"
  )
' "$artifacts_dir/agent-status.json"
```

```bash
jq -e '
  any(.sessions[];
    .role == "orchestrator" and
    .execution.scope == "host" and
    .router == "agent_hub" and
    .protocol == "uaa.agent.session" and
    (has("provider") | not) and
    (has("auth_authority") | not) and
    (has("world_id") | not) and
    (has("world_generation") | not)
  )
' "$artifacts_dir/agent-status.json"
```

Expected results:

- command exits `0`
- at least one session row has `role = "orchestrator"` and `execution.scope = "host"`
- the orchestrator row omits `provider`, `auth_authority`, `world_id`, and `world_generation`

### Case 3 — world-scoped members publish `world_id` and `world_generation`

Command:

```bash
substrate agent status --json | tee "$artifacts_dir/agent-status-world.json"
```

Assertions:

```bash
jq -e '
  any(.sessions[];
    .role != "orchestrator" and
    .execution.scope == "world" and
    (.world_id | type == "string") and
    (.world_generation | type == "number") and
    (has("provider") | not) and
    (has("auth_authority") | not)
  )
' "$artifacts_dir/agent-status-world.json"
```

```bash
jq -e '
  all(.sessions[];
    if .execution.scope == "host"
    then (has("world_id") | not) and (has("world_generation") | not)
    else true
    end
  )
' "$artifacts_dir/agent-status-world.json"
```

Expected results:

- command exits `0`
- every world-scoped member row includes both `world_id` and `world_generation`
- every host-scoped row omits both fields

### Case 4 — nested gateway-backed records publish `provider` and `auth_authority` on the nested record only

Trigger one nested gateway-backed request with the feature fixture, then rerun status:

```bash
substrate agent status --json | tee "$artifacts_dir/agent-status-nested.json"
```

Assertions:

```bash
jq -e '
  (.nested_llm_records | type == "array") and
  (length > 0) and
  all(.nested_llm_records[];
    .router == "substrate_gateway" and
    (.provider | type == "string") and
    (.auth_authority | type == "string") and
    (.protocol | type == "string") and
    (has("world_id") | not) and
    (has("world_generation") | not)
  )
' "$artifacts_dir/agent-status-nested.json"
```

```bash
jq -e '
  all(.sessions[];
    (has("provider") | not) and (has("auth_authority") | not)
  )
' "$artifacts_dir/agent-status-nested.json"
```

Expected results:

- command exits `0`
- every nested record includes `provider` and `auth_authority`
- every pure-agent session row omits `provider` and `auth_authority`
- no nested record includes `world_id` or `world_generation`

### Case 5 — canonical trace keeps the same pure-agent versus nested-record split

Command:

```bash
test -f "$trace"
jq -e '
  any(select(
    .event_type == "agent_event" and
    .router == "agent_hub" and
    .protocol == "uaa.agent.session" and
    (.provider? == null) and
    (.auth_authority? == null)
  ))
' "$trace"
```

```bash
jq -e '
  any(select(
    .event_type == "agent_event" and
    .router == "substrate_gateway" and
    (.provider | type == "string") and
    (.auth_authority | type == "string") and
    (.world_id? == null) and
    (.world_generation? == null)
  ))
' "$trace"
```

Expected results:

- both `jq -e` probes exit `0`
- the pure-agent trace family omits `provider` and `auth_authority`
- the nested record family omits `world_id` and `world_generation`

### Case 6 — `substrate agent doctor --json` proves healthy ordered checks

Command:

```bash
substrate agent doctor --json | tee "$artifacts_dir/agent-doctor.json"
```

Assertions:

```bash
jq -e '
  .healthy == true and
  .fail_closed == false and
  (.orchestrator.agent_id | type == "string") and
  .orchestrator.execution.scope == "host" and
  (.checks | map(.check)) == [
    "inventory_scan",
    "orchestrator_selection",
    "policy_allowlist",
    "world_boundary"
  ] and
  all(.checks[];
    .status == "pass" or .status == "not_applicable"
  )
' "$artifacts_dir/agent-doctor.json"
```

Expected results:

- command exits `0`
- `orchestrator.execution.scope` is `host`
- checks stay in the locked order
- `world_boundary` is `pass` or `not_applicable`

### Case 7 — `substrate agent doctor` fails closed for invalid orchestrator state

Run this case against three fixture variants:

- missing `agents.hub.orchestrator_agent_id`
- disabled orchestrator inventory item
- world-scoped orchestrator inventory item

Command:

```bash
substrate agent doctor --json
```

Expected results:

- command exits `2`
- output reports `healthy = false`
- output reports `fail_closed = true`
- the first failing check is `orchestrator_selection`
- the reason names the exact failing condition from `policy-spec.md`

### Case 8 — `substrate agent doctor` fails closed for world-boundary loss

Run this case against two fixture variants:

- required world-scoped member posture with a temporarily unavailable world boundary
- required world-scoped member posture on a platform or build that cannot satisfy the posture

Command:

```bash
substrate agent doctor --json
```

Expected results:

- temporary unavailability exits `3`
- unsupported required posture exits `4`
- `world_boundary` is the first failing check
- list and status evidence captured in the same run do not invent host-only success rows for the affected world-scoped member path

## One-owner-per-surface review checklist

Use this checklist after the command matrix passes.

### 1) Command contract owner

Check:

- `contract.md` owns CLI spelling, JSON keys, render order, omission rules, and exit-code mapping
- `manual_testing_playbook.md` consumes that contract without redefining any JSON field or exit code
- `substrate agents validate` appears only as the inventory-validation compatibility leaf

Pass condition:

- one owner exists for the successor command contract

### 2) Session and lifecycle owner

Check:

- `agent-hub-session-protocol-spec.md` owns capability descriptors, session handles, lifecycle states, and the machine-readable status objects
- this playbook uses the same `sessions` and `nested_llm_records` object names without widening them
- world reuse and restart wording in this playbook matches the protocol spec

Pass condition:

- one owner exists for session schema and lifecycle meaning

### 3) Policy and fail-closed owner

Check:

- `policy-spec.md` owns ordered deny evaluation and exact deny reasons
- `contract.md` owns operator-visible exit posture
- this playbook uses the same ordered doctor checks: `inventory_scan`, `orchestrator_selection`, `policy_allowlist`, `world_boundary`

Pass condition:

- one owner exists for fail-closed evaluation and one owner exists for exit posture

### 4) Telemetry owner

Check:

- `telemetry-spec.md` owns top-level field placement for pure-agent and nested records
- this playbook treats trace review as evidence only and does not rename fields
- pure-agent omission rules and nested-record presence rules match `telemetry-spec.md`

Pass condition:

- one owner exists for telemetry field placement and correlation wording

### 5) Parity and compatibility owners

Check:

- `platform-parity-spec.md` owns Linux, macOS, and Windows parity guarantees
- `compatibility-spec.md` owns ADR-0025 supersession and `backend_id` migration wording
- this playbook does not assign parity or compatibility meaning to any other file

Pass condition:

- one owner exists for parity and one owner exists for compatibility

### 6) Verification surface alignment

Check:

- the playbook names only these implementation-facing evidence surfaces:
  - `crates/shell/tests/agents_validate.rs`
  - `crates/shell/tests/agent_hub_trace_persistence.rs`
  - `crates/shell/tests/repl_world_first_routing_v1.rs`
  - `scripts/linux/world-provision.sh`
  - `scripts/mac/lima-warm.sh`
  - `scripts/mac/smoke.sh`
  - `scripts/windows/wsl-warm.ps1`
  - `scripts/windows/wsl-smoke.ps1`
- if execution work adds a new dedicated command or session-protocol test file, update this playbook in the same change

Pass condition:

- one bounded evidence list exists for runtime verification surfaces

## Pass summary

Pass this playbook only when:

- Cases 1 through 8 meet the exact exit codes and field assertions above
- pure-agent rows omit `provider` plus `auth_authority`
- world-scoped member rows publish `world_id` plus `world_generation`
- nested gateway-backed records publish `provider` plus `auth_authority` on the nested record only
- the one-owner-per-surface checklist passes without a conflicting owner
