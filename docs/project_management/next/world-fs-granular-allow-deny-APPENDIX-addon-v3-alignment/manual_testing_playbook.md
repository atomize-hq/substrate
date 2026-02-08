# Manual Testing Playbook — world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment

This playbook validates the add-on pack’s contract closures:
- Effective policy display output contract (Appendix A.6)
- Snapshot schema/protocol lockstep (PolicySnapshotV3; schema_version=3)
- Downstream operator-facing docs/surfaces alignment (no V2 key drift)

## Preconditions
- `substrate` is installed on PATH (or set `SUBSTRATE_BIN`).
- Recommended: run in a clean sandbox root to avoid touching your real `~/.substrate`:

```bash
export AXA_TEST_ROOT="$(mktemp -d)"
export SUBSTRATE_HOME="$AXA_TEST_ROOT/substrate-home"
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
mkdir -p "$AXA_TEST_ROOT/workspace"
cd "$AXA_TEST_ROOT/workspace"
echo "AXA_TEST_ROOT=$AXA_TEST_ROOT"
```

## Behavioral smoke script (Linux)
Run:
- `bash docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/smoke/linux-smoke.sh`

Expected:
- Exit `0`.

## Behavioral smoke script (macOS) (optional)
Run (macOS only):
- `bash docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/smoke/macos-smoke.sh`

Expected:
- Exit `0`.

## Case 1 — Effective policy display is V3-shaped (Appendix A.6)
Run:

```bash
"$SUBSTRATE_BIN" workspace init --force >/dev/null
"$SUBSTRATE_BIN" policy init --force >/dev/null

"$SUBSTRATE_BIN" policy set \
  'world_fs.host_visible=false' \
  'world_fs.fail_closed.routing=false' \
  'world_fs.write.enabled=true' \
  'world_fs.read.allow_list+=.' \
  >/dev/null

"$SUBSTRATE_BIN" policy show >policy.yaml
"$SUBSTRATE_BIN" policy show --json >policy.json
```

Expected:
- `policy.yaml` contains:
  - `world_fs:`
  - `host_visible: false`
  - `discover:` + `read:` + `write:`
  - and explicit empty deny lists (at least these must exist): `deny_list: []`
- `policy.yaml` does **not** contain legacy keys under `world_fs`:
  - `mode:`, `isolation:`, `require_world:`, `enforcement:`
- `policy.json` parses as JSON and includes:
  - `.world_fs.discover.deny_list == []`
  - `.world_fs.read.deny_list == []`
  - `.world_fs.write.deny_list == []`

## Case 2 — World-agent snapshot schema validation is V3-only
This is validated primarily via deterministic tests (recommended):

```bash
cargo test -p world-agent --tests -- --nocapture
```

Expected:
- Exit `0`.
- Test coverage includes rejecting legacy snapshot schema versions (`schema_version != 3`) for both HTTP and WS.

## CI parity (compile/test)
CI parity platforms: `linux,macos`.

Recommended gate:
- `make ci-compile-parity CI_WORKFLOW_REF="feat/world-fs-granular-allow-deny-appendix" CI_REMOTE=origin CI_CLEANUP=1`

## CI audit + evidence ledger (recommended)
Audit before dispatch:
- `scripts/ci-audit/ci_audit.sh --kind ci-testing --orch-branch "feat/world-fs-granular-allow-deny-appendix" --ledger-path "docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/logs/WFGADAXA2/ci-audit/ledger.jsonl"`
- `scripts/ci-audit/ci_audit.sh --kind feature-smoke --orch-branch "feat/world-fs-granular-allow-deny-appendix" --feature-dir "docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment" --ledger-path "docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/logs/WFGADAXA2/ci-audit/ledger.jsonl"`

Record after dispatch:
- `scripts/ci-audit/ci_audit_record.sh --ledger-path "docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/logs/WFGADAXA2/ci-audit/ledger.jsonl" --kind <ci-testing|feature-smoke> --orch-branch "feat/world-fs-granular-allow-deny-appendix" --run-id "<id>" --tested-sha "<sha>" --feature-dir "docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment"`
