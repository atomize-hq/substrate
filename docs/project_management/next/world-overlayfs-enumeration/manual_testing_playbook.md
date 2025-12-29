# Manual Testing Playbook — World OverlayFS Enumeration Reliability (ADR-0004)

Goal: validate that world execution shows files via directory enumeration and that fallback behavior is observable.

## Linux (required)

### 1) Enumeration correctness in world mode
```bash
set -euo pipefail
tmp="$(mktemp -d)"
cd "$tmp"
substrate --world -c 'touch a.txt; ls -a' | rg -n -- '^a\\.txt$' >/dev/null
```

Expected:
- Exit `0`.
- Output includes `a.txt`.

### 2) Host visibility is not assumed
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

## macOS / Windows
- Out of scope for this ADR.

