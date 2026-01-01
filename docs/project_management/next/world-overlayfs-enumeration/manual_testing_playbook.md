# Manual Testing Playbook — World OverlayFS Enumeration Reliability (ADR-0004)

Goal: validate that world execution shows files via directory enumeration and that strategy selection is observable.

## Prerequisites
- Platform: Linux
- Required commands: `substrate`, `rg`, `jq`, `mktemp`
- Smoke script entrypoint: `bash docs/project_management/next/world-overlayfs-enumeration/smoke/linux-smoke.sh`

## Linux (required)

### 0) Run the smoke script
```bash
set -euo pipefail
bash docs/project_management/next/world-overlayfs-enumeration/smoke/linux-smoke.sh
```

Expected:
- Exit `0`.
- Output contains `OK: overlayfs enumeration smoke`.

### 1) Doctor reports strategy + probe keys
```bash
set -euo pipefail
substrate world doctor --json | jq -e '
  .world_fs_strategy_primary == "overlay" and
  .world_fs_strategy_fallback == "fuse" and
  .world_fs_strategy_probe.id == "enumeration_v1" and
  .world_fs_strategy_probe.probe_file == ".substrate_enum_probe" and
  (.world_fs_strategy_probe.result | IN("pass"; "fail"))
'
```

Expected:
- Exit `0`.

### 2) Enumeration correctness in world mode
```bash
set -euo pipefail
tmp="$(mktemp -d)"
cd "$tmp"
substrate --world -c 'touch a.txt; ls -a' | rg -n -- '^a\\.txt$' >/dev/null
```

Expected:
- Exit `0`.
- Output includes `a.txt`.

### 3) Host visibility is not assumed
```bash
set -euo pipefail
tmp="$(mktemp -d)"
cd "$tmp"
substrate --world -c 'touch a.txt; ls -a' >/dev/null
test ! -e "$tmp/a.txt"
```

Expected:
- Exit `0`.
- `a.txt` does not appear on host unless a sync mechanism is enabled (out of scope here).

### 4) Trace includes world filesystem strategy fields
```bash
set -euo pipefail
tmp="$(mktemp -d)"
trace="$tmp/trace.jsonl"

SHIM_TRACE_LOG="$trace" substrate --world -c 'touch a.txt; ls -a' >/dev/null

jq -e -s '
  ([.[] | select(.event_type == "command_complete")] | last) as $e
  | ($e.world_fs_strategy_primary | IN("overlay"; "fuse"))
  and ($e.world_fs_strategy_final | IN("overlay"; "fuse"; "host"))
  and ($e.world_fs_strategy_fallback_reason | IN(
    "none";
    "primary_unavailable";
    "primary_mount_failed";
    "primary_probe_failed";
    "fallback_unavailable";
    "fallback_mount_failed";
    "fallback_probe_failed";
    "world_optional_fallback_to_host"
  ))
' "$trace" >/dev/null
```

Expected:
- Exit `0`.

## macOS / Windows
- Out of scope for this ADR.

## Smoke scripts
- Linux: `docs/project_management/next/world-overlayfs-enumeration/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/world-overlayfs-enumeration/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/world-overlayfs-enumeration/smoke/windows-smoke.ps1`
