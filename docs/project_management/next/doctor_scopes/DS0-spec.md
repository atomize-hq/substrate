# DS0-spec — Doctor scopes (host vs world)

Driven by:
- ADR: `docs/project_management/next/ADR-0007-host-and-world-doctor-scopes.md`
- Decision register: `docs/project_management/next/doctor_scopes/decision_register.md`

## Scope
- Add a new CLI surface: `substrate host doctor [--json]`.
- Redefine `substrate world doctor [--json]` as “world-scoped” by querying the world-agent for an authoritative world readiness report.
- Introduce a world-agent endpoint: `GET /v1/doctor/world`.
- Update internal consumers that parse doctor JSON (health/shim snapshots, world verify, docs).

## Definitions (terms used in this spec)
- **Host**: the OS where `substrate` CLI is running.
- **World**: the isolation runtime where the world-agent runs (Linux namespace world on Linux; Lima guest on macOS; WSL on Windows).
- **Host doctor**: diagnostics that are computable from the host without requiring guest-kernel inference.
- **World doctor**: diagnostics that are authoritative only from the world-agent / guest kernel perspective.

## User contract (authoritative)

### CLI

#### `substrate host doctor [--json]`
- Purpose: report host readiness for world routing (host prerequisites + transport readiness).
- Side effects: none (no provisioning, no spawning, no VM start).
- Output:
  - Text: PASS/WARN/FAIL lines (human-first) under a `== substrate host doctor ==` header.
  - JSON: a stable, versioned object (see “JSON contracts”).

#### `substrate world doctor [--json]`
- Purpose: report world enforcement readiness (agent-reported world facts) and include the host doctor report as a sibling block.
- Side effects: none (no provisioning, no spawning, no VM start).
- Behavior:
  - MUST compute the effective config/policy for the current directory (honoring global CLI overrides `--world/--no-world`).
  - If world isolation is disabled by effective config, MUST short-circuit (no socket probing, no agent calls).
  - Otherwise MUST:
    1) run the host doctor probes and include their results as the `host` block; then
    2) call the world-agent endpoint `GET /v1/doctor/world` and include the response as the `world` block.
- Output:
  - Text: PASS/WARN/FAIL lines grouped into explicit `== Host ==` and `== World ==` sections under a `== substrate world doctor ==` header.
  - JSON: a stable, versioned object (see “JSON contracts”).

### Exit codes
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

#### `substrate host doctor`
- `0`: `ok=true`
- `3`: required dependency unavailable for host probes (e.g., `limactl` missing on macOS when required to report host readiness)
- `4`: not supported / missing prerequisites (`ok=false` due to missing host prerequisites or world disabled)
- `1`: unexpected internal error (I/O error, panic, bug)

#### `substrate world doctor`
- `0`: `ok=true` (both `host.ok=true` and `world.ok=true`)
- `3`: world enabled but world-agent is unreachable (transport connect/probe/HTTP failure)
- `4`: world disabled/not provisioned, or world-agent reachable but cannot enforce required primitives (`world.ok=false` due to missing prerequisites)
- `2`: CLI usage/config error (invalid flags/args; invalid config/policy read)
- `1`: unexpected internal error (I/O error, panic, bug)

### Configuration
- No new configuration keys are introduced by DS0.
- No new environment variables are introduced by DS0.

## Agent API contract (authoritative)

### Endpoint
- Method: `GET`
- Path: `/v1/doctor/world`
- Request body: none
- Response: JSON object, schema versioned (below)

### `WorldDoctorReportV1` (world-agent response JSON)
```json
{
  "schema_version": 1,
  "ok": true,
  "collected_at_utc": "2026-01-08T00:00:00Z",
  "landlock": {
    "supported": true,
    "abi": 3,
    "reason": null
  },
  "world_fs_strategy": {
    "primary": "overlay",
    "fallback": "fuse",
    "probe": {
      "id": "enumeration_v1",
      "probe_file": ".substrate_enum_probe",
      "result": "pass",
      "failure_reason": null
    }
  }
}
```

#### Field contract (exact)
- `schema_version`: integer, must equal `1`.
- `ok`: boolean; must be `true` iff:
  - `landlock.supported == true`, and
  - `world_fs_strategy.probe.result == "pass"`.
- `collected_at_utc`: RFC3339 UTC timestamp with `Z` suffix (seconds precision).
- `landlock.supported`: boolean.
- `landlock.abi`: integer when supported; otherwise `null`.
- `landlock.reason`: string when `supported=false`; otherwise `null`.
- `world_fs_strategy.primary`: string enum, exactly `"overlay"`.
- `world_fs_strategy.fallback`: string enum, exactly `"fuse"`.
- `world_fs_strategy.probe.id`: string, exactly `"enumeration_v1"`.
- `world_fs_strategy.probe.probe_file`: string, exactly `".substrate_enum_probe"`.
- `world_fs_strategy.probe.result`: string enum: `"pass"` or `"fail"`.
- `world_fs_strategy.probe.failure_reason`: nullable string; non-null only when `result=="fail"`.

### Error handling
- If the world-agent cannot compute the report due to an internal error, it MUST:
  - return `HTTP 500` with a JSON error body consistent with existing agent error responses, and
  - MUST NOT return `ok=true`.

## JSON contracts (authoritative)

### `substrate host doctor --json` output: `HostDoctorEnvelopeV1`
```json
{
  "schema_version": 1,
  "platform": "linux",
  "world_enabled": true,
  "ok": true,
  "host": { "platform": "linux", "ok": true }
}
```

### `substrate world doctor --json` output: `WorldDoctorEnvelopeV1`
```json
{
  "schema_version": 1,
  "platform": "macos",
  "world_enabled": true,
  "ok": true,
  "host": { "platform": "macos", "ok": true },
  "world": { "schema_version": 1, "ok": true }
}
```

#### Envelope field contract (exact)
- `schema_version`: integer, must equal `1`.
- `platform`: string enum: `"linux"`, `"macos"`, `"windows"`.
- `world_enabled`: boolean; effective config after applying global CLI overrides `--world/--no-world`.
- `ok`: boolean:
  - host doctor: equals `host.ok`.
  - world doctor: equals `host.ok && world.ok`.
- `host`: `HostDoctorReportV1` (below).
- `world`: present only for world doctor:
  - when `world_enabled==false`: `world.status == "disabled"` and `world.ok==false`;
  - when `world_enabled==true` but agent unreachable: `world.status == "unreachable"` and `world.ok==false`;
  - otherwise: `world` equals the agent’s `WorldDoctorReportV1` response plus `status == "ok"` when `ok==true` and `status == "missing_prereqs"` when `ok==false`.

### `HostDoctorReportV1` (platform-specific; stable per platform)

#### Linux host report (`host.platform=="linux"`)
Required fields:
- `platform`: `"linux"`
- `ok`: boolean; must be `true` iff all of:
  - `world_enabled==true`, and
  - `world_socket.socket_exists==true`, and
  - `world_socket.probe_ok==true`, and
  - `(overlay_present==true) || (fuse.dev==true && fuse.bin==true)`, and
  - `cgroup_v2==true`, and
  - `nft_present==true`
- `overlay_present`: boolean
- `fuse.dev`: boolean
- `fuse.bin`: boolean
- `cgroup_v2`: boolean
- `nft_present`: boolean
- `dmesg_restrict`: string (or `"n/a"` when unknown)
- `overlay_root`: string path
- `copydiff_root`: string path
- `world_fs_mode`: string enum: `"writable"` or `"read_only"`
- `world_fs_isolation`: string enum: `"workspace"` or `"full"`
- `world_fs_require_world`: boolean
- `world_socket`: object with required fields:
  - `mode`: string enum: `"socket_activation"` or `"manual"`
  - `socket_path`: string
  - `socket_exists`: boolean
  - `probe_ok`: boolean
  - `probe_error`: nullable string
  - `systemd_error`: nullable string
  - `systemd_socket`: nullable object
  - `systemd_service`: nullable object

#### macOS host report (`host.platform=="macos"`)
Required fields:
- `platform`: `"macos"`
- `ok`: boolean; must be `true` iff all of:
  - `world_enabled==true`, and
  - `lima.installed==true`, and
  - `lima.virtualization==true`, and
  - `lima.vm_status=="Running"`, and
  - `lima.service_active==true`, and
  - `lima.agent_caps_ok==true`
- `world_fs_mode`: string enum: `"writable"` or `"read_only"`
- `world_fs_isolation`: string enum: `"workspace"` or `"full"`
- `world_fs_require_world`: boolean
- `lima`: object with required fields:
  - `installed`: boolean
  - `virtualization`: boolean
  - `vm_status`: string (must be `"Running"` to be ok)
  - `service_active`: boolean
  - `agent_caps_ok`: boolean
  - `vsock_proxy`: boolean

#### Windows host report (`host.platform=="windows"`)
Required fields:
- `platform`: `"windows"`
- `ok`: boolean; must be `false`
- `status`: string enum: `"unsupported"`
- `message`: string; must be non-empty

## Text output contract (authoritative; minimal)
- Line prefixes are exactly: `PASS  | `, `WARN  | `, `FAIL  | `, `INFO  | `.
- `substrate host doctor`:
  - First line: `== substrate host doctor ==`
  - Then a deterministic, platform-specific set of lines (not required to be stable across versions; JSON is the stable interface).
- `substrate world doctor`:
  - First line: `== substrate world doctor ==`
  - Then:
    - `== Host ==` section (same probe set as `substrate host doctor`)
    - `== World ==` section (agent-reported world doctor)

## Required docs updates (must land in DS0)
- `docs/COMMANDS.md`: add `substrate host doctor`; update `substrate world doctor` notes to match new scope.
- `docs/WORLD.md`: update doctor framing and reference the new world-agent doctor endpoint behavior.
- `docs/USAGE.md`: update “World Commands” bullets to include the new command and revised semantics.
- `docs/INSTALLATION.md`: replace any “world doctor is host readiness report” language.
- `docs/ISOLATION_SUPPORT_MATRIX.md`: remove “incomplete” framing for doctor scopes and document scope split explicitly.

## Acceptance criteria (DS0 must satisfy all)
- CLI:
  - `substrate host doctor --json` emits `HostDoctorEnvelopeV1` exactly as specified.
  - `substrate world doctor --json` emits `WorldDoctorEnvelopeV1` exactly as specified.
  - `substrate world doctor` never reports `ok=true` unless the agent report `ok=true` and the host report `ok=true`.
  - Windows behavior is explicit: `ok=false`, `status=unsupported`, exit code `4`.
- Agent API:
  - `GET /v1/doctor/world` exists and returns `WorldDoctorReportV1` on success.
  - The endpoint reports Landlock support/ABI from the world kernel on macOS (Lima guest).
- Smoke:
  - Linux smoke script validates `host` and `world` JSON shapes and requires `landlock.supported==true`.
  - macOS smoke script validates `host` and `world` JSON shapes and requires `landlock.supported==true`.
- Tests:
  - Update/extend tests that parse world doctor JSON (health/shim snapshots, world verify) to use the new `host`/`world` blocks.
  - Add a schema round-trip test for `WorldDoctorReportV1` in `agent-api-types`.

## Out of scope
- Any “world disabled” UX overhaul beyond the explicit short-circuit required by this spec.
- Any redesign of `substrate health` output ordering or verbosity beyond updating it to parse the new doctor schema.
