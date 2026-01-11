# Manual Testing Playbook — doctor_scopes (DS0)

Scope:
- Manual validation for the DS0 contract in `docs/project_management/_archived/doctor_scopes/DS0-spec.md`.
- This playbook is the human-readable equivalent of the smoke scripts in `docs/project_management/_archived/doctor_scopes/smoke/`.

Exit code taxonomy:
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Preconditions (all platforms)

- Build a `substrate` binary that includes the DS0 changes.
- Ensure world isolation is enabled for the current directory (effective config `world_enabled=true`), unless you are explicitly running the “world disabled” cases below.

## Linux (behavior platform)

### 1) Host doctor JSON contract

Commands:
- `substrate host doctor --json | jq -e '.schema_version==1 and .platform==\"linux\" and .world_enabled==true and .ok==true and .host.platform==\"linux\" and .host.ok==true'`

Expected:
- Exit code `0`.
- JSON includes `host.world_socket.socket_exists==true` and `host.world_socket.probe_ok==true`.

### 2) World doctor JSON contract (agent-reported world facts)

Commands:
- `substrate world doctor --json | jq -e '.schema_version==1 and .platform==\"linux\" and .world_enabled==true and .ok==true and .host.ok==true and .world.schema_version==1 and .world.ok==true and .world.landlock.supported==true and (.world.landlock.abi|type==\"number\") and .world.world_fs_strategy.probe.id==\"enumeration_v1\" and .world.world_fs_strategy.probe.result==\"pass\"'`

Expected:
- Exit code `0`.
- World block is present and indicates Landlock support/ABI and a passing enumeration probe.

### 3) World disabled short-circuit

Commands:
- `substrate --no-world world doctor --json | jq -e '.world_enabled==false and .ok==false and .world.status==\"disabled\" and .world.ok==false'`

Expected:
- Exit code `4`.
- No socket probing side effects; the report is a deterministic “disabled” status.

## macOS (behavior platform)

### 1) Host doctor JSON contract

Commands:
- `substrate host doctor --json | jq -e '.schema_version==1 and .platform==\"macos\" and .world_enabled==true and .ok==true and .host.platform==\"macos\" and .host.ok==true and .host.lima.installed==true and .host.lima.virtualization==true and .host.lima.vm_status==\"Running\" and .host.lima.service_active==true and .host.lima.agent_caps_ok==true'`

Expected:
- Exit code `0`.
- Host block reports Lima/Virtualization readiness and agent capability probe success.

### 2) World doctor JSON contract (guest-kernel facts via agent endpoint)

Commands:
- `substrate world doctor --json | jq -e '.schema_version==1 and .platform==\"macos\" and .world_enabled==true and .ok==true and .host.ok==true and .world.schema_version==1 and .world.ok==true and .world.landlock.supported==true and (.world.landlock.abi|type==\"number\") and .world.world_fs_strategy.probe.result==\"pass\"'`

Expected:
- Exit code `0`.
- World block includes Landlock support/ABI from the guest kernel and a passing enumeration probe.

### 3) World disabled short-circuit

Commands:
- `substrate --no-world world doctor --json | jq -e '.world_enabled==false and .ok==false and .world.status==\"disabled\" and .world.ok==false'`

Expected:
- Exit code `4`.

## Windows (CI parity platform only)

Windows is CI-parity-only for DS0. Behavior is intentionally explicit “unsupported” for the new doctor scopes.

### 1) Host doctor explicit unsupported

Commands:
- `substrate.exe host doctor --json | ConvertFrom-Json | % { if ($_.platform -ne \"windows\") { throw \"platform mismatch\" }; if ($_.ok -ne $false) { throw \"ok must be false\" }; if ($_.host.status -ne \"unsupported\") { throw \"expected unsupported\" } }`

Expected:
- Exit code `4`.

### 2) World doctor explicit unsupported

Commands:
- `$out = & substrate.exe world doctor --json; $code = $LASTEXITCODE; $obj = $out | ConvertFrom-Json; if ($obj.platform -ne \"windows\") { throw \"platform mismatch\" }; if ($obj.ok -ne $false) { throw \"ok must be false\" }; if ($obj.host.status -ne \"unsupported\") { throw \"expected host unsupported\" }; if ($obj.world.status -ne \"unsupported\") { throw \"expected world unsupported\" }; if ($code -ne 4) { throw \"expected exit code 4\" }`

Expected:
- Exit code `4`.

## Smoke scripts (required parity)

Smoke scripts must be runnable and must match the commands above (minimal subset):
- Linux: `docs/project_management/_archived/doctor_scopes/smoke/linux-smoke.sh`
- macOS: `docs/project_management/_archived/doctor_scopes/smoke/macos-smoke.sh`
- Windows: `docs/project_management/_archived/doctor_scopes/smoke/windows-smoke.ps1` (no-op; CI parity only)
